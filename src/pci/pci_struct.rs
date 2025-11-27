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
use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use bit_field::BitField;
use bitvec::{array::BitArray, order::Lsb0, BitArr};
use core::{cmp::Ordering, fmt::Debug, ops::Range, str::FromStr};

use super::{
    config_accessors::{PciConfigAccessor, PciConfigMmio},
    mem_alloc::BarAllocator,
    pci_access::{
        Bar, EndpointField, EndpointHeader, HeaderType, PciBarRW, PciBridgeHeader, PciCommand,
        PciConfigHeader, PciField, PciHeaderRW, PciMem, PciMemType, PciRW, PciRomRW,
    },
    PciConfigAddress,
    pci_access::{BaseClass, SubClass, Interface},
};

use crate::{error::{HvErrorNum, HvResult}, pci::vpci_dev::VpciDevType};

type VirtualPciConfigBits = BitArr!(for BIT_LENTH, in u8, Lsb0);

const MAX_DEVICE: u8 = 31;
const MAX_FUNCTION: u8 = 7;
pub const CONFIG_LENTH: u64 = 256;
pub const BIT_LENTH: usize = 128 * 5;

// PCIe Device/Port Type values
const PCI_EXP_TYPE_ROOT_PORT: u16 = 4;
const PCI_EXP_TYPE_UPSTREAM: u16 = 5;
const PCI_EXP_TYPE_DOWNSTREAM: u16 = 6;
const PCI_EXP_TYPE_PCIE_BRIDGE: u16 = 8;

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

    pub fn bus(&self) -> u8 {
        self.bus
    }

    pub fn device(&self) -> u8 {
        self.device
    }

    pub fn function(&self) -> u8 {
        self.function
    }

    pub fn from_address(address: PciConfigAddress) -> Bdf {
        let bdf = address >> 12;
        let function = (bdf & 0b111) as u8;
        let device = ((bdf >> 3) & 0b11111) as u8;
        let bus = (bdf >> 8) as u8;
        Bdf {
            bus,
            device,
            function,
        }
    }

    pub fn is_host_bridge(&self, bus_begin: u8) -> bool {
        if (self.bus, self.device, self.function) == (bus_begin, 0, 0) {
            true
        } else {
            false
        }
    }
}

impl Ord for Bdf {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bus
            .cmp(&other.bus)
            .then_with(|| self.device.cmp(&other.device))
            .then_with(|| self.function.cmp(&other.function))
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

