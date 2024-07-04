use alloc::vec::Vec;
use spin::Once;

use crate::{arch::zone::HvArchZoneConfig, platform};

pub const MEM_TYPE_RAM: u32 = 0;
pub const MEM_TYPE_IO: u32 = 1;
pub const CONFIG_MAX_MEMORY_REGIONS: usize = 16;
pub const CONFIG_MAX_INTERRUPTS: usize = 32;

// pub const CONFIG_KERNEL_ARGS_MAXLEN: usize = 256;

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
#[derive(Debug, Clone)]
pub struct HvZoneConfig {
    pub zone_id: u32,
    cpus: u64,
    num_memory_regions: u32,
    memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
    num_interrupts: u32,
    interrupts: [u32; CONFIG_MAX_INTERRUPTS],
    pub entry_point: u64,
    pub dtb_load_paddr: u64,

    pub arch: HvArchZoneConfig,
}

impl HvZoneConfig {
    pub fn new(
        zone_id: u32,
        cpus: u64,
        num_memory_regions: u32,
        memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
        num_interrupts: u32,
        interrupts: [u32; CONFIG_MAX_INTERRUPTS],
        entry_point: u64,
        dtb_load_paddr: u64,
        arch: HvArchZoneConfig,
    ) -> Self {
        Self {
            zone_id,
            cpus,
            num_memory_regions,
            memory_regions,
            num_interrupts,
            interrupts,
            entry_point,
            dtb_load_paddr,
            arch,
        }
    }

    pub fn memory_regions(&self) -> &[HvConfigMemoryRegion] {
        if self.num_memory_regions > CONFIG_MAX_MEMORY_REGIONS as u32 {
            panic!("Too many memory regions");
        }
        &self.memory_regions[..self.num_memory_regions as usize]
    }

    pub fn interrupts(&self) -> &[u32] {
        if self.num_interrupts > CONFIG_MAX_INTERRUPTS as u32 {
            panic!("Too many interrupts");
        }
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
}

pub static mut HV_ROOT_ZONE_CONFIG: Once<HvZoneConfig> = Once::new();

pub fn init() {
    unsafe { HV_ROOT_ZONE_CONFIG.call_once(|| platform::platform_root_zone_config()) };
}

pub fn root_zone_config() -> &'static HvZoneConfig {
    init();
    unsafe { HV_ROOT_ZONE_CONFIG.get().unwrap() }
}
