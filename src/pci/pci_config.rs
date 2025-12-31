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
    pci::pci_struct::{ArcRwLockVirtualPciConfigSpace, Bdf},
    zone::Zone,
};

#[cfg(any(
    all(feature = "iommu", target_arch = "aarch64"),
    target_arch = "x86_64"
))]
use crate::arch::iommu::iommu_add_device;

#[cfg(feature = "ecam_pcie")]
use crate::pci::{
    pci_struct::VirtualPciConfigSpace,
    vpci_dev::{get_handler, VpciDevType},
};

#[cfg(any(
    feature = "ecam_pcie",
    feature = "dwc_pcie",
    feature = "loongarch64_pcie"
))]
use crate::pci::{
    mem_alloc::BaseAllocator, pci_struct::RootComplex,
};

#[cfg(feature = "ecam_pcie")]
use crate::pci::pci_handler::mmio_vpci_handler;
#[cfg(feature = "dwc_pcie")]
use crate::{
    memory::mmio_generic_handler,
    pci::{
        config_accessors::{
            dwc::DwcConfigRegionBackend,
            dwc_atu::AtuConfig,
            PciRegionMmio,
        },
        pci_handler::{mmio_dwc_cfg_handler, mmio_dwc_io_handler, mmio_vpci_handler_dbi},
        PciConfigAddress,
    },
    platform,
};

#[cfg(feature = "loongarch64_pcie")]
use crate::pci::pci_handler::mmio_vpci_direct_handler;

pub static GLOBAL_PCIE_LIST: Lazy<Mutex<BTreeMap<Bdf, ArcRwLockVirtualPciConfigSpace>>> =
    Lazy::new(|| {
        let m = BTreeMap::new();
        Mutex::new(m)
    });

/* add all dev to GLOBAL_PCIE_LIST */
pub fn hvisor_pci_init(pci_config: &[HvPciConfig]) -> HvResult {
    warn!("begin {:#?}", pci_config);
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
                warn!("dwc pcie");
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
        info!("begin enumerate {:#?}", e);
        for node in e {
            info!("node {:#?}", node);
            GLOBAL_PCIE_LIST
                .lock()
                .insert(node.get_bdf(), ArcRwLockVirtualPciConfigSpace::new(node));
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
                let bus = bdf.bus();
                let device = bdf.device();
                let function = bdf.function();

                /*
                 * vfunction = if (bus != bus_pre || device != device_pre) && function != 0
                 * In practice, remapping is performed only for new devices whose function is not 0;
                 * however, the check for function != 0 does not affect the final result.
                 */
                let vfunction = if bus != bus_pre || device != device_pre {
                    0
                } else {
                    function
                };

                let vbus = if bus > bus_pre {
                    vbus_pre += 1;
                    vbus_pre
                } else {
                    vbus_pre
                };

                let vbdf = Bdf::new(bdf.domain(), vbus, device, vfunction);

                device_pre = device;
                bus_pre = bus;

                info!("set bdf {:#?} to vbdf {:#?}", bdf, vbdf);

                #[cfg(any(
                    all(feature = "iommu", target_arch = "aarch64"),
                    target_arch = "x86_64"
                ))]
                {
                    let iommu_pt_addr = if self.iommu_pt.is_some() {
                        self.iommu_pt.as_ref().unwrap().root_paddr()
                    } else {
                        0
                    };
                    let device_id = (dev_config.bus as usize) << 8
                        | (dev_config.device as usize) << 3
                        | dev_config.function as usize;
                    iommu_add_device(_zone_id, device_id as _, iommu_pt_addr);
                }

                // Insert device into vpci_bus with calculated vbdf
                if let Some(dev) = guard.get(&bdf) {
                    if bdf.is_host_bridge(dev.read().get_host_bdf().bus())
                        || dev.with_config_value(|config_value| -> bool {
                            config_value.get_class().0 == 0x6
                        })
                    {
                        let mut vdev = dev.read().clone();
                        vdev.set_vbdf(vbdf);
                        self.vpci_bus.insert(vbdf, vdev);
                    } else {
                        let vdev = guard.remove(&bdf).unwrap();
                        let mut vdev_inner = vdev.read().clone();
                        vdev_inner.set_vbdf(vbdf);
                        self.vpci_bus.insert(vbdf, vdev_inner);
                    }
                } else {
                    // warn!("can not find dev {:#?}", bdf);
                    #[cfg(feature = "ecam_pcie")]
                    {
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
                                    self.vpci_bus.insert(vbdf, dev);
                                } else {
                                    warn!("can not find dev {:#?}, unknown device type", bdf);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("vpci bus init done\n {:#?}", self.vpci_bus);
        Ok(())
    }

    pub fn virtual_pci_mmio_init(
        &mut self,
        pci_rootcomplex_config: &[HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
        _num_pci_config: usize,
    ) {
        for rootcomplex_config in pci_rootcomplex_config {
            /* empty config */
            if rootcomplex_config.ecam_base == 0 {
                continue;
            }
            #[cfg(feature = "ecam_pcie")]
            {
                // use crate::pci::pci_handler::mmio_vpci_direct_handler;
                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_handler,
                    // mmio_vpci_direct_handler,
                    rootcomplex_config.ecam_base as usize,
                );
            }
            #[cfg(feature = "dwc_pcie")]
            {
                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_handler_dbi,
                    rootcomplex_config.ecam_base as usize,
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

                    let dbi_base = extend_config.dbi_base as PciConfigAddress;
                    let dbi_size = extend_config.dbi_size;
                    let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
                    let dbi_backend = DwcConfigRegionBackend::new(dbi_region);
                    if let Err(e) = atu.init_limit_hw_value(&dbi_backend) {
                        warn!("Failed to initialize ATU0 limit defaults: {:?}", e);
                    }

                    self.atu_configs
                        .insert_atu(rootcomplex_config.ecam_base as usize, atu);
                    self.atu_configs.insert_cfg_base_mapping(
                        extend_config.cfg_base as PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                    self.atu_configs.insert_io_base_mapping(
                        rootcomplex_config.io_base as PciConfigAddress,
                        rootcomplex_config.ecam_base as usize,
                    );
                }
            }
            #[cfg(feature = "loongarch64_pcie")]
            {
                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_direct_handler,
                    rootcomplex_config.ecam_base as usize,
                );
                let _ = self.page_table_emergency(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                );
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
    }
}
