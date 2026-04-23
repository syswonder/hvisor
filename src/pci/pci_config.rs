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

use alloc::collections::btree_map::BTreeMap;
use spin::{Lazy, Mutex};

use crate::{
    config::{HvPciConfig, HvPciDevConfig, CONFIG_MAX_PCI_DEV, CONFIG_PCI_BUS_MAXNUM},
    error::HvResult,
    pci::{
        msix::get_arch_msix_backend,
        pci_struct::{ArcRwLockVirtualPciConfigSpace, Bdf},
    },
    zone::Zone,
};

#[cfg(feature = "loongarch64_pcie")]
use alloc::vec::Vec;

#[cfg(any(
    all(feature = "iommu", target_arch = "aarch64"),
    all(feature = "iommu", target_arch = "riscv64"),
    target_arch = "x86_64"
))]
use crate::device::iommu::iommu_add_device_with_root_pt_addr;

#[cfg(feature = "ecam_pcie")]
use crate::pci::vpci_dev::{get_handler, VpciDevType};

#[cfg(any(
    feature = "ecam_pcie",
    feature = "dwc_pcie",
    feature = "loongarch64_pcie"
))]
use crate::pci::{mem_alloc::BaseAllocator, pci_struct::RootComplex};

#[cfg(feature = "ecam_pcie")]
use crate::pci::{config_accessors::ecam::EcamConfigAccessor, pci_handler::mmio_vpci_handler};

#[cfg(feature = "dwc_pcie")]
use crate::{
    memory::mmio_generic_handler,
    pci::{
        config_accessors::{
            dwc::DwcConfigAccessor, dwc::DwcConfigRegionBackend, dwc_atu::AtuConfig, PciRegionMmio,
        },
        pci_handler::{mmio_dwc_cfg_handler, mmio_dwc_io_handler, mmio_vpci_handler_dbi},
        PciConfigAddress,
    },
    platform,
};

#[cfg(feature = "loongarch64_pcie")]
use crate::pci::{
    config_accessors::loongarch64::LoongArchConfigAccessor, pci_handler::mmio_vpci_direct_handler,
};

pub static GLOBAL_PCIE_LIST: Lazy<Mutex<BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace>>> =
    Lazy::new(|| {
        let m = BTreeMap::new();
        Mutex::new(m)
    });

/* add all dev to GLOBAL_PCIE_LIST */
pub fn hvisor_pci_init(pci_config: &[HvPciConfig]) -> HvResult {
    warn!("begin {:#x?}", pci_config);

    // Track domains that have been initialized for DW MSI
    #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
    let mut initialized_domains: alloc::vec::Vec<u8> = alloc::vec::Vec::new();

    #[cfg(any(
        feature = "ecam_pcie",
        feature = "dwc_pcie",
        feature = "loongarch64_pcie"
    ))]
    for (_index, rootcomplex_config) in pci_config.iter().enumerate() {
        /* empty config */
        if rootcomplex_config.ecam_base == 0 {
            warn!("empty pcie config");
            continue;
        }

        let mut allocator = BaseAllocator::default();
        allocator.set_mem32(rootcomplex_config.mem32_base, rootcomplex_config.mem32_size);
        allocator.set_mem64(rootcomplex_config.mem64_base, rootcomplex_config.mem64_size);
        allocator.set_io(
            rootcomplex_config.io_base as u64,
            rootcomplex_config.io_size,
        );

        // TODO: refactor
        // in x86, we do not take the initiative to reallocate BAR space
        #[cfg(feature = "no_pcie_bar_realloc")]
        let allocator_opt: Option<BaseAllocator> = None;
        #[cfg(not(feature = "no_pcie_bar_realloc"))]
        let allocator_opt: Option<BaseAllocator> = Some(allocator);

        // #[cfg(feature = "loongarch64_pcie")]
        // let allocator_opt: Option<LoongArchAllocator> = {
        //     let mut allocator = LoongArchAllocator::default();
        //     allocator.set_mem(
        //         rootcomplex_config.mem64_base,
        //         rootcomplex_config.mem64_size,
        //     );
        //     allocator.set_io(
        //         rootcomplex_config.io_base,
        //         rootcomplex_config.io_size,
        //     );
        //     Some(allocator)
        // };

        let mut rootcomplex = {
            #[cfg(feature = "dwc_pcie")]
            {
                // warn!("dwc pcie");
                let ecam_base = rootcomplex_config.ecam_base;
                let atu_config = platform::ROOT_DWC_ATU_CONFIG
                    .iter()
                    .find(|atu_cfg| atu_cfg.ecam_base == ecam_base);

                let atu_config = match atu_config {
                    Some(cfg) => cfg,
                    None => {
                        warn!("No ATU config found for ecam_base 0x{:x}", ecam_base);
                        return hv_result_err!(EINVAL, "No ATU config found for ecam_base");
                    }
                };

                let root_bus = rootcomplex_config.bus_range_begin as u8;

                RootComplex::new_dwc(rootcomplex_config.ecam_base, atu_config, root_bus)
            }

            #[cfg(feature = "loongarch64_pcie")]
            {
                let root_bus = rootcomplex_config.bus_range_begin as u8;
                RootComplex::new_loongarch(
                    rootcomplex_config.ecam_base,
                    rootcomplex_config.ecam_size,
                    root_bus,
                )
            }

            #[cfg(feature = "ecam_pcie")]
            {
                RootComplex::new_ecam(rootcomplex_config.ecam_base)
            }
        };
        let range =
            rootcomplex_config.bus_range_begin as usize..rootcomplex_config.bus_range_end as usize;

        let domain = rootcomplex_config.domain;
        let e = rootcomplex.enumerate(Some(range), domain, allocator_opt);
        info!("begin enumerate {:#x?}", e);
        for node in e {
            info!("node {:#?}", node);
            GLOBAL_PCIE_LIST
                .lock()
                .insert(node.get_bdf(), ArcRwLockVirtualPciConfigSpace::new(node));
        }

        // Initialize DW MSI domain for this domain ID (only once per domain)
        #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
        {
            if !initialized_domains.contains(&domain) {
                let msi_irq = platform::ROOT_DWC_ATU_CONFIG
                    .iter()
                    .find(|cfg| cfg.ecam_base == rootcomplex_config.ecam_base)
                    .map(|cfg| cfg.dw_msi_irq as u32)
                    .unwrap_or(0);
                crate::pci::dwc_msi::init_dwc_msi_domain(domain, msi_irq)?;
                initialized_domains.push(domain);
            }
        }
    }
    info!("hvisor pci init done \n{:#?}", GLOBAL_PCIE_LIST);
    Ok(())
}

