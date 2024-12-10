use crate::{arch::zone::HvArchZoneConfig, config::*};
use crate::config::HvConfigMemoryRegion;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xa0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0xa0400000;
pub const ROOT_ZONE_ENTRY: u64 = 0xa0400000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 3] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x7ff00000,
        virtual_start: 0x7ff00000,
        size: 0x80000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff000000,
        virtual_start: 0xff000000,
        size: 0x1000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff170000,
        virtual_start: 0xff170000,
        size: 0x1000,
    }, // mmc0
];

// 35 36 37 38 -> pcie intx#
pub const ROOT_ZONE_IRQS: [u32; 3] = [53, 81, 67];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0xf9010000,
    gicd_size: 0x10000,
    gicr_base: 0x80a0000,
    gicr_size: 0xf60000,
    gicc_base: 0xf9020000,
    gicc_size: 0x20000,
    gich_base: 0xf9040000,
    gich_size: 0x20000,
    gicv_base: 0xf9060000,
    gicv_size: 0x20000,
};
