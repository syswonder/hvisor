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
//      Yulong Han <wheatfox17@icloud.com>
//      Ming Shen  <boneinscri@163.com>
use crate::pci_dev;
use crate::{arch::zone::HvArchZoneConfig, config::*, pci::vpci_dev::VpciDevType};

// ------------------------------------------------------------------------
// Memory layout assumptions for LoongArch 3A6000 + LS7A2000 evaluation board
// ------------------------------------------------------------------------
//
// The 3A6000 / LS7A2000 board exposes physical RAM through a non-contiguous
// PA layout — RAM size != PA span. Always cross-reference `/proc/iomem` on
// the running Linux to confirm before touching this file.
//
// ## Verified layout on 16 GiB dev board (this code's default target)
//
// ```
// 0x00200000..0x0edfffff   ~236 MB    low RAM         (kernel image, early init)
// 0x90400000..0xf8faffff   ~1.65 GB   mid RAM lo      (ACPI, EFI runtime, SMBIOS)
// 0xf8fc0000..0xfdbaffff   ~78 MB     mid RAM hi      (EFI runtime continuation)
// 0xfe190000..0x47fffffff  ~14.5 GB   hi RAM          (bulk RAM, PA top = 18 GiB)
// ```
//
// Total RAM = ~16.46 GiB nominal. PA TOP = `0x480000000` (nominal 18 GiB) on
// 16 GiB board because low/mid eat ~2 GiB of PA space.
//
// ## Adapting to other RAM sizes
//
// Only `BOARD_TOTAL_RAM_GIB` should need editing for a different memory size
// on the SAME 3A6000+LS7A2000 board family. Numbers below are based on the
// "low+mid PA ≈ 2 GiB constant" empirical assumption — verify with
// /proc/iomem before trusting.
//
// | RAM   | PA top (hi RAM end)   | hi RAM span (above 4 GiB) |
// |-------|------------------------|----------------------------|
// |  8 GiB | `0x280000000` ≈ 10 GiB |  ~6.0 GiB (TENTATIVE)     |
// | 16 GiB | `0x480000000` ≈ 18 GiB | ~14.0 GiB (verified)       |
// | 32 GiB | `0x880000000` ≈ 34 GiB | ~30.0 GiB (TENTATIVE)      |
//
// Different motherboards (e.g. desktop vs server, different SPD timings, BIOS
// version) may shift PA top — re-measure if /proc/iomem disagrees.
//
// ## hvisor footprint coordination
//
// Three files must agree on where hvisor lives + how big it is:
//   1. `platform/loongarch64/ls3a6000/linker.ld`  → BASE_ADDRESS
//   2. this file `HVISOR_LOAD_PA` + `HVISOR_FOOTPRINT_SIZE`
//   3. `/etc/grub.d/09_loongvisor` template's argv[2] passed to loongstub
//   4. `grub2-hvisor/grub-core/loader/loongarch64/loongstub.c` extra-reserve
//
// Keep all four in sync. The hi RAM region below is auto-split around
// `HVISOR_LOAD_PA..HVISOR_FOOTPRINT_END` so guest Linux's stage-2 can never
// see hvisor's own memory as RAM.

/// Total system RAM in GiB. Change ONLY this and rebuild — the rest of the
/// hi-RAM layout in this file derives from it (best-effort heuristic; verify
/// `/proc/iomem` if you change to non-16).
pub const BOARD_TOTAL_RAM_GIB: u64 = 32;

/// Top of high RAM physical address (one byte past the last). Derived from
/// `BOARD_TOTAL_RAM_GIB` via the empirical mapping in the table above. If a
/// future 3A6000 board has a different PA top, override here directly.
pub const BOARD_HI_RAM_TOP: u64 = match BOARD_TOTAL_RAM_GIB {
    8  => 0x280000000,  // tentative — verify on 8 GiB board
    16 => 0x480000000,  // verified on dev board (16 GiB)
    32 => 0x880000000,  // tentative — verify on 32 GiB board
    _  => panic!("unsupported BOARD_TOTAL_RAM_GIB; add an entry to the match"),
};

/// PA where hvisor itself is loaded. MUST match linker.ld `BASE_ADDRESS`
/// (low 48 bits) and grub.cfg `hvisor_loongarch ... argv[1]`.
pub const HVISOR_LOAD_PA: u64 = 0x380000000;  // 14 GiB

/// Reserved PA span owned by hvisor (text + bss + percpu + heap + frame pool).
/// MUST be ≥ actual hvisor build's MEM_SIZE (printed at end of hvisor `make`).
/// MUST match the value loongstub uses for EFI memory reservation, otherwise
/// EFI may place Boot Services Data into this range and hvisor's runtime
/// allocator will then trample EFI structures → DxeCore asserts in
/// `Locate.c`. 768 MB covers our current ~705 MB MEM_SIZE with headroom.
pub const HVISOR_FOOTPRINT_SIZE: u64 = 0x30000000;  // 768 MB

/// One-past-end of the reserved hvisor PA region.
pub const HVISOR_FOOTPRINT_END: u64 = HVISOR_LOAD_PA + HVISOR_FOOTPRINT_SIZE;


