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
pub mod addr;
pub mod frame;
pub mod heap;
pub mod mapper;
pub mod mm;
pub mod mmio;

use core::ops::{Deref, DerefMut};

use bitflags::bitflags;

pub use addr::{GuestPhysAddr, HostPhysAddr, PhysAddr, VirtAddr};
pub use frame::Frame;
pub use mm::{MemoryRegion, MemorySet, PARKING_INST_PAGE};
pub use mmio::*;

use crate::arch::paging;

pub const PAGE_SIZE: usize = paging::PageSize::Size4K as usize;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct MemFlags: u64 {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const DMA           = 1 << 3;
        const IO            = 1 << 4;
        const COMMUNICATION = 1 << 5;
        const LOADABLE      = 1 << 6;
        const ROOTSHARED    = 1 << 7;
        const NO_HUGEPAGES  = 1 << 8;
        const USER          = 1 << 9;
    }
}

#[repr(align(4096))]
pub struct AlignedPage([u8; PAGE_SIZE]);

impl AlignedPage {
    pub const fn new() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Deref for AlignedPage {
    type Target = [u8; PAGE_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
