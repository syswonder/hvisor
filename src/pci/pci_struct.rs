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
use core::{cmp::Ordering, fmt::Debug, ops::{Deref, DerefMut, Range}, str::FromStr};
use spin::RwLock;

use super::{
    config_accessors::{PciConfigAccessor, PciConfigMmio},
    mem_alloc::BarAllocator,
    pci_access::{
        Bar, EndpointField, EndpointHeader, HeaderType, PciBarRW, PciBridgeHeader, PciCommand,
        PciConfigHeader, PciField, PciHeaderRW, PciMem, PciMemType, PciRW, PciRomRW,
    },
    PciConfigAddress,
    pci_access::{BaseClass, SubClass, Interface, DeviceId, VendorId, DeviceRevision},
};

use crate::{error::{HvErrorNum, HvResult}, pci::vpci_dev::VpciDevType};

type VirtualPciConfigBits = BitArr!(for BIT_LENTH, in u8, Lsb0);

#[derive(Clone, Debug)]
pub struct ConfigValue {
    id: (DeviceId, VendorId),
    class_and_revision_id: (BaseClass, SubClass, Interface, DeviceRevision),
    bar_value: [u32; 6],
    rom_value: u32,
}

impl Default for ConfigValue {
    fn default() -> Self {
        Self {
            id: (0xFFFFu16, 0xFFFFu16),
            class_and_revision_id: (0xFFu8, 0u8, 0u8, 0u8),
            bar_value: [0; 6],
            rom_value: 0,
        }
    }
}

impl ConfigValue {
    pub fn new(id: (DeviceId, VendorId), class_and_revision_id: (BaseClass, SubClass, Interface, DeviceRevision)) -> Self {
        Self {
            id,
            class_and_revision_id,
            bar_value: [0; 6],
            rom_value: 0,
        }
    }

    pub fn get_id(&self) -> (DeviceId, VendorId) {
        self.id
    }

    pub fn set_id(&mut self, id: (DeviceId, VendorId)) {
        self.id = id;
    }

    pub fn get_class_and_revision_id(&self) -> (BaseClass, SubClass, Interface, DeviceRevision) {
        self.class_and_revision_id
    }

    pub fn get_class(&self) -> (BaseClass, SubClass, Interface) {
        let (base, sub, interface, _) = self.class_and_revision_id;
        (base, sub, interface)
    }

    pub fn get_revision(&self) -> DeviceRevision {
        self.class_and_revision_id.3
    }

    pub fn set_class_and_revision_id(&mut self, class_and_revision_id: (BaseClass, SubClass, Interface, DeviceRevision)) {
        self.class_and_revision_id = class_and_revision_id;
    }

    pub fn set_class(&mut self, class: (BaseClass, SubClass, Interface)) {
        let (_, _, _, revision) = self.class_and_revision_id;
        self.class_and_revision_id = (class.0, class.1, class.2, revision);
    }

    pub fn get_bar_value(&self, slot: usize) -> u32 {
        if slot < 6 {
            self.bar_value[slot]
        } else {
            0
        }
    }

    pub fn set_bar_value(&mut self, slot: usize, value: u32) {
        if slot < 6 {
            self.bar_value[slot] = value;
        }
    }

    pub fn get_bar_value_ref(&self, slot: usize) -> &u32 {
        &self.bar_value[slot]
    }

    pub fn get_bar_value_ref_mut(&mut self, slot: usize) -> &mut u32 {
        &mut self.bar_value[slot]
    }

    pub fn get_rom_value(&self) -> u32 {
        self.rom_value
    }

    pub fn set_rom_value(&mut self, value: u32) {
        self.rom_value = value;
    }
}

const MAX_DEVICE: u8 = 31;
const MAX_FUNCTION: u8 = 7;
pub const CONFIG_LENTH: u64 = 256;
pub const BIT_LENTH: usize = 128*8;

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
        bits[0x0..0x4].fill(true);   // ID
        bits[0x08..0x0c].fill(true); // CLASS
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

    pub fn set_bits(&mut self, range: Range<usize>) {
        self.bits[range].fill(true);
    }
}


