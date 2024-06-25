use super::paging::PagingInstr;
use crate::arch::paging::GenericPTE;
use crate::arch::paging::Level4PageTable;
use crate::memory::GuestPhysAddr;
use crate::memory::HostPhysAddr;
use crate::memory::MemFlags;
use crate::memory::PhysAddr;
use loongArch64::register::MemoryAccessType;

bitflags::bitflags! {
    /// Memory attribute fields in the LoongArch64 translation table format descriptors.
    #[derive(Clone, Copy, Debug)]
    pub struct DescriptorAttr: u64 {
        const V = 1 << 0; // Valid
        const D = 1 << 1; // Dirty
        const PLV = 0b11 << 2; // Privilege Level
        const MAT = 0b11 << 4; // Memory Access Type
        const G = 1 << 6; // Global
        const P = 1 << 7; // Present
        const W = 1 << 8; // Writable
        const NR = 1 << 61; // Not Readable
        const NX = 1 << 62; // Not Executable
        const RPLV = 1 << 63; // Relative Privilege Level Check
    }
}

impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if attr.contains(DescriptorAttr::W) {
            flags |= Self::WRITE;
        }
        if !attr.contains(DescriptorAttr::NR) {
            flags |= Self::READ;
        }
        if !attr.contains(DescriptorAttr::NX) {
            flags |= Self::EXECUTE;
        }
        flags
    }
}

impl From<MemFlags> for DescriptorAttr {
    fn from(flags: MemFlags) -> Self {
        let mut attr = Self::empty();
        if flags.contains(MemFlags::WRITE) {
            attr |= Self::W;
        }
        if !flags.contains(MemFlags::READ) {
            attr |= Self::NR;
        }
        if !flags.contains(MemFlags::EXECUTE) {
            attr |= Self::NX;
        }
        attr
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

// 12-47
const PTE_PPN_MASK: u64 = 0x0000_ffff_ffff_f000;

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }
}

impl GenericPTE for PageTableEntry {
    fn addr(&self) -> HostPhysAddr {
        PhysAddr::from(self.0 as usize & PTE_PPN_MASK as usize)
    }
    fn flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        todo!()
    }
    fn set_addr(&mut self, addr: HostPhysAddr) {
        todo!()
    }
    fn set_flags(&mut self, flags: MemFlags, is_huge: bool) {
        self.0 = DescriptorAttr::from(flags).bits();
    }
    fn set_table(&mut self, pa: HostPhysAddr) {
        self.set_addr(pa);
        self.set_flags_and_mat(
            DescriptorAttr::V | DescriptorAttr::W,
            MemoryAccessType::CoherentCached,
        );
    }
    fn clear(&mut self) {
        self.0 = 0;
    }
    fn is_huge(&self) -> bool {
        false
    }
}

impl PageTableEntry {
    pub fn set_flags_and_mat(&mut self, flags: DescriptorAttr, mat: MemoryAccessType) {
        self.0 = (self.0 & !DescriptorAttr::MAT.bits()) | flags.bits() | mat as u64;
    }
}

pub struct S2PTInstr;

impl PagingInstr for S2PTInstr {
    unsafe fn activate(root_pa: HostPhysAddr) {
        todo!()
    }
    fn flush(vaddr: Option<usize>) {
        todo!()
    }
}

pub type Stage1PageTable = Level4PageTable<GuestPhysAddr, PageTableEntry, S2PTInstr>;
