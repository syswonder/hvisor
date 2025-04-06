use crate::{
    arch::{
        cpu::{this_cpu_id, ArchCpu},
        cpuid::{CpuIdEax, ExtendedFeaturesEcx, FeatureInfoFlags},
        hpet,
        idt::{get_guest_vector, get_host_vector, IdtStruct, IdtVector},
        ipi,
        msr::Msr::{self, *},
        s2pt::Stage2PageFaultInfo,
        vmcs::*,
        vmx::{VmxCrAccessInfo, VmxExitInfo, VmxExitReason, VmxInterruptInfo, VmxIoExitInfo},
    },
    device::{
        irqchip::{
            inject_vector,
            pic::{
                ioapic::{ioapic_inject_irq, irqs},
                lapic::VirtLocalApic,
            },
        },
        uart::UartReg,
    },
    error::HvResult,
    memory::{mmio_handle_access, MMIOAccess, MemFlags},
    percpu::this_cpu_data,
};
use x86_64::registers::control::Cr4Flags;

core::arch::global_asm!(
    include_str!("trap.S"),
    sym arch_handle_trap
);

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

const VM_EXIT_INSTR_LEN_CPUID: u8 = 2;
const VM_EXIT_INSTR_LEN_HLT: u8 = 1;
const VM_EXIT_INSTR_LEN_RDMSR: u8 = 2;
const VM_EXIT_INSTR_LEN_WRMSR: u8 = 2;
const VM_EXIT_INSTR_LEN_VMCALL: u8 = 3;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub usr: [u64; 15],

    // pushed by 'trap.S'
    pub vector: u64,
    pub error_code: u64,

    // pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

lazy_static::lazy_static! {
    static ref IDT: IdtStruct = IdtStruct::new();
}

pub fn install_trap_vector() {
    IDT.load();
}

#[no_mangle]
pub fn arch_handle_trap(tf: &mut TrapFrame) {
    // println!("trap {} @ {:#x}", tf.vector, tf.rip);
    match tf.vector as u8 {
        IRQ_VECTOR_START..=IRQ_VECTOR_END => handle_irq(tf.vector as u8),
        _ => {
            println!(
                "Unhandled exception {} (error_code = {:#x}) @ {:#x}",
                tf.vector, tf.error_code, tf.rip
            );
        }
    }
}

fn handle_irq(vector: u8) {
    match vector {
        IdtVector::VIRT_IPI_VECTOR => ipi::handle_virt_ipi(),
        IdtVector::APIC_TIMER_VECTOR => inject_vector(
            this_cpu_id(),
            this_cpu_data().arch_cpu.virt_lapic.virt_timer_vector,
            None,
            true,
        ),
        _ => {
            inject_vector(
                this_cpu_id(),
                vector as _,
                //get_guest_vector(vector).unwrap() as _,
                None,
                false,
            );
        }
    }
    unsafe { VirtLocalApic::phys_local_apic().end_of_interrupt() };
}

fn handle_cpuid(arch_cpu: &mut ArchCpu) -> HvResult {
    use raw_cpuid::{cpuid, CpuIdResult};
    // FIXME: temporary hypervisor hack
    let signature = unsafe { &*("ACRNACRNACRN".as_ptr() as *const [u32; 3]) };
    let cr4_flags = Cr4Flags::from_bits_truncate(arch_cpu.cr(4) as _);
    let regs = arch_cpu.regs_mut();
    let rax: Result<CpuIdEax, u32> = (regs.rax as u32).try_into();
    let mut res: CpuIdResult = cpuid!(regs.rax, regs.rcx);

    if let Ok(function) = rax {
        res = match function {
            CpuIdEax::FeatureInfo => {
                let mut res = cpuid!(regs.rax, regs.rcx);
                let mut ecx = FeatureInfoFlags::from_bits_truncate(res.ecx as _);

                ecx.remove(FeatureInfoFlags::VMX);
                // ecx.remove(FeatureInfoFlags::TSC_DEADLINE);
                ecx.remove(FeatureInfoFlags::XSAVE);

                ecx.insert(FeatureInfoFlags::X2APIC);
                ecx.insert(FeatureInfoFlags::HYPERVISOR);
                res.ecx = ecx.bits() as _;

                let mut edx = FeatureInfoFlags::from_bits_truncate((res.edx as u64) << 32);
                // edx.remove(FeatureInfoFlags::TSC);
                res.edx = (edx.bits() >> 32) as _;

                res
            }
            CpuIdEax::StructuredExtendedFeatureInfo => {
                let mut res = cpuid!(regs.rax, regs.rcx);
                let mut ecx = ExtendedFeaturesEcx::from_bits_truncate(res.ecx as _);
                ecx.remove(ExtendedFeaturesEcx::WAITPKG);
                res.ecx = ecx.bits() as _;

                res
            }
            CpuIdEax::ProcessorFrequencyInfo => {
                if let Some(freq_mhz) = hpet::get_tsc_freq_mhz() {
                    CpuIdResult {
                        eax: freq_mhz,
                        ebx: freq_mhz,
                        ecx: freq_mhz,
                        edx: 0,
                    }
                } else {
                    cpuid!(regs.rax, regs.rcx)
                }
            }
            CpuIdEax::HypervisorInfo => CpuIdResult {
                eax: CpuIdEax::HypervisorFeatures as u32,
                ebx: signature[0],
                ecx: signature[1],
                edx: signature[2],
            },
            CpuIdEax::HypervisorFeatures => CpuIdResult {
                eax: 0,
                ebx: 0,
                ecx: 0,
                edx: 0,
            },
            _ => cpuid!(regs.rax, regs.rcx),
        };
    }

    trace!(
        "VM exit: CPUID({:#x}, {:#x}): {:?}",
        regs.rax,
        regs.rcx,
        res
    );
    regs.rax = res.eax as _;
    regs.rbx = res.ebx as _;
    regs.rcx = res.ecx as _;
    regs.rdx = res.edx as _;

    arch_cpu.advance_guest_rip(VM_EXIT_INSTR_LEN_CPUID)?;
    Ok(())
}

