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

use crate::pci::vpci_dev::VpciDevType;
use crate::pci_dev;

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
       // 0x32f00000
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start: 0x32f00000,
       //     virtual_start: 0x32f00000,
       //     size: 0x10000,
       // }, // pcie-phy
];

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    35, 36, 37, 38, 45, 52, 55, 56, 57, 59, 64, 67, 75, 96, 97, 98, 99, 100, 101, 102, 103, 104,
    105, 135, 150, 151, 152, 162, 172, 159,
]);

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0x38800000,
        gicd_size: 0x10000,
        gicr_base: 0x38880000,
        gicr_size: 0xc0000,
        gits_base: 0,
        gits_size: 0,
    }),
};

pub const ROOT_PCI_CONFIG: &[HvPciConfig] = &[HvPciConfig {
    ecam_base: 0x33800000,
    ecam_size: 0x400000,
    io_base: 0x1ff80000,
    io_size: 0x10000,
    pci_io_base: 0x0,
    mem32_base: 0x1800_0000,
    mem32_size: 0x7f0_0000,
    pci_mem32_base: 0x1800_0000,
    mem64_base: 0x0,
    mem64_size: 0x0,
    pci_mem64_base: 0x0,
    bus_range_begin: 0x0,
    bus_range_end: 0x1f,
    domain: 0x0,
}];

pub const ROOT_DWC_ATU_CONFIG: &[HvDwcAtuConfig] = &[HvDwcAtuConfig {
    ecam_base: 0x33800000,
    dbi_base: 0x33800000,
    dbi_size: 0x400000,
    apb_base: 0x0,
    apb_size: 0x0,
    cfg_base: 0x1ff00000,
    cfg_size: 0x80000,
    io_cfg_atu_shared: 1,
    io_atu_index: 1,
    dw_msi_irq: 172,
}];

pub const ROOT_PCI_DEVS: [HvPciDevConfig; 2] = [
    pci_dev!(0x0, 0x00, 0x0, 0x0, VpciDevType::Physical),
    pci_dev!(0x0, 0x01, 0x0, 0x0, VpciDevType::Physical),
];
