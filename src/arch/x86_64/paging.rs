#![allow(unused)]
use crate::error::{HvError, HvResult};
use crate::memory::{Frame, MemFlags, MemoryRegion, PhysAddr, VirtAddr};
use core::{fmt::Debug, marker::PhantomData};

#[derive(Debug)]
pub enum PagingError {
    NoMemory,
    NotMapped,
    AlreadyMapped,
    MappedToHugePage,
}

pub type PagingResult<T = ()> = Result<T, PagingError>;

#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    Size4K = 0x1000,
    Size2M = 0x20_0000,
    Size1G = 0x4000_0000,
}

impl PageSize {
    pub const fn is_aligned(self, addr: usize) -> bool {
        false
    }

    pub const fn align_down(self, addr: usize) -> usize {
        0
    }

    pub const fn page_offset(self, addr: usize) -> usize {
        0
    }

    pub const fn is_huge(self) -> bool {
        false
    }
}

pub trait GenericPTE: Debug + Clone {
    /// Returns the physical address mapped by this entry.
    fn addr(&self) -> PhysAddr;
    /// Returns the flags of this entry.
    fn flags(&self) -> MemFlags;
    /// Returns whether this entry is zero.
    fn is_unused(&self) -> bool;
    /// Returns whether this entry flag indicates present.
    fn is_present(&self) -> bool;
    /// Returns whether this entry maps to a huge frame.
    fn is_huge(&self) -> bool;
    /// Set physical address for terminal entries.
    fn set_addr(&mut self, paddr: PhysAddr);
    /// Set flags for terminal entries.
    fn set_flags(&mut self, flags: MemFlags, is_huge: bool);
    /// Set physical address and flags for intermediate table entries.
    fn set_table(&mut self, paddr: PhysAddr);
    /// Set this entry to zero.
    fn clear(&mut self);
}

pub trait PagingInstr {
    unsafe fn activate(root_paddr: PhysAddr);
    fn flush(vaddr: Option<usize>);
}

/// A basic read-only page table for address query only.
pub trait GenericPageTableImmut: Sized {
    type VA: From<usize> + Into<usize> + Copy;

    unsafe fn from_root(root_paddr: PhysAddr) -> Self;
    fn root_paddr(&self) -> PhysAddr;
    fn query(&self, vaddr: Self::VA) -> PagingResult<(PhysAddr, MemFlags, PageSize)>;
}

/// A extended mutable page table can change mappings.
pub trait GenericPageTable: GenericPageTableImmut {
    fn new() -> Self;

    fn map(&mut self, region: &MemoryRegion<Self::VA>) -> HvResult;
    fn unmap(&mut self, region: &MemoryRegion<Self::VA>) -> HvResult;
    fn update(
        &mut self,
        vaddr: Self::VA,
        paddr: PhysAddr,
        flags: MemFlags,
    ) -> PagingResult<PageSize>;

    fn clone(&self) -> Self;

    unsafe fn activate(&self);
    fn flush(&self, vaddr: Option<Self::VA>);
}

pub struct HvPageTable<VA, PTE: GenericPTE, I: PagingInstr> {
    _phantom: PhantomData<(VA, PTE, I)>,
}

impl<VA, PTE, I> HvPageTable<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    #[allow(dead_code)]
    pub fn dump(&self, limit: usize) {}

    /// Clone only the top level page table mapping from `src`.
    pub fn clone_from(src: &impl GenericPageTableImmut) -> Self {
        Self::new()
    }
}

impl<VA, PTE, I> GenericPageTableImmut for HvPageTable<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    type VA = VA;

    unsafe fn from_root(root_paddr: PhysAddr) -> Self {
        Self::new()
    }

    fn root_paddr(&self) -> PhysAddr {
        0
    }

    fn query(&self, vaddr: VA) -> PagingResult<(PhysAddr, MemFlags, PageSize)> {
        Ok((0, MemFlags::READ, PageSize::Size4K))
    }
}

impl<VA, PTE, I> GenericPageTable for HvPageTable<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    fn new() -> Self {
        Self::new()
    }

    fn map(&mut self, region: &MemoryRegion<VA>) -> HvResult {
        Ok(())
    }

    fn unmap(&mut self, region: &MemoryRegion<VA>) -> HvResult {
        Ok(())
    }

    fn update(&mut self, vaddr: VA, paddr: PhysAddr, flags: MemFlags) -> PagingResult<PageSize> {
        Ok(PageSize::Size4K)
    }

    fn clone(&self) -> Self {
        Self::clone_from(self)
    }

    unsafe fn activate(&self) {}

    fn flush(&self, vaddr: Option<Self::VA>) {}
}
