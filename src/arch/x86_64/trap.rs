// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{
        cpu::{this_cpu_id, ArchCpu},
        cpuid::{CpuIdEax, ExtendedFeaturesEcx, FeatureInfoFlags},
        hpet,
        idt::{IdtStruct, IdtVector},
        ipi,
        msr::Msr::{self, *},
        s2pt::Stage2PageFaultInfo,
        vmcs::*,
        vmx::{
            VmxCrAccessInfo, VmxExitInfo, VmxExitReason, VmxInstructionError, VmxInterruptInfo,
            VmxIoExitInfo,
        },
    },
    cpu_data::this_cpu_data,
    device::{
        irqchip::{
            inject_vector,
            pic::{ioapic::irqs, lapic::VirtLocalApic},
        },
        uart::{virt_console_io_read, virt_console_io_write, UartReg},
    },
    error::HvResult,
    hypercall::HyperCall,
    memory::{mmio_handle_access, MMIOAccess, MemFlags},
};
use bit_field::BitField;
use core::mem::size_of;
use x86_64::registers::control::Cr4Flags;

use super::{
    pci::{handle_pci_config_port_read, handle_pci_config_port_write},
    pio::{PCI_CONFIG_ADDR_PORT, PCI_CONFIG_DATA_PORT, UART_COM1_PORT},
};

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
        IdtVector::VIRT_IPI_VECTOR => {
            ipi::handle_virt_ipi();
        }
        IdtVector::APIC_SPURIOUS_VECTOR | IdtVector::APIC_ERROR_VECTOR => {}
        _ => {
            if vector >= 0x20 && this_cpu_data().arch_cpu.power_on {
                inject_vector(this_cpu_id(), vector, None, false);
            }
        }
    }
    unsafe { VirtLocalApic::phys_local_apic().end_of_interrupt() };
}

fn handle_cpuid(arch_cpu: &mut ArchCpu) -> HvResult {
    use raw_cpuid::{cpuid, CpuIdResult};
    // TODO: temporary hypervisor hack
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
            CpuIdEax::TscInfo => CpuIdResult {
                eax: 1,
                ebx: 1,
                ecx: hpet::get_tsc_freq_mhz().unwrap_or(0) * 1_000_000,
                edx: 0,
            },
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
    let regs = arch_cpu.regs_mut();
    debug!(
        "VM exit: VMCALL({:#x}): {:x?}",
        regs.rax,
        [regs.rdi, regs.rsi]
    );
    let (code, arg0, arg1) = (regs.rax, regs.rdi, regs.rsi);
    let cpu_data = this_cpu_data();
    let result = match HyperCall::new(cpu_data).hypercall(code as _, arg0, arg1) {
        Ok(ret) => ret as _,
        Err(e) => {
            error!("hypercall error: {:#?}", e);
            e.code()
        }
    };
    debug!("HVC result = {}", result);
    regs.rax = result as _;

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

    let mut value: u32 = 0;
    if !io_info.is_in {
        let rax = arch_cpu.regs().rax;
        value = match io_info.access_size {
            1 => rax & 0xff,
            2 => rax & 0xffff,
            4 => rax,
            _ => unreachable!(),
        } as _;

        // TODO: reconstruct
        if PCI_CONFIG_ADDR_PORT.contains(&io_info.port)
            || PCI_CONFIG_DATA_PORT.contains(&io_info.port)
        {
            handle_pci_config_port_write(&io_info, value);
        } else if UART_COM1_PORT.contains(&io_info.port) {
            virt_console_io_write(io_info.port, value);
        } else {
            /* info!(
                "unhandled port io write {:x} value: {:x}",
                io_info.port, value
            ); */
        }
    } else {
        if PCI_CONFIG_ADDR_PORT.contains(&io_info.port)
            || PCI_CONFIG_DATA_PORT.contains(&io_info.port)
        {
            value = handle_pci_config_port_read(&io_info);
        } else if UART_COM1_PORT.contains(&io_info.port) {
            value = virt_console_io_read(io_info.port);
        } else {
            // info!("unhandled port io read {:x}", io_info.port);
            value = 0x0;
        }
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
    }

    arch_cpu.advance_guest_rip(exit_info.exit_instruction_length as _)?;
    Ok(())
}

fn handle_msr_read(arch_cpu: &mut ArchCpu) -> HvResult {
    let rcx = arch_cpu.regs().rcx as u32;

    let res: HvResult<u64> = if rcx == IA32_APIC_BASE as u32 {
        let mut apic_base = unsafe { IA32_APIC_BASE.read() };
        apic_base |= 1 << 11 | 1 << 10; // report xAPIC and x2APIC enabled
        Ok(apic_base)
    } else if VirtLocalApic::msr_range().contains(&rcx) {
        if let Ok(msr) = Msr::try_from(rcx) {
            arch_cpu.virt_lapic.rdmsr(msr)
        } else {
            // MSR not in our enum but in x2APIC range — return 0 (safe default).
            Ok(0)
        }
    } else if let Ok(msr) = Msr::try_from(rcx) {
        if msr == IA32_GS_BASE {
            VmcsGuestNW::GS_BASE
                .read()
                .map(|v| v as u64)
                .map_err(|_| hv_err!(EIO))
        } else if msr == IA32_FS_BASE {
            VmcsGuestNW::FS_BASE
                .read()
                .map(|v| v as u64)
                .map_err(|_| hv_err!(EIO))
        } else {
            hv_result_err!(ENOSYS)
        }
    } else {
        hv_result_err!(ENOSYS)
    };

    match res {
        Ok(value) => {
            debug!("VM exit: RDMSR({:#x}) -> {:#x}", rcx, value);
            arch_cpu.regs_mut().rax = value & 0xffff_ffff;
            arch_cpu.regs_mut().rdx = value >> 32;
        }
        Err(e) => {
            warn!("Failed to handle RDMSR({:#x}): {:?}", rcx, e);
        }
    }

    arch_cpu.advance_guest_rip(VM_EXIT_INSTR_LEN_RDMSR)?;
    Ok(())
}

