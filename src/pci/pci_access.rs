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
// #![allow(dead_code)]
use alloc::string::String;
use bit_field::BitField;
use bitflags::bitflags;
use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
    slice,
};

use super::{
    config_accessors::{PciConfigMmio, PciRegion},
    pci_struct::ArcRwLockVirtualPciConfigSpace,
    PciConfigAddress,
};

use crate::{
    error::HvResult,
    memory::{
        GuestPhysAddr, HostPhysAddr, MMIOAccess, MemFlags, MemoryRegion, MemorySet, mmio_perform_access
    },
    pci::{pci_config::GLOBAL_PCIE_LIST, pci_struct::BIT_LENTH},
    percpu::this_zone,
    zone::{is_this_root_zone, this_zone_id},
};

#[cfg(feature = "dwc_pcie")]
use crate::pci::config_accessors::{
    dwc_atu::{
        AtuConfig, AtuType, ATU_BASE, ATU_REGION_SIZE,
        PCIE_ATU_UNR_REGION_CTRL1, PCIE_ATU_UNR_REGION_CTRL2,
        PCIE_ATU_UNR_LOWER_BASE, PCIE_ATU_UNR_UPPER_BASE,
        PCIE_ATU_UNR_LIMIT, PCIE_ATU_UNR_UPPER_LIMIT,
        PCIE_ATU_UNR_LOWER_TARGET, PCIE_ATU_UNR_UPPER_TARGET,
        ATU_ENABLE_BIT,
        AtuUnroll,
    },
    dwc::DwcConfigRegionBackend,
    PciRegionMmio,
};

use crate::pci::vpci_dev::VpciDevType;

pub type VendorId = u16;
pub type DeviceId = u16;
pub type DeviceRevision = u8;
pub type BaseClass = u8;
pub type SubClass = u8;
pub type Interface = u8;
pub type SubsystemId = u16;
pub type SubsystemVendorId = u16;
pub type InterruptLine = u8;
pub type InterruptPin = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderType {
    Endpoint,
    PciBridge,
    CardBusBridge,
    Unknown(u8),
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PciStatus: u16 {
        const DETECTED_PARITY_ERROR   = 1 << 15;
        const SIGNALED_SYSTEM_ERROR   = 1 << 14;
        const RECEIVED_MASTER_ABORT   = 1 << 13;
        const RECEIVED_TARGET_ABORT   = 1 << 12;
        const SIGNALED_TARGET_ABORT   = 1 << 11;
        const DEVSEL_MASK             = 0b11 << 9;
        const MASTER_PARITY_ERROR     = 1 << 8;
        const FAST_BACK_TO_BACK       = 1 << 7;
        // resersed bit 6
        const CAP_66MHZ               = 1 << 5;
        const CAPABILITIES_LIST       = 1 << 4;
        const INTERRUPT_STATUS        = 1 << 3;
        // resersed bit 0-2
        const _ = !0;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PciCommand: u16 {
        const IO_ENABLE = 1 << 0;
        const MEMORY_ENABLE = 1 << 1;
        const BUS_MASTER_ENABLE = 1 << 2;
        const SPECIAL_CYCLE_ENABLE = 1 << 3;
        const MEMORY_WRITE_AND_INVALIDATE = 1 << 4;
        const VGA_PALETTE_SNOOP = 1 << 5;
        const PARITY_ERROR_RESPONSE = 1 << 6;
        const IDSEL_STEP_WAIT_CYCLE_CONTROL = 1 << 7;
        const SERR_ENABLE = 1 << 8;
        const FAST_BACK_TO_BACK_ENABLE = 1 << 9;
        const INTERRUPT_DISABLE = 1 << 10;
        const _ = !0;
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum PciMemType {
    Mem32,
    Mem64High,
    Mem64Low,
    Io,
    Rom,
    #[default]
    Unused,
}

impl Debug for PciMemType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PciMemType::Mem32 => write!(f, "Mem32"),
            PciMemType::Mem64High => write!(f, "Mem64High"),
            PciMemType::Mem64Low => write!(f, "Mem64Low"),
            PciMemType::Io => write!(f, "IO"),
            PciMemType::Unused => write!(f, "Unused"),
            PciMemType::Rom => write!(f, "Rom"),
        }
    }
}

/* PciMem
 * virtaul_value: the vaddr guset zone can rw, same with as the corresponding value in virtualconfigspace.space
 * value: the paddr which hvisor and hw can rw, init when hvisor init the pci bus
 * size: the size of mem region, when size_read is true return !(size - 1)
 * size_read: if software write 0xffff_ffff to bar, size_read will set so next time hvisor can rerturn !(size - 1) indicating size to the software
 */
#[derive(Default, Clone, Copy)]
pub struct PciMem {
    bar_type: PciMemType,
    virtual_value: u64,
    value: u64,
    size: u64,
    prefetchable: bool,
    size_read: bool,
}

impl PciMem {
    pub fn new_bar(bar_type: PciMemType, value: u64, size: u64, prefetchable: bool) -> Self {
        Self {
            bar_type,
            virtual_value: 0,
            value,
            size,
            prefetchable,
            size_read: false,
        }
    }

    pub fn new_io(value: u64, size: u64) -> Self {
        Self {
            bar_type: PciMemType::Io,
            virtual_value: 0,
            value,
            size,
            prefetchable: false,
            size_read: false,
        }
    }

    pub fn init(value: u64, size: u64) -> Self {
        Self {
            bar_type: PciMemType::Unused,
            virtual_value: 0,
            value,
            size,
            prefetchable: false,
            size_read: false,
        }
    }

    pub fn new_rom(value: u64, size: u64) -> Self {
        Self {
            bar_type: PciMemType::Rom,
            virtual_value: 0,
            value,
            size,
            prefetchable: false,
            size_read: false,
        }
    }

    pub fn set_size(&mut self, size: u64){
        self.size = size;
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_size_with_flag(&self) -> u64 {
        match self.bar_type {
            PciMemType::Mem32 | PciMemType::Rom | PciMemType::Io => !(self.size - 1u64),
            PciMemType::Mem64Low => {
                let bar_size = !(self.size - 1);
                bar_size.get_bits(0..32)
            }
            PciMemType::Mem64High => {
                let bar_size = !(self.size - 1);
                bar_size.get_bits(32..64) >> 32
            }
            PciMemType::Unused => {
                /* for unused bar, size is 0
                 */
                0
            }
            _ => {
                warn!("{:#?} not support size", self.bar_type);
                0
            }
        }
    }

    pub fn set_bar_type(&mut self, bar_type: PciMemType){
        self.bar_type = bar_type;
    }

    pub fn set_prefetchable(&mut self, prefetchable: bool){
        self.prefetchable = prefetchable;
    }

    pub fn config_init(&mut self, bar_type: PciMemType, prefetchable: bool, size: u64, value: u64) {
        self.set_bar_type(bar_type);
        self.set_prefetchable(prefetchable);
        self.set_size(size);
        self.set_value(value);
    }

    pub fn is_enabled(&self) -> bool {
        if self.bar_type == PciMemType::default() {
            false
        } else {
            true
        }
    }

    /* the longest of mmio read is 32 */
    pub fn get_value(&self) -> u32 {
        match self.bar_type {
            PciMemType::Mem64High => (self.value >> 32) as u32,
            _ => self.value as u32,
        }
    }

    /* when update map of bar region,
     * need to read u64 to get whole address
     * the virtual_value is same with value
     */
    pub fn get_value64(&self) -> u64 {
        self.value as u64
    }

    /* Automatically add flags */
    pub fn set_value(&mut self, address: u64) {
        let mut val = address;

        match self.bar_type {
            PciMemType::Io => {
                // bit0 = 1
                val |= 0x1;
            }
            PciMemType::Mem32 => {
                // bit1..2 = 00
                val &= !0x6;
                if self.prefetchable {
                    val |= 0x8;
                }
            }
            PciMemType::Mem64Low | PciMemType::Mem64High => {
                // bit1..=2 = 0b10
                val &= !0x6;
                val |= 0x4;
                if self.prefetchable {
                    val |= 0x8;
                }
            }
            _ => {
                warn!("please init bar first");
            }
        }

        self.value = val;
    }

    pub fn get_type(&self) -> PciMemType {
        self.bar_type
    }

    pub fn get_prefetchable(&self) -> bool {
        self.prefetchable
    }

    pub fn set_size_read(&mut self) {
        self.size_read = true;
    }

    pub fn clear_size_read(&mut self) {
        self.size_read = false;
    }

    pub fn get_size_read(&self) -> bool {
        self.size_read
    }

    pub fn get_virtual_value(&self) -> u32 {
        match self.bar_type {
            PciMemType::Mem64High => (self.virtual_value >> 32) as u32,
            _ => self.virtual_value as u32,
        }
    }

    pub fn get_virtual_value64(&self) -> u64 {
        self.virtual_value
    }

    pub fn set_virtual_value(&mut self, address: u64) {
        let mut val = address;

        match self.bar_type {
            PciMemType::Io => {
                // bit0 = 1
                val |= 0x1;
            }
            PciMemType::Mem32 => {
                // bit1..2 = 00
                val &= !0x6;
                if self.prefetchable {
                    val |= 0x8;
                }
            }
            PciMemType::Mem64Low | PciMemType::Mem64High => {
                // bit1..=2 = 0b10
                val &= !0x6;
                val |= 0x4;
                if self.prefetchable {
                    val |= 0x8;
                }
            }
            _ => {
                warn!("unkown bar type: {:#?}", self.bar_type);
            }
        }

        self.virtual_value = val;
    }
}

impl Debug for PciMem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.bar_type {
            PciMemType::Mem32 => {
                let pre = if self.prefetchable { "pre" } else { "" };
                let paddr = self.value & !0xf;
                let vaddr = self.virtual_value & !0xf;
                let size = self.size;
                write!(
                    f,
                    "[{:#?} 0x{:x}-0x{:x} {}] => [0x{:x}-0x{:x}]",
                    self.bar_type,
                    paddr,
                    paddr + size,
                    pre,
                    vaddr,
                    vaddr + size
                )
            }
            PciMemType::Mem64Low | PciMemType::Mem64High => {
                let pre = if self.prefetchable { "pre" } else { "" };
                let paddr = self.value & !0xf;
                let vaddr = self.virtual_value & !0xf;
                let size = self.size;
                write!(
                    f,
                    "[{:#?} 0x{:x} size 0x{:x} 64bit {}] => [0x{:x}-0x{:x}]",
                    self.bar_type,
                    paddr,
                    paddr + size,
                    pre,
                    vaddr,
                    vaddr + size
                )
            }
            PciMemType::Rom => {
                let paddr = self.value & !0xf;
                let vaddr = self.virtual_value & !0xf;
                let size = self.size;
                write!(
                    f,
                    "{:#?} [0x{:x}-0x{:x}] => [0x{:x}-0x{:x}]",
                    self.bar_type,
                    paddr,
                    paddr + size,
                    vaddr,
                    vaddr + size
                )
            }
            _ => {
                write!(f, "[{:#?}]", self.bar_type)
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Bar {
    bararr: [PciMem; 6],
}

impl Index<usize> for Bar {
    type Output = PciMem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bararr[index]
    }
}

impl IndexMut<usize> for Bar {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bararr[index]
    }
}

impl<'a> IntoIterator for &'a Bar {
    type Item = &'a PciMem;
    type IntoIter = slice::Iter<'a, PciMem>;

