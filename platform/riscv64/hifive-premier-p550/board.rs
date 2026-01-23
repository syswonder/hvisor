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
pub const BOARD_NAME: &str = "hifive-premier-p550";

pub const BOARD_NCPUS: usize = 4;
#[rustfmt::skip]
pub static BOARD_HARTID_MAP: [usize; BOARD_NCPUS] = [
    0x0,            // core0   \
    0x1,            // core1    | -> cluster0 -> CPU
    0x2,            // core2    |
    0x3,            // core3   / 
];
pub const TIMEBASE_FREQ: u64 = 0xf4240; // 1MHz

pub const PLIC_BASE: usize = 0xc000000;
pub const PLIC_SIZE: usize = 0x4000000;
pub const BOARD_PLIC_INTERRUPTS_NUM: usize = 1023; // except irq 0
pub const NUM_CONTEXTS_PER_HART: usize = 2; // M-mode、S-mode

pub const SIFIVE_CCACHE_BASE: usize = 0x2010000; // SiFive composable cache controller
pub const SIFIVE_CCACHE_SIZE: usize = 0x4000; // 16KB

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = 0x1;

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 7] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x80000000,
        virtual_start: 0x80000000,
        size: 0x8000_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x50900000,
        virtual_start: 0x50900000,
        size: 0x10000,
    }, // serial0
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x51810000,
    //     virtual_start: 0x51810000,
    //     size: 0x8000,
    // }, // scu
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x50c00000,
    //     virtual_start: 0x50c00000,
    //     size: 0x100000,
    // }, // iommu
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x51c00000,
    //     virtual_start: 0x51c00000,
    //     size: 0x400000,
    // }, // npu
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x51c00000,
    //     virtual_start: 0x51c00000,
    //     size: 0x400000,
    // }, // npu
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x50920000,
    //     virtual_start: 0x50920000,
    //     size: 0x10000,
    // }, // serial2
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
    }, // L3 cache-controller, now hvisor has virtual sifive ccache.
    // HvConfigMemoryRegion {
    //     mem_type: MEM_TYPE_IO,
    //     physical_start: 0x8000000,
    //     virtual_start: 0x8000000,
    //     size: 0x400000,
    // }, // L3 Loosely-Integrated Memory (L3 LIM)
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xc0_0000_0000,
        virtual_start: 0xc0_0000_0000,
        // mem-port -> sys-port (here easily equal to mem size), this is needed for uncache access in eic7700x soc.
        size: 0x4_0000_0000,
    }, // Sys-port. (here related to DMA)
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start:  0x104000,
       //     virtual_start: 0x104000,
       //     size: 0x4000,
       // }, // pL2Cache. (cpu0)
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start:  0x108000,
       //     virtual_start: 0x108000,
       //     size: 0x4000,
       // }, // pL2Cache. (cpu1)
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start: 0x50420000,
       //     virtual_start: 0x50420000,
       //     size: 0x10000,
       // }, // sata
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start: 0x51600000,
       //     virtual_start: 0x51600000,
       //     size: 0x200000,
       // }, // pinctrl
];

// Note: all here's irqs are hardware irqs,
//  only these irq can be transferred to the physical PLIC.
pub const HW_IRQS: [u32; 21] = [
    0x1, 0x2, 0x3, 0x4, // cache controller
    0x4f, // emmc
    0x51, // sd-card
    0x64, // uart0
    0x164, 0x168, 0x165, 0x166, // iommu
    0x183, // npu
    0x75, 0x77, 0x79, 0x7b, 0x7d, 0x7f, 0x81, 0x83, // mailbox
    0x123, // i2c
    // 0x01, 0x03, 0x04, 0x02, // cache controller
];

// irqs belong to the root zone.
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    0x1, 0x2, 0x3, 0x4, // cache controller
    0x51, // sd-card
    0x64, // uart0
    0x164, 0x168, 0x165, 0x166, // iommu
    0x183, // npu
    0x75, 0x77, 0x79, 0x7b, 0x7d, 0x7f, 0x81, 0x83, // mailbox
    0x123, // i2c
]);

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: PLIC_BASE,
    plic_size: PLIC_SIZE,
    aplic_base: 0x0,
    aplic_size: 0x0,
};

// Virtio zone1 cmd:
//      nohup ./hvisor virtio start zone1-linux-virtio.json &
//      ./hvisor zone start zone1-linux.json
