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
use core::{cmp::Ordering, fmt::Debug, hint::spin_loop, ops::Range, str::FromStr};

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use bit_field::BitField;
use bitvec::{array::BitArray, order::Lsb0, BitArr};

use crate::{
    error::{HvErrorNum, HvResult},
    pci::pci_access::{Bar, PciBarRW, PciHeaderRW, PciRomRW},
};

use super::{
    mem_alloc::BarAllocator,
    pci_access::{
        EndpointField, EndpointHeader, HeaderType, PciBridgeHeader, PciCommand, PciConfigHeader,
        PciMem, PciMemType, PciRW,
    },
    pci_mem::{PciRegion, PciRegionMmio},
    PciConfigAddress,
};

type VirtualPciConfigBits = BitArr!(for BIT_LENTH, in u8, Lsb0);

const MAX_DEVICE: u8 = 31;
const MAX_FUNCTION: u8 = 7;
pub const CONFIG_LENTH: u64 = 256;
const BIT_LENTH: usize = 256 * 2;

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct Bdf {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl Bdf {
    #[allow(dead_code)]
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    #[allow(dead_code)]
    pub fn is_zero(&self) -> bool {
        if self.bus == 0 && self.device == 0 && self.function == 0 {
            return true;
        }
        false
    }

    pub fn from_address(address: PciConfigAddress) -> Self {
        let bdf = address >> 12;
        let function = (bdf & 0b111) as u8;
        let device = ((bdf >> 3) & 0b11111) as u8;
        let bus = (bdf >> 8) as u8;
        Self {
            bus,
            device,
            function,
        }
    }

    pub fn to_address(&self, offset: usize) -> PciConfigAddress {
        let mut address = offset as PciConfigAddress;
        address.set_bits(12..15, self.function as u64);
        address.set_bits(15..20, self.device as u64);
        address.set_bits(20..28, self.bus as u64);
        address
    }

    pub fn is_host_bridge(&self) -> bool {
        if (self.device, self.function) == (0, 0) {
            true
        } else {
            false
        }
    }
}

impl Ord for Bdf {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_address(0).cmp(&other.to_address(0))
    }
}

impl PartialOrd for Bdf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Bdf {
    type Err = HvErrorNum;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 0000:00:04.0
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(HvErrorNum::EINVAL);
        }

        let bus = u8::from_str_radix(parts[1], 16)
            .map_err(|_| HvErrorNum::EINVAL)
            .unwrap();
        let device_function: Vec<&str> = parts[2].split('.').collect();
        if device_function.len() != 2 {
            panic!("Invalid device.function format");
        }

        let device = u8::from_str_radix(device_function[0], 16)
            .map_err(|_| HvErrorNum::EINVAL)
            .unwrap();
        let function = u8::from_str_radix(device_function[1], 10)
            .map_err(|_| HvErrorNum::EINVAL)
            .unwrap();

        Ok(Bdf {
            bus,
            device,
            function,
        })
    }
}

impl Debug for Bdf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{}",
            0, self.bus, self.device, self.function
        )
    }
}

/* 0: ro;
 * 1: rw
 */
#[derive(Debug, Clone)]
pub struct VirtualPciConfigControl {
    bits: VirtualPciConfigBits,
}

impl VirtualPciConfigControl {
    /* 0x0F, 0x8F, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x08, */
    pub fn endpoint() -> Self {
        Self {
            bits: !BitArray::ZERO,
        }
    }

    pub fn bridge() -> Self {
        Self {
            bits: !BitArray::ZERO,
        }
    }

    pub fn host_bridge() -> Self {
        Self {
            bits: !BitArray::ZERO,
        }
    }
}

/* 0: read hw
 * 1: read emu
 */
#[derive(Debug, Clone)]
pub struct VirtualPciAccessBits {
    bits: VirtualPciConfigBits,
}

impl VirtualPciAccessBits {
    pub fn endpoint() -> Self {
        let mut bits = BitArray::ZERO;
        bits[0x10..0x34].fill(true); //bar and rom
        Self { bits }
    }

    pub fn bridge() -> Self {
        Self {
            bits: BitArray::ZERO,
        }
    }

