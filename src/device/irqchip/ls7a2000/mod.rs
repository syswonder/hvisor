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
    zone::Zone,
};
use chip::*;
use loongArch64::register::tcfg;
use spin::Mutex;

pub mod chip;

pub fn primary_init_early() {
    if this_cpu_id() != 0 {
        info!("loongarch64: irqchip: primary_init_early: do nothing on secondary cpus");
        return;
    }
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
// configuartion stuff here for convenience
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

    clear_all_ipi(this_cpu_id());
    enable_ipi(this_cpu_id());
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

/// inject irq to THIS cpu
pub fn inject_irq(_irq: usize, is_hardware: bool) {
    debug!(
        "loongarch64: inject_irq: _irq: {}, is_hardware: {}",
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
    let mut status = GLOBAL_IRQ_INJECT_STATUS.lock();
    status.cpu_status[this_cpu_id()].status = InjectionStatus::Injecting;

    tcfg::set_en(true); // start timer to avoid endless timer injection
                        // please only enable this for debugging because it may cause overheads for realtime nonroots
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
        "loongarch64: clear_hwi_injected_irq: current gintc: {:#x}",
        gintc_raw
    );
    let mut status = GLOBAL_IRQ_INJECT_STATUS.lock();
    status.cpu_status[this_cpu_id()].status = InjectionStatus::Idle;

    tcfg::set_en(false); // stop timer
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        // clear all SR regs
        clear_extioi_sr();
        let extioi_sr = get_extioi_sr();
        info!(
            "loongarch64: irqchip: arch_irqchip_reset: extioi_sr: {}",
            extioi_sr
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InjectionStatus {
    Injecting,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PercpuInjectionStatus {
    pub status: InjectionStatus,
    pub irqs: [u32; 32],
}

#[derive(Debug)]
pub struct GlobalInjectionStatus {
    pub cpu_status: [PercpuInjectionStatus; MAX_CPU_NUM],
}

pub static GLOBAL_IRQ_INJECT_STATUS: Mutex<GlobalInjectionStatus> =
    Mutex::new(GlobalInjectionStatus {
        cpu_status: [PercpuInjectionStatus {
            status: InjectionStatus::Idle,
            irqs: [0; 32],
        }; MAX_CPU_NUM],
    });