/* VirtualPciConfigSpace
 * bdf: the bdf hvisor seeing(same with the bdf without hvisor)
 * vbdf: the bdf zone seeing, it can set just you like without sr-iov
 * config_value: tmp value for config space
 * control: control the satus of rw every bit in config space
 * access: Determines whether the variable is read from config_value or hw
 * backend: the hw rw interface
 */
#[derive(Clone)]
pub struct VirtualPciConfigSpace {
    host_bdf: Bdf,
    parent_bdf: Bdf,
    bdf: Bdf,
    vbdf: Bdf,
    config_type: HeaderType,

    base: PciConfigAddress,

    config_value: ConfigValue,
    control: VirtualPciConfigControl,
    access: VirtualPciAccessBits,

    backend: Arc<dyn PciRW>,

    bararr: Bar,
    rom: PciMem,
    capabilities: PciCapabilityList,

    dev_type: VpciDevType,
}

#[derive(Clone)]
pub struct ArcRwLockVirtualPciConfigSpace(Arc<RwLock<VirtualPciConfigSpace>>);

impl ArcRwLockVirtualPciConfigSpace {
    pub fn new(dev: VirtualPciConfigSpace) -> Self {
        Self(Arc::new(RwLock::new(dev)))
    }
    
    pub fn inner(&self) -> &Arc<RwLock<VirtualPciConfigSpace>> {
        &self.0
    }

    pub fn access(&self, offset: PciConfigAddress, size: usize) -> bool {
        self.0.read().access(offset, size)
    }

    pub fn get_bdf(&self) -> Bdf {
        self.0.read().get_bdf()
    }

    pub fn get_vbdf(&self) -> Bdf {
        self.0.read().get_vbdf()
    }

    pub fn get_dev_type(&self) -> VpciDevType {
        self.0.read().get_dev_type()
    }

    pub fn get_config_type(&self) -> HeaderType {
        self.0.read().get_config_type()
    }

    pub fn get_bararr(&self) -> Bar {
        self.0.read().get_bararr()
    }

    pub fn get_rom(&self) -> PciMem {
        self.0.read().get_rom()
    }

    pub fn read_emu(&self, field: EndpointField) -> HvResult<usize> {
        self.0.write().read_emu(field)
    }

    pub fn read_emu64(&self, field: EndpointField) -> HvResult<u64> {
        self.0.write().read_emu64(field)
    }

    pub fn write_emu(&self, field: EndpointField, value: usize) -> HvResult {
        self.0.write().write_emu(field, value)
    }

    // Legacy method for backward compatibility
    // pub fn write_emu_legacy(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
    //     self.0.write().write_emu_legacy(offset, size, value)
    // }

    pub fn read_hw(&self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
        self.0.write().read_hw(offset, size)
    }

    pub fn write_hw(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        self.0.write().write_hw(offset, size, value)
    }


    /// Execute a closure with a reference to the bar at the given slot
    pub fn with_bar_ref<F, R>(&self, slot: usize, f: F) -> R
    where
        F: FnOnce(&PciMem) -> R,
    {
        let guard = self.0.read();
        let bar = guard.get_bar_ref(slot);
        f(bar)
    }

    /// Execute a closure with a mutable reference to the bar at the given slot
    pub fn with_bar_ref_mut<F, R>(&self, slot: usize, f: F) -> R
    where
        F: FnOnce(&mut PciMem) -> R,
    {
        let mut guard = self.0.write();
        let bar = guard.get_bar_ref_mut(slot);
        f(bar)
    }

