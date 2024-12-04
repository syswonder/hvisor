use super::paging::{GenericPTE, HvPageTable, PagingInstr};
use crate::memory::addr::{GuestPhysAddr, HostPhysAddr, PhysAddr};
use crate::memory::MemFlags;
use core::fmt;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl GenericPTE for PageTableEntry {
    fn addr(&self) -> HostPhysAddr {
        0
    }

    fn flags(&self) -> MemFlags {
        MemFlags::READ
    }

    fn is_unused(&self) -> bool {
        false
    }

    fn is_present(&self) -> bool {
        false
    }

    fn is_huge(&self) -> bool {
        false
    }

    fn set_addr(&mut self, paddr: HostPhysAddr) {}

    fn set_flags(&mut self, flags: MemFlags, is_huge: bool) {}

    fn set_table(&mut self, paddr: HostPhysAddr) {}

    fn clear(&mut self) {}
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub struct S2PTInstr;

impl PagingInstr for S2PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {}
    fn flush(_vaddr: Option<usize>) {}
}

pub type Stage2PageTable = HvPageTable<GuestPhysAddr, PageTableEntry, S2PTInstr>;
