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
use spin::RwLock;

use super::{
    config_accessors::{PciConfigAccessor, PciConfigMmio},
    mem_alloc::BarAllocator,
    pci_access::{
        Bar, EndpointField, EndpointHeader, HeaderType, PciBarRW, PciBridgeHeader, PciCommand,
        PciConfigHeader, PciField, PciHeaderRW, PciMem, PciMemType, PciRW, PciRomRW,
    },
    pci_access::{BaseClass, DeviceId, DeviceRevision, Interface, SubClass, VendorId},
    PciConfigAddress,
};

use crate::{
    config::HvPciDevConfig,
    device::virtio_trampoline::VirtioPCIDataInfo,
    error::{HvErrorNum, HvResult},
    memory::MMIOAccess,
    pci::{
        msix::{MsixBackend, MsixTable, MsixTableEntry},
        vpci_dev::VpciDevType,
    },
};

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
    pub fn new(
        id: (DeviceId, VendorId),
        class_and_revision_id: (BaseClass, SubClass, Interface, DeviceRevision),
    ) -> Self {
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

    pub fn set_class_and_revision_id(
        &mut self,
        class_and_revision_id: (BaseClass, SubClass, Interface, DeviceRevision),
    ) {
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
pub const BIT_LENTH: usize = 512 * 8; // 4096 bytes = full PCIe extended config space

// PCIe Device/Port Type values
const PCI_EXP_TYPE_ROOT_PORT: u16 = 4;
const PCI_EXP_TYPE_UPSTREAM: u16 = 5;
const PCI_EXP_TYPE_DOWNSTREAM: u16 = 6;
const PCI_EXP_TYPE_PCIE_BRIDGE: u16 = 8;

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct Bdf {
    pub domain: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl Bdf {
    pub fn new(domain: u8, bus: u8, device: u8, function: u8) -> Self {
        Self {
            domain,
            bus,
            device,
            function,
        }
    }

    pub fn new_from_config(config: HvPciDevConfig) -> Self {
        Self::new(config.domain, config.bus, config.device, config.function)
    }

    pub fn domain(&self) -> u8 {
        self.domain
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

    pub fn is_host_bridge(&self, bus_begin: u8) -> bool {
        if (self.bus, self.device, self.function) == (bus_begin, 0, 0) {
            true
        } else {
            false
        }
    }

    pub fn requester_id(&self) -> u16 {
        let bus = self.bus as u16;
        let device = self.device as u16;
        let function = self.function as u16;
        bus << 8 | device << 3 | function
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

        let domain = u8::from_str_radix(parts[0], 16)
            .map_err(|_| HvErrorNum::EINVAL)
            .unwrap();

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
            domain,
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
        bits[0x0..0x4].fill(true); // ID
        bits[0x08..0x0c].fill(true); // CLASS
        bits[0x10..0x34].fill(true); // BARs and ROM
        bits[0x34..0x38].fill(true); // Capability Pointer
        bits[0x40..0x100].fill(true); // Capability region (caps start at 0x40)
        Self { bits }
    }

    pub fn bridge() -> Self {
        let mut bits = BitArray::ZERO;
        bits[0x10..0x18].fill(true); // BARs
        bits[0x38..0x3c].fill(true); // ROM
        bits[0x34..0x38].fill(true); // Capability Pointer
        bits[0x40..0x100].fill(true); // Capability region (caps start at 0x40)
        Self { bits }
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

#[derive(Clone, Copy, Debug)]
pub struct MsixInfo {
    pub bar_id: u8,
    pub offset: u64,
    pub entry_count: u32, // number of MSIX table entries
    pub bar_paddr: u64,   // physical address of the BAR
}

#[derive(Clone, Debug)]
pub struct MsiInfo {
    pub msi_count: u32,
    // doorbell vm write to trigger interrupt
    pub msi_doorbell: u64,
    pub msix_info: Option<MsixInfo>,
}

impl MsiInfo {
    pub fn new(msi_count: u32) -> Self {
        Self {
            msi_count,
            msi_doorbell: 0,
            msix_info: None,
        }
    }

    pub fn set_doorbell(&mut self, doorbell: u64) {
        self.msi_doorbell = doorbell;
    }

    pub fn set_msix_info(&mut self, bar_id: u8, offset: u64, entry_count: u32, bar_paddr: u64) {
        self.msix_info = Some(MsixInfo {
            bar_id,
            offset,
            entry_count,
            bar_paddr,
        });
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

    // MSI/MSIX info for this device
    msi_info: Option<MsiInfo>,
    msix_table: Option<Arc<RwLock<MsixTable>>>,
}

#[derive(Clone)]
pub struct VirtualPciConfigSpaceWithZone {
    pub zone_id: Option<u32>,
    pub config_space: VirtualPciConfigSpace,
}

impl core::ops::Deref for VirtualPciConfigSpaceWithZone {
    type Target = VirtualPciConfigSpace;

    fn deref(&self) -> &Self::Target {
        &self.config_space
    }
}

impl core::ops::DerefMut for VirtualPciConfigSpaceWithZone {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config_space
    }
}

impl Debug for VirtualPciConfigSpaceWithZone {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "zone_id: {:?}, ", self.zone_id)?;
        self.config_space.fmt(f)
    }
}

#[derive(Clone)]
pub struct ArcRwLockVirtualPciConfigSpace(Arc<RwLock<VirtualPciConfigSpaceWithZone>>);

impl ArcRwLockVirtualPciConfigSpace {
    pub fn new(dev: VirtualPciConfigSpace) -> Self {
        Self(Arc::new(RwLock::new(VirtualPciConfigSpaceWithZone {
            zone_id: None,
            config_space: dev,
        })))
    }

    pub fn get_zone_id(&self) -> Option<u32> {
        self.0.read().zone_id
    }

    pub fn set_zone_id(&self, zone_id: Option<u32>) {
        self.0.write().zone_id = zone_id;
    }

    pub fn access(&self, offset: PciConfigAddress, size: usize) -> bool {
        self.read().access(offset, size)
    }

    pub fn get_bdf(&self) -> Bdf {
        self.read().get_bdf()
    }

    pub fn get_vbdf(&self) -> Bdf {
        self.read().get_vbdf()
    }

    pub fn get_dev_type(&self) -> VpciDevType {
        self.read().get_dev_type()
    }

    pub fn get_config_type(&self) -> HeaderType {
        self.read().get_config_type()
    }

    pub fn get_bararr(&self) -> Bar {
        self.read().get_bararr()
    }

    pub fn get_rom(&self) -> PciMem {
        self.read().get_rom()
    }

    pub fn read_emu(&self, field: EndpointField) -> HvResult<usize> {
        self.write().read_emu(field)
    }

    pub fn read_emu64(&self, field: EndpointField) -> HvResult<u64> {
        self.write().read_emu64(field)
    }

    pub fn write_emu(&self, field: EndpointField, value: usize) -> HvResult {
        self.write().write_emu(field, value)
    }

    // Legacy method for backward compatibility
    // pub fn write_emu_legacy(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
    //     self.write().write_emu_legacy(offset, size, value)
    // }

    pub fn read_hw(&self, offset: PciConfigAddress, size: usize) -> HvResult<usize> {
        self.write().read_hw(offset, size)
    }

    pub fn write_hw(&self, offset: PciConfigAddress, size: usize, value: usize) -> HvResult {
        self.write().write_hw(offset, size, value)
    }

    /// Execute a closure with a reference to the bar at the given slot
    pub fn with_bar_ref<F, R>(&self, slot: usize, f: F) -> R
    where
        F: FnOnce(&PciMem) -> R,
    {
        let guard = self.read();
        let bar = guard.get_bar_ref(slot);
        f(bar)
    }

    /// Execute a closure with a mutable reference to the bar at the given slot
    pub fn with_bar_ref_mut<F, R>(&self, slot: usize, f: F) -> R
    where
        F: FnOnce(&mut PciMem) -> R,
    {
        let mut inner = self.write();
        let bar = inner.get_bar_ref_mut(slot);
        f(bar)
    }

    /// Execute a closure with a reference to the config_value
    pub fn with_config_value<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ConfigValue) -> R,
    {
        let guard = self.read();
        f(guard.get_config_value())
    }

    /// Execute a closure with a mutable reference to the config_value
    pub fn with_config_value_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ConfigValue) -> R,
    {
        let mut inner = self.write();
        inner.with_config_value_mut(f)
    }

    /// Execute a closure with a reference to the rom
    pub fn with_rom_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PciMem) -> R,
    {
        let guard = self.read();
        let rom = &guard.rom;
        f(rom)
    }

    /// Execute a closure with a mutable reference to the rom
    pub fn with_rom_ref_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PciMem) -> R,
    {
        let mut inner = self.write();
        let rom = &mut inner.rom;
        f(rom)
    }

    /// Execute a closure with a reference to the capabilities list
    pub fn with_cap<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PciCapabilityList) -> R,
    {
        let guard = self.read();
        f(&guard.capabilities)
    }

    pub fn with_msi_info<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&MsiInfo) -> R,
    {
        let guard = self.0.read();
        guard.msi_info.as_ref().map(|msi_info| f(msi_info))
    }

    pub fn with_msi_info_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut MsiInfo) -> R,
    {
        let mut guard = self.0.write();
        guard.msi_info.as_mut().map(|msi_info| f(msi_info))
    }

    pub fn read(&self) -> spin::RwLockReadGuard<'_, VirtualPciConfigSpaceWithZone> {
        self.0.read()
    }

    pub fn write(&self) -> spin::RwLockWriteGuard<'_, VirtualPciConfigSpaceWithZone> {
        self.0.write()
    }

    pub fn is_my_bar_addr(&self, addr: usize) -> Option<usize> {
        self.0.read().is_my_bar_addr(addr)
    }

    pub fn bar_mmio_distribute(&self, bar: usize, mmio_ac: &mut MMIOAccess) -> HvResult {
        self.0.read().bar_mmio_distribute(bar, mmio_ac)
    }

    pub fn try_inject_msix_irq(&self, msix_backend: &Arc<RwLock<dyn MsixBackend>>) {
        self.0.read().inject_msix_irq(msix_backend);
    }
}

impl Debug for ArcRwLockVirtualPciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let guard = self.0.read();
        write!(f, "zone_id: {:?}, ", guard.zone_id)?;
        guard.config_space.fmt(f)
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
        self.bararr.clone()
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
        self.rom.clone()
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
            _ => {}
        }
    }

    pub fn is_my_bar_addr(&self, addr: usize) -> Option<usize> {
        let mut index = 0;
        for i in &self.bararr {
            if let Some(res) = i.get_virtual_addr() {
                // warn!("get res:{:x}",res);
                if res as usize == addr {
                    return Some(index);
                }
            }
            index += 1;
            // if i.get_virtual_addr() == addr as u32{
            //     return true;
            // }
        }
        None
    }

    pub fn bar_mmio_distribute(&self, bar: usize, mmio_ac: &mut MMIOAccess) -> HvResult {
        self.capabilities.handle_bar_read(bar, mmio_ac)
    }

    // pub fn get_msix_backend(&self) -> Option<Arc<RwLock<dyn MsixBackend>>> {
    //     self.msix_backend.clone()
    // }

    pub fn get_msix_entry(&self, data_info: VirtioPCIDataInfo) -> Option<MsixTableEntry> {
        // let data_info=VirtioPCIDataInfo::from_u64(data_req_id);
        let msix_idx = data_info.get_msix_vector_idx();
        let msix_table = self.msix_table.clone()?;
        let entry = msix_table.read().get_entry(msix_idx as usize);
        entry
    }

    pub fn get_pending_msix(&self) -> Option<Vec<MsixTableEntry>> {
        let msix_table = self.msix_table.clone()?;
        let res = msix_table.write().get_pending_msix_vector();
        res
    }

    pub fn inject_msix_irq(&self, msix_backend: &Arc<RwLock<dyn MsixBackend>>) {
        // let data_info = VirtioPCIDataInfo::from_u64(data_req_id);
        // let msix_entry = match self.get_msix_entry(data_info){
        //     Some(x)=>x,
        //     None=>{
        //         warn!("can't find corresponding msix entry!");
        //         return;
        //     }
        // };
        // let msix_backend = match self.get_msix_backend() {
        //     Some(x) => x,
        //     None => {
        //         // warn!("There is no msix backend in this device!");
        //         return;
        //     }
        // };

        let msix_entries = match self.get_pending_msix() {
            Some(x) => x,
            None => {
                // warn!("can't find corresponding msix entry!");
                return;
            }
        };
        for i in msix_entries {
            msix_backend
                .read()
                .activate_irq(self.bdf.requester_id() as usize, &i);
        }
    }
}