    /// Execute a closure with a reference to the config_value
    pub fn with_config_value<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ConfigValue) -> R,
    {
        let guard = self.0.read();
        f(&guard.config_value)
    }

    /// Execute a closure with a mutable reference to the config_value
    pub fn with_config_value_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ConfigValue) -> R,
    {
        let mut guard = self.0.write();
        f(&mut guard.config_value)
    }

    /// Execute a closure with a reference to the rom
    pub fn with_rom_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PciMem) -> R,
    {
        let guard = self.0.read();
        let rom = &guard.rom;
        f(rom)
    }

    /// Execute a closure with a mutable reference to the rom
    pub fn with_rom_ref_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PciMem) -> R,
    {
        let mut guard = self.0.write();
        let rom = &mut guard.rom;
        f(rom)
    }

    /// Execute a closure with a reference to the capabilities list
    pub fn with_cap<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PciCapabilityList) -> R,
    {
        let guard = self.0.read();
        f(&guard.capabilities)
    }

    pub fn read(&self) -> spin::RwLockReadGuard<'_, VirtualPciConfigSpace> {
        self.0.read()
    }

    pub fn write(&self) -> spin::RwLockWriteGuard<'_, VirtualPciConfigSpace> {
        self.0.write()
    }
}

impl Debug for ArcRwLockVirtualPciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.read().fmt(f)
    }
}

// impl core::ops::Deref for ArcRwLockVirtualPciConfigSpace {
//     type Target = Arc<RwLock<VirtualPciConfigSpace>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<VirtualPciConfigSpace> for ArcRwLockVirtualPciConfigSpace {
//     fn from(dev: VirtualPciConfigSpace) -> Self {
//         Self::new(dev)
//     }
// }

