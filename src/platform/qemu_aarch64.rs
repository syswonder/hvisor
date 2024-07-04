use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const ROOT_ZONE_DTB_ADDR: usize = 0xa0000000;
pub const ROOT_ZONE_ENTRY: usize = 0xa0400000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 3] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x50000000,
        virtual_start: 0x50000000,
        size: 0x80000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x9000000,
        virtual_start: 0x9000000,
        size: 0x1000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xa000000,
        virtual_start: 0xa000000,
        size: 0x4000,
    }, // virtio
];

pub const ROOT_ZONE_IRQS: [u32; 4] = [33, 64, 77, 79];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0x8000000,
    gicd_size: 0x10000,
    gicr_base: 0x80a0000,
    gicr_size: 0xf60000,
};
