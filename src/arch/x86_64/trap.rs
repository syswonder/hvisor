use crate::{
    arch::{
        cpu::ArchCpu,
        device::all_virt_devices,
        idt::IdtStruct,
        lapic::{local_apic, vectors::*},
        msr::Msr::{self, *},
        vmx::*,
    },
    device::irqchip::pic::lapic::VirtLocalApic,
    error::{HvError, HvResult},
};

core::arch::global_asm!(
    include_str!("trap.S"),
    sym arch_handle_trap
);

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

const VM_EXIT_INSTR_LEN_CPUID: u8 = 2;
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
        APIC_TIMER_VECTOR => {
            // println!("Timer");
            unsafe { local_apic().end_of_interrupt() };
        }
        _ => {
            println!("Unhandled irq {}", vector);
        }
    }
}

fn handle_cpuid(arch_cpu: &mut ArchCpu) -> HvResult {
    use raw_cpuid::{cpuid, CpuIdResult};

    const LEAF_FEATURE_INFO: u32 = 0x1;
    const LEAF_HYPERVISOR_INFO: u32 = 0x4000_0000;
    const LEAF_HYPERVISOR_FEATURE: u32 = 0x4000_0001;
    const VENDOR_STR: &[u8; 12] = b"HVISORHVISOR";
    let vendor_regs = unsafe { &*(VENDOR_STR.as_ptr() as *const [u32; 3]) };

    let regs = arch_cpu.regs_mut();
    let function = regs.rax as u32;
    let res = match function {
        LEAF_FEATURE_INFO => {
            const FEATURE_VMX: u32 = 1 << 5;
            const FEATURE_HYPERVISOR: u32 = 1 << 31;
            let mut res = cpuid!(regs.rax, regs.rcx);
            res.ecx &= !FEATURE_VMX;
            res.ecx |= FEATURE_HYPERVISOR;
            res
        }
        LEAF_HYPERVISOR_INFO => CpuIdResult {
            eax: LEAF_HYPERVISOR_FEATURE,
            ebx: vendor_regs[0],
            ecx: vendor_regs[1],
            edx: vendor_regs[2],
        },
        LEAF_HYPERVISOR_FEATURE => CpuIdResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        },
        _ => cpuid!(regs.rax, regs.rcx),
    };

    debug!(
        "VM exit: CPUID({:#x}, {:#x}): {:?}",
        regs.rax, regs.rcx, res
    );
    regs.rax = res.eax as _;
    regs.rbx = res.ebx as _;
    regs.rcx = res.ecx as _;
    regs.rdx = res.edx as _;

    advance_guest_rip(VM_EXIT_INSTR_LEN_CPUID)?;
    Ok(())
}

fn handle_external_interrupt() -> HvResult {
    let int_info = interrupt_exit_info()?;
    trace!("VM-exit: external interrupt: {:#x?}", int_info);
    assert!(int_info.valid);
    handle_irq(int_info.vector);
    Ok(())
}

fn handle_hypercall(arch_cpu: &ArchCpu) -> HvResult {
    let regs = arch_cpu.regs();
    debug!(
        "VM exit: VMCALL({:#x}): {:?}",
        regs.rax,
        [regs.rdi, regs.rsi, regs.rdx, regs.rcx]
    );
    advance_guest_rip(VM_EXIT_INSTR_LEN_VMCALL)?;
    Ok(())
}

fn handle_io_instruction(arch_cpu: &mut ArchCpu, exit_info: &VmxExitInfo) -> HvResult {
    let io_info = io_exit_info()?;
    trace!(
        "VM exit: I/O instruction @ {:#x}: {:#x?}",
        exit_info.guest_rip,
        io_info,
    );
    if io_info.is_string {
        error!("INS/OUTS instructions are not supported!");
        return hv_result_err!(ENOSYS);
    }
    if io_info.is_repeat {
        error!("REP prefixed I/O instructions are not supported!");
        return hv_result_err!(ENOSYS);
    }

    if let Some(dev) = all_virt_devices().find_port_io_device(io_info.port) {
        if io_info.is_in {
            let value = dev.read(io_info.port, io_info.access_size)?;
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
            dev.write(io_info.port, io_info.access_size, value)?;
        }
    } else {
        panic!(
            "Unsupported I/O port {:#x} access: {:#x?}",
            io_info.port, io_info
        )
    }
    advance_guest_rip(exit_info.exit_instruction_length as _)?;
    Ok(())
}

