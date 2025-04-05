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

use crate::percpu::this_cpu_data;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitvec::prelude::*;
use spin::Mutex;

/// Virtual Platform-Level Interrupt Controller (vPLIC)
#[allow(unused)]
pub struct VirtualPLIC {
    /// Base address of the vPLIC in guest physical memory
    base_addr: usize,
    /// Maximum number of interrupts (excluding interrupt 0)
    max_interrupts: usize,
    /// Number of Hart contexts (contains S-mode and M-mode), only S-mode works
    num_contexts: usize,
    /// Inner state of the vPLIC (thread-safe)
    inner: Arc<Mutex<VirtualPLICInner>>,
}

/// Inner state of the vPLIC
struct VirtualPLICInner {
    /// Hardware interrupt signals (bitmap)
    hw: BitVec,
    /// Active interrupts (bitmap), indicating irq is handling by one hart.
    active: BitVec,
    /// Interrupt priorities (indexed by interrupt ID)
    priority: Vec<u32>,
    /// Pending interrupts (bitmap)
    pending: BitVec,
    /// Interrupt enable bits (per Hart, indexed by context ID)
    enable: Vec<BitVec>,
    /// Thresholds for each Hart (indexed by context ID)
    threshold: Vec<u32>,
}

impl VirtualPLIC {
    /// Create a new VirtualPLIC, need to specify the max num of interrupts and num of hart contexts.
    pub fn new(base_addr: usize, max_interrupts: usize, num_contexts: usize) -> Self {
        let vplic = VirtualPLICInner {
            hw: bitvec![0; max_interrupts + 1],
            active: bitvec![0; max_interrupts + 1],
            priority: vec![0; max_interrupts + 1],
            pending: bitvec![0; max_interrupts + 1],
            enable: (0..num_contexts)
                .map(|_| bitvec![0; max_interrupts + 1])
                .collect(),
            threshold: vec![0; num_contexts],
        };

        VirtualPLIC {
            base_addr,
            max_interrupts,
            num_contexts,
            inner: Arc::new(Mutex::new(vplic)),
        }
    }

    /// Set one interrupt as hardware interrupt.
    pub fn vplic_set_hw(&self, intr_id: usize, hw: bool) {
        let mut inner = self.inner.lock();
        inner.vplic_set_hw(intr_id, hw);
    }

    /// Inject an interrupt into the vPLIC.
    pub fn inject_irq(&self, vcontext_id: usize, intr_id: usize, hw: bool) {
        debug!("Inject interrupt {} to vcontext {}", intr_id, vcontext_id);
        let mut inner = self.inner.lock();
        if hw != inner.vplic_get_hw(intr_id) {
            error!("inject_irq {}: hw args is not eauql to vplic's", intr_id);
            return;
        }
        if !inner.vplic_get_pending(intr_id) {
            // write to pending
            inner.vplic_set_pending(intr_id, true);
            if inner.hw[intr_id] {
                inner.vplic_update_hart_line(vcontext_id);
            } else {
                for vcontext_id in 0..self.num_contexts {
                    if vcontext_id % 2 == 0 {
                        continue;
                    }
                    if inner.vplic_get_enable(intr_id, vcontext_id)
                        && inner.vplic_get_priority(intr_id)
                            > inner.vplic_get_threshold(vcontext_id)
                    {
                        inner.vplic_update_hart_line(vcontext_id);
                    }
                }
            }
        }
    }

    pub fn update_hart_line(&self, vcontext_id: usize) {
        let mut inner = self.inner.lock();
        inner.vplic_update_hart_line(vcontext_id);
    }

