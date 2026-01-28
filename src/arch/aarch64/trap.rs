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
//
use aarch64_cpu::{asm::wfi, registers::*};
use core::arch::global_asm;

use super::cpu::GeneralRegisters;
use crate::arch::sysreg::smc_call;
use crate::zone::zone_error;
use crate::{
    arch::{
        cpu::mpidr_to_cpuid,
        sysreg::{read_sysreg, write_sysreg},
    },
    cpu_data::{get_cpu_data, this_cpu_data, this_zone},
    device::irqchip::gic_handle_irq,
    event::{send_event, IPI_EVENT_SHUTDOWN, IPI_EVENT_WAKEUP},
    hypercall::{HyperCall, SGI_IPI_ID},
    memory::{mmio_handle_access, MMIOAccess},
    zone::{is_this_root_zone, remove_zone},
};

global_asm!(
    include_str!("./trap.S"),
    sym arch_handle_exit
);

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod ExceptionType {
    pub const EXIT_REASON_EL2_ABORT: u64 = 0x0;
    pub const EXIT_REASON_EL2_IRQ: u64 = 0x1;
    pub const EXIT_REASON_EL1_ABORT: u64 = 0x2;
    pub const EXIT_REASON_EL1_IRQ: u64 = 0x3;
    pub const EXIT_REASON_EL1_AARCH32_ABORT: u64 = 0x4;
    pub const EXIT_REASON_EL1_AARCH32_IRQ: u64 = 0x5;
}
const SMC_TYPE_MASK: u64 = 0x3F000000;
#[allow(non_snake_case)]
pub mod SmcType {
    pub const ARCH_SC: u64 = 0x0;
    pub const SIP_SC: u64 = 0x02000000;
    pub const STANDARD_SC: u64 = 0x04000000;
    pub const TOS_SC_START: u64 = 0x32000000;
    pub const TOS_SC_END: u64 = 0x3F000000;
}

const PSCI_VERSION_1_1: u64 = 0x10001;
const PSCI_TOS_NOT_PRESENT_MP: u64 = 2;
const ARM_SMCCC_VERSION_1_1: u64 = 0x10001;

#[allow(unused)]
const ARM_SMCCC_NOT_SUPPORTED: i64 = -1;

extern "C" {
    fn _hyp_trap_vector();
}

pub fn install_trap_vector() {
    // Set the trap vector.
    VBAR_EL2.set(_hyp_trap_vector as _);
}

// ----------------------------------------------

#[allow(non_snake_case)]
pub mod PsciFnId {
    pub const PSCI_VERSION: u64 = 0x84000000;
    pub const PSCI_CPU_SUSPEND_32: u64 = 0x84000001;
    pub const PSCI_CPU_OFF_32: u64 = 0x84000002;
    pub const PSCI_CPU_ON_32: u64 = 0x84000003;
    pub const PSCI_AFFINITY_INFO_32: u64 = 0x84000004;
    pub const PSCI_MIG_INFO_TYPE: u64 = 0x84000006;
    pub const PSCI_SYSTEM_OFF: u64 = 0x84000008;
    pub const PSCI_FEATURES: u64 = 0x8400000a;

    pub const PSCI_CPU_SUSPEND_64: u64 = 0xc4000001;
    pub const PSCI_CPU_OFF_64: u64 = 0xc4000002;
    pub const PSCI_CPU_ON_64: u64 = 0xc4000003;
    pub const PSCI_AFFINITY_INFO_64: u64 = 0xc4000004;
}
#[allow(non_snake_case)]
pub mod SMCccFnId {
    pub const SMCCC_VERSION: u64 = 0x80000000;
    pub const SMCCC_ARCH_FEATURES: u64 = 0x80000001;
}

#[allow(unused)]
pub enum TrapReturn {
    TrapHandled = 1,
    TrapUnhandled = 0,
    TrapForbidden = -1,
}

/*From hyp_vec->handle_vmexit x0:guest regs x1:exit_reason sp =stack_top-32*8*/
pub fn arch_handle_exit(regs: &mut GeneralRegisters) -> ! {
    let mpidr = MPIDR_EL1.get();
    let _cpu_id = mpidr_to_cpuid(mpidr);
    trace!("cpu exit, exit_reson:{:#x?}", regs.exit_reason);
    match regs.exit_reason as u64 {
        ExceptionType::EXIT_REASON_EL1_IRQ | ExceptionType::EXIT_REASON_EL1_AARCH32_IRQ => {
            irqchip_handle_irq1()
        }
        ExceptionType::EXIT_REASON_EL1_ABORT | ExceptionType::EXIT_REASON_EL1_AARCH32_ABORT => {
            arch_handle_trap_el1(regs)
        }
        ExceptionType::EXIT_REASON_EL2_ABORT => arch_handle_trap_el2(regs),
        ExceptionType::EXIT_REASON_EL2_IRQ => irqchip_handle_irq2(),
        _ => arch_dump_exit(regs.exit_reason),
    }
    unsafe { vmreturn(regs as *const _ as usize) }
}

