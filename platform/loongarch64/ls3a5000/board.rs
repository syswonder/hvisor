use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "ls3a5000";

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x10000f000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x200000;
pub const ROOT_ZONE_ENTRY: u64 = 0x9000000000e71000;
pub const ROOT_ZONE_CPUS: u64 = 1 << 0;

pub const ROOT_ZONE_NAME: &str = "root-linux-la64";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 7] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x00200000,
        virtual_start: 0x00200000,
        size: 0x0ee00000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x90000000,
        virtual_start: 0x90000000,
        size: 0x10000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xf0000000,
        virtual_start: 0xf0000000,
        size: 0x10000000, // 0xf0000000 - 0xffffffff, 256M
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xc0000000,
        virtual_start: 0xc0000000,
        size: 0x30000000, // 0xc0000000 - 0xefffffff, 768M
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start: 0x1fe00000,
        size: 0x2000,
    }, // serial
    // map special region
    // 2024.4.12
    // linux's strscpy called gpa at 0x9000_0000_0000_0000 which is ldx x, 0x9000_0000_0000_0000(a1) + 0x0(a0) why ?
    // __memcpy_fromio 0xf0000 why?
    // (0x0, 0x10000, ZONE_MEM_FLAG_R | ZONE_MEM_FLAG_W | ZONE_MEM_FLAG_X)
    // (0xf0000, 0x10000, ZONE_MEM_FLAG_R | ZONE_MEM_FLAG_W | ZONE_MEM_FLAG_X)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1000,
        virtual_start: 0x0,
        size: 0x10000,
    }, // 0x0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xf0000,
        virtual_start: 0xf0000,
        size: 0x10000,
    }, // 0xf0000
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_RAM,
       //     physical_start: 0x10000,
       //     virtual_start: 0x10000,
       //     size: 0x10000,
       // }, // 0x10000
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_RAM,
       //     physical_start: 0xf000000,
       //     virtual_start: 0xf000000,
       //     size: 0x1000,
       // },
];

pub const ROOT_ZONE_IRQS: [u32; 0] = [];
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };
