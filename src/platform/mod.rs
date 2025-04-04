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
        HvConfigMemoryRegion, HvIvcConfig, HvPciConfig, HvZoneConfig, CONFIG_MAX_INTERRUPTS,
        CONFIG_MAX_IVC_CONGIGS, CONFIG_MAX_MEMORY_REGIONS, CONFIG_MAX_PCI_DEV, CONFIG_NAME_MAXLEN,
    },
    consts::INVALID_ADDRESS,
};

pub mod __board; // riscv64 uses some private PLIC constants in board.rs ... so we have to `pub` it - wheatfox
pub use __board::*;

pub fn platform_root_zone_config() -> HvZoneConfig {
    // fill zero for memory regions and interrupts

    let mut memory_regions = [HvConfigMemoryRegion {
        mem_type: 0,
        physical_start: 0,
        virtual_start: 0,
        size: 0,
    }; CONFIG_MAX_MEMORY_REGIONS];

    memory_regions[..ROOT_ZONE_MEMORY_REGIONS.len()].copy_from_slice(&ROOT_ZONE_MEMORY_REGIONS);

    let mut ivc_configs = [HvIvcConfig::default(); CONFIG_MAX_IVC_CONGIGS];
    let mut num_ivc_configs = 0;
    #[cfg(target_arch = "aarch64")]
    {
        num_ivc_configs = ROOT_ZONE_IVC_CONFIG.len() as _;
        ivc_configs[..num_ivc_configs].copy_from_slice(&ROOT_ZONE_IVC_CONFIG);
    }

    let mut interrupts = [0; CONFIG_MAX_INTERRUPTS];
    interrupts[..ROOT_ZONE_IRQS.len()].copy_from_slice(&ROOT_ZONE_IRQS);

    let mut name = [0; CONFIG_NAME_MAXLEN];
    name[..ROOT_ZONE_NAME.len()].copy_from_slice(ROOT_ZONE_NAME.as_bytes());

    let mut pci_devs = [0; CONFIG_MAX_PCI_DEV];
    let mut root_pci_cfg = HvPciConfig::new_empty();
    let mut num_pci_devs: u64 = 0;
    #[cfg(all(feature = "pci", target_arch = "aarch64"))]
    {
        pci_devs[..ROOT_PCI_DEVS.len()].copy_from_slice(&ROOT_PCI_DEVS);
        root_pci_cfg = ROOT_PCI_CONFIG;
        num_pci_devs = ROOT_PCI_DEVS.len() as _;
    }

    HvZoneConfig::new(
        0,
        ROOT_ZONE_CPUS,
        ROOT_ZONE_MEMORY_REGIONS.len() as u32,
        memory_regions,
        ROOT_ZONE_IRQS.len() as u32,
        interrupts,
        num_ivc_configs as _,
        ivc_configs,
        ROOT_ZONE_ENTRY,
        ROOT_ZONE_KERNEL_ADDR,
        INVALID_ADDRESS as _,
        ROOT_ZONE_DTB_ADDR,
        INVALID_ADDRESS as _,
        name,
        ROOT_ARCH_ZONE_CONFIG,
        root_pci_cfg,
        num_pci_devs,
        pci_devs,
    )
}
