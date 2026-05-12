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

use crate::pci_dev;
use crate::{
    arch::{
        mmu::MemoryType,
        zone::{GicConfig, Gicv3Config, HvArchZoneConfig},
    },
    config::*,
    pci::vpci_dev::VpciDevType,
};

pub const BOARD_NAME: &str = "sysoul-x3300";

pub const BOARD_NCPUS: usize = 8;
pub const BOARD_UART_BASE: u64 = 0xfeb5_0000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x000,   // cpu0
    0x100,   // cpu1
    0x200,   // cpu2
    0x300,   // cpu3
    0x400,   // cpu4
    0x500,   // cpu5
    0x600,   // cpu6
    0x700,   // cpu7
];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x20_0000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (        start,           end,               type)
    (  0x0000_0000,   0x0020_0000, MemoryType::Device),     // Includes low-address SRAM, marked as Device
    (  0x0020_0000,   0x0840_0000, MemoryType::Normal),
    (  0x0940_0000,   0xf000_0000, MemoryType::Normal),
    (  0xf000_0000, 0x1_0000_0000, MemoryType::Device),     // Dense device region, marked as Device.
    (0x1_0000_0000, 0x3_fc00_0000, MemoryType::Normal),
 // (0x3_fc50_0000, 0x3_fff0_0000, MemoryType::Normal),
    (0x3_fc40_0000, 0x4_0000_0000, MemoryType::Normal),     // aligned to 2 MiB
    (0x4_f000_0000, 0x5_0000_0000, MemoryType::Normal),
];

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x1000_0000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x1040_0000;
pub const ROOT_ZONE_ENTRY: u64 = 0x1040_0000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 2) - 1;

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    // /proc/iomem System RAM
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0020_0000,
        virtual_start: 0x0020_0000,
        size: 0x0820_0000,
    },
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x0940_0000,
    //     virtual_start: 0x0940_0000,
    //     size: 0xe6c0_0000,
    // },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0940_0000,
        virtual_start: 0x0940_0000,
        size: 0x06c0_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1000_0000,
        virtual_start: 0x1000_0000,
        size: 0x1000_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2000_0000,
        virtual_start: 0x2000_0000,
        size: 0x8000_0000,
    }, // this is a gap allocated for android modules
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xa000_0000,
        virtual_start: 0xa000_0000,
        size: 0x5000_0000,
    },
    // memory allocated to android
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1_0000_0000,
        virtual_start: 0x1_0000_0000,
        size: 0x2_fc00_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x3_fc50_0000,
        virtual_start: 0x3_fc50_0000,
        size: 0x03a0_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x4_f000_0000,
        virtual_start: 0x4_f000_0000,
        size: 0x1000_0000,
    },
    // Ramoops
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0011_0000,
        virtual_start: 0x0011_0000,
        size: 0x000f_0000,
    },
    // /proc/iomem Devices I/O
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfb00_0000,
        virtual_start: 0xfb00_0000,
        size: 0x0020_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfc00_0000,
        virtual_start: 0xfc00_0000,
        size: 0x0200_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe00_0000,
        virtual_start: 0xfe00_0000,
        size: 0x0060_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfea0_0000,
        virtual_start: 0xfea0_0000,
        size: 0x0050_0000,
    },
    // SRAM and Other Devices
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x0010_f000,
        virtual_start: 0x0010_f000,
        // size: 0x0100, // 10f000.sram
        size: 0x1000, // aligned with page size
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff00_1000,
        virtual_start: 0xff00_1000,
        size: 0x000e_e000, //ff001000.sram
    },
    // Unknown Region, maybe we should ask vendor for help
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x0010_0000,
        virtual_start: 0x0010_0000,
        size: 0xf000,
    },
];

pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    0x27, // arm_pmu
    0x29, // gic
    0x2a, 0x2b, 0x2d, 0x2e, // timer
    0x40, // hvisor
    0x69, // dmc
    0xeb, // sdmmc
    0x103, 0x102, // gmac0
    0x10a, 0x109, // gmac1
    0x135, // gpio0
    0x136, // gpio1
    0x137, // gpio2
    0x138, // gpio3
    0x139, // gpio4
    0x141, // rktimer
    0x15d, // i2c0
    0x15e, // i2c1
    0x166, // spi0
    0x168, // spi2
    0x16e, // uart3
    0x1ad, // tsadc
    0x1b0, // rng
]);

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0xfe60_0000,
        gicd_size: 0x0001_0000,
        gicr_base: 0xfe68_0000,
        gicr_size: 0x0010_0000,
        gits_base: 0x0,
        gits_size: 0x0,
    }),
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_PCI_DEVS: [HvPciDevConfig; 0] = [];