fn handle_msr_write(arch_cpu: &mut ArchCpu) -> HvResult {
    let rcx = arch_cpu.regs().rcx as u32;
    let value = (arch_cpu.regs().rax & 0xffff_ffff) | (arch_cpu.regs().rdx << 32);
    debug!("VM exit: WRMSR({:#x}) <- {:#x}", rcx, value);

    let msr_opt = Msr::try_from(rcx).ok();

    let res: HvResult<()> = if rcx == IA32_APIC_BASE as u32 {
        Ok(()) // ignore — guest can't change APIC mode
    } else if VirtLocalApic::msr_range().contains(&rcx) {
        if let Some(msr) = msr_opt {
            arch_cpu.virt_lapic.wrmsr(msr, value)
        } else {
            // x2APIC MSR not in our enum (e.g. SELF_IPI at 0x83F).
            // Silently ignore — the guest thinks it succeeded.
            Ok(())
        }
    } else if msr_opt == Some(IA32_TSC_DEADLINE) {
        arch_cpu.virt_lapic.wrmsr(IA32_TSC_DEADLINE, value)
    } else if msr_opt == Some(IA32_GS_BASE) {
        VmcsGuestNW::GS_BASE
            .write(value as usize)
            .map_err(|_| hv_err!(EIO))
    } else if msr_opt == Some(IA32_FS_BASE) {
        VmcsGuestNW::FS_BASE
            .write(value as usize)
            .map_err(|_| hv_err!(EIO))
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
    // Print more details for debugging
    info!("Triple fault details:");
    info!(
        "  RIP={:#x}, RSP={:#x}",
        exit_info.guest_rip,
        VmcsGuestNW::RSP.read().unwrap_or(0)
    );
    info!(
        "  RAX={:#x}, RBX={:#x}, RCX={:#x}, RDX={:#x}",
        arch_cpu.regs().rax,
        arch_cpu.regs().rbx,
        arch_cpu.regs().rcx,
        arch_cpu.regs().rdx
    );
    info!(
        "  RSI={:#x}, RDI={:#x}",
        arch_cpu.regs().rsi,
        arch_cpu.regs().rdi
    );
    info!(
        "  CR0={:#x}, CR3={:#x}, CR4={:#x}",
        VmcsGuestNW::CR0.read().unwrap_or(0),
        VmcsGuestNW::CR3.read().unwrap_or(0),
        VmcsGuestNW::CR4.read().unwrap_or(0)
    );
    info!(
        "  CS selector={:#x}, GDTR base={:#x}",
        VmcsGuest16::CS_SELECTOR.read().unwrap_or(0),
        VmcsGuestNW::GDTR_BASE.read().unwrap_or(0)
    );

    panic!(
        "VM exit: Triple fault @ {:#x}, instr length: {:x}\n {:#x?}",
        exit_info.guest_rip, exit_info.exit_instruction_length, arch_cpu
    );
}

pub fn handle_vmexit(arch_cpu: &mut ArchCpu) -> HvResult {
    let exit_info = VmxExitInfo::new()?;
    debug!("VM exit: {:#x?}", exit_info);

    if exit_info.entry_failure {
        // Get VM instruction error for more details
        let vm_instr_error =
            Vmcs::instruction_error().unwrap_or_else(|_| VmxInstructionError::from(0xFF));

        // Read all guest state for debugging
        let cr0 = VmcsGuestNW::CR0.read().unwrap_or(0);
        let cr3 = VmcsGuestNW::CR3.read().unwrap_or(0);
        let cr4 = VmcsGuestNW::CR4.read().unwrap_or(0);
        let cs_ar = VmcsGuest32::CS_ACCESS_RIGHTS.read().unwrap_or(0);
        let ss_ar = VmcsGuest32::SS_ACCESS_RIGHTS.read().unwrap_or(0);
        let tr_ar = VmcsGuest32::TR_ACCESS_RIGHTS.read().unwrap_or(0);
        let tr_limit = VmcsGuest32::TR_LIMIT.read().unwrap_or(0);
        let gdtr_base = VmcsGuestNW::GDTR_BASE.read().unwrap_or(0);
        let idtr_base = VmcsGuestNW::IDTR_BASE.read().unwrap_or(0);

        error!("VM entry failed!");
        error!(
            "  VM-instruction error: {:?} ({})",
            vm_instr_error,
            vm_instr_error.as_str()
        );
        error!("  Guest state:");
        error!("    CR0={:#x}, CR3={:#x}, CR4={:#x}", cr0, cr3, cr4);
        error!(
            "    CS_AR={:#x}, SS_AR={:#x}, TR_AR={:#x}, TR_LIMIT={:#x}",
            cs_ar, ss_ar, tr_ar, tr_limit
        );
        error!("    GDTR_BASE={:#x}, IDTR_BASE={:#x}", gdtr_base, idtr_base);

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
