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
    arch::{
        mmu::MemoryType,
        zone::{GicConfig, Gicv3Config, HvArchZoneConfig},
    },
    config::*,
};

// [   17.796762]   node   0: [mem 0x0000000000200000-0x000000000047ffff]
// [   17.797335]   node   0: [mem 0x0000000000480000-0x000000000087ffff]
// [   17.797907]   node   0: [mem 0x0000000000880000-0x00000000083fffff]
// [   17.798479]   node   0: [mem 0x0000000009400000-0x00000000efffffff]
// [   17.799053]   node   0: [mem 0x0000000100000000-0x00000003fbffffff]
// [   17.799628]   node   0: [mem 0x00000003fc500000-0x00000003ffefffff]
// [   17.800200]   node   0: [mem 0x00000004f0000000-0x00000004ffffffff]

pub const BOARD_NAME: &str = "rk3588";

pub const BOARD_NCPUS: usize = 8;
pub const BOARD_UART_BASE: u64 = 0xfeb50000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x0,     // cpu0
    0x100,   // cpu1
    0x200,   // cpu2
    0x300,   // cpu3
    0x400,   // cpu4
    0x500,   // cpu5
    0x600,   // cpu6
    0x700,   // cpu7
];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x200000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (       start,           end,                type)
    (         0x0,    0xf0000000,  MemoryType::Normal),
    (  0xf0000000,   0x100000000,  MemoryType::Device),
    ( 0x100000000,   0x3fc000000,  MemoryType::Normal)
];

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x10000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x09400000;
pub const ROOT_ZONE_ENTRY: u64 = 0x09400000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x100000,
        virtual_start: 0x100000,
        size: 0xf0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x200000,
        virtual_start: 0x200000,
        size: 0x8200000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x9400000,
        virtual_start: 0x9400000,
        size: 0xe6c00000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x100000000,
        virtual_start: 0x100000000,
        size: 0x2fc000000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x3fc500000,
        virtual_start: 0x3fc500000,
        size: 0x3a00000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x4f0000000,
        virtual_start: 0x4f0000000,
        size: 0x10000000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfc000000,
        virtual_start: 0xfc000000,
        size: 0x2000000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe800000,
        virtual_start: 0xfe800000,
        size: 0x800000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe000000,
        virtual_start: 0xfe000000,
        size: 0x600000,
    },
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x0000000000200000,
    //     virtual_start: 0x0000000000200000,
    //     size: 0x00000000083fffff - 0x0000000000200000 + 1,
    // }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x0000000009400000,
    //     virtual_start: 0x0000000009400000,
    //     size: 0x00000000efffffff - 0x0000000009400000 + 1,
    // }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x0000000100000000,
    //     virtual_start: 0x0000000100000000,
    //     size: 0x00000003fbffffff - 0x0000000100000000 + 1,
    // }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x00000003fc500000,
    //     virtual_start: 0x00000003fc500000,
    //     size: 0x00000003ffefffff - 0x00000003fc500000 + 1,
    // }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x00000004f0000000,
    //     virtual_start: 0x00000004f0000000,
    //     size: 0x00000004ffffffff - 0x00000004f0000000 + 1,
    // }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xfeb50000,
    //     virtual_start: 0xfeb50000,
    //     size: 0x100,
    // }, // uart
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xfd000000,
    //     virtual_start: 0xfd000000,
    //     size: 0x1000000,
    // },
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x100000,
    //     virtual_start: 0x100000,
    //     size: 0x100000,
    // }
];

// pub const ROOT_ZONE_IRQS_BITMAP: [u32; 10] = [39, 64, 235, 237, 309, 312, 360, 365, 429, 455];
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    39, 41, 42, 43, 45, 46, 64, 120, 121, 235, 237, 247, 248, 250, 251, 252, 265, 266, 309, 312,
    313, 355, 360, 365, 423, 424, 425, 429, 455,
]);

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0xfe600000,
        gicd_size: 0x10000,
        gicr_base: 0xfe680000,
        gicr_size: 0x100000,
        gits_base: 0x8080000,
        gits_size: 0x20000,
    }),
};

pub const ROOT_PCI_CONFIG: HvPciConfig = HvPciConfig {
    ecam_base: 0x4010000000,
    ecam_size: 0x10000000,
    io_base: 0x3eff0000,
    io_size: 0x10000,
    pci_io_base: 0x0,
    mem32_base: 0x10000000,
    mem32_size: 0x2eff0000,
    pci_mem32_base: 0x10000000,
    mem64_base: 0x8000000000,
    mem64_size: 0x8000000000,
    pci_mem64_base: 0x8000000000,
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_PCI_DEVS: [u64; 2] = [0, 1 << 3];
