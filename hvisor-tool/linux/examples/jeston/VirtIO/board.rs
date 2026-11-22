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
    arch::{mmu::MemoryType, zone::{HvArchZoneConfig, GicConfig, Gicv3Config, Uefi, UefiConfig}},
    config::*,
};

pub const BOARD_NAME: &str = "jeston-orin";
pub const BOARD_NCPUS: usize = 6;
pub const BOARD_UART_BASE: u64 = 0xc280000;
pub static BOARD_EARLY_CACHE_INVALIDATE_MASKS: [u64; BOARD_NCPUS] = [0b111; BOARD_NCPUS];
#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x0,   // cpu0
    0x100, // cpu1
    0x200, // cpu2
    0x300, // cpu3
    0x10200, // cpu4
    0x10300, // cpu5
];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x200000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (       start,           end,                type)
    (  0x00000000,   0x080000000,  MemoryType::Device),
    (  0x80000000,   0x280000000,  MemoryType::Normal),
    (0x2000000000,  0x2500000000,  MemoryType::Device),
    (0x3000000000,  0x4000000000,  MemoryType::Device)
];

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xA0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0xA0400000;
pub const ROOT_ZONE_ENTRY: u64 = 0xA0400000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x90000000,
        virtual_start: 0x90000000,
        size: 0x2E000000, // 1G
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xC2000000,
        virtual_start: 0xC2000000,
        size: 0x3E000000, // 1G
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x100000000,
        virtual_start: 0x100000000,
        size: 0x100000000, // 1G
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x260000000,
        virtual_start: 0x260000000,
        size: 0x10000000,
    }, // ram - UEFI
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x00000000,
        virtual_start: 0x00000000,
        size: 0x0f400000,
    }, // general
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start: 0x10000000,
        size: 0x70000000,
    }, // general
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x2000000000,
        virtual_start: 0x2000000000,
        size: 0x500000000,
    }, // pcie
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x3000000000,
        virtual_start: 0x3000000000,
        size: 0x1000000000,
    }, // pcie
];


pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;

// serial-c - phy - pcie - phy - hsp - timer - hsp - phy - phy - memory-controller - external-memory-controller
// pcie - phy - phy
// bpmp-fabric
// gpu
// host1x
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    146,
    372,
    83, 84,
    373,
    165, 166, 167, 168,
    208, 152, 153, 154, 155, 156, 157, 158, 159,
    374,
    375,
    255,
    256,
    64,
    388, 389,
    253,
    254,
    206,
    99, 100, 102, 103,
    480, 481, 482, 483, 484, 485, 486, 487]);


pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0xf400000,
        gicd_size: 0x10000,
        gicr_base: 0xf440000,
        gicr_size: 0x200000,
        gits_base: 0x0,
        gits_size: 0x0,
    }),
    uefi_config: UefiConfig::Uefi(Uefi {
        memory_map_addr: 0xB0000000,
        memory_map_size: 0x990,
        sys_map_addr: 0xB1000000,
    })
};


pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];
