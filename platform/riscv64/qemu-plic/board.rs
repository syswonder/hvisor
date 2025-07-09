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
pub const BOARD_NAME: &str = "qem-plic";

pub const BOARD_NCPUS: usize = 4;

pub const ACLINT_SSWI_BASE: usize = 0x2F00000;

pub const PLIC_BASE: usize = 0xc000000;

pub const BOARD_PLIC_INTERRUPTS_NUM: usize = 1023; // except irq 0

// This device is used for qemu-quit.
pub const SIFIVE_TEST_BASE: u64 = 0x100000;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

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
        physical_start: 0x10000000,
        virtual_start: 0x10000000,
        size: 0x1000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30000000,
        virtual_start: 0x30000000,
        size: 0x10000000,
    }, // pci
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10001000,
        virtual_start: 0x10001000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10002000,
        virtual_start: 0x10002000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10003000,
        virtual_start: 0x10003000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10004000,
        virtual_start: 0x10004000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10005000,
        virtual_start: 0x10005000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10008000,
        virtual_start: 0x10008000,
        size: 0x1000,
    }, // virtio
];

// Note: all here's irqs are hardware irqs,
//  only these irq can be transferred to the physical PLIC.
pub const HW_IRQS: [u32; 11] = [1, 2, 3, 4, 5, 8, 10, 33, 34, 35, 36];

// irqs belong to the root zone.
pub const ROOT_ZONE_IRQS: [u32; 11] = [1, 2, 3, 4, 5, 8, 10, 33, 34, 35, 36];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0xc000000,
    plic_size: 0x4000000,
    aplic_base: 0xd000000,
    aplic_size: 0x8000,
};