    pub fn virt_dev() -> Self {
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

    pub fn virt_dev() -> Self {
        Self {
            bits: !BitArray::ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PciConfigSpace {
    data: [u8; BIT_LENTH],
}

impl PciConfigSpace {
    pub fn new() -> Self {
        Self {
            data: [0u8; BIT_LENTH],
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn get_range(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..offset + size]
    }

    pub fn get_range_mut(&mut self, offset: usize, size: usize) -> &mut [u8] {
        &mut self.data[offset..offset + size]
    }

    pub fn set<F: PciField>(&mut self, field: F, value: u32) {
        let offset = field.to_offset();
        let size = field.size();
        match size {
            1 => {
                self.get_range_mut(offset, 1)[0] = value as u8;
            }
            2 => {
                self.get_range_mut(offset, 2).copy_from_slice(&(value as u16).to_le_bytes());
            }
            4 => {
                self.get_range_mut(offset, 4).copy_from_slice(&value.to_le_bytes());
            }
            _ => {
                // For other sizes, write as many bytes as needed
                let bytes = value.to_le_bytes();
                self.get_range_mut(offset, size.min(bytes.len())).copy_from_slice(&bytes[..size.min(bytes.len())]);
            }
        }
    }

    pub fn init_with_type(dev_type: VpciDevType) -> Self {
        crate::pci::vpci_dev::init_config_space_with_type(dev_type)
    }
}

impl Default for PciConfigSpace {
    fn default() -> Self {
        Self::new()
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
    host_bdf: Bdf,
    parent_bdf: Bdf,
    bdf: Bdf,
    vbdf: Bdf,
    config_type: HeaderType,
    class: (BaseClass, SubClass, Interface),

    base: PciConfigAddress,

    pub space: PciConfigSpace,
    control: VirtualPciConfigControl,
    access: VirtualPciAccessBits,

    backend: Arc<dyn PciRW>,

    bararr: Bar,
    rom: PciMem,
    capabilities: PciCapabilityList,

    dev_type: VpciDevType,
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

    pub fn get_dev_type(&self) -> VpciDevType {
        self.dev_type
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
                // warn!("TODO updating space");
            }
        }
    }
}

impl Debug for VirtualPciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\n  bdf {:#?}\n  base {:#x}\n  type {:#?}\n  {:#?} {:#?}",
            self.bdf, self.base, self.config_type, self.bararr, self.rom
        )
    }
}

impl VirtualPciConfigSpace {
    pub fn virt_dev(
        bdf: Bdf,
        base: PciConfigAddress,
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::Endpoint,
            class: (0u8,0u8,0u8),
            base,
            space: PciConfigSpace::init_with_type(VpciDevType::StandardVdev),
            control: VirtualPciConfigControl::virt_dev(),
            access: VirtualPciAccessBits::virt_dev(),
            backend: Arc::new(EndpointHeader::new_with_region(PciConfigMmio::new(base, CONFIG_LENTH))),
            bararr: Bar::default(),
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::StandardVdev,
        }
    }
    pub fn endpoint(
        bdf: Bdf,
        base: PciConfigAddress,
        backend: Arc<dyn PciRW>,
        bararr: Bar,
        rom: PciMem,
        class: (BaseClass, SubClass, Interface),
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::Endpoint,
            class,
            base,
            space: PciConfigSpace::new(),
            control: VirtualPciConfigControl::endpoint(),
            access: VirtualPciAccessBits::endpoint(),
            backend,
            bararr,
            rom,
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
        }
    }

