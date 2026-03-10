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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use crate::config::root_zone_config;
use crate::device::irqchip::aia::imsic::imsic_trigger;
use crate::zone::Zone;
use crate::{arch::cpu::ArchCpu, cpu_data::this_cpu_data, memory::GuestPhysAddr};
use riscv_decode::Instruction;
use spin::Once;
use spin::RwLock;

/*
    APLIC Memory Map:

    base + 0x0000 4 bytes domaincfg
    base + 0x0004 4 bytes sourcecfg[1]
    base + 0x0008 4 bytes sourcecfg[2]
    ...
    base + 0x0FFC 4 bytes sourcecfg[1023]
    base + 0x1BC0 4 bytes mmsiaddrcfg (machine-level interrupt domains only)
    base + 0x1BC4 4 bytes mmsiaddrcfgh ”
    base + 0x1BC8 4 bytes smsiaddrcfg ”
    base + 0x1BCC 4 bytes smsiaddrcfgh ”
    base + 0x1C00 4 bytes setip[0]
    base + 0x1C04 4 bytes setip[1]
    ...
    base + 0x1C7C 4 bytes setip[31]
    base + 0x1CDC 4 bytes setipnum
    base + 0x1D00 4 bytes in clrip[0]
    base + 0x1D04 4 bytes in clrip[1]
    ..
    base + 0x1D7C 4 bytes in clrip[31]
    base + 0x1DDC 4 bytes clripnum
    base + 0x1E00 4 bytes setie[0]
    base + 0x1E04 4 bytes setie[1]
    ...
    base + 0x1E7C 4 bytes setie[31]
    base + 0x1EDC 4 bytes setienum
    base + 0x1F00 4 bytes clrie[0]
    base + 0x1F04 4 bytes clrie[1]
    ..
    base + 0x1F7C 4 bytes clrie[31]
    base + 0x1FDC 4 bytes clrienum
    base + 0x2000 4 bytes setipnum le
    base + 0x2004 4 bytes setipnum be
    base + 0x3000 4 bytes genmsi
    base + 0x3004 4 bytes target[1]
    base + 0x3008 4 bytes target[2]
    ...
    base + 0x3FFC 4 bytes target[1023]
*/

// S-mode Interrupt Domain
pub const APLIC_DOMAINCFG_BASE: usize = 0x0000;
pub const APLIC_SOURCECFG_BASE: usize = 0x0004; // [APLIC_SOURCECFG_BASE, APLIC_SOURCECFG_TOP)
pub const APLIC_SOURCECFG_TOP: usize = 0x1000;
pub const APLIC_MSIADDR_BASE: usize = 0x1BC8; // smsiaddrcfg
pub const APLIC_SETIP_BASE: usize = 0x1C00; // [APLIC_SETIP_BASE, APLIC_SETIP_TOP)
pub const APLIC_SETIP_TOP: usize = 0x1C80;
pub const APLIC_SETIPNUM_BASE: usize = 0x1CDC;
pub const APLIC_CLRIP_BASE: usize = 0x1D00; // [APLIC_CLRIP_BASE, APLIC_CLRIP_TOP)
pub const APLIC_CLRIP_TOP: usize = 0x1D80;
pub const APLIC_CLRIPNUM_BASE: usize = 0x1DDC;
pub const APLIC_SETIE_BASE: usize = 0x1E00; // [APLIC_SETIE_BASE, APLIC_SETIE_TOP)
pub const APLIC_SETIE_TOP: usize = 0x1E80;
pub const APLIC_SETIENUM_BASE: usize = 0x1EDC;
pub const APLIC_CLRIE_BASE: usize = 0x1F00; // [APLIC_CLRIE_BASE, APLIC_CLRIE_TOP)
pub const APLIC_CLRIE_TOP: usize = 0x1F80;
pub const APLIC_CLRIENUM_BASE: usize = 0x1FDC;
pub const APLIC_SETIPNUM_LE_BASE: usize = 0x2000;
pub const APLIC_SETIPNUM_BE_BASE: usize = 0x2004;
pub const APLIC_GENMSI_BASE: usize = 0x3000;
pub const APLIC_TARGET_BASE: usize = 0x3004; // [APLIC_TARGET_BASE, APLIC_TARGET_TOP)
pub const APLIC_TARGET_TOP: usize = 0x4000;

#[allow(dead_code)]
pub enum DeliveryMode {
    Direct = 0,
    MSI = 1,
}

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum SourceMode {
    Inactive = 0,
    Detached = 1,
    RisingEdge = 4,
    FallingEdge = 5,
    LevelHigh = 6,
    LevelLow = 7,
}

#[repr(C)]
pub struct Aplic {
    pub base: usize,
}