    fn into_iter(self) -> Self::IntoIter {
        self.bararr.iter()
    }
}

impl<'a> IntoIterator for &'a mut Bar {
    type Item = &'a mut PciMem;
    type IntoIter = slice::IterMut<'a, PciMem>;

    fn into_iter(self) -> Self::IntoIter {
        self.bararr.iter_mut()
    }
}

impl Debug for Bar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BARs [")?;
        let mut i = 0;
        let mut is_null = true;
        while i < self.bararr.len() {
            let bar = &self.bararr[i];
            let address = bar.value & !0xf;
            // let address = bar.value;
            match bar.bar_type {
                PciMemType::Mem32 => {
                    is_null = false;
                    write!(
                        f,
                        "\n   slot {} [mem 0x{:x}-0x{:x}",
                        i,
                        address,
                        address + bar.size
                    )?;
                    if bar.prefetchable {
                        write!(f, " pre")?;
                    }
                    write!(f, "]")?;
                }
                PciMemType::Mem64Low => {
                    is_null = false;
                    write!(
                        f,
                        "\n   slot {} [mem 0x{:x}-0x{:x} 64bit",
                        i,
                        address,
                        address + bar.size
                    )?;
                    if bar.prefetchable {
                        write!(f, " pre")?;
                    }
                    write!(f, "]")?;
                    i += 1;
                }
                PciMemType::Io => {
                    writeln!(f, "   IO @ 0x{:x}", bar.value)?;
                }
                _ => {}
            }
            i += 1;
        }
        if is_null {
            writeln!(f, "]")
        } else {
            write!(f, "\n]")
        }
    }
}

pub trait PciRWBase: Debug + Send + Sync {
    fn backend(&self) -> &dyn PciRegion;
}

pub trait PciRW: Debug + Send + Sync + PciRWBase {
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
        match size {
            1 => self.backend().read_u8(offset).map(|v| v as usize),
            2 => self.backend().read_u16(offset).map(|v| v as usize),
            4 => self.backend().read_u32(offset).map(|v| v as usize),
            _ => {
                hv_result_err!(EFAULT, "pci: invalid mmio read size: {size}")
            }
        }
    }
    fn write(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        match size {
            1 => self.backend().write_u8(offset, value as u8),
            2 => self.backend().write_u16(offset, value as u16),
            4 => self.backend().write_u32(offset, value as u32),
            _ => {
                hv_result_err!(EFAULT, "pci: invalid mmio write size: {size}")
            }
        }
    }
}

pub trait PciHeaderRW: PciRWBase {
    fn id(&self) -> (DeviceId, VendorId) {
        let id = self.backend().read_u32(0x00).unwrap();
        (
            id.get_bits(0..16) as VendorId,
            id.get_bits(16..32) as DeviceId,
        )
    }

    fn header_type(&self) -> HeaderType {
        match self.backend().read_u8(0x0e).unwrap().get_bits(0..7) {
            0x00 => HeaderType::Endpoint,
            0x01 => HeaderType::PciBridge,
            0x02 => HeaderType::CardBusBridge,
            v => HeaderType::Unknown(v as u8),
        }
    }

    fn has_multiple_functions(&self) -> bool {
        self.backend().read_u8(0x0e).unwrap().get_bit(7)
    }

    fn revision_and_class(&self) -> (BaseClass, SubClass, Interface, DeviceRevision) {
        let value = self.backend().read_u32(0x08).unwrap();
        (
            value.get_bits(24..32) as BaseClass,
            value.get_bits(16..24) as SubClass,
            value.get_bits(8..16) as Interface,
            value.get_bits(0..8) as DeviceRevision,
        )
    }

    fn status(&self) -> PciStatus {
        let status = self.backend().read_u16(0x06).unwrap();
        PciStatus::from_bits_truncate(status)
    }

    fn command(&self) -> PciCommand {
        let command = self.backend().read_u16(0x04).unwrap();
        PciCommand::from_bits_truncate(command)
    }

    fn update_command<F>(&mut self, f: F)
    where
        F: FnOnce(PciCommand) -> PciCommand,
    {
        let mut data = self.backend().read_u16(0x04).unwrap();
        let new_command = f(PciCommand::from_bits_retain(data.get_bits(0..16)));
        data.set_bits(0..16, new_command.bits());
        let _ = self.backend().write_u16(0x04, data);
    }
}

pub trait PciBarRW: PciRWBase {
    fn bar_limit(&self) -> u8;

    fn parse_bar(&self) -> Bar {
        let mut bararr = Bar::default();

        let mut slot = 0u8;
        while slot < self.bar_limit() {
            warn!("parse bar slot {}", slot);
            let value = self.read_bar(slot).unwrap();

            if !value.get_bit(0) {
                let pre = value.get_bit(3);

                match value.get_bits(1..3) {
                    0b00 => {
                        // 32-bit memory space
                        let size = {
                            let _ = self.write_bar(slot, 0xffffffff);
                            let mut readback = self.read_bar(slot).unwrap();
                            let _ = self.write_bar(slot, value as u32);

                            if readback == 0x0 {
                                // bar is null
                                slot += 1;
                                continue;
                            }
                            readback.set_bits(0..4, 0);
                            1 << readback.trailing_zeros()
                        };
                        bararr[slot as usize] =
                            PciMem::new_bar(PciMemType::Mem32, value as u64, size as u64, pre);
                    }
                    0b10 => {
                        // 64-bit memory space
                        if slot == 5 {
                            warn!("read bar64 in last bar");
                            break;
                        }

                        let value_high = self.read_bar(slot + 1).unwrap();
                        let size = {
                            let _ = self.write_bar(slot, 0xffffffff);
                            let _ = self.write_bar(slot + 1, 0xffffffff);
                            let mut readback_low = self.read_bar(slot).unwrap();
                            let readback_high = self.read_bar(slot + 1).unwrap();
                            let _ = self.write_bar(slot, value as u32);
                            let _ = self.write_bar(slot + 1, value_high as u32);

                            readback_low.set_bits(0..4, 0);

                            if readback_low != 0 {
                                (1 << readback_low.trailing_zeros()) as u64
                            } else {
                                1u64 << ((readback_high.trailing_zeros() + 32) as u64)
                            }
                        };
                        // let value64 = (value as u64) | ((value_high as u64) << 32);

                        bararr[slot as usize] =
                            PciMem::new_bar(PciMemType::Mem64Low, value as u64, size, pre);
                        bararr[(slot + 1) as usize] =
                            PciMem::new_bar(PciMemType::Mem64High, value_high as u64, size, pre);
                        slot += 1; // need extra add 1
                    }
                    _ => {
                        warn!("unknown bar type");
                    }
                }
            } else {
                // IO space
                let size = {
                    let _ = self.write_bar(slot, 0xffffffff);
                    let mut readback = self.read_bar(slot).unwrap();
                    let _ = self.write_bar(slot, readback as u32);

                    readback.set_bit(0, false);
                    if readback == 0x0 {
                        slot += 1;
                        continue;
                    }

                    1 << readback.trailing_zeros()
                };
                bararr[slot as usize] = PciMem::new_io(value as u64, size as u64);
            }
            slot += 1;
        }
        bararr
    }

    fn read_bar(&self, slot: u8) -> HvResult<usize> {
        // println!("read bar slot {}", slot);
        self.backend()
            .read_u32((0x10 + (slot as u16) * 4) as PciConfigAddress)
            .map(|r| r as usize)
    }

    fn write_bar(&self, slot: u8, value: u32) -> HvResult {
        // println!("write bar slot {} {}", slot, value);
        self.backend()
            .write_u32((0x10 + (slot as u16) * 4) as PciConfigAddress, value)
    }
}


pub trait PciRomRW: PciRWBase {
    fn rom_offset(&self) -> u64;
    fn parse_rom(&self) -> PciMem {
        let offset = self.rom_offset();
        let value = self.backend().read_u32(offset).unwrap();

        let size = {
            let _ = self.backend().write_u32(offset, 0xfffff800);
            let mut readback = self.backend().read_u32(offset).unwrap();
            let _ = self.backend().write_u32(offset, value);
            if readback == 0x0 {
                return PciMem::default();
            }
            readback.set_bits(0..4, 0);
            1 << readback.trailing_zeros()
        };
        PciMem::new_rom(value as u64, size)
    }
}

