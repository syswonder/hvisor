#![allow(unused)]

use crate::zone::Zone;
use fdt::Fdt;

pub mod chip;

pub fn primary_init_early() {
    warn!("loongarch64: irqchip: primary_init_early do nothing");
}
pub fn primary_init_late() {
    warn!("loongarch64: irqchip: primary_init_late do nothing");
}
pub fn percpu_init() {
    warn!("percpu_init do nothing");
}
pub fn inject_irq(_irq: usize, is_hardware: bool) {
    warn!("loongarch64: irqchip: inject_irq do nothing");
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        warn!("loongarch64: irqchip: arch_irqchip_reset do nothing");
    }
}
