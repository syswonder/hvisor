use crate::{
    arch::{acpi, mmio::MMIoDevice, zone::HvArchZoneConfig},
    error::HvResult,
    memory::{mmio_generic_handler, GuestPhysAddr, MMIOAccess},
    zone::Zone,
};
use ::acpi::{mcfg::Mcfg, sdt::Signature};
use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
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
            self.mmio_region_register(start, size, mmio_generic_handler, 0);
        }
    }
}

/*fn pci_config_space_mmio_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    mmio_handler(mmio, &VIRT_PCI_CONFIG_SPACE.0)
}*/

pub fn get_config_space_info() -> HvResult<(u64, u64)> {
    let bytes = acpi::root_get_table(&Signature::MCFG)
        .unwrap()
        .get_bytes()
        .clone();
    let mcfg = unsafe { &*(bytes.as_ptr() as *const Mcfg) };

    for entry in mcfg.entries() {
        assert!(entry.pci_segment_group == 0);
        let size = ((entry.bus_number_end as u64 - entry.bus_number_start as u64) + 1) << 20;
        return Ok((entry.base_address, size));
    }

    hv_result_err!(ENODEV)
}

pub fn probe_root_pci_devices(config_base_hpa: usize) -> (Vec<u16>, usize, u8) {
    let mut bdfs: Vec<u16> = Vec::new();
    let mut config_space_size = 0usize;

    // info!("entry start: {:x} size: {:x}", start, size);
    let mut buses: VecDeque<u8> = VecDeque::new();
    let mut max_bus: u8 = 0;
    buses.push_back(max_bus);

    while !buses.is_empty() {
        let bus = buses.pop_front().unwrap();
        let bus_config_hpa = (config_base_hpa as usize) + ((bus as usize) << 20);
        let mut bus_empty: bool = true;

        for dev_func in 0u8..=255 {
            let bdf = ((bus as u16) << 8) + (dev_func as u16);
            let bdf_config_hpa = bus_config_hpa + ((dev_func as usize) << 12);

            let vendor_id = unsafe { *(bdf_config_hpa as *const u16) };
            if vendor_id == 0xffff {
                continue;
            }

            let device_id = unsafe { *((bdf_config_hpa + 0x2) as *const u16) };
            let header_type = unsafe { *((bdf_config_hpa + 0xe) as *const u8) };

            info!(
                    "bdf: {:x}, bus: {:x}, dev_func: {:x}, vendor id: {:x}, device id: {:x}, header type: {:x}",
                    bdf, bus, dev_func, vendor_id, device_id, header_type
                );

            bdfs.push(bdf);
            bus_empty = false;

            // pci bridge
            if header_type == 0x1 {
                let secondary_bus = unsafe { *((bdf_config_hpa + 0x19) as *const u8) };
                buses.push_back(secondary_bus);
            }
        }

        if !bus_empty && bus > max_bus {
            max_bus = bus;
        }
    }

    config_space_size = ((max_bus as usize - 0usize) + 1) << 20;
    // info!("config space size: {:x}", config_space_size);

    (bdfs, config_space_size, max_bus)
}
