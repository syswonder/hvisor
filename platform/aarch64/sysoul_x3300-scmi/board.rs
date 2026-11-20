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
    // Zone1 RAM: 0x3000_0000 - 0x4000_0000 (assigned to zone1)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x4000_0000,
        virtual_start: 0x4000_0000,
        size: 0xb000_0000,
    },
    // Zone1 RAM (large block) + Zone0 RAM
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
    // GPU region (0xfb00_0000-0xfb20_0000) moved to zone1
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
    0x27, // arm-pmu
    0x69, // dmc
    0x2d, // timer
    0x2e, // timer
    0x2b, // timer
    0x2a, // timer
    // GPU IRQs moved to zone1:
    // 0x7e, // gpu@fb000000
    // 0x7d, // gpu@fb000000
    // 0x7c, // gpu@fb000000
    0xfc,  // usb@fc000000
    0xf7,  // usb@fc800000
    0xf8,  // usb@fc840000
    0xfa,  // usb@fc880000
    0xfb,  // usb@fc8c0000
    0x191, // iommu@fc900000
    0x193, // iommu@fc900000
    0x196, // iommu@fc900000
    0x18f, // iommu@fc900000
    0x19d, // iommu@fcb00000
    0x19f, // iommu@fcb00000
    0x1a2, // iommu@fcb00000
    0x19b, // iommu@fcb00000
    0xfe,  // usb@fcd00000
    0x1a9, // usb2-phy@0
    0x1a7, // usb2-phy@8000
    0x1a8, // usb2-phy@c000
    0x15d, // i2c@fd880000
    0x16b, // serial@fd890000
    0x178, // pwm@fd8b0000
    0x178, // pwm@fd8b0010
    0x178, // pwm@fd8b0020
    0x178, // pwm@fd8b0030
    0x179, // pwm@fd8b0030
    // NPU IRQs moved to zone1:
    // 0x8e, // npu@fdab0000
    // 0x8f, // npu@fdab0000
    // 0x90, // npu@fdab0000
    // 0x8e, // iommu@fdab9000
    // 0x8f, // iommu@fdab9000
    // 0x90, // iommu@fdab9000
    0x98, // vepu@fdb50000
    0x97, // vdpu@fdb50400
    0x96, // iommu@fdb50800
    0x97, // avsd-plus@fdb51000
    0x92, // rga@fdb60000
    0x92, // iommu@fdb60f00
    0x93, // rga@fdb70000
    0x93, // iommu@fdb70f00
    0x94, // rga@fdb80000
    0xa1, // jpegd@fdb90000
    0xa2, // iommu@fdb90480
    0x9a, // jpege-core@fdba0000
    0x99, // iommu@fdba0800
    0x9c, // jpege-core@fdba4000
    0x9b, // iommu@fdba4800
    0x9e, // jpege-core@fdba8000
    0x9d, // iommu@fdba8800
    0xa0, // jpege-core@fdbac000
    0x9f, // iommu@fdbac800
    0x95, // iep@fdbb0000
    0x95, // iommu@fdbb0800
    0x85, // rkvenc-core@fdbd0000
    0x83, // iommu@fdbdf000
    0x84, // iommu@fdbdf000
    0x88, // rkvenc-core@fdbe0000
    0x86, // iommu@fdbef000
    0x87, // iommu@fdbef000
    0x7f, // rkvdec-core@fdc38000
    0x80, // iommu@fdc38700
    0x81, // rkvdec-core@fdc48000
    0x82, // iommu@fdc48700
    0x8c, // av1d@fdc70000
    0x8b, // av1d@fdc70000
    0x8a, // av1d@fdc70000
    0x8d, // iommu@fdca0000
    0xa7, // rkisp-unite@fdcb0000
    0xa9, // rkisp-unite@fdcb0000
    0xaa, // rkisp-unite@fdcb0000
    0xa3, // rkisp@fdcb0000
    0xa5, // rkisp@fdcb0000
    0xa6, // rkisp@fdcb0000
    0xa4, // rkisp-unite-mmu@fdcb7f00
    0xa8, // rkisp-unite-mmu@fdcb7f00
    0xa4, // iommu@fdcb7f00
    0xa7, // rkisp@fdcc0000
    0xa9, // rkisp@fdcc0000
    0xaa, // rkisp@fdcc0000
    0xa8, // iommu@fdcc7f00
    0xab, // rkispp@fdcd0000
    0xac, // iommu@fdcd0f00
    0xad, // rkispp@fdcd8000
    0xae, // iommu@fdcd8f00
    0xbb, // rkcif@fdce0000
    0x91, // iommu@fdce0800
    0xaf, // mipi0-csi2@fdd10000
    0xb0, // mipi0-csi2@fdd10000
    0xb1, // mipi1-csi2@fdd20000
    0xb2, // mipi1-csi2@fdd20000
    0xb3, // mipi2-csi2@fdd30000
    0xb4, // mipi2-csi2@fdd30000
    0xb5, // mipi3-csi2@fdd40000
    0xb6, // mipi3-csi2@fdd40000
    // 0xbc, // vop@fdd90000 — moved to zone1
    // 0xbc, // iommu@fdd97e00 — moved to zone1
    0xe3, // spdif-tx@fddb0000
    0xd8, // i2s@fddc0000
    0xe4, // spdif-tx@fdde0000
    0xd9, // i2s@fddf0000
    0xdd, // i2s@fddfc000
    0xe7, // spdif-rx@fde08000
    0xc7, // dsi@fde20000
    0xc8, // dsi@fde30000
    0xbf, // hdcp@fde40000
    0xc1, // dp@fde50000
    0xc0, // hdcp@fde70000
    // 0xc9, // hdmi@fde80000 — moved to zone1
    // 0xca, // hdmi@fde80000 — moved to zone1
    // 0xcb, // hdmi@fde80000 — moved to zone1
    // 0xcc, // hdmi@fde80000 — moved to zone1
    // 0x188, // hdmi@fde80000 — moved to zone1
    0xc3,  // edp@fdec0000
    0x118, // pcie@fe180000
    0x117, // pcie@fe180000
    0x116, // pcie@fe180000
    0x115, // pcie@fe180000
    0x114, // pcie@fe180000
    0x115, // legacy-interrupt-controller
    0x11d, // pcie@fe190000
    0x11c, // pcie@fe190000
    0x11b, // pcie@fe190000
    0x11a, // pcie@fe190000
    0x119, // pcie@fe190000
    0x11a, // legacy-interrupt-controller
    0x10a, // ethernet@fe1c0000
    0x109, // ethernet@fe1c0000
    0x131, // sata@fe210000
    0x133, // sata@fe230000
    0xee,  // spi@fe2b0000
    0xeb,  // mmc@fe2c0000
    0xec,  // mmc@fe2d0000
    0xed,  // mmc@fe2e0000
    0xf1,  // crypto@fe370000
    0x1b0, // rng@fe378000
    0xd4,  // i2s@fe470000
    0xd5,  // i2s@fe480000
    0xd6,  // i2s@fe490000
    0xd7,  // i2s@fe4a0000
    0xea,  // vad@fe4d0000
    0xe1,  // spdif-tx@fe4e0000
    0xe2,  // spdif-tx@fe4f0000
    0x29,  // interrupt-controller@fe600000
    0x76,  // dma-controller@fea10000
    0x77,  // dma-controller@fea10000
    0x78,  // dma-controller@fea30000
    0x79,  // dma-controller@fea30000
    0x175, // can@fea50000
    0x176, // can@fea60000
    0x177, // can@fea70000
    0x75,  // decompress@fea80000
    0x15e, // i2c@fea90000
    0x15f, // i2c@feaa0000
    0x160, // i2c@feab0000
    0x161, // i2c@feac0000
    0x162, // i2c@fead0000
    0x141, // timer@feae0000
    0x15b, // watchdog@feaf0000
    0x166, // spi@feb00000
    0x167, // spi@feb10000
    0x168, // spi@feb20000
    0x169, // spi@feb30000
    0x16c, // serial@feb40000
    0x16d, // serial@feb50000
    0x16e, // serial@feb60000
    0x16f, // serial@feb70000
    0x170, // serial@feb80000
    0x171, // serial@feb90000
    0x172, // serial@feba0000
    0x173, // serial@febb0000
    0x174, // serial@febc0000
    0x17a, // pwm@febd0000
    0x17a, // pwm@febd0010
    0x17a, // pwm@febd0020
    0x17a, // pwm@febd0030
    0x17b, // pwm@febd0030
    0x17c, // pwm@febe0000
    0x17c, // pwm@febe0010
    0x17c, // pwm@febe0020
    0x17c, // pwm@febe0030
    0x17d, // pwm@febe0030
    0x17e, // pwm@febf0000
    0x17e, // pwm@febf0010
    0x17e, // pwm@febf0020
    0x17e, // pwm@febf0030
    0x17f, // pwm@febf0030
    0x1ad, // tsadc@fec00000
    0x1ae, // saradc@fec10000
    0x5d,  // mailbox@fec60000
    0x5e,  // mailbox@fec60000
    0x5f,  // mailbox@fec60000
    0x60,  // mailbox@fec60000
    0x65,  // mailbox@fec70000
    0x66,  // mailbox@fec70000
    0x67,  // mailbox@fec70000
    0x68,  // mailbox@fec70000
    0x163, // i2c@fec80000
    0x164, // i2c@fec90000
    0x165, // i2c@feca0000
    0x16a, // spi@fecb0000
    0x6d,  // mailbox@fece0000
    0x6e,  // mailbox@fece0000
    0x6f,  // mailbox@fece0000
    0x70,  // mailbox@fece0000
    0x7a,  // dma-controller@fed10000
    0x7b,  // dma-controller@fed10000
    0x135, // gpio@fd8a0000
    0x136, // gpio@fec20000
    0x137, // gpio@fec30000
    0x138, // gpio@fec40000
    0x139, // gpio@fec50000
    0xfd,  // usb@fc400000
    0x1aa, // usb2-phy@4000
    0xb7,  // mipi4-csi2@fdd50000
    0xb8,  // mipi4-csi2@fdd50000
    0xb9,  // mipi5-csi2@fdd60000
    0xba,  // mipi5-csi2@fdd60000
    0xe6,  // spdif-tx@fddb8000
    0xdc,  // i2s@fddc8000
    0xe5,  // spdif-tx@fdde8000
    0xda,  // i2s@fddf4000
    0xdb,  // i2s@fddf8000
    0xde,  // i2s@fde00000
    0xe8,  // spdif-rx@fde10000
    0xe9,  // spdif-rx@fde18000
    0xc2,  // dp@fde60000
    // 0xcd, // hdmi@fdea0000 — moved to zone1
    // 0xce, // hdmi@fdea0000 — moved to zone1
    // 0xcf, // hdmi@fdea0000 — moved to zone1
    // 0xd0, // hdmi@fdea0000 — moved to zone1
    // 0x189, // hdmi@fdea0000 — moved to zone1
    0xc4,  // edp@fded0000
    0xd1,  // hdmirx-controller@fdee0000
    0x1d4, // hdmirx-controller@fdee0000
    0xd3,  // hdmirx-controller@fdee0000
    0x127, // pcie@fe150000
    0x126, // pcie@fe150000
    0x125, // pcie@fe150000
    0x124, // pcie@fe150000
    0x123, // pcie@fe150000
    0x124, // legacy-interrupt-controller
    0x122, // pcie@fe160000
    0x121, // pcie@fe160000
    0x120, // pcie@fe160000
    0x11f, // pcie@fe160000
    0x11e, // pcie@fe160000
    0x11f, // legacy-interrupt-controller
    0x113, // pcie@fe170000
    0x112, // pcie@fe170000
    0x111, // pcie@fe170000
    0x110, // pcie@fe170000
    0x10f, // pcie@fe170000
    0x110, // legacy-interrupt-controller
    0x103, // ethernet@fe1b0000
    0x102, // ethernet@fe1b0000
    0x132, // sata@fe220000
    0x1c7, // fiq-debugger
    0x40,  // hvisor_virtio_device
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
