use super::sysreg::write_sysreg;
use crate::device::irqchip::gicv2::vgic::set_sgi_irq;
pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    let sgi_id: u64 = sgi_num;
    let target_list: u64 = 1 << cpu_id;
    set_sgi_irq(sgi_id as usize, target_list as usize, 0);
}
