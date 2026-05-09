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
//      Ming Shen  <boneinscri@163.com>
use crate::pci_dev;
use crate::{arch::zone::HvArchZoneConfig, config::*, pci::vpci_dev::VpciDevType};

pub const BOARD_NAME: &str = "ls3a5000";

pub const BOARD_NCPUS: usize = 4;

pub const CPU_BOOT_CONTEXT_ADDRESS: usize = 0x90000001e0000000;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x10000f000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x200000;
pub const ROOT_ZONE_ENTRY: u64 = 0xe71000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux-la64";


pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x200000,
        virtual_start:  0x200000,
        size: 0xec00000,
    }, // RAM

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start:  0x1fe00000,
        size: 0x2000,
    }, // IO important MMIO
    
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10080000,
        virtual_start:  0x10080000,
        size: 0x1000,
    }, // serial

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10090000,
        virtual_start:  0x10090000,
        size: 0x1000,
    }, // IO 

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x18000000,
        virtual_start:  0x18000000,
        size: 0x1000,
    }, // IO???? unknown

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100a0000,
        virtual_start:  0x100a0000,
        size: 0x1000,
    }, // IO 

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100d0000,
        virtual_start:  0x100d0000,
        size: 0x1000,
    }, // IO 

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100e0000,
        virtual_start:  0x100e0000,
        size: 0x1000,
    }, // IO

    // 46f000000-47f7fffff
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x46f000000,
        virtual_start:  0x46f000000,
        size: 0x10800000,
    }, // Reserved RAM

    // 47f800000 - 47fffffff
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x47f800000,
        virtual_start:  0x47f800000,
        size: 0x800000,
    }, // Reserved RAM
    
    // ====== for start_image ======
    // 0x90000000 - 0xf9ffffff 0x6a000000
    // 0xf7000000 - 0xf7ffffff 0x1000000
    //(0xf9000000 - 0xf9ffffff 0x1000000)
    // 0xfa000000 - 0xfaffffff 0x1000000
    // 0xfb000000 - 0xfbffffff 0x1000000
    // 0xfc000000 - 0xfcffffff 0x1000000
    // 0xfd000000 - 0xfdffffff 0x1000000
    // 0xfe000000 - 0xfeffffff 0x1000000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x90000000,
        virtual_start:  0x90000000,
        size: 0x6a000000,
    }, // RAM

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xfa000000,
        virtual_start:  0xfa000000,
        size: 0x5000000,
    }, // RAM

    // 0x800000000 ～ 0x87fffffff
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x800000000,
        virtual_start:  0x800000000,
        size: 0x80000000,
    }, // RAM

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x7f0000000,
        virtual_start:  0x7f0000000,
        size: 0x10000000,
    }, // RAM

    // ==== for start_image ====

    // addition
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        // mem_type: MEM_TYPE_PCH_PCI,
        physical_start: 0x10000000,
        virtual_start:  0x10000000,
        size: 0x1000,
    }, // IO!!!!????? PCH-PCI
    
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10002000,
        virtual_start:  0x10002000,
        size: 0x1000,
    }, // IO!!!!?????

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10010000,
        virtual_start:  0x10010000,
        size: 0x1000,
    }, // IO!!!!?????

    // ===========unknown region===============
    // addition
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x100000000,
        virtual_start:  0x100000000,
        size: 0x200000000,
        // size: 0xe0000000,
    }, // RAM

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x200000000,
    //     virtual_start:  0x200000000,
    //     size: 0x100000000,
    //     // size: 0x10000,
    // }, // RAM
    // =========unknown region======

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe0030000000,
        virtual_start:  0xe0030000000,
        size: 0x50000000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xefe00000000,
        virtual_start:  0xefe00000000,
        size: 0x20000000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xefdfe000000,
        virtual_start:  0xefdfe000000,
        size: 0x2000,
    }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xe8000000000,
    //     virtual_start:  0xe8000000000,
    //     size: 0x10000000,
    // }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xe8015150000,
    //     virtual_start:  0xe8015150000,
    //     size: 0x8000
    // }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xe8015000000,
    //     virtual_start:  0xe8015000000,
    //     size: 0x100000
    // }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IOMMU,
    //     physical_start: 0xe0030000000,
    //     virtual_start:  0xe0030000000,
    //     size: 0x50000000,
    // }, // PCI-IOMMU

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IOMMU,
    //     physical_start: 0xefe00000000,
    //     virtual_start:  0xefe00000000,
    //     size: 0x20000000,
    // }, // PCI-IOMMU
];

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[]);
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };
pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    ecam_base: 0xfe00000000,
    ecam_size: 0x20000000,
    io_base: 0x18408000,
    io_size: 0x8000,
    pci_io_base: 0x00008000,
    mem32_base: 0x0,
    mem32_size: 0x0,
    pci_mem32_base: 0x0,
    mem64_base: 0x60000000,
    mem64_size: 0x30000000,
    pci_mem64_base: 0x60000000,
    bus_range_begin: 0,
    bus_range_end: 0xff,
    domain: 0x0,
}];