    pub fn bridge(
        bdf: Bdf,
        base: PciConfigAddress,
        backend: Arc<dyn PciRW>,
        bararr: Bar,
        rom: PciMem,
        class: (BaseClass, SubClass, Interface),
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::PciBridge,
            class,
            base,
            space: PciConfigSpace::new(),
            control: VirtualPciConfigControl::bridge(),
            access: VirtualPciAccessBits::bridge(),
            backend,
            bararr,
            rom,
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
        }
    }

    pub fn unknown(bdf: Bdf, base: PciConfigAddress, backend: Arc<dyn PciRW>) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::Endpoint,
            class: (0u8,0u8,0u8),
            base,
            space: PciConfigSpace::new(),
            control: VirtualPciConfigControl::endpoint(),
            access: VirtualPciAccessBits::endpoint(),
            backend,
            bararr: Bar::default(),
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
        }
    }

    pub fn host_bridge(bdf: Bdf, base: PciConfigAddress, backend: Arc<dyn PciRW>, class: (BaseClass, SubClass, Interface)) -> Self {
        Self {
            host_bdf: bdf,
            parent_bdf: bdf,
            bdf: bdf,
            vbdf: bdf,
            config_type: HeaderType::Endpoint,
            class,
            base,
            space: PciConfigSpace::new(),
            control: VirtualPciConfigControl::host_bridge(),
            access: VirtualPciAccessBits::host_bridge(),
            backend,
            bararr: Bar::default(),
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
        }
    }

    pub fn set_host_bdf(&mut self, host_bdf: Bdf) {
        self.host_bdf = host_bdf;
    }

    pub fn get_host_bdf(&self) -> Bdf {
        self.host_bdf
    }

    pub fn set_parent_bdf(&mut self, parent_bdf: Bdf) {
        self.parent_bdf = parent_bdf;
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

    pub fn get_base(&self) -> PciConfigAddress {
        self.base
    }

    /* now the space_init just with bar
     */
    pub fn space_init(&mut self) {
        for (slot, bar) in self.bararr.into_iter().enumerate() {
            let offset = 0x10 + slot * 4;
            let bytes = bar.get_value().to_le_bytes();
            self.space.get_range_mut(offset, 4).copy_from_slice(&bytes);
        }
        match self.config_type {
            HeaderType::Endpoint => {
                let bytes = self.rom.get_value().to_le_bytes();
                self.space.get_range_mut(0x30, 4).copy_from_slice(&bytes);
            }
            HeaderType::PciBridge => {
                let bytes = self.rom.get_value().to_le_bytes();
                self.space.get_range_mut(0x38, 4).copy_from_slice(&bytes);
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
                let slice = self.space.get_range(offset as usize, size);
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
        let slice = self.space.get_range(offset as usize, 8);
        let value = u64::from_le_bytes(slice.try_into().unwrap()) as u64;
        Ok(value)
    }

    pub fn write_emu(&mut self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        if self.writable(offset, size) {
            match size {
                1 | 2 | 4 => {
                    let slice = self.space.get_range_mut(offset as usize, size);
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

#[derive(Debug)]
pub struct PciIterator<B: BarAllocator> {
    allocator: Option<B>,
    stack: Vec<Bridge>,
    segment: PciConfigAddress,
    bus_range: Range<usize>,
    function: u8,
    is_mulitple_function: bool,
    is_finish: bool,
    accessor: Arc<dyn PciConfigAccessor>,
}

impl<B: BarAllocator> PciIterator<B> {
    fn address(&self, parent_bus: u8, bdf: Bdf) -> PciConfigAddress {
        let offset = 0;

        match self.accessor.get_physical_address(bdf, offset, parent_bus) {
            Ok(addr) => addr,
            Err(_) => 0x0,
        }
    }

    fn get_node(&mut self) -> Option<VirtualPciConfigSpace> {
        // Handle placeholder: pop it first, use bus_range.start for initial scan
        let was_placeholder = self
            .stack
            .last()
            .map(|b| b.mmio.is_placeholder())
            .unwrap_or(false);
        if was_placeholder {
            self.stack.pop(); // Remove placeholder
        }

        let (bus, device, function, parent_bus) = if let Some(parent) = self.stack.last() {
            // device is already added in next()
            (parent.bus, parent.device, self.function, parent.primary_bus)
        } else {
            // host bridge, so device is 0
            let bus_begin = self.bus_range.start as u8;
            (bus_begin, 0, self.function, bus_begin)
        };

        let bdf = Bdf::new(bus, device, function);

        let address = self.address(parent_bus, bdf);
        info!("get node {:x} {:#?}", address, bdf);

        let region = PciConfigMmio::new(address, CONFIG_LENTH);
        let pci_header = PciConfigHeader::new_with_region(region);
        let (vender_id, _device_id) = pci_header.id();

        warn!("vender_id {:#x}", vender_id);

        // Check if device exists
        if vender_id == 0xffff || self.accessor.skip_device(bdf) {
            if function == 0 {
                // Function 0 doesn't exist, so device doesn't exist at all
                // Skip all functions and move to next device
                warn!(
                    "get none - device not present (vendor_id=0xffff) at {:#?}",
                    bdf
                );
                self.function = 0;
                self.is_mulitple_function = false;
            } else {
                // Function > 0 doesn't exist, but device might have other functions
                warn!(
                    "get none - function not present (vendor_id=0xffff) at {:#?}",
                    bdf
                );
            }
            return None;
        }

        // only check is_mulitple_function for function 0
        if self.function == 0 {
            self.is_mulitple_function = pci_header.has_multiple_functions();
        }

        let (_, base_class, sub_class, interface) = pci_header.revision_and_class();
        let class = (base_class, sub_class, interface);

        match pci_header.header_type() {
            HeaderType::Endpoint => {
                // For endpoint: push host_bridge if we popped placeholder
                if was_placeholder {
                    let bus_begin = self.bus_range.start as u8;
                    let host_bridge = Bridge::host_bridge(self.segment, bus_begin, self.is_mulitple_function, self.function);
                    self.stack.push(host_bridge);
                }

                let mut ep = EndpointHeader::new_with_region(region);
                let rom = Self::rom_init(&mut ep);

                let bararr =
                    Self::bar_mem_init(ep.bar_limit().into(), &mut self.allocator, &mut ep);

                info!("get node bar mem init end {:#?}", bararr);

                let ep = Arc::new(ep);
                let mut node = VirtualPciConfigSpace::endpoint(bdf, address, ep, bararr, rom, class);

                let _ = node.capability_enumerate();

                Some(node)
            }
            HeaderType::PciBridge => {
                // For bridge: don't push host_bridge, it will be handled in Iterator::next()
                warn!("bridge");
                let mut bridge = PciBridgeHeader::new_with_region(region);
                let rom = Self::rom_init(&mut bridge);

                let bararr =
                    Self::bar_mem_init(bridge.bar_limit().into(), &mut self.allocator, &mut bridge);

                let bridge = Arc::new(bridge);
                let mut node = VirtualPciConfigSpace::bridge(bdf, address, bridge, bararr, rom, class);

                let _ = node.capability_enumerate();

                Some(node)
            }
            _ => {
                warn!("unknown type");
                let pci_header = Arc::new(pci_header);
                Some(VirtualPciConfigSpace::unknown(bdf, address, pci_header))
            }
        }
    }

    fn rom_init<D: PciRomRW + PciHeaderRW>(dev: &mut D) -> PciMem {
        let mut rom = dev.parse_rom();
        rom.set_virtual_value(rom.get_value() as u64);
        rom
    }

    fn bar_mem_init<D: PciBarRW + PciHeaderRW>(
        bar_max: usize,
        allocator: &mut Option<B>,
        dev: &mut D,
    ) -> Bar {
        let mut bararr = dev.parse_bar();

        info!("{:#?}", bararr);

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
                    PciMemType::Io => {
                        //TODO: alloc io in hvisor, just set virt value for now
                        let value = bararr[i].get_value64();
                        bararr[i].set_virtual_value(value);
                        let _ = dev.write_bar(i as u8, value as u32);
                    }
                    _ => {}
                }
                i += 1;
            }
        } else {
            // use default bar address as virt bar address
            let mut i = 0;
            while i < bar_max {
                match bararr[i].get_type() {
                    PciMemType::Mem32 => {
                        let value = bararr[i].get_value64();
                        bararr[i].set_virtual_value(value as u64);
                        let _ = dev.write_bar(i as u8, value as u32);
                    }
                    PciMemType::Mem64Low => {
                        let value = bararr[i].get_value64();
                        bararr[i].set_virtual_value(value);
                        let _ = dev.write_bar(i as u8, value as u32);
                        i += 1;
                        bararr[i].set_virtual_value(value);
                        let _ = dev.write_bar(i as u8, (value >> 32) as u32);
                    }
                    PciMemType::Io => {
                        let value = bararr[i].get_value64();
                        bararr[i].set_virtual_value(value);
                        let _ = dev.write_bar(i as u8, value as u32);
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
                unreachable!("get bridge none");
            }
        }
    }

    fn next_device_not_ok(&mut self) -> bool {
        if let Some(parent) = self.stack.last_mut() {
            // only one child and skip this bus
            if parent.has_secondary_link {
                parent.device = MAX_DEVICE;
            }

            if parent.device == MAX_DEVICE {
                if let Some(mut parent) = self.stack.pop() {
                    self.is_finish = parent.subordinate_bus as usize == self.bus_range.end;

                    parent.update_bridge_bus();
                    self.function = parent.function;
                    self.is_mulitple_function = parent.is_mulitple_function;
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

            if self.is_mulitple_function && self.function < MAX_FUNCTION {
                // Device supports multiple functions and we haven't checked all functions yet
                self.function += 1;
                return;
            }

            self.function = 0;
            return;
        }

        // Try next function first if device supports multiple functions
        if self.is_mulitple_function && self.function < MAX_FUNCTION {
            // Device supports multiple functions and we haven't checked all functions yet
            self.function += 1;
            return;
        }

        // All functions of this device have been checked (or device doesn't support multiple functions)
        // Move to next device
        self.function = 0;
        while self.next_device_not_ok() {
            // Keep moving to next device until we find a valid one or finish
        }
    }
}

impl<B: BarAllocator> Iterator for PciIterator<B> {
    type Item = VirtualPciConfigSpace;

    fn next(&mut self) -> Option<Self::Item> {
        info!("pci dev next");
        while !self.is_finish {
            if let Some(mut node) = self.get_node() {
                node.space_init();
                let bus_begin = self.bus_range.start as u8;
                /* 
                 * when first time to enumerate, placeholder is pop in get_node
                 * the message of host bridge must be got after get_node()
                 * so we push host bridge to stack here
                 */ 
                if self.stack.is_empty() {
                    let host_bridge = Bridge::host_bridge(self.segment, bus_begin, self.is_mulitple_function, self.function);
                    self.stack.push(host_bridge);
                }
                let parent = self.stack.last().unwrap(); // Safe because we just ensured it exists
                let host_bdf = Bdf::new(bus_begin, 0, 0);
                let parent_bdf = Bdf::new(parent.bus, parent.device, 0);
                let parent_bus = parent.primary_bus;
                node.set_host_bdf(host_bdf);
                node.set_parent_bdf(parent_bdf);
                self.next(match node.class.0 {
                    0x6 if node.class.1 != 0x0 => {
                        let bdf = Bdf::new(parent.subordinate_bus + 1, 0, 0);
                        Some(
                            self.get_bridge().next_bridge(
                                self.address(parent_bus, bdf),
                                node.has_secondary_link(),
                                self.is_mulitple_function,
                                self.function
                            ),
                        )
                    }
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
    bus: u8,
    device: u8,
    function: u8,
    subordinate_bus: u8,
    secondary_bus: u8,
    primary_bus: u8,
    mmio: PciConfigMmio,
    has_secondary_link: bool,
    is_mulitple_function: bool,
}

impl Bridge {
    // Create a placeholder bridge for initializing the stack.
    // This is not a real host bridge, just a placeholder to track bus hierarchy.
    pub fn placeholder() -> Self {
        Self {
            bus: 0,
            device: 0,
            function: 0,
            subordinate_bus: 0,
            secondary_bus: 0,
            primary_bus: 0,
            mmio: PciConfigMmio::new(0, 0), // Dummy mmio for placeholder
            has_secondary_link: false,
            is_mulitple_function: false,
        }
    }

    pub fn host_bridge(address: PciConfigAddress, bus_begin: u8, is_mulitple_function: bool, function: u8) -> Self {
        Self {
            bus: bus_begin,
            device: 0,
            function,
            subordinate_bus: bus_begin,
            secondary_bus: bus_begin,
            primary_bus: bus_begin,
            mmio: PciConfigMmio::new(address, CONFIG_LENTH),
            has_secondary_link: false,
            is_mulitple_function,
        }
    }

    pub fn next_bridge(&self, address: PciConfigAddress, has_secondary_link: bool, is_mulitple_function: bool, function: u8) -> Self {
        let mmio = PciConfigMmio::new(address, CONFIG_LENTH);
        Self {
            bus: self.subordinate_bus + 1,
            device: 0,
            function,
            subordinate_bus: self.subordinate_bus + 1,
            secondary_bus: self.subordinate_bus + 1,
            primary_bus: self.bus,
            mmio,
            has_secondary_link,
            is_mulitple_function,
        }
    }

    pub fn update_bridge_bus(&mut self) {
        // Skip update for placeholder bridges (they don't have real mmio)
        if self.mmio.is_placeholder() {
            return;
        }
        // we need to update the bridge bus number if we want linux not to update bus number
        unsafe {
            let ptr = self.mmio.access::<u32>(0x18);
            let mut value = ptr.read_volatile();
            value.set_bits(16..24, self.subordinate_bus.into());
            value.set_bits(8..16, self.secondary_bus.into());
            value.set_bits(0..8, self.primary_bus.into());
            ptr.write_volatile(value);
        }
    }

    pub fn set_has_secondary_link(&mut self, value: bool) {
        self.has_secondary_link = value;
    }
}

/* In fact, the size will be managed by the pci_mmio_handler, so only base is needed here */
pub struct RootComplex {
    pub mmio_base: PciConfigAddress,
    pub accessor: Arc<dyn PciConfigAccessor>, // Unified accessor
}

impl RootComplex {
    fn __enumerate<B: BarAllocator>(
        &mut self,
        range: Option<Range<usize>>,
        bar_alloc: Option<B>,
    ) -> PciIterator<B> {
        let mmio_base = self.mmio_base;
        let range = range.unwrap_or_else(|| 0..0x100);
        PciIterator {
            allocator: bar_alloc,
            stack: vec![Bridge::placeholder()],
            segment: mmio_base,
            bus_range: range,
            function: 0,
            is_mulitple_function: false,
            is_finish: false,
            accessor: self.accessor.clone(), // accessor to iterator
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
    base_to_bdf: BTreeMap<PciConfigAddress, Bdf>,
}

impl VirtualRootComplex {
    pub fn new() -> Self {
        Self {
            devs: BTreeMap::new(),
            base_to_bdf: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        bdf: Bdf,
        dev: VirtualPciConfigSpace,
    ) -> Option<VirtualPciConfigSpace> {
        let base = dev.get_base();
        self.base_to_bdf.insert(base, bdf);
        self.devs.insert(bdf, dev)
    }

    pub fn devs(&mut self) -> &mut BTreeMap<Bdf, VirtualPciConfigSpace> {
        &mut self.devs
    }

    pub fn get(&self, bdf: &Bdf) -> Option<&VirtualPciConfigSpace> {
        self.devs.get(bdf)
    }

    pub fn get_mut(&mut self, bdf: &Bdf) -> Option<&mut VirtualPciConfigSpace> {
        self.devs.get_mut(bdf)
    }

    /* because the base of device may discontinuous，get device by base is simpler */
    pub fn get_device_by_base(
        &mut self,
        base: PciConfigAddress,
    ) -> Option<&mut VirtualPciConfigSpace> {
        let bdf = self.base_to_bdf.get(&base).copied()?;
        self.devs.get_mut(&bdf)
    }
}

#[derive(Debug)]
pub struct CapabilityIterator {
    backend: Arc<dyn PciRW>,
    offset: PciConfigAddress,
}

impl CapabilityIterator {
    pub fn get_offset(&self) -> PciConfigAddress {
        self.offset
    }

    pub fn get_next_cap(&mut self) -> HvResult {
        let address =
            self.backend.read(self.offset, 2).unwrap().get_bits(8..16) as PciConfigAddress;
        self.offset = address;
        Ok(())
    }

    pub fn get_id(&self) -> PciConfigAddress {
        self.backend.read(self.offset, 2).unwrap().get_bits(0..8) as PciConfigAddress
    }

    pub fn get_extension(&self) -> u16 {
        self.backend.read(self.offset, 4).unwrap().get_bits(16..32) as u16
    }
}

impl Iterator for CapabilityIterator {
    type Item = PciCapability;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == 0x34 {
            let first_cap_offset = self.backend.read(0x34, 1).unwrap() as PciConfigAddress;
            if first_cap_offset == 0 {
                return None;
            }
            self.offset = first_cap_offset;
        }

        while self.get_offset() != 0 {
            debug!("get cap {:#x}", self.get_offset());
            // Get current capability before moving to next
            let cap =
                PciCapability::from_address(self.get_offset(), self.get_id(), self.get_extension());
            // Move to next capability
            let _ = self.get_next_cap();
            if let Some(cap) = cap {
                return Some(cap);
            }
        }
        None
    }
}

#[derive(Clone)]
pub enum PciCapability {
    // Power management capability, Cap ID = `0x01`
    PowerManagement(PciCapabilityRegion),
    // Accelerated graphics port capability, Cap ID = `0x02`
    AcceleratedGraphicsPort(PciCapabilityRegion),
    // Vital product data capability, Cap ID = `0x3`
    VitalProductData(PciCapabilityRegion),
    // Slot identification capability, Cap ID = `0x04`
    SlotIdentification(PciCapabilityRegion),
    // Message signalling interrupts capability, Cap ID = `0x05`
    Msi(PciCapabilityRegion),
    // CompactPCI HotSwap capability, Cap ID = `0x06`
    CompactPCIHotswap(PciCapabilityRegion),
    // PCI-X capability, Cap ID = `0x07`
    PciX(PciCapabilityRegion),
    // HyperTransport capability, Cap ID = `0x08`
    HyperTransport(PciCapabilityRegion),
    // Vendor-specific capability, Cap ID = `0x09`
    Vendor(PciCapabilityRegion),
    // Debug port capability, Cap ID = `0x0A`
    DebugPort(PciCapabilityRegion),
    // CompactPCI Central Resource Control capability, Cap ID = `0x0B`
    CompactPCICentralResourceControl(PciCapabilityRegion),
    // PCI Standard Hot-Plug Controller capability, Cap ID = `0x0C`
    PciHotPlugControl(PciCapabilityRegion),
    // Bridge subsystem vendor/device ID capability, Cap ID = `0x0D`
    BridgeSubsystemVendorId(PciCapabilityRegion),
    // AGP Target PCI-PCI bridge capability, Cap ID = `0x0E`
    AGP3(PciCapabilityRegion),
    // PCI Express capability, Cap ID = `0x10`
    PciExpress(PciCapabilityRegion),
    // MSI-X capability, Cap ID = `0x11`
    MsiX(PciCapabilityRegion),
    // Unknown capability
    Unknown(PciCapabilityRegion),
}

impl PciCapability {
    fn from_address(
        offset: PciConfigAddress,
        id: PciConfigAddress,
        extension: u16,
    ) -> Option<PciCapability> {
        match id {
            0x00 => None,
            0x01 => Some(PciCapability::PowerManagement(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x02 => Some(PciCapability::AcceleratedGraphicsPort(
                PciCapabilityRegion::new(offset, extension),
            )),
            0x03 => Some(PciCapability::VitalProductData(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x04 => Some(PciCapability::SlotIdentification(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x05 => Some(PciCapability::Msi(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x06 => Some(PciCapability::CompactPCIHotswap(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x07 => Some(PciCapability::PciX(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x08 => Some(PciCapability::HyperTransport(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x09 => Some(PciCapability::Vendor(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x0A => Some(PciCapability::DebugPort(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x0B => Some(PciCapability::CompactPCICentralResourceControl(
                PciCapabilityRegion::new(offset, extension),
            )),
            0x0C => Some(PciCapability::PciHotPlugControl(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x0D => Some(PciCapability::BridgeSubsystemVendorId(
                PciCapabilityRegion::new(offset, extension),
            )),
            0x0E => Some(PciCapability::AGP3(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x10 => Some(PciCapability::PciExpress(PciCapabilityRegion::new(
                offset, extension,
            ))),
            0x11 => Some(PciCapability::MsiX(PciCapabilityRegion::new(
                offset, extension,
            ))),
            _ => Some(PciCapability::Unknown(PciCapabilityRegion::new(
                offset, extension,
            ))),
        }
    }

    fn get_offset(&self) -> PciConfigAddress {
        match *self {
            PciCapability::PowerManagement(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::AcceleratedGraphicsPort(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::VitalProductData(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::SlotIdentification(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::Msi(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::CompactPCIHotswap(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::PciX(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::HyperTransport(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::Vendor(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::DebugPort(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::CompactPCICentralResourceControl(PciCapabilityRegion {
                offset, ..
            }) => offset,
            PciCapability::PciHotPlugControl(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::BridgeSubsystemVendorId(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::AGP3(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::PciExpress(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::MsiX(PciCapabilityRegion { offset, .. }) => offset,
            PciCapability::Unknown(PciCapabilityRegion { offset, .. }) => offset,
        }
    }
}

impl Debug for PciCapability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            PciCapability::PowerManagement(PciCapabilityRegion { offset, .. }) => {
                write!(f, "PowerManagement {:x}", offset)
            }
            PciCapability::AcceleratedGraphicsPort(PciCapabilityRegion { offset, .. }) => {
                write!(f, "AcceleratedGraphicsPort {:x}", offset)
            }
            PciCapability::VitalProductData(PciCapabilityRegion { offset, .. }) => {
                write!(f, "VitalProductData {:x}", offset)
            }
            PciCapability::SlotIdentification(PciCapabilityRegion { offset, .. }) => {
                write!(f, "SlotIdentification {:x}", offset)
            }
            PciCapability::Msi(PciCapabilityRegion { offset, .. }) => write!(f, "Msi {:x}", offset),
            PciCapability::CompactPCIHotswap(PciCapabilityRegion { offset, .. }) => {
                write!(f, "CompactPCIHotswap {:x}", offset)
            }
            PciCapability::PciX(PciCapabilityRegion { offset, .. }) => {
                write!(f, "PciX {:x}", offset)
            }
            PciCapability::HyperTransport(PciCapabilityRegion { offset, .. }) => {
                write!(f, "HyperTransport {:x}", offset)
            }
            PciCapability::Vendor(PciCapabilityRegion { offset, .. }) => {
                write!(f, "Vendor {:x}", offset)
            }
            PciCapability::DebugPort(PciCapabilityRegion { offset, .. }) => {
                write!(f, "DebugPort {:x}", offset)
            }
            PciCapability::CompactPCICentralResourceControl(PciCapabilityRegion {
                offset, ..
            }) => write!(f, "CompactPCICentralResourceControl {:x}", offset),
            PciCapability::PciHotPlugControl(PciCapabilityRegion { offset, .. }) => {
                write!(f, "PciHotPlugControl {:x}", offset)
            }
            PciCapability::BridgeSubsystemVendorId(PciCapabilityRegion { offset, .. }) => {
                write!(f, "BridgeSubsystemVendorId {:x}", offset)
            }
            PciCapability::AGP3(PciCapabilityRegion { offset, .. }) => {
                write!(f, "AGP3 {:x}", offset)
            }
            PciCapability::PciExpress(PciCapabilityRegion { offset, .. }) => {
                write!(f, "PciExpress {:x}", offset)
            }
            PciCapability::MsiX(PciCapabilityRegion { offset, .. }) => {
                write!(f, "MsiX {:x}", offset)
            }
            PciCapability::Unknown(PciCapabilityRegion { offset, .. }) => {
                write!(f, "Unknown {:x}", offset)
            }
        }
    }
}

#[derive(Clone)]
pub struct PciCapabilityRegion {
    offset: PciConfigAddress,
    extension: u16,
}

impl PciCapabilityRegion {
    pub fn new(offset: PciConfigAddress, extension: u16) -> Self {
        Self { offset, extension }
    }
}

pub type PciCapabilityList = BTreeMap<PciConfigAddress, PciCapability>;

impl VirtualPciConfigSpace {
    fn _capability_enumerate(&self, backend: Arc<dyn PciRW>) -> CapabilityIterator {
        CapabilityIterator {
            backend,
            offset: 0x34,
        }
    }

    pub fn capability_enumerate(&mut self) {
        let mut capabilities = PciCapabilityList::new();
        for capability in self._capability_enumerate(self.backend.clone()) {
            match capability {
                PciCapability::Msi(_) => {}
                PciCapability::MsiX(_) => {}
                PciCapability::PciExpress(_) => {}
                _ => {}
            }
            capabilities.insert(capability.get_offset(), capability);
        }
        info!("capability {:#?}", capabilities);
        self.capabilities = capabilities;
    }

    //TODO: check secondary link by read cap
    pub fn has_secondary_link(&self) -> bool {
        match self.config_type {
            HeaderType::PciBridge => {
                // Find PciExpress capability
                // for (_, capability) in &self.capabilities {
                //     if let PciCapability::PciExpress(PciCapabilityRegion { offset, .. }) = capability {
                //         // Read PCIe Capability Register at offset + 0x00
                //         // Bits 4:0 contain the Device/Port Type
                //         if let Ok(cap_reg) = self.backend.read(*offset, 2) {
                //             let type_val = (cap_reg as u16).get_bits(0..5);
                //             if type_val == PCI_EXP_TYPE_ROOT_PORT || type_val == PCI_EXP_TYPE_PCIE_BRIDGE {
                //                 return true;
                //             } else if type_val == PCI_EXP_TYPE_UPSTREAM || type_val == PCI_EXP_TYPE_DOWNSTREAM {
                //                 // Parent check is not implemented, set to false for now
                //                 return false;
                //             }
                //         }
                //         break;
                //     }
                // }
                // false
                #[cfg(feature = "dwc_pcie")]
                return true;
                #[cfg(not(feature = "dwc_pcie"))]
                return false;
            }
            _ => false,
        }
    }
}
