use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x10000f000;
pub const ROOT_ENTRY: u64 = 0x9000000000cb5000; // vmlinux's entry addr

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 6] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x00200000,
        virtual_start: 0x00200000,
        size: 0x0ee00000,
    }, // ram1
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x90000000,
        virtual_start: 0x90000000,
        size: 0x10000000,
    }, // ram2
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start: 0x1fe00000,
        size: 0x1000,
    }, // serial

    // map special region
    // 2024.4.12
    // linux's strscpy called gpa at 0x9000_0000_0000_0000 which is ldx x, 0x9000_0000_0000_0000(a1) + 0x0(a0) why ?
    // __memcpy_fromio 0xf0000 why?
    // (0x0, 0x10000, ZONE_MEM_FLAG_R | ZONE_MEM_FLAG_W | ZONE_MEM_FLAG_X)
    // (0xf0000, 0x10000, ZONE_MEM_FLAG_R | ZONE_MEM_FLAG_W | ZONE_MEM_FLAG_X)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0,
        virtual_start: 0x0,
        size: 0x10000,
    }, // 0x0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xf0000,
        virtual_start: 0xf0000,
        size: 0x10000,
    }, // 0xf0000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x10000,
        virtual_start: 0x10000,
        size: 0x10000,
    }, // 0x10000
];

pub const ROOT_ZONE_IRQS: [u32; 32] = [0; 32];
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };

pub fn platform_root_zone_config() -> HvZoneConfig {
    // pad mem region to MAX_MEMORY_REGIONS
    let mut memory_regions: [HvConfigMemoryRegion; CONFIG_MAX_MEMORY_REGIONS] =
        [HvConfigMemoryRegion::new_empty(); CONFIG_MAX_MEMORY_REGIONS];
    for i in 0..ROOT_ZONE_MEMORY_REGIONS.len() {
        memory_regions[i] = ROOT_ZONE_MEMORY_REGIONS[i];
    }
    let root_zone_config = HvZoneConfig::new(
        0,
        1,
        3,
        memory_regions,
        4,
        ROOT_ZONE_IRQS,
        ROOT_ENTRY as _,
        ROOT_ZONE_MEMORY_REGIONS[0].physical_start,
        ROOT_ZONE_MEMORY_REGIONS[0].size,
        ROOT_ZONE_DTB_ADDR,
        0,
        ROOT_ARCH_ZONE_CONFIG,
    );
    root_zone_config
}
