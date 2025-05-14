use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "ok6254";

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x88000000;  // 设备树地址
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x82000000;  // 内核加载地址
pub const ROOT_ZONE_ENTRY: u64 = 0x82000000;  // 内核入口地址
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) ; // 2个cpu


pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 10] = [
    // RAM区域：覆盖DTS中memory节点定义的物理内存，排除保留区域
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x80000000,   // 起始地址
        virtual_start: 0x80000000,    // 直接映射
        size: 0x80000000,             // memory@0x80000000
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x0,  
        virtual_start: 0x0,
        size: 0x1800000,             // serial
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x2000000,  
        virtual_start: 0x2000000,    
        size: 0x5000000,      
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfc40000,  
        virtual_start: 0xfc40000,    
        size: 0x70000,      // spi
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x8000000,  
        virtual_start: 0x8000000,
        size: 0x200000,      // eth
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xf900000,  
        virtual_start: 0xf900000,    
        size: 0x30000,      // usb
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfa00000,  
        virtual_start: 0xfa00000,
        size: 0x100000,    // mmc
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe000000,  
        virtual_start: 0xe000000,
        size: 0x20000,    // watchdog e000000 e010000
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfd00000,  
        virtual_start: 0xfd00000,
        size: 0x20000,    // gpu 0xfd00000 0x20000
    },
    
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,  
        virtual_start: 0x10000000,
        size: 0x70000000,    // 0x0 ~ 0x80000000
    },
];

pub const ROOT_ZONE_IRQS: [u32; 17] = [
    0,1,27,66,114,115,165,194,210,228,258,266,195,118,
    211,5,
    0x20+0x20, // hvisor

    // 15+0x20,19+0x20,22+0x20,23+0x20,24+0x20,30+0x20,0x22,0x22+0x20,64+0x20,66+0x20,114+0x20,115+0x20,116+0x20,118+0x20,139+0x20,165+0x20,171+0x20,194+0x20,195+0x20,196+0x20,197+0x20,204+0x20,205+0x20,206+0x20,208+0x20,209+0x20,210+0x20,228+0x20,266
    // ,252,253,277,278,280,281,282,370,371,372,373,374,375
    // ,133,82,83
];

// int irq_list[12] = {0,1,27,66,114,115,165,194,210,228,258,266};

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { 
    gicd_base: 0x1800000,  
    gicd_size: 0x10000,    
    gicr_base: 0x1880000,  
    gicr_size: 0xc0000,    
    gicc_base: 0x0,  
    gicc_size: 0x0,    
    gicc_offset: 0x0,     
    gich_base: 0x0, 
    gich_size: 0x0, 
    gicv_base: 0x0, 
    gicv_size: 0x00000, 
    gits_base: 0x1820000, 
    gits_size: 0x10000, 
};


pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];
