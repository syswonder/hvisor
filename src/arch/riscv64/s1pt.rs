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
#![allow(unused)]
use bit_field::BitField;
use core::fmt;
use numeric_enum_macro::numeric_enum;
use riscv::register::satp;
use tock_registers::interfaces::Writeable;

use crate::memory::{
    addr::{HostPhysAddr, PhysAddr},
    MemFlags,
};

use super::paging::{GenericPTE, HvPageTable, PagingInstr};
// |Reserved|  PPN  |RSW |Attr|
// |  63-54 | 53-10 |9-8 |7-0 |

bitflags::bitflags! {
    /// Memory attribute fields in the Sv39 translation table format descriptors.
    #[derive(Clone, Copy, Debug)]
    pub struct DescriptorAttr: u64 {
        // Attribute fields in stage 1 Sv39 Block and Page descriptors:

        const VALID =       1 << 0;
        // WHEN R|W|X is 0, this PTE is pointer to next level page table,else Block descriptor
        const READABLE =    1 << 1;
        const WRITABLE =    1 << 2;
        const EXECUTABLE =  1 << 3;
        const USER =        1 << 4;
        const GLOBAL =      1 << 5;
        const ACCESSED =    1 << 6;
        const DIRTY =       1 << 7;
        // RSW fields is bit[8..9]:Reserved for Software

    }
}

impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if !attr.contains(DescriptorAttr::VALID) {
            return flags;
        }
        if attr.contains(DescriptorAttr::READABLE) {
            flags |= Self::READ;
        }
        if attr.contains(DescriptorAttr::WRITABLE) {
            flags |= Self::WRITE;
        }
        if attr.contains(DescriptorAttr::EXECUTABLE) {
            flags |= Self::EXECUTE;
        }
        if attr.contains(DescriptorAttr::USER) {
            flags |= Self::USER;
        }
        flags
    }
}

impl From<MemFlags> for DescriptorAttr {
    fn from(flags: MemFlags) -> Self {
        let mut attr = Self::empty();
        attr |= Self::VALID;
        if flags.contains(MemFlags::READ) {
            attr |= Self::READABLE;
        }
        if flags.contains(MemFlags::WRITE) {
            attr |= Self::WRITABLE;
        }
        if flags.contains(MemFlags::EXECUTE) {
            attr |= Self::EXECUTABLE;
        }
        if flags.contains(MemFlags::USER) {
            attr |= Self::USER;
        }
        attr
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);
const PTE_PPN_MASK: u64 = 0x3F_FFFF_FFFF_FC00; //[10..53]ppn
const PPN_MASK: u64 = 0xFF_FFFF_FFFF_F000; //[12..55]ppn
impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }
}

impl GenericPTE for PageTableEntry {
    fn addr(&self) -> HostPhysAddr {
        PhysAddr::from(((self.0 & PTE_PPN_MASK) << 2) as usize) //[10:53] ppn
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
        DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::READABLE)
            | DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::WRITABLE)
            | DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::EXECUTABLE)
    }

    fn set_addr(&mut self, paddr: HostPhysAddr) {
        self.0 = (self.0.get_bits(0..7)) | ((paddr as u64 & PPN_MASK) >> 2);
    }

    fn set_flags(&mut self, flags: MemFlags) {
        let mut attr: DescriptorAttr = flags.into();
        self.0 = (attr.bits() & !PTE_PPN_MASK as u64) | (self.0 as u64 & PTE_PPN_MASK as u64);
    }

    fn set_table(&mut self, paddr: HostPhysAddr) {
        self.set_addr(paddr);
        let attr = DescriptorAttr::VALID;
        self.0 = (attr.bits() & !PTE_PPN_MASK as u64) | (self.0 as u64 & PTE_PPN_MASK as u64);
    }

    fn clear(&mut self) {
        self.0 = 0
    }
}

impl PageTableEntry {
    fn pt_flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stage1PageTableEntry")
            .field("raw", &self.0)
            .field("paddr", &self.addr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.pt_flags())
            .finish()
    }
}

pub struct S1PTInstr;

impl PagingInstr for S1PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {
        info!("activate hv stage 1 page table");
        unsafe {
            satp::set(satp::Mode::Sv39, 0, root_paddr >> 12);
            //core::arch::asm!("sfence.vma");
        }
    }

    fn flush(_vaddr: Option<usize>) {
        // do nothing
    }
}

pub type Stage1PageTable = HvPageTable<HostPhysAddr, PageTableEntry, S1PTInstr>;
