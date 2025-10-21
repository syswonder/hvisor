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
#![allow(dead_code)]

//! Memory management.

use alloc::collections::btree_map::{BTreeMap, Entry};
use core::fmt::{Debug, Formatter, Result};
use spin::Once;

use super::AlignedPage;
use super::{mapper::Mapper, MemFlags};
use crate::arch::paging::{GenericPageTable, PageSize, PagingResult};
use crate::arch::Stage2PageTable;
use crate::error::HvResult;
use crate::memory::addr::is_aligned;
use crate::memory::PhysAddr;

#[derive(Clone)]
pub struct MemoryRegion<VA> {
    pub start: VA,
    pub size: usize,
    pub flags: MemFlags,
    pub mapper: Mapper,
}

pub struct MemorySet<PT: GenericPageTable>
where
    PT::VA: Ord,
{
    regions: BTreeMap<PT::VA, MemoryRegion<PT::VA>>,
    pt: PT,
}

impl<VA: From<usize> + Into<usize> + Copy> MemoryRegion<VA> {
    pub(super) fn new(start: VA, size: usize, flags: MemFlags, mapper: Mapper) -> Self {
        let start = start.into();
        Self {
            start: start.into(),
            size,
            flags,
            mapper,
        }
    }

    /// Test whether this region is overlap with `other`.
    fn is_overlap_with(&self, other: &Self) -> bool {
        let p0 = self.start.into();
        let p1 = p0 + self.size;
        let p2 = other.start.into();
        let p3 = p2 + other.size;
        !(p1 <= p2 || p0 >= p3)
    }
}

impl<PT: GenericPageTable> MemorySet<PT>
where
    PT::VA: Ord,
{
    pub fn new(pt_level: usize) -> Self {
        Self {
            regions: BTreeMap::new(),
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            pt: PT::new(pt_level),
            #[cfg(target_arch = "loongarch64")]
            pt: PT::new(),
            #[cfg(target_arch = "x86_64")]
            pt: PT::new(),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            regions: self.regions.clone(),
            pt: self.pt.clone(),
        }
    }

    pub fn root_paddr(&self) -> usize {
        self.pt.root_paddr()
    }

    fn test_free_area(&self, other: &MemoryRegion<PT::VA>) -> bool {
        if let Some((_, before)) = self.regions.range(..other.start).last() {
            if before.is_overlap_with(other) {
                return false;
            }
        }
        if let Some((_, after)) = self.regions.range(other.start..).next() {
            if after.is_overlap_with(other) {
                return false;
            }
        }
        true
    }

    /// Add a memory region to this set.
    pub fn insert(&mut self, region: MemoryRegion<PT::VA>) -> HvResult {
        info!("region.start: {:#X}", region.start.into());
        assert!(is_aligned(region.start.into()));
        assert!(is_aligned(region.size));
        if region.size == 0 {
            return Ok(());
        }
        if !self.test_free_area(&region) {
            warn!(
                "MemoryRegion overlapped in MemorySet: {:#x?}\n{:#x?}",
                region, self
            );
            return hv_result_err!(EINVAL);
        }
        self.pt.map(&region)?;
        self.regions.insert(region.start, region);
        Ok(())
    }

    /// Find and remove memory region which starts from `start`.
    pub fn delete(&mut self, start: PT::VA) -> HvResult {
        if let Entry::Occupied(e) = self.regions.entry(start) {
            self.pt.unmap(e.get())?;
            e.remove();
            Ok(())
        } else {
            hv_result_err!(
                EINVAL,
                format!(
                    "MemorySet::delete(): no memory region starts from {:#x?}",
                    start.into()
                )
            )
        }
    }

    pub fn clear(&mut self) {
        for region in self.regions.values() {
            self.pt.unmap(region).unwrap();
        }
        self.regions.clear();
    }

    pub unsafe fn activate(&self) {
        self.pt.activate();
    }

    pub unsafe fn page_table_query(
        &self,
        vaddr: PT::VA,
    ) -> PagingResult<(PhysAddr, MemFlags, PageSize)> {
        self.pt.query(vaddr)
    }
}

impl<VA: Into<usize> + Copy> Debug for MemoryRegion<VA> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let start = self.start.into();
        f.debug_struct("MemoryRegion")
            .field("vaddr_range", &(start..start + self.size))
            .field("size", &self.size)
            .field("flags", &self.flags)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<PT: GenericPageTable> Debug for MemorySet<PT>
where
    PT::VA: Ord,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("MemorySet")
            .field("regions", &self.regions.values())
            .field("page_table", &core::any::type_name::<PT>())
            .field("page_table_root", &self.pt.root_paddr())
            .finish()
    }
}

impl<PT: GenericPageTable> Drop for MemorySet<PT>
where
    PT::VA: Ord,
{
    fn drop(&mut self) {
        debug!("Drop {:#x?}", self);
        self.clear();
    }
}

pub static PARKING_MEMORY_SET: Once<MemorySet<Stage2PageTable>> = Once::new();

pub static mut PARKING_INST_PAGE: AlignedPage = AlignedPage::new();