/*      32                            16                              0
 *      +-----------------------------+------------------------------+
 *      |       Device ID             |       Vendor ID              | 0x00
 *      |                             |                              |
 *      +-----------------------------+------------------------------+
 *      |         Status              |       Command                | 0x04
 *      |                             |                              |
 *      +-----------------------------+---------------+--------------+
 *      |               Class Code                    |   Revision   | 0x08
 *      |                                             |      ID      |
 *      +--------------+--------------+---------------+--------------+
 *      |     BIST     |    Header    |    Latency    |  Cacheline   | 0x0c
 *      |              |     type     |     timer     |    size      |
 *      +--------------+--------------+---------------+--------------+
 */
#[derive(Debug, Clone)]
pub struct PciConfigHeader(PciConfigMmio);

impl PciRWBase for PciConfigHeader {
    fn backend(&self) -> &dyn PciRegion {
        &self.0
    }
}
impl PciRW for PciConfigHeader {}
impl PciHeaderRW for PciConfigHeader {}

impl PciConfigHeader {
    pub fn new_with_region(region: PciConfigMmio) -> Self {
        PciConfigHeader(region)
    }
}

/*     32                           16                              0
 *     +-----------------------------------------------------------+ 0x00
 *     |                                                           |
 *     |                Predefined region of header                |
 *     |                                                           |
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 0                  | 0x10
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 1                  | 0x14
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 2                  | 0x18
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 3                  | 0x1c
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 4                  | 0x20
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 5                  | 0x24
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  CardBus CIS Pointer                      | 0x28
 *     |                                                           |
 *     +----------------------------+------------------------------+
 *     |       Subsystem ID         |    Subsystem vendor ID       | 0x2c
 *     |                            |                              |
 *     +----------------------------+------------------------------+
 *     |               Expansion ROM Base Address                  | 0x30
 *     |                                                           |
 *     +--------------------------------------------+--------------+
 *     |                 Reserved                   |  Capability  | 0x34
 *     |                                            |   Pointer    |
 *     +--------------------------------------------+--------------+
 *     |                         Reserved                          | 0x38
 *     |                                                           |
 *     +--------------+--------------+--------------+--------------+
 *     |   Max_Lat    |   Min_Gnt    |  Interrupt   |  Interrupt   | 0x3c
 *     |              |              |   pin        |   line       |
 *     +--------------+--------------+--------------+--------------+
 */
pub trait PciField: Debug {
    fn to_offset(&self) -> usize;
    fn size(&self) -> usize;
}

pub enum EndpointField {
    ID,
    Command,
    Status,
    RevisionIDAndClassCode,
    CacheLineSize,
    LatencyTime,
    HeaderType,
    Bist,
    Bar(usize),
    CardCisPointer,
    SubsystemVendorId,
    SubsystemId,
    ExpansionRomBar,
    CapabilityPointer,
    InterruptLine,
    InterruptPin,
    MinGnt,
    MaxLat,
    Unknown(usize),
}

impl Debug for EndpointField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EndpointField {{")?;
        let _ =match self {
            EndpointField::ID => write!(f, "ID"),
            EndpointField::Command => write!(f, "Command"),
            EndpointField::Status => write!(f, "Status"),
            EndpointField::RevisionIDAndClassCode => write!(f, "RevisionIDAndClassCode"),
            EndpointField::CacheLineSize => write!(f, "CacheLineSize"),
            EndpointField::LatencyTime => write!(f, "LatencyTime"),
            EndpointField::HeaderType => write!(f, "HeaderType"),
            EndpointField::Bist => write!(f, "Bist"),
            EndpointField::Bar(slot) => write!(f, "Bar({})", slot),
            EndpointField::CardCisPointer => write!(f, "CardCisPointer"),
            EndpointField::SubsystemVendorId => write!(f, "SubsystemVendorId"),
            EndpointField::SubsystemId => write!(f, "SubsystemId"),
            EndpointField::ExpansionRomBar => write!(f, "ExpansionRomBar"),
            EndpointField::CapabilityPointer => write!(f, "CapabilityPointer"),
            EndpointField::InterruptLine => write!(f, "InterruptLine"),
            EndpointField::InterruptPin => write!(f, "InterruptPin"),
            EndpointField::MinGnt => write!(f, "MinGnt"),
            EndpointField::MaxLat => write!(f, "MaxLat"),
            EndpointField::Unknown(offset) => write!(f, "Unknown({})", offset),
        };
        write!(f, "}}")
    }
}

impl PciField for EndpointField {
    fn to_offset(&self) -> usize {
        match self {
            EndpointField::ID => 0x00,
            EndpointField::Command => 0x04,
            EndpointField::Status => 0x06,
            EndpointField::RevisionIDAndClassCode => 0x08,
            EndpointField::CacheLineSize => 0x0c,
            EndpointField::LatencyTime => 0x0d,
            EndpointField::HeaderType => 0x0e,
            EndpointField::Bist => 0x0f,
            EndpointField::Bar(slot) => (0x10 + slot * 4) as usize,
            EndpointField::CardCisPointer => 0x28,
            EndpointField::SubsystemVendorId => 0x2c,
            EndpointField::SubsystemId => 0x2e,
            EndpointField::ExpansionRomBar => 0x30,
            EndpointField::CapabilityPointer => 0x34,
            EndpointField::InterruptLine => 0x3c,
            EndpointField::InterruptPin => 0x3d,
            EndpointField::MinGnt => 0x3e,
            EndpointField::MaxLat => 0x3f,
            EndpointField::Unknown(offset) => *offset,
        }
    }

    fn size(&self) -> usize {
        match self {
            EndpointField::ID => 4,
            EndpointField::Command => 2,
            EndpointField::Status => 2,
            EndpointField::RevisionIDAndClassCode => 4,
            EndpointField::CacheLineSize => 1,
            EndpointField::LatencyTime => 1,
            EndpointField::HeaderType => 1,
            EndpointField::Bist => 1,
            EndpointField::Bar(_) => 4,
            EndpointField::CardCisPointer => 4,
            EndpointField::SubsystemVendorId => 2,
            EndpointField::SubsystemId => 2,
            EndpointField::ExpansionRomBar => 4,
            EndpointField::CapabilityPointer => 1,
            EndpointField::InterruptLine => 1,
            EndpointField::InterruptPin => 1,
            EndpointField::MinGnt => 1,
            EndpointField::MaxLat => 1,
            EndpointField::Unknown(_) => 4, // Default to 4 bytes for unknown fields
        }
    }
}

impl EndpointField {
    pub fn from(offset: usize, size: usize) -> Self {
        match (offset, size) {
            (0x00, 4) => EndpointField::ID,
            (0x04, 2) => EndpointField::Command,
            (0x06, 2) => EndpointField::Status,
            (0x08, 4) => EndpointField::RevisionIDAndClassCode,
            (0x0c, 1) => EndpointField::CacheLineSize,
            (0x0d, 1) => EndpointField::LatencyTime,
            (0x0e, 1) => EndpointField::HeaderType,
            (0x0f, 1) => EndpointField::Bist,
            // (0x10, 4) | (0x14, 4) | (0x18, 4) | (0x1c, 4) | (0x20, 4) | (0x24, 4) => {
            //     EndpointField::Bar
            // }
            (0x10, 4) => EndpointField::Bar(0),
            (0x14, 4) => EndpointField::Bar(1),
            (0x18, 4) => EndpointField::Bar(2),
            (0x1c, 4) => EndpointField::Bar(3),
            (0x20, 4) => EndpointField::Bar(4),
            (0x24, 4) => EndpointField::Bar(5),
            (0x28, 4) => EndpointField::CardCisPointer,
            (0x2c, 2) => EndpointField::SubsystemVendorId,
            (0x2e, 2) => EndpointField::SubsystemId,
            (0x30, 4) => EndpointField::ExpansionRomBar,
            (0x34, 4) => EndpointField::CapabilityPointer,
            (0x3c, 1) => EndpointField::InterruptLine,
            (0x3d, 1) => EndpointField::InterruptPin,
            (0x3e, 1) => EndpointField::MinGnt,
            (0x3f, 1) => EndpointField::MaxLat,
            (x, _) => EndpointField::Unknown(x),
        }
    }

}

#[derive(Debug, Clone)]
pub struct EndpointHeader(PciConfigMmio);

impl PciRWBase for EndpointHeader {
    fn backend(&self) -> &dyn PciRegion {
        &self.0
    }
}
impl PciRW for EndpointHeader {}
impl PciHeaderRW for EndpointHeader {}
impl PciBarRW for EndpointHeader {
    fn bar_limit(&self) -> u8 {
        6
    }
}
impl PciRomRW for EndpointHeader {
    fn rom_offset(&self) -> u64 {
        0x30
    }
}

impl EndpointHeader {
    pub fn new_with_region(region: PciConfigMmio) -> Self {
        EndpointHeader(region)
    }
}

/*     32                           16                              0
 *     +-----------------------------------------------------------+ 0x00
 *     |                                                           |
 *     |                Predefined region of header                |
 *     |                                                           |
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 0                  | 0x10
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |                  Base Address Register 1                  | 0x14
 *     |                                                           |
 *     +--------------+--------------+--------------+--------------+
 *     | Secondary    | Subordinate  |  Secondary   | Primary Bus  | 0x18
 *     |Latency Timer | Bus Number   |  Bus Number  |   Number     |
 *     +--------------+--------------+--------------+--------------+
 *     |      Secondary Status       |  I/O Limit   |   I/O Base   | 0x1C
 *     |                             |              |              |
 *     +-----------------------------+--------------+--------------+
 *     |        Memory Limit         |         Memory Base         | 0x20
 *     |                             |                             |
 *     +-----------------------------+-----------------------------+
 *     |  Prefetchable Memory Limit  |  Prefetchable Memory Base   | 0x24
 *     |                             |                             |
 *     +-----------------------------+-----------------------------+
 *     |             Prefetchable Base Upper 32 Bits               | 0x28
 *     |                                                           |
 *     +-----------------------------------------------------------+
 *     |             Prefetchable Limit Upper 32 Bits              | 0x2C
 *     |                                                           |
 *     +-----------------------------+-----------------------------+
 *     |   I/O Limit Upper 16 Bits   |   I/O Base Upper 16 Bits    | 0x30
 *     |                             |                             |
 *     +-----------------------------+--------------+--------------+
 *     |              Reserved                      |  Capability  | 0x34
 *     |                                            |   Pointer    |
 *     +--------------------------------------------+--------------+
 *     |                  Expansion ROM base address               | 0x38
 *     |                                                           |
 *     +-----------------------------+--------------+--------------+
 *     |    Bridge Control           |  Interrupt   | Interrupt    | 0x3C
 *     |                             |     PIN      |   Line       |
 *     +-----------------------------+--------------+--------------+
 */
