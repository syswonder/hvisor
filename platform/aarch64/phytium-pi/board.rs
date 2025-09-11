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

pub const BOARD_NAME: &str = "phytium-pi";

pub const BOARD_NCPUS: usize = 4;
pub const BOARD_UART_BASE: u64 = 0x2800d000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x200,     // cpu0
    0x201,    // cpu1
    0x00,    // cpu2
    0x100,  // cpu3
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
pub const ROOT_ZONE_CPUS: u64 = (1 << 1) | (1 << 0);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 11] = [
    // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x80000000,
        virtual_start: 0x80000000,
        size: 0x80000000,
    },
    // soc@0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x28000000,
        virtual_start: 0x28000000,
        size: 0x00100000,
    },
    //iommu
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30000000,
        virtual_start: 0x30000000,
        size: 0x800000,
    },
    // ethernet0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x3200c000,
        virtual_start: 0x3200c000,
        size: 0x00002000, // 8KB
    },
    // USB
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x31800000, // usb2@31800000
        virtual_start: 0x31800000,
        size: 0x00080000, // 512KB
    },
    // USB2 @32800000 - Host Mode
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32800000,
        virtual_start: 0x32800000,
        size: 0x00040000, // 256KB
    },
    // USB2 @32840000 - Host Mode
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32840000,
        virtual_start: 0x32840000,
        size: 0x00040000, // 256KB
    },
    // USB3 @31a08000 - XHCI Controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x31a08000,
        virtual_start: 0x31a08000,
        size: 0x00018000, // 96KB
    },
    // USB3 @31a28000 - XHCI Controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x31a28000,
        virtual_start: 0x31a28000,
        size: 0x00018000, // 96KB
    },
    //mailbox
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32a00000,
        virtual_start: 0x32a00000,
        size: 0x1000,
    },
    //sram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32a10000,
        virtual_start: 0x32a10000,
        size: 0x2000,
    },
];
//46-usb2,54-mailbox 64-usb2,87-net,104、105-mmc,116-uart,133、138-i2c,191-spi
pub const ROOT_ZONE_IRQS: [u32; 14] =
    [46, 54, 64, 65, 75, 76, 78, 87, 104, 105, 116, 133, 138, 191];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0x30800000,
        gicd_size: 0x20000,
        gicr_base: 0x30880000,
        gicr_size: 0x80000,
        gits_base: 0,
        gits_size: 0,
    }),
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];
