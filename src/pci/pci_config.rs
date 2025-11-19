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
    arch::iommu::iommu_add_device,
    config::{CONFIG_MAX_PCI_DEV, CONFIG_PCI_BUS_MAXNUM, HvPciConfig, HvPciDevConfig},
    error::HvResult,
    memory::mmio_generic_handler,
    pci::{
        config_accessors::BdfAddressConversion, mem_alloc::BaseAllocator, pci_access::{mmio_vpci_handler, mmio_vpci_handler_dbi}, pci_struct::{Bdf, VirtualPciConfigSpace}
    },
    zone::Zone,
};

use crate::platform;
use super::pci_struct::RootComplex;

pub static GLOBAL_PCIE_LIST: Lazy<Mutex<BTreeMap<Bdf, VirtualPciConfigSpace>>> = Lazy::new(|| {
    let m = BTreeMap::new();
    Mutex::new(m)
});

/* add all dev to GLOBAL_PCIE_LIST */
pub fn hvisor_pci_init(pci_config: &[HvPciConfig]) -> HvResult {
    warn!("begin {:#?}", pci_config);
    #[cfg(any(feature = "ecam_pcie", feature = "dwc_pcie", feature = "loongarch64_pcie"))]
    for (index, rootcomplex_config) in pci_config.iter().enumerate() {
        /* empty config */
        if rootcomplex_config.ecam_base == 0 {
            warn!("empty pcie config");
            continue;
        }

        let mut allocator = BaseAllocator::default();
        allocator.set_mem32(
            rootcomplex_config.pci_mem32_base as u32,
            rootcomplex_config.mem32_size as u32,
        );
        allocator.set_mem64(
            rootcomplex_config.pci_mem64_base,
            rootcomplex_config.mem64_size,
        );

        // TODO: refactor
        // in x86, we do not take the initiative to reallocate BAR space
        #[cfg(target_arch = "no_pcie_bar_realloc")]
        let allocator_opt: Option<BaseAllocator> = None;
        #[cfg(not(target_arch = "no_pcie_bar_realloc"))]
        let allocator_opt: Option<BaseAllocator> = Some(allocator);

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
                RootComplex::new_loongarch(rootcomplex_config.ecam_base)
            }
            
            #[cfg(feature = "ecam_pcie")]
            {
                RootComplex::new_ecam(rootcomplex_config.ecam_base)
            }
            
        };
        let range = rootcomplex_config.bus_range_begin as usize..rootcomplex_config.bus_range_end as usize;
        let e = rootcomplex.enumerate(Some(range), allocator_opt);
        info!("begin enumerate {:#?}", e);
        for node in e {
            info!("node {:#?}", node);
            GLOBAL_PCIE_LIST.lock().insert(node.get_bdf(), node);
        }
    }
    info!("hvisor pci init done \n{:#?}", GLOBAL_PCIE_LIST);
    Ok(())
}

impl Zone {
    pub fn guest_pci_init(
        &mut self,
        zone_id: usize,
        alloc_pci_devs: &[HvPciDevConfig; CONFIG_MAX_PCI_DEV],
        num_pci_devs: u64,
    ) -> HvResult {
        let mut guard = GLOBAL_PCIE_LIST.lock();
        let mut i = 0;
        while i < num_pci_devs {
            let dev_config = alloc_pci_devs[i as usize];
            let bdf = <Bdf as BdfAddressConversion>::from_address(dev_config.bdf << 12);
            let vbdf = <Bdf as BdfAddressConversion>::from_address(dev_config.vbdf << 12);
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
                iommu_add_device(zone_id, dev_config.bdf as _, iommu_pt_addr);
            }
            if bdf.is_host_bridge() {
                if let Some(vdev) = guard.get(&bdf) {
                    let mut vdev = vdev.clone();
                    vdev.set_vbdf(vbdf);
                    self.vpci_bus.insert(vbdf, vdev);
                } else {
                    warn!("can not find host bridge {:#?}", bdf);
                }
            } else {
                if let Some(mut vdev) = guard.remove(&bdf) {
                    vdev.set_vbdf(vbdf);
                    self.vpci_bus.insert(vbdf, vdev);
                } else {
                    warn!("can not find dev {:#?}", bdf);
                }
            }
            i += 1;
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
                self.mmio_region_register(
                rootcomplex_config.ecam_base as usize,
                rootcomplex_config.ecam_size as usize,
                mmio_vpci_handler,
                rootcomplex_config.ecam_base as usize,
            );
            }
            // #[cfg(feature = "dwc_pcie")]
            {
                self.mmio_region_register(
                    rootcomplex_config.ecam_base as usize,
                    rootcomplex_config.ecam_size as usize,
                    mmio_vpci_handler_dbi,
                    rootcomplex_config.ecam_base as usize,
                );
                #[cfg(feature = "dwc_pcie")]
                {
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
                                mmio_vpci_handler,
                                cfg0_base as usize,
                            );
                        }
                        
                        let cfg1_base = extend_config.cfg_base + cfg_size_half;
                        if cfg1_base != 0 && cfg_size_half != 0 {
                            self.mmio_region_register(
                                cfg1_base as usize,
                                cfg_size_half as usize,
                                mmio_vpci_handler,
                                cfg1_base as usize,
                            );
                        }
                    } else {
                        warn!("No extend config found for base 0x{:x}", rootcomplex_config.ecam_base);
                    }
                }
            }
        }
    }
}