use super::entry::vmreturn;
use crate::arch::sysreg::{read_sysreg, write_sysreg};
use crate::control::{park_cpu, send_event};
use crate::device::gicv3::gicv3_handle_irq_el1;
use crate::hypercall::{HyperCall, SGI_EVENT_ID};
use crate::memory::{mmio_handle_access, MMIOAccess};
use crate::num::sign_extend;
use crate::percpu::{get_cpu_data, mpidr_to_cpuid, this_cell, this_cpu_data, GeneralRegisters};
use crate::percpu::{park_current_cpu, PerCpu};
use aarch64_cpu::registers::*;
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod ExceptionType {
    pub const EXIT_REASON_EL2_ABORT: u64 = 0x0;
    pub const EXIT_REASON_EL2_IRQ: u64 = 0x1;
    pub const EXIT_REASON_EL1_ABORT: u64 = 0x2;
    pub const EXIT_REASON_EL1_IRQ: u64 = 0x3;
}
const SMC_TYPE_MASK: u64 = 0x3F000000;
#[allow(non_snake_case)]
pub mod SmcType {
    pub const ARCH_SC: u64 = 0x0;
    pub const STANDARD_SC: u64 = 0x4000000;
}

const PSCI_VERSION_1_1: u64 = 0x10001;
const PSCI_TOS_NOT_PRESENT_MP: u64 = 2;
const ARM_SMCCC_VERSION_1_0: u64 = 0x10000;

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

#[allow(dead_code)]
pub enum TrapReturn {
    TrapHandled = 1,
    TrapUnhandled = 0,
    TrapForbidden = -1,
}
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame<'a> {
    pub regs: &'a mut GeneralRegisters,
    pub esr: u64, //ESR_EL2 exception reason
    pub spsr: u64, //SPSR_EL2 exception info
                  //pub sp: u64,
}
impl<'a> TrapFrame<'a> {
    pub fn new(regs: &'a mut GeneralRegisters) -> Self {
        Self {
            regs,
            esr: ESR_EL2.get(),
            spsr: SPSR_EL2.get(),
            //sp: 0,
        }
    }
}

/*From hyp_vec->handle_vmexit x0:guest regs x1:exit_reason sp =stack_top-32*8*/
pub fn arch_handle_exit(regs: &mut GeneralRegisters) -> ! {
    let mpidr = MPIDR_EL1.get();
    let _cpu_id = mpidr_to_cpuid(mpidr);
    trace!("cpu exit, exit_reson:{:#x?}", regs.exit_reason);
    match regs.exit_reason as u64 {
        ExceptionType::EXIT_REASON_EL1_IRQ => irqchip_handle_irq1(),
        ExceptionType::EXIT_REASON_EL1_ABORT => arch_handle_trap_el1(regs),
        ExceptionType::EXIT_REASON_EL2_ABORT => arch_handle_trap_el2(regs),
        ExceptionType::EXIT_REASON_EL2_IRQ => irqchip_handle_irq2(),
        _ => arch_dump_exit(regs.exit_reason),
    }
    unsafe {
        vmreturn(regs as *const _ as usize);
    }
}

fn irqchip_handle_irq1() {
    //debug!("irq from el1");
    gicv3_handle_irq_el1();
}
fn irqchip_handle_irq2() {
    error!("irq not handle from el2");
    loop {}
}

