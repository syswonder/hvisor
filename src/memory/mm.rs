#![allow(dead_code)]

//! Memory management.

use alloc::collections::btree_map::{BTreeMap, Entry};
use core::fmt::{Debug, Formatter, Result};

use super::addr::{align_down, align_up};
use super::{mapper::Mapper, paging::GenericPageTable, MemFlags};
use crate::error::HvResult;

#[derive(Clone)]
pub struct MemoryRegion<VA> {
    pub start: VA,
    pub size: usize,
    pub flags: MemFlags,
    pub(super) mapper: Mapper,
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
        let start = align_down(start.into());
        let size = align_up(size);
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

    pub fn page_table(&self) -> &PT {
        &self.pt
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
