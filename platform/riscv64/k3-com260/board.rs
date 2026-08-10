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
use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "k3-com260";
// Here only contains X100 cores, not including A100 cores.
pub const BOARD_NCPUS: usize = 8;
#[rustfmt::skip]
pub static BOARD_HARTID_MAP: [usize; BOARD_NCPUS] = [
    0x0, // X100 core0
    0x1, // X100 core1
    0x2, // X100 core2
    0x3, // X100 core3
    0x4, // X100 core4 
    0x5, // X100 core5
    0x6, // X100 core6
    0x7, // X100 core7
];
pub const TIMEBASE_FREQ: u64 = 0x16e3600; // 24MHz

pub const APLIC_S_BASE: usize = 0xe0804000;
pub const APLIC_S_SIZE: usize = 0x4000;
pub const BOARD_APLIC_INTERRUPTS_NUM: usize = 512;
pub const IMSIC_S_BASE: usize = 0xe0400000;
pub const IMSIC_GUEST_NUM: usize = 63;
pub const IMSIC_GUEST_INDEX: usize = 1;
pub const IMSIC_NUM_IDS: usize = 0x1FF;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x1_0f00_0000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x1_1000_0000;
pub const ROOT_ZONE_ENTRY: u64 = 0x1_1000_0000;
pub const ROOT_ZONE_CPUS: u64 = 0xF; // core0 ~ core3
pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: &[HvConfigMemoryRegion] = &[
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x107000000, // 0x1_0200_0000 + 0x500_0000
        virtual_start: 0x107000000,
        size: 0x100000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4017000,
        virtual_start: 0xd4017000,
        size: 0x1000,
    }, // serial@d4017000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4050000,
        virtual_start: 0xd4050000,
        size: 0x10000,
    }, // system-controller@d4050000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4015000,
        virtual_start: 0xd4015000,
        size: 0x1000,
    }, // system-controller@d4015000  
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4090000,
        virtual_start: 0xd4090000,
        size: 0x10000,
    }, // clock-controller@d4090000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd401e000,
        virtual_start: 0xd401e000,
        size: 0x1000,
    }, // pinctrl@d401e000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4282000,
        virtual_start: 0xd4282000,
        size: 0x1000,
    }, // system-controller@d4282800
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xc0e00000,
        virtual_start: 0xc0e00000,
        size: 0x40000,
    }, // ufshc@0xc0e00000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4019000,
        virtual_start: 0xd4019000,
        size: 0x1000,
    }, // gpio@d4019000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xd4280000,
        virtual_start: 0xd4280000,
        size: 0x1000,
    }, // mmc@d4280000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xcac82000,
        virtual_start: 0xcac82000,
        size: 0x2000,
    }, // ethernet@cac82000
];

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x20;
pub const HW_IRQS: &[u32] = &[
    42, // serial@d4017000
    58, // gpio@d4019000
    60, // pinctrl@d401e000
    99, // mmc@d4280000
    135, // ufshc@0xc0e00000
    133, 277, // ethernet@cac82000
];
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[
    42, // serial@d4017000
    58, // gpio@d4019000
    60, // pinctrl@d401e000
    99, // mmc@d4280000
    135, // ufshc@0xc0e00000
    133, 277, // ethernet@cac82000
]);

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0x0,
    plic_size: 0x0,
    aplic_base: APLIC_S_BASE,
    aplic_size: APLIC_S_SIZE,
};

pub const ROOT_PCI_CONFIG: [HvPciConfig; 0] = [];

pub const ROOT_ZONE_IVC_CONFIG: &[HvIvcConfig] = &[];

pub const ROOT_PCI_DEVS: &[HvPciDevConfig] = &[];
