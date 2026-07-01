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
use alloc::sync::Arc;
use alloc::vec::Vec;
// use psci::error::INVALID_ADDRESS;
use crate::consts::{INVALID_ADDRESS, MAX_CPU_NUM};
use crate::pci::pci_struct::VirtualRootComplex;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(dwc_pcie)]
use crate::pci::{config_accessors::dwc_atu::AtuConfig, PciConfigAddress};
#[cfg(dwc_pcie)]
use alloc::collections::btree_map::BTreeMap;

use crate::arch::mm::new_s2_memory_set;
use crate::arch::s2pt::Stage2PageTable;
use crate::config::{HvZoneConfig, CONFIG_NAME_MAXLEN};

use crate::cpu_data::{get_cpu_data, this_zone, CpuSet};
use crate::error::HvResult;
use crate::memory::addr::GuestPhysAddr;
use crate::memory::{MMIOConfig, MMIOHandler, MMIORegion, MemorySet};
use core::panic;
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(pci_init_delay, dwc_pcie))]
use crate::config::{HvPciConfig, HvPciDevConfig, CONFIG_MAX_PCI_DEV, CONFIG_PCI_BUS_MAXNUM};
#[cfg(all(pci_init_delay, dwc_pcie))]
use crate::pci::pci_config::GLOBAL_PCIE_LIST;
#[cfg(all(pci_init_delay, dwc_pcie))]
use crate::pci::pci_struct::Bdf;

#[cfg(dwc_pcie)]
#[derive(Debug)]
pub struct VirtualAtuConfigs {
    ecam_to_atu: BTreeMap<usize, AtuConfig>,
    io_base_to_ecam: BTreeMap<PciConfigAddress, usize>,
    cfg_base_to_ecam: BTreeMap<PciConfigAddress, usize>,
}

#[cfg(dwc_pcie)]
impl VirtualAtuConfigs {
    pub fn new() -> Self {
        Self {
            ecam_to_atu: BTreeMap::new(),
            io_base_to_ecam: BTreeMap::new(),
            cfg_base_to_ecam: BTreeMap::new(),
        }
    }

    pub fn get_atu_by_ecam(&self, ecam_base: usize) -> Option<&AtuConfig> {
        self.ecam_to_atu.get(&ecam_base)
    }

    pub fn get_atu_by_ecam_mut(&mut self, ecam_base: usize) -> Option<&mut AtuConfig> {
        self.ecam_to_atu.get_mut(&ecam_base)
    }

    pub fn insert_atu(&mut self, ecam_base: usize, atu: AtuConfig) -> Option<AtuConfig> {
        self.ecam_to_atu.insert(ecam_base, atu)
    }

    pub fn get_atu_by_io_base(&self, io_base: PciConfigAddress) -> Option<&AtuConfig> {
        let ecam = self.io_base_to_ecam.get(&io_base);
        if let Some(ecam) = ecam {
            self.get_atu_by_ecam(*ecam)
        } else {
            None
        }
    }

    pub fn get_ecam_by_io_base(&self, io_base: PciConfigAddress) -> Option<usize> {
        self.io_base_to_ecam.get(&io_base).copied()
    }

    pub fn insert_io_base_mapping(&mut self, io_base: PciConfigAddress, ecam_base: usize) {
        self.io_base_to_ecam.insert(io_base, ecam_base);
    }

    pub fn get_atu_by_cfg_base(&self, cfg_base: PciConfigAddress) -> Option<&AtuConfig> {
        let ecam = self.cfg_base_to_ecam.get(&cfg_base);
        if let Some(ecam) = ecam {
            self.get_atu_by_ecam(*ecam)
        } else {
            None
        }
    }

    pub fn get_ecam_by_cfg_base(&self, cfg_base: PciConfigAddress) -> Option<usize> {
        self.cfg_base_to_ecam.get(&cfg_base).copied()
    }

    pub fn insert_cfg_base_mapping(&mut self, cfg_base: PciConfigAddress, ecam_base: usize) {
        self.cfg_base_to_ecam.insert(cfg_base, ecam_base);
    }
}

