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
/// Pending bitmap size; bits are `hvip` local interrupt numbers.
pub const GUEST_IRQ_NR: usize = 64;

// Architecture-private vCPU state placeholder.
pub struct ArchVcpu;

impl ArchVcpu {
    pub const fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    #[inline]
    pub fn prepare_post(&self, _irq_id: usize, _source_cpu: Option<usize>) -> bool {
        true
    }

    pub fn clear_irq_state(&self) {}
}

pub fn flush_guest_irqs() {}