#[allow(dead_code)]
impl Aplic {
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    /// Aplic init
    pub fn init_global(&self, hart_id: usize, max_interrupts: usize) {
        // Note: for riscv-aia, hvisor supports msi-mode, but doesn't support direct-mode
        self.set_domaincfg(false, false, false);

        // Clear all pending and enabled bits
        for i in 0..((max_interrupts + 31) / 32) {
            self.set_setip(i, 0);
            self.set_setie(i, 0);
        }

        // Sets the default value of target and sourcecfg
        for i in 1..=max_interrupts {
            // [1, 1023]
            // Set irq inactive
            self.set_sourcecfg(i as u32, SourceMode::Inactive);
            // Set target to current hart
            self.set_target_msi(i as u32, hart_id as u32, 0, i as u32);
        }

        // Set little endian, msi-mode, global irq enabled
        self.set_domaincfg(false, true, true);
    }

    /// Set domain configuration
    pub fn set_domaincfg(&self, bigendian: bool, msimode: bool, enabled: bool) {
        let enabled = u32::from(enabled);
        let msimode = u32::from(msimode);
        let bigendian = u32::from(bigendian);
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        let src = (enabled << 8) | (msimode << 2) | bigendian;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Get domain configuration
    pub fn get_domaincfg(&self) -> u32 {
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Get bigendian from domain configuration
    pub fn get_bigendian(&self) -> bool {
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        unsafe { core::ptr::read_volatile(addr as *const u32) & 0b1 != 0 }
    }

    /// Get MSI_Mode from domain configuration
    pub fn get_msimode(&self) -> bool {
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        unsafe { core::ptr::read_volatile(addr as *const u32) & 0b100 != 0 }
    }

    /// Set sourcecfg, interrupt source type
    pub fn set_sourcecfg(&self, irq: u32, mode: SourceMode) {
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = mode as u32;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Set sourcecfg, delegate
    pub fn set_sourcecfg_delegate(&self, irq: u32, child: u32) {
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = 1 << 10 | child & 0x3ff;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Get sourcecfg
    pub fn get_sourcecfg(&self, irq: u32) -> u32 {
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Set msiaddr, smsiaddrcfg & smsiaddrcfgh
    pub fn set_msiaddr(&self, address: usize) {
        let addr = self.base + APLIC_MSIADDR_BASE;
        let src_lo = (address >> 12) as u32;
        let src_hi = (address >> 44) as u32;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src_lo);
            core::ptr::write_volatile((addr + 4) as *mut u32, src_hi);
        }
    }

    /// Get msiaddr, smsiaddrcfg & smsiaddrcfgh
    pub fn get_msiaddr(&self) -> usize {
        let addr = self.base + APLIC_MSIADDR_BASE;
        let src_lo = unsafe { core::ptr::read_volatile(addr as *const u32) } as usize;
        let src_hi = unsafe { core::ptr::read_volatile((addr + 4) as *const u32) } as usize;
        (src_hi << 44) | (src_lo << 12)
    }

    /// Get setip register
    pub fn get_setip(&self, irqidx: usize) -> u32 {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_SETIP_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Set setip register
    pub fn set_setip(&self, irqidx: usize, src: u32) {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_SETIP_BASE + irqidx * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Set setipnum register
    pub fn set_setipnum(&self, value: u32) {
        assert!(value > 0 && value < 1024);
        let addr = self.base + APLIC_SETIPNUM_BASE;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Set setipnum register
    pub fn set_setipnum_le(&self, value: u32) {
        assert!(value > 0 && value < 1024);
        let addr = self.base + APLIC_SETIPNUM_LE_BASE;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Get in_clrip register
    pub fn get_in_clrip(&self, irqidx: usize) -> u32 {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Set in_clrip register
    pub fn set_in_clrip(&self, irqidx: usize, src: u32) {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Set clripnum register
    pub fn set_clripnum(&self, value: u32) {
        assert!(value > 0 && value < 1024);
        let addr = self.base + APLIC_CLRIPNUM_BASE;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Get setie register
    pub fn get_setie(&self, irqidx: usize) -> u32 {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_SETIE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Set setie register
    pub fn set_setie(&self, irqidx: usize, value: u32) {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_SETIE_BASE + irqidx * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Set setienum register
    pub fn set_setienum(&self, value: u32) {
        assert!(value > 0 && value < 1024);
        let addr = self.base + APLIC_SETIENUM_BASE;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Get clrie register
    pub fn get_clrie(&self, irqidx: usize) -> u32 {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }

    /// Set clrie register
    pub fn set_clrie(&self, irqidx: usize, value: u32) {
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Set clrienum register
    pub fn set_clrienum(&self, value: u32) {
        assert!(value > 0 && value < 1024);
        let addr = self.base + APLIC_CLRIENUM_BASE;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Set setipnum_le register
    pub fn setipnum_le(&self, value: u32) {
        assert!(value > 0 && value < 1024);
        let addr = self.base + APLIC_SETIPNUM_LE_BASE;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }

    /// Set target, msi-mode
    pub fn set_target_msi(&self, irq: u32, hart: u32, guest: u32, eiid: u32) {
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src = ((hart & 0x3FFF) << 18) | ((guest & 0x3F) << 12) | (eiid & 0x3FF);
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Set target, direct-mode
    pub fn set_target_direct(&self, irq: u32, hart: u32, prio: u32) {
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src = ((hart & 0x3FFF) << 18) | (prio & 0xFF);
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }

    /// Get target register
    pub fn get_target(&self, irq: u32) -> u32 {
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
}
