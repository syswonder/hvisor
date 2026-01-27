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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use crate::pci::vpci_dev::VpciDevType;
use crate::{arch::zone::HvArchZoneConfig, config::*, pci_dev};

// ==================================================
//              System Level Config
// ==================================================
#[allow(unused)]
pub const BOARD_NAME: &str = "ur-dp1000";
// CPU Topology Information
pub const BOARD_NCPUS: usize = 8;
#[rustfmt::skip]
pub static BOARD_HARTID_MAP: [usize; BOARD_NCPUS] = [
    0x0,            // core0   \
    0x1,            // core1    | -> cluster0 \
    0x2,            // core2    |              |
    0x3,            // core3   /               | -> CPU
    0x10,           // core4   \               |
    0x11,           // core5    | -> cluster1 /
    0x12,           // core6    |
    0x13,           // core7   /
];
// Timebase frequency
pub const TIMEBASE_FREQ: u64 = 10_000_000;
// PLIC Configuration
pub const PLIC_BASE: usize = 0x9000000;
pub const PLIC_SIZE: usize = 0x4000000;
// Number of interrupts is defined in dts, its max value is 1023.
// riscv,ndev = <0xa0>
pub const BOARD_PLIC_INTERRUPTS_NUM: usize = 0xa0; // except irq 0, here range is [1, 160]
pub const NUM_CONTEXTS_PER_HART: usize = 3; // M-mode、S-mode and VS-mode(unused)

// ==================================================
//              Root Zone Config
// ==================================================
pub const ROOT_ZONE_NAME: &str = "root-linux";
// ROOT_ZONE_DTB_ADDR is HPA (Host physical Address).
pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8F000000;
// ROOT_ZONE_KERNEL_ADDR is HPA (Host Physical Address), but it isn't used now.
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
// ROOT_ZONE_ENTRY is GPA (Guest Physical Address).
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = 0x3; // core0,1,2,3
#[rustfmt::skip]
pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    // Main memory (~32GB), reserved 0x500_0000 (80M) for high-level privilege softwares (opensbi & hvisor).
    HvConfigMemoryRegion { mem_type: MEM_TYPE_RAM, physical_start: 0x8500_0000, virtual_start: 0x8500_0000, size: 0x8_0000_0000 - 0x500_0000 },
    // Serial0
    HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x2030_0000, virtual_start: 0x2030_0000, size: 0x1_0000 },
    // // Pcie_x16
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x2100_0000, virtual_start: 0x2100_0000, size: 0x100_0000 }, // IP_register
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x4fff_0000, virtual_start: 0x4fff_0000, size: 0x1_0000 }, // Configuration Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x4fbf_0000, virtual_start: 0x4fbf_0000, size: 0x40_0000 }, // IO Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x4000_0000, virtual_start: 0x4000_0000, size: 0xfbf_0000 }, // Mem32 Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x40_0000_0000, virtual_start: 0x40_0000_0000, size: 0xd_0000_0000 }, // Mem64 Space
    // // Pcie_x4a 
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x2300_0000, virtual_start: 0x2300_0000, size: 0x100_0000 }, // IP register
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x6fff_0000, virtual_start: 0x6fff_0000, size: 0x1_0000 }, // Configuration Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x6fbf_0000, virtual_start: 0x6fbf_0000, size: 0x40_0000 }, // IO Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x6000_0000, virtual_start: 0x6000_0000, size: 0xfbf_0000 }, // Mem32 Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x80_0000_0000, virtual_start: 0x80_0000_0000, size: 0xd_0000_0000 }, // Mem64 Space
    // // Pcie_x4b
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x2400_0000, virtual_start: 0x2400_0000, size: 0x100_0000 }, // IP register
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x7fff_0000, virtual_start: 0x7fff_0000, size: 0x1_0000 }, // Configuration Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x7fbf_0000, virtual_start: 0x7fbf_0000, size: 0x40_0000 }, // IO Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x7000_0000, virtual_start: 0x7000_0000, size: 0xfbf_0000 }, // Mem32 Space
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0xc0_0000_0000, virtual_start: 0xc0_0000_0000, size: 0xd_0000_0000 }, // Mem64 Space
    // // Eth
    // HvConfigMemoryRegion { mem_type: MEM_TYPE_IO, physical_start: 0x3800_0000, virtual_start: 0x3800_0000, size: 0x100_0000 },
];
// No mapped io regions in root zone.
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: PLIC_BASE,
    plic_size: PLIC_SIZE,
    aplic_base: 0x0, // UNUSED for PLIC
    aplic_size: 0x0, // UNUSED for PLIC
};

