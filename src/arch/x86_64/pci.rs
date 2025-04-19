use crate::{
    arch::{acpi, idt, mmio::MMIoDevice, zone::HvArchZoneConfig},
    error::HvResult,
    memory::{
        mmio_generic_handler, mmio_handle_access, mmio_perform_access, GuestPhysAddr, MMIOAccess,
    },
    percpu::this_zone,
    zone::Zone,
};
use ::acpi::{mcfg::Mcfg, sdt::Signature};
use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    sync::Arc,
    vec::Vec,
};
use bit_field::BitField;
use core::{mem::size_of, ops::Range, panic};

use super::{
    pio::{PCI_CONFIG_ADDR_PORT, PCI_CONFIG_DATA_PORT},
    vmx::VmxIoExitInfo,
};

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

pub fn probe_root_pci_devices(
    config_base_hpa: usize,
) -> (
    Vec<usize>,
    BTreeMap<usize, usize>,
    BTreeMap<usize, usize>,
    usize,
    u8,
) {
    let mut bdfs: Vec<usize> = Vec::new();
    // key: data reg hpa, value: bdf
    let mut msi_data_reg_map: BTreeMap<usize, usize> = BTreeMap::new();
    // key: msi-x table bar, value: bdf
    let mut msix_bar_map: BTreeMap<usize, usize> = BTreeMap::new();
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

            bdfs.push(bdf as _);
            bus_empty = false;

            // pci bridge
            if header_type == 0x1 {
                let secondary_bus = unsafe { *((bdf_config_hpa + 0x19) as *const u8) };
                buses.push_back(secondary_bus);
            }

            // probe msi/msi-x capability registers
            let mut cap_pointer = unsafe { *((bdf_config_hpa + 0x34) as *const u8) } as usize;
            while cap_pointer != 0 {
                let cap_hpa = bdf_config_hpa + cap_pointer;
                let cap_id = unsafe { *(cap_hpa as *const u8) };

                if cap_id == 0x5 {
                    // msi capablility
                    let msg_ctrl_reg = unsafe { *((cap_hpa + 0x2) as *const u16) };
                    let is_64b = msg_ctrl_reg.get_bit(7);

                    let data_reg_hpa = match is_64b {
                        true => cap_hpa + 0xc,
                        false => cap_hpa + 0x8,
                    };
                    msi_data_reg_map.insert(data_reg_hpa, bdf as _);
                    info!("msi data reg hpa: {:x?}", data_reg_hpa);
                } else if cap_id == 0x11 {
                    // msi-x capability
                    let msg_ctrl_reg = unsafe { *((cap_hpa + 0x2) as *const u16) };
                    let table_size = msg_ctrl_reg.get_bits(0..=10) as usize;
                    let table_bir =
                        unsafe { *((cap_hpa + 0x4) as *const u16) }.get_bits(0..=2) as usize;

                    // find msi-x table bar
                    let bar_hpa = bdf_config_hpa + 0x10 + (table_bir) * size_of::<u32>();
                    let mut bar = unsafe { *(bar_hpa as *const u32) } as usize;
                    assert!(!bar.get_bit(0)); // memory request
                    match bar.get_bits(1..=2) {
                        0b00 => {
                            // 32-bit decoding
                            bar &= !(0xfff);
                        }
                        0b10 => {
                            // 64-bit decoding
                            let bar_high =
                                unsafe { *((bar_hpa + size_of::<u32>()) as *const u32) } as usize;
                            bar = (bar_high << 6) + bar.get_bits(26..=31);
                        }
                        _ => {
                            panic!("MSI-X table BAR type error!");
                        }
                    }

                    info!(
                        "table size: {:x}, table bir: {:x}, bar: {:x}",
                        table_size, table_bir, bar
                    );
                    msix_bar_map.insert(bar, bdf as _);

                    for i in 0..=table_size {
                        let data_reg_hpa = bar + i * size_of::<u128>() + 2 * size_of::<u32>();
                        msi_data_reg_map.insert(data_reg_hpa, bdf as _);
                        info!("msi-x data reg hpa: {:x?}", data_reg_hpa);
                    }
                }

                info!("cap id: {:x}, hpa: {:x}", cap_id, cap_hpa);
                cap_pointer = unsafe { *((cap_hpa + 1) as *const u8) } as usize;
            }
        }

        if !bus_empty && bus > max_bus {
            max_bus = bus;
        }
    }

    config_space_size = ((max_bus as usize - 0usize) + 1) << 20;
    // info!("config space size: {:x}", config_space_size);

    (
        bdfs,
        msi_data_reg_map,
        msix_bar_map,
        config_space_size,
        max_bus,
    )
}

