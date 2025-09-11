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
#![allow(dead_code)]
use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use alloc::string::String;
use bit_field::BitField;
use bitflags::bitflags;
use core::slice;

use crate::{
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MMIOAccess, MemFlags, MemoryRegion},
    percpu::this_zone,
    zone::Zone,
};

use super::{
    pci_mem::{PciRegion, PciRegionMmio},
    pci_struct::Bdf,
    PciConfigAddress,
};

pub trait PciRW: Debug + Send + Sync {
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<usize>;
    fn write(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult;
}

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

    pub fn new_io(value: u64) -> Self {
        Self {
            bar_type: PciMemType::Io,
            virtual_value: 0,
            value,
            size: 0,
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

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_size_with_flag(&mut self) -> u64 {
        match self.bar_type {
            PciMemType::Mem32 | PciMemType::Rom => !(self.size - 1u64),
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
            _ => {}
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
            _ => {}
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
pub struct PciConfigHeader(PciRegionMmio);

macro_rules! impl_pci_rw {
    ($ty:ty) => {
        impl PciRW for $ty {
            fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
                match size {
                    1 => self.0.read_u8(offset).map(|v| v as usize),
                    2 => self.0.read_u16(offset).map(|v| v as usize),
                    4 => self.0.read_u32(offset).map(|v| v as usize),
                    _ => {
                        hv_result_err!(EFAULT, "pci: invalid mmio read size: {size}")
                    }
                }
            }
            fn write(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
                match size {
                    1 => self.0.write_u8(offset, value as u8),
                    2 => self.0.write_u16(offset, value as u16),
                    4 => self.0.write_u32(offset, value as u32),
                    _ => {
                        hv_result_err!(EFAULT, "pci: invalid mmio write size: {size}")
                    }
                }
            }
        }
    };
}

macro_rules! impl_pci_header {
    ($ty:ty) => {
        impl $ty {
            pub fn id(&self) -> (DeviceId, VendorId) {
                let id = self.0.read_u32(0x00).unwrap();
                (
                    id.get_bits(0..16) as VendorId,
                    id.get_bits(16..32) as DeviceId,
                )
            }

            pub fn header_type(&self) -> HeaderType {
                match self.0.read_u8(0x0e).unwrap().get_bits(0..7) {
                    0x00 => HeaderType::Endpoint,
                    0x01 => HeaderType::PciBridge,
                    0x02 => HeaderType::CardBusBridge,
                    v => HeaderType::Unknown(v as u8),
                }
            }

            pub fn has_multiple_functions(&self) -> bool {
                self.0.read_u8(0x0c).unwrap().get_bit(7)
            }

            pub fn revision_and_class(&self) -> (DeviceRevision, BaseClass, SubClass, Interface) {
                let value = self.0.read_u32(0x08).unwrap();
                (
                    value.get_bits(0..8) as DeviceRevision,
                    value.get_bits(24..32) as BaseClass,
                    value.get_bits(16..24) as SubClass,
                    value.get_bits(8..16) as Interface,
                )
            }

            pub fn status(&self) -> PciStatus {
                let status = self.0.read_u16(0x06).unwrap();
                PciStatus::from_bits_truncate(status)
            }

            pub fn command(&self) -> PciCommand {
                let command = self.0.read_u16(0x04).unwrap();
                PciCommand::from_bits_truncate(command)
            }

            pub fn update_command<F>(&mut self, f: F)
            where
                F: FnOnce(PciCommand) -> PciCommand,
            {
                let mut data = self.0.read_u16(0x04).unwrap();
                let new_command = f(PciCommand::from_bits_retain(data.get_bits(0..16)));
                data.set_bits(0..16, new_command.bits());
                let _ = self.0.write_u16(0x04, data);
            }
        }
    };
}

impl_pci_rw!(PciConfigHeader);
impl_pci_header!(PciConfigHeader);

impl PciConfigHeader {
    pub fn new_with_region(region: PciRegionMmio) -> Self {
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
 *     |                 Reserved                   | Capabilities | 0x34
 *     |                                            |   Pointer    |
 *     +--------------------------------------------+--------------+
 *     |                         Reserved                          | 0x38
 *     |                                                           |
 *     +--------------+--------------+--------------+--------------+
 *     |   Max_Lat    |   Min_Gnt    |  Interrupt   |  Interrupt   | 0x3c
 *     |              |              |   pin        |   line       |
 *     +--------------+--------------+--------------+--------------+
 */
pub enum EndpointField {
    ID,
    Command,
    Status,
    RevisionIDAndClassCode,
    CacheLineSize,
    LatencyTime,
    HeaderType,
    Bist,
    Bar,
    CardCisPointer,
    SubsystemVendorId,
    SubsystemId,
    ExpansionRomBar,
    CapabilitiesPointer,
    InterruptLine,
    InterruptPin,
    MinGnt,
    MaxLat,
    Unknown(usize),
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
            (0x10, 4) | (0x14, 4) | (0x18, 4) | (0x1c, 4) | (0x20, 4) | (0x24, 4) => {
                EndpointField::Bar
            }
            (0x28, 4) => EndpointField::CardCisPointer,
            (0x2c, 2) => EndpointField::SubsystemVendorId,
            (0x2e, 2) => EndpointField::SubsystemId,
            (0x30, 4) => EndpointField::ExpansionRomBar,
            (0x34, 4) => EndpointField::CapabilitiesPointer,
            (0x3c, 1) => EndpointField::InterruptLine,
            (0x3d, 1) => EndpointField::InterruptPin,
            (0x3e, 1) => EndpointField::MinGnt,
            (0x3f, 1) => EndpointField::MaxLat,
            (x, _) => EndpointField::Unknown(x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EndpointHeader(PciRegionMmio);

impl_pci_rw!(EndpointHeader);
impl_pci_header!(EndpointHeader);

impl EndpointHeader {
    pub fn new_with_region(region: PciRegionMmio) -> Self {
        EndpointHeader(region)
    }

    pub fn parse_bar(&self) -> Bar {
        let mut bararr = Bar::default();

        let mut slot = 0u8;
        while slot < 6 {
            let value = self.read_bar(slot).unwrap();

            if !value.get_bit(0) {
                let pre = value.get_bit(3);

                match value.get_bits(1..3) {
                    0b00 => {
                        let size = {
                            let _ = self.write_bar(slot, 0xfffffff0);
                            let mut readback = self.read_bar(slot).unwrap();
                            let _ = self.write_bar(slot, readback as u32);

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
                        if slot == 5 {
                            warn!("read bar64 in last bar");
                            break;
                        }

                        let value_high = self.read_bar(slot + 1).unwrap();
                        let size = {
                            let _ = self.write_bar(slot, 0xfffffff0);
                            let _ = self.write_bar(slot + 1, 0xfffffff0);
                            let mut readback_low = self.read_bar(slot).unwrap();
                            let readback_high = self.read_bar(slot + 1).unwrap();
                            let _ = self.write_bar(slot, readback_low as u32);
                            let _ = self.write_bar(slot + 1, readback_high as u32);

                            readback_low.set_bits(0..4, 0);

                            if readback_low != 0 {
                                (1 << readback_low.trailing_zeros()) as u64
                            } else {
                                1u64 << ((readback_high.trailing_zeros() + 32) as u64)
                            }
                        };
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
                bararr[slot as usize] = PciMem::new_io(value as u64);
            }
            slot += 1;
        }
        bararr
    }

    pub fn read_bar(&self, slot: u8) -> HvResult<usize> {
        // println!("read bar slot {}", slot);
        self.0
            .read_u32((0x10 + (slot as u16) * 4) as PciConfigAddress)
            .map(|r| r as usize)
    }

    pub fn write_bar(&self, slot: u8, value: u32) -> HvResult {
        // println!("write bar slot {} {}", slot, value);
        self.0
            .write_u32((0x10 + (slot as u16) * 4) as PciConfigAddress, value)
    }

    pub fn parse_rom(&self) -> PciMem {
        let offset = 0x30;
        let value = self.0.read_u32(offset).unwrap();

        let size = {
            let _ = self.0.write_u32(offset, 0xfffff800);
            let mut readback = self.0.read_u32(offset).unwrap();
            let _ = self.0.write_u32(offset, value);
            if readback == 0x0 {
                return PciMem::default();
            }
            readback.set_bits(0..4, 0);
            1 << readback.trailing_zeros()
        };
        PciMem::new_rom(value as u64, size)
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
#[derive(Debug, Clone)]
pub struct PciBridgeHeader(PciRegionMmio);

impl_pci_rw!(PciBridgeHeader);
impl_pci_header!(PciBridgeHeader);

impl PciBridgeHeader {
    pub fn new_with_region(region: PciRegionMmio) -> Self {
        PciBridgeHeader(region)
    }
}

impl PciBridgeHeader {}

pub fn mmio_vpci_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    let offset = (mmio.address & 0xfff) as PciConfigAddress;
    let vbdf = Bdf::from_address(mmio.address as u64);
    let size = mmio.size;
    let value = mmio.value;

    let zone = this_zone();
    let mut guard = zone.write();
    let (vbus, gpm) = {
        let Zone { gpm, vpci_bus, .. } = &mut *guard;
        (vpci_bus, gpm)
    };

    let mut dev = None;
    for node in vbus.devs().iter_mut() {
        if node.1.get_vbdf() == vbdf {
            debug!("vbdf find {:#?}", vbdf);
            dev = Some(node.1);
            break;
        }
    }

    if let Some(dev) = dev {
        match dev.access(offset, size) {
            false => {
                debug!(
                    "hw vbdf {:#?} reg 0x{:x} try {} {}",
                    vbdf,
                    offset,
                    if mmio.is_write { "write" } else { "read" },
                    if mmio.is_write {
                        format!(" 0x{:x}", mmio.value)
                    } else {
                        String::new()
                    }
                );
                if mmio.is_write {
                    dev.write_hw(offset, size, value)?;
                } else {
                    mmio.value = dev.read_hw(offset, size).unwrap();
                }
            }
            true => {
                debug!(
                    "emu vbdf {:#?} reg 0x{:x} try {} {}",
                    vbdf,
                    offset,
                    if mmio.is_write { "write" } else { "read" },
                    if mmio.is_write {
                        format!(" 0x{:x}", mmio.value)
                    } else {
                        String::new()
                    }
                );
                match dev.get_config_type() {
                    HeaderType::Endpoint => {
                        match EndpointField::from(offset as usize, size) {
                            EndpointField::Bar => {
                                let slot = ((offset - 0x10) / 4) as usize;
                                /* the write of bar needs to start from dev,
                                 * where the bar variable here is just a copy
                                 */
                                let bar = &mut dev.get_bararr()[slot];
                                let bar_type = bar.get_type();
                                if bar_type != PciMemType::default() {
                                    if mmio.is_write {
                                        if (value & 0xfffffff0) == 0xfffffff0 {
                                            dev.set_bar_size_read(slot);
                                        } else {
                                            let _ = dev.write_emu(offset, size, value);
                                            /* for mem64, Mem64High always write after Mem64Low,
                                             * so update bar when write Mem64High
                                             */
                                            if (bar_type == PciMemType::Mem32)
                                                | (bar_type == PciMemType::Mem64High)
                                            {
                                                let old_vaddr = bar.get_virtual_value64() & &!0xf;
                                                let new_vaddr = {
                                                    if bar_type == PciMemType::Mem64High {
                                                        /* last 4bit is flag, not address and need ignore
                                                         * flag will auto add when set_value and set_virtual_value
                                                         */
                                                        dev.read_emu64(offset - 0x4).unwrap() & !0xf
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
                                                if !gpm
                                                    .try_delete(old_vaddr.try_into().unwrap())
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
                                                let paddr = bar.get_value64();
                                                debug!(
                                                    "old_vaddr {:x} new_vaddr {:x} paddr {:x}",
                                                    old_vaddr, new_vaddr, paddr
                                                );

                                                dev.set_bar_virtual_value(slot, new_vaddr);
                                                if bar_type == PciMemType::Mem64High {
                                                    dev.set_bar_virtual_value(slot - 1, new_vaddr);
                                                }

                                                gpm.insert(MemoryRegion::new_with_offset_mapper(
                                                    new_vaddr as GuestPhysAddr,
                                                    paddr as HostPhysAddr,
                                                    bar.get_size() as _,
                                                    MemFlags::READ | MemFlags::WRITE,
                                                ))?;
                                                /* after update gpm, mem barrier is needed */
                                                unsafe {
                                                    core::arch::asm!("isb");
                                                    core::arch::asm!("tlbi vmalls12e1is");
                                                    core::arch::asm!("dsb nsh");
                                                }
                                            }
                                        }
                                    } else {
                                        mmio.value = if bar.get_size_read() {
                                            let r = bar.get_size_with_flag().try_into().unwrap();
                                            dev.clear_bar_size_read(slot);
                                            r
                                        } else {
                                            bar.get_virtual_value().try_into().unwrap()
                                        };
                                    }
                                } else {
                                    mmio.value = 0;
                                }
                            }
                            EndpointField::ExpansionRomBar => {
                                let mut rom = dev.get_rom();
                                if mmio.is_write {
                                    if (mmio.value & 0xfffff800) == 0xfffff800 {
                                        rom.set_size_read();
                                    } else {
                                        // let old_vaddr = dev.read_emu(offset, size).unwrap() as u64;
                                        let _ = dev.write_emu(offset, size, value);
                                        // TODO: add gpm change for rom
                                    }
                                } else {
                                    mmio.value = if rom.get_size_read() {
                                        dev.read_emu(offset, size).unwrap()
                                    } else {
                                        rom.get_size_with_flag().try_into().unwrap()
                                    };
                                }
                            }
                            _ => {}
                        }
                    }
                    HeaderType::PciBridge => {
                        // TODO: add emu for bridge, actually it is same with endpoint
                    }
                    _ => {
                        warn!("unhanled pci type {:#?}", dev.get_config_type());
                    }
                }
            }
        }
    } else {
        debug!("not found dev");
        /* if the dev is None, just return 0xFFFF_FFFF when read ID */
        if !mmio.is_write {
            match EndpointField::from(offset as usize, size) {
                EndpointField::ID => {
                    mmio.value = 0xFFFF_FFFF;
                }
                _ => {
                    warn!("unhandled pci mmio read");
                    mmio.value = 0;
                }
            }
        }
    }

    debug!(
        "vbdf {:#?} reg 0x{:x} {} 0x{:x}",
        vbdf,
        offset,
        if mmio.is_write { "write" } else { "read" },
        mmio.value
    );

    Ok(())
}
