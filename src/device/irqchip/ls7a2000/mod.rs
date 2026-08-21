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

/// Per-pCPU software HWI bitmap, serialized with GINTC VIP updates.
#[percpu::def_percpu]
static GUEST_HWI_ASSERTED: Mutex<u32> = Mutex::new(0);

// The caller ensures the cpu_id is valid.
#[inline(always)]
fn get_guest_hwi_state(cpu: usize) -> &'static Mutex<u32> {
    unsafe { GUEST_HWI_ASSERTED.remote_ref_raw(cpu) }
}

fn sync_guest_irqs_for_cpu(cpu: usize, update: impl FnOnce(&mut u32) -> bool) -> bool {
    let mut state = get_guest_hwi_state(cpu).lock();
    let changed = update(&mut state);

    // GINTC.VIP is a per-CPU register, so only the local CPU can apply the
    // bitmap immediately. Remote CPUs are kicked after the lock is released.
    if cpu == this_cpu_id() {
        use crate::arch::register::gintc;
        let desired_vip = (*state as usize) & 0xff;
        let old_vip = gintc::read().vip();
        if old_vip != desired_vip {
            gintc::write_vip(desired_vip);
        }
    }
    changed && cpu != this_cpu_id()
}

pub fn sync_guest_irqs() {
    sync_guest_irqs_for_cpu(this_cpu_id(), |_| false);
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
    let need_remote_sync = sync_guest_irqs_for_cpu(cpu, |state| {
        let old = *state;
        let new = if asserted { old | mask } else { old & !mask };
        if old == new {
            false
        } else {
            *state = new;
            true
        }
    });
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
    let need_remote_sync = sync_guest_irqs_for_cpu(cpu, |state| {
        let old = *state;
        *state = 0;
        old != 0
    });
    if need_remote_sync {
        send_event(cpu, SGI_IPI_ID as usize, IPI_EVENT_VIRTIO_INJECT_IRQ);
    }
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
