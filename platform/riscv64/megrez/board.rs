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
        physical_start: 0x50900000,
        virtual_start: 0x50900000,
        size: 0x10000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x50460000,
        virtual_start: 0x50460000,
        size: 0x10000,
    }, // mmc
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x50440000,
        virtual_start: 0x50440000,
        size: 0x2000,
    }, // hsp_sp_top_csr
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x51828000,
        virtual_start: 0x51828000,
        size: 0x80000,
    }, // sys-crg (clock-controller, reset-controller) (SD card needs)
    // Cache controller is needed, otherwise terminal will report "VFS: cannot open root device..."
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
        physical_start:  0xc0_0000_0000,
        virtual_start: 0xc0_0000_0000,
        // mem-port -> sys-port (here easily equal to mem size)
        size: 0x8000_0000,
    }, // Sys-port. (here related to DMA)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start:  0x104000,
        virtual_start: 0x104000,
        size: 0x4000,
    }, // pL2Cache. (cpu0)
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start:  0x108000,
    //     virtual_start: 0x108000,
    //     size: 0x4000,
    // }, // pL2Cache. (cpu1)
]; 

// Note: all here's irqs are hardware irqs,
//  only these irq can be transferred to the physical PLIC.
pub const HW_IRQS: [u32; 6] = [0x01, 0x03, 0x04, 0x02, 0x51, 0x64];

// irqs belong to the root zone.
pub const ROOT_ZONE_IRQS: [u32; 6] =  [0x01, 0x03, 0x04, 0x02, 0x51, 0x64];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0xc000000,
    plic_size: 0x4000000,
    aplic_base: 0xd000000,
    aplic_size: 0x8000,
};

// Virtio zone1 cmd:
//      nohup ./hvisor virtio start zone1-linux-virtio.json &
//      ./hvisor zone start zone1-linux.json