fn handle_cr_access(arch_cpu: &mut ArchCpu) -> HvResult {
    let cr_access_info = VmxCrAccessInfo::new()?;
    panic!(
        "VM-exit: CR{} access:\n{:#x?}",
        cr_access_info.cr_n, arch_cpu
    );

    match cr_access_info.cr_n {
        0 => {}
        _ => {}
    }

    Ok(())
}

fn handle_external_interrupt() -> HvResult {
    let int_info = VmxInterruptInfo::new()?;
    trace!("VM-exit: external interrupt: {:#x?}", int_info);
    assert!(int_info.valid);
    handle_irq(int_info.vector);
    Ok(())
}

fn handle_hypercall(arch_cpu: &mut ArchCpu) -> HvResult {
    let regs = arch_cpu.regs();
    debug!(
        "VM exit: VMCALL({:#x}): {:?}",
        regs.rax,
        [regs.rdi, regs.rsi, regs.rdx, regs.rcx]
    );
    arch_cpu.advance_guest_rip(VM_EXIT_INSTR_LEN_VMCALL)?;
    Ok(())
}

fn handle_io_instruction(arch_cpu: &mut ArchCpu, exit_info: &VmxExitInfo) -> HvResult {
    let io_info = VmxIoExitInfo::new()?;

    /*info!(
        "VM exit: I/O instruction @ {:#x}: {:#x?}",
        exit_info.guest_rip, io_info,
    );*/

    if io_info.is_string {
        error!("INS/OUTS instructions are not supported!");
        return hv_result_err!(ENOSYS);
    }
    if io_info.is_repeat {
        error!("REP prefixed I/O instructions are not supported!");
        return hv_result_err!(ENOSYS);
    }

    /*if let Some(dev) = all_virt_devices().find_port_io_device(io_info.port) {
        if io_info.is_in {
            let value = dev.read(io_info.port, 0)?;
            let rax = &mut arch_cpu.regs_mut().rax;
            // SDM Vol. 1, Section 3.4.1.1:
            // * 32-bit operands generate a 32-bit result, zero-extended to a 64-bit result in the
            //   destination general-purpose register.
            // * 8-bit and 16-bit operands generate an 8-bit or 16-bit result. The upper 56 bits or
            //   48 bits (respectively) of the destination general-purpose register are not modified
            //   by the operation.
            match io_info.access_size {
                1 => *rax = (*rax & !0xff) | (value & 0xff) as u64,
                2 => *rax = (*rax & !0xffff) | (value & 0xffff) as u64,
                4 => *rax = value as u64,
                _ => unreachable!(),
            }
        } else {
            let rax = arch_cpu.regs().rax;
            let value = match io_info.access_size {
                1 => rax & 0xff,
                2 => rax & 0xffff,
                4 => rax,
                _ => unreachable!(),
            } as u32;
            dev.write(io_info.port, value, 0)?;
        }
    } else {
        debug!(
            "Unsupported I/O port {:#x} access: {:#x?} \n {:#x?}",
            io_info.port, io_info, arch_cpu
        )
    }*/

    arch_cpu.advance_guest_rip(exit_info.exit_instruction_length as _)?;
    Ok(())
}

