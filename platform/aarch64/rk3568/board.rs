use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "rk3568";

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xa0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x60080000 ;
pub const ROOT_ZONE_ENTRY: u64 = 0x60080000 ;
//pub const ROOT_ZONE_CPUS: u64 = (1 << 0) ;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0)|(1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 16] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x200000,
        virtual_start: 0x200000,
        size: 0x8200000,
    }, // memory
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x9400000,
        virtual_start: 0x9400000,
        size: 0xe6c00000,
    }, // memory
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0,
        virtual_start: 0x0,
        size: 0x200000,
    }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x110000,
    //     virtual_start: 0x110000,
    //     size: 0xf0000,
    // }, // ramoops
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x10f000,
    //     virtual_start: 0x10f000,
    //     size: 0x1000,
    // }, //scmi-shmem
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1f0000000,
        virtual_start: 0x1f0000000,
        size: 0x10000000,
    }, // memory
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE660000,
        virtual_start: 0xFE660000,
        size: 0x10000,
    }, //serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDC20000,
        virtual_start: 0xFDC20000,
        size: 0x10000,
    }, //syscon
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDC60000,
        virtual_start: 0xFDC60000,
        size: 0x30000,
    }, //syscon
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDD20000,
        virtual_start: 0xFDD20000,
        size: 0x10000,
    }, //clock-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE530000,
        virtual_start: 0xFE530000,
        size: 0x10000,
    }, //dmac
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE5f0000,
        virtual_start: 0xFE5f0000,
        size: 0x10000,
    }, //timer
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDD00000,
        virtual_start: 0xFDD00000,
        size: 0x10000,
    }, //clock-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE750000,
        virtual_start: 0xFE750000,
        size: 0x10000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE740000,
        virtual_start: 0xFE740000,
        size: 0x10000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDD60000,
        virtual_start: 0xFDD60000,
        size: 0x10000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE760000,
        virtual_start: 0xFE760000,
        size: 0x20000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE310000,
        virtual_start: 0xFE310000,
        size: 0x10000,
    }, //sdhci
];

pub const ROOT_ZONE_IRQS: [u32; 18] = [
   0x46, 0x104, 0x105, 0x106, 0x107, 0x2d, 0x2e, 0x2b, 0x2a, 0x29, 0x33, 0x96, 0x11c, 0x44, 0x43, 0x42, 0x41, 0x8d];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0xfd400000,
    gicd_size: 0x10000,
    gicr_base: 0xfd460000,
    gicr_size: 0xc0000,
    gicc_base: 0,
    gicc_size: 0,
    gicc_offset: 0x0,
    gich_base: 0,
    gich_size: 0,
    gicv_base: 0,
    gicv_size: 0,
    gits_base: 0xfd440000,
    gits_size: 0x20000,
};
pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_PCI_DEVS: [u64; 0] = [];
