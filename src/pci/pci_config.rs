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
    config::{HvPciConfig, HvPciDevConfig, CONFIG_MAX_PCI_DEV, CONFIG_PCI_BUS_MAXNUM},
    error::HvResult,
    pci::{
        mem_alloc::BaseAllocator,
        pci_access::mmio_vpci_handler,
        pci_struct::{Bdf, VirtualPciConfigSpace},
    },
    zone::Zone,
};

use super::pci_struct::RootComplex;

pub static GLOBAL_PCIE_LIST: Lazy<Mutex<BTreeMap<Bdf, VirtualPciConfigSpace>>> = Lazy::new(|| {
    let m = BTreeMap::new();
    Mutex::new(m)
});

/* add all dev to GLOBAL_PCIE_LIST */
pub fn hvisor_pci_init(pci_rootcomplex_config: &[HvPciConfig; CONFIG_PCI_BUS_MAXNUM]) -> HvResult {
    for rootcomplex_config in pci_rootcomplex_config {
        /* empty config */
        if rootcomplex_config.ecam_base == 0 {
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
        #[cfg(target_arch = "x86_64")]
        let allocator_opt: Option<BaseAllocator> = None;
        #[cfg(not(target_arch = "x86_64"))]
        let allocator_opt: Option<BaseAllocator> = Some(allocator);

        let mut rootcomplex = RootComplex::new(rootcomplex_config.ecam_base);
        for mut node in rootcomplex.enumerate(None, allocator_opt) {
            node.capability_enumerate();
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
            let bdf = Bdf::from_address(dev_config.bdf << 12);
            let vbdf = Bdf::from_address(dev_config.vbdf << 12);
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
                iommu_add_device(self.id, (bdf.to_address(0) >> 12) as _, iommu_pt_addr);
            }
            if bdf.is_host_bridge() {
                if let Some(mut vdev) = guard.get(&bdf) {
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
        _num_pci_bus: u64,
    ) {
        for rootcomplex_config in pci_rootcomplex_config {
            /* empty config */
            if rootcomplex_config.ecam_base == 0 {
                continue;
            }
            self.mmio_region_register(
                rootcomplex_config.ecam_base as usize,
                rootcomplex_config.ecam_size as usize,
                mmio_vpci_handler,
                0,
            );
        }
    }
}
