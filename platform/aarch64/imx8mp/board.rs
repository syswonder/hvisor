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
use crate::{
    arch::{mmu::MemoryType, zone::HvArchZoneConfig},
    config::*,
};

pub const BOARD_NAME: &str = "imx8mp";

pub const BOARD_NCPUS: usize = 4;
pub const BOARD_UART_BASE: u64 = 0x30890000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x0,   // cpu0
    0x1,   // cpu1
    0x2,   // cpu2
    0x3,   // cpu3
];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x200000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (       start,           end,                type)
    (         0x0,    0x40000000,  MemoryType::Device),
    (  0x40000000,   0x100000000,  MemoryType::Normal),
];

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
    35, 36, 37, 38, 45, 52, 55, 56, 57, 59, 64, 67, 75, 96, 97, 98, 99, 100, 101, 102, 103, 104,
    105, 135, 150, 151, 152, 162,
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
