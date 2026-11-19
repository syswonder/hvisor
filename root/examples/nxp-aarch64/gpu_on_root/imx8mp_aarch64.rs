use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xa0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0xa0400000;
pub const ROOT_ZONE_ENTRY: u64 = 0xa0400000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 8] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x50000000,
        virtual_start: 0x50000000,
        size: 0x80000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30000000,
        virtual_start: 0x30000000,
        size: 0x400000,
    }, // bus@30000000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30c00000,
        virtual_start: 0x30c00000,
        size: 0x400000,
    }, 
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30800000,
        virtual_start: 0x30800000,
        size: 0x400000,
    }, 
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x38000000,
        virtual_start: 0x38000000,
        size: 0x8000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x38008000,
        virtual_start: 0x38008000,
        size: 0x8000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x38500000,
        virtual_start: 0x38500000,
        size: 0x20000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32c00000,
        virtual_start: 0x32c00000,
        size: 0x400000,
    }, // hdmi
    // bus@30800000
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start: 0x30890000,
       //     virtual_start: 0x30890000,
       //     size: 0x1000,
       // }, // serial
];

pub const ROOT_ZONE_IRQS: [u32; 28] = [
    35, 36, 37, 38, 45, 52, 55, 56, 57, 59, 64, 67, 75, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 135, 150, 151, 152, 162
];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0x38800000,
    gicd_size: 0x10000,
    gicr_base: 0x38880000,
    gicr_size: 0xc0000,
    gicc_base: 0,
    gicc_size: 0,
    gicc_offset: 0x0,
    gich_base: 0,
    gich_size: 0,
    gicv_base: 0,
    gicv_size: 0,
    gits_base: 0,
    gits_size: 0,
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];
