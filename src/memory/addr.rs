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
//! Definition of phyical and virtual addresses.

#![allow(dead_code)]

use core::fmt::Debug;

use crate::consts::PAGE_SIZE;

pub type VirtAddr = usize;
pub type PhysAddr = usize;

pub type GuestVirtAddr = usize;
pub type GuestPhysAddr = usize;

pub type HostVirtAddr = VirtAddr;
pub type HostPhysAddr = PhysAddr;

pub static mut PHYS_VIRT_OFFSET: usize = 0; //HV_BASE - HV_PHY_BASE;

pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    vaddr - unsafe { PHYS_VIRT_OFFSET }
}

pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr + unsafe { PHYS_VIRT_OFFSET }
}

pub const fn align_down(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

pub const fn align_up(addr: usize) -> usize {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn is_aligned(addr: usize) -> bool {
    page_offset(addr) == 0
}

pub const fn page_count(size: usize) -> usize {
    align_up(size) / PAGE_SIZE
}

pub const fn page_offset(addr: usize) -> usize {
    addr & (PAGE_SIZE - 1)
}