pub enum BridgeField {
    ID,
    Command,
    Status,
    RevisionIDAndClassCode,
    CacheLineSize,
    LatencyTime,
    HeaderType,
    Bist,
    Bar,
    PrimaryBusNumber,
    SecondaryBusNumber,
    SubordinateBusNumber,
    SecondaryLatencyTimer,
    IOBase,
    IOLimit,
    SecondaryStatus,
    MemoryBase,
    MemoryLimit,
    PrefetchableMemoryBase,
    PrefetchableMemoryLimit,
    PrefetchableBaseUpper32Bits,
    PrefetchableLimitUpper32Bits,
    UIBaseUpper16Bits,
    IOLimitUpper16Bits,
    CapabilityPointer,
    ExpansionRomBar,
    InterruptLine,
    InterruptPin,
    BridgeControl,
    Unknown(usize),
}

impl Debug for BridgeField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BridgeField {{")?;
        let _ = match self {
            BridgeField::ID => write!(f, "ID"),
            BridgeField::Command => write!(f, "Command"),
            BridgeField::Status => write!(f, "Status"),
            BridgeField::RevisionIDAndClassCode => write!(f, "RevisionIDAndClassCode"),
            BridgeField::CacheLineSize => write!(f, "CacheLineSize"),
            BridgeField::LatencyTime => write!(f, "LatencyTime"),
            BridgeField::HeaderType => write!(f, "HeaderType"),
            BridgeField::Bist => write!(f, "Bist"),
            BridgeField::Bar => write!(f, "Bar"),
            BridgeField::PrimaryBusNumber => write!(f, "PrimaryBusNumber"),
            BridgeField::SecondaryBusNumber => write!(f, "SecondaryBusNumber"),
            BridgeField::SubordinateBusNumber => write!(f, "SubordinateBusNumber"),
            BridgeField::SecondaryLatencyTimer => write!(f, "SecondaryLatencyTimer"),
            BridgeField::IOBase => write!(f, "IOBase"),
            BridgeField::IOLimit => write!(f, "IOLimit"),
            BridgeField::SecondaryStatus => write!(f, "SecondaryStatus"),
            BridgeField::MemoryBase => write!(f, "MemoryBase"),
            BridgeField::MemoryLimit => write!(f, "MemoryLimit"),
            BridgeField::PrefetchableMemoryBase => write!(f, "PrefetchableMemoryBase"),
            BridgeField::PrefetchableMemoryLimit => write!(f, "PrefetchableMemoryLimit"),
            BridgeField::PrefetchableBaseUpper32Bits => write!(f, "PrefetchableBaseUpper32Bits"),
            BridgeField::PrefetchableLimitUpper32Bits => write!(f, "PrefetchableLimitUpper32Bits"),
            BridgeField::UIBaseUpper16Bits => write!(f, "UIBaseUpper16Bits"),
            BridgeField::IOLimitUpper16Bits => write!(f, "IOLimitUpper16Bits"),
            BridgeField::CapabilityPointer => write!(f, "CapabilityPointer"),
            BridgeField::ExpansionRomBar => write!(f, "ExpansionRomBar"),
            BridgeField::InterruptLine => write!(f, "InterruptLine"),
            BridgeField::InterruptPin => write!(f, "InterruptPin"),
            BridgeField::BridgeControl => write!(f, "BridgeControl"),
            BridgeField::Unknown(offset) => write!(f, "Unknown({})", offset),
        };
        write!(f, "}}")
    }
}

impl PciField for BridgeField {
    fn to_offset(&self) -> usize {
        match self {
            BridgeField::ID => 0x00,
            BridgeField::Command => 0x04,
            BridgeField::Status => 0x06,
            BridgeField::RevisionIDAndClassCode => 0x08,
            BridgeField::CacheLineSize => 0x0c,
            BridgeField::LatencyTime => 0x0d,
            BridgeField::HeaderType => 0x0e,
            BridgeField::Bist => 0x0f,
            BridgeField::Bar => 0x10,
            BridgeField::PrimaryBusNumber => 0x18,
            BridgeField::SecondaryBusNumber => 0x19,
            BridgeField::SubordinateBusNumber => 0x1a,
            BridgeField::SecondaryLatencyTimer => 0x1b,
            BridgeField::IOBase => 0x1c,
            BridgeField::IOLimit => 0x1d,
            BridgeField::SecondaryStatus => 0x1e,
            BridgeField::MemoryBase => 0x20,
            BridgeField::MemoryLimit => 0x22,
            BridgeField::PrefetchableMemoryBase => 0x24,
            BridgeField::PrefetchableMemoryLimit => 0x26,
            BridgeField::PrefetchableBaseUpper32Bits => 0x28,
            BridgeField::PrefetchableLimitUpper32Bits => 0x2c,
            BridgeField::UIBaseUpper16Bits => 0x30,
            BridgeField::IOLimitUpper16Bits => 0x32,
            BridgeField::CapabilityPointer => 0x34,
            BridgeField::ExpansionRomBar => 0x38,
            BridgeField::InterruptLine => 0x3c,
            BridgeField::InterruptPin => 0x3d,
            BridgeField::BridgeControl => 0x3e,
            BridgeField::Unknown(offset) => *offset,
        }
    }

    fn size(&self) -> usize {
        match self {
            BridgeField::ID => 4,
            BridgeField::Command => 2,
            BridgeField::Status => 2,
            BridgeField::RevisionIDAndClassCode => 4,
            BridgeField::CacheLineSize => 1,
            BridgeField::LatencyTime => 1,
            BridgeField::HeaderType => 1,
            BridgeField::Bist => 1,
            BridgeField::Bar => 4,
            BridgeField::PrimaryBusNumber => 1,
            BridgeField::SecondaryBusNumber => 1,
            BridgeField::SubordinateBusNumber => 1,
            BridgeField::SecondaryLatencyTimer => 1,
            BridgeField::IOBase => 1,
            BridgeField::IOLimit => 1,
            BridgeField::SecondaryStatus => 2,
            BridgeField::MemoryBase => 2,
            BridgeField::MemoryLimit => 2,
            BridgeField::PrefetchableMemoryBase => 2,
            BridgeField::PrefetchableMemoryLimit => 2,
            BridgeField::PrefetchableBaseUpper32Bits => 4,
            BridgeField::PrefetchableLimitUpper32Bits => 4,
            BridgeField::UIBaseUpper16Bits => 2,
            BridgeField::IOLimitUpper16Bits => 2,
            BridgeField::CapabilityPointer => 1,
            BridgeField::ExpansionRomBar => 4,
            BridgeField::InterruptLine => 1,
            BridgeField::InterruptPin => 1,
            BridgeField::BridgeControl => 2,
            BridgeField::Unknown(_) => 4, // Default to 4 bytes for unknown fields
        }
    }
}