// impl From<Arc<RwLock<VirtualPciConfigSpace>>> for ArcRwLockVirtualPciConfigSpace {
//     fn from(arc: Arc<RwLock<VirtualPciConfigSpace>>) -> Self {
//         Self(arc)
//     }
// }

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

    pub fn get_bar_ref(&self, slot: usize) -> &PciMem {
        &self.bararr[slot]
    }

    pub fn get_bar_ref_mut(&mut self, slot: usize) -> &mut PciMem {
        &mut self.bararr[slot]
    }


    pub fn set_bar_size_read(&mut self, slot: usize) {
        self.bararr[slot].set_size_read();
    }

    pub fn set_bar_virtual_value(&mut self, slot: usize, value: u64) {
        self.bararr[slot].set_virtual_value(value);
    }

    pub fn set_bar_physical_value(&mut self, slot: usize, value: u64) {
        self.bararr[slot].set_value(value);
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

    pub fn get_config_value(&self) -> &ConfigValue {
        &self.config_value
    }

    /// Execute a closure with a mutable reference to the config_value
    pub fn with_config_value_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ConfigValue) -> R,
    {
        f(&mut self.config_value)
    }

    /// Execute a closure with a mutable reference to the bararr
    pub fn with_bararr_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Bar) -> R,
    {
        f(&mut self.bararr)
    }

    pub fn with_cap_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut PciCapabilityList) -> R,
    {
        f(&mut self.capabilities)
    }

    pub fn with_access_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut VirtualPciAccessBits) -> R,
    {
        f(&mut self.access)
    }

    // TODO: check whether need update config
    pub fn update_config(&mut self, offset: PciConfigAddress, size: usize, _value: usize) {
        match self.get_config_type() {
            HeaderType::Endpoint => {
                match EndpointField::from(offset as usize, size) {
                    EndpointField::Bar(_) => {
                        // Bar values are cached in config_value.bar_value, updated in write_emu
                    }
                    _ => {}
                }
            }
            _ => {
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
    pub(super) fn virt_dev_init_default(
        bdf: Bdf,
        base: PciConfigAddress,
        dev_type: VpciDevType,
        config_value: ConfigValue,
        bararr: Bar,
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: bdf,
            config_type: HeaderType::Endpoint,
            base,
            config_value,
            control: VirtualPciConfigControl::virt_dev(),
            access: VirtualPciAccessBits::virt_dev(),
            backend: Arc::new(EndpointHeader::new_with_region(PciConfigMmio::new(base, CONFIG_LENTH))),
            bararr,
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type,
        }
    }

    pub fn virt_dev(
        bdf: Bdf,
        base: PciConfigAddress,
        dev_type: VpciDevType,
    ) -> Self {
        crate::pci::vpci_dev::virt_dev_init(bdf, base, dev_type)
    }
    pub fn endpoint(
        bdf: Bdf,
        base: PciConfigAddress,
        backend: Arc<dyn PciRW>,
        bararr: Bar,
        rom: PciMem,
        class_and_revision_id: (DeviceRevision, BaseClass, SubClass, Interface),
        id: (DeviceId, VendorId),
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::Endpoint,
            base,
            config_value: ConfigValue::new(id, class_and_revision_id),
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
        class_and_revision_id: (DeviceRevision, BaseClass, SubClass, Interface),
        id: (DeviceId, VendorId),
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::PciBridge,
            base,
            config_value: ConfigValue::new(id, class_and_revision_id),
            control: VirtualPciConfigControl::bridge(),
            access: VirtualPciAccessBits::bridge(),
            backend,
            bararr,
            rom,
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
        }
    }

    pub fn unknown(bdf: Bdf, base: PciConfigAddress, backend: Arc<dyn PciRW>, id: (DeviceId, VendorId)) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::Endpoint,
            base,
            // Default class: base=0xFF, others 0, revision 0
            config_value: ConfigValue::new(id, (0xFFu8,0u8,0u8,0u8)),
            control: VirtualPciConfigControl::endpoint(),
            access: VirtualPciAccessBits::endpoint(),
            backend,
            bararr: Bar::default(),
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
        }
    }

    pub fn host_bridge(bdf: Bdf, base: PciConfigAddress, backend: Arc<dyn PciRW>, class_and_revision_id: (DeviceRevision, BaseClass, SubClass, Interface)) -> Self {
        Self {
            host_bdf: bdf,
            parent_bdf: bdf,
            bdf: bdf,
            vbdf: bdf,
            config_type: HeaderType::Endpoint,
            base,
            config_value: ConfigValue::new((0xFFFFu16, 0xFFFFu16), class_and_revision_id), // Default ID for host bridge
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
     * Note: space field removed, bar values are cached in config_value.bar_value
     */
    pub fn config_value_init(&mut self) {
        // Initialize bar_value cache from bar values
        for slot in 0..6 {
            let bar_value = self.bararr[slot].get_value();
            self.config_value.set_bar_value(slot, bar_value as u32);
        }
    }
}

impl VirtualPciConfigSpace {
    pub fn read_hw(&mut self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
        let r = self.backend.read(offset, size);
        if let Ok(value) = r {
            self.update_config(offset, size, value);
        }
        r
    }

    pub fn write_hw(&mut self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        if self.writable(offset, size) {
            let r = self.backend.write(offset, size, value);
            if r.is_ok() {
                self.update_config(offset, size, value);
            }
            r
        } else {
            hv_result_err!(EPERM, "pci: invalid write to hw")
        }
    }

    pub fn read_emu(&mut self, field: EndpointField) -> HvResult<usize> {
        let offset = field.to_offset() as PciConfigAddress;
        let size = field.size();
    
        match field {
            EndpointField::ID => {
                // Read ID from cached config_value.id field
                let id = self.config_value.get_id();
                let id_value = ((id.0 as u32) << 16) | (id.1 as u32);
                Ok(id_value as usize)
            }
            EndpointField::RevisionIDAndClassCode => {
                let (base, sub, interface, revision) = self.config_value.get_class_and_revision_id();
                let value = ((base as u32) << 24)
                    | ((sub as u32) << 16)
                    | ((interface as u32) << 8)
                    | (revision as u32);
                Ok(value as usize)
            }
            EndpointField::Bar(slot) => {
                // Read bar_value from cache
                if slot < 6 {
                    Ok(self.config_value.get_bar_value(slot) as usize)
                } else {
                    hv_result_err!(EFAULT, "pci: invalid bar slot: {slot}")
                }
            }
            EndpointField::ExpansionRomBar => {
                // Read rom_value from cache
                Ok(self.config_value.get_rom_value() as usize)
            }
            _ => {
                // For other fields, read from backend
                warn!("read emu {:#?} failed, try read from hw", field);
                self.backend.read(offset, size)
            }
        }
    }

    pub fn read_emu64(&mut self, field: EndpointField) -> HvResult<u64> {
        // Read 64-bit value (used for bar64)
        // For Bar(slot), read from slot and slot+1
        match field {
            EndpointField::Bar(slot) if slot < 5 => {
                // Read from bar_value cache
                let low = self.config_value.get_bar_value(slot) as u64;
                let high = self.config_value.get_bar_value(slot + 1) as u64;
                Ok(low | (high << 32))
            }
            _ => {
                // Fallback to backend read
                warn!("read emu64 {:#?} failed", field);
                Ok(0)
            }
        }
    }

    pub fn write_emu(&mut self, field: EndpointField, value: usize) -> HvResult {
        match field {
            EndpointField::Bar(slot) => {
                // Update bar_value cache when writing bar
                if slot < 6 {
                    self.config_value.set_bar_value(slot, value as u32);
                }
            }
            EndpointField::ExpansionRomBar => {
                // Update rom_value cache when writing rom bar
                self.config_value.set_rom_value(value as u32);
            }
            _ => {
                // For other fields, write to backend
                warn!("write emu {:#?} denied", field);
            }
        }
        Ok(())
    }
}

    // Legacy method for backward compatibility - converts offset/size to EndpointField
    // pub fn write_emu_legacy(&mut self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
    //     let field = EndpointField::from(offset as usize, size);
    //     self.write_emu(field, value)
    // }


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
    fn get_pci_addr_base(&self, parent_bus: u8, bdf: Bdf) -> PciConfigAddress {
        match self.accessor.get_pci_addr_base(bdf, parent_bus) {
            Ok(addr) => addr,
            Err(_) => 0x0,
        }
    }

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
        let pci_addr_base = self.get_pci_addr_base(parent_bus, bdf);
        info!("get node {:x} {:#?}", address, bdf);

        let region = PciConfigMmio::new(address, CONFIG_LENTH);
        let pci_header = PciConfigHeader::new_with_region(region);
        let (vender_id, device_id) = pci_header.id();

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

        let class_and_revision = pci_header.revision_and_class();

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
                let mut node = VirtualPciConfigSpace::endpoint(
                    bdf,
                    pci_addr_base,
                    ep,
                    bararr,
                    rom,
                    class_and_revision,
                    (device_id, vender_id),
                );

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
                let mut node = VirtualPciConfigSpace::bridge(
                    bdf,
                    pci_addr_base,
                    bridge,
                    bararr,
                    rom,
                    class_and_revision,
                    (device_id, vender_id),
                );

                let _ = node.capability_enumerate();

                Some(node)
            }
            _ => {
                warn!("unknown type");
                let pci_header = Arc::new(pci_header);
                Some(VirtualPciConfigSpace::unknown(bdf, pci_addr_base, pci_header, (device_id, vender_id)))
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
                        let value = a.alloc_memory32(bararr[i].get_size() as u64).unwrap();
                        bararr[i].set_value(value as u64);
                        bararr[i].set_virtual_value(value as u64);
                        let _ = dev.write_bar(i as u8, value as u32);
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
                node.config_value_init();
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
                self.next(match node.config_value.get_class().0 {
                    // class code 0x6 is bridge and class.1 0x0 is host bridge
                    0x6 if node.config_value.get_class().1 != 0x0 => {
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
    devs: BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace>,
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
    ) -> Option<ArcRwLockVirtualPciConfigSpace> {
        let base = dev.get_base();
        info!("pci insert base {:#x} to bdf {:#?}", base, bdf);
        self.base_to_bdf.insert(base, bdf);
        self.devs.insert(bdf, ArcRwLockVirtualPciConfigSpace::new(dev))
    }

    pub fn devs(&mut self) -> &mut BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace> {
        &mut self.devs
    }

    pub fn get(&self, bdf: &Bdf) -> Option<&ArcRwLockVirtualPciConfigSpace> {
        self.devs.get(bdf)
    }

    pub fn get_mut(&mut self, bdf: &Bdf) -> Option<&mut ArcRwLockVirtualPciConfigSpace> {
        self.devs.get_mut(bdf)
    }

    /* because the base of device may discontinuous，get device by base is simpler */
    pub fn get_device_by_base(
        &mut self,
        base: PciConfigAddress,
    ) -> Option<ArcRwLockVirtualPciConfigSpace> {
        let bdf = self.base_to_bdf.get(&base).copied()?;
        self.devs.get(&bdf).cloned()
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
                PciCapability::from_address(self.get_offset(), self.get_id(), self.backend.clone());
            // Move to next capability
            let _ = self.get_next_cap();
            if let Some(cap) = cap {
                return Some(cap);
            }
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    // Power management capability, Cap ID = `0x01`
    PowerManagement,
    // Accelerated graphics port capability, Cap ID = `0x02`
    AcceleratedGraphicsPort,
    // Vital product data capability, Cap ID = `0x3`
    VitalProductData,
    // Slot identification capability, Cap ID = `0x04`
    SlotIdentification,
    // Message signalling interrupts capability, Cap ID = `0x05`
    Msi,
    // CompactPCI HotSwap capability, Cap ID = `0x06`
    CompactPCIHotswap,
    // PCI-X capability, Cap ID = `0x07`
    PciX,
    // HyperTransport capability, Cap ID = `0x08`
    HyperTransport,
    // Vendor-specific capability, Cap ID = `0x09`
    Vendor,
    // Debug port capability, Cap ID = `0x0A`
    DebugPort,
    // CompactPCI Central Resource Control capability, Cap ID = `0x0B`
    CompactPCICentralResourceControl,
    // PCI Standard Hot-Plug Controller capability, Cap ID = `0x0C`
    PciHotPlugControl,
    // Bridge subsystem vendor/device ID capability, Cap ID = `0x0D`
    BridgeSubsystemVendorId,
    // AGP Target PCI-PCI bridge capability, Cap ID = `0x0E`
    AGP3,
    // PCI Express capability, Cap ID = `0x10`
    PciExpress,
    // MSI-X capability, Cap ID = `0x11`
    MsiX,
    // Unknown capability
    Unknown,
}

impl Debug for CapabilityType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CapabilityType::PowerManagement => write!(f, "PowerManagement(0x01)"),
            CapabilityType::AcceleratedGraphicsPort => write!(f, "AcceleratedGraphicsPort(0x02)"),
            CapabilityType::VitalProductData => write!(f, "VitalProductData(0x03)"),
            CapabilityType::SlotIdentification => write!(f, "SlotIdentification(0x04)"),
            CapabilityType::Msi => write!(f, "Msi(0x05)"),
            CapabilityType::CompactPCIHotswap => write!(f, "CompactPCIHotswap(0x06)"),
            CapabilityType::PciX => write!(f, "PciX(0x07)"),
            CapabilityType::HyperTransport => write!(f, "HyperTransport(0x08)"),
            CapabilityType::Vendor => write!(f, "Vendor(0x09)"),
            CapabilityType::DebugPort => write!(f, "DebugPort(0x0A)"),
            CapabilityType::CompactPCICentralResourceControl => write!(f, "CompactPCICentralResourceControl(0x0B)"),
            CapabilityType::PciHotPlugControl => write!(f, "PciHotPlugControl(0x0C)"),
            CapabilityType::BridgeSubsystemVendorId => write!(f, "BridgeSubsystemVendorId(0x0D)"),
            CapabilityType::AGP3 => write!(f, "AGP3(0x0E)"),
            CapabilityType::PciExpress => write!(f, "PciExpress(0x10)"),
            CapabilityType::MsiX => write!(f, "MsiX(0x11)"),
            CapabilityType::Unknown => write!(f, "Unknown(0x00)"),
        }
    }
}

