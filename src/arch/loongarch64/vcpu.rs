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
//
use crate::cpu_data::this_cpu_data;
use crate::device::irqchip::ls7a2000::{clear_injected_irq, inject_irq};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::Mutex;

/// Pending bitmap size; bits are guest ESTAT.IS lines (0..=12).
pub const GUEST_IRQ_NR: usize = 13;

/// Guest ESTAT line of the IPI interrupt.
pub const INT_IPI: usize = 12;

/// Level-triggered bitmap numbers; the rest are edges.
pub const GUEST_LINES: &[usize] = &[INT_IPI];

const IPI_ENABLE_RESET: u32 = 0xffff_ffff;

pub struct ArchVcpu {
    pub ipi: VirtIpiRegs,
}

impl ArchVcpu {
    pub const fn new() -> Self {
        Self {
            ipi: VirtIpiRegs::new(),
        }
    }

    #[inline]
    pub fn prepare_post(&self, _irq_id: usize, _source_cpu: Option<usize>) -> bool {
        true
    }

    pub fn clear_irq_state(&self) {
        self.ipi.status.store(0, Ordering::Release);
    }
}

/// Guest-visible IPI registers of one vCPU.
pub struct VirtIpiRegs {
    pub status: AtomicU32,
    pub status_lock: Mutex<()>,
    pub enable: AtomicU32,
    pub mailbox: [AtomicU64; 4],
}

impl VirtIpiRegs {
    pub const fn new() -> Self {
        Self {
            status: AtomicU32::new(0),
            status_lock: Mutex::new(()),
            enable: AtomicU32::new(IPI_ENABLE_RESET),
            mailbox: [const { AtomicU64::new(0) }; 4],
        }
    }

    pub fn reset(&self) {
        self.status.store(0, Ordering::Release);
        self.enable.store(IPI_ENABLE_RESET, Ordering::Release);
        for mailbox in &self.mailbox {
            mailbox.store(0, Ordering::Release);
        }
    }
}

/// Sync dirty guest ESTAT lines.
pub fn flush_guest_irqs() {
    this_cpu_data()
        .vcpu
        .irq
        .take_dirty_lines(|irq_id, level| match level {
            true => inject_irq(irq_id, false),
            false => clear_injected_irq(irq_id),
        });
}
