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
// Per-CPU queue of virtual IRQs waiting for a free List register.
use crate::arch::cpu::this_cpu_id;
use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use spin::{Mutex, Once};

// virtual interrupts waiting to inject
pub(super) static PENDING_VIRQS: Once<PendingIrqs> = Once::new();
pub(super) struct PendingIrqs {
    inner: Vec<Mutex<VecDeque<(usize, bool)>>>,
}

impl PendingIrqs {
    pub(super) fn new(max_cpus: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_cpus {
            let v = Mutex::new(VecDeque::new());
            vs.push(v)
        }
        Self { inner: vs }
    }

    pub(super) fn add_irq(&self, irq_id: usize, is_hardware: bool) -> Option<()> {
        match self.inner.get(this_cpu_id()) {
            Some(pending_irqs) => {
                let mut irqs = pending_irqs.lock();
                irqs.push_back((irq_id, is_hardware));
                Some(())
            }
            _ => None,
        }
    }

    pub(super) fn fetch_irq(&self) -> Option<(usize, bool)> {
        match self.inner.get(this_cpu_id()) {
            Some(pending_irqs) => {
                let mut irqs = pending_irqs.lock();
                irqs.pop_front()
            }
            _ => None,
        }
    }

    pub(super) fn has_pending(&self) -> bool {
        self.inner
            .get(this_cpu_id())
            .map(|pending_irqs| !pending_irqs.lock().is_empty())
            .unwrap_or(false)
    }
}
