#![allow(unused)]

use crate::{
    arch::register::{read_gcsr_estat, write_gcsr_estat},
    zone::Zone,
};
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

const INT_SWI0: usize = 0;
const INT_SWI1: usize = 1;
const INT_HWI0: usize = 2;
const INT_HWI1: usize = 3;
const INT_HWI2: usize = 4;
const INT_HWI3: usize = 5;
const INT_HWI4: usize = 6;
const INT_HWI5: usize = 7;
const INT_HWI6: usize = 8;
const INT_HWI7: usize = 9;
const INT_PERF: usize = 10;
const INT_TIMER: usize = 11;
const INT_IPI: usize = 12;

pub fn inject_irq(_irq: usize, is_hardware: bool) {
    debug!(
        "loongarch64: inject_irq, _irq: {}, is_hardware: {}",
        _irq, is_hardware
    );
    if _irq > INT_IPI {
        error!("loongarch64: inject_irq: _irq > {}, not valid", INT_IPI);
        return;
    }
    let bit = 1 << _irq;
    if _irq >= INT_HWI0 && _irq <= INT_HWI7 {
        // use gintc to inject
        use crate::arch::register::gintc;
        gintc::set_hwis(bit >> INT_HWI0);
    } else {
        // use gcsr to inject, just set the bit
        let mut gcsr_estat = read_gcsr_estat();
        gcsr_estat |= bit;
        write_gcsr_estat(gcsr_estat);
    }
}

pub fn clear_hwi_injected_irq() {
    use crate::arch::register::gintc;
    gintc::set_hwis(0);
    // gintc::set_hwip(0);
    // gintc::set_hwic(0xff);
    let mut gintc_raw = 0usize;
    use core::arch::asm;
    unsafe {
        asm!("csrrd {0}, 0x52", out(reg) gintc_raw);
    }
    debug!(
        "loongarch64: clear_hwi_injected_irq, current gintc: {:#x}",
        gintc_raw
    );
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        warn!("loongarch64: irqchip: arch_irqchip_reset do nothing");
    }
}
