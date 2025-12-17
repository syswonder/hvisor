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
    pci::vpci_dev::VpciDevType,
};
use crate::pci_dev;

pub const BOARD_NAME: &str = "rk3568";

pub const BOARD_NCPUS: usize = 4;
pub const BOARD_UART_BASE: u64 = 0xfe660000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x0,     // cpu0
    0x100,   // cpu1
    0x200,   // cpu2
    0x300,   // cpu3
];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x200000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (       start,           end,                type)
    (         0x0,    0xf0000000,  MemoryType::Normal),
    (  0xf0000000,   0x100000000,  MemoryType::Device),
    ( 0x100000000,   0x3c0000000,  MemoryType::Normal),
    ( 0x3c0000000,   0x3d0000000,  MemoryType::Device)
];

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xa0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x00280000 ;
pub const ROOT_ZONE_ENTRY: u64 = 0x00280000 ;
//pub const ROOT_ZONE_CPUS: u64 = (1 << 0) ;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x3c0400000,
    //     virtual_start: 0x3c0400000,
    //     size: 0x400000,
    // }, //pcie
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xfe270000,
    //     virtual_start: 0xfe270000,
    //     size: 0x10000,
    // }, //pcie
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xf2000000,
    //     virtual_start: 0xf2000000,
    //     size: 0x100000,
    // }, //pcie
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xf2100000,
    //     virtual_start: 0xf2100000,
    //     size: 0x100000,
    // }, //pcie
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xf2200000,
    //     virtual_start: 0xf2200000,
    //     size: 0x1e00000,
    // }, //pcie
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x340000000,
        virtual_start: 0x340000000,
        size: 0x40000000,
    }, //pcie
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfdcb8000,
        virtual_start: 0xfdcb8000,
        size: 0x10000,
    }, //syscon pcie30_phy_grf
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfdc90000,
        virtual_start: 0xfdc90000,
        size: 0x10000,
    }, //syscon pipe_phy_grf2
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfdc50000,
        virtual_start: 0xfdc50000,
        size: 0x10000,
    }, //syscon pipegrf
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe8c0000,
        virtual_start: 0xfe8c0000,
        size: 0x20000,
    }, // pcie30phy
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfdd90000,
        virtual_start: 0xfdd90000,
        size: 0x1000,
    }, // power-management
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe840000,
        virtual_start: 0xfe840000,
        size: 0x1000,
    }, // combphy2_psq
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x200000,
        virtual_start: 0x200000,
        size: 0x8200000,
    }, // memory
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x9400000,
        virtual_start: 0x9400000,
        size: 0xe6c00000,
    }, // memory
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0,
        virtual_start: 0x0,
        size: 0x200000,
    }, // ram
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x110000,
    //     virtual_start: 0x110000,
    //     size: 0xf0000,
    // }, // ramoops
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x10f000,
    //     virtual_start: 0x10f000,
    //     size: 0x1000,
    // }, //scmi-shmem
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xfd440000,
        virtual_start: 0xfd440000,
        size: 0x20000,
    }, // its
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1f0000000,
        virtual_start: 0x1f0000000,
        size: 0x10000000,
    }, // memory
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe2b0000,
        virtual_start: 0xfe2b0000,
        size: 0x4000,
    }, //dwmmc mmc1
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe2c0000,
        virtual_start: 0xfe2c0000,
        size: 0x4000,
    }, //dwmmc mmc2
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe000000,
        virtual_start: 0xfe000000,
        size: 0x4000,
    }, //dwmmc mmc3
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE660000,
        virtual_start: 0xFE660000,
        size: 0x10000,
    }, //serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE680000,
        virtual_start: 0xFE680000,
        size: 0x10000,
    }, //serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDC20000,
        virtual_start: 0xFDC20000,
        size: 0x10000,
    }, //syscon
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDC60000,
        virtual_start: 0xFDC60000,
        size: 0x30000,
    }, //syscon
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDD20000,
        virtual_start: 0xFDD20000,
        size: 0x10000,
    }, //clock-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE530000,
        virtual_start: 0xFE530000,
        size: 0x10000,
    }, //dmac
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE5f0000,
        virtual_start: 0xFE5f0000,
        size: 0x10000,
    }, //timer
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDD00000,
        virtual_start: 0xFDD00000,
        size: 0x10000,
    }, //clock-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE750000,
        virtual_start: 0xFE750000,
        size: 0x10000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE740000,
        virtual_start: 0xFE740000,
        size: 0x10000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFDD60000,
        virtual_start: 0xFDD60000,
        size: 0x10000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE760000,
        virtual_start: 0xFE760000,
        size: 0x20000,
    }, //gpio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xFE310000,
        virtual_start: 0xFE310000,
        size: 0x10000,
    }, //sdhci
];

pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    0x84, 0x98, 0x40, 0x104, 0x105, 0x106, 0x107, 0x2d, 0x2e, 0x2b, 0x2a, 0x29, 0x33, 0x96, 0x11c,
    0x44, 0x43, 0x42, 0x41, 0x8d,
]);

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0xfd400000,
        gicd_size: 0x10000,
        gicr_base: 0xfd460000,
        gicr_size: 0xc0000,
        gits_base: 0xfd440000,
        gits_size: 0x20000,
    }),
};
pub const ROOT_PCI_CONFIG: &[HvPciConfig] = &[
    // HvPciConfig {
    //     ecam_base: 0xfe260000,
    //     ecam_size: 0x400000,
    //     io_base: 0xf4100000,
    //     io_size: 0x100000,
    //     pci_io_base: 0xf4100000,
    //     mem32_base: 0xf4200000,
    //     mem32_size: 0x1e00000,
    //     pci_mem32_base: 0xf4200000,
    //     mem64_base: 0x300000000,
    //     mem64_size: 0x40000000,
    //     pci_mem64_base: 0x300000000,
    //     bus_range_begin: 0x0,
    //     bus_range_end: 0x10,
    // },
    HvPciConfig {
        ecam_base: 0x3c0400000,
        ecam_size: 0x400000,
        io_base: 0xf2100000,
        io_size: 0x100000,
        pci_io_base: 0xf2100000,
        mem32_base: 0xf2200000,
        mem32_size: 0x1e00000,
        pci_mem32_base: 0xf2200000,
        mem64_base: 0x340000000,
        mem64_size: 0x40000000,
        pci_mem64_base: 0x340000000,
        bus_range_begin: 0x10,
        bus_range_end: 0x1f,
    },
    // HvPciConfig {
    //     ecam_base: 0xfe280000,
    //     ecam_size: 0x400000,
    //     io_base: 0xf0100000,
    //     io_size: 0x100000,
    //     pci_io_base: 0xf0100000,
    //     mem32_base: 0xf0200000,
    //     mem32_size: 0x1e00000,
    //     pci_mem32_base: 0xf0200000,
    //     mem64_base: 0x380000000,
    //     mem64_size: 0x40000000,
    //     pci_mem64_base: 0x380000000,
    //     bus_range_begin: 0x20,
    //     bus_range_end: 0x2f,
    // }
];

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_DWC_ATU_CONFIG: &[HvDwcAtuConfig] = &[
    HvDwcAtuConfig {
        ecam_base: 0x3c0400000,
        dbi_base: 0x3c0400000,
        dbi_size: 0x10000,
        apb_base: 0xfe270000,
        apb_size: 0x10000,
        cfg_base: 0xf2000000,
        cfg_size: 0x80000*2,
        io_cfg_atu_shared: 0,
    },
];

pub const ROOT_PCI_DEVS: [HvPciDevConfig; 2] = [
    pci_dev!(0x10, 0x0, 0x0, VpciDevType::Physical),
    pci_dev!(0x11, 0x0, 0x0, VpciDevType::Physical),
];