    pub fn host_bridge() -> Self {
        Self {
            bits: BitArray::ZERO,
        }
    }
}

/* VirtualPciConfigSpace
 * bdf: the bdf hvisor seeing(same with the bdf without hvisor)
 * vbdf: the bdf zone seeing, it can set just you like without sr-iov
 * space: the space where emulate the config space
 * control: control the satus of rw every bit in config space
 * access: Determines whether the variable is read from space or hw
 * backend: the hw rw interface
 */
#[derive(Clone)]
pub struct VirtualPciConfigSpace {
    bdf: Bdf,
    vbdf: Bdf,
    config_type: HeaderType,

    space: [u8; BIT_LENTH],
    control: VirtualPciConfigControl,
    access: VirtualPciAccessBits,

    backend: Arc<dyn PciRW>,

    bararr: Bar,
    rom: PciMem,
}

impl VirtualPciConfigSpace {
    /* false: some bits ro */
    pub fn writable(&self, offset: PciConfigAddress, size: usize) -> bool {
        self.control.bits[offset as usize..offset as usize + size]
            .last_zero()
            .is_none()
    }

    /* false: some bits need read from hw */
    pub fn access(&self, offset: PciConfigAddress, size: usize) -> bool {
        self.access.bits[offset as usize..offset as usize + size]
            .last_zero()
            .is_none()
    }

    pub fn get_bararr(&self) -> Bar {
        self.bararr
    }

    pub fn set_bar_size_read(&mut self, slot: usize) {
        self.bararr[slot].set_size_read();
    }

    pub fn set_bar_virtual_value(&mut self, slot: usize, value: u64) {
        self.bararr[slot].set_virtual_value(value);
    }

    pub fn clear_bar_size_read(&mut self, slot: usize) {
        self.bararr[slot].clear_size_read();
    }

    pub fn get_rom(&self) -> PciMem {
        self.rom
    }

    // TODO: update sapce when first time read value from hw, and next read will more quick
    pub fn update_space(&mut self, offset: PciConfigAddress, size: usize, _value: usize) {
        match self.get_config_type() {
            HeaderType::Endpoint => {
                match EndpointField::from(offset as usize, size) {
                    EndpointField::Bar => {
                        // let updating_range = offset as usize..offset as usize+ size;
                        // let bytes = &value.to_le_bytes()[..size];
                        // info!("[{:x}-{:x}] bytes {:#?} \n{:x}", updating_range.start, updating_range.end, bytes, value);
                        // self.space[updating_range.clone()].copy_from_slice(bytes);
                        // self.access.bits[updating_range].fill(true);
                    }
                    _ => {}
                }
            }
            _ => {
                warn!("TODO updating space");
            }
        }
    }
}

impl Debug for VirtualPciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\n  bdf {:#?}\n  vbdf {:#?}\n  type {:#?}\n  {:#?}",
            self.bdf, self.vbdf, self.config_type, self.bararr
        )
    }
}

impl VirtualPciConfigSpace {
    pub fn endpoint(bdf: Bdf, backend: Arc<dyn PciRW>, bararr: Bar, rom: PciMem) -> Self {
        Self {
            bdf,
            vbdf: Bdf::default(),
            space: [0u8; BIT_LENTH],
            control: VirtualPciConfigControl::endpoint(),
            access: VirtualPciAccessBits::endpoint(),
            config_type: HeaderType::Endpoint,
            backend,
            bararr,
            rom,
        }
    }

    pub fn bridge(bdf: Bdf, backend: Arc<dyn PciRW>, bararr: Bar) -> Self {
        Self {
            bdf,
            vbdf: Bdf::default(),
            space: [0u8; BIT_LENTH],
            control: VirtualPciConfigControl::bridge(),
            access: VirtualPciAccessBits::bridge(),
            config_type: HeaderType::PciBridge,
            backend,
            bararr,
            rom: PciMem::default(),
        }
    }

