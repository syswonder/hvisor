#[cfg(feature = "gicv3")]
use crate::arch::sysreg::write_sysreg;
#[cfg(feature = "gicv2")]
use crate::device::irqchip::set_sgi_irq;
pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    #[cfg(feature = "gicv3")] {
        let aff3: u64 = 0 << 48;
        let aff2: u64 = 0 << 32;
        let aff1: u64 = 0 << 16;
        let irm: u64 = 0 << 40;
        let sgi_id: u64 = sgi_num << 24;
        let target_list: u64 = 1 << cpu_id;
        let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
        write_sysreg!(icc_sgi1r_el1, val);
        debug!("write sgi sys value = {:#x}", val);
    }
    #[cfg(feature = "gicv2")] {
        let sgi_id: u64 = sgi_num;
        let target_list: u64 = 1 << cpu_id;
        set_sgi_irq(sgi_id as usize, target_list as usize, 0);
    }
}
