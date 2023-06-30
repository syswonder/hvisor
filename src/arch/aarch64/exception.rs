use super::entry::vmreturn;
use crate::device::gicv3::gicv3_handle_irq_el1;
use crate::header::{HvHeaderStuff, HEADER_STUFF};
use crate::hypercall::HyperCall;
use crate::percpu::PerCpu;
use crate::percpu::{get_cpu_data, this_cpu_data, GeneralRegisters};
use aarch64_cpu::{asm, registers::*};
use tock_registers::interfaces::*;
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
pub mod SmcType {
    pub const STANDARD_SC: u64 = 0x4000000;
}
pub mod PsciFnId {
    pub const PSCI_CPU_OFF_32: u64 = 0x84000002;
    pub const PSCI_AFFINITY_INFO_32: u64 = 0x84000004;
    pub const PSCI_AFFINITY_INFO_64: u64 = 0xc4000004;
}

pub enum trap_return {
    TRAP_HANDLED = 1,
    TRAP_UNHANDLED = 0,
    TRAP_FORBIDDEN = -1,
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
pub fn arch_handle_exit(regs: &mut GeneralRegisters) -> Result<(), ()> {
    let mpidr = MPIDR_EL1.get();
    let cpu_id = mpidr & 0xff00ffffff;
    debug!("cpu exit");
    match regs.exit_reason as u64 {
        ExceptionType::EXIT_REASON_EL1_IRQ => irqchip_handle_irq1(),
        ExceptionType::EXIT_REASON_EL1_ABORT => arch_handle_trap(regs),
        ExceptionType::EXIT_REASON_EL2_ABORT => arch_dump_exit(regs.exit_reason),
        ExceptionType::EXIT_REASON_EL2_IRQ => irqchip_handle_irq2(),
        _ => arch_dump_exit(regs.exit_reason),
    }
    unsafe {
        vmreturn(regs as *const _ as usize);
    }

    Ok(())
}
fn irqchip_handle_irq1() {
    debug!("irq from el1");
    gicv3_handle_irq_el1();
}
fn irqchip_handle_irq2() {
    error!("irq not handle from el2");
    loop {}
}
fn arch_handle_trap(regs: &mut GeneralRegisters) {
    let mut frame = TrapFrame::new(regs);
    let mut ret = trap_return::TRAP_UNHANDLED;
    match ESR_EL2.read_as_enum(ESR_EL2::EC) {
        Some(ESR_EL2::EC::Value::HVC64) => handle_hvc(&frame),
        Some(ESR_EL2::EC::Value::SMC64) => handle_smc(&mut frame),
        Some(ESR_EL2::EC::Value::TrappedMsrMrs) => handle_sysreg(&mut frame),
        //TODO: handle sysreg
        _ => {
            error!(
                "Unsupported Exception EC:{:#x?}!",
                ESR_EL2.read(ESR_EL2::EC)
            );
            error!("esr_el2: iss {:#x?}", ESR_EL2.read(ESR_EL2::ISS));
            loop {}
            ret = trap_return::TRAP_UNHANDLED;
        }
    }
}
fn handle_sysreg(frame: &mut TrapFrame) {
    arch_skip_instruction(frame); //skip sgi write
}
fn handle_hvc(frame: &TrapFrame) {
    /*
    if ESR_EL2.read(ESR_EL2::ISS) != 0x4a48 {
        return;
    }
    */
    let (code, arg0, arg1) = (frame.regs.usr[0], frame.regs.usr[1], frame.regs.usr[2]);
    let cpu_data = unsafe { this_cpu_data() as &mut PerCpu };

    debug!(
        "HVC from CPU{},code:{:#x?},arg0:{:#x?},arg1:{:#x?}",
        cpu_data.id, code, arg0, arg1
    );
    HyperCall::new(cpu_data).hypercall(code as _, arg0, arg1);
}
fn handle_smc(frame: &mut TrapFrame) {
    let (code, arg0, arg1, arg2) = (
        frame.regs.usr[0],
        frame.regs.usr[1],
        frame.regs.usr[2],
        frame.regs.usr[3],
    );
    let cpu_data = unsafe { this_cpu_data() as &mut PerCpu };
    debug!(
        "SMC from CPU{},func_id:{:#x?},arg0:{},arg1:{},arg2:{}",
        cpu_data.id, code, arg0, arg1, arg2
    );
    match code & SMC_TYPE_MASK {
        SmcType::STANDARD_SC => handle_psci(cpu_data, frame, code, arg0, arg1, arg2),
        _ => {
            error!("unsupported smc")
        }
    }

    arch_skip_instruction(frame); //skip the smc ins
}
fn handle_psci(
    cpu_data: &mut PerCpu,
    frame: &mut TrapFrame,
    code: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
) {
    match code {
        PsciFnId::PSCI_CPU_OFF_32 => unsafe {
            cpu_data.wait_for_poweron = true;
            //HCR_EL2.modify(HCR_EL2::IMO::SET);
            core::arch::asm!(
                "
                wfi
        ",
            );
            info!("weak up at el2!");
            loop {}
        },
        PsciFnId::PSCI_AFFINITY_INFO_32 => {
            let cpu_data = get_cpu_data(arg0);
            frame.regs.usr[0] = cpu_data.wait_for_poweron.into();
        }
        PsciFnId::PSCI_AFFINITY_INFO_64 => {
            let cpu_data = get_cpu_data(arg0);
            frame.regs.usr[0] = cpu_data.wait_for_poweron.into();
        }
        _ => {
            error!("unsupported smc standard service")
        }
    }
}
fn arch_skip_instruction(frame: &TrapFrame) {
    let mut pc = ELR_EL2.get();
    let ins = match ESR_EL2.read(ESR_EL2::IL) {
        0 => 2, //16 bit ins
        1 => 4, //32 bit ins
        _ => 0,
    };
    pc = pc + ins;
    ELR_EL2.set(pc);
}
fn arch_dump_exit(reason: u64) {
    error!("Unsupported Exit:{:#x?}!", reason);
    loop {}
}