impl CapabilityType {
    fn from_id(id: PciConfigAddress) -> Self {
        match id {
            0x01 => CapabilityType::PowerManagement,
            0x02 => CapabilityType::AcceleratedGraphicsPort,
            0x03 => CapabilityType::VitalProductData,
            0x04 => CapabilityType::SlotIdentification,
            0x05 => CapabilityType::Msi,
            0x06 => CapabilityType::CompactPCIHotswap,
            0x07 => CapabilityType::PciX,
            0x08 => CapabilityType::HyperTransport,
            0x09 => CapabilityType::Vendor,
            0x0A => CapabilityType::DebugPort,
            0x0B => CapabilityType::CompactPCICentralResourceControl,
            0x0C => CapabilityType::PciHotPlugControl,
            0x0D => CapabilityType::BridgeSubsystemVendorId,
            0x0E => CapabilityType::AGP3,
            0x10 => CapabilityType::PciExpress,
            0x11 => CapabilityType::MsiX,
            _ => CapabilityType::Unknown,
        }
    }

    pub fn to_id(&self) -> PciConfigAddress {
        match self {
            CapabilityType::PowerManagement => 0x01,
            CapabilityType::AcceleratedGraphicsPort => 0x02,
            CapabilityType::VitalProductData => 0x03,
            CapabilityType::SlotIdentification => 0x04,
            CapabilityType::Msi => 0x05,
            CapabilityType::CompactPCIHotswap => 0x06,
            CapabilityType::PciX => 0x07,
            CapabilityType::HyperTransport => 0x08,
            CapabilityType::Vendor => 0x09,
            CapabilityType::DebugPort => 0x0A,
            CapabilityType::CompactPCICentralResourceControl => 0x0B,
            CapabilityType::PciHotPlugControl => 0x0C,
            CapabilityType::BridgeSubsystemVendorId => 0x0D,
            CapabilityType::AGP3 => 0x0E,
            CapabilityType::PciExpress => 0x10,
            CapabilityType::MsiX => 0x11,
            CapabilityType::Unknown => 0x00,
        }
    }
}

