#![allow(unused)]

use crate::{
    arch::{
        cpu::this_cpu_id,
        ipi::*,
        register::{read_gcsr_estat, write_gcsr_estat},
    },
    zone::Zone,
};
use chip::*;

pub mod chip;

pub fn primary_init_early() {
    if this_cpu_id() != 0 {
        info!("loongarch64: irqchip: primary_init_early do nothing on secondary cpus");
        return;
    }
    info!("loongarch64: irqchip: primary_init_early checking iochip configs");
    print_chip_info();
    csr_disable_new_codec();
    legacy_int_enable_all();
    extioi_mode_disable();
    info!("loongarch64: irqchip: testing percore IPI feature");
    let is_ipi_percore = get_ipi_percore();
    info!(
        "loongarch64: irqchip: percore IPI feature: {}",
        is_ipi_percore
    );
}
pub fn primary_init_late() {
    warn!("loongarch64: irqchip: primary_init_late do nothing");
}
pub fn percpu_init() {
    info!("loongarch64: irqchip: running percpu_init");
    clear_all_ipi(this_cpu_id());
    enable_ipi(this_cpu_id());
    ecfg_ipi_enable();
    info!("loongarch64: irqchip: dumping ipi registers");
    dump_ipi_registers();
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

/// inject irq to THIS cpu
pub fn inject_irq(_irq: usize, is_hardware: bool) {
    debug!(
        "loongarch64: inject_irq, _irq: {}, is_hardware: {}",
        _irq, is_hardware
    );
    print!("\0");
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

/// clear the injecting irq ctrl bit on THIS cpu
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
    print!("\0");
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        warn!("loongarch64: irqchip: arch_irqchip_reset do nothing");
    }
}