impl Debug for VirtualPciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\n  bdf {:#?}\n  base {:#x}\n  type {:#?}\n  msi_info {:#x?}\n  {:#?}\n {:#?}\n {:#?}",
            self.bdf,
            self.base,
            self.config_type,
            self.msi_info,
            self.bararr,
            self.rom,
            self.capabilities
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
        msix_table: Option<Arc<RwLock<MsixTable>>>,
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
            backend: Arc::new(EndpointHeader::new_with_region(PciConfigMmio::new(
                base,
                CONFIG_LENTH,
            ))),
            bararr,
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type,
            msi_info: None,
            msix_table,
        }
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
            msi_info: None,
            msix_table: None,
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
            msi_info: None,
            msix_table: None,
        }
    }

    pub fn set_backend(&mut self, backend: Arc<dyn PciRW>) {
        self.backend = backend
    }

    pub fn unknown(
        bdf: Bdf,
        base: PciConfigAddress,
        backend: Arc<dyn PciRW>,
        id: (DeviceId, VendorId),
    ) -> Self {
        Self {
            host_bdf: Bdf::default(),
            parent_bdf: Bdf::default(),
            bdf,
            vbdf: Bdf::default(),
            config_type: HeaderType::Endpoint,
            base,
            // Default class: base=0xFF, others 0, revision 0
            config_value: ConfigValue::new(id, (0xFFu8, 0u8, 0u8, 0u8)),
            control: VirtualPciConfigControl::endpoint(),
            access: VirtualPciAccessBits::endpoint(),
            backend,
            bararr: Bar::default(),
            rom: PciMem::default(),
            capabilities: PciCapabilityList::new(),
            dev_type: VpciDevType::Physical,
            msi_info: None,
            msix_table: None,
        }
    }

    pub fn host_bridge(
        bdf: Bdf,
        base: PciConfigAddress,
        backend: Arc<dyn PciRW>,
        class_and_revision_id: (DeviceRevision, BaseClass, SubClass, Interface),
    ) -> Self {
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
            msi_info: None,
            msix_table: None,
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

    pub fn get_msi_count(&self) -> u32 {
        self.msi_info
            .as_ref()
            .map(|info| info.msi_count)
            .unwrap_or(0)
    }

    /// Build MSI/MSIX info structure based on device capabilities
    pub fn build_msi_info(&mut self) {
        let mut msi_count = 0u32;
        let mut msix_count = 0u32;
        let mut msix_bar_id = 0u8;
        let mut msix_offset = 0u64;
        let mut has_msix = false;

        // Check if the device has MSI or MSIX capability and calculate both
        for (_offset, cap) in self.capabilities.iter() {
            match cap.get_type() {
                CapabilityType::Msi => {
                    // For MSI: read offset+2, bits 2-0 contain MMC (Multiple Message Capable)
                    // Supported messages = 2^(MMC+1)
                    if let Ok(val) = cap.with_region(|region| region.read(0x02, 2)) {
                        let mmc = (val & 0x0E) >> 1; // bits 2-0
                        msi_count = (1u32 << (mmc + 1)) as u32;
                    }
                }
                CapabilityType::MsiX => {
                    // For MSIX: read offset+2, bits 10-0 contain table size
                    // Supported messages = table_size + 1
                    if let Ok(val) = cap.with_region(|region| region.read(0x02, 2)) {
                        let table_size = (val & 0x07FF) as u32; // bits 10-0
                        msix_count = table_size + 1;
                    }

                    // Extract MSIX table location (offset+4)
                    // Bits 2-0: BAR ID (0-5), Bits 31-3: table offset
                    if let Ok(table_info) = cap.with_region(|region| region.read(0x04, 4)) {
                        msix_bar_id = (table_info & 0x07) as u8;
                        msix_offset = ((table_info >> 3) as u64) << 3; // multiply by 8 since offset is in 8-byte increments
                        has_msix = true;
                    }
                }
                _ => {}
            }
        }

        // Create MsiInfo if device has MSI or MSIX capability
        let interrupt_count = core::cmp::max(msi_count, msix_count);
        if interrupt_count > 0 {
            let mut msi_info = MsiInfo::new(interrupt_count);

            if has_msix {
                // Read the BAR's physical address
                let bar_paddr = self.bararr[msix_bar_id as usize].get_value64() & !0xf;
                msi_info.set_msix_info(msix_bar_id, msix_offset, msix_count, bar_paddr);
            }

            self.msi_info = Some(msi_info);
        }
    }

    pub fn get_msi_info(&self) -> Option<&MsiInfo> {
        self.msi_info.as_ref()
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

    pub fn set_msix_table(&mut self, msix_table: Arc<RwLock<MsixTable>>) {
        self.msix_table = Some(msix_table)
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
                let (base, sub, interface, revision) =
                    self.config_value.get_class_and_revision_id();
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
    domain: u8,
    function: u8,
    is_mulitple_function: bool,
    is_finish: bool,
    accessor: Arc<dyn PciConfigAccessor>,
}

impl<B: BarAllocator> PciIterator<B> {
    fn get_pci_addr_base(&self, bdf: Bdf) -> PciConfigAddress {
        match self.accessor.get_pci_addr_base(bdf) {
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

        let bdf = Bdf::new(self.domain, bus, device, function);

        let address = self.address(parent_bus, bdf);
        let pci_addr_base = self.get_pci_addr_base(bdf);
        // info!("get node {:x} {:#?}", address, bdf);

        let region = PciConfigMmio::new(address, CONFIG_LENTH);
        let pci_header = PciConfigHeader::new_with_region(region);
        let (vender_id, device_id) = pci_header.id();

        // warn!("vender_id {:#x}", vender_id);

        // Check if device exists
        if vender_id == 0xffff || self.accessor.skip_device(bdf) {
            if function == 0 {
                // Function 0 doesn't exist, so device doesn't exist at all
                // Skip all functions and move to next device
                // info!(
                //     "get none - device not present (vendor_id=0xffff) at {:#?}",
                //     bdf
                // );
                self.function = 0;
                self.is_mulitple_function = false;
            } else {
                // Function > 0 doesn't exist, but device might have other functions
                // warn!(
                //     "get none - function not present (vendor_id=0xffff) at {:#?}",
                //     bdf
                // );
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
                    let host_bridge = Bridge::host_bridge(
                        self.segment,
                        bus_begin,
                        self.is_mulitple_function,
                        self.function,
                    );
                    self.stack.push(host_bridge);
                }

                let mut ep = EndpointHeader::new_with_region(region);
                let rom = Self::rom_init(&mut self.allocator, &mut ep);

                let bararr =
                    Self::bar_mem_init(ep.bar_limit().into(), &mut self.allocator, &mut ep);

                // info!("get node bar mem init end {:#?}", bararr);

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
                // Build MSI/MSIX info once during device discovery
                node.build_msi_info();

                Some(node)
            }
            HeaderType::PciBridge => {
                // For bridge: don't push host_bridge, it will be handled in Iterator::next()
                warn!("bridge");
                let mut bridge = PciBridgeHeader::new_with_region(region);
                let rom = Self::rom_init(&mut self.allocator, &mut bridge);

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
                // Build MSI/MSIX info once during device discovery
                node.build_msi_info();

                Some(node)
            }
            _ => {
                warn!("unknown type");
                let pci_header = Arc::new(pci_header);
                Some(VirtualPciConfigSpace::unknown(
                    bdf,
                    pci_addr_base,
                    pci_header,
                    (device_id, vender_id),
                ))
            }
        }
    }

    fn rom_init<D: PciRomRW + PciHeaderRW + PciRW>(
        allocator: &mut Option<B>,
        dev: &mut D,
    ) -> PciMem {
        let mut rom = dev.parse_rom();
        if rom.get_type() == PciMemType::Rom {
            if let Some(a) = allocator {
                let value = a.alloc_memory32(rom.get_size() as u64).unwrap();
                rom.set_value(value);
                rom.set_virtual_value(value);
                // Do not enable ROM yet, write 0 (ROM disabled)
                // VM will enable it later by writing address + enable bit
                // info!(
                //     "allocated rom address: {:#x}, write 0 (disabled) to hardware",
                //     value
                // );
                let _ = dev.write(dev.rom_offset(), 4, 0 as _);
            } else {
                let value = rom.get_value() as u64;
                rom.set_value(value);
                rom.set_virtual_value(value);
            }
        }
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
                        // let _ = dev.write_bar(i as u8, value as u32);
                        i += 1;
                        bararr[i].set_virtual_value(value);
                        // let _ = dev.write_bar(i as u8, (value >> 32) as u32);
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
            if parent.has_only_one_child {
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
        while !self.is_finish {
            if let Some(mut node) = self.get_node() {
                node.config_value_init();
                let bus_begin = self.bus_range.start as u8;
                let domain = self.domain;
                /*
                 * when first time to enumerate, placeholder is pop in get_node
                 * the message of host bridge must be got after get_node()
                 * so we push host bridge to stack here
                 */
                if self.stack.is_empty() {
                    let host_bridge = Bridge::host_bridge(
                        self.segment,
                        bus_begin,
                        self.is_mulitple_function,
                        self.function,
                    );
                    self.stack.push(host_bridge);
                }
                let parent = self.stack.last().unwrap(); // Safe because we just ensured it exists
                let host_bdf = Bdf::new(domain, bus_begin, 0, 0);
                let parent_bdf = Bdf::new(domain, parent.bus, parent.device, 0);
                let parent_bus = parent.primary_bus;
                node.set_host_bdf(host_bdf);
                node.set_parent_bdf(parent_bdf);
                self.next(match node.config_value.get_class().0 {
                    // class code 0x6 is bridge and class.1 0x0 is host bridge
                    0x6 if node.config_value.get_class().1 == 0x4 => {
                        // When no_pcie_bar_realloc is enabled, use the firmware-programmed
                        // secondary bus number instead of calculating our own. Firmware
                        // (UEFI/BIOS) may skip bus numbers for subordinate bus reservation,
                        // causing calculated bus numbers to diverge from actual hardware
                        // bus assignments — making devices behind bridges invisible.
                        #[cfg(feature = "no_pcie_bar_realloc")]
                        let next_bus = {
                            let bridge_base = node.get_base();
                            let bus_reg = unsafe {
                                let ptr = PciConfigMmio::new(bridge_base, CONFIG_LENTH)
                                    .access::<u32>(0x18);
                                ptr.read_volatile()
                            };
                            let fw_secondary = ((bus_reg >> 8) & 0xFF) as u8;
                            let fw_subordinate = ((bus_reg >> 16) & 0xFF) as u8;
                            info!(
                                "bridge {:#?}: firmware secondary_bus={:#x}, subordinate_bus={:#x}",
                                node.bdf, fw_secondary, fw_subordinate
                            );
                            // Validate firmware bus number against configured range.
                            // While the zone config maker is primarily responsible for
                            // providing a valid range, this guard prevents invalid ECAM
                            // accesses if firmware programs an out-of-range value.
                            let range_start = self.bus_range.start as u8;
                            let range_end = self.bus_range.end as u8;
                            if fw_secondary != 0
                                && fw_secondary >= range_start
                                && fw_secondary <= range_end
                            {
                                fw_secondary
                            } else {
                                if fw_secondary != 0 {
                                    warn!(
                                        "bridge {:#?}: firmware secondary_bus {:#x} out of range [{:#x}, {:#x}], falling back to calculated",
                                        node.bdf, fw_secondary, range_start, range_end
                                    );
                                }
                                parent.subordinate_bus + 1
                            }
                        };
                        #[cfg(not(feature = "no_pcie_bar_realloc"))]
                        let next_bus = parent.subordinate_bus + 1;

                        let bdf = Bdf::new(domain, next_bus, 0, 0);
                        // Use the current bridge's own bus as the immediate parent bus for
                        // CFG address computation. For multi-level bridges (especially on
                        // DWC), using parent.primary_bus (the upstream of the *parent*)
                        // would select the wrong CFG0/CFG1 path and fail to reach devices
                        // behind deeper bridges.
                        let immediate_parent_bus = parent.bus;
                        Some(self.get_bridge().next_bridge(
                            self.address(immediate_parent_bus, bdf),
                            node.has_only_one_child(),
                            self.is_mulitple_function,
                            self.function,
                            next_bus,
                        ))
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
    has_only_one_child: bool,
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
            has_only_one_child: false,
            is_mulitple_function: false,
        }
    }

    pub fn host_bridge(
        address: PciConfigAddress,
        bus_begin: u8,
        is_mulitple_function: bool,
        function: u8,
    ) -> Self {
        Self {
            bus: bus_begin,
            device: 0,
            function,
            subordinate_bus: bus_begin,
            secondary_bus: bus_begin,
            primary_bus: bus_begin,
            mmio: PciConfigMmio::new(address, CONFIG_LENTH),
            has_only_one_child: false,
            is_mulitple_function,
        }
    }

    pub fn next_bridge(
        &self,
        address: PciConfigAddress,
        has_only_one_child: bool,
        is_mulitple_function: bool,
        function: u8,
        target_bus: u8,
    ) -> Self {
        let mmio = PciConfigMmio::new(address, CONFIG_LENTH);
        Self {
            bus: target_bus,
            device: 0,
            function,
            subordinate_bus: target_bus,
            secondary_bus: target_bus,
            primary_bus: self.bus,
            mmio,
            has_only_one_child,
            is_mulitple_function,
        }
    }

    pub fn update_bridge_bus(&mut self) {
        // Skip update for placeholder bridges (they don't have real mmio)
        if self.mmio.is_placeholder() {
            return;
        }
        // When no_pcie_bar_realloc is enabled, firmware already assigned correct bus
        // numbers — don't overwrite them.
        #[cfg(feature = "no_pcie_bar_realloc")]
        return;

        #[cfg(not(feature = "no_pcie_bar_realloc"))]
        {
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
    }

    pub fn set_has_only_one_child(&mut self, value: bool) {
        self.has_only_one_child = value;
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
        domain: u8,
        bar_alloc: Option<B>,
    ) -> PciIterator<B> {
        let mmio_base = self.mmio_base;
        let range = range.unwrap_or_else(|| 0..0x100);
        PciIterator {
            allocator: bar_alloc,
            stack: vec![Bridge::placeholder()],
            segment: mmio_base,
            bus_range: range,
            domain,
            function: 0,
            is_mulitple_function: false,
            is_finish: false,
            accessor: self.accessor.clone(), // accessor to iterator
        }
    }

    pub fn enumerate<B: BarAllocator>(
        &mut self,
        range: Option<Range<usize>>,
        domain: u8,
        bar_alloc: Option<B>,
    ) -> PciIterator<B> {
        self.__enumerate(range, domain, bar_alloc)
    }
}

#[derive(Debug)]
/// MSI information for a specific domain in a VM
/// Tracks the MSI interrupts needed for this domain and the hardware base interrupt bit
pub struct DomainMsiInfo {
    /// Total number of MSI interrupts needed for all devices in this domain
    pub msi_count: u32,
    /// Hardware MSI base bit index (allocated from domain allocator)
    pub hwirq_bit: u32,
    /// Virtual doorbell address set by the VM (PCIE_MSI_ADDR_LO + PCIE_MSI_ADDR_HI)
    pub vm_doorbell_addr: u64,
}

impl DomainMsiInfo {
    pub fn new(msi_count: u32, hwirq_bit: u32) -> Self {
        Self {
            msi_count,
            hwirq_bit,
            vm_doorbell_addr: 0,
        }
    }

    /// Set the virtual doorbell address (from VM)
    pub fn set_vm_doorbell(&mut self, addr: u64) {
        self.vm_doorbell_addr = addr;
    }

    /// Get the virtual doorbell address
    pub fn get_vm_doorbell(&self) -> u64 {
        self.vm_doorbell_addr
    }

    /// Get MSI mask based on msi_count
    /// Returns a mask with msi_count bits set (0-based, e.g. msi_count=4 -> mask=0xf)
    pub fn get_msi_mask(&self) -> u32 {
        if self.msi_count >= 32 {
            0xffffffff
        } else {
            (1u32 << self.msi_count) - 1
        }
    }
}

#[derive(Debug)]
pub struct VirtualRootComplex {
    devs: BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace>,
    base_to_bdf: BTreeMap<PciConfigAddress, Bdf>,
    // MSI interrupt information per domain (domain_id -> DomainMsiInfo)
    domain_msi_info: BTreeMap<u8, DomainMsiInfo>,
    accessor: Option<Arc<dyn PciConfigAccessor>>,
    msix_backend: Option<Arc<RwLock<dyn MsixBackend>>>,
}

impl VirtualRootComplex {
    pub fn new() -> Self {
        Self {
            devs: BTreeMap::new(),
            base_to_bdf: BTreeMap::new(),
            domain_msi_info: BTreeMap::new(),
            accessor: None,
            msix_backend: None,
        }
    }

    pub fn set_accessor(&mut self, accessor: Arc<dyn PciConfigAccessor>) {
        self.accessor = Some(accessor);
    }

    pub fn insert(
        &mut self,
        bdf: Bdf,
        dev: VirtualPciConfigSpace,
    ) -> Option<ArcRwLockVirtualPciConfigSpace> {
        let parent_bus = dev.parent_bdf.bus();
        let offset = 0;
        let base = if let Some(accessor) = &self.accessor {
            match accessor.get_physical_address(bdf, offset, parent_bus) {
                Ok(addr) => addr,
                Err(_) => {
                    warn!("can not get physical address for device {:#?}(vbdf), reset device base same to hardware", bdf);
                    dev.get_base()
                }
            }
        } else {
            warn!("can not found accessor for vpci bus, reset device base same to hardware");
            dev.get_base()
        };
        info!("pci insert base {:#x} to bdf {:#?}", base, bdf);
        self.base_to_bdf.insert(base, bdf);
        self.devs
            .insert(bdf, ArcRwLockVirtualPciConfigSpace::new(dev))
    }

    pub fn devs(&mut self) -> &mut BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace> {
        &mut self.devs
    }

    pub fn devs_ref(&self) -> &BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace> {
        &self.devs
    }

    pub fn read_devs(&self) -> &BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace> {
        &self.devs
    }

    pub fn get(&self, bdf: &Bdf) -> Option<&ArcRwLockVirtualPciConfigSpace> {
        self.devs.get(bdf)
    }

    pub fn get_mut(&mut self, bdf: &Bdf) -> Option<&mut ArcRwLockVirtualPciConfigSpace> {
        self.devs.get_mut(bdf)
    }

    /* because the base of device may discontinuous，get device by base is simpler */
    pub fn get_device_by_base(
        &self,
        base: PciConfigAddress,
    ) -> Option<ArcRwLockVirtualPciConfigSpace> {
        let bdf = self.base_to_bdf.get(&base).copied()?;
        self.devs.get(&bdf).cloned()
    }

    /// Add MSI count for a specific domain with allocated hardware interrupt bit
    pub fn add_msi_count_for_domain(&mut self, domain: u8, msi_count: u32, hwirq_bit: u32) {
        let vm_doorbell = self
            .domain_msi_info
            .get(&domain)
            .map(|info| info.get_vm_doorbell())
            .unwrap_or(0);

        let mut info = DomainMsiInfo::new(msi_count, hwirq_bit);
        info.set_vm_doorbell(vm_doorbell);
        self.domain_msi_info.insert(domain, info);
    }

    /// Get MSI info for a specific domain
    pub fn get_domain_msi_info(&self, domain: u8) -> Option<&DomainMsiInfo> {
        self.domain_msi_info.get(&domain)
    }

    /// Get reference to domain MSI info map
    pub fn domain_msi_info(&self) -> &BTreeMap<u8, DomainMsiInfo> {
        &self.domain_msi_info
    }

    /// Get mutable reference to domain MSI info map
    pub fn domain_msi_info_mut(&mut self) -> &mut BTreeMap<u8, DomainMsiInfo> {
        &mut self.domain_msi_info
    }

    pub fn get_msix_backend(&self) -> Option<Arc<RwLock<dyn MsixBackend>>> {
        self.msix_backend.clone()
    }

    pub fn set_msix_backend(&mut self, backend: Option<Arc<RwLock<dyn MsixBackend>>>) {
        self.msix_backend = backend
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
            CapabilityType::CompactPCICentralResourceControl => {
                write!(f, "CompactPCICentralResourceControl(0x0B)")
            }
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
    // bar_usage:Option<Arc<RwLock<dyn AreaInBar>>>
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
                let region = Arc::new(RwLock::new(StandardPciCapabilityRegion::new(
                    offset, 32, backend,
                )));
                return Some(PciCapability {
                    cap_type: CapabilityType::Msi,
                    region,
                    // bar_usage:None,
                });
            }
            _ => {
                let region = Arc::new(RwLock::new(StandardPciCapabilityRegion::new(
                    offset, 32, backend,
                )));
                Some(PciCapability {
                    cap_type: CapabilityType::from_id(id),
                    region,
                    // bar_usage:None,
                })
            }
        }
    }

    pub fn new_virt(region: Arc<RwLock<dyn PciCapabilityRegion>>) -> Self {
        Self {
            cap_type: CapabilityType::Vendor,
            region,
            // bar_usage:None,
        }
    }

    pub fn new_cap(cap_type: CapabilityType, region: Arc<RwLock<dyn PciCapabilityRegion>>) -> Self {
        Self {
            cap_type,
            region,
            // bar_usage:None
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

    // fn set_bar_area(&mut self,bar_area:Arc<RwLock<dyn AreaInBar>>){
    //     self.bar_usage = Some(bar_area)
    // }
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

    fn bar_area(&self) -> Option<Arc<RwLock<dyn AreaInBar>>> {
        None
    }

    fn bar_usage(&self) -> Option<usize> {
        None
    }

    fn bar_addr_range(&self) -> Option<Range<usize>> {
        None
    }

    fn set_bar_area(&mut self, _bar_area: Arc<RwLock<dyn AreaInBar>>) {}
}

pub struct StandardPciCapabilityRegion {
    offset: PciConfigAddress,
    size: usize,
    backend: Arc<dyn PciRW>,
}

impl StandardPciCapabilityRegion {
    pub fn new(offset: PciConfigAddress, size: usize, backend: Arc<dyn PciRW>) -> Self {
        Self {
            offset,
            size,
            backend,
        }
    }
}

impl PciCapabilityRegion for StandardPciCapabilityRegion {
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<u32> {
        self.backend
            .read(self.offset + offset, size)
            .map(|v| v as u32)
    }

    fn write(&mut self, offset: PciConfigAddress, size: usize, value: u32) -> HvResult {
        self.backend
            .write(self.offset + offset, size, value as usize)
    }

    fn get_offset(&self) -> PciConfigAddress {
        self.offset
    }

    fn get_size(&self) -> usize {
        self.size
    }
}

#[derive(Clone)]
pub struct PciCapabilityList {
    capability_ptr: u8,
    cap_in_config: BTreeMap<PciConfigAddress, PciCapability>,
    // cap_in_bar: Arc<RwLock<BarAreaManager>>,
}

impl PciCapabilityList {
    pub fn new() -> Self {
        Self {
            capability_ptr: 0,
            cap_in_config: BTreeMap::new(),
            // cap_in_bar: Arc::new(RwLock::new(BarAreaManager::new())),
        }
    }

    pub fn get_capability_pointer(&self) -> u8 {
        self.capability_ptr
    }

    pub fn set_capability_pointer(&mut self, ptr: u8) {
        self.capability_ptr = ptr;
    }

    pub fn try_read_cap(&self, offset: PciConfigAddress, size: usize) -> Option<usize> {
        let cap = self.which_cap(offset)?;
        let relative_offset = offset - cap.get_offset();
        match cap.region.clone().read().read(relative_offset, size) {
            Ok(res) => Some(res as usize),
            Err(_) => None,
        }
    }

    pub fn try_write_cap(
        &self,
        offset: PciConfigAddress,
        size: usize,
        value: usize,
    ) -> Option<usize> {
        let cap = self.which_cap(offset)?;
        let relative_offset = offset - cap.get_offset();
        match cap
            .region
            .clone()
            .write()
            .write(relative_offset, size, value as u32)
        {
            Ok(_) => Some(0),
            Err(_) => None,
        }
    }

    pub fn which_cap(&self, offset: PciConfigAddress) -> Option<PciCapability> {
        let kv_pair = self.cap_in_config.range(..=offset).next_back()?;
        Some(kv_pair.1.clone())
    }

    pub fn insert_cap(
        &mut self,
        // addr: PciConfigAddress,
        cap: PciCapability,
    ) -> Option<PciCapability> {
        // cap.get_offset()
        self.cap_in_config.insert(cap.get_offset(), cap)
    }

    // pub fn register_bar_area(
    //     &mut self,
    //     bar: usize,
    //     bar_relative_addr: GuestPhysAddr,
    //     size_in_bar: usize,
    //     data: Arc<RwLock<dyn AreaInBar>>,
    // ) {
    //     self.cap_in_bar
    //         .write()
    //         .insert(bar, bar_relative_addr, size_in_bar, data);
    // }

    pub fn cap_in_config_ref(&self) -> &BTreeMap<PciConfigAddress, PciCapability> {
        &self.cap_in_config
    }

    fn find_cap_by_bar(&self, bar: usize, addr: usize) -> Option<PciCapability> {
        for i in self.cap_in_config.values() {
            if bar != i.region.read().bar_usage()? {
                continue;
            }
            if i.region.read().bar_addr_range()?.contains(&addr) {
                return Some(i.clone());
            }
        }
        None
    }

    pub fn handle_bar_read(&self, bar: usize, mmio_ac: &mut MMIOAccess) -> HvResult {
        let addr = mmio_ac.address;
        let cap = self.find_cap_by_bar(bar, addr);
        match cap {
            Some(x) => {
                let bar = x
                    .region
                    .read()
                    .bar_area()
                    .expect("This bar area should not be none");
                if mmio_ac.is_write {
                    return bar.write().write(mmio_ac);
                } else {
                    return bar.write().read(mmio_ac);
                }
            }
            None => {
                warn!("can't find this addr:0x{:x} in bar:{}", addr, bar);
                return Ok(());
            }
        }
        // self.cap_in_bar.read().handle_bar_access(bar, mmio_ac)
    }

    // pub fn get_msix_table(&self)->Option<PciCapability>{
    //     for (_,i) in self.cap_in_config.iter(){
    //         if i.cap_type == CapabilityType::MsiX{
    //             return Some(i.clone());
    //         }
    //     }
    //     return None;
    // }
}

impl Debug for PciCapabilityList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PciCapabilityList {{\n")?;
        for (offset, capability) in &self.cap_in_config {
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
            capabilities
                .cap_in_config
                .insert(capability.get_offset(), capability);
        }
        info!("capability {:#?}", capabilities);
        self.capabilities = capabilities;
    }

    // detect whether this bridge secondary bus can have only one child device.
    pub fn has_only_one_child(&self) -> bool {
        match self.config_type {
            HeaderType::PciBridge => {
                // Parse PCIe Device/Port Type from PCI Express Capability Register
                // (capability offset + 0x02, bits 7:4).
                for capability in self._capability_enumerate(self.backend.clone()) {
                    if capability.get_type() != CapabilityType::PciExpress {
                        continue;
                    }

                    let offset = capability.get_offset();
                    if let Ok(cap_reg) = self.backend.read(offset + 0x2, 2) {
                        let port_type = (cap_reg as u16).get_bits(4..8) as u16;
                        return match port_type {
                            // Root Port / Downstream Port: secondary bus has a single downstream link.
                            PCI_EXP_TYPE_ROOT_PORT | PCI_EXP_TYPE_DOWNSTREAM => true,
                            // Upstream Port / PCIe-to-PCI bridge can have multiple children behind it.
                            PCI_EXP_TYPE_UPSTREAM | PCI_EXP_TYPE_PCIE_BRIDGE => false,
                            _ => false,
                        };
                    }

                    // Capability exists but cannot be read safely.
                    return false;
                }

                // Non-PCIe bridge (or no PCIe capability): keep full secondary-bus scan.
                false
            }
            _ => false,
        }
    }
}

/// Bar area is just a MMIO memory area
/// There are many virtio capability structures such as commoncfg being put in bar
/// Any structure put in bar has to implement this trait and be registered by function 'register_bar_area'
pub trait AreaInBar: Send + Sync + Debug {
    fn read(&mut self, mmio_ac: &mut MMIOAccess) -> HvResult;

    fn write(&mut self, mmio_ac: &MMIOAccess) -> HvResult;
}
