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
#[cfg(irq_gicv2)]
use crate::device::irqchip::gicv2::{gicd::GICV2_SGIS_NUM, pending::MAX_CPU_NUM};
#[cfg(irq_gicv2)]
use core::sync::atomic::{AtomicU8, Ordering};

/// Pending bitmap size; bits are GIC vINTIDs.
pub const GUEST_IRQ_NR: usize = crate::config::CONFIG_MAX_INTERRUPTS;

/// Level-triggered bitmap numbers; the rest are edges.
pub const GUEST_LINES: &[usize] = &[];

/// Architecture-private vCPU state, separate from the host pCPU's `ArchCpu`.
pub struct ArchVcpu {
    #[cfg(irq_gicv2)]
    pub sgi_source: [AtomicU8; GICV2_SGIS_NUM],
}

impl ArchVcpu {
    pub const fn new() -> Self {
        Self {
            #[cfg(irq_gicv2)]
            sgi_source: [const { AtomicU8::new(0) }; GICV2_SGIS_NUM],
        }
    }

    #[inline]
    pub fn prepare_post(&self, irq_id: usize, source_cpu: Option<usize>) -> bool {
        #[cfg(irq_gicv2)]
        if irq_id < GICV2_SGIS_NUM {
            let source_cpu = source_cpu
                .filter(|source_cpu| *source_cpu < MAX_CPU_NUM)
                .expect("GICv2 SGI source must be validated by the caller");
            let source_bit = 1u8 << source_cpu;
            let previous_sources = self.sgi_source[irq_id].fetch_or(source_bit, Ordering::AcqRel);
            return previous_sources & source_bit == 0;
        }
        #[cfg(not(irq_gicv2))]
        let _ = (irq_id, source_cpu);
        true
    }

    pub fn clear_irq_state(&self) {
        #[cfg(irq_gicv2)]
        for source in &self.sgi_source {
            source.store(0, Ordering::Release);
        }
    }
}

/// Install the current vCPU's posted IRQs into the GIC list registers.
#[inline]
pub fn flush_guest_irqs() {
    #[cfg(irq_gicv2)]
    crate::device::irqchip::gicv2::lr::flush_current_guest_irqs();
    #[cfg(irq_gicv3)]
    crate::device::irqchip::gicv3::lr::flush_current_guest_irqs();
}
