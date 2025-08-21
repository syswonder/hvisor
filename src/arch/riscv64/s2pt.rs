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
use super::{
    csr::{read_csr, write_csr, CSR_HGATP},
    paging::{GenericPTE, HvPageTable, PagingInstr},
};
use bit_field::BitField;
use core::fmt;
use numeric_enum_macro::numeric_enum;
use tock_registers::interfaces::Writeable;

use crate::memory::{
    addr::{GuestPhysAddr, HostPhysAddr, PhysAddr},
    MemFlags,
};
// |Reserved|  PPN  |RSW |Attr|
// |  63-54 | 53-10 |9-8 |7-0 |

pub const HGATP_MODE_SHIFT: usize = 60;
pub const HGATP_MODE_SV57X4: usize = 10;
pub const HGATP_MODE_SV48X4: usize = 9;
pub const HGATP_MODE_SV39X4: usize = 8;

// Default G-stage mode is Sv39x4.
static mut GSTAGE_MODE: usize = HGATP_MODE_SV39X4;
pub static mut GSTAGE_PT_LEVEL: usize = 3;

pub fn riscv_gstage_mode_detect() {
    info!("Detecting stage-2 paging mode...");
    unsafe {
        // Attempt to set the HGATP.mode to Sv57x4 first.
        let mode_sv57x4 = HGATP_MODE_SV57X4 << HGATP_MODE_SHIFT;
        write_csr!(CSR_HGATP, mode_sv57x4);
        if (read_csr!(CSR_HGATP) >> HGATP_MODE_SHIFT) == HGATP_MODE_SV57X4 {
            GSTAGE_MODE = HGATP_MODE_SV57X4;
            GSTAGE_PT_LEVEL = 5;
            info!("Sv57x4 is supported.");
            write_csr!(CSR_HGATP, 0);
            hfence_gvma_all();
            return;
        }
        // Attemp to set the HGATP.mode to Sv48x4.
        let mode_sv48x4 = HGATP_MODE_SV48X4 << HGATP_MODE_SHIFT;
        write_csr!(CSR_HGATP, mode_sv48x4);
        if (read_csr!(CSR_HGATP) >> HGATP_MODE_SHIFT) == HGATP_MODE_SV48X4 {
            GSTAGE_MODE = HGATP_MODE_SV48X4;
            GSTAGE_PT_LEVEL = 4;
            info!("Sv48x4 is supported.");
        }
        info!("Default stage-2 paging mode is Sv39x4.");
        write_csr!(CSR_HGATP, 0);
        hfence_gvma_all();
    }
}

#[inline(always)]
unsafe fn hfence_gvma_all() {
    core::arch::asm!("hfence.gvma");
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct DescriptorAttr: u64 {
        // In risc-v, bits[0..=7] is base attribute bits in PTE.
        const VALID =       1 << 0;
        // WHEN R|W|X is 0, this PTE is pointer to next level page table,else Block descriptor
        const READABLE =    1 << 1;
        const WRITABLE =    1 << 2;
        const EXECUTABLE =  1 << 3;
        const USER =        1 << 4;
        const GLOBAL =      1 << 5;
        const ACCESSED =    1 << 6;
        const DIRTY =       1 << 7;
        // RSW fields is bit[8..=9]:Reserved for Software
        const RSW =         0b11 << 8;
        // Reserved bits[54..=60]
        const RESERVED =    0b111_1111 << 54;
        // Svpbmt extension defines PBMT bitfields (Page-Based Memory Types) bits[61..=62]
        const NOCACHE =     1 << 61; // Non-cacheable, idempotent, weakly-ordered (RVWMO), main memory
        const IO =          1 << 62; // Non-cacheable, non-idempotent, strongly-ordered (I/O ordering), I/O
        // Svnapot extension defines NAPOT bitfield (Naturally Aligned Power Of Two) bits[63]
        const NAPOT =       1 << 63;
        // Combined flags for PTE_HV_FLAGS (Accessed, Dirty, Global)
        const PTE_HV_FLAGS = Self::ACCESSED.bits() | Self::DIRTY.bits() | Self::GLOBAL.bits();
        // Combined flags for PTE_VM_FLAGS (Accessed, Dirty, User)
        const PTE_VM_FLAGS = Self::ACCESSED.bits() | Self::DIRTY.bits() | Self::USER.bits();
        // Combined flags for leaf descriptor (Read, Write, Execute)
        const PTE_LEAF_FLAGS = Self::READABLE.bits() | Self::WRITABLE.bits() | Self::EXECUTABLE.bits();
    }
}

