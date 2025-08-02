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
use alloc::vec::Vec;
use spin::Once;

use crate::{arch::zone::HvArchZoneConfig, platform};

pub const MEM_TYPE_RAM: u32 = 0;
pub const MEM_TYPE_IO: u32 = 1;
pub const MEM_TYPE_VIRTIO: u32 = 2;

pub const CONFIG_MAGIC_VERSION: usize = 0x2;
pub const CONFIG_MAX_MEMORY_REGIONS: usize = 64;
pub const CONFIG_MAX_INTERRUPTS: usize = 32;
pub const CONFIG_NAME_MAXLEN: usize = 32;
pub const CONFIG_MAX_IVC_CONGIGS: usize = 2;
pub const CONFIG_MAX_PCI_DEV: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy)]

pub struct HvConfigMemoryRegion {
    pub mem_type: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub size: u64,
}

impl HvConfigMemoryRegion {
    pub fn new_empty() -> Self {
        Self {
            mem_type: 0,
            physical_start: 0,
            virtual_start: 0,
            size: 0,
        }
    }
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
        }
    }
}
// Every time you change the HvZoneConfig, you need to change the `CONFIG_MAGIC_VERSION`
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvZoneConfig {
    pub zone_id: u32,
    cpus: u64,
    num_memory_regions: u32,
    memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
    num_interrupts: u32,
    interrupts: [u32; CONFIG_MAX_INTERRUPTS],
    num_ivc_configs: u32,
    ivc_configs: [HvIvcConfig; CONFIG_MAX_IVC_CONGIGS],
    pub entry_point: u64,
    pub kernel_load_paddr: u64,
    pub kernel_size: u64,
    pub dtb_load_paddr: u64,
    pub dtb_size: u64,
    pub name: [u8; CONFIG_NAME_MAXLEN],
    pub arch_config: HvArchZoneConfig,
    pub pci_config: HvPciConfig,
    pub num_pci_devs: u64,
    pub alloc_pci_devs: [u64; CONFIG_MAX_PCI_DEV],
}

impl HvZoneConfig {
    pub fn new(
        zone_id: u32,
        cpus: u64,
        num_memory_regions: u32,
        memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
        num_interrupts: u32,
        interrupts: [u32; CONFIG_MAX_INTERRUPTS],
        num_ivc_configs: u32,
        ivc_configs: [HvIvcConfig; CONFIG_MAX_IVC_CONGIGS],
        entry_point: u64,
        kernel_load_paddr: u64,
        kernel_size: u64,
        dtb_load_paddr: u64,
        dtb_size: u64,
        name: [u8; CONFIG_NAME_MAXLEN],
        arch: HvArchZoneConfig,
        pci: HvPciConfig,
        num_pci_devs: u64,
        alloc_pci_devs: [u64; CONFIG_MAX_PCI_DEV],
    ) -> Self {
        Self {
            zone_id,
            cpus,
            num_memory_regions,
            memory_regions,
            num_interrupts,
            interrupts,
            num_ivc_configs,
            ivc_configs,
            entry_point,
            kernel_load_paddr,
            kernel_size,
            dtb_load_paddr,
            dtb_size,
            name,
            arch_config: arch,
            pci_config: pci,
            num_pci_devs: num_pci_devs,
            alloc_pci_devs: alloc_pci_devs,
        }
    }

    pub fn memory_regions(&self) -> &[HvConfigMemoryRegion] {
        // hvisor tool will check the length of memory regions, so we can uncheck here.
        &self.memory_regions[..self.num_memory_regions as usize]
    }

    pub fn interrupts(&self) -> &[u32] {
        &self.interrupts[..self.num_interrupts as usize]
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
}

pub static mut HV_ROOT_ZONE_CONFIG: Once<HvZoneConfig> = Once::new();

pub fn init() {
    unsafe { HV_ROOT_ZONE_CONFIG.call_once(|| platform::platform_root_zone_config()) };
}

pub fn root_zone_config() -> &'static HvZoneConfig {
    init();
    unsafe { HV_ROOT_ZONE_CONFIG.get().unwrap() }
}

pub const IVC_PROTOCOL_USER: u32 = 0x0;
pub const IVC_PROTOCOL_HVISOR: u32 = 0x1;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
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
