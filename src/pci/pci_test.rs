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
#![allow(dead_code)]
use core::str::FromStr;

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use spin::{lazy::Lazy, mutex::Mutex};

use crate::{
    memory::{MMIOAccess, mmio_perform_access},
    pci::{pci_access::EndpointHeader, config_accessors::PciConfigMmio},
    percpu::this_zone,
};

use super::pci_struct::CONFIG_LENTH;
use super::{
    mem_alloc::BaseAllocator,
    pci_access::mmio_vpci_handler,
    pci_struct::{Bdf, RootComplex, VirtualPciConfigSpace},
};

pub static GLOBAL_PCIE_LIST_TEST: Lazy<Mutex<BTreeMap<Bdf, VirtualPciConfigSpace>>> =
    Lazy::new(|| {
        let m = BTreeMap::new();
        Mutex::new(m)
    });

pub fn pcie_test() {
    info!("pcie test");
    let mut allocator = BaseAllocator::default();
    allocator.set_mem32(0x10000000, 0x2efeffff);
    allocator.set_mem64(0x8000000000, 0xffffffffff - 0x8000000000);

    let mut root = RootComplex::new(0x4010000000);
    for node in root.enumerate(None, Some(allocator)) {
        GLOBAL_PCIE_LIST_TEST.lock().insert(node.get_bdf(), node);
    }
}

pub fn pcie_guest_init() {
    let zone = this_zone();
    let vbus = &mut zone.write().vpci_bus;

    let mut guard = GLOBAL_PCIE_LIST_TEST.lock();

    let vbdf = Bdf::from_str("0000:00:00.0").unwrap();
    let bdf = Bdf::from_str("0000:00:00.0").unwrap();
    // warn!("address {}", bdf.to_address(0));
    let backend = EndpointHeader::new_with_region(PciConfigMmio::new(
        bdf.to_address(0) + 0x4010000000,
        CONFIG_LENTH,
    ));
    let dev = VirtualPciConfigSpace::host_bridge(bdf, Arc::new(backend));
    vbus.insert(vbdf, dev);

    let vbdf = Bdf::from_str("0000:00:01.0").unwrap();
    let bdf = Bdf::from_str("0000:00:01.0").unwrap();
    if let Some(mut dev) = guard.remove(&bdf) {
        // let _ = dev.write_hw(0x20, 4, 0xffffffff);
        // let value1 = dev.read_hw(0x20, 4).unwrap();
        // let _ = dev.write_hw(0x24, 4, 0xffffffff);
        // let value2 = dev.read_hw(0x24, 4).unwrap();
        // info!("{:#?} bar64 {:x}, {:x}", bdf, (value1 as u64), ((value2 as u64) << 32u64));
        dev.set_vbdf(vbdf);
        vbus.insert(vbdf, dev);
    } else {
        warn!("can not find dev");
    }

    let vbdf = Bdf::from_str("0000:00:02.0").unwrap();
    let bdf = Bdf::from_str("0000:00:02.0").unwrap();
    if let Some(mut dev) = guard.remove(&bdf) {
        dev.set_vbdf(vbdf);
        vbus.insert(vbdf, dev);
    } else {
        warn!("can not find dev");
    }

    let vbdf = Bdf::from_str("0000:00:03.0").unwrap();
    let bdf = Bdf::from_str("0000:00:03.0").unwrap();
    if let Some(mut dev) = guard.remove(&bdf) {
        dev.set_vbdf(vbdf);
        vbus.insert(vbdf, dev);
    } else {
        warn!("can not find dev");
    }

    info!("{:#?}", vbus);
    info!("pcie guest init done");
}

pub fn pcie_guest_test() {
    let mut mmio = MMIOAccess {
        address: Bdf::from_str("0000:00:01.0").unwrap().to_address(0x24) as _,
        size: 4,
        is_write: false,
        value: 0x0,
    };
    let ret = mmio_vpci_handler(&mut mmio, 0);
    info!("{:#?}", ret);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );

    info!("pcie guest test passed");
}

pub fn pcie_dwc_test() {
    info!("pcie dwc test begin");
    let mut mmio = MMIOAccess {
        address: 0x3c0400000,
        size: 4,
        is_write: false,
        value: 0x0,
    };
    let ret = mmio_perform_access(0, &mut mmio);
    info!("{:#?}", ret);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );
    use core::ptr::{read_volatile, write_volatile};
    unsafe {
        let a = read_volatile(0x3c0000900 as *const u32);
        info!("{a}");
    };

    info!("pcie dwc test passed");
}