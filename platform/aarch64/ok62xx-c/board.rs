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
//

use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "ok6254";
pub const BOARD_NCPUS: usize = 4;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x88000000; // DTB load address
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x82000000; // kernel load address
pub const ROOT_ZONE_ENTRY: u64 = 0x82000000; // kernel entry point
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1); // 2 cpus
pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 12] = [
    // Ram regions: defined here are based on the DTS (Device Tree Source) file
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x80000000,
        virtual_start: 0x80000000,
        size: 0x80000000,
    }, // memory@0x80000000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x0,
        virtual_start: 0x0,
        size: 0x1800000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x2800000,
        virtual_start: 0x2800000,
        size: 0x1000000,
    }, // serial@2800000-serial@2860000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x4000000,
        virtual_start: 0x4000000,
        size: 0x1000000,
    }, // pinctrl@4084000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfc40000,
        virtual_start: 0xfc40000,
        size: 0x70000,
    }, // spi
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x8000000,
        virtual_start: 0x8000000,
        size: 0x200000,
    }, // eth
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xf900000,
        virtual_start: 0xf900000,
        size: 0x30000,
    }, // usb
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfa00000,
        virtual_start: 0xfa00000,
        size: 0x100000,
    }, // mmc
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe000000,
        virtual_start: 0xe000000,
        size: 0x20000,
    }, // watchdog e000000 e010000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfd00000,
        virtual_start: 0xfd00000,
        size: 0x20000,
    }, // gpu 0xfd00000 0x20000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xa003000,
        virtual_start: 0xa003000,
        size: 0x1000,
    }, // gpu 0xfd00000 0x20000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start: 0x10000000,
        size: 0x70000000,
    }, // 0x10000000 ~ 0x80000000
];

pub const ROOT_ZONE_IRQS: [u32; 17] = [
    0, 1, 27, 66, 114, 115, 165, 194, 210, 228, 258, 266, 195, 118, 211, 5, 0x40, // hvisor
];

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
