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
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{
        iommu,
        paging::{GenericPTE, Level4PageTable, PagingInstr},
        vmcs::*,
    },
    consts::PAGE_SIZE,
    error::HvResult,
    memory::{
        addr::{GuestPhysAddr, HostPhysAddr, PhysAddr},
        MemFlags,
    },
    cpu_data::this_cpu_data,
    zone::this_zone_id,
};
use bit_field::BitField;
use bitflags::bitflags;
use core::{arch::asm, fmt};

bitflags! {
    /// EPT entry flags. (SDM Vol. 3C, Section 28.3.2)
    pub struct DescriptorAttr: u64 {
        /// Read access.
        const READ =                1 << 0;
        /// Write access.
        const WRITE =               1 << 1;
        /// Execute access.
        const EXECUTE =             1 << 2;
        /// EPT memory type. Only for terminate pages.
        const MEM_TYPE_MASK =       0b111 << 3;
        /// Ignore PAT memory type. Only for terminate pages.
        const IGNORE_PAT =          1 << 6;
        /// Specifies that the entry maps a huge frame instead of a page table.
        /// Only allowed in P2 or P3 tables.
        const HUGE_PAGE =           1 << 7;
        /// If bit 6 of EPTP is 1, accessed flag for EPT.
        const ACCESSED =            1 << 8;
        /// If bit 6 of EPTP is 1, dirty flag for EPT.
        const DIRTY =               1 << 9;
        /// Execute access for user-mode linear addresses.
        const EXECUTE_FOR_USER =    1 << 10;
    }
}

/// INVEPT type. (SDM Vol. 3C, Section 30.3)
#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum InvS2PTType {
    /// The logical processor invalidates all mappings associated with bits
    /// 51:12 of the EPT pointer (EPTP) specified in the INVEPT descriptor.
    /// It may invalidate other mappings as well.
    SingleContext = 1,
    /// The logical processor invalidates mappings associated with all EPTPs.
    Global = 2,
}

bitflags! {
    /// Extended-Page-Table Pointer. (SDM Vol. 3C, Section 24.6.11)
    pub struct S2PTPointer: u64 {
        /// EPT paging-structure memory type: Uncacheable (UC).
        #[allow(clippy::identity_op)]
        const MEM_TYPE_UC = 0 << 0;
        /// EPT paging-structure memory type: Write-back (WB).
        #[allow(clippy::identity_op)]
        const MEM_TYPE_WB = 6 << 0;
        /// EPT page-walk length 1.
        const WALK_LENGTH_1 = 0 << 3;
        /// EPT page-walk length 2.
        const WALK_LENGTH_2 = 1 << 3;
        /// EPT page-walk length 3.
        const WALK_LENGTH_3 = 2 << 3;
        /// EPT page-walk length 4.
        const WALK_LENGTH_4 = 3 << 3;
        /// Setting this control to 1 enables accessed and dirty flags for EPT.
        const ENABLE_ACCESSED_DIRTY = 1 << 6;
    }
}

impl S2PTPointer {
    pub fn from_table_phys(root_paddr: HostPhysAddr) -> Self {
        let aligned_addr = root_paddr & !(PAGE_SIZE - 1);
        let flags = unsafe { Self::from_bits_retain(aligned_addr as u64) };
        flags | Self::MEM_TYPE_WB | Self::WALK_LENGTH_4 | Self::ENABLE_ACCESSED_DIRTY
    }
}

numeric_enum_macro::numeric_enum! {
    #[repr(u8)]
    #[derive(Debug, PartialEq, Clone, Copy)]
    /// EPT memory typing. (SDM Vol. 3C, Section 28.3.7)
    enum MemType {
        Uncacheable = 0,
        WriteCombining = 1,
        WriteThrough = 4,
        WriteProtected = 5,
        WriteBack = 6,
    }
}

impl DescriptorAttr {
    fn set_mem_type(&mut self, mem_type: MemType) {
        let mut bits = self.bits();
        bits.set_bits(3..6, mem_type as u64);
        *self = Self::from_bits_truncate(bits)
    }

    fn mem_type(&self) -> Result<MemType, u8> {
        MemType::try_from(self.bits().get_bits(3..6) as u8)
    }
}

impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if attr.contains(DescriptorAttr::READ) {
            flags |= Self::READ;
        }
        if attr.contains(DescriptorAttr::WRITE) {
            flags |= Self::WRITE;
        }
        if attr.contains(DescriptorAttr::EXECUTE) {
            flags |= Self::EXECUTE;
        }
        if let Ok(MemType::Uncacheable) = attr.mem_type() {
            flags |= Self::IO;
        }
        flags
    }
}

