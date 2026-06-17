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
use crate::{arch::zone::HvArchZoneConfig, config::*, pci::vpci_dev::VpciDevType, pci_dev};

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
pub const IOMMU_SYS_BASE: usize = 0x3010000;
pub const IOMMU_SYS_SIZE: usize = 0x1000;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2);
pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x85000000,
        virtual_start: 0x85000000,
        size: 0x7B000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start: 0x10000000,
        size: 0x1000,
    }, // serial
];

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x20;
pub const HW_IRQS: &[u32] = &[7, 10, 33, 34];
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[10, 33]); // ARCH= riscv .It doesn't matter temporarily.

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0x0,
    plic_size: 0x0,
    aplic_base: APLIC_S_BASE,
    aplic_size: APLIC_S_SIZE,
};

pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    ecam_base: 0x30000000,
    ecam_size: 0x10000000,
    io_base: 0x3000000,
    io_size: 0x10000,
    pci_io_base: 0x0,
    mem32_base: 0x40000000,
    mem32_size: 0x40000000,
    pci_mem32_base: 0x40000000,
    mem64_base: 0x400000000,
    mem64_size: 0x400000000,
    pci_mem64_base: 0x400000000,
    bus_range_begin: 0,
    bus_range_end: 0xff,
    domain: 0x0,
}];

pub const ROOT_ZONE_IVC_CONFIG: &[HvIvcConfig] = &[];

pub const ROOT_PCI_DEVS: &[HvPciDevConfig] = &[
    pci_dev!(0x0, 0x0, 0x0, 0x0 => 0x0, 0x0, 0x0, VpciDevType::Physical),
    pci_dev!(0x0, 0x0, 0x1, 0x0 => 0x0, 0x1, 0x0, VpciDevType::Physical),
    // pci_dev!(0x0, 0x0, 0x3, 0x0 => 0x0, 0x3, 0x0, VpciDevType::Physical),
    // pci_dev!(0x0, 0x0, 0x5, 0x0 => 0x0, 0x5, 0x0, VpciDevType::StandardVdev),
];