fn handle_msr_read(arch_cpu: &mut ArchCpu) -> HvResult {
    let rcx = arch_cpu.regs().rcx as u32;

    if let Ok(msr) = Msr::try_from(rcx) {
        let res = if msr == IA32_APIC_BASE {
            // FIXME: non root linux
            let mut apic_base = unsafe { IA32_APIC_BASE.read() };
            // info!("APIC BASE: {:x}", apic_base);
            apic_base |= 1 << 11 | 1 << 10; // enable xAPIC and x2APIC
            Ok(apic_base)
        } else if VirtLocalApic::msr_range().contains(&rcx) {
            arch_cpu.virt_lapic.rdmsr(msr)
        } else {
            hv_result_err!(ENOSYS)
        };

        if let Ok(value) = res {
            debug!("VM exit: RDMSR({:#x}) -> {:#x}", rcx, value);
            arch_cpu.regs_mut().rax = value & 0xffff_ffff;
            arch_cpu.regs_mut().rdx = value >> 32;
        } else {
            warn!("Failed to handle RDMSR({:#x}): {:?}", rcx, res);
        }
    } else {
        warn!("Unrecognized RDMSR({:#x})", rcx);
    }

    arch_cpu.advance_guest_rip(VM_EXIT_INSTR_LEN_RDMSR)?;
    Ok(())
}

fn handle_msr_write(arch_cpu: &mut ArchCpu) -> HvResult {
    let rcx = arch_cpu.regs().rcx as u32;
    let msr = Msr::try_from(rcx).unwrap();
    let value = (arch_cpu.regs().rax & 0xffff_ffff) | (arch_cpu.regs().rdx << 32);
    debug!("VM exit: WRMSR({:#x}) <- {:#x}", rcx, value);

    let res = if msr == IA32_APIC_BASE {
        Ok(()) // ignore
    } else if VirtLocalApic::msr_range().contains(&rcx) || msr == IA32_TSC_DEADLINE {
        arch_cpu.virt_lapic.wrmsr(msr, value)
    } else {
        hv_result_err!(ENOSYS)
    };

    if res.is_err() {
        warn!(
            "Failed to handle WRMSR({:#x}) <- {:#x}: {:?}\n{:#x?}",
            rcx, value, res, arch_cpu
        );
    }
    arch_cpu.advance_guest_rip(VM_EXIT_INSTR_LEN_WRMSR)?;
    Ok(())
}

fn handle_s2pt_violation(arch_cpu: &mut ArchCpu, exit_info: &VmxExitInfo) -> HvResult {
    let fault_info = Stage2PageFaultInfo::new()?;
    mmio_handle_access(&mut MMIOAccess {
        address: fault_info.fault_guest_paddr,
        size: 0,
        is_write: fault_info.access_flags.contains(MemFlags::WRITE),
        value: 0,
    })?;

    Ok(())
}

fn handle_triple_fault(arch_cpu: &mut ArchCpu, exit_info: &VmxExitInfo) -> HvResult {
    panic!(
        "VM exit: Triple fault @ {:#x}, instr length: {:x}",
        exit_info.guest_rip, exit_info.exit_instruction_length
    );
    // arch_cpu.advance_guest_rip(exit_info.exit_instruction_length as _)?;
    Ok(())
}

pub fn handle_vmexit(arch_cpu: &mut ArchCpu) -> HvResult {
    let exit_info = VmxExitInfo::new()?;
    debug!("VM exit: {:#x?}", exit_info);

    if exit_info.entry_failure {
        panic!("VM entry failed: {:#x?}", exit_info);
    }

    let res = match exit_info.exit_reason {
        VmxExitReason::EXTERNAL_INTERRUPT => handle_external_interrupt(),
        VmxExitReason::TRIPLE_FAULT => handle_triple_fault(arch_cpu, &exit_info),
        VmxExitReason::INTERRUPT_WINDOW => Vmcs::set_interrupt_window(false),
        VmxExitReason::CPUID => handle_cpuid(arch_cpu),
        VmxExitReason::HLT => {
            arch_cpu.advance_guest_rip(VM_EXIT_INSTR_LEN_HLT)?;
            Ok(())
        }
        VmxExitReason::VMCALL => handle_hypercall(arch_cpu),
        VmxExitReason::CR_ACCESS => handle_cr_access(arch_cpu),
        VmxExitReason::IO_INSTRUCTION => handle_io_instruction(arch_cpu, &exit_info),
        VmxExitReason::MSR_READ => handle_msr_read(arch_cpu),
        VmxExitReason::MSR_WRITE => handle_msr_write(arch_cpu),
        VmxExitReason::EPT_VIOLATION => handle_s2pt_violation(arch_cpu, &exit_info),
        _ => panic!(
            "Unhandled VM-Exit reason {:?}:\n{:#x?}",
            exit_info.exit_reason, arch_cpu
        ),
    };

    if res.is_err() {
        panic!(
            "Failed to handle VM-exit {:?}:\n{:#x?}\n{:?}",
            exit_info.exit_reason,
            arch_cpu,
            res.err()
        );
    }

    Ok(())
}