impl From<MemFlags> for DescriptorAttr {
    fn from(flags: MemFlags) -> Self {
        if flags.is_empty() {
            return Self::empty();
        }
        let mut attr = Self::empty();
        if flags.contains(MemFlags::READ) {
            attr |= Self::READ;
        }
        if flags.contains(MemFlags::WRITE) {
            attr |= Self::WRITE;
        }
        if flags.contains(MemFlags::EXECUTE) {
            attr |= Self::EXECUTE | Self::EXECUTE_FOR_USER;
        }
        if !flags.contains(MemFlags::IO) {
            attr.set_mem_type(MemType::WriteBack);
        } else {
            attr.set_mem_type(MemType::WriteThrough);
            //  attr &= !Self::READ;
        }
        attr
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = 0x000f_ffff_ffff_f000; // 12..52

    fn memory_type(&self) -> MemType {
        DescriptorAttr::from_bits_truncate(self.0)
            .mem_type()
            .unwrap()
    }
}

impl GenericPTE for PageTableEntry {
    fn addr(&self) -> HostPhysAddr {
        self.0 as usize & Self::PHYS_ADDR_MASK
    }

    fn flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }

    fn is_unused(&self) -> bool {
        self.0 == 0
    }

    fn is_present(&self) -> bool {
        self.0 & 0x7 != 0 // RWX != 0
    }

    fn is_huge(&self) -> bool {
        DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::HUGE_PAGE)
    }

    fn set_addr(&mut self, paddr: HostPhysAddr) {
        self.0 =
            (self.0 & !Self::PHYS_ADDR_MASK as u64) | (paddr as u64 & Self::PHYS_ADDR_MASK as u64);
    }

    fn set_flags(&mut self, flags: MemFlags, is_huge: bool) {
        let mut attr = DescriptorAttr::from(flags);
        if is_huge {
            attr |= DescriptorAttr::HUGE_PAGE;
        }
        self.0 = (attr.bits() & !Self::PHYS_ADDR_MASK as u64)
            | (self.0 as u64 & Self::PHYS_ADDR_MASK as u64);
    }

    fn set_table(&mut self, paddr: HostPhysAddr) {
        let attr = DescriptorAttr::READ | DescriptorAttr::WRITE | DescriptorAttr::EXECUTE;
        self.0 = (attr.bits() & !Self::PHYS_ADDR_MASK as u64)
            | (paddr as u64 & Self::PHYS_ADDR_MASK as u64);
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
            .field("flags", &self.flags())
            .field("memory_type", &self.memory_type())
            .finish()
    }
}

/// Invalidate Translations Derived from EPT. (SDM Vol. 3C, Section 30.3)
///
/// Invalidates mappings in the translation lookaside buffers (TLBs) and
/// paging-structure caches that were derived from extended page tables (EPT).
/// (See Chapter 28, “VMX Support for Address Translation”.) Invalidation is
/// based on the INVEPT type specified in the register operand and the INVEPT
/// descriptor specified in the memory operand.
unsafe fn invs2pt(inv_type: InvS2PTType, s2ptp: u64) {
    let invs2pt_desc = [s2ptp, 0];
    asm!("invept {0}, [{1}]", in(reg) inv_type as u64, in(reg) &invs2pt_desc);
}

pub struct S2PTInstr;

impl PagingInstr for S2PTInstr {
    unsafe fn activate(root_paddr: HostPhysAddr) {
        let s2ptp = S2PTPointer::from_table_phys(root_paddr).bits();
        crate::arch::vmcs::VmcsControl64::EPTP.write(s2ptp).unwrap();
        unsafe { invs2pt(InvS2PTType::SingleContext, s2ptp) };

        // if this cpu is boot cpu and it is running
        if this_cpu_data().arch_cpu.power_on && this_cpu_data().boot_cpu {
            iommu::fill_dma_translation_tables(this_zone_id(), root_paddr);
        }
    }

    fn flush(_vaddr: Option<usize>) {}
}

/// Information about nested page faults.
#[derive(Debug)]
pub struct Stage2PageFaultInfo {
    /// Access type that caused the nested page fault.
    pub access_flags: MemFlags,
    /// Guest physical address that caused the nested page fault.
    pub fault_guest_paddr: GuestPhysAddr,
}

impl Stage2PageFaultInfo {
    pub fn new() -> HvResult<Self> {
        // SDM Vol. 3C, Section 27.2.1, Table 27-7
        let qualification = VmcsReadOnlyNW::EXIT_QUALIFICATION.read()?;
        let fault_guest_paddr = VmcsReadOnly64::GUEST_PHYSICAL_ADDR.read()? as usize;
        let mut access_flags = MemFlags::empty();
        if qualification.get_bit(0) {
            access_flags |= MemFlags::READ;
        }
        if qualification.get_bit(1) {
            access_flags |= MemFlags::WRITE;
        }
        if qualification.get_bit(2) {
            access_flags |= MemFlags::EXECUTE;
        }
        Ok(Stage2PageFaultInfo {
            access_flags,
            fault_guest_paddr,
        })
    }
}

pub type Stage2PageTable = Level4PageTable<GuestPhysAddr, PageTableEntry, S2PTInstr>;

pub fn stage2_mode_detect() {
    info!("Dynamical detection of stage-2 paging mode is not supported yet.");
}
