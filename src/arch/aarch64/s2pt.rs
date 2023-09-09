use core::fmt;
use bitflags::bitflags;
use numeric_enum_macro::numeric_enum;
use aarch64_cpu::registers::VTTBR_EL2;

use crate::memory::addr::{GuestPhysAddr, HostPhysAddr, PhysAddr};
use crate::memory::{GenericPTE, Level4PageTable, MemFlags, PAGE_SIZE, PagingInstr};

bitflags::bitflags! {
    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    #[derive(Clone, Copy, Debug)]
    pub struct DescriptorAttr: u64 {
        // Attribute fields in stage 2 VMSAv8-64 Block and Page descriptors:

        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;
        /// Memory attributes index field.
        const ATTR      =   0b1111 << 2;
        /// Access permission: accessable at EL0/1, Read / Write.
        const S2AP_R      =   1 << 6;
        /// Access permission: accessable at EL0/1, Write.
        const S2AP_W      =   1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER     =   1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
    }
}

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    enum MemType {
        Device = 1,
        Normal = 15,
    }
}

impl DescriptorAttr {
    const ATTR_INDEX_MASK: u64 = 0b1111_00;

    const fn from_mem_type(mem_type: MemType) -> Self {
        let mut bits = (mem_type as u64) << 2;
        if matches!(mem_type, MemType::Normal) {
            bits |= Self::INNER.bits() | Self::SHAREABLE.bits();
        }
        Self::from_bits_truncate(bits)
    }

    fn mem_type(&self) -> MemType {
        let idx = (self.bits() & Self::ATTR_INDEX_MASK) >> 2;
        match idx {
            1 => MemType::Device,
            15 => MemType::Normal,
            _ => panic!("Invalid memory attribute index"),
        }
    }
}

impl MemType {
    fn empty() -> Self {
        Self::try_from(0).unwrap()
    }
}

impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if attr.contains(DescriptorAttr::VALID) && attr.contains(DescriptorAttr::S2AP_R) {
            flags |= Self::READ;
        }
        if attr.contains(DescriptorAttr::S2AP_W) {
            flags |= Self::WRITE;
        }
        if attr.mem_type() == MemType::Device {
            flags |= Self::IO;
        }
        flags
    }
}

impl From<MemFlags> for DescriptorAttr {
    fn from(flags: MemFlags) -> Self {
        let mut attr = if flags.contains(MemFlags::IO) {
            Self::from_mem_type(MemType::Device)
        } else {
            Self::from_mem_type(MemType::Normal)
        };
        attr |= Self::VALID | Self::AF;
        if flags.contains(MemFlags::READ) {
            attr |= Self::S2AP_R;
        }
        if flags.contains(MemFlags::WRITE) {
            attr |= Self::S2AP_W;
        }
        attr
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = 0xffff_ffff_ffff & !(PAGE_SIZE - 1);

    pub const fn empty() -> Self {
        Self(0)
    }
}

impl GenericPTE for PageTableEntry {
    fn addr(&self) -> HostPhysAddr {
        PhysAddr::from(self.0 as usize & Self::PHYS_ADDR_MASK)
    }

    fn flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }

    fn is_unused(&self) -> bool {
        self.0 == 0
    }

    fn is_present(&self) -> bool {
        DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::VALID)
    }

    fn is_huge(&self) -> bool {
        !DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::NON_BLOCK)
    }

    fn set_addr(&mut self, paddr: HostPhysAddr) {
        self.0 = (self.0 & !Self::PHYS_ADDR_MASK as u64) | (paddr as u64 & Self::PHYS_ADDR_MASK as u64);
    }

    fn set_flags(&mut self, flags: MemFlags, is_huge: bool) {
        let mut mem_type:MemType = MemType::Normal;
        if flags.contains(MemFlags::IO) {
            mem_type = MemType::Device;
        }
        let mut flags:DescriptorAttr = flags.into();
        if !is_huge {
            flags |= DescriptorAttr::NON_BLOCK;
        }
        self.set_flags_and_mem_type(flags, mem_type);
    }

    fn set_table(&mut self, paddr: HostPhysAddr) {
        self.set_addr(paddr);
        self.set_flags_and_mem_type(
            DescriptorAttr::VALID | DescriptorAttr::NON_BLOCK,
            MemType::Normal,
        );
    }

    fn clear(&mut self) {
        self.0 = 0
    }
}

impl PageTableEntry {
    fn pt_flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }

    fn memory_type(&self) -> MemType {
        DescriptorAttr::from_bits_truncate(self.0).mem_type()
    }

    fn set_flags_and_mem_type(&mut self, flags: DescriptorAttr, mem_type: MemType) {
        let mut attr = flags | DescriptorAttr::from_mem_type(mem_type);
        self.0 = (attr.bits() & !Self::PHYS_ADDR_MASK as u64) | (self.0 as u64 & Self::PHYS_ADDR_MASK as u64);
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stage2PageTableEntry")
            .field("raw", &self.0)
            .field("paddr", &self.addr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.pt_flags())
            .field("memory_type", &self.memory_type())
            .finish()
    }
}

pub struct S2PTInstr;

impl PagingInstr for S2PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {
        VTTBR_EL2.set_baddr(root_paddr as _);
    }

    fn flush(_vaddr: Option<usize>) {
        // do nothing
    }
}

pub type Stage2PageTable = Level4PageTable<GuestPhysAddr, PageTableEntry, S2PTInstr>;
