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
pub mod aplic;
pub mod imsic;
use crate::zone::Zone;

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        /*
           Reset priority, threshold, enable, and so on related to this zone.
        */
        todo!();
    }

    fn insert_irq_to_bitmap(&mut self, irq: u32) {
        let irq_index = irq / 32;
        let irq_bit = irq % 32;
        self.irq_bitmap[irq_index as usize] |= 1 << irq_bit;
    }

    /// irq_bitmap_init, and set these irqs' hw bit in vplic to true.
    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {
        // insert to zone.irq_bitmap
        for irq in irqs {
            let irq_id = *irq;
            // They are hardware interrupts.
            self.insert_irq_to_bitmap(irq_id);
        }
        // print irq_bitmap
        for (index, &word) in self.irq_bitmap.iter().enumerate() {
            for bit_position in 0..32 {
                if word & (1 << bit_position) != 0 {
                    let interrupt_number = index * 32 + bit_position;
                    info!(
                        "Found interrupt in Zone {} irq_bitmap: {}",
                        self.id, interrupt_number
                    );
                }
            }
        }
    }
}