pub fn mmio_msix_table_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    let hpa = base + mmio.address;

    let zone = this_zone();
    let zone_id = zone.read().id;

    let bdf = acpi::is_msi_data_reg(hpa);
    if bdf.is_some() && zone.write().pciroot.is_assigned_device(bdf.unwrap()) {
        mmio_msi_data_reg_handler(mmio, base, bdf.unwrap(), zone_id)
    } else {
        mmio_perform_access(base, mmio);
        Ok(())
    }
}

pub fn mmio_msi_data_reg_handler(
    mmio: &mut MMIOAccess,
    base: usize,
    bdf: usize,
    zone_id: usize,
) -> HvResult {
    let hpa = base + mmio.address;

    let host_vector = unsafe { core::ptr::read_volatile(hpa as *mut u32) } as u8;
    if mmio.is_write {
        let alloc_host_vector = idt::get_host_vector(mmio.value as _, zone_id).unwrap();
        if host_vector != alloc_host_vector {
            idt::clear_vectors(host_vector, zone_id);
            mmio.value = alloc_host_vector as _;
            mmio_perform_access(base, mmio);
        }
    } else {
        if let Some(guest_vector) = idt::get_guest_vector(host_vector, zone_id) {
            mmio.value = guest_vector as _;
        } else {
            mmio.value = host_vector as _;
        }
    }
    trace!(
        "mmio_msi_data_reg_handler! hpa: {:x}, bdf: {:x}, is write: {:x?}, read value: {:x}, write value: {:x}",
        base + mmio.address,
        bdf,
        mmio.is_write,
        host_vector,
        mmio.value
    );
    Ok(())
}

fn get_pci_mmio_addr() -> Option<usize> {
    let addr = this_zone().read().pio_bitmap.pci_config_addr as usize;
    let (base, _) = crate::arch::acpi::root_get_config_space_info().unwrap();

    let enable = addr.get_bit(31);
    let bdf = addr.get_bits(8..=23);
    let reg = addr.get_bits(2..=7);

    if enable {
        // info!("pio: {:x}, bdf: {:x}", base + (bdf << 12) + (reg << 2), bdf);
        Some(base + (bdf << 12) + (reg << 2))
    } else {
        None
    }
}

pub fn handle_pci_config_port_read(io_info: &VmxIoExitInfo) -> u32 {
    let mut value = 0u32;
    if PCI_CONFIG_ADDR_PORT.contains(&io_info.port) {
        value = this_zone().read().pio_bitmap.pci_config_addr;

        let offset_bit = 8 * (io_info.port - PCI_CONFIG_ADDR_PORT.start) as usize;
        value = value.get_bits(offset_bit..offset_bit + (8 * io_info.access_size) as usize);
    } else {
        if let Some(mmio_addr) = get_pci_mmio_addr() {
            let offset: usize = (io_info.port - PCI_CONFIG_DATA_PORT.start) as usize;
            if this_zone()
                .read()
                .find_mmio_region(mmio_addr + offset, io_info.access_size as _)
                .is_some()
            {
                let mut mmio_access = MMIOAccess {
                    address: mmio_addr + offset,
                    size: io_info.access_size as _,
                    is_write: false,
                    value: 0,
                };
                mmio_handle_access(&mut mmio_access);
                value = mmio_access.value as _;
                // info!("value: {:x}", value);
            }
        }
    }
    value
}

pub fn handle_pci_config_port_write(io_info: &VmxIoExitInfo, value: u32) {
    if PCI_CONFIG_ADDR_PORT.contains(&io_info.port) {
        let offset_bit = 8 * (io_info.port - PCI_CONFIG_ADDR_PORT.start) as usize;
        this_zone().write().pio_bitmap.pci_config_addr.set_bits(
            offset_bit..offset_bit + (8 * (io_info.access_size as usize)),
            value,
        );
    } else {
        if let Some(mmio_addr) = get_pci_mmio_addr() {
            let offset: usize = (io_info.port - PCI_CONFIG_DATA_PORT.start) as usize;
            if this_zone()
                .read()
                .find_mmio_region(mmio_addr + offset, io_info.access_size as _)
                .is_some()
            {
                mmio_handle_access(&mut MMIOAccess {
                    address: mmio_addr + offset,
                    size: io_info.access_size as _,
                    is_write: true,
                    value: value as _,
                });
            }
        }
    }
}
