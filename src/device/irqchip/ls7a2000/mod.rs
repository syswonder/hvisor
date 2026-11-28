// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//      Yulong Han <wheatfox17@icloud.com>
//
#![allow(unused)]

use crate::{
    arch::{
        clock::*,
        cpu::this_cpu_id,
        ipi::*,
        register::{read_gcsr_estat, write_gcsr_estat},
    },
    consts::MAX_CPU_NUM,
    device::irqchip::{post_guest_irq, GuestIrqSignal},
    zone::Zone,
};
use chip::*;

pub mod chip;

pub fn primary_init_early() {
    info!("loongarch64: irqchip: primary_init_early: checking iochip configs");
    print_chip_info();
    csr_disable_new_codec();
    // legacy_int_enable_all();
    // extioi_mode_disable();
    info!("loongarch64: irqchip: testing percore IPI feature");
    let is_ipi_percore = get_ipi_percore();
    info!(
        "loongarch64: irqchip: percore IPI feature: {}",
        is_ipi_percore
    );
}
pub fn primary_init_late() {
    info!("loongarch64: irqchip: primary_init_late: running primary_init_late");

    info!("loongarch64: irqchip: primary_init_late: testing UART1");
    crate::device::uart::loongson_uart::__test_uart1();

    info!("loongarch64: irqchip: primary_init_late: probing pci");
    probe_pci();

    info!("loongarch64: irqchip: primary_init_late: clearing extioi SR regs");
    clear_extioi_sr();
    let extioi_sr = get_extioi_sr();
    info!(
        "loongarch64: irqchip: primary_init_late: extioi_sr: {}",
        extioi_sr
    );

    info!("loongarch64: irqchip: primary_init_late finished");
}

// actually these configures are from cpucfg, not irqchip, but we put all
// configuration stuff here for convenience
pub fn clock_cpucfg_dump() {
    info!(
        "loongarch64: irqchip: clock_cpucfg_dump: cc_freq: {}",
        get_cpucfg_cc_freq()
    );
    info!(
        "loongarch64: irqchip: clock_cpucfg_dump: cc_mul: {}",
        get_cpucfg_cc_mul()
    );
    info!(
        "loongarch64: irqchip: clock_cpucfg_dump: cc_div: {}",
        get_cpucfg_cc_div()
    );
}

pub fn percpu_init() {
    info!("loongarch64: irqchip: percpu_init: running percpu_init");

    clear_all_ipi();
    enable_ipi();
    ecfg_ipi_enable();
    clock_cpucfg_dump();
    // timer_test_tick();
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

/// Drive guest HWI line `irq` of `cpu`; the target applies it at its next
/// flush. Returns `false` for an invalid CPU or HWI number.
pub fn post_guest_hwi(cpu: usize, irq: usize, asserted: bool) -> bool {
    if cpu >= MAX_CPU_NUM || !(INT_HWI0..=INT_HWI7).contains(&irq) {
        error!(
            "loongarch64: invalid guest IRQ line: cpu={} irq={} asserted={}",
            cpu, irq, asserted
        );
        return false;
    }
    post_guest_irq(cpu, irq, None, GuestIrqSignal::Level(asserted));
    true
}

/// Apply a dirty guest line to this CPU: HWIs to GINTC.VIP, others to GCSR
/// ESTAT.
pub fn sync_guest_line(irq: usize, level: bool) {
    if (INT_HWI0..=INT_HWI7).contains(&irq) {
        use crate::arch::register::gintc;
        let mask = 1 << (irq - INT_HWI0);
        let old_vip = gintc::read().vip();
        let new_vip = if level {
            old_vip | mask
        } else {
            old_vip & !mask
        };
        if new_vip != old_vip {
            gintc::write_vip(new_vip);
        }
        return;
    }
    let bit = 1 << irq;
    let mut gcsr_estat = read_gcsr_estat();
    if level {
        gcsr_estat |= bit;
    } else {
        gcsr_estat &= !bit;
    }
    write_gcsr_estat(gcsr_estat);
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        // Other zones may still be running and must retain their pending IRQs.
        for cpu in self.cpu_set().iter() {
            clear_extioi_sr_for_cpu(cpu);
        }
        let extioi_sr = get_extioi_sr();
        info!(
            "loongarch64: irqchip: arch_irqchip_reset: extioi_sr: {}",
            extioi_sr
        );
    }
}
