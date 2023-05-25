use super::vmreturn;
use crate::percpu::GeneralRegisters;
use core::arch::{asm, global_asm};

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod ExceptionType {
    pub const EXIT_REASON_EL2_ABORT: u64 = 0x0;
    pub const EXIT_REASON_EL1_ABORT: u64 = 0x1;
    pub const EXIT_REASON_EL1_IRQ: u64 = 0x2;
}
/*From hyp_vec->handle_vmexit x0:guest regs x1:exit_reason sp =stack_top-32*8*/
fn arch_handle_exit(regs: &GeneralRegisters) -> Result<(), ()> {
    match regs.exit_reason as u64 {
        ExceptionType::EXIT_REASON_EL1_ABORT => irqchip_handle_irq(),
        ExceptionType::EXIT_REASON_EL1_IRQ => arch_handle_trap(regs),
        ExceptionType::EXIT_REASON_EL2_ABORT => arch_dump_exit(),
        _ => arch_dump_exit(),
    }
    unsafe {
        vmreturn(regs as *const _ as usize);
    }

    Ok(())
}
fn irqchip_handle_irq() {}
fn arch_handle_trap(regs: &GeneralRegisters) {}
fn arch_dump_exit() {}