impl BridgeField {
    pub fn from(offset: usize, size: usize) -> Self {
        match (offset, size) {
            (0x00, 4) => BridgeField::ID,
            (0x04, 2) => BridgeField::Command,
            (0x06, 2) => BridgeField::Status,
            (0x08, 4) => BridgeField::RevisionIDAndClassCode,
            (0x0c, 1) => BridgeField::CacheLineSize,
            (0x0d, 1) => BridgeField::LatencyTime,
            (0x0e, 1) => BridgeField::HeaderType,
            (0x0f, 1) => BridgeField::Bist,
            (0x10, 4) | (0x14, 4) => BridgeField::Bar,
            (0x18, 1) => BridgeField::PrimaryBusNumber,
            (0x19, 1) => BridgeField::SecondaryBusNumber,
            (0x1a, 1) => BridgeField::SubordinateBusNumber,
            (0x1b, 1) => BridgeField::SecondaryLatencyTimer,
            (0x1c, 1) => BridgeField::IOBase,
            (0x1d, 1) => BridgeField::IOLimit,
            (0x1e, 2) => BridgeField::SecondaryStatus,
            (0x20, 2) => BridgeField::MemoryBase,
            (0x22, 2) => BridgeField::MemoryLimit,
            (0x24, 2) => BridgeField::PrefetchableMemoryBase,
            (0x26, 2) => BridgeField::PrefetchableMemoryLimit,
            (0x28, 4) => BridgeField::PrefetchableBaseUpper32Bits,
            (0x2c, 4) => BridgeField::PrefetchableLimitUpper32Bits,
            (0x30, 2) => BridgeField::UIBaseUpper16Bits,
            (0x32, 2) => BridgeField::IOLimitUpper16Bits,
            (0x34, 1) => BridgeField::CapabilityPointer,
            (0x38, 4) => BridgeField::ExpansionRomBar,
            (0x3c, 1) => BridgeField::InterruptLine,
            (0x3d, 1) => BridgeField::InterruptPin,
            (0x3e, 2) => BridgeField::BridgeControl,
            (x, _) => BridgeField::Unknown(x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PciBridgeHeader(PciConfigMmio);

impl PciRWBase for PciBridgeHeader {
    fn backend(&self) -> &dyn PciRegion {
        &self.0
    }
}
impl PciRW for PciBridgeHeader {}
impl PciHeaderRW for PciBridgeHeader {}
impl PciBarRW for PciBridgeHeader {
    fn bar_limit(&self) -> u8 {
        2
    }
}
impl PciRomRW for PciBridgeHeader {
    fn rom_offset(&self) -> u64 {
        0x38
    }
}

impl PciBridgeHeader {
    pub fn new_with_region(region: PciConfigMmio) -> Self {
        PciBridgeHeader(region)
    }
}

impl PciBridgeHeader {}

fn handle_config_space_access(
    dev: ArcRwLockVirtualPciConfigSpace,
    mmio: &mut MMIOAccess,
    offset: PciConfigAddress,
    zone_id: usize,
) -> HvResult {
    let size = mmio.size;
    let value = mmio.value;
    let is_write = mmio.is_write;
    
    let vbdf = dev.get_bdf();
    let dev_type = dev.get_dev_type();

    if (offset as usize) >= BIT_LENTH {
        warn!("invalid pci offset {:#x}", offset);
        if !is_write {
            mmio.value = 0;
        }
        return Ok(());
    }

    match dev.access(offset, size) {
        false => {
            // Hardware access path
            info!(
                "hw vbdf {:#?} reg 0x{:x} try {} {}",
                vbdf,
                offset,
                if is_write { "write" } else { "read" },
                if is_write {
                    format!(" 0x{:x}", mmio.value)
                } else {
                    String::new()
                }
            );
            if is_write {
                dev.write_hw(offset, size, value)?;
            } else {
                mmio.value = dev.read_hw(offset, size).unwrap();
            }
        }
        true => {
            // Emulation access path
            info!(
                "emu vbdf {:#?} reg 0x{:x} try {} {}",
                vbdf,
                offset,
                if is_write { "write" } else { "read" },
                if is_write {
                    format!(" 0x{:x}", mmio.value)
                } else {
                    String::new()
                }
            );
            match dev_type {
                super::vpci_dev::VpciDevType::Physical => {
                    let config_type = dev.get_config_type();
                    match config_type {
                        HeaderType::Endpoint => {
                            match EndpointField::from(offset as usize, size) {
                                EndpointField::Bar(slot) => {
                                    // let slot = ((offset - 0x10) / 4) as usize;
                                    let slot = slot as usize;
                                    let bar_type = dev.with_bar_ref(slot, |bar| bar.get_type());
                                    if bar_type != PciMemType::default() {
                                        if is_write {
                                            if (value & 0xfffffff0) == 0xfffffff0 {
                                                dev.with_bar_ref_mut(slot, |bar| bar.set_size_read());
                                            } else {
                                                let _ = dev.write_emu(EndpointField::Bar(slot), value);
                                                /* for mem64, Mem64High always write after Mem64Low,
                                                 * so update bar when write Mem64High
                                                 */
                                                if bar_type == PciMemType::Mem64Low {
                                                    let new_vaddr = (value as u64) & !0xf;
                                                    dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                                    // Sync virtual_value back to emu value
                                                    let virtual_value = dev.with_bar_ref(slot, |bar| bar.get_virtual_value());
                                                    let _ = dev.write_emu(EndpointField::Bar(slot), virtual_value as usize);
                                                } else if (bar_type == PciMemType::Mem32)
                                                    | (bar_type == PciMemType::Mem64High)
                                                    | (bar_type == PciMemType::Io)
                                                {
                                                    let old_vaddr = dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                                    let new_vaddr = {
                                                        if bar_type == PciMemType::Mem64High {
                                                            /* last 4bit is flag, not address and need ignore
                                                             * flag will auto add when set_value and set_virtual_value
                                                             * Read from config_value.bar_value cache instead of space
                                                             */
                                                            let low_value = dev.with_config_value(|cv| cv.get_bar_value(slot - 1)) as u64;
                                                            let high_value = (value as u32 as u64) << 32;
                                                            (low_value | high_value) & !0xf
                                                        } else {
                                                            (value as u64) & !0xf
                                                        }
                                                    };
                                                    /* Linux traverses the PCI bus twice. During the first traversal,
                                                     * it does not assign addresses to the BARs; it simply writes back the same
                                                     * values. In the second traversal, it reorders the BARs and assigns
                                                     * addresses to them. Each time the guest writes to a BAR,
                                                     * it attempts to remove the previous mapping and add a new one.
                                                     * However, on the first access there is no prior mapping, so a single warning
                                                     * is normal. Subsequent warnings should be treated with caution.
                                                     *
                                                     * TODO: When adding a new device or removing an old one, reloading
                                                     * the PCIe bus, will the newly written BAR address overlap with
                                                     * the old BAR addresses, potentially causing the update to fail?
                                                     */
                                                    let paddr = dev.with_bar_ref(slot, |bar| bar.get_value64());
                                                    info!(
                                                        "old_vaddr {:x} new_vaddr {:x} paddr {:x}",
                                                        old_vaddr, new_vaddr, paddr
                                                    );
        
                                                    dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                                    if bar_type == PciMemType::Mem64High {
                                                        dev.with_bar_ref_mut(slot - 1, |bar| bar.set_virtual_value(new_vaddr));
                                                    }
                                                    
                                                    // // Sync virtual_value back to space after processing (adding flags)
                                                    // let virtual_value = dev.get_bar_virtual_value(slot);
                                                    // let bar_offset = (0x10 + slot * 4) as PciConfigAddress;
                                                    // let _ = dev.write_emu(bar_offset, 4, virtual_value as usize);
                                                    // if bar_type == PciMemType::Mem64High {
                                                    //     let virtual_value_low = dev.get_bar_virtual_value(slot - 1);
                                                    //     let bar_offset_low = (0x10 + (slot - 1) * 4) as PciConfigAddress;
                                                    //     let _ = dev.write_emu(bar_offset_low, 4, virtual_value_low as usize);
                                                    // }
        
                                                    let bar_size = {
                                                        let size = dev.with_bar_ref(slot, |bar| bar.get_size());
                                                        if crate::memory::addr::is_aligned(size as usize) {
                                                            size
                                                        } else {
                                                            crate::memory::PAGE_SIZE as u64
                                                        }
                                                    };

                                                    let new_vaddr = if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                                    } else {
                                                        new_vaddr
                                                    };
        
                                                    let zone = this_zone();
                                                    let mut guard = zone.write();
                                                    let gpm = &mut guard.gpm;
                                                    
                                                    if !gpm
                                                        .try_delete(old_vaddr.try_into().unwrap(), bar_size as usize)
                                                        .is_ok()
                                                    {
                                                        /* The first delete from the guest will fail
                                                         * because the region has not yet been inserted
                                                         */
                                                        warn!(
                                                            "delete bar {}: can not found 0x{:x}",
                                                            slot, old_vaddr
                                                        );
                                                    }
                                                    
                                                    gpm.try_insert(MemoryRegion::new_with_offset_mapper(
                                                        new_vaddr as GuestPhysAddr,
                                                        paddr as HostPhysAddr,
                                                        bar_size as _,
                                                        MemFlags::READ | MemFlags::WRITE,
                                                    ))?;

                                                    /* after update gpm, mem barrier is needed
                                                     */
                                                    #[cfg(target_arch = "aarch64")]
                                                    unsafe {
                                                        core::arch::asm!("isb");
                                                        core::arch::asm!("tlbi vmalls12e1is");
                                                        core::arch::asm!("dsb nsh");
                                                    }
                                                    /* after update gpm, need to flush iommu table
                                                     * in x86_64
                                                     */
                                                    #[cfg(target_arch = "x86_64")]
                                                    crate::arch::iommu::flush(
                                                        zone_id,
                                                        vbdf.bus,
                                                        (vbdf.device << 3) + vbdf.function,
                                                    );
                                                }
                                            }
                                        } else {
                                            let size_read = dev.with_bar_ref(slot, |bar| bar.get_size_read());
                                            mmio.value = if size_read {
                                                let r = dev.with_bar_ref(slot, |bar| bar.get_size_with_flag()).try_into().unwrap();
                                                dev.with_bar_ref_mut(slot, |bar| bar.clear_size_read());
                                                r
                                            } else {
                                                dev.with_bar_ref(slot, |bar| bar.get_virtual_value()) as usize
                                                // let emu_value = dev.read_emu(EndpointField::Bar(slot)).unwrap() as usize;
                                                // let virtual_value = dev.with_bar_ref(slot, |bar| bar.get_virtual_value()) as usize;
                                                // info!("emu value {:#x} virtual_value {:#x}", emu_value, virtual_value);
                                                // virtual_value
                                            };
                                        }
                                    } else {
                                        mmio.value = 0;
                                    }
                                }
                                EndpointField::ExpansionRomBar => {
                                    let rom = dev.get_rom();
                                    let rom_size_read = rom.get_size_read();
                                    if is_write {
                                        if (mmio.value & 0xfffff800) == 0xfffff800 {
                                            // Note: get_rom() returns a copy, so we need to get it again after setting
                                            // For now, we'll just set the flag through the dev guard
                                            // TODO: Add a method to set rom size_read flag
                                            warn!("ExpansionRomBar size_read not yet implemented");
                                        } else {
                                            // let old_vaddr = dev.read_emu(offset, size).unwrap() as u64;
                                            let _ = dev.write_emu(EndpointField::ExpansionRomBar, value);
                                            // TODO: add gpm change for rom
                                        }
                                    } else {
                                        mmio.value = if rom_size_read {
                                            dev.read_emu(EndpointField::ExpansionRomBar).unwrap()
                                        } else {
                                            rom.get_size_with_flag().try_into().unwrap()
                                        };
                                    }
                                }
                                EndpointField::ID => {
                                    if !is_write {
                                        mmio.value = dev.read_emu(EndpointField::ID).unwrap() as usize;
                                    }
                                }
                                _ => {
                                    mmio.value = 0;
                                }
                            }
                        }
                        HeaderType::PciBridge => {
                            // TODO: add emu for bridge, actually it is same with endpoint
                            warn!("bridge emu rw");
                        }
                        _ => {
                            warn!("unhanled pci type {:#?}", config_type);
                        }
                    }
                }
                _ => {
                    // warn!("virt pci standard rw offset {:#x}, size {:#x}", offset, size);
                    let result = dev.with_cap(|capabilities| {
                        if let Some((cap_offset, cap)) = capabilities.range(..=offset).next_back() {
                            info!("find cap at offset {:#x}, cap {:#?}", cap_offset, cap.get_type());
                            let end = *cap_offset + cap.get_size() as u64;
                            if offset >= end {
                                // hv_result_err!(ENOENT)
                                warn!("virt pci cap rw offset {:#x} out of range", offset);
                            }
                            let relative_offset = offset - *cap_offset;
                            
                            if is_write {
                                cap.with_region_mut(|region| {
                                    match region.write(relative_offset, size, mmio.value as u32) {
                                        Ok(()) => Ok(0),
                                        Err(e) => {
                                            warn!("Failed to write capability at offset 0x{:x}: {:?}", offset, e);
                                            Err(e)
                                        }
                                    }
                                })
                            } else {
                                cap.with_region(|region| {
                                    match region.read(relative_offset, size) {
                                        Ok(val) => Ok(val),
                                        Err(e) => {
                                            warn!("Failed to read capability at offset 0x{:x}: {:?}", offset, e);
                                            Err(e)
                                        }
                                    }
                                })
                            }
                        } else {
                            hv_result_err!(ENOENT)
                        }
                    });
                    
                    match result {
                        Ok(val) => {
                            if !is_write {
                                mmio.value = val as usize;
                            }
                        }
                        Err(_) => {
                            if mmio.is_write {
                                super::vpci_dev::vpci_dev_write_cfg(dev_type, dev.clone(), offset, size, value).unwrap();
                            } else {
                                mmio.value = super::vpci_dev::vpci_dev_read_cfg(dev_type, dev.clone(), offset, size).unwrap() as usize;
                            }
                        }
                    }
                }
            }
        }
    }

    info!(
        "vbdf {:#?} reg 0x{:x} {} 0x{:x}",
        vbdf,
        offset,
        if is_write { "write" } else { "read" },
        mmio.value
    );

    Ok(())
}

fn handle_device_not_found(mmio: &mut MMIOAccess, offset: PciConfigAddress) {
    /* if the dev is None, just return 0xFFFF_FFFF when read ID */
    if !mmio.is_write {
        match EndpointField::from(offset as usize, mmio.size) {
            EndpointField::ID => {
                mmio.value = 0xFFFF_FFFF;
            }
            _ => {
                // warn!("unhandled pci mmio read, addr: {:#x?}", mmio.address);
                mmio.value = 0;
            }
        }
    }
}

pub fn mmio_vpci_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    // info!("mmio_vpci_handler {:#x}", mmio.address);
    let zone_id = this_zone_id();
    let zone = this_zone();
    let offset = (mmio.address & 0xfff) as PciConfigAddress;
    let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;

    let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
        let mut guard = zone.write();
        let vbus = &mut guard.vpci_bus;
        vbus.get_device_by_base(base)
    };


    if let Some(dev) = dev {
        handle_config_space_access(dev, mmio, offset, zone_id)?;
    } else {
        handle_device_not_found(mmio, offset);
    }

    Ok(())
}
#[cfg(feature = "dwc_pcie")]
pub fn mmio_dwc_io_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    {
        let zone = this_zone();
        let guard = zone.read();
        
        let atu_config = guard.atu_configs.get_atu_by_io_base(_base as PciConfigAddress)
            .and_then(|atu| {
                guard.atu_configs.get_ecam_by_io_base(_base as PciConfigAddress)
                    .map(|ecam| (*atu, ecam))
            });
        
        drop(guard);
        
        if let Some((atu, ecam_base)) = atu_config {
            use crate::platform;
            if let Some(extend_config) = platform::ROOT_DWC_ATU_CONFIG
                .iter()
                .find(|cfg| cfg.ecam_base == ecam_base as u64)
            {
                // Create DBI backend
                let dbi_base = extend_config.dbi_base as PciConfigAddress;
                let dbi_size = extend_config.dbi_size;
                let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
                let dbi_backend = DwcConfigRegionBackend::new(dbi_region);
                
                // Call AtuUnroll to program the ATU
                AtuUnroll::dw_pcie_prog_outbound_atu_unroll(&dbi_backend, &atu)?;
            }
            mmio_perform_access(atu.pci_target as usize, mmio);
        } else {
            warn!("No ATU config yet, do nothing");
        }
    }
    Ok(())
}

