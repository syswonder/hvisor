use crate::{
    arch::{
        acpi,
        mmio::{mmio_handler, MMIoDevice},
        zone::HvArchZoneConfig,
    },
    error::HvResult,
    memory::{GuestPhysAddr, MMIOAccess},
    zone::Zone,
};
use ::acpi::{mcfg::Mcfg, sdt::Signature};
use alloc::{sync::Arc, vec::Vec};
use core::ops::Range;

lazy_static::lazy_static! {
    static ref VIRT_PCI_CONFIG_SPACE: (Arc<dyn MMIoDevice>,) = (Arc::new(VirtPciConfigSpace::new()),);
}

pub struct VirtPciConfigSpace {}

impl VirtPciConfigSpace {
    fn new() -> Self {
        Self {}
    }
}

impl MMIoDevice for VirtPciConfigSpace {
    fn gpa_range(&self) -> &Vec<Range<usize>> {
        todo!()
    }

    fn read(&self, gpa: GuestPhysAddr) -> HvResult<u64> {
        let value = unsafe { core::ptr::read_unaligned(gpa as *const u64) };
        // info!("pci config read! gpa: {:x}, value: {:x}", gpa, value);
        Ok(value)
    }

    fn write(&self, gpa: GuestPhysAddr, value: u64, size: usize) -> HvResult {
        info!(
            "pci config write! gpa: {:x}, value: {:x}, size: {:x}",
            gpa, value, size,
        );

        todo!()
    }

    fn trigger(&self, signal: usize) -> HvResult {
        todo!()
    }
}

impl Zone {
    pub fn pci_config_space_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        let bytes = acpi::root_get_table(&Signature::MCFG)
            .unwrap()
            .get_bytes()
            .clone();
        let mcfg = unsafe { &*(bytes.as_ptr() as *const Mcfg) };

        for entry in mcfg.entries() {
            let start = entry.base_address as usize;
            let size =
                ((entry.bus_number_end as usize - entry.bus_number_start as usize) + 1) << 20;
            // info!("entry start: {:x} size: {:x}", start, size);
            self.mmio_region_register(start, size, pci_config_space_mmio_handler, 0);
        }
    }
}

fn pci_config_space_mmio_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    mmio_handler(mmio, &VIRT_PCI_CONFIG_SPACE.0)
}
