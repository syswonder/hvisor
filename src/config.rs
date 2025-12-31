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
use core::fmt::Debug;
use spin::Once;

use crate::{arch::zone::HvArchZoneConfig, pci::vpci_dev::VpciDevType, platform};

pub const MEM_TYPE_RAM: u32 = 0;
pub const MEM_TYPE_IO: u32 = 1;
pub const MEM_TYPE_VIRTIO: u32 = 2;

pub const CONFIG_MAGIC_VERSION: usize = 0x4;
pub const CONFIG_MAX_MEMORY_REGIONS: usize = 64;

pub type BitmapWord = u32;
pub const CONFIG_MAX_INTERRUPTS: usize = 1024;
pub const CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD: usize = 32;

pub const CONFIG_NAME_MAXLEN: usize = 32;
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
// Every time you change the HvZoneConfig, you need to change the `CONFIG_MAGIC_VERSION`
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvZoneConfig {
    pub zone_id: u32,
    cpus: u64,
    num_memory_regions: u32,
    memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
    interrupts_bitmap: [BitmapWord; CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD],
    num_ivc_configs: u32,
    ivc_configs: [HvIvcConfig; CONFIG_MAX_IVC_CONFIGS],
    pub entry_point: u64,
    pub kernel_load_paddr: u64,
    pub kernel_size: u64,
    pub dtb_load_paddr: u64,
    pub dtb_size: u64,
    pub name: [u8; CONFIG_NAME_MAXLEN],
    pub arch_config: HvArchZoneConfig,
    pub num_pci_bus: u64,
    pub pci_config: [HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
    pub num_pci_devs: u64,
    pub alloc_pci_devs: [HvPciDevConfig; CONFIG_MAX_PCI_DEV],
}

impl HvZoneConfig {
    pub fn new(
        zone_id: u32,
        cpus: u64,
        num_memory_regions: u32,
        memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
        interrupts_bitmap: [BitmapWord;
            CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD],
        num_ivc_configs: u32,
        ivc_configs: [HvIvcConfig; CONFIG_MAX_IVC_CONFIGS],
        entry_point: u64,
        kernel_load_paddr: u64,
        kernel_size: u64,
        dtb_load_paddr: u64,
        dtb_size: u64,
        name: [u8; CONFIG_NAME_MAXLEN],
        arch: HvArchZoneConfig,
        num_pci_bus: u64,
        pci: [HvPciConfig; CONFIG_PCI_BUS_MAXNUM],
        num_pci_devs: u64,
        alloc_pci_devs: [HvPciDevConfig; CONFIG_MAX_PCI_DEV],
    ) -> Self {
        Self {
            zone_id,
            cpus,
            num_memory_regions,
            memory_regions,
            interrupts_bitmap,
            num_ivc_configs,
            ivc_configs,
            entry_point,
            kernel_load_paddr,
            kernel_size,
            dtb_load_paddr,
            dtb_size,
            name,
            arch_config: arch,
            num_pci_bus,
            pci_config: pci,
            num_pci_devs: num_pci_devs,
            alloc_pci_devs: alloc_pci_devs,
        }
    }

    pub fn memory_regions(&self) -> &[HvConfigMemoryRegion] {
        // hvisor tool will check the length of memory regions, so we can uncheck here.
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

pub static mut HV_ROOT_ZONE_CONFIG: Once<HvZoneConfig> = Once::new();

pub fn init() {
    unsafe { HV_ROOT_ZONE_CONFIG.call_once(|| platform::platform_root_zone_config()) };
}

pub fn root_zone_config() -> &'static HvZoneConfig {
    init();
    unsafe { HV_ROOT_ZONE_CONFIG.get().unwrap() }
}

#[allow(unused)]
pub const IVC_PROTOCOL_USER: u32 = 0x0;

#[allow(unused)]
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

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HvPciDevConfig {
    pub domain: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub dev_type: VpciDevType,
}

#[macro_export]
macro_rules! pci_dev {
    ($domain:expr, $bus:expr, $dev:expr, $func:expr, $dev_type:expr) => {
        HvPciDevConfig {
            domain: $domain,
            bus: $bus,
            device: $dev,
            function: $func,
            dev_type: $dev_type,
        }
    };
}

impl Debug for HvPciDevConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bdf = crate::pci::pci_struct::Bdf::new(self.domain, self.bus, self.device, self.function);
        write!(f, "bdf {:#?}", bdf)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HvDwcAtuConfig {
    // ECAM (Enhanced Configuration Access Mechanism) base address
    // This is used to match with HvPciConfig::ecam_base
    pub ecam_base: u64,
    pub dbi_base: u64,
    pub dbi_size: u64,
    pub apb_base: u64,
    pub apb_size: u64,
    pub cfg_base: u64,
    pub cfg_size: u64,
    // set 1 if io base use atu0, when hvisor need set mmio for io
    // normally, when num-viewport less than 4, io_cfg_atu_shared is 1, otherwise is 0
    pub io_cfg_atu_shared: u64,
}

impl HvDwcAtuConfig {
    pub const fn new_empty() -> Self {
        // Use ATU_UNUSED for ATU indices that are not used by default
        // ATU_UNUSED is u32::MAX, cast to usize for consistency
        // Default ATU types: CFG0=4, CFG1=5, MEM=0, IO=2
        Self {
            ecam_base: 0,
            dbi_base: 0,
            dbi_size: 0,
            apb_base: 0,
            apb_size: 0,
            cfg_base: 0,
            cfg_size: 0,
            io_cfg_atu_shared: 0,
        }
    }
}

impl Default for HvDwcAtuConfig {
    fn default() -> Self {
        Self::new_empty()
    }
}
pub const fn get_irqs_bitmap<const N: usize>(
    numbers: &[u32; N],
) -> [BitmapWord; CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD] {
    assert!(
        CONFIG_MAX_INTERRUPTS % CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD == 0,
        "Configuration error: CONFIG_MAX_INTERRUPTS must be a multiple of 32 for a [u32] bitmap without rounding.",
    );

    let mut bitmap = [0; CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD];

    let mut i = 0;
    while i < N {
        let num = numbers[i];

        assert!(
            (num as usize) < CONFIG_MAX_INTERRUPTS,
            "Input IRQ number is out of bounds. It must be less than CONFIG_MAX_INTERRUPTS.",
        );

        let word_index = num as usize / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD;
        let bit_index = num as usize & (CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD - 1);

        bitmap[word_index] |= (1 as BitmapWord) << bit_index;
        i += 1;
    }

    bitmap
}
