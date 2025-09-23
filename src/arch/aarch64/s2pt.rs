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
#![allow(unused)]
use aarch64_cpu::registers::VTTBR_EL2;
use crate::memory::addr::{GuestPhysAddr, HostPhysAddr, PhysAddr};
use super::paging::{GenericPTE, HvPageTable, PagingInstr};

pub struct S2PTInstr;

impl PagingInstr for S2PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {
        debug!("activating stage 2 page table at {:#x}", root_paddr);
        VTTBR_EL2.set_baddr(root_paddr as _);
        core::arch::asm!("isb");
        core::arch::asm!("tlbi vmalls12e1is");
        core::arch::asm!("dsb nsh");
    }

    fn flush(_vaddr: Option<usize>) {
        // do nothing
    }
}

pub type Stage2PageTable = HvPageTable<GuestPhysAddr, S2PTInstr>;

pub fn stage2_mode_detect() {
    info!("Dynamical detection of stage-2 paging mode is not supported yet.");
}