/* 00:00.0, 00:00.1, 00:00.2, 00:00.3, 00:04.0, 00:04.1*/
/* 00:05.0, 00:05.1, 00:06.0, 00:06.1, 00:06.2 */
/* 00:07.0, 00:08.0, 00:09.0, 00:0a.0, 00:0b.0 */
/* 00:0c.0, 00:0d.0, 00:0f.0, 00:10.0, 00:13.0 */
/* 00:16.0, 00:19.0, 02:00.0, 05:00.0 */
/* BUS 8 on X16 slot */
/* 08:00.0, 08:00.1, 08:00.2, 08:00.3 net */
/* BUS 6 on X4 slot */
/* 06:00.0, 06:00.1, 06:00.2, 06:00.3 net */
pub const ROOT_PCI_DEVS: &[HvPciDevConfig] = &[
    pci_dev!(0x0, 0x0, 0x0, 0x0, VpciDevType::Physical), // 00:00.0
    pci_dev!(0x0, 0x0, 0x0, 0x1, VpciDevType::Physical), // 00:00.1
    pci_dev!(0x0, 0x0, 0x0, 0x2, VpciDevType::Physical), // 00:00.2
    pci_dev!(0x0, 0x0, 0x0, 0x3, VpciDevType::Physical), // 00:00.3
    pci_dev!(0x0, 0x0, 0x4, 0x0, VpciDevType::Physical), // 00:04.0
    pci_dev!(0x0, 0x0, 0x4, 0x1, VpciDevType::Physical), // 00:04.1
    pci_dev!(0x0, 0x0, 0x5, 0x0, VpciDevType::Physical), // 00:05.0
    pci_dev!(0x0, 0x0, 0x5, 0x1, VpciDevType::Physical), // 00:05.1
    // pci_dev!(0x0, 0x0, 0x6, 0x0, VpciDevType::Physical),  // 00:06.0
    // pci_dev!(0x0, 0x0, 0x6, 0x1, VpciDevType::Physical),  // 00:06.1
    // pci_dev!(0x0, 0x0, 0x6, 0x2, VpciDevType::Physical),  // 00:06.2
    pci_dev!(0x0, 0x0, 0x7, 0x0, VpciDevType::Physical), // 00:07.0
    pci_dev!(0x0, 0x0, 0x8, 0x0, VpciDevType::Physical), // 00:08.0
    pci_dev!(0x0, 0x0, 0x9, 0x0, VpciDevType::Physical), // 00:09.0
    pci_dev!(0x0, 0x0, 0xa, 0x0, VpciDevType::Physical), // 00:0a.0
    pci_dev!(0x0, 0x0, 0xb, 0x0, VpciDevType::Physical), // 00:0b.0
    pci_dev!(0x0, 0x0, 0xc, 0x0, VpciDevType::Physical), // 00:0c.0
    pci_dev!(0x0, 0x0, 0xd, 0x0, VpciDevType::Physical), // 00:0d.0
    pci_dev!(0x0, 0x0, 0xf, 0x0, VpciDevType::Physical), // 00:0f.0
    pci_dev!(0x0, 0x0, 0x10, 0x0, VpciDevType::Physical), // 00:10.0
    pci_dev!(0x0, 0x0, 0x13, 0x0, VpciDevType::Physical), // 00:13.0
    pci_dev!(0x0, 0x0, 0x16, 0x0, VpciDevType::Physical), // 00:16.0
    pci_dev!(0x0, 0x0, 0x19, 0x0, VpciDevType::Physical), // 00:19.0
    pci_dev!(0x0, 0x2, 0x0, 0x0, VpciDevType::Physical), // 02:00.0
    pci_dev!(0x0, 0x5, 0x0, 0x0, VpciDevType::Physical), // 05:00.0
    pci_dev!(0x0, 0x6, 0x0, 0x0, VpciDevType::Physical), // 06:00.0
];

// bus << 8 | dev << 5 | func << 3

// pub const ROOT_PCI_DEVS: [u64; 0] = [];

// "alloc_pci_devs": [0,1,2,3,32,33,40,41,56,64,72,80,88,96,104,120,128,152,176,200,512,1280,2051]
