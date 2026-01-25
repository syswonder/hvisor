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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

/*
    PLIC Memory Map:

    base + 0x000000: Reserved (interrupt source 0 does not exist)
    base + 0x000004: Interrupt source 1 priority
    base + 0x000008: Interrupt source 2 priority
    ...
    base + 0x000FFC: Interrupt source 1023 priority
    base + 0x001000: Interrupt Pending bit 0-31
    base + 0x00107C: Interrupt Pending bit 992-1023
    ...
    base + 0x002000: Enable bits for sources 0-31 on context 0
    base + 0x002004: Enable bits for sources 32-63 on context 0
    ...
    base + 0x00207C: Enable bits for sources 992-1023 on context 0
    base + 0x002080: Enable bits for sources 0-31 on context 1
    base + 0x002084: Enable bits for sources 32-63 on context 1
    ...
    base + 0x0020FC: Enable bits for sources 992-1023 on context 1
    base + 0x002100: Enable bits for sources 0-31 on context 2
    base + 0x002104: Enable bits for sources 32-63 on context 2
    ...
    base + 0x00217C: Enable bits for sources 992-1023 on context 2
    ...
    base + 0x1F1F80: Enable bits for sources 0-31 on context 15871
    base + 0x1F1F84: Enable bits for sources 32-63 on context 15871
    base + 0x1F1FFC: Enable bits for sources 992-1023 on context 15871
    ...
    base + 0x1FFFFC: Reserved
    base + 0x200000: Priority threshold for context 0
    base + 0x200004: Claim/complete for context 0
    base + 0x200008: Reserved
    ...
    base + 0x200FFC: Reserved
    base + 0x201000: Priority threshold for context 1
    base + 0x201004: Claim/complete for context 1
    ...
    base + 0x3FFF000: Priority threshold for context 15871
    base + 0x3FFF004: Claim/complete for context 15871
    base + 0x3FFF008: Reserved
    ...
    base + 0x3FFFFFC: Reserved
*/

// PLIC reg offset
pub const PLIC_PRIORITY_OFFSET: usize = 0x0000;
#[allow(unused)]
pub const PLIC_PENDING_OFFSET: usize = 0x1000;
pub const PLIC_ENABLE_OFFSET: usize = 0x2000;
pub const PLIC_THRESHOLD_OFFSET: usize = 0x200000;
pub const PLIC_CLAIM_OFFSET: usize = 0x200004;
pub const PLIC_COMPLETE_OFFSET: usize = 0x200004;

pub const PLIC_MAX_IRQ: usize = 1023; // 1-1023, in PLIC, irq 0 does not exist.
pub const PLIC_MAX_CONTEXT: usize = 15872;

use crate::platform::BOARD_PLIC_INTERRUPTS_NUM;
use crate::platform::NUM_CONTEXTS_PER_HART;

/// Plic struct
pub struct Plic {
    base: usize,
}

#[allow(unused)]
impl Plic {
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    /// Plic init global
    pub fn init_global(&self, num_interrupts: usize, num_contexts: usize) {
        if num_interrupts > PLIC_MAX_IRQ {
            panic!("PLIC: num_interrupts is too large");
        }
        if num_contexts > PLIC_MAX_CONTEXT {
            panic!("PLIC: num_contexts is too large");
        }
        info!(
            "PLIC init global: num_interrupts = {}, num_contexts = {}",
            num_interrupts, num_contexts
        );
        // set priority to 0
        for i in 1..=num_interrupts {
            self.set_priority(i, 0);
        }
        // set enable to 0
        for i in 0..num_contexts {
            for j in 0..((num_interrupts + 31) / 32) {
                self.set_enable(i, j * 4, 0);
            }
        }
    }

    /// Plic init per hart
    pub fn init_per_hart(&self, cpu_id: usize) {
        // set threshold to 0
        info!("PLIC init per hart: cpu_id = {}", cpu_id);
        let context = cpu_id * NUM_CONTEXTS_PER_HART + 1;
        self.set_threshold(context, 0);
    }

