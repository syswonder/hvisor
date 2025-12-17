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

use super::{
    mem_alloc::BaseAllocator,
    pci_access::{mmio_vpci_direct_handler, mmio_vpci_handler},
    pci_struct::{Bdf, VirtualPciConfigSpace, CONFIG_LENTH, RootComplex},
};

use crate::{
    memory::{mmio_perform_access, MMIOAccess},
    pci::{config_accessors::PciConfigMmio, pci_access::EndpointHeader},
    percpu::this_zone,
};

#[cfg(feature = "ecam_pcie")]
use crate::pci::config_accessors::ecam::EcamConfigAccessor;

pub static GLOBAL_PCIE_LIST_TEST: Lazy<Mutex<BTreeMap<Bdf, VirtualPciConfigSpace>>> =
    Lazy::new(|| {
        let m = BTreeMap::new();
        Mutex::new(m)
    });

#[cfg(feature = "ecam_pcie")]
pub fn pcie_test() {
    warn!("pcie test");
    let mut allocator = BaseAllocator::default();
    allocator.set_mem32(0x10000000, 0x2efeffff);
    allocator.set_mem64(0x8000000000, 0xffffffffff - 0x8000000000);

    let mut root = RootComplex::new_ecam(0x4010000000);
    for node in root.enumerate(None, Some(allocator)) {
        GLOBAL_PCIE_LIST_TEST.lock().insert(node.get_bdf(), node);
    }
    warn!("pcie guest init done");
    warn!("{:#?}", GLOBAL_PCIE_LIST_TEST);
}

pub fn pcie_guest_init() {
    let zone = this_zone();
    let vbus = &mut zone.write().vpci_bus;

    let mut guard = GLOBAL_PCIE_LIST_TEST.lock();

    let vbdf = Bdf::from_str("0000:00:00.0").unwrap();
    let bdf = Bdf::from_str("0000:00:00.0").unwrap();
    let base = 0x4010000000; // Base address for test
    let backend = EndpointHeader::new_with_region(PciConfigMmio::new(base, CONFIG_LENTH));
    let dev = VirtualPciConfigSpace::host_bridge(bdf, base, Arc::new(backend), (0x6u8,0x0u8,0u8));
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

pub fn ecam_pcie_guest_test() {
    let zone = this_zone();
    let bdf = Bdf::from_str("0000:00:01.0").unwrap();
    // Get base from VirtualPciConfigSpace and add offset
    // Use a block scope to ensure the read lock is released before calling mmio_vpci_direct_handler
    let address = {
        let vbus = &zone.read().vpci_bus;
        if let Some(vdev) = vbus.get(&bdf) {
            vdev.read().get_base()
        } else {
            warn!("can not find dev {:#?} for test", bdf);
            0
        }
    };
    let value = 0;
    let test_address = address + 0x14;

    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: false,
        value: value,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );


    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: true,
        value: 0xFFFF_FFFF,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );


    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: false,
        value: value,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );

    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: true,
        value: 0x80000000,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );

    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: false,
        value: value,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );

    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: true,
        value: 0x70000000,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );

    let mut mmio = MMIOAccess {
        address: test_address as _,
        size: 4,
        is_write: false,
        value: value,
    };
    let _ = mmio_vpci_direct_handler(&mut mmio, 0);
    info!(
        "mmio offset {:x}, is_wirte {}, size {}, value 0x{:x}",
        mmio.address, mmio.is_write, mmio.size, mmio.value
    );

    info!("pcie guest test passed");

    loop {}
}

pub fn dwc_pcie_guest_test() {
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
    use core::ptr::read_volatile;
    unsafe {
        let a = read_volatile(0x3c0000900 as *const u32);
        info!("{a}");
    };

    info!("pcie dwc test passed");
}