impl Zone {
    pub fn guest_pci_init(
        &mut self,
        _zone_id: usize,
        alloc_pci_devs: &[HvPciDevConfig; CONFIG_MAX_PCI_DEV],
        num_pci_devs: u64,
        pci_config: &[HvPciConfig],
        _num_pci_config: usize,
    ) -> HvResult {
        let mut inner = self.write();
        let mut guard = GLOBAL_PCIE_LIST.lock();
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
            #[cfg(feature = "dwc_pcie")]
            {
                use alloc::sync::Arc;
                let atu_config = platform::ROOT_DWC_ATU_CONFIG
                    .iter()
                    .find(|atu_cfg| atu_cfg.ecam_base == ecam_base);

                match atu_config {
                    Some(cfg) => {
                        let root_bus = bus_range_begin;
                        let accessor = Arc::new(DwcConfigAccessor::new(cfg, root_bus));
                        inner.vpci_bus_mut().set_accessor(accessor);
                    }
                    None => {
                        warn!("No ATU config found for ecam_base 0x{:x}", ecam_base);
                        continue;
                    }
                }
            }

            #[cfg(feature = "loongarch64_pcie")]
            {
                use alloc::sync::Arc;
                let root_bus = bus_range_begin;
                let accessor = Arc::new(LoongArchConfigAccessor::new(
                    ecam_base,
                    target_pci_config.ecam_size,
                    root_bus,
                ));
                inner.vpci_bus_mut().set_accessor(accessor);
            }

            #[cfg(feature = "ecam_pcie")]
            {
                use alloc::sync::Arc;
                let accessor = Arc::new(EcamConfigAccessor::new(ecam_base));
                inner.vpci_bus_mut().set_accessor(accessor);
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

            let mut vbus_pre = bus_range_begin;
            let mut bus_pre = bus_range_begin;
            let mut device_pre = 0u8;
            let mut domain_msi_count: u32 = 0;
            let mut vdevice_pre = 0u8;
            let msix_backend = get_arch_msix_backend();
            if let Some(x) = msix_backend.clone() {
                x.write().enable();
            }
            inner.vpci_bus_mut().set_msix_backend(msix_backend.clone());

            /*
             * To allow Linux to successfully recognize the devices we add, hvisor needs
             * to adjust the devices’ BDFs. Linux always assumes that the PCIe buses
             * it discovers are contiguous, and that device function numbers always start from 0.
             *
             * 1.   The bus number of a virtual BDF (vBDF) must start from range_begin and
             *      be contiguous. Once the physical bus number increases—regardless of
             *      how much it increases—the corresponding virtual bus number (vbus)
             *      can only increase by 1.
             *
             * 2.   If the function number of a vBDF is not 0, and it is found that
             *      the device with function 0 of the same vBDF does not belong to the current zone,
             *      then the function number of the current vBDF should be set to 0.
             */
            for dev_config in &filtered_devices {
                let bdf = Bdf::new_from_config(*dev_config);
                // let bus = bdf.bus();
                // let device = bdf.device();
                // let function = bdf.function();

                // /*
                //  * vfunction = if (bus != bus_pre || device != device_pre) && function != 0
                //  * In practice, remapping is performed only for new devices whose function is not 0;
                //  * however, the check for function != 0 does not affect the final result.
                //  */
                // let vfunction = if bus != bus_pre || device != device_pre {
                //     0
                // } else {
                //     function
                // };

                // let vbus = if bus > bus_pre {
                //     vbus_pre += 1;
                //     vbus_pre
                // } else {
                //     vbus_pre
                // };

                // // Remap device number to be contiguous, starting from 0
                // let vdevice = if bus != bus_pre || device != device_pre {
                //     // New bus or new device, increment device counter
                //     if bus != bus_pre {
                //         vdevice_pre = 0;
                //     } else {
                //         vdevice_pre += 1;
                //     }
                //     vdevice_pre
                // } else {
                //     // Same bus and device, keep the same virtual device number
                //     vdevice_pre
                // };

                // let vbdf = Bdf::new(bdf.domain(), vbus, vdevice, vfunction);

                // device_pre = device;
                // bus_pre = bus;

                // TODO: adjust vbdf will cause line interrupt injecet error, so remove it temporarily
                let vbdf = bdf;

                info!("set bdf {:#?} to vbdf {:#?}", bdf, vbdf);

                #[cfg(any(
                    all(feature = "iommu", target_arch = "aarch64"),
                    all(feature = "iommu", target_arch = "riscv64"),
                    target_arch = "x86_64"
                ))]
                {
                    let iommu_pt_addr = if inner.iommu_pt().is_some() {
                        inner.iommu_pt().unwrap().root_paddr()
                    } else {
                        0
                    };
                    let device_id = (dev_config.bus as usize) << 8
                        | (dev_config.device as usize) << 3
                        | dev_config.function as usize;
                    #[cfg(feature = "share_s2pt")]
                    iommu_add_device_with_root_pt_addr(
                        _zone_id,
                        device_id as _,
                        inner.gpm().root_paddr(),
                    );
                    #[cfg(not(feature = "share_s2pt"))]
                    iommu_add_device_with_root_pt_addr(_zone_id, device_id as _, iommu_pt_addr);
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
                        inner.vpci_bus_mut().insert(vbdf, vdev);
                    } else {
                        // Check if device is already allocated to another zone
                        if dev.get_zone_id().is_none() {
                            dev.set_zone_id(Some(_zone_id as u32));
                            let mut vdev_inner = dev.read().config_space.clone();
                            vdev_inner.set_vbdf(vbdf);
                        let msi_count = vdev_inner.get_msi_count();
                        domain_msi_count += msi_count;
                            inner.vpci_bus_mut().insert(vbdf, vdev_inner);
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
                    #[cfg(feature = "ecam_pcie")]
                    {
                        let dev_type = dev_config.dev_type;
                        warn!("dev_type:{:?}", dev_config);
                        match dev_type {
                            VpciDevType::Physical => {
                                warn!("can not find dev {:#?}", bdf);
                            }
                            _ => {
                                if let Some(_handler) = get_handler(dev_type) {
                                    use crate::pci::vpci_dev::virt_dev_init;
                                    let base = ecam_base
                                        + ((bdf.bus() as u64) << 20)
                                        + ((bdf.device() as u64) << 15)
                                        + ((bdf.function() as u64) << 12);
                                    let dev = virt_dev_init(bdf, base, dev_type);
                                    if let Some(x) = dev {
                                        inner.vpci_bus_mut().insert(vbdf, x);
                                    }
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
                #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
                {
                    // Get the DW MSI domain allocator and allocate hwbit
                    if let Some(mut domain_lock) =
                        crate::pci::dwc_msi::get_dwc_msi_domain_mut(target_domain)
                    {
                        if let Some(domain_msi) = domain_lock.get_mut(&target_domain) {
                            let zone_cpu_set = inner.cpu_set();
                            let target_cpu = zone_cpu_set.first_cpu().unwrap_or(0);
                            match domain_msi.allocate_for_cpu(target_cpu, domain_msi_count) {
                                Ok(hwirq_bit) => {
                                    // Register the MSI info for this domain
                                    inner.vpci_bus_mut().add_msi_count_for_domain(
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

                #[cfg(not(feature = "dwc_msi"))]
                {
                    // Without dwc_msi feature, just register without hardware bit allocation
                    inner.vpci_bus_mut().add_msi_count_for_domain(
                        target_domain,
                        domain_msi_count,
                        0, // hwirq_bit is 0 when not using dwc_msi
                    );
                }
            }
        }
        info!("vpci bus init done\n {:#x?}", inner.vpci_bus());
        Ok(())
    }

    pub fn virtual_pci_mmio_init(
        &mut self,
        pci_rootcomplex_config: &[HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
        _num_pci_config: usize,
    ) {
        #[cfg(feature = "loongarch64_pcie")]
        let mut emergency_map_regions: Vec<(usize, usize)> = Vec::new();

        let mut inner = self.write();
        for rootcomplex_config in pci_rootcomplex_config {
            /* empty config */

            if rootcomplex_config.ecam_base == 0 {
                continue;
            }
            #[cfg(feature = "ecam_pcie")]
            {
                // use crate::pci::pci_handler::mmio_vpci_direct_handler;
                inner.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_handler,
                    // mmio_vpci_direct_handler,
                    rootcomplex_config.ecam_base as usize,
                );
            }
            #[cfg(feature = "dwc_pcie")]
            {
                // Encode domain_id into the arg parameter: arg = ecam_base + domain_id
                // Since ecam_base is 4KB aligned, its low 12 bits are 0
                // domain_id (0-15) fits in the low bits without interfering
                let encoded_arg =
                    rootcomplex_config.ecam_base as usize + (rootcomplex_config.domain as usize);
                inner.mmio_region_register(
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
                        inner.mmio_region_register(
                            extend_config.apb_base as usize,
                            extend_config.apb_size as usize,
                            mmio_generic_handler,
                            extend_config.apb_base as usize,
                        );
                    }

                    let cfg_size_half = extend_config.cfg_size / 2;
                    let cfg0_base = extend_config.cfg_base;
                    if cfg0_base != 0 && cfg_size_half != 0 {
                        inner.mmio_region_register(
                            cfg0_base as usize,
                            cfg_size_half as usize,
                            mmio_dwc_cfg_handler,
                            cfg0_base as usize,
                        );
                    }

                    let cfg1_base = extend_config.cfg_base + cfg_size_half;
                    if cfg1_base != 0 && cfg_size_half != 0 {
                        inner.mmio_region_register(
                            cfg1_base as usize,
                            cfg_size_half as usize,
                            mmio_dwc_cfg_handler,
                            cfg1_base as usize,
                        );
                    }

                    if extend_config.io_cfg_atu_shared != 0 {
                        inner.mmio_region_register(
                            rootcomplex_config.io_base as usize,
                            rootcomplex_config.io_size as usize,
                            mmio_dwc_io_handler,
                            rootcomplex_config.io_base as usize,
                        );
                    }

                    let mut atu = AtuConfig::default();

                    let dbi_base = extend_config.dbi_base as PciConfigAddress;
                    let dbi_size = extend_config.dbi_size;
                    let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
                    let dbi_backend = DwcConfigRegionBackend::new(dbi_region);
                    if let Err(e) = atu.init_limit_hw_value(&dbi_backend) {
                        warn!("Failed to initialize ATU0 limit defaults: {:?}", e);
                    }

                    inner
                        .atu_configs_mut()
                        .insert_atu(rootcomplex_config.ecam_base as usize, atu);
                    inner.atu_configs_mut().insert_cfg_base_mapping(
                        extend_config.cfg_base as PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                    inner.atu_configs_mut().insert_io_base_mapping(
                        rootcomplex_config.io_base as PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                }
            }
            #[cfg(feature = "loongarch64_pcie")]
            {
                inner.mmio_region_register(
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
            #[cfg(not(any(
                feature = "ecam_pcie",
                feature = "dwc_pcie",
                feature = "loongarch64_pcie"
            )))]
            {
                warn!(
                    "No extend config found for base 0x{:x}",
                    rootcomplex_config.ecam_base
                );
            }
        }

        #[cfg(feature = "loongarch64_pcie")]
        {
            drop(inner);
            for (base, size) in emergency_map_regions {
                let _ = self.page_table_emergency(base, size);
            }
        }
    }
}