#[derive(Clone)]
pub struct PciCapability {
    cap_type: CapabilityType,
    region: Arc<RwLock<dyn PciCapabilityRegion>>,
}

impl PciCapability {
    pub fn get_type(&self) -> CapabilityType {
        self.cap_type
    }

    /// Execute a closure with a read lock on the capability region
    pub fn with_region<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn PciCapabilityRegion) -> R,
    {
        let guard = self.region.read();
        f(&*guard)
    }

    /// Execute a closure with a write lock on the capability region
    pub fn with_region_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut dyn PciCapabilityRegion) -> R,
    {
        let mut guard = self.region.write();
        f(&mut *guard)
    }

    fn from_address(
        offset: PciConfigAddress,
        id: PciConfigAddress,
        backend: Arc<dyn PciRW>,
    ) -> Option<PciCapability> {
        match CapabilityType::from_id(id) {
            CapabilityType::Unknown => None,
            CapabilityType::Msi => {
                let region = Arc::new(RwLock::new(StandardPciCapabilityRegion::new(offset, 32, backend)));
                return Some(PciCapability {
                    cap_type: CapabilityType::Msi,
                    region,
                });
            }
            _ => {        
                let region = Arc::new(RwLock::new(StandardPciCapabilityRegion::new(offset, 32, backend)));
                Some(PciCapability {
                    cap_type: CapabilityType::from_id(id),
                    region,
                })
            }
        }
    }

    pub fn new_virt(cap_type: CapabilityType, region: Arc<RwLock<dyn PciCapabilityRegion>>) -> Self {
        Self {
            cap_type,
            region,
        }
    }

    pub fn get_offset(&self) -> PciConfigAddress {
        self.with_region(|region| region.get_offset())
    }
    
    pub fn get_size(&self) -> usize {
        self.with_region(|region| region.get_size())
    }
    
    fn next_cap(&self) -> HvResult<PciConfigAddress> {
        self.with_region(|region| region.next_cap())
    }
}

