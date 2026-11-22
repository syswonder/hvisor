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

//! Zone configuration ABI.
//!
//! The layout of these structures is part of the interface between hvisor,
//! the hvisor kernel module, and hvisor-tool. Do not reorder fields without
//! bumping CONFIG_MAGIC_VERSION.

use alloc::vec::Vec;

use super::pci::HvPciDevConfig;

pub const MEM_TYPE_RAM: u32 = 0;
pub const MEM_TYPE_IO: u32 = 1;
pub const MEM_TYPE_VIRTIO: u32 = 2;

pub const CONFIG_MAGIC_VERSION: usize = 0x7;
pub const CONFIG_MAX_MEMORY_REGIONS: usize = 64;

pub type BitmapWord = u32;
pub const CONFIG_MAX_INTERRUPTS: usize = 1024;
pub const CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD: usize = 32;

pub const CONFIG_NAME_MAXLEN: usize = 32;
pub const CONFIG_MAX_ZONES: usize = 32;
pub const CONFIG_MAX_IVC_CONFIGS: usize = 2;
pub const CONFIG_PCI_BUS_MAXNUM: usize = 4;
pub const CONFIG_MAX_PCI_DEV: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HvConfigMemoryRegion {
    pub mem_type: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub size: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HvPciConfig {
    pub ecam_base: u64,
    pub ecam_size: u64,
    pub io_base: u64,
    pub io_size: u64,
    pub pci_io_base: u64,
    pub mem32_base: u64,
    pub mem32_size: u64,
    pub pci_mem32_base: u64,
    pub mem64_base: u64,
    pub mem64_size: u64,
    pub pci_mem64_base: u64,
    pub bus_range_begin: u32,
    pub bus_range_end: u32,
    pub domain: u8,
}

impl HvPciConfig {
    pub const fn new_empty() -> Self {
        Self {
            ecam_base: 0,
            ecam_size: 0,
            io_base: 0,
            io_size: 0,
            pci_io_base: 0,
            mem32_base: 0,
            mem32_size: 0,
            pci_mem32_base: 0,
            mem64_base: 0,
            mem64_size: 0,
            pci_mem64_base: 0,
            bus_range_begin: 0,
            bus_range_end: 0,
            domain: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HvIvcConfig {
    pub ivc_id: u32,
    pub peer_id: u32,
    pub control_table_ipa: u64,
    pub shared_mem_ipa: u64,
    pub rw_sec_size: u32,
    pub out_sec_size: u32,
    pub interrupt_num: u32,
    pub max_peers: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HvZoneBootMode {
    pub zone_id: u32,
    pub multiboot_enabled: u32,
    pub multiboot_info_paddr: u64,
}

/// Architecture-independent part of the zone configuration ABI.
///
/// Architecture-specific data lives in the concrete ZoneConfig wrapper, not in
/// this structure.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvZoneConfigCommon {
    pub zone_id: u32,
    pub cpus: u64,
    pub num_memory_regions: u32,
    pub memory_regions: [HvConfigMemoryRegion; 64],
    pub interrupts_bitmap: [BitmapWord; 32],
    pub num_ivc_configs: u32,
    pub ivc_configs: [HvIvcConfig; 2],
    pub entry_point: u64,
    pub kernel_load_paddr: u64,
    pub kernel_size: u64,
    pub dtb_load_paddr: u64,
    pub dtb_size: u64,
    pub name: [u8; 32],
    pub num_pci_bus: u64,
    pub pci_config: [HvPciConfig; 4],
    pub num_pci_devs: u64,
    pub alloc_pci_devs: [HvPciDevConfig; 32],
}

impl HvZoneConfigCommon {
    pub fn memory_regions(&self) -> &[HvConfigMemoryRegion] {
        &self.memory_regions[..self.num_memory_regions as usize]
    }

    pub fn interrupts_bitmap(&self) -> &[BitmapWord] {
        &self.interrupts_bitmap
    }

    pub fn cpus(&self) -> Vec<u64> {
        let mut v = Vec::new();
        for i in 0..64u64 {
            if (self.cpus >> i) & 1 == 1 {
                v.push(i);
            }
        }
        v
    }

    pub fn ivc_config(&self) -> &[HvIvcConfig] {
        &self.ivc_configs[..self.num_ivc_configs as usize]
    }

    #[allow(unused)]
    pub fn pci_config(&self) -> &[HvPciConfig] {
        &self.pci_config[..self.num_pci_bus as usize]
    }
}
