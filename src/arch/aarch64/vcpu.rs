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
use core::sync::atomic::{AtomicU8, Ordering};

/// Architecture-private vCPU state, separate from the host pCPU's `ArchCpu`.
pub struct ArchVcpu {
    #[cfg(irq_gicv2)]
    pub sgi_source: [AtomicU8; 16],
}

impl ArchVcpu {
    pub const fn new() -> Self {
        Self {
            #[cfg(irq_gicv2)]
            sgi_source: [const { AtomicU8::new(0) }; 16],
        }
    }

    pub fn clear_irq_state(&self) {
        #[cfg(irq_gicv2)]
        for source in &self.sgi_source {
            source.store(0, Ordering::Release);
        }
    }
}