// ==================================================
//              Interrupt Config
// ==================================================
pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x20;
// Note: all here's irqs are hardware irqs,
//  only these irq can be transferred to the physical PLIC.
#[rustfmt::skip]
pub const HW_IRQS: &[u32] = &[
    17, 18, 25, 26, // uart0, uart1, uart2, uart3
    19, 27, // spi0, spi1
    20, 21, 28, 29, // i2c0, i2c1, i2c2, i2c3
    33, // watchdog@20210000
    34, // gpio@20200000 porta
    84, // ethernet1@38000000
    152, //dma-controller@39000000
    43, 44, 45, 46, 47, 48, // pcie_x16@21000000, msi, inta, intb, intc, intd, aer
    63, 64, 65, 66, 67, 68, // pcie_x4a@23000000, msi, inta, intb, intc, intd, aer
    73, 74, 75, 76, 77, 78, // pcie_x4b@24000000, msi, inta, intb, intc, intd, aer
];
// irqs belong to the root zone.
#[rustfmt::skip]
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    17, // uart0,
    // 84, // ethernet1@38000000
    // 43, 44, 45, 46, 47, 48, // pcie_x16@21000000, msi, inta, intb, intc, intd, aer
    // 63, 64, 65, 66, 67, 68, // pcie_x4a@23000000, msi, inta, intb, intc, intd, aer
    // 73, 74, 75, 76, 77, 78, // pcie_x4b@24000000, msi, inta, intb, intc, intd, aer
]);

// =====================================================
//              PCIe Config
// Note: here only config the pciex4b, the others,
//       pciex16 and pciex4a are directly passed through
// =====================================================
// pub const ROOT_PCI_CONFIG: &[HvPciConfig] = &[HvPciConfig {
//     ecam_base: 0x24000000, // reg = <0x0 0x24000000 0x0 0x01000000>, /* IP registers */
//     ecam_size: 0x1000000,
//     io_base: 0x7fbf0000, // IO Space <0x81000000  0x0 0x7fbf0000  0x0 0x7fbf0000 0x0 0x00400000>
//     io_size: 0x400000,
//     pci_io_base: 0x7fbf0000,
//     mem32_base: 0x70000000, // Mem32 Space <0x82000000  0x0 0x70000000  0x0 0x70000000 0x0 0x0fbf0000>
//     mem32_size: 0xfbf0000,
//     pci_mem32_base: 0x70000000,
//     mem64_base: 0xc000000000, // Mem64 Space <0xc3000000 0xc0 0x00000000 0xc0 0x00000000 0xd 0x00000000>
//     mem64_size: 0xd00000000,
//     pci_mem64_base: 0xc000000000,
//     bus_range_begin: 0x0, // bus-range = <0x0 0xff>
//     bus_range_end: 0xff,
// }];
// pub const ROOT_DWC_ATU_CONFIG: &[HvDwcAtuConfig] = &[HvDwcAtuConfig {
//     ecam_base: 0x24000000, // reg = <0x0 0x24000000 0x0 0x01000000>, /* IP registers */
//     dbi_base: 0x24000000,  // reg = <0x0 0x24000000 0x0 0x01000000>, /* IP registers */
//     dbi_size: 0x1000000,
//     apb_base: 0x0,        // UNUSED
//     apb_size: 0x0,        // UNUSED
//     cfg_base: 0x7fff0000, //       <0x0 0x7fff0000 0x0 0x00010000>; /* Configuration space */
//     cfg_size: 0x10000,
//     io_cfg_atu_shared: 0, // IO and Configuration space share the same ATU
// }];
// pub const ROOT_PCI_DEVS: &[HvPciDevConfig] = &[
//     pci_dev!(0x0, 0x0, 0x0, VpciDevType::Physical), // 00:00.0 PCIe x16 Root Complex                                                                0x0
//     pci_dev!(0x1, 0x0, 0x0, VpciDevType::Physical), // 01:00.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch                                            0x100
//     // pci_dev!(0x2, 0x0, 0x0, VpciDevType::Physical), // 02:00.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch                                         0x200
//     // pci_dev!(0x2, 0x2, 0x0, VpciDevType::Physical), // 02:02.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch (04:00.0 USB controller is behind it)   0x210
//     // pci_dev!(0x2, 0x3, 0x0, VpciDevType::Physical), // 02:03.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch                                         0x218
//     // pci_dev!(0x2, 0x8, 0x0, VpciDevType::Physical), // 02:08.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch (06:00.0 SATA controller is behind it)  0x240
//     pci_dev!(0x2, 0xa, 0x0, VpciDevType::Physical), // 02:0a.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch (07:00.0 USB controller is behind it)      0x250
//     // pci_dev!(0x2, 0xb, 0x0, VpciDevType::Physical), // 02:0b.0 ASM2812 6-port PCIe x4 Gen3 Packet Switch                                         0x258
//     // pci_dev!(0x4, 0x0, 0x0, VpciDevType::Physical), // 04:00.0 USB controller                                                                 0x400
//     // pci_dev!(0x6, 0x0, 0x0, VpciDevType::Physical), // 06:00.0 SATA controller                                                                0x600
//     pci_dev!(0x7, 0x0, 0x0, VpciDevType::Physical), // 07:00.0 USB controller                                                                    0x700
// ];