#[cfg(feature = "dwc_pcie")]
pub fn mmio_dwc_cfg_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    info!("mmio_dwc_cfg_handler {:#x}", mmio.address + _base);    
    let zone = this_zone();
    let guard = zone.read();

    let atu_config = guard.atu_configs.get_atu_by_cfg_base(_base as PciConfigAddress)
        .and_then(|atu| {
            guard.atu_configs.get_ecam_by_cfg_base(_base as PciConfigAddress)
                .map(|ecam| (*atu, ecam))
        });
    
    drop(guard);
    
    if let Some((atu, ecam_base)) = atu_config {
        // Get dbi_base from platform config (usually dbi_base == ecam_base)
        use crate::platform;
        if let Some(extend_config) = platform::ROOT_DWC_ATU_CONFIG
            .iter()
            .find(|cfg| cfg.ecam_base == ecam_base as u64)
        {
            // Create DBI backend
            let dbi_base = extend_config.dbi_base as PciConfigAddress;
            let dbi_size = extend_config.dbi_size;
            let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
            let dbi_backend = DwcConfigRegionBackend::new(dbi_region);

            // warn!("atu config {:#?}", atu);
            
            // Call AtuUnroll to program the ATU
            AtuUnroll::dw_pcie_prog_outbound_atu_unroll(&dbi_backend, &atu)?;
        }
        
        let offset = (mmio.address & 0xfff) as PciConfigAddress;
        let zone_id = this_zone_id();
        let zone = this_zone();
        let mut is_dev_belong_to_zone = false;
        
        let base = mmio.address as PciConfigAddress - offset + atu.pci_target;
        
        let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
            let mut guard = zone.write();
            let vbus = &mut guard.vpci_bus;
            if let Some(dev) = vbus.get_device_by_base(base) {
                is_dev_belong_to_zone = true;
                Some(dev)
            } else {
                drop(guard);
                // Clone Arc first while holding GLOBAL_PCIE_LIST lock, then release it
                // This avoids holding multiple locks simultaneously
                let dev_clone = {
                    let global_pcie_list = GLOBAL_PCIE_LIST.lock();
                    global_pcie_list
                        .values()
                        .find(|dev| {
                            let dev_guard = dev.read();
                            dev_guard.get_base() == base
                        })
                        .cloned()
                };
                dev_clone
            }
        };
        
        let dev = match dev {
            Some(dev) => dev,
            None => {
                handle_device_not_found(mmio, offset);
                return Ok(());
            }
        };
        
        let is_root = is_this_root_zone();
        
        handle_config_space_access_direct(dev, mmio, offset, zone_id, is_root, is_dev_belong_to_zone);
    } else {
        warn!("No ATU config yet, do nothing");
    }
    Ok(())
}

