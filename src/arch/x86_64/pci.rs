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
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{acpi, idt, mmio::MMIoDevice, pio::get_pio_bitmap, zone::HvArchZoneConfig},
    cpu_data::this_zone,
    error::HvResult,
    memory::{
        mmio_generic_handler, mmio_handle_access, mmio_perform_access, GuestPhysAddr, MMIOAccess,
    },
    zone::{this_zone_id, Zone},
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

fn get_pci_mmio_addr() -> Option<usize> {
    let addr = get_pio_bitmap(this_zone_id()).pci_config_addr as usize;
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
        value = get_pio_bitmap(this_zone_id()).pci_config_addr;

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
        get_pio_bitmap(this_zone_id()).pci_config_addr.set_bits(
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