fn handle_msr_read(arch_cpu: &mut ArchCpu) -> HvResult {
    let rcx = arch_cpu.regs().rcx as u32;
    let msr = Msr::try_from(rcx).unwrap();

    let res = if msr == IA32_APIC_BASE {
        let mut apic_base = unsafe { IA32_APIC_BASE.read() };
        apic_base |= 1 << 11 | 1 << 10; // enable xAPIC and x2APIC
        Ok(apic_base)
    } else if VirtLocalApic::msr_range().contains(&rcx) {
        VirtLocalApic::rdmsr(arch_cpu, msr)
    } else {
        hv_result_err!(ENOSYS)
    };

    if let Ok(value) = res {
        debug!("VM exit: RDMSR({:#x}) -> {:#x}", rcx, value);
        arch_cpu.regs_mut().rax = value & 0xffff_ffff;
        arch_cpu.regs_mut().rdx = value >> 32;
    } else {
        panic!("Failed to handle RDMSR({:#x}): {:?}", rcx, res);
    }
    advance_guest_rip(VM_EXIT_INSTR_LEN_RDMSR)?;
    Ok(())
}

fn handle_msr_write(arch_cpu: &mut ArchCpu) -> HvResult {
    let rcx = arch_cpu.regs().rcx as u32;
    let msr = Msr::try_from(rcx).unwrap();
    let value = (arch_cpu.regs().rax & 0xffff_ffff) | (arch_cpu.regs().rdx << 32);
    debug!("VM exit: WRMSR({:#x}) <- {:#x}", rcx, value);

    let res = if msr == IA32_APIC_BASE {
        Ok(()) // ignore
    } else if VirtLocalApic::msr_range().contains(&rcx) {
        VirtLocalApic::wrmsr(arch_cpu, msr, value)
    } else {
        hv_result_err!(ENOSYS)
    };

    if res.is_err() {
        panic!(
            "Failed to handle WRMSR({:#x}) <- {:#x}: {:?}",
            rcx, value, res
        );
    }
    advance_guest_rip(VM_EXIT_INSTR_LEN_WRMSR)?;
    Ok(())
}

fn handle_s2pt_violation(arch_cpu: &mut ArchCpu, guest_rip: usize) -> HvResult {
    let fault_info = s2pt_violation_info()?;
    panic!(
        "VM exit: S2PT violation @ {:#x}, fault_paddr={:#x}, access_flags=({:?}), {:#x?}",
        guest_rip, fault_info.fault_guest_paddr, fault_info.access_flags, arch_cpu
    );
}

pub fn handle_vmexit(arch_cpu: &mut ArchCpu) -> HvResult {
    let exit_info = exit_info()?;
    debug!("VM exit: {:#x?}", exit_info);

    if exit_info.entry_failure {
        panic!("VM entry failed: {:#x?}", exit_info);
    }

    let res = match exit_info.exit_reason {
        VmxExitReason::CPUID => handle_cpuid(arch_cpu),
        VmxExitReason::EPT_VIOLATION => handle_s2pt_violation(arch_cpu, exit_info.guest_rip),
        VmxExitReason::EXTERNAL_INTERRUPT => handle_external_interrupt(),
        VmxExitReason::INTERRUPT_WINDOW => set_interrupt_window(false),
        VmxExitReason::IO_INSTRUCTION => handle_io_instruction(arch_cpu, &exit_info),
        VmxExitReason::MSR_READ => handle_msr_read(arch_cpu),
        VmxExitReason::MSR_WRITE => handle_msr_write(arch_cpu),
        VmxExitReason::VMCALL => handle_hypercall(arch_cpu),
        _ => panic!(
            "Unhandled VM-Exit reason {:?}:\n{:#x?}",
            exit_info.exit_reason, arch_cpu
        ),
    };

    if res.is_err() {
        panic!(
            "Failed to handle VM-exit {:?}:\n{:#x?}",
            exit_info.exit_reason, arch_cpu
        );
    }

    Ok(())
}
