#![allow(dead_code)]

//! Memory management.

use alloc::collections::btree_map::{BTreeMap, Entry};
use core::fmt::{Debug, Formatter, Result};
use spin::Once;

use super::{mapper::Mapper, paging::GenericPageTable, MemFlags};
use super::{AlignedPage, VirtAddr, NUM_TEMPORARY_PAGES, PAGE_SIZE, TEMPORARY_MAPPING_BASE};
use crate::error::HvResult;
use crate::memory::addr::is_aligned;
use crate::memory::paging::{PageSize, PagingResult};
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
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            pt: PT::new(),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            regions: self.regions.clone(),
            pt: self.pt.clone(),
        }
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

    pub fn map_partial(&mut self, mem: &MemoryRegion<PT::VA>) -> HvResult {
        // Todo: Check if the memory area is included in the memory set.
        self.pt.map(mem)
    }

    pub fn unmap_partial(&mut self, mem: &MemoryRegion<PT::VA>) -> HvResult {
        // Todo: Check if the memory area is included in the memory set.
        self.pt.unmap(mem)
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
    /// Map a physical address to a temporary virtual address.
    /// It should only used when access an address in el2 but hypervisor doesn't have the mapping.
    pub fn map_temporary(
        &mut self,
        start_paddr: PhysAddr,
        size: usize,
        flags: MemFlags,
    ) -> HvResult<VirtAddr> {
        if size > NUM_TEMPORARY_PAGES * PAGE_SIZE {
            warn!("Trying to map a too big space in temporary area");
            return hv_result_err!(EINVAL);
        }
        let region: MemoryRegion<PT::VA> = MemoryRegion::new_with_offset_mapper(
            TEMPORARY_MAPPING_BASE.into(),
            start_paddr,
            size,
            flags,
        );
        self.pt.map(&region)?;
        self.regions.insert(region.start, region);
        Ok(TEMPORARY_MAPPING_BASE)
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

// pub static PARKING_MEMORY_SET: Once<MemorySet<Stage2PageTable>> = Once::new();

pub static mut PARKING_INST_PAGE: AlignedPage = AlignedPage::new();
