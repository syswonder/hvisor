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
use super::{AlignedPage, MemFlags, MemoryRegion, PhysAddr};

static EMPTY_PAGE: AlignedPage = AlignedPage::new();

#[derive(Clone, Debug)]
pub enum Mapper {
    Offset(usize),
    Fixed(usize),
}

impl Mapper {
    #[allow(dead_code)]
    pub fn offset(&self) -> usize {
        match self {
            Self::Offset(ref off) => *off,
            Self::Fixed(ref _paddr) => 0,
        }
    }

    pub fn map_fn<VA: Into<usize>>(&self, vaddr: VA) -> PhysAddr {
        match self {
            Self::Offset(ref off) => (vaddr.into()).wrapping_sub(*off),
            Self::Fixed(ref paddr) => *paddr,
        }
    }
}

impl<VA: From<usize> + Into<usize> + Copy> MemoryRegion<VA> {
    #[allow(unused)]
    pub fn new_with_empty_mapper(start: VA, size: usize, flags: MemFlags) -> Self {
        let paddr = EMPTY_PAGE.as_ptr() as usize;
        Self::new(start, size, flags, Mapper::Fixed(paddr))
    }

    pub fn new_with_offset_mapper(
        start_vaddr: VA,
        start_paddr: PhysAddr,
        size: usize,
        flags: MemFlags,
    ) -> Self {
        let start_vaddr = start_vaddr.into();
        let start_paddr = start_paddr;
        // bug: vaddr > paddr?
        let phys_virt_offset = start_vaddr.wrapping_sub(start_paddr);
        Self::new(
            start_vaddr.into(),
            size,
            flags,
            Mapper::Offset(phys_virt_offset),
        )
    }
}

// impl MemoryRegion<GuestPhysAddr> {
//     #[allow(unused)]
//     pub fn from_hv_memregion(mem: &HvMemoryRegion, comm_page_addr: Option<HostPhysAddr>) -> Self {
//         let host_pa = if mem.flags.contains(MemFlags::COMMUNICATION) {
//             comm_page_addr.unwrap()
//         } else {
//             mem.phys_start as HostPhysAddr
//         };
//         Self::new_with_offset_mapper(
//             mem.virt_start as GuestPhysAddr,
//             host_pa,
//             mem.size as _,
//             mem.flags,
//         )
//     }
// }
