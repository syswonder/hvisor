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
    config::{HvPciConfig, HvPciDevConfig, CONFIG_MAX_PCI_DEV, CONFIG_PCI_BUS_MAXNUM}, error::HvResult, memory::{mmio_perform_access, MMIOAccess}, pci::{
        mem_alloc::BaseAllocator,
        pci_access::mmio_vpci_handler,
        pci_struct::{Bdf, VirtualPciConfigSpace},
    }, zone::Zone
};

use super::pci_struct::RootComplex;

pub static GLOBAL_PCIE_LIST: Lazy<Mutex<BTreeMap<Bdf, VirtualPciConfigSpace>>> = Lazy::new(|| {
    let m = BTreeMap::new();
    Mutex::new(m)
});

/* add all dev to GLOBAL_PCIE_LIST */
pub fn hvisor_pci_init(pci_config: &[HvPciConfig]) -> HvResult {
    warn!("begin {:#?}", pci_config);
    for rootcomplex_config in pci_config {
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

        let mut rootcomplex = {
            #[cfg(feature = "dwc_pcie")]
            {
                warn!("dwc pcie");
                // DWC PCIe: need dbi_base, cfg_base, etc.
                // Get DWC related parameters from config
                // Use ecam_base as dbi_base for now, should read from config
                let dbi_base = 0x3c0400000; // TODO: should read from config
                let cfg_base = rootcomplex_config.ecam_base; // TODO: should read from config
                let cfg_size = rootcomplex_config.ecam_size; // TODO: should read from config
                let first_busno = rootcomplex_config.bus_range_begin as u8;
                
                RootComplex::new_dwc(dbi_base, cfg_base, cfg_size, first_busno, None)
            }
            
            #[cfg(feature = "loongarch64_pcie")]
            {
                RootComplex::new_loongarch(rootcomplex_config.ecam_base)
            }
            
            #[cfg(all(not(feature = "dwc_pcie"), not(feature = "loongarch64_pcie")))]
            {
                // default use ECAM
                RootComplex::new(rootcomplex_config.ecam_base)
            }
        };

        let e = rootcomplex.enumerate(None, Some(allocator));
        info!("begin enumerate {:#?}", e);
        for node in e{
            // Capabilities are already enumerated in get_node() during device discovery
            GLOBAL_PCIE_LIST.lock().insert(node.get_bdf(), node);
        }
    }
    info!("hvisor pci init done \n{:#?}", GLOBAL_PCIE_LIST);
    Ok(())
}

impl Zone {
    pub fn guest_pci_init(
        &mut self,
        alloc_pci_devs: &[HvPciDevConfig; CONFIG_MAX_PCI_DEV],
        num_pci_devs: u64,
    ) -> HvResult {
        let mut guard = GLOBAL_PCIE_LIST.lock();
        let mut i = 0;
        while i < num_pci_devs {
            let dev_config = alloc_pci_devs[i as usize];
            let bdf = Bdf::from_address(dev_config.bdf << 12);
            let vbdf = Bdf::from_address(dev_config.vbdf << 12);
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
            // self.mmio_region_register(
            //     0xfe270000 as usize,
            //     0x10000 as usize,
            //     mmio_vpci_handler_apb,
            //     0xfe270000,
            // );
            // self.mmio_region_register(
            //     0x3c0400000 as usize,
            //     0x400000 as usize,
            //     mmio_vpci_handler_dbi,
            //     0x3c0400000,
            // );
            // self.mmio_region_register(
            //     0xf2000000 as usize,
            //     0x2000000 as usize,
            //     mmio_vpci_handler_conf,
            //     0xf2000000,
            // );
        }
    }
}

pub fn mmio_vpci_handler_apb(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    mmio_perform_access(_base, mmio);
    warn!(
        "apb 0x{:x}+0x{:x} {} 0x{:x}",
        _base,
        mmio.address,
        if mmio.is_write { "write" } else { "read" },
        mmio.value
    );
    Ok(())
}
pub fn mmio_vpci_handler_dbi(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    mmio_perform_access(_base, mmio);
    warn!(
        "dbi 0x{:x}+0x{:x} {} 0x{:x}",
        _base,
        mmio.address,
        if mmio.is_write { "write" } else { "read" },
        mmio.value
    );
    Ok(())
}
pub fn mmio_vpci_handler_conf(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    mmio_perform_access(_base, mmio);
    warn!(
        "conf 0x{:x}+0x{:x} {} 0x{:x}",
        _base,
        mmio.address,
        if mmio.is_write { "write" } else { "read" },
        mmio.value
    );
    Ok(())
}