impl Debug for PciCapability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.cap_type)
    }
}

pub trait PciCapabilityRegion: Send + Sync {
    /// Read from capability region at relative offset
    /// offset: relative offset from capability start (0 = capability start)
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<u32>;
    
    /// Write to capability region at relative offset
    /// offset: relative offset from capability start (0 = capability start)
    fn write(&mut self, offset: PciConfigAddress, size: usize, value: u32) -> HvResult;
    
    /// Get absolute offset of capability in config space
    fn get_offset(&self) -> PciConfigAddress;
    
    /// Get size of capability
    fn get_size(&self) -> usize;
    
    /// Get next capability offset by reading next pointer
    /// Default implementation: read 2 bytes at offset 0 (capability start), extract bits(8..16) as next pointer
    fn next_cap(&self) -> HvResult<PciConfigAddress> {
        let value = self.read(0, 2)?;
        let next_offset = (value as u16).get_bits(8..16) as PciConfigAddress;
        Ok(next_offset)
    }
}

pub struct StandardPciCapabilityRegion {
    offset: PciConfigAddress,
    size: usize,
    backend: Arc<dyn PciRW>,
}

impl StandardPciCapabilityRegion {
    pub fn new(offset: PciConfigAddress, size: usize, backend: Arc<dyn PciRW>) -> Self {
        Self { offset, size, backend }
    }
}

