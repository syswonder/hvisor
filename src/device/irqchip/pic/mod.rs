pub mod i8259;
pub mod lapic;

use crate::zone::Zone;
use core::arch::asm;

pub fn enable_irq() {
    unsafe { asm!("sti") };
}

pub fn disable_irq() {
    unsafe { asm!("cli") };
}

pub fn inject_irq(_irq: usize, _is_hardware: bool) {}

pub fn percpu_init() {}

pub fn primary_init_early() {
    warn!("x86_64: irqchip: primary_init_early do nothing");
}

pub fn primary_init_late() {}

impl Zone {
    pub fn arch_irqchip_reset(&self) {}
}