#[cfg(feature = "dwc_pcie")]
pub fn mmio_vpci_handler_dbi(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    // info!("mmio_vpci_handler_dbi {:#x}", mmio.address);

    /* 0x0-0x100 is outbound atu0 reg
     * 0x100-0x200 is inbound atu0 reg just handle outbound right now
     * so MAX is ATU_BASE + ATU_REGION_SIZE/2
     */
    if mmio.address >= ATU_BASE && mmio.address < ATU_BASE + ATU_REGION_SIZE/2
    {
        let zone = this_zone();
        let mut guard = zone.write();
        let ecam_base = _base;
        let atu_offset = mmio.address - ATU_BASE;

        // warn!("set atu0 register {:#X} value {:#X}", atu_offset, mmio.value);
        
        let mut atu = guard.atu_configs.get_atu_by_ecam_mut(ecam_base).unwrap();

        // info!("atu config write {:#?}", atu);
        
        if mmio.is_write {
            if mmio.size == 4 {
                match atu_offset {
                    PCIE_ATU_UNR_REGION_CTRL1 => {
                        // info!("set atu0 region ctrl1 value {:#X}", mmio.value);
                        atu.atu_type = AtuType::from_u8((mmio.value & 0xff) as u8);
                    }
                    PCIE_ATU_UNR_REGION_CTRL2 => {
                        // Enable bit is written here, but we just track it
                        // The actual enable is handled by the driver
                    }
                    PCIE_ATU_UNR_LOWER_BASE => {
                        // info!("set atu0 lower base value {:#X}", mmio.value);
                        atu.cpu_base = (atu.cpu_base & !0xffffffff) | (mmio.value as PciConfigAddress);
                    }
                    PCIE_ATU_UNR_UPPER_BASE => {
                        // info!("set atu0 upper base value {:#X}", mmio.value);
                        atu.cpu_base = (atu.cpu_base & 0xffffffff) | ((mmio.value as PciConfigAddress) << 32);
                    }
                    PCIE_ATU_UNR_LIMIT => {
                        // info!("set atu0 limit value {:#X}", mmio.value);
                        atu.cpu_limit = (atu.cpu_limit & !0xffffffff) | (mmio.value as PciConfigAddress);
                    }
                    PCIE_ATU_UNR_UPPER_LIMIT => {
                        // Update the upper 32 bits of cpu_limit
                        atu.cpu_limit = (atu.cpu_limit & 0xffffffff) | ((mmio.value as PciConfigAddress) << 32);
                    }
                    PCIE_ATU_UNR_LOWER_TARGET => {
                        // info!("set atu0 lower target value {:#X}", mmio.value);
                        atu.pci_target = (atu.pci_target & !0xffffffff) | (mmio.value as PciConfigAddress);
                    }
                    PCIE_ATU_UNR_UPPER_TARGET => {
                        // info!("set atu0 upper target value {:#X}", mmio.value);
                        atu.pci_target = (atu.pci_target & 0xffffffff) | ((mmio.value as PciConfigAddress) << 32);
                    }
                    _ => {
                        warn!("invalid atu0 write {:#x} + {:#x}", atu_offset, mmio.size);
                    }
                }
            } else {
                warn!("invalid atu0 read size {:#x}", mmio.size);
            }
        } else {
            // Read from virtual ATU
            // warn!("read atu0 {:#x}", atu_offset);
            match atu_offset {
                PCIE_ATU_UNR_REGION_CTRL1 => {
                    mmio.value = atu.atu_type as usize;
                }
                PCIE_ATU_UNR_REGION_CTRL2 => {
                    mmio.value = ATU_ENABLE_BIT as usize;
                }
                PCIE_ATU_UNR_LOWER_BASE => {
                    mmio.value = (atu.cpu_base & 0xffffffff) as usize;
                }
                PCIE_ATU_UNR_UPPER_BASE => {
                    mmio.value = ((atu.cpu_base >> 32) & 0xffffffff) as usize;
                }
                PCIE_ATU_UNR_LIMIT => {
                    let limit_value = (atu.cpu_limit & 0xffffffff) as usize;
                    // If limit is 0, return 0x3ffffff instead
                    mmio.value = if limit_value == 0 { 0x3ffffff } else { limit_value };
                }
                PCIE_ATU_UNR_UPPER_LIMIT => {
                    // Return the upper 32 bits of cpu_limit
                    // If it's 0xffffffff, return 0x40000000 instead
                    let upper_limit = ((atu.cpu_limit >> 32) & 0xffffffff) as usize;
                    mmio.value = if upper_limit == 0xffffffff { 0x40000000 } else { upper_limit };
                }
                PCIE_ATU_UNR_LOWER_TARGET => {
                    mmio.value = (atu.pci_target & 0xffffffff) as usize;
                }
                PCIE_ATU_UNR_UPPER_TARGET => {
                    mmio.value = ((atu.pci_target >> 32) & 0xffffffff) as usize;
                }
                _ => {
                    warn!("invalid atu0 read {:#x}", atu_offset);
                    mmio_perform_access(_base, mmio);
                }
            }
        }
    } else if mmio.address > ATU_BASE + ATU_REGION_SIZE/2 {
        mmio_perform_access(_base, mmio);
        // // other atu
        // let is_root = is_this_root_zone();
        // let atu_offset = (mmio.address - ATU_BASE) % ATU_REGION_SIZE;
        
        // // 0x0-0x100 is inbound ATU (needs emulation)
        // // 0x100-0x200 is outbound ATU (direct passthrough)
        // if atu_offset >= 0x100 {
        //     // Outbound ATU: direct passthrough
        //     mmio_perform_access(_base, mmio);
        //     return Ok(());
        // }
        
        // // Inbound ATU: needs emulation (0x0-0x100 range)
        // // For non-root zones, only allow access to LIMIT registers
        // if !is_root {
        //     if mmio.is_write {
        //         // For non-root zones, only allow writes to LIMIT registers
        //         if atu_offset == PCIE_ATU_UNR_LIMIT || atu_offset == PCIE_ATU_UNR_UPPER_LIMIT {
        //             // Allow the write to proceed, but we'll handle read specially
        //             mmio_perform_access(_base, mmio);
        //         } else {
        //             // Reject access to other registers for non-root zones
        //             warn!("non-root zone attempted to access ATU register {:#x} at offset {:#x}", mmio.address, atu_offset);
        //             return Ok(());
        //         }
        //     } else {
        //         // For reads, handle LIMIT registers specially
        //         if atu_offset == PCIE_ATU_UNR_LIMIT {
        //             // Read from hardware first
        //             mmio_perform_access(_base, mmio);
        //             // If value is 0, return 0x3ffffff instead
        //             if mmio.value == 0 {
        //                 mmio.value = 0x3ffffff;
        //             }
        //         } else if atu_offset == PCIE_ATU_UNR_UPPER_LIMIT {
        //             // Read upper limit from hardware
        //             mmio_perform_access(_base, mmio);
        //             // If value is 0xffffffff, return 0x40000000 instead
        //             if mmio.value == 0xffffffff {
        //                 mmio.value = 0x40000000;
        //             }
        //         } else {
        //             // Reject access to other registers for non-root zones
        //             warn!("non-root zone attempted to read ATU register {:#x} at offset {:#x}", mmio.address, atu_offset);
        //             return Ok(());
        //         }
        //     }
        // } else {
        //     // For root zones, handle LIMIT registers specially but allow all other accesses
        //     if !mmio.is_write {
        //         if atu_offset == PCIE_ATU_UNR_LIMIT {
        //             // Read from hardware first
        //             mmio_perform_access(_base, mmio);
        //             // If value is 0, return 0x3ffffff instead
        //             if mmio.value == 0 {
        //                 mmio.value = 0x3ffffff;
        //             }
        //         } else if atu_offset == PCIE_ATU_UNR_UPPER_LIMIT {
        //             // Read upper limit from hardware
        //             mmio_perform_access(_base, mmio);
        //             // If value is 0xffffffff, return 0x40000000 instead
        //             if mmio.value == 0xffffffff {
        //                 mmio.value = 0x40000000;
        //             }
        //         } else {
        //             // For other registers, perform normal access
        //             mmio_perform_access(_base, mmio);
        //         }
        //     } else {
        //         // For writes, perform normal access (including LIMIT registers)
        //         mmio_perform_access(_base, mmio);
        //     }
        // }
    } else if mmio.address >= BIT_LENTH {
        // dbi read
        mmio_perform_access(_base, mmio);
    } else {
        warn!("mmio_vpci_handler_dbi read {:#x}", mmio.address);
        let offset = (mmio.address & 0xfff) as PciConfigAddress;
        let zone_id = this_zone_id();
        let zone = this_zone();
        let mut is_dev_belong_to_zone = false;

        let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;

        let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
            let mut guard = zone.write();
            let vbus = &mut guard.vpci_bus;
            if let Some(dev) = vbus.get_device_by_base(base) {
                is_dev_belong_to_zone = true;
                Some(dev)
            } else {
                drop(guard);
                // Clone Arc first while holding GLOBAL_PCIE_LIST lock, then release it
                // This avoids holding multiple locks simultaneously
                let dev_clone = {
                    let global_pcie_list = GLOBAL_PCIE_LIST.lock();
                    global_pcie_list
                        .values()
                        .find(|dev| {
                            let dev_guard = dev.read();
                            dev_guard.get_base() == base
                        })
                        .cloned()
                };
                dev_clone
            }
        };
        
        let dev = match dev {
            Some(dev) => dev,
            None => {
                handle_device_not_found(mmio, offset);
                return Ok(());
            }
        };
        
        let is_root = is_this_root_zone();
        
        handle_config_space_access_direct(dev, mmio, offset, zone_id, is_root, is_dev_belong_to_zone);
    }

    Ok(())
}