impl PciCapabilityRegion for StandardPciCapabilityRegion {
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<u32> {
        self.backend.read(self.offset + offset, size).map(|v| v as u32)
    }
    
    fn write(&mut self, offset: PciConfigAddress, size: usize, value: u32) -> HvResult {
        self.backend.write(self.offset + offset, size, value as usize)
    }
    
    fn get_offset(&self) -> PciConfigAddress {
        self.offset
    }
    
    fn get_size(&self) -> usize {
        self.size
    }
}

#[derive(Clone)]
pub struct PciCapabilityList(BTreeMap<PciConfigAddress, PciCapability>);

impl PciCapabilityList {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

// impl Default for PciCapabilityList {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl Deref for PciCapabilityList {
    type Target = BTreeMap<PciConfigAddress, PciCapability>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PciCapabilityList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for PciCapabilityList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PciCapabilityList {{\n")?;
        for (offset, capability) in &self.0 {
            write!(f, "0x{:x} {:?}\n", offset, capability)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

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
            match capability.get_type() {
                CapabilityType::Msi => {}
                CapabilityType::MsiX => {}
                CapabilityType::PciExpress => {}
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
                // warn!("has_secondary_link {:#?}", self.capabilities);
                // for (_, capability) in &self.capabilities {
                //     if capability.cap_type == CapabilityType::PciExpress {
                //         // Read PCIe Capability Register at offset + 0x00
                //         // Bits 4:0 contain the Device/Port Type
                //         let offset = capability.get_offset();
                //         if let Ok(cap_reg) = self.backend.read(offset, 2) {
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
                // #[cfg(feature = "dwc_pcie")]
                // return true;
                // #[cfg(not(feature = "dwc_pcie"))]
                return false;
            }
            _ => false,
        }
    }
}