pub struct Zone {
    name: [u8; CONFIG_NAME_MAXLEN],
    id: usize,
    is_err: AtomicBool,
    inner: RwLock<ZoneInner>,
}

pub struct ZoneInner {
    mmio: Vec<MMIOConfig>,
    cpu_num: usize,
    cpu_set: CpuSet,
    irq_bitmap: [u32; 1024 / 32],
    gpm: MemorySet<Stage2PageTable>,
    iommu_pt: Option<MemorySet<Stage2PageTable>>,
    vpci_bus: VirtualRootComplex,
    #[cfg(dwc_pcie)]
    atu_configs: VirtualAtuConfigs,
}

impl Zone {
    #[allow(dead_code)]
    pub fn new(zoneid: usize, name: &[u8]) -> Self {
        Self {
            name: name.try_into().unwrap(),
            id: zoneid,
            is_err: AtomicBool::new(false),
            inner: RwLock::new(ZoneInner::new()),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, ZoneInner> {
        self.inner.read()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, ZoneInner> {
        self.inner.write()
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> [u8; CONFIG_NAME_MAXLEN] {
        self.name
    }

    pub fn is_err(&self) -> bool {
        self.is_err.load(Ordering::Acquire)
    }

    pub fn set_err(&self) {
        self.is_err.store(true, Ordering::Release);
    }

    pub fn cpu_set(&self) -> CpuSet {
        self.read().cpu_set()
    }
}

impl ZoneInner {
    pub fn new() -> Self {
        Self {
            gpm: new_s2_memory_set(),
            mmio: Vec::new(),
            cpu_num: 0,
            cpu_set: CpuSet::new(MAX_CPU_NUM as usize, 0),
            irq_bitmap: [0; 1024 / 32],
            iommu_pt: if cfg!(iommu) {
                Some(new_s2_memory_set())
            } else {
                None
            },
            vpci_bus: VirtualRootComplex::new(),
            #[cfg(dwc_pcie)]
            atu_configs: VirtualAtuConfigs::new(),
        }
    }

    // pub fn suspend(&self) {
    //     trace!("suspending cpu_set = {:#x?}", self.cpu_set);
    //     self.cpu_set.iter_except(this_cpu_id()).for_each(|cpu_id| {
    //         trace!("try to suspend cpu_id = {:#x?}", cpu_id);
    //         suspend_cpu(cpu_id);
    //     });
    //     info!("send sgi done!");
    // }

    // pub fn resume(&self) {
    //     trace!("resuming cpu_set = {:#x?}", self.cpu_set);
    //     self.cpu_set.iter_except(this_cpu_id()).for_each(|cpu_id| {
    //         trace!("try to resume cpu_id = {:#x?}", cpu_id);
    //         resume_cpu(cpu_id);
    //     });
    // }

    // pub fn owns_cpu(&self, id: usize) -> bool {
    //     self.cpu_set.contains_cpu(id)
    // }

    /// Register a mmio region and its handler.
    pub fn mmio_region_register(
        &mut self,
        start: GuestPhysAddr,
        size: usize,
        handler: MMIOHandler,
        arg: usize,
    ) {
        if let Some(mmio) = self.mmio.iter_mut().find(|mmio| mmio.region.start == start) {
            warn!("duplicated mmio region {:#x?}", mmio);
            if mmio.region.size != size {
                error!("duplicated mmio region size not match, PLEASE CHECK!!!");
            }
            mmio.handler = handler;
            mmio.arg = arg;
        } else {
            self.mmio.push(MMIOConfig {
                region: MMIORegion { start, size },
                handler,
                arg,
            })
        }
    }
    #[allow(dead_code)]
    /// Remove the mmio region beginning at `start`.
    pub fn mmio_region_remove(&mut self, start: GuestPhysAddr) {
        if let Some((idx, _)) = self
            .mmio
            .iter()
            .enumerate()
            .find(|(_, mmio)| mmio.region.start == start)
        {
            self.mmio.remove(idx);
        }
    }
    /// Find the mmio region contains (addr..addr+size).
    pub fn find_mmio_region(
        &self,
        addr: GuestPhysAddr,
        size: usize,
    ) -> Option<(MMIORegion, MMIOHandler, usize)> {
        self.mmio
            .iter()
            .find(|cfg| cfg.region.contains_region(addr, size))
            .map(|cfg| (cfg.region, cfg.handler, cfg.arg))
    }
    /// If irq_id belongs to this zone
    pub fn irq_in_zone(&self, irq_id: u32) -> bool {
        let idx = (irq_id / 32) as usize;
        let bit_pos = (irq_id % 32) as usize;
        (self.irq_bitmap[idx] & (1 << bit_pos)) != 0
    }

    pub fn cpu_set(&self) -> CpuSet {
        self.cpu_set
    }

    pub fn cpu_num(&self) -> usize {
        self.cpu_num
    }

    pub fn set_cpu_num(&mut self, cpu_num: usize) {
        self.cpu_num = cpu_num;
    }

    pub fn cpu_set_mut(&mut self) -> &mut CpuSet {
        &mut self.cpu_set
    }

    pub fn irq_bitmap(&self) -> &[u32; 1024 / 32] {
        &self.irq_bitmap
    }

    pub fn irq_bitmap_mut(&mut self) -> &mut [u32; 1024 / 32] {
        &mut self.irq_bitmap
    }

    pub fn gpm(&self) -> &MemorySet<Stage2PageTable> {
        &self.gpm
    }

    pub fn gpm_mut(&mut self) -> &mut MemorySet<Stage2PageTable> {
        &mut self.gpm
    }

    pub fn iommu_pt(&self) -> Option<&MemorySet<Stage2PageTable>> {
        self.iommu_pt.as_ref()
    }

    pub fn iommu_pt_mut(&mut self) -> Option<&mut MemorySet<Stage2PageTable>> {
        self.iommu_pt.as_mut()
    }

    pub fn vpci_bus(&self) -> &VirtualRootComplex {
        &self.vpci_bus
    }

    pub fn vpci_bus_mut(&mut self) -> &mut VirtualRootComplex {
        &mut self.vpci_bus
    }

    #[cfg(dwc_pcie)]
    pub fn atu_configs(&self) -> &VirtualAtuConfigs {
        &self.atu_configs
    }

    #[cfg(dwc_pcie)]
    pub fn atu_configs_mut(&mut self) -> &mut VirtualAtuConfigs {
        &mut self.atu_configs
    }

    #[cfg(all(pci_init_delay, dwc_pcie))]
    pub fn guest_pci_init_delay(
        &mut self,
        _zone_id: usize,
        alloc_pci_devs: &[HvPciDevConfig; CONFIG_MAX_PCI_DEV],
        num_pci_devs: u64,
        pci_config: &[HvPciConfig],
        _num_pci_config: usize,
    ) -> HvResult {
        let guard = GLOBAL_PCIE_LIST.lock();
        for target_pci_config in pci_config {
            // Skip empty config
            if target_pci_config.ecam_base == 0 {
                continue;
            }

            #[allow(unused_variables)]
            let ecam_base = target_pci_config.ecam_base;
            let target_domain = target_pci_config.domain;
            let bus_range_begin = target_pci_config.bus_range_begin as u8;

            // Create accessor for VirtualRootComplex, similar to RootComplex
            #[cfg(dwc_pcie)]
            {
                use crate::pci::config_accessors::dwc::DwcConfigAccessor;
                use crate::platform;
                use alloc::sync::Arc;

                let atu_config = platform::ROOT_DWC_ATU_CONFIG
                    .iter()
                    .find(|atu_cfg| atu_cfg.ecam_base == ecam_base);

                match atu_config {
                    Some(cfg) => {
                        let root_bus = bus_range_begin;
                        let accessor = Arc::new(DwcConfigAccessor::new(cfg, root_bus));
                        self.vpci_bus_mut().set_accessor(accessor);
                    }
                    None => {
                        warn!("No ATU config found for ecam_base 0x{:x}", ecam_base);
                        continue;
                    }
                }
            }

            #[cfg(loongarch64_pcie)]
            {
                use crate::pci::config_accessors::loongarch64::LoongArchConfigAccessor;
                use alloc::sync::Arc;

                let root_bus = bus_range_begin;
                let accessor = Arc::new(LoongArchConfigAccessor::new(
                    ecam_base,
                    target_pci_config.ecam_size,
                    root_bus,
                ));
                self.vpci_bus_mut().set_accessor(accessor);
            }

            #[cfg(ecam_pcie)]
            {
                use crate::pci::config_accessors::ecam::EcamConfigAccessor;
                use alloc::sync::Arc;

                let accessor = Arc::new(EcamConfigAccessor::new(ecam_base));
                self.vpci_bus_mut().set_accessor(accessor);
            }

            let mut filtered_devices: alloc::vec::Vec<HvPciDevConfig> = alloc::vec::Vec::new();
            for i in 0..num_pci_devs {
                let dev_config = alloc_pci_devs[i as usize];
                if dev_config.domain == target_domain {
                    filtered_devices.push(dev_config);
                }
            }

            // Skip if no devices for this domain
            if filtered_devices.is_empty() {
                continue;
            }

            filtered_devices.sort_by(|a, b| {
                a.bus
                    .cmp(&b.bus)
                    .then_with(|| a.device.cmp(&b.device))
                    .then_with(|| a.function.cmp(&b.function))
            });

            let mut domain_msi_count: u32 = 0;

            for dev_config in &filtered_devices {
                let bdf = Bdf::new_from_config(*dev_config);
                let vbdf = Bdf::new(
                    bdf.domain(),
                    dev_config.v_bus,
                    dev_config.v_device,
                    dev_config.v_function,
                );

                info!("set bdf {:#?} to vbdf {:#?}", bdf, vbdf);

                #[cfg(any(
                    all(iommu, target_arch = "aarch64"),
                    all(iommu, target_arch = "riscv64"),
                    target_arch = "x86_64"
                ))]
                {
                    let iommu_pt_addr = if self.iommu_pt().is_some() {
                        self.iommu_pt().unwrap().root_paddr()
                    } else {
                        0
                    };
                    let device_id = (dev_config.bus as usize) << 8
                        | (dev_config.device as usize) << 3
                        | dev_config.function as usize;
                    #[cfg(share_s2pt)]
                    crate::device::iommu::iommu_add_device_with_root_pt_addr(
                        _zone_id,
                        device_id as _,
                        self.gpm().root_paddr(),
                    );
                    #[cfg(not(share_s2pt))]
                    crate::device::iommu::iommu_add_device_with_root_pt_addr(
                        _zone_id,
                        device_id as _,
                        iommu_pt_addr,
                    );
                }

                // Insert device into vpci_bus with calculated vbdf
                if let Some(dev) = guard.get(&bdf) {
                    if bdf.is_host_bridge(dev.read().get_host_bdf().bus())
                        || dev.with_config_value(|config_value| -> bool {
                            config_value.get_class().0 == 0x6
                        })
                    {
                        let mut vdev = dev.read().config_space.clone();
                        vdev.set_vbdf(vbdf);
                        let msi_count = vdev.get_msi_count();
                        domain_msi_count += msi_count;
                        self.vpci_bus_mut().insert(vbdf, vdev);
                    } else {
                        // Allow allocation if zone_id is None (unassigned), or if zone_id is
                        // Some(0) and the device is a SRIOV VF (initially assigned to root zone
                        // during enumeration, can be reassigned to a guest zone).
                        let is_sriov_vf_from_root = dev.get_zone_id() == Some(0)
                            && dev.read().get_sriov_vf_info().is_some();
                        let is_pf = dev.read().get_sriov_info().is_some();
                        if dev.get_zone_id().is_none() || is_sriov_vf_from_root {
                            if is_pf && _zone_id != 0 {
                                warn!(
                                    "The SR-IOV PF {:#x?} can only be assigned to the root VM",
                                    bdf
                                );
                            } else {
                                dev.set_zone_id(Some(_zone_id as u32));
                                let mut vdev_inner = dev.read().config_space.clone();
                                vdev_inner.set_vbdf(vbdf);
                                let msi_count = vdev_inner.get_msi_count();
                                domain_msi_count += msi_count;
                                self.vpci_bus_mut().insert(vbdf, vdev_inner);
                            }
                        } else {
                            warn!(
                                "Device {:#?} is already allocated to zone {:?}",
                                bdf,
                                dev.get_zone_id()
                            );
                        }
                    }
                } else {
                    warn!("can not find dev {:#?} in GLOBAL_PCIE_LIST (not detected during enumeration)", bdf);
                    #[cfg(ecam_pcie)]
                    {
                        use crate::pci::pci_struct::VirtualPciConfigSpace;
                        use crate::pci::vpci_dev::{get_handler, VpciDevType};

                        let dev_type = dev_config.dev_type;
                        match dev_type {
                            VpciDevType::Physical => {
                                warn!("can not find dev {:#?}", bdf);
                            }
                            _ => {
                                if let Some(_handler) = get_handler(dev_type) {
                                    let base = ecam_base
                                        + ((bdf.bus() as u64) << 20)
                                        + ((bdf.device() as u64) << 15)
                                        + ((bdf.function() as u64) << 12);
                                    let dev = VirtualPciConfigSpace::virt_dev(bdf, base, dev_type);
                                    self.vpci_bus_mut().insert(vbdf, dev);
                                } else {
                                    warn!("can not find dev {:#?}, unknown device type", bdf);
                                }
                            }
                        }
                    }
                }
            }

            // After processing all devices for this domain, allocate hardware MSI bits
            if domain_msi_count > 0 {
                #[cfg(all(dwc_msi, dwc_pcie))]
                {
                    // Get the DW MSI domain allocator and allocate hwbit
                    if let Some(mut domain_lock) =
                        crate::pci::dwc_msi::get_dwc_msi_domain_mut(target_domain)
                    {
                        if let Some(domain_msi) = domain_lock.get_mut(&target_domain) {
                            let zone_cpu_set = self.cpu_set();
                            let target_cpu = zone_cpu_set.first_cpu().unwrap_or(0);
                            match domain_msi.allocate_for_cpu(target_cpu, domain_msi_count) {
                                Ok(hwirq_bit) => {
                                    // Register the MSI info for this domain
                                    self.vpci_bus_mut().add_msi_count_for_domain(
                                        target_domain,
                                        domain_msi_count,
                                        hwirq_bit,
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to allocate MSI for domain {}: {:?}",
                                        target_domain, e
                                    );
                                }
                            }
                        }
                    }
                }

                #[cfg(not(dwc_msi))]
                {
                    // Without dwc_msi feature, just register without hardware bit allocation
                    self.vpci_bus_mut().add_msi_count_for_domain(
                        target_domain,
                        domain_msi_count,
                        0, // hwirq_bit is 0 when not using dwc_msi
                    );
                }
            }
        }
        info!("vpci bus init done\n {:#x?}", self.vpci_bus());
        Ok(())
    }

    #[cfg(all(pci_init_delay, dwc_pcie))]
    pub fn virtual_pci_dbi_pref_init(
        &mut self,
        pci_rootcomplex_config: &[HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
        _num_pci_config: usize,
    ) {
        use crate::pci::pci_handler::mmio_vpci_handler_dbi;

        for rootcomplex_config in pci_rootcomplex_config {
            if rootcomplex_config.ecam_base == 0 {
                continue;
            }

            let encoded_arg =
                rootcomplex_config.ecam_base as usize + (rootcomplex_config.domain as usize);
            self.mmio_region_register(
                rootcomplex_config.ecam_base as usize,
                rootcomplex_config.ecam_size as usize,
                mmio_vpci_handler_dbi,
                encoded_arg,
            );
        }
    }

    #[cfg(all(pci_init_delay, dwc_pcie))]
    pub fn virtual_pci_mmio_init_delay(
        &mut self,
        pci_rootcomplex_config: &[HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
        _num_pci_config: usize,
    ) {
        #[cfg(loongarch64_pcie)]
        let mut emergency_map_regions: alloc::vec::Vec<(usize, usize)> = alloc::vec::Vec::new();

        for rootcomplex_config in pci_rootcomplex_config {
            if rootcomplex_config.ecam_base == 0 {
                continue;
            }
            #[cfg(ecam_pcie)]
            {
                use crate::pci::pci_handler::mmio_vpci_handler;

                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_handler,
                    rootcomplex_config.ecam_base as usize,
                );
            }
            #[cfg(dwc_pcie)]
            {
                use crate::memory::mmio_generic_handler;
                use crate::pci::config_accessors::dwc_atu::AtuConfig;
                use crate::pci::config_accessors::{dwc::DwcConfigRegionBackend, PciRegionMmio};
                use crate::pci::pci_handler::{
                    mmio_dwc_cfg_handler, mmio_dwc_io_handler, mmio_vpci_handler_dbi,
                };
                use crate::platform;

                let encoded_arg =
                    rootcomplex_config.ecam_base as usize + (rootcomplex_config.domain as usize);
                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_handler_dbi,
                    encoded_arg,
                );

                let extend_config = platform::ROOT_DWC_ATU_CONFIG
                    .iter()
                    .find(|extend_cfg| extend_cfg.ecam_base == rootcomplex_config.ecam_base);

                if let Some(extend_config) = extend_config {
                    if extend_config.apb_base != 0 && extend_config.apb_size != 0 {
                        self.mmio_region_register(
                            extend_config.apb_base as usize,
                            extend_config.apb_size as usize,
                            mmio_generic_handler,
                            extend_config.apb_base as usize,
                        );
                    }

                    let cfg_size_half = extend_config.cfg_size / 2;
                    let cfg0_base = extend_config.cfg_base;
                    if cfg0_base != 0 && cfg_size_half != 0 {
                        self.mmio_region_register(
                            cfg0_base as usize,
                            cfg_size_half as usize,
                            mmio_dwc_cfg_handler,
                            cfg0_base as usize,
                        );
                    }

                    let cfg1_base = extend_config.cfg_base + cfg_size_half;
                    if cfg1_base != 0 && cfg_size_half != 0 {
                        self.mmio_region_register(
                            cfg1_base as usize,
                            cfg_size_half as usize,
                            mmio_dwc_cfg_handler,
                            cfg1_base as usize,
                        );
                    }

                    if extend_config.io_cfg_atu_shared != 0 {
                        self.mmio_region_register(
                            rootcomplex_config.io_base as usize,
                            rootcomplex_config.io_size as usize,
                            mmio_dwc_io_handler,
                            rootcomplex_config.io_base as usize,
                        );
                    }

                    let mut atu = AtuConfig::default();

                    let dbi_base = extend_config.dbi_base as crate::pci::PciConfigAddress;
                    let dbi_size = extend_config.dbi_size;
                    let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
                    let dbi_backend = DwcConfigRegionBackend::new(dbi_region);
                    if let Err(e) = atu.init_limit_hw_value(&dbi_backend) {
                        warn!("Failed to initialize ATU0 limit defaults: {:?}", e);
                    }

                    self.atu_configs_mut()
                        .insert_atu(rootcomplex_config.ecam_base as usize, atu);
                    self.atu_configs_mut().insert_cfg_base_mapping(
                        extend_config.cfg_base as crate::pci::PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                    self.atu_configs_mut().insert_cfg_base_mapping(
                        cfg1_base as crate::pci::PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                    self.atu_configs_mut().insert_io_base_mapping(
                        rootcomplex_config.io_base as crate::pci::PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                }
            }
            #[cfg(loongarch64_pcie)]
            {
                use crate::pci::pci_handler::mmio_vpci_direct_handler;

                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_direct_handler,
                    rootcomplex_config.ecam_base as usize,
                );
                emergency_map_regions.push((
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                ));
            }
            #[cfg(not(any(ecam_pcie, dwc_pcie, loongarch64_pcie)))]
            {
                warn!(
                    "No extend config found for base 0x{:x}",
                    rootcomplex_config.ecam_base
                );
            }
        }

        // Note: emergency_map_regions requires access to self (for Zone), so this must be handled at Zone level
    }
}

static ZONE_LIST: RwLock<Vec<Arc<Zone>>> = RwLock::new(vec![]);

pub fn root_zone() -> Arc<Zone> {
    ZONE_LIST.read().get(0).cloned().unwrap()
}

pub fn is_this_root_zone() -> bool {
    Arc::ptr_eq(&this_zone(), &root_zone())
}

/// Add zone to CELL_LIST
pub fn add_zone(zone: Arc<Zone>) {
    ZONE_LIST.write().push(zone);
}

/// Remove zone from ZONE_LIST
pub fn remove_zone(zone_id: usize) {
    let mut zone_list = ZONE_LIST.write();
    let (idx, _) = zone_list
        .iter()
        .enumerate()
        .find(|(_, zone)| zone.id() == zone_id)
        .unwrap();
    let removed_zone = zone_list.remove(idx);
    assert_eq!(Arc::strong_count(&removed_zone), 1);
}

pub fn find_zone(zone_id: usize) -> Option<Arc<Zone>> {
    ZONE_LIST
        .read()
        .iter()
        .find(|zone| zone.id() == zone_id)
        .cloned()
}

pub fn all_zones_info() -> Vec<ZoneInfo> {
    let zone_list = ZONE_LIST.read();

    zone_list
        .iter()
        .map(|zone| ZoneInfo {
            zone_id: zone.id() as u32,
            cpus: zone.read().cpu_set().bitmap,
            name: zone.name(),
            is_err: zone.is_err() as u8,
        })
        .collect()
}

pub fn this_zone_id() -> usize {
    this_zone().id()
}

pub fn zone_create(config: &HvZoneConfig) -> HvResult<Arc<Zone>> {
    // we create the new zone here
    // TODO: create Zone with cpu_set
    let zone_id = config.zone_id as usize;

    if find_zone(zone_id).is_some() {
        return hv_result_err!(
            EINVAL,
            format!("Failed to create zone: zone_id {} already exists", zone_id)
        );
    }

    let mut zone = Zone::new(zone_id, &config.name);
    zone.pt_init(config.memory_regions())?;
    zone.mmio_init(&config.arch_config);

    let mut cpu_num = 0;
    for cpu_id in config.cpus().iter() {
        if let Some(existing_zone) = get_cpu_data(*cpu_id as _).zone.clone() {
            return hv_result_err!(
                EBUSY,
                format!(
                    "Failed to create zone: cpu {} already belongs to zone {}",
                    cpu_id,
                    existing_zone.id()
                )
            );
        }
        zone.write().cpu_set_mut().set_bit(*cpu_id as _);
        cpu_num += 1;
    }
    zone.write().set_cpu_num(cpu_num);

    #[cfg(pci)]
    {
        #[cfg(pci_init_delay)]
        {
            #[cfg(dwc_pcie)]
            {
                let num_pci_bus = config.num_pci_bus as usize;
                if zone_id == 0 {
                    let mut inner = zone.write();
                    inner.virtual_pci_dbi_pref_init(&config.pci_config, num_pci_bus);
                } else {
                    let _ = zone.virtual_pci_mmio_init(&config.pci_config, num_pci_bus);
                    let _ = zone.guest_pci_init(
                        zone_id,
                        &config.alloc_pci_devs,
                        config.num_pci_devs,
                        &config.pci_config,
                        num_pci_bus,
                    );
                }
            }
        }

        #[cfg(not(pci_init_delay))]
        {
            let _ = zone.virtual_pci_mmio_init(&config.pci_config, config.num_pci_bus as usize);
            let _ = zone.guest_pci_init(
                zone_id,
                &config.alloc_pci_devs,
                config.num_pci_devs,
                &config.pci_config,
                config.num_pci_bus as usize,
            );
        }
    }

    #[cfg(viommu)]
    {
        use crate::platform::{IOMMU_SYS_BASE, IOMMU_SYS_SIZE};
        // Create viommu instance and register mmio handler for target zone.
        crate::device::iommu::viommu_init(zone_id);
        crate::device::iommu::viommu_mmio_handler_register(&zone, IOMMU_SYS_BASE, IOMMU_SYS_SIZE);
    }

    // #[cfg(target_arch = "aarch64")]
    // zone.ivc_init(config.ivc_config());

    /* loongarch page table emergency */
    /* Kai: Maybe unnecessary but i can't boot vms on my 3A6000 PC without this function. */
    // #[cfg(target_arch = "loongarch64")]
    // zone.page_table_emergency(
    //     config.pci_config[0].ecam_base as _,
    //     config.pci_config[0].ecam_size as _,
    // )?;

    let cpu_set = zone.read().cpu_set();
    info!("zone cpu_set: {:#b}", cpu_set.bitmap);

    zone.arch_zone_pre_configuration(config)?;
    // #[cfg(target_arch = "aarch64")]
    // zone.ivc_init(config.ivc_config());

    #[cfg(all(iommu, target_arch = "aarch64"))]
    zone.iommu_pt_init(config.memory_regions(), &config.arch_config)?;

    /* loongarch page table emergency */
    /* Kai: Maybe unnecessary but i can't boot vms on my 3A6000 PC without this function. */
    // #[cfg(target_arch = "loongarch64")]
    // zone.page_table_emergency(
    //     config.pci_config.ecam_base as _,
    //     config.pci_config.ecam_size as _,
    // )?;

    /*zone.pci_init(
        &config.pci_config,
        config.num_pci_devs as _,
        &config.alloc_pci_devs,
    );*/

    zone.arch_zone_post_configuration(config)?;

    // Reset the zone arch-related resources, e.g. invalid data cache
    zone.arch_zone_reset(config)?;

    // Initialize the virtual interrupt controller, it needs zone.cpu_num
    zone.virqc_init(config);

    zone.irq_bitmap_init(config.interrupts_bitmap());

    let mut dtb_ipa = INVALID_ADDRESS as u64;
    for region in config.memory_regions() {
        // region contains config.dtb_load_paddr?
        if region.physical_start <= config.dtb_load_paddr
            && region.physical_start + region.size > config.dtb_load_paddr
        {
            dtb_ipa = region.virtual_start + config.dtb_load_paddr - region.physical_start;
        }
    }

    let new_zone_pointer = Arc::new(zone);
    {
        cpu_set.iter().for_each(|cpuid| {
            let cpu_data = get_cpu_data(cpuid);
            cpu_data.zone = Some(new_zone_pointer.clone());
            //chose boot cpu
            if cpuid == cpu_set.first_cpu().unwrap() {
                cpu_data.boot_cpu = true;
            }
            cpu_data.cpu_on_entry = config.entry_point as _;
            cpu_data.dtb_ipa = dtb_ipa as _;
            #[cfg(target_arch = "aarch64")]
            {
                cpu_data.arch_cpu.is_aarch32 = config.arch_config.is_aarch32 != 0;
            }
        });
    }

    Ok(new_zone_pointer)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    zone_id: u32,
    cpus: u64,
    name: [u8; CONFIG_NAME_MAXLEN],
    is_err: u8,
}
// Be careful about dead lock for zone.write()
pub fn zone_error() {
    if is_this_root_zone() {
        panic!("root zone has some error");
    }
    let zone = this_zone();
    let zone_id = zone.id();
    error!("zone {} has some error, please shut down it", zone_id);

    zone.set_err();
    drop(zone);
}

#[test_case]
fn test_add_and_remove_zone() {
    let zone_count = 50;
    let zone_count_before = ZONE_LIST.read().len();
    for i in 0..zone_count {
        let u8name_array = [i as u8; CONFIG_NAME_MAXLEN];
        let zone = Zone::new(i, &u8name_array);
        ZONE_LIST.write().push(Arc::new(zone));
    }
    for i in 0..zone_count {
        remove_zone(i);
    }
    assert_eq!(ZONE_LIST.read().len(), zone_count_before);
}