fn arch_handle_trap_el1(regs: &mut GeneralRegisters) {
    let mut frame = TrapFrame::new(regs);
    let mut _ret = TrapReturn::TrapUnhandled;

    trace!(
        "arch_handle_trap ec={:#x?} elr={:#x?}",
        ESR_EL2.read(ESR_EL2::EC),
        ESR_EL2.read(ESR_EL2::ISS)
    );

    match ESR_EL2.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::HVC64) => handle_hvc(&mut frame),
        Some(ESR_EL2::EC::Value::SMC64) => handle_smc(&mut frame),
        Some(ESR_EL2::EC::Value::TrappedMsrMrs) => handle_sysreg(&mut frame),
        Some(ESR_EL2::EC::Value::DataAbortLowerEL) => handle_dabt(&mut frame),
        Some(ESR_EL2::EC::Value::InstrAbortLowerEL) => handle_iabt(&mut frame),
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
    match ESR_EL2.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::HVC64) => {
            error!("EL2 Exception: HVC64 call, ELR_EL2: {:#x?}", ELR_EL2.get())
        }

        Some(ESR_EL2::EC::Value::SMC64) => {
            error!("EL2 Exception: SMC64 call, ELR_EL2: {:#x?}", ELR_EL2.get())
        }

        Some(ESR_EL2::EC::Value::DataAbortCurrentEL) => {
            error!("EL2 Exception: Data Abort, ELR_EL2: {:#x?}, FAR_EL2: {:#x?}", ELR_EL2.get(),  FAR_EL2.get())
        }

        Some(ESR_EL2::EC::Value::InstrAbortCurrentEL) => {
            error!("EL2 Exception: Instruction Abort, ELR_EL2: {:#x?}, FAR_EL2: {:#x?}", ELR_EL2.get(), FAR_EL2.get())
        }

        // ... 其他异常类型
        _ => {
            error!("Unhandled EL2 Exception: EC={:#x?}", 1);
        }
    }
    loop {}
}