    pub fn unknown(bdf: Bdf, backend: Arc<dyn PciRW>) -> Self {
        Self {
            bdf,
            vbdf: Bdf::default(),
            space: [0u8; BIT_LENTH],
            control: VirtualPciConfigControl::endpoint(),
            access: VirtualPciAccessBits::endpoint(),
            config_type: HeaderType::Endpoint,
            backend,
            bararr: Bar::default(),
            rom: PciMem::default(),
        }
    }

    pub fn host_bridge(bdf: Bdf, backend: Arc<dyn PciRW>) -> Self {
        Self {
            bdf: bdf,
            vbdf: bdf,
            space: [0u8; BIT_LENTH],
            control: VirtualPciConfigControl::host_bridge(),
            access: VirtualPciAccessBits::host_bridge(),
            config_type: HeaderType::Endpoint,
            backend,
            bararr: Bar::default(),
            rom: PciMem::default(),
        }
    }

    pub fn get_bdf(&self) -> Bdf {
        self.bdf
    }

    pub fn get_vbdf(&self) -> Bdf {
        self.vbdf
    }

    pub fn get_config_type(&self) -> HeaderType {
        self.config_type
    }

    pub fn set_vbdf(&mut self, vbdf: Bdf) {
        self.vbdf = vbdf;
    }

    /* now the space_init just with bar
     */
    pub fn space_init(&mut self) {
        for (slot, bar) in self.bararr.into_iter().enumerate() {
            let offset = 0x10 + slot * 4;
            let bytes = bar.get_value().to_le_bytes();
            self.space[offset..offset + 4].copy_from_slice(&bytes);
        }
        match self.config_type {
            HeaderType::Endpoint => {
                let bytes = self.rom.get_value().to_le_bytes();
                self.space[0x30..0x34].copy_from_slice(&bytes);
            }
            HeaderType::PciBridge => {
                let bytes = self.rom.get_value().to_le_bytes();
                self.space[0x38..0x3c].copy_from_slice(&bytes);
            }
            _ => {}
        }
    }
}

impl VirtualPciConfigSpace {
    pub fn read_hw(&mut self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
        let r = self.backend.read(offset, size);
        if let Ok(value) = r {
            self.update_space(offset, size, value);
        }
        r
    }

    pub fn write_hw(&mut self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        if self.writable(offset, size) {
            let r = self.backend.write(offset, size, value);
            if r.is_ok() {
                self.update_space(offset, size, value);
            }
            r
        } else {
            hv_result_err!(EPERM, "pci: invalid write to hw")
        }
    }

    pub fn read_emu(&mut self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
        match size {
            1 | 2 | 4 => {
                let slice = &self.space[offset as usize..offset as usize + size];
                let value = match size {
                    1 => slice[0] as usize,
                    2 => u16::from_le_bytes(slice.try_into().unwrap()) as usize,
                    4 => u32::from_le_bytes(slice.try_into().unwrap()) as usize,
                    _ => unreachable!(),
                };
                Ok(value)
            }
            _ => {
                hv_result_err!(EFAULT, "pci: invalid virtual mmio read size: {size}")
            }
        }
    }

    pub fn read_emu64(&mut self, offset: PciConfigAddress) -> HvResult<u64> {
        let slice = &self.space[offset as usize..offset as usize + 8];
        let value = u64::from_le_bytes(slice.try_into().unwrap()) as u64;
        Ok(value)
    }

    pub fn write_emu(&mut self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        if self.writable(offset, size) {
            match size {
                1 | 2 | 4 => {
                    let slice = &mut self.space[offset as usize..offset as usize + size];
                    match size {
                        1 => slice[0] = value as u8,
                        2 => slice.copy_from_slice(&u16::to_le_bytes(value as u16)),
                        4 => slice.copy_from_slice(&u32::to_le_bytes(value as u32)),
                        _ => unreachable!(),
                    }
                    Ok(())
                }
                _ => {
                    hv_result_err!(EFAULT, "pci: invalid virtual mmio write size: {size}")
                }
            }
        } else {
            hv_result_err!(EPERM, "pci: invalid write to hw")
        }
    }
}

pub struct PciIterator<B: BarAllocator> {
    allocator: Option<B>,
    stack: Vec<Bridge>,
    segment: PciConfigAddress,
    bus_max: u8,
    function: u8,
    is_mulitple_function: bool,
    is_finish: bool,
}

