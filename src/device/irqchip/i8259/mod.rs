use crate::zone::Zone;

pub fn inject_irq(_irq: usize, _is_hardware: bool) {}

pub fn percpu_init() {}

pub fn primary_init_early() {}

pub fn primary_init_late() {}

impl Zone {
    pub fn arch_irqchip_reset(&self) {}
}