fn handle_config_space_access_direct(
    dev: ArcRwLockVirtualPciConfigSpace,
    mmio: &mut MMIOAccess,
    offset: PciConfigAddress,
    zone_id: usize,
    is_root: bool,
    is_dev_belong_to_zone: bool,
) -> HvResult {
    let size = mmio.size;
    let value = mmio.value;
    let is_write = mmio.is_write;
    let vbdf = dev.get_bdf();

    if (offset as usize) >= BIT_LENTH {
        warn!("invalid pci offset {:#x}", offset);
        if !is_write {
            mmio.value = 0;
        }
        return Ok(());
    }

    if is_dev_belong_to_zone || is_root {
        match dev.access(offset, size) {
            false => {
                info!(
                    "hw vbdf {:#?} reg 0x{:x} try {} {}",
                    vbdf,
                    offset,
                    if is_write { "write" } else { "read" },
                    if is_write {
                        format!(" 0x{:x}", mmio.value)
                    } else {
                        String::new()
                    }
                );
                if is_write {
                    dev.write_hw(offset, size, value)?;
                } else {
                    mmio.value = dev.read_hw(offset, size).unwrap();
                }
            }
            true => {
                info!(
                    "emu vbdf {:#?} reg 0x{:x} try {} {}",
                    vbdf,
                    offset,
                    if is_write { "write" } else { "read" },
                    if is_write {
                        format!(" 0x{:x}", mmio.value)
                    } else {
                        String::new()
                    }
                );
                match dev.get_config_type() {
                    HeaderType::Endpoint => {
                        match EndpointField::from(offset as usize, size) {
                            EndpointField::ID => {
                                if !is_write {
                                    if is_dev_belong_to_zone {
                                        mmio.value = dev.read_emu(EndpointField::ID).unwrap();
                                    } else {
                                        if is_root {
                                            /* just a id no one used now
                                             * here let root allocate resources but not drive the device
                                             */
                                            mmio.value = 0xFFFD_4106;
                                        }
                                    }
                                }
                            }
                            EndpointField::RevisionIDAndClassCode => {
                                if !is_write {
                                    if is_dev_belong_to_zone {
                                        mmio.value = dev
                                            .read_emu(EndpointField::RevisionIDAndClassCode)
                                            .unwrap();
                                    } else if is_root {
                                        // Default: base class 0xff, subclass/progIF/revision set to 0
                                        mmio.value = 0xff00_0000;
                                    }
                                }
                            }
                            EndpointField::Bar(slot) => {
                                let bar_type = dev.with_bar_ref(slot, |bar| bar.get_type());
                                if bar_type != PciMemType::default() {
                                    if is_write {
                                        if (value & 0xfffffff0) == 0xfffffff0 {
                                            dev.with_bar_ref_mut(slot, |bar| bar.set_size_read());
                                        } else {
                                            let _ = dev.write_emu(EndpointField::Bar(slot), value);
                                            if is_root {
                                                let _ = dev.write_hw(offset, size, value);
                                            }
                                            // For Mem64Low, update virtual_value but don't update GPM yet
                                            // (GPM will be updated when Mem64High is written)
                                            if bar_type == PciMemType::Mem64Low {
                                                let new_vaddr = (value as u64) & !0xf;
                                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                                // Sync virtual_value back to emu value
                                                let virtual_value = dev.with_bar_ref(slot, |bar| bar.get_virtual_value());
                                                let _ = dev.write_emu(EndpointField::Bar(slot), virtual_value as usize);
                                            } else if (bar_type == PciMemType::Mem32)
                                            | (bar_type == PciMemType::Mem64High)
                                            | (bar_type == PciMemType::Io) {
                                                let old_vaddr = dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                                let new_vaddr = {
                                                    if bar_type == PciMemType::Mem64High {
                                                        /* last 4bit is flag, not address and need ignore
                                                        * flag will auto add when set_value and set_virtual_value
                                                        * Read from config_value.bar_value cache instead of space
                                                        */
                                                        let low_value = dev.with_config_value(|cv| cv.get_bar_value(slot - 1)) as u64;
                                                        let high_value = (value as u32 as u64) << 32;
                                                        (low_value | high_value) & !0xf
                                                    } else {
                                                        (value as u64) & !0xf
                                                    }
                                                };

                                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                                if bar_type == PciMemType::Mem64High {
                                                    dev.with_bar_ref_mut(slot - 1, |bar| bar.set_virtual_value(new_vaddr));
                                                }
                                                
                                                // Sync virtual_value back to space after processing (adding flags)
                                                // TODO: check whether need sync here
                                                let virtual_value = dev.with_bar_ref(slot, |bar| bar.get_virtual_value());
                                                let _ = dev.write_emu(EndpointField::Bar(slot), virtual_value as usize);
                                                if bar_type == PciMemType::Mem64High {
                                                    let virtual_value_low = dev.with_bar_ref(slot - 1, |bar| bar.get_virtual_value());
                                                    let _ = dev.write_emu(EndpointField::Bar(slot - 1), virtual_value_low as usize);
                                                }

                                                let paddr = if is_root {
                                                    dev.with_bar_ref_mut(slot, |bar| bar.set_value(new_vaddr));
                                                    if bar_type == PciMemType::Mem64High {
                                                        dev.with_bar_ref_mut(slot - 1, |bar| bar.set_value(new_vaddr));
                                                    }
                                                    new_vaddr as HostPhysAddr
                                                } else {
                                                    dev.with_bar_ref(slot, |bar| bar.get_value64()) as HostPhysAddr
                                                };

                                                let bar_size = {
                                                    let size = dev.with_bar_ref(slot, |bar| bar.get_size());
                                                    if crate::memory::addr::is_aligned(size as usize) {
                                                        size
                                                    } else {
                                                        crate::memory::PAGE_SIZE as u64
                                                    }
                                                };
                                                let new_vaddr = if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                                    crate::memory::addr::align_up(new_vaddr as usize) as u64
                                                } else {
                                                    new_vaddr as u64
                                                };

                                                if !is_root {
                                                    let zone = this_zone();
                                                    let mut guard = zone.write();
                                                    let gpm = &mut guard.gpm;
                                                    
                                                    if !gpm.try_delete(old_vaddr.try_into().unwrap(), bar_size as usize).is_ok() {
                                                        warn!(
                                                            "delete bar {}: can not found 0x{:x}",
                                                            slot, old_vaddr
                                                        );
                                                    }
                                                    gpm.try_insert(MemoryRegion::new_with_offset_mapper(
                                                        new_vaddr as GuestPhysAddr,
                                                        paddr as HostPhysAddr,
                                                        bar_size as _,
                                                        MemFlags::READ | MemFlags::WRITE,
                                                    ))?;
                                                    /* after update gpm, mem barrier is needed
                                                        */
                                                    #[cfg(target_arch = "aarch64")]
                                                    unsafe {
                                                        core::arch::asm!("isb");
                                                        core::arch::asm!("tlbi vmalls12e1is");
                                                        core::arch::asm!("dsb nsh");
                                                    }
                                                    /* after update gpm, need to flush iommu table
                                                        * in x86_64
                                                        */
                                                    #[cfg(target_arch = "x86_64")]
                                                    crate::arch::iommu::flush(
                                                        zone_id,
                                                        vbdf.bus,
                                                        (vbdf.device << 3) + vbdf.function,
                                                    );
                                                }
                                            }
                                        }
                                    } else {
                                        // Re-fetch bar to get the latest virtual_value after potential write updates
                                        let size_read = dev.with_bar_ref(slot, |bar| bar.get_size_read());
                                        mmio.value = if size_read {
                                            let r = dev.with_bar_ref(slot, |bar| bar.get_size_with_flag()).try_into().unwrap();
                                            dev.with_bar_ref_mut(slot, |bar| bar.clear_size_read());
                                            r
                                        } else {
                                            dev.with_bar_ref(slot, |bar| bar.get_virtual_value()) as usize
                                            // let emu_value = dev.read_emu(EndpointField::Bar(slot)).unwrap() as usize;
                                            // let virtual_value = dev.with_bar_ref(slot, |bar| bar.get_virtual_value()) as usize;
                                            // info!("emu value {:#x} virtual_value {:#x}", emu_value, virtual_value);
                                            // virtual_value
                                        }
                                    }
                                } else {
                                    mmio.value = 0;
                                }
                            }
                            EndpointField::ExpansionRomBar => {
                                let rom = dev.get_rom();
                                let rom_size_read = rom.get_size_read();
                                if is_write {
                                    if (mmio.value & 0xfffff800) == 0xfffff800 {
                                        // TODO: Add method to set rom size_read flag
                                        warn!("ExpansionRomBar size_read not yet implemented for direct handler");
                                    } else {
                                        // let old_vaddr = dev.read_emu(offset, size).unwrap() as u64;
                                        let _ = dev.write_emu(EndpointField::ExpansionRomBar, value);
                                        // TODO: add gpm change for rom
                                    }
                                } else {
                                    mmio.value = if rom_size_read {
                                        dev.read_emu(EndpointField::ExpansionRomBar).unwrap()
                                    } else {
                                        rom.get_size_with_flag().try_into().unwrap()
                                    };
                                }
                            }
                            _ => {
                                mmio.value = 0;
                            }
                        }
                    }
                    HeaderType::PciBridge => {
                        // TODO: add emu for bridge, actually it is same with endpoint
                        warn!("bridge emu rw");
                    }
                    _ => {
                        warn!("unhanled pci type {:#?}", dev.get_config_type());
                    }
                }
            }                
            _ => {
                // warn!("virt pci standard rw offset {:#x}, size {:#x}", offset, size);
                let result = dev.with_cap(|capabilities| {
                    if let Some((cap_offset, cap)) = capabilities.range(..=offset).next_back() {
                        info!("find cap at offset {:#x}, cap {:#?}", cap_offset, cap.get_type());
                        let end = *cap_offset + cap.get_size() as u64;
                        if offset >= end {
                            // hv_result_err!(ENOENT)
                            warn!("virt pci cap rw offset {:#x} out of range", offset);
                        }
                        let relative_offset = offset - *cap_offset;
                        
                        if is_write {
                            cap.with_region_mut(|region| {
                                match region.write(relative_offset, size, mmio.value as u32) {
                                    Ok(()) => Ok(0),
                                    Err(e) => {
                                        warn!("Failed to write capability at offset 0x{:x}: {:?}", offset, e);
                                        Err(e)
                                    }
                                }
                            })
                        } else {
                            cap.with_region(|region| {
                                match region.read(relative_offset, size) {
                                    Ok(val) => Ok(val),
                                    Err(e) => {
                                        warn!("Failed to read capability at offset 0x{:x}: {:?}", offset, e);
                                        Err(e)
                                    }
                                }
                            })
                        }
                    } else {
                        hv_result_err!(ENOENT)
                    }
                });
                
                match result {
                    Ok(val) => {
                        if !is_write {
                            mmio.value = val as usize;
                        }
                    }
                    Err(_) => {
                        if mmio.is_write {
                            super::vpci_dev::vpci_dev_write_cfg(dev.get_dev_type(), dev.clone(), offset, size, value).unwrap();
                        } else {
                            mmio.value = super::vpci_dev::vpci_dev_read_cfg(dev.get_dev_type(), dev.clone(), offset, size).unwrap() as usize;
                        }
                    }
                }
            }
        }
    }
    info!(
        "vbdf {:#?} reg 0x{:x} {} 0x{:x}",
        vbdf,
        offset,
        if is_write { "write" } else { "read" },
        mmio.value
    );

    Ok(())
}

pub fn mmio_vpci_direct_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    let zone_id = this_zone_id();
    let zone = this_zone();
    let offset = (mmio.address & 0xfff) as PciConfigAddress;
    let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;
    let mut is_dev_belong_to_zone = false;


    let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
        let mut guard = zone.write();
        let vbus = &mut guard.vpci_bus;
        if let Some(dev) = vbus.get_device_by_base(base) {
            is_dev_belong_to_zone = true;
            Some(dev)
        } else {
            drop(guard);
            let global_pcie_list = GLOBAL_PCIE_LIST.lock();
            global_pcie_list
                .values()
                .find(|dev| dev.read().get_base() == base)
                .cloned()
        }
    };

    let dev = match dev {
        Some(dev) => dev,
        None => {
            handle_device_not_found(mmio, offset);
            return Ok(());
        }
    };

    let is_root = is_this_root_zone();

    let _ = handle_config_space_access_direct(dev, mmio, offset, zone_id, is_root, is_dev_belong_to_zone);
    
    Ok(())
}