impl<B: BarAllocator> PciIterator<B> {
    fn address(&self) -> PciConfigAddress {
        let parent = self.stack.last().unwrap();
        let bus = parent.secondary_bus;
        let device = parent.device;

        let mut address: PciConfigAddress = 0;
        address.set_bits(12..15, self.function as PciConfigAddress);
        address.set_bits(15..20, device as PciConfigAddress);
        address.set_bits(20..28, bus as PciConfigAddress);
        address += self.segment;
        address
    }

    fn get_node(&mut self) -> Option<VirtualPciConfigSpace> {
        let address = self.address();

        let region = PciRegionMmio::new(address, CONFIG_LENTH);
        let pci_header = PciConfigHeader::new_with_region(region);
        let (vender_id, _device_id) = pci_header.id();
        if vender_id == 0xffff {
            return None;
        }

        self.is_mulitple_function = pci_header.has_multiple_functions();

        match pci_header.header_type() {
            HeaderType::Endpoint => {
                let mut ep = EndpointHeader::new_with_region(region);
                let rom = ep.parse_rom();

                let bararr =
                    Self::bar_mem_init(ep.bar_limit().into(), &mut self.allocator, &mut ep);

                let ep = Arc::new(ep);
                let bdf = Bdf::from_address(address);
                Some(VirtualPciConfigSpace::endpoint(bdf, ep, bararr, rom))
            }
            HeaderType::PciBridge => {
                warn!("bridge");
                let mut bridge = PciBridgeHeader::new_with_region(region);

                let bararr =
                    Self::bar_mem_init(bridge.bar_limit().into(), &mut self.allocator, &mut bridge);

                let bridge = Arc::new(bridge);
                let bdf = Bdf::from_address(address);
                Some(VirtualPciConfigSpace::bridge(bdf, bridge, bararr))
            }
            _ => {
                warn!("unknown type");
                let pci_header = Arc::new(pci_header);
                let bdf = Bdf::from_address(address);
                Some(VirtualPciConfigSpace::unknown(bdf, pci_header))
            }
        }
    }

    fn bar_mem_init<D: PciBarRW + PciHeaderRW>(
        bar_max: usize,
        allocator: &mut Option<B>,
        dev: &mut D,
    ) -> Bar {
        let mut bararr = dev.parse_bar();

        if let Some(a) = allocator {
            dev.update_command(|mut cmd| {
                cmd.remove(PciCommand::IO_ENABLE);
                cmd.remove(PciCommand::MEMORY_ENABLE);
                cmd
            });

            let mut i = 0;
            while i < bar_max {
                match bararr[i].get_type() {
                    PciMemType::Mem32 => {
                        let value = a.alloc_memory32(bararr[i].get_size() as u32).unwrap();
                        bararr[i].set_value(value as u64);
                        bararr[i].set_virtual_value(value as u64);
                        let _ = dev.write_bar(i as u8, value);
                    }
                    PciMemType::Mem64Low => {
                        let value = a.alloc_memory64(bararr[i].get_size()).unwrap();
                        bararr[i].set_value(value);
                        bararr[i].set_virtual_value(value);
                        let _ = dev.write_bar(i as u8, value as u32);
                        i += 1;
                        bararr[i].set_value(value);
                        bararr[i].set_virtual_value(value);
                        let _ = dev.write_bar(i as u8, (value >> 32) as u32);
                    }
                    _ => {}
                }
                i += 1;
            }
        }
        bararr
    }

    fn get_bridge(&self) -> Bridge {
        let a = self.stack.last();
        match a {
            Some(bridge) => bridge.clone(),
            None => {
                unreachable!("get null stack")
            }
        }
    }

    fn is_next_function_max(&mut self) -> bool {
        if self.is_mulitple_function {
            if self.function == MAX_FUNCTION {
                self.function = 0;
                true
            } else {
                self.function += 1;
                false
            }
        } else {
            self.function = 0;
            true
        }
    }

