use alloc::collections::btree_map::BTreeMap;
use spin::{Lazy, Mutex};

use crate::{
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

        let mut rootcomplex = RootComplex::new(rootcomplex_config.ecam_base);
        for node in rootcomplex.enumerate(None, Some(allocator)) {
            GLOBAL_PCIE_LIST.lock().insert(node.get_bdf(), node);
        }
    }
    info!("hvisor pci init \n{:#?}", GLOBAL_PCIE_LIST);
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
            let bdf = Bdf::from_address(dev_config.bdf);
            let vbdf = Bdf::from_address(dev_config.vbdf);
            if let Some(mut vdev) = guard.remove(&bdf) {
                vdev.set_vbdf(vbdf);
                self.vpci_bus.insert(vbdf, vdev);
            } else {
                warn!("can not find dev {:#?}", bdf);
            }
            i += 1;
        }
        info!("vpci bus init end\n {:#?}", self.vpci_bus);
        Ok(())
    }

    pub fn virtual_pci_mmio_init(
        &mut self,
        pci_rootcomplex_config: &[HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
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