fn handle_iabt(_frame: &mut TrapFrame) {
    let iss = ESR_EL2.read(ESR_EL2::ISS);
    let op = iss >> 6 & 0x1;
    let hpfar = read_sysreg!(HPFAR_EL2);
    let hdfar = read_sysreg!(FAR_EL2);
    let mut address = hpfar << 8;
    address |= hdfar & 0xfff;
    error!("error ins access {} at {:#x?}!", op, address);
    error!("esr_el2: iss {:#x?}", iss);
    loop {}
    //TODO finish dabt handle
    // arch_skip_instruction(frame);
}
fn handle_dabt(frame: &mut TrapFrame) {
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
        value: if srt == 31 {
            0
        } else {
            frame.regs.usr[srt as usize]
        },
    };

    trace!("handle_dabt: {:#x?}", mmio_access);

    match mmio_handle_access(&mut mmio_access) {
        Ok(_) => {
            if !is_write && srt != 31 {
                if sse {
                    mmio_access.value = sign_extend(mmio_access.value, 8 * size);
                }
                frame.regs.usr[srt as usize] = mmio_access.value;
            }
        }
        Err(e) => {
            panic!("mmio_handle_access: {:#x?}", e);
        }
    }
    //TODO finish dabt handle
    arch_skip_instruction(frame);
}
fn handle_sysreg(frame: &mut TrapFrame) {
    //TODO check sysreg type
    //send sgi
    trace!("esr_el2: iss {:#x?}", ESR_EL2.read(ESR_EL2::ISS));
    let rt = (ESR_EL2.get() >> 5) & 0x1f;
    let val = frame.regs.usr[rt as usize];
    trace!("esr_el2 rt{}: {:#x?}", rt, val);
    let sgi_id: u64 = (val & (0xf << 24)) >> 24;
    if this_cpu_data().wait_for_poweron {
        warn!("skip send sgi {:#x?}", sgi_id);
    } else {
        // if sgi_id != 0 {
        //     warn!("send sgi {:#x?}", sgi_id);
        // }
        write_sysreg!(icc_sgi1r_el1, val);
    }

    arch_skip_instruction(frame); //skip sgi write
}
fn handle_hvc(frame: &mut TrapFrame) {
    /*
    if ESR_EL2.read(ESR_EL2::ISS) != 0x4a48 {
        return;
    }
    */
    let (code, arg0, arg1) = (frame.regs.usr[0], frame.regs.usr[1], frame.regs.usr[2]);
    let cpu_data = this_cpu_data() as &mut PerCpu;

    info!(
        "HVC from CPU{},code:{:#x?},arg0:{:#x?},arg1:{:#x?}",
        cpu_data.id, code, arg0, arg1
    );
    let result = HyperCall::new(cpu_data)
        .hypercall(code as _, arg0, arg1)
        .unwrap();
    frame.regs.usr[0] = result as _;
}
fn handle_smc(frame: &mut TrapFrame) {
    let (code, arg0, arg1, arg2) = (
        frame.regs.usr[0],
        frame.regs.usr[1],
        frame.regs.usr[2],
        frame.regs.usr[3],
    );
    let cpu_data = this_cpu_data() as &mut PerCpu;
    info!(
        "SMC from CPU{}, func_id:{:#x?}, arg0:{:#x?}, arg1:{:#x?}, arg2:{:#x?}",
        cpu_data.id, code, arg0, arg1, arg2
    );
    let result = match code & SMC_TYPE_MASK {
        SmcType::ARCH_SC => handle_arch_smc(frame, code, arg0, arg1, arg2),
        SmcType::STANDARD_SC => handle_psci_smc(frame, code, arg0, arg1, arg2),
        _ => {
            error!("unsupported smc");
            0
        }
    };

    frame.regs.usr[0] = result;

    arch_skip_instruction(frame); //skip the smc ins
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

fn psci_emulate_cpu_on(frame: &mut TrapFrame) -> u64 {
    // Todo: Check if `cpu` is in the cpuset of current cell
    let cpu = mpidr_to_cpuid(frame.regs.usr[1]);
    info!("psci: try to wake up cpu {}", cpu);

    let target_data = get_cpu_data(cpu);
    let _lock = target_data.ctrl_lock.lock();

    if target_data.wait_for_poweron {
        target_data.cpu_on_entry = frame.regs.usr[2];
        // todo: set cpu_on_context
        target_data.reset = true;
    } else {
        error!("psci: cpu {} already on", cpu);
        return u64::MAX - 3;
    };

    drop(_lock);
    info!("sending to {}", cpu);
    send_event(cpu, SGI_EVENT_ID);
    0
}

fn handle_psci_smc(frame: &mut TrapFrame, code: u64, arg0: u64, _arg1: u64, _arg2: u64) -> u64 {
    match code {
        PsciFnId::PSCI_VERSION => PSCI_VERSION_1_1,
        PsciFnId::PSCI_CPU_OFF_32 | PsciFnId::PSCI_CPU_OFF_64 => {
            park_current_cpu();
            0
        }
        PsciFnId::PSCI_AFFINITY_INFO_32 | PsciFnId::PSCI_AFFINITY_INFO_64 => {
            get_cpu_data(arg0).wait_for_poweron as _
        }
        PsciFnId::PSCI_MIG_INFO_TYPE => PSCI_TOS_NOT_PRESENT_MP,
        PsciFnId::PSCI_FEATURES => psci_emulate_features_info(frame.regs.usr[1]),
        PsciFnId::PSCI_CPU_ON_32 | PsciFnId::PSCI_CPU_ON_64 => psci_emulate_cpu_on(frame),
        PsciFnId::PSCI_SYSTEM_OFF => {
            this_cell().read().suspend();
            for cpu in this_cell().read().cpu_set.iter_except(this_cpu_data().id) {
                park_cpu(cpu);
            }
            park_current_cpu();
            0
        }

        _ => {
            error!("unsupported smc standard service {}", code);
            0
        }
    }
}

fn handle_arch_smc(_frame: &mut TrapFrame, code: u64, _arg0: u64, _arg1: u64, _arg2: u64) -> u64 {
    match code {
        SMCccFnId::SMCCC_VERSION => ARM_SMCCC_VERSION_1_0,
        SMCccFnId::SMCCC_ARCH_FEATURES => !0,
        _ => {
            error!("unsupported ARM smc service");
            return !0;
        }
    }
}

fn arch_skip_instruction(_frame: &TrapFrame) {
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