    fn next_device_not_ok(&mut self) -> bool {
        if let Some(parent) = self.stack.last_mut() {
            if parent.device == MAX_DEVICE {
                if let Some(mut parent) = self.stack.pop() {
                    self.is_finish = parent.subordinate_bus == self.bus_max;

                    parent.update_bridge_bus();
                    self.function = 0;
                    return true;
                } else {
                    self.is_finish = true;
                }
            } else {
                parent.device += 1;
            }
        } else {
            self.is_finish = true;
        }

        false
    }

    fn next(&mut self, current_bridge: Option<Bridge>) {
        if let Some(bridge) = current_bridge {
            for parent in &mut self.stack {
                parent.subordinate_bus += 1;
            }

            self.stack.push(bridge.clone());

            self.function = 0;
            return;
        }

        if self.is_next_function_max() {
            while self.next_device_not_ok() {
                spin_loop();
            }
        }
    }
}

impl<B: BarAllocator> Iterator for PciIterator<B> {
    type Item = VirtualPciConfigSpace;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_finish {
            if let Some(mut node) = self.get_node() {
                node.space_init();
                self.next(match node.config_type {
                    HeaderType::PciBridge => Some(self.get_bridge().next_bridge(self.address())),
                    _ => None,
                });
                return Some(node);
            } else {
                self.next(None);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Bridge {
    device: u8,
    subordinate_bus: u8,
    secondary_bus: u8,
    primary_bus: u8,
    mmio: PciRegionMmio,
}

impl Bridge {
    pub fn host_bridge(address: PciConfigAddress) -> Self {
        Self {
            device: 0,
            subordinate_bus: 0,
            secondary_bus: 0,
            primary_bus: 0,
            mmio: PciRegionMmio::new(address, CONFIG_LENTH),
        }
    }

    pub fn next_bridge(&self, address: PciConfigAddress) -> Self {
        let mmio = PciRegionMmio::new(address, CONFIG_LENTH);
        Self {
            device: 0,
            subordinate_bus: self.subordinate_bus + 1,
            secondary_bus: self.subordinate_bus + 1,
            primary_bus: self.secondary_bus,
            mmio,
        }
    }

    pub fn update_bridge_bus(&mut self) {
        let mut value = self.mmio.read_u32(0x18).unwrap();
        value.set_bits(16..24, self.subordinate_bus.into());
        value.set_bits(8..16, self.secondary_bus.into());
        value.set_bits(0..8, self.primary_bus.into());
        let _ = self.mmio.write_u32(0x18, value);
    }
}

/* In fact, the size will be managed by the pci_mmio_handler, so only base is needed here */
pub struct RootComplex {
    pub mmio_base: PciConfigAddress,
}

impl RootComplex {
    pub fn new(mmio_base: PciConfigAddress) -> Self {
        Self { mmio_base }
    }

    fn __enumerate<B: BarAllocator>(
        &mut self,
        range: Option<Range<usize>>,
        bar_alloc: Option<B>,
    ) -> PciIterator<B> {
        let mmio_base = self.mmio_base;
        let range = range.unwrap_or_else(|| 0..0x100);
        PciIterator {
            allocator: bar_alloc,
            stack: vec![Bridge::host_bridge(mmio_base)],
            segment: mmio_base,
            bus_max: (range.end - 1) as _,
            function: 0,
            is_mulitple_function: false,
            is_finish: false,
        }
    }

    pub fn enumerate<B: BarAllocator>(
        &mut self,
        range: Option<Range<usize>>,
        bar_alloc: Option<B>,
    ) -> PciIterator<B> {
        self.__enumerate(range, bar_alloc)
    }
}

#[derive(Debug)]
pub struct VirtualRootComplex {
    devs: BTreeMap<Bdf, VirtualPciConfigSpace>,
}

impl VirtualRootComplex {
    pub fn new() -> Self {
        Self {
            devs: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        bdf: Bdf,
        dev: VirtualPciConfigSpace,
    ) -> Option<VirtualPciConfigSpace> {
        self.devs.insert(bdf, dev)
    }

    pub fn devs(&mut self) -> &mut BTreeMap<Bdf, VirtualPciConfigSpace> {
        &mut self.devs
    }
}