    /// vPLIC emul access.
    pub fn vplic_emul_access(
        &self,
        offset: usize,
        size: usize,
        value: usize,
        is_write: bool,
    ) -> u32 {
        /*
         * Note: interrupt source 0 does not exist.
         */
        if size != 4 || offset & 0x3 != 0 {
            error!("vplic_emul_access: only allowed word accesses");
            return 0;
        }
        if value > u32::MAX as usize {
            error!("vplic_emul_access: value is out of range");
            return 0;
        }

        /*
         * In VirtualPLICInner, we don't check, so in this function, we must check operations.
         */
        match offset {
            // PLIC priority
            0x0000..=0x0FFC => {
                let intr_id = offset / 4;
                // In PLIC, irq 0 does not exist.
                if intr_id > self.max_interrupts || intr_id == 0 {
                    error!("vplic_priority_access: invalid interrupt ID: {}", intr_id);
                    return 0;
                }
                debug!(
                    "vplic_priority_{}: intr_id {}, prio {}",
                    if is_write { "write" } else { "read" },
                    intr_id,
                    value
                );
                let mut inner = self.inner.lock();
                if is_write {
                    inner.vplic_set_priority(intr_id, value as u32);
                    if inner.hw[intr_id] {
                        super::host_plic().set_priority(intr_id, value as u32);
                    } else {
                        // only support S-mode hart.
                        for vcontext_id in 0..self.num_contexts {
                            if vcontext_id % 2 == 0 {
                                continue;
                            }
                            if inner.vplic_get_enable(intr_id, vcontext_id) {
                                inner.vplic_update_hart_line(vcontext_id);
                            }
                        }
                    }
                    return 0;
                } else {
                    return inner.vplic_get_priority(intr_id);
                }
            }
            // PLIC pending
            0x1000..=0x107C => {
                if is_write {
                    error!("vplic_emul_access: pending is read-only");
                    return 0;
                } else {
                    let reg_offset = offset - 0x1000;
                    let reg_idx = reg_offset / 4;
                    // calculate the irq_range in the 4 bytes, due to PLIC's word access.
                    let bits = reg_idx * 32;
                    let irq_start = bits;
                    let irq_end = bits + 31;
                    let mut pending = 0;
                    let mut inner = self.inner.lock();
                    // irq_end isn't beyond max_interrupts.
                    for irq in irq_start..=irq_end.min(self.max_interrupts) {
                        pending |= (inner.vplic_get_pending(irq) as u32) << (irq - irq_start);
                    }
                    return pending;
                }
            }
            // PLIC enable
            offset if offset >= 0x2000 && offset < (0x2000 + 0x80 * self.num_contexts) => {
                let vcontext_id = (offset - 0x2000) / 0x80;
                if vcontext_id >= self.num_contexts || vcontext_id % 2 == 0 {
                    // context should be a S-mode hart context.
                    error!("Invalid context ID {}", vcontext_id);
                    return 0;
                }
                let reg_offset = (offset - 0x2000) % 0x80;
                let reg_idx = reg_offset / 4;
                // calculate the irq_range in the 4 bytes, due to PLIC's word access.
                let bits = reg_idx * 32;
                let irq_start = bits;
                let irq_end = bits + 31;
                let mut inner = self.inner.lock();
                if is_write {
                    for irq in irq_start..=irq_end.min(self.max_interrupts) {
                        let irq_enable = (value & (1 << (irq - irq_start))) != 0;
                        if inner.enable[vcontext_id][irq] == irq_enable {
                            continue;
                        }
                        debug!(
                            "vplic_enable_access: set vcontext {} irq {} to {}",
                            vcontext_id, irq, irq_enable
                        );
                        inner.vplic_set_enable(irq, vcontext_id, irq_enable);
                        if inner.hw[irq] {
                            // vcontext_id to pcontext_id shuold move to here.
                            let pcontext_id = super::vcontext_to_pcontext(vcontext_id);
                            super::host_plic().set_enable_num(pcontext_id, irq, irq_enable);
                        } else {
                            inner.vplic_update_hart_line(vcontext_id);
                        }
                    }
                    return 0;
                } else {
                    let mut enable = 0;
                    for irq in irq_start..=irq_end.min(self.max_interrupts) {
                        enable |= (inner.enable[vcontext_id][irq] as u32) << (irq - irq_start);
                    }
                    return enable;
                }
            }
            // PLIC threshold
            offset if offset >= 0x200000 && (offset - 0x200000) % 0x1000 == 0 => {
                let vcontext_id = (offset - 0x200000) / 0x1000;
                if vcontext_id >= self.num_contexts || vcontext_id % 2 == 0 {
                    // context should be a S-mode hart context.
                    error!("Invalid context ID {}", vcontext_id);
                    return 0;
                }
                debug!(
                    "vplic_threshold_{}: vcontext {} threshold {}",
                    if is_write { "write" } else { "read" },
                    vcontext_id,
                    value
                );
                let mut inner = self.inner.lock();
                if is_write {
                    inner.vplic_set_threshold(vcontext_id, value as u32);
                    let pcontext_id = super::vcontext_to_pcontext(vcontext_id);
                    super::host_plic().set_threshold(pcontext_id, value as u32);
                    inner.vplic_update_hart_line(vcontext_id);
                    return 0;
                } else {
                    return inner.vplic_get_threshold(vcontext_id);
                }
            }
            // PLIC claim/complete
            offset if offset >= 0x200004 && (offset - 0x200004) % 0x1000 == 0 => {
                let vcontext_id = (offset - 0x200004) / 0x1000;
                if vcontext_id >= self.num_contexts || vcontext_id % 2 == 0 {
                    // context should be a S-mode hart context.
                    error!("Invalid context ID {}", vcontext_id);
                    return 0;
                }
                let mut inner = self.inner.lock();
                if is_write {
                    // implement complete operation
                    let irq_id = value;
                    if inner.hw[value] {
                        let pcontext_id = super::vcontext_to_pcontext(vcontext_id);
                        super::host_plic().complete(pcontext_id, irq_id);
                    }
                    inner.active.set(irq_id, false);

                    // if there is still pending interrupt, set the VSEIP bit.
                    inner.vplic_update_hart_line(vcontext_id);
                    return 0;
                } else {
                    // implement claim operation
                    let claimed_irq = inner.vplic_get_next_pending(vcontext_id);
                    inner.pending.set(claimed_irq as usize, false);
                    inner.active.set(claimed_irq as usize, true);

                    // if there is still pending interrupt, set the VSEIP bit.
                    inner.vplic_update_hart_line(vcontext_id);
                    return claimed_irq as u32;
                }
            }
            _ => {
                error!("Undefined PLIC offset: {:#x}", offset);
            }
        }
        return 0;
    }
}

#[allow(unused)]
impl VirtualPLICInner {
    /// vPLIC get hardware interrupt.
    fn vplic_get_hw(&self, intr_id: usize) -> bool {
        self.hw[intr_id]
    }

    /// vPLIC set hardware interrupt.
    fn vplic_set_hw(&mut self, intr_id: usize, hw: bool) {
        self.hw.set(intr_id, hw);
    }

    /// vPLIC get active interrupt.
    fn vplic_get_active(&self, intr_id: usize) -> bool {
        self.active[intr_id]
    }

    /// vPLIC set active interrupt.
    fn vplic_set_active(&mut self, intr_id: usize, active: bool) {
        self.active.set(intr_id, active);
    }

    /// vPLIC get priority.
    fn vplic_get_priority(&self, intr_id: usize) -> u32 {
        self.priority[intr_id]
    }

    /// vPLIC set priority.
    fn vplic_set_priority(&mut self, intr_id: usize, priority: u32) {
        self.priority[intr_id] = priority;
    }

    /// vPLIC get interrupt pending bit.
    fn vplic_get_pending(&self, intr_id: usize) -> bool {
        self.pending[intr_id]
    }

    fn vplic_set_pending(&mut self, intr_id: usize, pend: bool) {
        self.pending.set(intr_id, pend);
    }

    /// vPLIC get enable bit.
    fn vplic_get_enable(&self, intr_id: usize, context: usize) -> bool {
        self.enable[context][intr_id]
    }

    /// vPLIC set enable bit.
    fn vplic_set_enable(&mut self, intr_id: usize, context: usize, enable: bool) {
        self.enable[context].set(intr_id, enable);
    }

    /// vPLIC get threshold.
    fn vplic_get_threshold(&self, context: usize) -> u32 {
        self.threshold[context]
    }

    /// vPLIC set threshold.
    fn vplic_set_threshold(&mut self, context: usize, threshold: u32) {
        self.threshold[context] = threshold;
    }

    /// vPLIC get next pending interrupt with the highest priority.
    fn vplic_get_next_pending(&self, context: usize) -> u32 {
        let mut max_prio = 0;
        let mut next_irq = 0;
        for irq in 1..self.priority.len() {
            // active: confirm claimed_irq is not claim again.
            if self.pending[irq] && !self.active[irq] && self.enable[context][irq] {
                // get the highest priority irq.
                if self.priority[irq] > max_prio {
                    max_prio = self.priority[irq];
                    next_irq = irq;
                }
            }
        }
        if max_prio > self.threshold[context] {
            next_irq as u32
        } else {
            0
        }
    }

    /// Update line like physical PLIC.
    fn vplic_update_hart_line(&self, vcontext_id: usize) {
        // Due to vplic's state update, we should signal related vontext like physical PLIC does.
        let pcontext_id = super::vcontext_to_pcontext(vcontext_id);
        debug!(
            "vPLIC update line to vcontext_id {}, pcontext_id {}",
            vcontext_id, pcontext_id
        );
        if pcontext_id / 2 == this_cpu_data().id {
            let irq_id = self.vplic_get_next_pending(vcontext_id);
            if irq_id != 0 {
                unsafe {
                    riscv::register::hvip::set_vseip();
                }
            } else {
                unsafe {
                    riscv::register::hvip::clear_vseip();
                }
            }
        } else {
            use crate::event::{send_event, IPI_EVENT_UPDATE_HART_LINE};
            let cpu_id = pcontext_id / 2;
            info!("vplic_update_hart_line to cpu {}", cpu_id);
            // the second arg don't need.
            send_event(cpu_id, 0, IPI_EVENT_UPDATE_HART_LINE);
        }
    }
}