fn irqchip_handle_irq1() {
    trace!("irq from el1");
    gic_handle_irq();
}

fn irqchip_handle_irq2() {
    error!("irq not handle from el2");
    loop {}
}

fn arch_handle_trap_el1(regs: &mut GeneralRegisters) {
    let mut _ret = TrapReturn::TrapUnhandled;

    trace!(
        "arch_handle_trap ec={:#x?} elr={:#x?}",
        ESR_EL2.read(ESR_EL2::EC),
        ESR_EL2.read(ESR_EL2::ISS)
    );

    match ESR_EL2.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::HVC64) => handle_hvc(regs),
        Some(ESR_EL2::EC::Value::SMC64) => handle_smc(regs),
        Some(ESR_EL2::EC::Value::TrappedMsrMrs) => handle_sysreg(regs),
        Some(ESR_EL2::EC::Value::DataAbortLowerEL) => handle_dabt(regs),
        Some(ESR_EL2::EC::Value::InstrAbortLowerEL) => handle_iabt(regs),
        _ => {
            error!(
                "Unsupported Exception EC:{:#x?}!",
                ESR_EL2.read(ESR_EL2::EC)
            );
            error!("esr_el2: iss {:#x?}", ESR_EL2.read(ESR_EL2::ISS));
            loop {}
            // ret = TrapReturn::TrapUnhandled;
        }
    }
}

fn arch_handle_trap_el2(_regs: &mut GeneralRegisters) {
    let elr = ELR_EL2.get();
    let esr = ESR_EL2.get();
    let far = FAR_EL2.get();
    match ESR_EL2.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::HVC64) => {
            println!("EL2 Exception: HVC64 call, ELR_EL2: {:#x?}", ELR_EL2.get());
        }
        Some(ESR_EL2::EC::Value::SMC64) => {
            println!("EL2 Exception: SMC64 call, ELR_EL2: {:#x?}", ELR_EL2.get());
        }
        Some(ESR_EL2::EC::Value::DataAbortCurrentEL) => {
            println!(
                "EL2 Exception: Data Abort, ELR_EL2: {:#x?}, ESR_EL2: {:#x?}, FAR_EL2: {:#x?}",
                elr, esr, far
            );
            loop {}
        }
        Some(ESR_EL2::EC::Value::InstrAbortCurrentEL) => {
            println!(
                "EL2 Exception: Instruction Abort, ELR_EL2: {:#x?}, ESR_EL2: {:#x?},FAR_EL2: {:#x?}",
                elr, esr, far
            );
        }
        _ => {
            println!(
                "Unhandled EL2 Exception: EC={:#x?}",
                ESR_EL2.read(ESR_EL2::EC)
            );
        }
    }
    loop {}
}

fn handle_iabt(_regs: &mut GeneralRegisters) {
    let iss = ESR_EL2.read(ESR_EL2::ISS);
    let op = iss >> 6 & 0x1;
    let hpfar = read_sysreg!(HPFAR_EL2);
    let far = read_sysreg!(FAR_EL2);
    let address = (far & 0xfff) | (hpfar << 8);
    error!(
        "Failed to fetch instruction (op={}) at {:#x?}, ELR_EL2={:#x?}!",
        op,
        address,
        ELR_EL2.get()
    );
    loop {}
    // TODO: finish iabt handle
    // arch_skip_instruction(frame);
}

fn handle_dabt(regs: &mut GeneralRegisters) {
    let iss = ESR_EL2.read(ESR_EL2::ISS);
    let is_write = (iss >> 6 & 0x1) != 0;
    let srt = iss >> 16 & 0x1f;
    let sse = (iss >> 21 & 0x1) != 0;
    let sas = iss >> 22 & 0x3;

    let size = 1 << sas;
    let hpfar = read_sysreg!(HPFAR_EL2);
    let far = read_sysreg!(FAR_EL2);
    let address = (far & 0xfff) | (hpfar << 8);

    let mut mmio_access = MMIOAccess {
        address: address as _,
        size,
        is_write,
        value: if is_write && srt != 31 {
            regs.usr[srt as usize] as _
        } else {
            0
        },
    };

    trace!("handle_dabt: {:#x?}", mmio_access);

    match mmio_handle_access(&mut mmio_access) {
        Ok(_) => {
            if !is_write && srt != 31 {
                if sse {
                    mmio_access.value =
                        ((mmio_access.value << (32 - 8 * size)) as i32) as usize >> (32 - 8 * size);
                }
                regs.usr[srt as usize] = mmio_access.value as _;
            }
        }
        Err(e) => {
            error!("mmio_handle_access: {:#x?}", e);
            zone_error();
        }
    }
    //TODO finish dabt handle
    arch_skip_instruction(regs);
}

