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
    event::{send_event, IPI_EVENT_VIRTIO_INJECT_IRQ},
    hypercall::SGI_IPI_ID,
    zone::Zone,
};
use chip::*;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

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

static GUEST_HWI_ASSERTED: [AtomicU32; MAX_CPU_NUM] = {
    const C: AtomicU32 = AtomicU32::new(0);
    [C; MAX_CPU_NUM]
};

/// Serializes the software HWI bitmap with GINTC VIP updates for each pCPU.
static GUEST_HWI_LOCKS: [Mutex<()>; MAX_CPU_NUM] = {
    const L: Mutex<()> = Mutex::new(());
    [L; MAX_CPU_NUM]
};

fn sync_guest_irqs_unlocked(cpu: usize) {
    use crate::arch::register::gintc;
    let desired_vip = GUEST_HWI_ASSERTED[cpu].load(Ordering::Relaxed) as usize & 0xff;
    let old_vip = gintc::read().vip();
    if old_vip != desired_vip {
        gintc::write_vip(desired_vip);
    }
}

pub fn set_guest_irq_line(cpu: usize, irq: usize, asserted: bool) -> bool {
    if cpu >= MAX_CPU_NUM || !(INT_HWI0..=INT_HWI7).contains(&irq) {
        error!(
            "loongarch64: invalid guest IRQ line: cpu={} irq={} asserted={}",
            cpu, irq, asserted
        );
        return false;
    }

    let mask = 1u32 << (irq - INT_HWI0);
    let need_remote_sync = {
        let _guard = GUEST_HWI_LOCKS[cpu].lock();
        let state = &GUEST_HWI_ASSERTED[cpu];
        let old = state.load(Ordering::Relaxed);
        let new = if asserted { old | mask } else { old & !mask };
        if old == new {
            false
        } else {
            state.store(new, Ordering::Relaxed);
            if cpu == this_cpu_id() {
                sync_guest_irqs_unlocked(cpu);
                false
            } else {
                true
            }
        }
    };
    // Drop the lock before kicking the target; its handler takes the same lock.
    if need_remote_sync {
        send_event(cpu, SGI_IPI_ID as usize, IPI_EVENT_VIRTIO_INJECT_IRQ);
    }
    true
}

pub fn clear_guest_irq_lines(cpu: usize) {
    if cpu >= MAX_CPU_NUM {
        return;
    }
    let need_remote_sync = {
        let _guard = GUEST_HWI_LOCKS[cpu].lock();
        let old = GUEST_HWI_ASSERTED[cpu].swap(0, Ordering::Relaxed);
        if old == 0 {
            false
        } else if cpu == this_cpu_id() {
            sync_guest_irqs_unlocked(cpu);
            false
        } else {
            true
        }
    };
    if need_remote_sync {
        send_event(cpu, SGI_IPI_ID as usize, IPI_EVENT_VIRTIO_INJECT_IRQ);
    }
}

pub fn sync_guest_irqs() {
    let cpu = this_cpu_id();
    let _guard = GUEST_HWI_LOCKS[cpu].lock();
    sync_guest_irqs_unlocked(cpu);
}

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
        set_guest_irq_line(this_cpu_id(), _irq, true);
    } else {
        // use gcsr to inject, just set the bit
        let mut gcsr_estat = read_gcsr_estat();
        gcsr_estat |= bit;
        write_gcsr_estat(gcsr_estat);
    }
}

pub fn clear_injected_irq(_irq: usize) {
    debug!("loongarch64: clear_injected_irq: _irq: {}", _irq);
    if _irq > INT_IPI {
        error!(
            "loongarch64: clear_injected_irq: _irq > {}, not valid",
            INT_IPI
        );
        return;
    }
    let bit = 1 << _irq;
    if _irq >= INT_HWI0 && _irq <= INT_HWI7 {
        set_guest_irq_line(this_cpu_id(), _irq, false);
    } else {
        let mut gcsr_estat = read_gcsr_estat();
        gcsr_estat &= !bit;
        write_gcsr_estat(gcsr_estat);
    }
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
