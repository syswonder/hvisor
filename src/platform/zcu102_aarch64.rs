use crate::{arch::zone::HvArchZoneConfig, config::*};
use crate::config::HvConfigMemoryRegion;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x04000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x00200000;
pub const ROOT_ZONE_ENTRY: u64 = 0x00200000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 4] = [
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x800000000,
    //     virtual_start: 0x800000000,
    //     size: 0x80000000,
    // }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x00000000,
        virtual_start: 0x00000000,
        size: 0x40000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xfd070000,
        virtual_start: 0xfd070000,
        size: 0x30000,
    }, // memory-controller
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

pub const ROOT_ZONE_IRQS: [u32; 7] = [53, 81, 67, 175, 176, 177, 178];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0xf9010000,
    gicd_size: 0x10000,
    gicr_base: 0x80a0000,
    gicr_size: 0xf60000,
    gits_base: 0x20000,
    gits_size: 0x20000,
    gicc_base: 0xf9020000,
    gicc_size: 0x20000,
    gicc_offset: 0xf000,
    gich_base: 0xf9040000,
    gich_size: 0x20000,
    gicv_base: 0xf9060000,
    gicv_size: 0x20000,
};


pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];