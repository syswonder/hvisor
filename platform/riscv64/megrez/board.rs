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
use crate::{arch::zone::HvArchZoneConfig, config::*};

#[allow(unused)]
pub const BOARD_NAME: &str = "milkv-megrez";

pub const BOARD_NCPUS: usize = 4;

pub const PLIC_BASE: usize = 0xc000000;

pub const BOARD_PLIC_INTERRUPTS_NUM: usize = 1023; // except irq 0

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x83000000,
        virtual_start: 0x83000000,
        size: 0x7D000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x50440000,
        virtual_start: 0x50440000,
        size: 0x2000,
    }, // hsp_sp_top_csr (SD card needs)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x50460000,
        virtual_start: 0x50460000,
        size: 0x10000,
    }, // mmc
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x50900000,
        virtual_start: 0x50900000,
        size: 0x30000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x51828000,
        virtual_start: 0x51828000,
        size: 0x80000,
    }, // sys-crg (clock-controller, reset-controller) (SD card needs)
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x51810000,
    //     virtual_start: 0x51810000,
    //     size: 0x8000,
    // }, // eswin,win2030-scu-sys-con (SD card needs)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1700000,
        virtual_start: 0x1700000,
        size: 0x4000,
    }, // bus-error-unit(hart0)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x2010000,
        virtual_start: 0x2010000,
        size: 0x4000,
    }, // cache-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x8000000,
        virtual_start: 0x8000000,
        size: 0x400000,
    }, // cache-controller
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start:  0xc000000000,
        virtual_start: 0xc000000000,
        size: 0x100000000,
    }, // Big addr.
]; 

// Note: all here's irqs are hardware irqs,
//  only these irq can be transferred to the physical PLIC.
pub const HW_IRQS: [u32; 10] = [0x01, 0x03, 0x04, 0x02, 0x51, 0x64, 0x205, 0x206, 0x207, 0x208];

// irqs belong to the root zone.
pub const ROOT_ZONE_IRQS: [u32; 10] =  [0x01, 0x03, 0x04, 0x02, 0x51, 0x64, 0x205, 0x206, 0x207, 0x208];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0xc000000,
    plic_size: 0x4000000,
    aplic_base: 0xd000000,
    aplic_size: 0x8000,
};