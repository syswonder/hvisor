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

#[allow(unused)]
pub const BOARD_NAME: &str = "qem-plic";
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
pub const PLIC_BASE: usize = 0xc000000;
pub const PLIC_SIZE: usize = 0x4000000;
pub const BOARD_PLIC_INTERRUPTS_NUM: usize = 1023; // except irq 0

// This device is used for qemu-quit.
#[allow(unused)]
pub const SIFIVE_TEST_BASE: u64 = 0x100000;
pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);
pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
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
        physical_start: 0x10008000,
        virtual_start: 0x10008000,
        size: 0x1000,
    }, // virtio
];

// Note: all here's irqs are hardware irqs,
//  only these irq can be transferred to the physical PLIC.
// For qemu-virt, 32 is pcie pinA, 33 is pcie pinB, 34 is pcie pinC, 35 is pcie pinD, now only use 33,34
pub const HW_IRQS: &[u32] = &[10, 33, 34];

// irqs belong to the root zone.
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[10, 33]);

// Interrupt-controller config for the root zone.
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: PLIC_BASE,
    plic_size: PLIC_SIZE,
    aplic_base: 0x0,
    aplic_size: 0x0,
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
    pci_dev!(0x0, 0x0, 0x0, 0x0, VpciDevType::Physical),
    pci_dev!(0x0, 0x0, 0x1, 0x0, VpciDevType::Physical),
    // pci_dev!(0x0, 0x0, 0x3, 0x0, VpciDevType::Physical),
    // pci_dev!(0x0, 0x0, 0x5, 0x0, VpciDevType::StandardVdev),
];