pub const BOARD_NAME: &str = "ls3a6000";

pub const BOARD_NCPUS: usize = 8;

pub const CPU_BOOT_CONTEXT_ADDRESS: usize = 0x90000001e0000000;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x10000f000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x200000;
pub const ROOT_ZONE_ENTRY: u64 = 0xe71000;
// pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7);
// pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) ;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6);

pub const ROOT_ZONE_NAME: &str = "root-linux-la64";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
     HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0,
        virtual_start:  0x0,
        size: 0x200000,
    }, // ????

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x200000,
        virtual_start:  0x200000,
        size: 0xec00000,
    }, // RAM

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe00000,
        virtual_start:  0x1fe00000,
        size: 0x2000,
    }, // IO important MMIO

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x1fe10000,
        virtual_start:  0x1fe10000,
        size: 0x2000,
    }, // IO important MMIO
    
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10080000,
        virtual_start:  0x10080000,
        size: 0x1000,
    }, // serial

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10090000,
        virtual_start:  0x10090000,
        size: 0x1000,
    }, // IO 

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x18000000,
        virtual_start:  0x18000000,
        size: 0x1000,
    }, // IO???? unknown

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100a0000,
        virtual_start:  0x100a0000,
        size: 0x1000,
    }, // IO 

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100d0000,
        virtual_start:  0x100d0000,
        size: 0x1000,
    }, // IO 

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x100e0000,
        virtual_start:  0x100e0000,
        size: 0x1000,
    }, // IO

    // ====== for start_image ======
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x90000000,
        virtual_start:  0x90000000,
        size: 0x6a000000,
    }, // RAM 0x90000000 ~ 0xf9ffffff

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0xfa000000,
        virtual_start:  0xfa000000,
        size: 0x5000000,
    }, // RAM 0xfa000000 ~ 0xfeffffff
    // ==== for start_image ====

    // addition
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start:  0x10000000,
        size: 0x1000,
    }, // IO PCH-PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10002000,
        virtual_start:  0x10002000,
        size: 0x1000,
    }, // IO

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10010000,
        virtual_start:  0x10010000,
        size: 0x1000,
    }, // IO

    // // 0x100000000 ~ 0x87fffffff （30GB)
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_RAM,
    //     physical_start: 0x100000000,
    //     virtual_start:  0x100000000,
    //     size: 0x780000000,
    // }, // RAM

    // === hi RAM (PA 0x100000000 .. BOARD_HI_RAM_TOP) ===
    //
    // Split into two segments around hvisor's own footprint so guest Linux's
    // stage-2 never sees hvisor's runtime pages as plain RAM. The hvisor PA
    // window [HVISOR_LOAD_PA, HVISOR_FOOTPRINT_END) is OMITTED here on
    // purpose; loongstub also marks the same window as EfiReservedMemoryType
    // so EFI Boot Services Data won't land in it either.
    //
    // Sizes are computed from constants — if BOARD_TOTAL_RAM_GIB or the
    // hvisor location changes, both regions adjust automatically.
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x100000000,                                  // 4 GiB
        virtual_start:  0x100000000,
        size: HVISOR_LOAD_PA - 0x100000000,                           // hi RAM below hvisor
    }, // RAM (below hvisor footprint)

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: HVISOR_FOOTPRINT_END,                         // hvisor top
        virtual_start:  HVISOR_FOOTPRINT_END,
        size: BOARD_HI_RAM_TOP - HVISOR_FOOTPRINT_END,                // hi RAM above hvisor
    }, // RAM (above hvisor footprint, up to BOARD_HI_RAM_TOP)

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe0030000000,
        virtual_start:  0xe0030000000,
        size: 0x50000000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xefe00000000,
        virtual_start:  0xefe00000000,
        size: 0x20000000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xefdfe000000,
        virtual_start:  0xefdfe000000,
        size: 0x2000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xefdfc000000,
        virtual_start:  0xefdfc000000,
        size: 0x2000000,
    }, // PCI (covers 0xefdfc0003ce)

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe8000000000,
        virtual_start:  0xe8000000000,
        size: 0xc00000000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe8c15000000,
        virtual_start:  0xe8c15000000,
        size: 0x162000,
    }, // PCI

    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xe0000000000,
        virtual_start:  0xe0000000000,
        size: 0x30000000,
    }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xe8000000000,
    //     virtual_start:  0xe8000000000,
    //     size: 0x10000000,
    // }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xe8015150000,
    //     virtual_start:  0xe8015150000,
    //     size: 0x8000
    // }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0xe8015000000,
    //     virtual_start:  0xe8015000000,
    //     size: 0x100000
    // }, // PCI

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IOMMU,
    //     physical_start: 0xe0030000000,
    //     virtual_start:  0xe0030000000,
    //     size: 0x50000000,
    // }, // PCI-IOMMU

    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IOMMU,
    //     physical_start: 0xefe00000000,
    //     virtual_start:  0xefe00000000,
    //     size: 0x20000000,
    // }, // PCI-IOMMU
];

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 32 + 0x20;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[]);
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig { dummy: 0 };
pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];

// pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
//     bus_range_begin: 0x0,
//     bus_range_end: 0x1f,
//     ecam_base: 0xfe00000000,
//     ecam_size: 0x20000000,
//     io_base: 0x18408000,
//     io_size: 0x8000,
//     pci_io_base: 0x00008000,
//     mem32_base: 0x0,
//     mem32_size: 0x0,
//     pci_mem32_base: 0x0,
//     mem64_base: 0x60000000,
//     mem64_size: 0x20000000,
//     pci_mem64_base: 0x60000000,
//     domain: 0x0,
// }];
pub const ROOT_PCI_CONFIG: [HvPciConfig; 0] = [];

/* 00:00.0, 00:00.1, 00:00.2, 00:00.3, 00:04.0, 00:04.1*/
/* 00:05.0, 00:05.1, 00:06.0, 00:06.1, 00:06.2 */
/* 00:07.0, 00:08.0, 00:09.0, 00:0a.0, 00:0b.0 */
/* 00:0c.0, 00:0d.0, 00:0f.0, 00:10.0, 00:13.0 */
/* 00:16.0, 00:19.0, 02:00.0, 05:00.0 */
/* BUS 8 on X16 slot */
/* 08:00.0, 08:00.1, 08:00.2, 08:00.3 net */
/* BUS 6 on X4 slot */
/* 06:00.0, 06:00.1, 06:00.2, 06:00.3 net */
// pub const ROOT_PCI_DEVS: [HvPciDevConfig; 26] = [
//     pci_dev!(0x0, 0x0, 0x0, 0x0, VpciDevType::Physical), // 00:00.0
//     pci_dev!(0x0, 0x0, 0x0, 0x1, VpciDevType::Physical), // 00:00.1
//     pci_dev!(0x0, 0x0, 0x0, 0x2, VpciDevType::Physical), // 00:00.2
//     pci_dev!(0x0, 0x0, 0x0, 0x3, VpciDevType::Physical), // 00:00.3
//     pci_dev!(0x0, 0x0, 0x4, 0x0, VpciDevType::Physical), // 00:04.0
//     pci_dev!(0x0, 0x0, 0x4, 0x1, VpciDevType::Physical), // 00:04.1
//     pci_dev!(0x0, 0x0, 0x5, 0x0, VpciDevType::Physical), // 00:05.0
//     pci_dev!(0x0, 0x0, 0x5, 0x1, VpciDevType::Physical), // 00:05.1
//     pci_dev!(0x0, 0x0, 0x6, 0x0, VpciDevType::Physical), // 00:06.0
//     pci_dev!(0x0, 0x0, 0x6, 0x1, VpciDevType::Physical), // 00:06.1
//     pci_dev!(0x0, 0x0, 0x6, 0x2, VpciDevType::Physical), // 00:06.2
//     pci_dev!(0x0, 0x0, 0x7, 0x0, VpciDevType::Physical), // 00:07.0
//     pci_dev!(0x0, 0x0, 0x8, 0x0, VpciDevType::Physical), // 00:08.0
//     pci_dev!(0x0, 0x0, 0x9, 0x0, VpciDevType::Physical), // 00:09.0
//     pci_dev!(0x0, 0x0, 0xa, 0x0, VpciDevType::Physical), // 00:0a.0
//     pci_dev!(0x0, 0x0, 0xb, 0x0, VpciDevType::Physical), // 00:0b.0
//     pci_dev!(0x0, 0x0, 0xc, 0x0, VpciDevType::Physical), // 00:0c.0
//     pci_dev!(0x0, 0x0, 0xd, 0x0, VpciDevType::Physical), // 00:0d.0
//     pci_dev!(0x0, 0x0, 0xf, 0x0, VpciDevType::Physical), // 00:0f.0
//     pci_dev!(0x0, 0x0, 0x10, 0x0, VpciDevType::Physical), // 00:10.0
//     pci_dev!(0x0, 0x0, 0x13, 0x0, VpciDevType::Physical), // 00:13.0
//     pci_dev!(0x0, 0x0, 0x16, 0x0, VpciDevType::Physical), // 00:16.0
//     pci_dev!(0x0, 0x0, 0x19, 0x0, VpciDevType::Physical), // 00:19.0
//     pci_dev!(0x0, 0x2, 0x0, 0x0, VpciDevType::Physical), // 02:00.0
//     pci_dev!(0x0, 0x5, 0x0, 0x0, VpciDevType::Physical), // 05:00.0
//     pci_dev!(0x0, 0x6, 0x0, 0x0, VpciDevType::Physical), // 06:00.0
// ];
pub const ROOT_PCI_DEVS: [HvPciDevConfig; 0] = [];

// bus << 8 | dev << 5 | func << 3

// pub const ROOT_PCI_DEVS: [u64; 0] = [];

// "alloc_pci_devs": [0,1,2,3,32,33,40,41,56,64,72,80,88,96,104,120,128,152,176,200,512,1280,2051]
