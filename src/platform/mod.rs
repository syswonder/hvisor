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
use crate::{
    config::{
        HvConfigMemoryRegion, HvIvcConfig, HvPciConfig, HvPciDevConfig, HvZoneConfig,
        CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD, CONFIG_MAX_INTERRUPTS, CONFIG_MAX_IVC_CONFIGS,
        CONFIG_MAX_MEMORY_REGIONS, CONFIG_MAX_PCI_DEV, CONFIG_NAME_MAXLEN, CONFIG_PCI_BUS_MAXNUM,
    },
    consts::INVALID_ADDRESS,
};

pub mod __board; // riscv64 uses some private PLIC constants in board.rs ... so we have to `pub` it - wheatfox
pub use __board::*;

// define a macro to check if the length of the array is less than the limit
macro_rules! check {
    ($len:expr, $limit:expr, $name:expr) => {
        if $len > $limit {
            panic!("board's {} ({}) exceeds limit ({})", $name, $len, $limit);
        }
    };
}

pub fn platform_root_zone_config() -> HvZoneConfig {
    // fill zero for memory regions and interrupts

    let mut memory_regions = [HvConfigMemoryRegion {
        mem_type: 0,
        physical_start: 0,
        virtual_start: 0,
        size: 0,
    }; CONFIG_MAX_MEMORY_REGIONS];

    check!(
        ROOT_ZONE_MEMORY_REGIONS.len(),
        CONFIG_MAX_MEMORY_REGIONS,
        "ROOT_ZONE_MEMORY_REGIONS"
    );
    memory_regions[..ROOT_ZONE_MEMORY_REGIONS.len()].copy_from_slice(&ROOT_ZONE_MEMORY_REGIONS);

    let mut ivc_configs: [HvIvcConfig; 2] = [HvIvcConfig::default(); CONFIG_MAX_IVC_CONFIGS];
    let mut _num_ivc_configs = 0;

    #[cfg(target_arch = "aarch64")]
    {
        _num_ivc_configs = ROOT_ZONE_IVC_CONFIG.len() as _;
        ivc_configs[.._num_ivc_configs].copy_from_slice(&ROOT_ZONE_IVC_CONFIG);
    }

    let mut interrupts_bitmap = [0; CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD];
    interrupts_bitmap[..ROOT_ZONE_IRQS_BITMAP.len()].copy_from_slice(&ROOT_ZONE_IRQS_BITMAP);

    let mut name = [0; CONFIG_NAME_MAXLEN];
    check!(ROOT_ZONE_NAME.len(), CONFIG_NAME_MAXLEN, "ROOT_ZONE_NAME");
    name[..ROOT_ZONE_NAME.len()].copy_from_slice(ROOT_ZONE_NAME.as_bytes());

    let mut pci_devs = [HvPciDevConfig::default(); CONFIG_MAX_PCI_DEV];
    let mut _root_pci_cfg = [HvPciConfig::new_empty(); CONFIG_PCI_BUS_MAXNUM];
    let mut _num_pci_devs: u64 = 0;
    let mut _num_pci_bus: u64 = 0;

    #[cfg(feature = "pci")]
    {
        check!(ROOT_PCI_DEVS.len(), CONFIG_MAX_PCI_DEV, "ROOT_PCI_DEVS");
        pci_devs[..ROOT_PCI_DEVS.len()].copy_from_slice(&ROOT_PCI_DEVS);
        check!(
            ROOT_PCI_CONFIG.len(),
            CONFIG_PCI_BUS_MAXNUM,
            "ROOT_PCI_CONFIG"
        );
        _root_pci_cfg[..ROOT_PCI_CONFIG.len()].copy_from_slice(&ROOT_PCI_CONFIG);
        _num_pci_devs = ROOT_PCI_DEVS.len() as _;
        _num_pci_bus = ROOT_PCI_CONFIG.len() as _;
    }

    HvZoneConfig::new(
        0,
        ROOT_ZONE_CPUS,
        ROOT_ZONE_MEMORY_REGIONS.len() as u32,
        memory_regions,
        interrupts_bitmap,
        _num_ivc_configs as _,
        ivc_configs,
        ROOT_ZONE_ENTRY,
        ROOT_ZONE_KERNEL_ADDR,
        INVALID_ADDRESS as _,
        ROOT_ZONE_DTB_ADDR,
        INVALID_ADDRESS as _,
        name,
        ROOT_ARCH_ZONE_CONFIG,
        _num_pci_bus,
        _root_pci_cfg,
        _num_pci_devs,
        pci_devs,
    )
}