fn handle_sysreg(regs: &mut GeneralRegisters) {
    //TODO check sysreg type
    //send sgi
    trace!("esr_el2: iss {:#x?}", ESR_EL2.read(ESR_EL2::ISS));
    let rt = (ESR_EL2.get() >> 5) & 0x1f;
    let val = regs.usr[rt as usize];
    trace!("esr_el2 rt{}: {:#x?}", rt, val);
    let sgi_id: u64 = (val & (0xf << 24)) >> 24;
    if !this_cpu_data().arch_cpu.power_on {
        warn!("skip send sgi {:#x?}", sgi_id);
    } else {
        trace!("send sgi {:#x?}", sgi_id);
        write_sysreg!(icc_sgi1r_el1, val);
    }

    arch_skip_instruction(regs); //skip sgi write
}

fn handle_hvc(regs: &mut GeneralRegisters) {
    /*
    if ESR_EL2.read(ESR_EL2::ISS) != 0x4a48 {
        return;
    }
    */
    let (code, arg0, arg1) = (regs.usr[0], regs.usr[1], regs.usr[2]);
    let cpu_data = this_cpu_data();

    trace!(
        "HVC from CPU{},code:{:#x?},arg0:{:#x?},arg1:{:#x?}",
        cpu_data.id,
        code,
        arg0,
        arg1
    );
    let result = match HyperCall::new(cpu_data).hypercall(code as _, arg0, arg1) {
        Ok(ret) => ret as _,
        Err(e) => {
            error!("hypercall error: {:#?}", e);
            e.code()
        }
    };
    debug!("HVC result = {}", result);
    regs.usr[0] = result as _;
}

fn handle_smc(regs: &mut GeneralRegisters) {
    let (code, arg0, arg1, arg2) = (regs.usr[0], regs.usr[1], regs.usr[2], regs.usr[3]);
    //info!(
    //    "SMC from CPU{}, func_id:{:#x?}, arg0:{:#x?}, arg1:{:#x?}, arg2:{:#x?}",
    //    cpu_data.id, code, arg0, arg1, arg2
    //);
    let result = match code & SMC_TYPE_MASK {
        SmcType::ARCH_SC => handle_arch_smc(regs, code, arg0, arg1, arg2),
        SmcType::STANDARD_SC => handle_psci_smc(regs, code, arg0, arg1, arg2),
        SmcType::TOS_SC_START..=SmcType::TOS_SC_END | SmcType::SIP_SC => {
            let ret = smc_call(code, &regs.usr[1..18]);
            regs.usr[0] = ret[0];
            regs.usr[1] = ret[1];
            regs.usr[2] = ret[2];
            regs.usr[3] = ret[3];
            ret[0]
        }
        _ => {
            warn!("unsupported smc {:#x?}", code);
            0
        }
    };
    regs.usr[0] = result;

    arch_skip_instruction(regs); //skip the smc ins
}

fn psci_emulate_features_info(code: u64) -> u64 {
    match code {
        PsciFnId::PSCI_VERSION
        | PsciFnId::PSCI_CPU_SUSPEND_32
        | PsciFnId::PSCI_CPU_SUSPEND_64
        | PsciFnId::PSCI_CPU_OFF_32
        | PsciFnId::PSCI_CPU_ON_32
        | PsciFnId::PSCI_CPU_ON_64
        | PsciFnId::PSCI_AFFINITY_INFO_32
        | PsciFnId::PSCI_AFFINITY_INFO_64
        | PsciFnId::PSCI_FEATURES
        | SMCccFnId::SMCCC_VERSION => 0,
        _ => !0,
    }
}

fn psci_emulate_cpu_on(regs: &mut GeneralRegisters) -> u64 {
    // Todo: Check if `cpu` is in the cpuset of current zone
    let cpu = mpidr_to_cpuid(regs.usr[1]);
    info!("psci: try to wake up cpu {}", cpu);

    let target_data = get_cpu_data(cpu as _);
    let _lock = target_data.ctrl_lock.lock();

    if !target_data.arch_cpu.power_on {
        target_data.cpu_on_entry = regs.usr[2] as _;
        target_data.arch_cpu.power_on = true;
        send_event(cpu as _, SGI_IPI_ID as _, IPI_EVENT_WAKEUP);
    } else {
        error!("psci: cpu {} already on", cpu);
        return u64::MAX - 3;
    };
    drop(_lock);

    0
}

