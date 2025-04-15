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
pub const ROOT_ZONE_ENTRY: u64 = 0x9000000000e71000;
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
        physical_start: 0xf0000000,
        virtual_start: 0xf0000000,
        size: 0x10000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xc0000000,
        virtual_start: 0xc0000000,
        size: 0x30000000,
    }, // ram
    /* devices and controllers */
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start: 0x1fe00000,
        size: 0x2000,
    }, // uart0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10080000,
        virtual_start: 0x10080000,
        size: 0x1000,
    }, // uart1
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x100d0000,
    //     virtual_start: 0x100d0000,
    //     size: 0x1000,
    // }, // rtc
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
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xefe_0000_0000,
        virtual_start: 0xfe_0000_0000,
        size: 0x20000000,
    }, // pci config space (HT)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x18408000,
        virtual_start: 0x18408000,
        size: 0x00008000,
    }, // pci io resource
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x60000000,
        virtual_start: 0x60000000,
        size: 0x20000000,
    }, // pci mem resource
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
];

pub const ROOT_ZONE_IRQS: [u32; 0] = [];
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };
