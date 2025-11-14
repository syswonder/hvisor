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
//  ForeverYolo <2572131118@qq.com>
use crate::config::HvConfigMemoryRegion;
use crate::{
    arch::{
        mmu::MemoryType,
        zone::{GicConfig, Gicv2Config, HvArchZoneConfig},
    },
    config::*,
};

pub const BOARD_NAME: &str = "zcu102";

pub const BOARD_NCPUS: usize = 4;
pub const BOARD_UART_BASE: u64 = 0xff000000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x0,    // cpu0
    0x1,    // cpu1
    0x2,    // cpu2
    0x3,    // cpu3
];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x200000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (       start,           end,                type)
    (         0x0,    0xf0000000,  MemoryType::Normal),
    (  0xf0000000,   0x100000000,  MemoryType::Device),
];

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x04000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x00200000;
pub const ROOT_ZONE_ENTRY: u64 = 0x00200000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x800000000,
    //     virtual_start: 0x800000000,
    //     size: 0x80000000,
    // }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x00000000,
        virtual_start: 0x00000000,
        size: 0x40000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x50000000,
        virtual_start: 0x50000000,
        size: 0x25000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xfd070000,
        virtual_start: 0xfd070000,
        size: 0x30000,
    }, // memory-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff000000,
        virtual_start: 0xff000000,
        size: 0x1000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff170000,
        virtual_start: 0xff170000,
        size: 0x1000,
    }, // mmc0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff0e0000,
        virtual_start: 0xff0e0000,
        size: 0x1000,
    }, // ethernet
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff020000,
        virtual_start: 0xff020000,
        size: 0x1000,
    }, // i2c
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff030000,
        virtual_start: 0xff030000,
        size: 0x1000,
    }, // i2c
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff0a0000,
        virtual_start: 0xff0a0000,
        size: 0x1000,
    }, // gpio
];

pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] =
    &get_irqs_bitmap(&[53, 81, 175, 176, 177, 178, 64, 50, 48, 49, 95]);
// serial-mmc-pmu-pmu-pmu-pmu-(hvisor_virtio_device)-gpio-i2c(ff030000)-i2c(ff020000)-ethernet

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv2(Gicv2Config {
        gicd_base: 0xf9010000,
        gicd_size: 0x10000,
        gicc_base: 0xf9020000,
        gicc_size: 0x20000,
        gicc_offset: 0xf000,
        gich_base: 0xf9040000,
        gich_size: 0x20000,
        gicv_base: 0xf9060000,
        gicv_size: 0x20000,
    }),
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];
