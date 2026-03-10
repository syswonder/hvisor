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
pub const BOARD_NAME: &str = "qemu-aia";
pub const BOARD_NCPUS: usize = 4;
#[rustfmt::skip]
pub static BOARD_HARTID_MAP: [usize; BOARD_NCPUS] = [
    0x0,            // core0   \
    0x1,            // core1    | -> cluster0 -> CPU
    0x2,            // core2    |
    0x3,            // core3   / 
];
pub const TIMEBASE_FREQ: u64 = 10_000_000; // 10MHz
pub const ACLINT_SSWI_BASE: usize = 0x2F00000;

// This device is used for qemu-quit.
#[allow(unused)]
pub const SIFIVE_TEST_BASE: u64 = 0x100000;

pub const APLIC_S_BASE: usize = 0xd000000;
pub const APLIC_S_SIZE: usize = 0x8000; // Related to BOARD_NCPUS.
pub const BOARD_APLIC_INTERRUPTS_NUM: usize = 1023;
pub const IMSIC_S_BASE: usize = 0x2800_0000;
pub const IMSIC_GUEST_NUM: usize = 1; // hvisor only supports 1 guest now.
pub const IMSIC_GUEST_INDEX: usize = 1;
pub const IMSIC_NUM_IDS: usize = 0xFF;
pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2);
pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x83000000,
        virtual_start: 0x83000000,
        size: 0x1D000000,
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

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x20;
pub const HW_IRQS: [u32; 11] = [1, 2, 3, 4, 5, 8, 10, 33, 34, 35, 36];
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] =
    &get_irqs_bitmap(&[1, 2, 3, 4, 5, 8, 10, 33, 34, 35, 36]); // ARCH= riscv .It doesn't matter temporarily.

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0x0,
    plic_size: 0x0,
    aplic_base: APLIC_S_BASE,
    aplic_size: APLIC_S_SIZE,
};
