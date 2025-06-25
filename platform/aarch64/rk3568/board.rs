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

use crate::{arch::zone::{HvArchZoneConfig,GicConfig,Gicv3Config}, config::*};

pub const BOARD_NAME: &str = "rk3568";

pub const BOARD_NCPUS: usize = 4;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0xa0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x60080000 ;
pub const ROOT_ZONE_ENTRY: u64 = 0x60080000 ;
//pub const ROOT_ZONE_CPUS: u64 = (1 << 0) ;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0)|(1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 19] = [
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xfd400000,
    //     virtual_start: 0xfd400000,
    //     size: 0x10000,
    // }, // gic
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xfd460000,
    //     virtual_start: 0xfd460000,
    //     size: 0xc0000,
    // }, // gic
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe000000,
        virtual_start: 0xfe000000,
        size: 0x4000,
    }, // dwmmc
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
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x0,
    //     virtual_start: 0x0,
    //     size: 0x200000,
    // }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x110000,
        virtual_start: 0x110000,
        size: 0xf0000,
    }, // ramoops
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10f000,
        virtual_start: 0x10f000,
        size: 0x1000,
    }, //scmi-shmem
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1f0000000,
        virtual_start: 0x1f0000000,
        size: 0x10000000,
    }, // memory
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

pub const ROOT_ZONE_IRQS: [u32; 20] = [
    0x84, 0x98, 0x40, 0x104, 0x105, 0x106, 0x107, 0x2d, 0x2e, 0x2b, 0x2a, 0x29, 0x33, 0x96, 0x11c, 0x44, 0x43, 0x42, 0x41, 0x8d];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gic_version: 3,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0xfe600000,
        gicd_size: 0x10000,
        gicr_base: 0xfe680000,
        gicr_size: 0x100000,
        gits_base: 0x8080000,
        gits_size: 0x20000,
    }),
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_PCI_DEVS: [u64; 0] = [];