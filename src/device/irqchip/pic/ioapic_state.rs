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

pub const NUM_PINS: usize = 24;
const MASKED: u64 = 1 << 16;
const LEVEL: u64 = 1 << 15;
const REMOTE_IRR: u64 = 1 << 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub entry: u64,
    pub level: bool,
}

pub struct IoApicState {
    rte: [u64; NUM_PINS],
    asserted: u32,
    in_service: [Option<u8>; NUM_PINS],
    stopping: bool,
}

impl Default for IoApicState {
    fn default() -> Self {
        Self {
            rte: [MASKED; NUM_PINS],
            asserted: 0,
            in_service: [None; NUM_PINS],
            stopping: false,
        }
    }
}

impl IoApicState {
    pub fn entry(&self, pin: usize) -> Option<u64> {
        self.rte.get(pin).map(|entry| {
            if self.in_service[pin].is_some() {
                entry | REMOTE_IRR
            } else {
                entry & !REMOTE_IRR
            }
        })
    }

    fn deliver(&mut self, pin: usize) -> Option<Delivery> {
        if self.stopping {
            return None;
        }
        let entry = *self.rte.get(pin)?;
        let vector = entry as u8;
        if entry & MASKED != 0 || vector < 0x20 {
            return None;
        }
        let level = entry & LEVEL != 0;
        if level {
            if self.asserted & (1 << pin) == 0 || self.in_service[pin].is_some() {
                return None;
            }
            self.in_service[pin] = Some(vector);
        }
        Some(Delivery { entry, level })
    }

    pub fn set_line(&mut self, pin: usize, high: bool) -> Option<Delivery> {
        self.rte.get(pin)?;
        if self.stopping {
            return None;
        }
        let bit = 1 << pin;
        let was_high = self.asserted & bit != 0;
        if high {
            self.asserted |= bit;
        } else {
            self.asserted &= !bit;
        }
        if !high || was_high {
            return None;
        }
        // An edge received while masked is lost. A level remains asserted.
        self.deliver(pin)
    }

    pub fn pulse(&mut self, pin: usize) -> Option<Delivery> {
        let delivery = self.set_line(pin, true);
        self.set_line(pin, false);
        delivery
    }

    pub fn write_half(&mut self, pin: usize, high: bool, value: u32) -> Option<Delivery> {
        let entry = self.rte.get_mut(pin)?;
        if high {
            *entry = (*entry & 0xffff_ffff) | ((value as u64) << 32);
        } else {
            let low = value & !(REMOTE_IRR as u32);
            *entry = (*entry & !0xffff_ffff) | low as u64;
        }
        // Linux's old-IOAPIC EOI workaround changes a level RTE to edge.
        if *entry & LEVEL == 0 {
            self.in_service[pin] = None;
        }
        if *entry & LEVEL != 0 {
            self.deliver(pin)
        } else {
            None
        }
    }

    pub fn eoi(&mut self, vector: u8) -> [Option<Delivery>; NUM_PINS] {
        let mut deliveries = [None; NUM_PINS];
        for pin in 0..NUM_PINS {
            if self.in_service[pin] == Some(vector) {
                self.in_service[pin] = None;
                deliveries[pin] = self.deliver(pin);
            }
        }
        deliveries
    }

    pub fn begin_shutdown(&mut self) {
        self.stopping = true;
        self.asserted = 0;
        self.in_service.fill(None);
    }
}
