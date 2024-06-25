use fdt::Fdt;
use crate::zone::Zone;

pub fn primary_init_early(host_fdt: &Fdt) {
    todo!()
}
pub fn primary_init_late() {
    todo!()
}
pub fn percpu_init() {
    todo!()
}
pub fn inject_irq(_irq: usize, is_hardware: bool) {
    todo!()
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        todo!()
    }
}
