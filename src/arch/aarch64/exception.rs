use super::entry::vmreturn;
use crate::hypercall::HyperCall;
use crate::percpu::PerCpu;
use crate::percpu::{this_cpu_data, GeneralRegisters};
use aarch64_cpu::{asm, registers::*};
use tock_registers::interfaces::*;
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod ExceptionType {
    pub const EXIT_REASON_EL2_ABORT: u64 = 0x0;
    pub const EXIT_REASON_EL1_ABORT: u64 = 0x1;
    pub const EXIT_REASON_EL1_IRQ: u64 = 0x2;
}
pub enum trap_return {
    TRAP_HANDLED = 1,
    TRAP_UNHANDLED = 0,
    TRAP_FORBIDDEN = -1,
}
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame<'a> {
    // Pushed by ` hyp_vec.S/handle_vmexit`
    pub regs: &'a GeneralRegisters,
    pub esr: u64,
    pub spsr: u64,
    //pub sp: u64,
}
impl<'a> TrapFrame<'a> {
    pub fn new(regs: &'a GeneralRegisters) -> Self {
        Self {
            regs,
            esr: ESR_EL2.get(),
            spsr: SPSR_EL2.get(),
            //sp: 0,
        }
    }
}
/*From hyp_vec->handle_vmexit x0:guest regs x1:exit_reason sp =stack_top-32*8*/
pub fn arch_handle_exit(regs: &GeneralRegisters) -> Result<(), ()> {
    match regs.exit_reason as u64 {
        ExceptionType::EXIT_REASON_EL1_IRQ => irqchip_handle_irq(),
        ExceptionType::EXIT_REASON_EL1_ABORT => arch_handle_trap(regs),
        ExceptionType::EXIT_REASON_EL2_ABORT => arch_dump_exit(),
        _ => arch_dump_exit(),
    }
    unsafe {
        vmreturn(regs as *const _ as usize);
    }

    Ok(())
}
fn irqchip_handle_irq() {}
fn arch_handle_trap(regs: &GeneralRegisters) {
    let frame = TrapFrame::new(regs);
    let mut ret = trap_return::TRAP_UNHANDLED;
    match ESR_EL2.read(ESR_EL2::EC) {
        0x16 => handle_hvc(&frame),
        _ => {
            ret = trap_return::TRAP_UNHANDLED;
        }
    }
}
fn handle_hvc(frame: &TrapFrame) {
    /*
    if ESR_EL2.read(ESR_EL2::ISS) != 0x4a48 {
        return;
    }
    */
    let (code, arg0, arg1) = (frame.regs.usr[0], frame.regs.usr[1], frame.regs.usr[2]);
    let cpu_data = unsafe { this_cpu_data() as &mut PerCpu };
    HyperCall::new(cpu_data).hypercall(code as _, arg0, arg1);
}
fn arch_dump_exit() {}