impl From<DescriptorAttr> for MemFlags {
    // Convert DescriptorAttr(arch-related) to MemFlags(abstract)
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
    // Convert MemFlags(abstract) to DescriptorAttr(arch-related)
    fn from(flags: MemFlags) -> Self {
        let mut attr = Self::empty();
        attr |= Self::VALID | Self::PTE_VM_FLAGS; // Set the valid bit and PTE_VM_FLAGS
        if flags.contains(MemFlags::READ) {
            attr |= Self::READABLE;
        }
        if flags.contains(MemFlags::WRITE) {
            attr |= Self::WRITABLE;
        }
        if flags.contains(MemFlags::EXECUTE) {
            attr |= Self::EXECUTABLE;
        }
        attr
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);
const PTE_PPN_MASK: u64 = 0x3F_FFFF_FFFF_FC00; //[10..53]ppn
const PA_PPN_MASK: u64 = 0xFF_FFFF_FFFF_F000; //[12..55]ppn
impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }
    // Get the page table entry flags as MemFlags
    fn pt_flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
}

impl GenericPTE for PageTableEntry {
    // Get physical address embedded in the page table entry.
    fn addr(&self) -> HostPhysAddr {
        PhysAddr::from(((self.0 & PTE_PPN_MASK) << 2) as usize) //[10:53] ppn
    }

    // Get the page table entry flags as MemFlags
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
        // Note: It is not possible to determine huge pages only by flags.
        // But one huge page entry must be a leaf node.
        DescriptorAttr::from_bits_truncate(self.0).intersects(DescriptorAttr::PTE_LEAF_FLAGS)
    }

    // Set the physical address embedded in the page table entry.
    fn set_addr(&mut self, paddr: HostPhysAddr) {
        // In RISC-V PTE, the physical address is stored in bits [10:53] of the entry.
        self.0 = (self.0 & !PTE_PPN_MASK) | ((paddr as u64 & PA_PPN_MASK) >> 2);
    }

    // Set the flags embedded in the page table entry.
    fn set_flags(&mut self, flags: MemFlags) {
        let mut attr: DescriptorAttr = flags.into();
        self.0 = (attr.bits() & !PTE_PPN_MASK) | (self.0 & PTE_PPN_MASK);
    }

    // Set the page table entry as a table entry.
    fn set_table(&mut self, paddr: HostPhysAddr) {
        self.set_addr(paddr);
        let attr = DescriptorAttr::VALID;
        self.0 = (attr.bits() & !PTE_PPN_MASK) | (self.0 & PTE_PPN_MASK);
    }

    fn clear(&mut self) {
        self.0 = 0
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stage2PageTableEntry")
            .field("raw", &self.0)
            .field("paddr", &self.addr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.pt_flags())
            .finish()
    }
}

pub struct S2PTInstr;

impl PagingInstr for S2PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {
        info!("guest stage2 PT activate, gstage_mode: {:#x?}", GSTAGE_MODE);
        match GSTAGE_MODE {
            HGATP_MODE_SV57X4 => {
                info!("Using Sv57x4 mode.");
            }
            HGATP_MODE_SV48X4 => {
                info!("Using Sv48x4 mode.");
            }
            _ => {
                info!("Using default Sv39x4 mode.");
            }
        }
        unsafe {
            let mut bits: usize = 0;
            let mode: usize = GSTAGE_MODE; // GSTAGE_MODE is dynamically detected.
            let vmid: usize = 0; // Default VMID is 0, in hvisor one CPU only supports one VMID.
            bits.set_bits(60..64, mode as usize); // Set the HGATP mode.
            bits.set_bits(44..58, vmid); // Set the VMID.
            bits.set_bits(0..44, root_paddr >> 12); // Set the root page table's PPN.
            write_csr!(CSR_HGATP, bits);
            // info!("flush TLB: hfence.gvma, hfence.vvma");
            let hgatp: usize = read_csr!(CSR_HGATP);
            info!("HGATP after activation: {:#x?}", hgatp);
            // hfence_gvma_all();
            riscv_h::asm::hfence_gvma(0, 0); // Flush all stage-2 TLB entries.
        }
    }

    fn flush(_vaddr: Option<usize>) {
        warn!("S2PTInstr::flush is not implemented.");
    }
}

// Here HvPageTable means Hvisor's Abstract Page Table, which is used for stage-2 paging & stage-1 paging.
// Stage2PageTable is a specific type used for GPA->HPA.
pub type Stage2PageTable = HvPageTable<GuestPhysAddr, PageTableEntry, S2PTInstr>;
