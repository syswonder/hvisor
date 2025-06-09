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
//      Yulong Han <wheatfox17@icloud.com>
//
use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "ls3a5000";

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x10000f000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x200000;
pub const ROOT_ZONE_ENTRY: u64 = 0x9000000000da4000;
pub const ROOT_ZONE_CPUS: u64 = 1 << 0;

pub const ROOT_ZONE_NAME: &str = "root-linux-la64";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    /* memory regions */
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
        physical_start: 0xf000_0000,
        virtual_start: 0xf000_0000,
        size: 0x1000_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1_6000_0000,
        virtual_start: 0x1_6000_0000,
        size: 0x1000_0000,
    }, // linux0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xc000_0000,
        virtual_start: 0xc000_0000,
        size: 0x3000_0000,
    }, // linux1
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xa000_0000,
        virtual_start: 0xa000_0000,
        size: 0x2000_0000,
    }, // linux2
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1_0000_0000,
        virtual_start: 0x1_0000_0000,
        size: 0x2000_0000,
    }, // linux3
    /* devices and controllers */
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start: 0x1fe00000,
        size: 0x1000,
    }, // uart0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10080000,
        virtual_start: 0x10080000,
        size: 0x1000,
    }, // uart1, passthrough now
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100d0000,
        virtual_start: 0x100d0000,
        size: 0x1000,
    }, // rtc, passthrough now
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start: 0x10000000,
        size: 0x1000,
    }, // pch-pic irq controller
    /* PCI related stuffs ... */
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x1a000000,
    //     virtual_start: 0x1a000000,
    //     size: 0x02000000,
    // }, // pci
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xefe_0000_0000,
    //     virtual_start: 0xfe_0000_0000,
    //     size: 0x20000000,
    // }, // pci config space (HT)
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x18408000,
    //     virtual_start: 0x18408000,
    //     size: 0x00008000,
    // }, // pci io resource
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x60000000,
    //     virtual_start: 0x60000000,
    //     size: 0x20000000,
    // }, // pci mem resource
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1001_0000,
        virtual_start: 0x1001_0000,
        size: 0x0001_0000,
    }, // ?
    /* map special regions - 2024.4.12 */
    // linux's strscpy called gpa at 0x9000_0000_0000_0000 which is ldx x, 0x9000_0000_0000_0000(a1) + 0x0(a0) why ?
    // __memcpy_fromio 0xf0000 why?
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
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1_4000_0000,
        virtual_start: 0x1_4000_0000,
        size: 0x1000_0000, // 256M shared memory across all zones
    }, // SHARD_MEM
];

pub const ROOT_ZONE_IRQS: [u32; 0] = [];
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };

pub const ROOT_PCI_CONFIG: HvPciConfig = HvPciConfig {
    ecam_base: 0xfe00000000,
    ecam_size: 0x20000000,
    io_base: 0x18408000,
    io_size: 0x8000,
    pci_io_base: 0x00008000,
    mem32_base: 0x0,
    mem32_size: 0x0,
    pci_mem32_base: 0x0,
    mem64_base: 0x60000000,
    mem64_size: 0x20000000,
    pci_mem64_base: 0x60000000,
};

/* 00:00.0, 00:00.1, 00:00.2, 00:00.3, 00:04.0, 00:04.1*/
/* 00:05.0, 00:05.1, 00:06.0, 00:06.1, 00:06.2 */
/* 00:07.0, 00:08.0, 00:09.0, 00:0a.0, 00:0b.0 */
/* 00:0c.0, 00:0d.0, 00:0f.0, 00:10.0, 00:13.0 */
/* 00:16.0, 00:19.0, 02:00.0, 05:00.0 */
/* BUS 8 on X16 slot */
/* 08:00.0, 08:00.1, 08:00.2, 08:00.3 net */
/* BUS 6 on X4 slot */
/* 06:00.0, 06:00.1, 06:00.2, 06:00.3 net */
pub const ROOT_PCI_DEVS: [u64; 26] = [
    0,
    1,
    2,
    3,
    4 << 3,
    (4 << 3) + 1,
    5 << 3,
    (5 << 3) + 1,
    // 00:06.xx is VGA and Graphics card
    (6 << 3),
    (6 << 3) + 1,
    (6 << 3) + 2,
    7 << 3,
    8 << 3, // bus 0 device 8: AHCI
    9 << 3,
    0xa << 3,
    0xb << 3,
    0xc << 3,
    0xd << 3,
    0xf << 3,
    0x10 << 3,
    0x13 << 3,
    0x16 << 3,
    0x19 << 3,
    2 << 8,
    5 << 8,
    // bus 6 (x4 slot) is PCIe network card
    // (8 << 8), // bus 8 net
    (6 << 8), // bus 6 net
];

// bus << 8 | dev << 5 | func << 3

// pub const ROOT_PCI_DEVS: [u64; 0] = [];

// "alloc_pci_devs": [0,1,2,3,32,33,40,41,56,64,72,80,88,96,104,120,128,152,176,200,512,1280,2051]