    /// Plic set priority
    pub fn set_priority(&self, irq_id: usize, priority: u32) {
        let addr = self.base + PLIC_PRIORITY_OFFSET + irq_id * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, priority);
        }
    }

    /// Plic get priority
    pub fn get_priority(&self, irq_id: usize) -> u32 {
        let addr = self.base + PLIC_PRIORITY_OFFSET + irq_id * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Plic set enable
    pub fn set_enable(&self, context: usize, irq_base: usize, value: u32) {
        let addr = self.base + PLIC_ENABLE_OFFSET + context * 0x80 + irq_base;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Plic set enable by irq number
    pub fn set_enable_num(&self, context: usize, irq_id: usize, enable: bool) {
        let addr = self.base + PLIC_ENABLE_OFFSET + context * 0x80 + irq_id / 32 * 4;
        let mut value = unsafe { core::ptr::read_volatile(addr as *const u32) };
        if enable {
            value |= 1 << (irq_id % 32);
        } else {
            value &= !(1 << (irq_id % 32));
        }
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Plic get enable
    #[inline(always)]
    pub fn get_enable(&self, context: usize, irq_base: usize) -> u32 {
        let addr = self.base + PLIC_ENABLE_OFFSET + context * 0x80 + irq_base;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Plic get pending
    #[inline(always)]
    fn get_pending(&self, irq_base: usize) -> u32 {
        let addr = self.base + PLIC_PENDING_OFFSET + irq_base;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Plic set threshold
    pub fn set_threshold(&self, context: usize, value: u32) {
        let addr = self.base + PLIC_THRESHOLD_OFFSET + context * 0x1000;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Plic get threshold
    pub fn get_threshold(&self, context: usize) -> u32 {
        let addr = self.base + PLIC_THRESHOLD_OFFSET + context * 0x1000;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Plic claim
    pub fn claim(&self, context: usize) -> u32 {
        let addr = self.base + PLIC_CLAIM_OFFSET + context * 0x1000;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Plic complete
    pub fn complete(&self, context: usize, irq_id: usize) {
        let addr = self.base + PLIC_COMPLETE_OFFSET + context * 0x1000;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, irq_id as u32);
        }
    }

    fn plic_check_enable_first_pending(&self, context: usize, ie: &mut [u32; 32]) -> bool {
        let nr_irq_groups = (BOARD_PLIC_INTERRUPTS_NUM + 1) / 32;
        // Read Current interrupt enables
        for i in 0..nr_irq_groups {
            ie[i] = self.get_enable(context, i * 4);
        }
        // Check for pending interrupts and enable only the first one found
        for i in 0..nr_irq_groups {
            let pending_irqs = self.get_pending(i * 4) & ie[i];
            // Find first pending irq
            if pending_irqs != 0 {
                let nbits = pending_irqs.trailing_zeros();
                for j in 0..nr_irq_groups {
                    let value = if j == i { 1 << nbits } else { 0 };
                    self.set_enable(context, j * 4, value);
                }
                return true;
            }
        }
        false
    }

    fn plic_restore_enable_state(&self, context: usize, ie: &[u32; 32]) {
        let nr_irq_groups = (BOARD_PLIC_INTERRUPTS_NUM + 1) / 32;
        for i in 0..nr_irq_groups {
            self.set_enable(context, i * 4, ie[i]);
        }
    }

    /// Get hw irq from PLIC
    pub fn plic_get_hwirq(&self, context: usize) -> u32 {
        /*
         * https://github.com/RVCK-Project/rvck/blob/rvck-6.6/drivers/irqchip/irq-sifive-plic.c:
         *      Due to the implementation of the claim register in the UltraRISC DP1000
         *      platform PLIC not conforming to the specification, this is a hardware
         *      bug. Therefore, when an interrupt is pending, we need to disable the other
         *      interrupts before reading the claim register. After processing the interrupt,
         *      we should then restore the enable register.
         */

        let mut hwirq = 0;
        if cfg!(feature = "dp1000_plic") {
            let mut ie: [u32; 32] = [0; 32]; // max 1024 irqs
            hwirq = if self.plic_check_enable_first_pending(context, &mut ie) {
                self.claim(context)
            } else {
                0
            };
            self.plic_restore_enable_state(context, &ie);
        } else {
            hwirq = self.claim(context);
        }

        hwirq
    }
}