fn handle_psci_smc(
    regs: &mut GeneralRegisters,
    code: u64,
    arg0: u64,
    _arg1: u64,
    _arg2: u64,
) -> u64 {
    match code {
        PsciFnId::PSCI_VERSION => PSCI_VERSION_1_1,
        PsciFnId::PSCI_CPU_SUSPEND_32 | PsciFnId::PSCI_CPU_SUSPEND_64 => {
            wfi();
            gic_handle_irq();
            0
        }
        PsciFnId::PSCI_CPU_OFF_32 | PsciFnId::PSCI_CPU_OFF_64 => {
            todo!();
        }
        PsciFnId::PSCI_AFFINITY_INFO_32 | PsciFnId::PSCI_AFFINITY_INFO_64 => {
            !get_cpu_data(arg0 as _).arch_cpu.power_on as _
        }
        PsciFnId::PSCI_MIG_INFO_TYPE => PSCI_TOS_NOT_PRESENT_MP,
        PsciFnId::PSCI_FEATURES => psci_emulate_features_info(regs.usr[1]),
        PsciFnId::PSCI_CPU_ON_32 | PsciFnId::PSCI_CPU_ON_64 => psci_emulate_cpu_on(regs),
        PsciFnId::PSCI_SYSTEM_OFF => {
            let zone = this_zone();
            let zone_id = zone.read().id;
            let is_root = is_this_root_zone();

            for cpu_id in zone.read().cpu_set.iter_except(this_cpu_data().id) {
                let target_cpu = get_cpu_data(cpu_id);
                let _lock = target_cpu.ctrl_lock.lock();
                target_cpu.zone = None;
                send_event(cpu_id, SGI_IPI_ID as _, IPI_EVENT_SHUTDOWN);
            }

            this_cpu_data().zone = None;
            drop(zone);
            remove_zone(zone_id);

            if is_root {
                psci::system_off().unwrap();
            }

            this_cpu_data().arch_cpu.idle();
        }

        _ => {
            warn!("unsupported smc standard service {:#x?}", code);
            0
        }
    }
}

fn handle_arch_smc(
    _regs: &mut GeneralRegisters,
    code: u64,
    _arg0: u64,
    _arg1: u64,
    _arg2: u64,
) -> u64 {
    match code {
        SMCccFnId::SMCCC_VERSION => ARM_SMCCC_VERSION_1_1,
        SMCccFnId::SMCCC_ARCH_FEATURES => !0,
        _ => {
            error!("unsupported ARM smc service");
            return !0;
        }
    }
}

fn arch_skip_instruction(_regs: &mut GeneralRegisters) {
    //ELR_EL2: ret address
    let mut pc = ELR_EL2.get();
    //ESR_EL2::IL exception instruction length
    let ins = match ESR_EL2.read(ESR_EL2::IL) {
        0 => 2, //16 bit ins
        1 => 4, //32 bit ins
        _ => 0,
    };
    //skip ins
    pc = pc + ins;
    ELR_EL2.set(pc);
}

fn arch_dump_exit(reason: u64) {
    //TODO hypervisor coredump
    error!("Unsupported Exit:{:#x?}, elr={:#x?}", reason, ELR_EL2.get());
    loop {}
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn vmreturn(_gu_regs: usize) -> ! {
    core::arch::asm!(
        "
        /* x0: guest registers */
        mov	sp, x0
        ldp	x1, x0, [sp], #16	/* x1 is the exit_reason */
        ldp	x1, x2, [sp], #16
        ldp	x3, x4, [sp], #16
        ldp	x5, x6, [sp], #16
        ldp	x7, x8, [sp], #16
        ldp	x9, x10, [sp], #16
        ldp	x11, x12, [sp], #16
        ldp	x13, x14, [sp], #16
        ldp	x15, x16, [sp], #16
        ldp	x17, x18, [sp], #16
        ldp	x19, x20, [sp], #16
        ldp	x21, x22, [sp], #16
        ldp	x23, x24, [sp], #16
        ldp	x25, x26, [sp], #16
        ldp	x27, x28, [sp], #16
        ldp	x29, x30, [sp], #16
        /*now el2 sp point to per cpu stack top*/
        eret                            //ret to el2_entry hvc #0 now,depend on ELR_EL2
        
    ",
        options(noreturn),
    )
}
