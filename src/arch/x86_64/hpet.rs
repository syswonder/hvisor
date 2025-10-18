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
//  Solicey <lzoi_lth@163.com>

use crate::memory::VirtAddr;
use bit_field::BitField;
use core::{arch::x86_64::_rdtsc, time::Duration, u32};
use spin::Mutex;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
    registers::{ReadOnly, ReadWrite},
};

type TimeValue = Duration;

lazy_static::lazy_static! {
    static ref HPET: Hpet = {
        let mut hpet = Hpet::new(0xfed0_0000);
        hpet.init();
        hpet
    };
}

bitflags::bitflags! {
    struct TimerConfigCaps: u64 {
        /// 0 - this timer generates edge-triggered interrupts. 1 - this timer
        /// generates level-triggered interrupts.
        const TN_INT_TYPE_CNF = 1 << 1;
        /// Setting this bit to 1 enables triggering of interrupts.
        const TN_INT_ENB_CNF =  1 << 2;
        /// If Tn_PER_INT_CAP is 1, then writing 1 to this field enables periodic
        /// timer.
        const TN_TYPE_CNF =     1 << 3;
        /// If this read-only bit is set to 1, this timer supports periodic mode.
        const TN_PER_INT_CAP =  1 << 4;
        /// If this read-only bit is set to 1, the size of the timer is 64-bit.
        const TN_SIZE_CAP =     1 << 5;
        /// This field is used to allow software to directly set periodic timer's
        /// accumulator.
        const TN_VAL_SET_CNF =  1 << 6;
        /// For 64-bit timer, if this field is set, the timer will be forced to
        /// work in 32-bit mode.
        const TN_32MODE_CNF =   1 << 8;
    }
}

register_structs! {
    HpetRegs {
        /// General Capabilities and ID Register.
        (0x000 => general_caps: ReadOnly<u64>),
        (0x008 => _reserved_0),
        /// General Configuration Register.
        (0x010 => general_config: ReadWrite<u64>),
        (0x018 => _reserved_1),
        /// General Interrupt Status Register.
        (0x020 => general_intr_status: ReadWrite<u64>),
        (0x028 => _reserved_2),
        /// Main Counter Value Register.
        (0x0f0 => main_counter_value: ReadWrite<u64>),
        (0x0f8 => _reserved_3),
        (0x100 => @END),
    }
}

register_structs! {
    HpetTimerRegs {
        /// Timer N Configuration and Capability Register.
        (0x0 => config_caps: ReadWrite<u64>),
        /// Timer N Comparator Value Register.
        (0x8 => comparator_value: ReadWrite<u64>),
        /// Timer N FSB Interrupt Route Register.
        (0x10 => fsb_int_route: ReadWrite<u64>),
        (0x18 => _reserved_0),
        (0x20 => @END),
    }
}

struct Hpet {
    base_vaddr: VirtAddr,
    num_timers: u8,
    period_fs: u64,
    freq_hz: u64,
    freq_mhz: u64,
    ticks_per_ms: u64,
    is_64_bit: bool,
}

impl Hpet {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self {
            base_vaddr,
            num_timers: 0,
            period_fs: 0,
            freq_hz: 0,
            freq_mhz: 0,
            ticks_per_ms: 0,
            is_64_bit: false,
        }
    }

    const fn regs(&self) -> &HpetRegs {
        unsafe { &*(self.base_vaddr as *const HpetRegs) }
    }

    const fn timer_regs(&self, n: u8) -> &HpetTimerRegs {
        assert!(n < self.num_timers);
        unsafe { &*((self.base_vaddr + 0x100 + n as usize * 0x20) as *const HpetTimerRegs) }
    }

    fn init(&mut self) {
        println!("Initializing HPET...");
        let cap = self.regs().general_caps.get();
        let num_timers = cap.get_bits(8..=12) as u8 + 1;
        let period_fs = cap.get_bits(32..);
        let is_64_bit = cap.get_bit(13);
        let freq_hz = 1_000_000_000_000_000 / period_fs;
        println!(
            "HPET: {}.{:06} MHz, {}-bit, {} timers",
            freq_hz / 1_000_000,
            freq_hz % 1_000_000,
            if is_64_bit { 64 } else { 32 },
            num_timers
        );

        self.num_timers = num_timers;
        self.period_fs = period_fs;
        self.freq_hz = freq_hz;
        self.freq_mhz = freq_hz / 1_000_000;
        self.ticks_per_ms = freq_hz / 1000;
        self.is_64_bit = is_64_bit;

        self.set_enable(false);
        for i in 0..num_timers {
            // disable timer interrupts
            let config_caps =
                unsafe { TimerConfigCaps::from_bits_retain(self.timer_regs(i).config_caps.get()) };
            self.timer_regs(i)
                .config_caps
                .set((config_caps - TimerConfigCaps::TN_INT_ENB_CNF).bits());
        }
        self.set_enable(true);
    }

    fn set_enable(&mut self, enable: bool) {
        const LEG_RT_CNF: u64 = 1 << 1; // Legacy replacement mapping will disable PIT IRQs
        const ENABLE_CNF: u64 = 1 << 0;
        let config = &self.regs().general_config;
        if enable {
            config.set(LEG_RT_CNF | ENABLE_CNF);
        } else {
            config.set(0);
        }
    }

    fn wait_millis(&self, millis: u64) {
        let main_counter_value = &self.regs().main_counter_value;
        let ticks = millis * self.ticks_per_ms;
        let init = main_counter_value.get();
        while main_counter_value.get().wrapping_sub(init) < ticks {}
    }
}

pub fn busy_wait(duration: Duration) {
    busy_wait_until(current_time() + duration);
}

fn busy_wait_until(deadline: TimeValue) {
    while current_time() < deadline {
        core::hint::spin_loop();
    }
}

pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(current_time_nanos())
}

pub fn current_ticks() -> u64 {
    HPET.regs().main_counter_value.get()
}

pub fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * 1_000 / HPET.freq_mhz
}

pub fn current_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

pub fn wait_millis(millis: u64) {
    HPET.wait_millis(millis);
}

pub fn get_tsc_freq_mhz() -> Option<u32> {
    let mut best_freq_mhz = u32::MAX;
    for _ in 0..5 {
        let tsc_start = unsafe { _rdtsc() };
        let hpet_start = current_ticks();
        wait_millis(10);
        let tsc_end = unsafe { _rdtsc() };
        let hpet_end = current_ticks();

        let nanos = ticks_to_nanos(hpet_end.wrapping_sub(hpet_start));
        let freq_mhz = ((tsc_end - tsc_start) * 1_000 / nanos) as u32;

        if freq_mhz < best_freq_mhz {
            best_freq_mhz = freq_mhz;
        }
    }
    if best_freq_mhz != u32::MAX {
        Some(best_freq_mhz)
    } else {
        None
    }
}
