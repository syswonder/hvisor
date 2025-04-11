use cortex_a::registers::DAIF::D;

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
use crate::{arch::zone::HvArchZoneConfig, config::*, memory::GuestPhysAddr};

pub const MEM_TYPE_OTHER_ZONES: u32 = 5;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x00000000;
pub const ROOT_ZONE_BOOT_STACK: GuestPhysAddr = 0x7000;
pub const ROOT_ZONE_ENTRY: u64 = 0x8000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x500_0000; // hpa
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

const ROOT_ZONE_RSDP_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x50e_0000,
    virtual_start: 0xe_0000,
    size: 0x2_0000,
};

const ROOT_ZONE_ACPI_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x4020_0000, // hpa
    virtual_start: 0x3520_0000,  // gpa
    size: 0xf000,                // modify size accordingly
};

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_CMDLINE: &str =
    "console=ttyS0 earlyprintk=serial nointremap root=/dev/vda rw init=/init\0";
//"console=ttyS0 earlyprintk=serial rdinit=/init nokaslr nointremap\0"; // noapic

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 8] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x500_0000,
        virtual_start: 0x0,
        size: 0xe_0000,
    }, // ram
    ROOT_ZONE_RSDP_REGION, // rsdp
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x510_0000,
        virtual_start: 0x10_0000,
        size: 0x14f0_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2000_0000,
        virtual_start: 0x1500_0000,
        size: 0x20_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2020_0000,
        virtual_start: 0x1520_0000,
        size: 0x2000_0000,
    }, // ram
    ROOT_ZONE_ACPI_REGION, // acpi
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfed0_0000,
        virtual_start: 0xfed0_0000,
        size: 0x1000,
    }, // hpet
    // TODO: e820 mem space probe
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_OTHER_ZONES,
        physical_start: 0x4030_0000,
        virtual_start: 0x4030_0000,
        size: 0x2000_0000,
    }, // zone 1
];

const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0xc000;
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xd000;
const ROOT_ZONE_VMLINUX_ENTRY_ADDR: GuestPhysAddr = 0x10_0000;

pub const ROOT_ZONE_IRQS: [u32; 32] = [0; 32];
pub const ROOT_ZONE_IOAPIC_BASE: usize = 0xfec0_0000;
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    ioapic_base: ROOT_ZONE_IOAPIC_BASE,
    ioapic_size: 0x1000,
    kernel_entry_gpa: ROOT_ZONE_VMLINUX_ENTRY_ADDR,
    cmdline_load_gpa: ROOT_ZONE_CMDLINE_ADDR,
    setup_load_gpa: ROOT_ZONE_SETUP_ADDR,
    initrd_load_gpa: 0x0,
    initrd_size: 0x0,
    rsdp_memory_region_id: 0x1,
    acpi_memory_region_id: 0x5,
    initrd_memory_region_id: 0x0,
};

pub const ROOT_PCI_DEVS: [u64; 7] = [0x0, 0x8, 0x18, 0xf8, 0xfa, 0xfb, 0x100]; // 0x10,
