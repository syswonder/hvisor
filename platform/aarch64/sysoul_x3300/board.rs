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

use crate::pci_dev;
use crate::{
    arch::{
        mmu::MemoryType,
        zone::{GicConfig, Gicv3Config, HvArchZoneConfig, UefiConfig},
    },
    config::*,
    pci::vpci_dev::VpciDevType,
};

pub const BOARD_NAME: &str = "sysoul-x3300";

pub const BOARD_NCPUS: usize = 8;
pub const BOARD_UART_BASE: u64 = 0xfeb5_0000;

#[rustfmt::skip]
pub static BOARD_MPIDR_MAPPINGS: [u64; BOARD_NCPUS] = [
    0x000,   // cpu0
    0x100,   // cpu1
    0x200,   // cpu2
    0x300,   // cpu3
    0x400,   // cpu4
    0x500,   // cpu5
    0x600,   // cpu6
    0x700,   // cpu7
];

/// Early boot cache invalidate mask (per CPU): bit0->L1(D), bit1->L2, bit2->L3.
pub static BOARD_EARLY_CACHE_INVALIDATE_MASKS: [u64; BOARD_NCPUS] = [0b111; BOARD_NCPUS];

/// The physical memory layout of the board.
/// Each address should align to 2M (0x20_0000).
/// Addresses must be in ascending order.
#[rustfmt::skip]
pub const BOARD_PHYSMEM_LIST: &[(u64, u64, MemoryType)] = &[
 // (        start,           end,               type)
    (  0x0000_0000,   0x0020_0000, MemoryType::Device),     // Includes low-address SRAM, marked as Device
    (  0x0020_0000,   0x0840_0000, MemoryType::Normal),
    (  0x0940_0000,   0xf000_0000, MemoryType::Normal),
    (  0xf000_0000, 0x1_0000_0000, MemoryType::Device),     // Dense device region, marked as Device.
    (0x1_0000_0000, 0x3_fc00_0000, MemoryType::Normal),
 // (0x3_fc50_0000, 0x3_fff0_0000, MemoryType::Normal),
    (0x3_fc40_0000, 0x4_0000_0000, MemoryType::Normal),     // aligned to 2 MiB
    (0x4_f000_0000, 0x5_0000_0000, MemoryType::Normal),
];

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x2000_0000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x2040_0000;
pub const ROOT_ZONE_ENTRY: u64 = 0x2040_0000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 2) - 1;

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    // /proc/iomem System RAM
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0020_0000,
        virtual_start: 0x0020_0000,
        size: 0x0820_0000,
    },
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x0940_0000,
    //     virtual_start: 0x0940_0000,
    //     size: 0xe6c0_0000,
    // },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0940_0000,
        virtual_start: 0x0940_0000,
        size: 0x06c0_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1000_0000,
        virtual_start: 0x1000_0000,
        size: 0x1000_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2000_0000,
        virtual_start: 0x2000_0000,
        size: 0x2000_0000,
    }, // zone0 kernel/dtb area
    // Root RAM 0x4000_0000..0xf000_0000; the 0x5800_0000..0xd800_0000
    // sub-range belongs to zone2 (android) and is no-map reserved in zone0.dts.
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x4000_0000,
        virtual_start: 0x4000_0000,
        size: 0xb000_0000,
    },
    // Root high RAM; the 0x1_0000_0000..0x2_fc00_0000 sub-range belongs to
    // zone1 (npu) and is no-map reserved in zone0.dts.
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1_0000_0000,
        virtual_start: 0x1_0000_0000,
        size: 0x2_fc00_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x3_fc50_0000,
        virtual_start: 0x3_fc50_0000,
        size: 0x03a0_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x4_f000_0000,
        virtual_start: 0x4_f000_0000,
        size: 0x1000_0000,
    },
    // Ramoops
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0011_0000,
        virtual_start: 0x0011_0000,
        size: 0x000f_0000,
    },
    // /proc/iomem Devices I/O
    // GPU region (0xfb00_0000-0xfb20_0000) moved to zone2 (android)
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xfb00_0000,
    //     virtual_start: 0xfb00_0000,
    //     size: 0x0020_0000,
    // },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfc00_0000,
        virtual_start: 0xfc00_0000,
        size: 0x15a_4000, // fc000000..fd5a4000
    },
    // VOP-GRF: keep in root zone for DDR MCU access
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfd5a_4000,
        virtual_start: 0xfd5a_4000,
        size: 0x2000, // fd5a4000..fd5a6000
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfd5a_6000,
        virtual_start: 0xfd5a_6000,
        size: 0x7ea_000, // fd5a6000..fdd90000
    },
    // VOP: keep in root zone for DDR MCU line-flag synchronization
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfdd9_0000,
        virtual_start: 0xfdd9_0000,
        size: 0x8000, // fdd90000..fdd98000
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfdd9_8000,
        virtual_start: 0xfdd9_8000,
        size: 0x26_8000, // fdd98000..fe000000
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe00_0000,
        virtual_start: 0xfe00_0000,
        size: 0x0060_0000,
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfea0_0000,
        virtual_start: 0xfea0_0000,
        size: 0x0050_0000,
    },
    // SRAM and Other Devices
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x0010_f000,
        virtual_start: 0x0010_f000,
        // size: 0x0100, // 10f000.sram
        size: 0x1000, // aligned with page size
    },
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff00_1000,
        virtual_start: 0xff00_1000,
        size: 0x000e_e000, //ff001000.sram
    },
    // Unknown Region, maybe we should ask vendor for help
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x0010_0000,
        virtual_start: 0x0010_0000,
        size: 0xf000,
    },
    // Unknown Region, maybe we should ask vendor for help
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xa_0000_0000,
        virtual_start: 0xa_0000_0000,
        size: 0x1_0000_0000,
    },
];

pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    // core / platform
    0x27, // arm-pmu
    0x29, // gic
    0x2a, 0x2b, 0x2d, 0x2e, // arch timer
    0x40, // hvisor_virtio_device
    0x69, // dmc
    0x76, 0x77, 0x78, 0x79, // dma-controller@fea10000/fea30000
    0x7a, 0x7b, // dma-controller@fed10000
    0xeb, // sdmmc@fe2c0000 (root SD rootfs)
    0x109, 0x10a, // ethernet@fe1c0000 (gmac1, root network)
    0x135, // gpio@fd8a0000 (gpio0: SD vmmc enable, rk806 int etc.)
    // 0x137, // gpio2 -> zone2 (android panel reset/enable)
    0x138, // gpio@fec40000 (gpio3)
    0x139, // gpio@fec50000 (gpio4)
    0x141, // timer@feae0000 (rktimer)
    0x15d, // i2c@fd880000 (i2c0: rk860/pmic aux)
    0x163, // i2c@fec80000 (i2c6: rtc hym8563)
    0x168, // spi@feb20000 (spi2: rk806 PMIC)
    0x16d, // serial@feb50000 (uart2: root console ttyS2)
    0x1ad, // tsadc@fec00000
    0x1c7, // fiq-debugger (uart2 console ttyFIQ0, irq-mode)
    0x1ae, // saradc@fec10000
    0x1b0, // rng@fe378000
    // usb: only usbdrd3_1 (usb@fc400000) + its usb2phy stay in root;
    // usb@fc000000/fcd00000 dwc3, ehci/ohci pairs, usb2phy0-2 and the
    // usb iommus moved to zone2 (android), see zone2-android.dts/json.
    0xfd, // usb@fc400000 dwc3 (usbdrd3_1)
    0x1aa, // usb2phy1 (usb2-phy@4000, root usbdrd3_1 companion)
    // Everything else (camera/mpp/audio/pcie/sata/touch/extra uart/i2c/spi/
    // mailbox/display chains) is disabled in zone0.dts &{...} overrides or
    // passed through to zone1/zone2 - see image/dts/zone0.dts.
]);
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    is_aarch32: 0,
    uefi_config: UefiConfig::NoUefi,
    gic_config: GicConfig::Gicv3(Gicv3Config {
        gicd_base: 0xfe60_0000,
        gicd_size: 0x0001_0000,
        gicr_base: 0xfe68_0000,
        gicr_size: 0x0010_0000,
        gits_base: 0x0,
        gits_size: 0x0,
    }),
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

pub const ROOT_PCI_DEVS: [HvPciDevConfig; 0] = [];
