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
use crate::{arch::zone::HvArchZoneConfig, config::*, memory::GuestPhysAddr, pci::vpci_dev::VpciDevType};

pub const MEM_TYPE_RESERVED: u32 = 5;

pub const BOARD_NCPUS: usize = 4;

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
const ROOT_ZONE_RSDP_REGION_ID: usize = 0x1;

const ROOT_ZONE_ACPI_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x3a30_0000, // hpa
    virtual_start: 0x3530_0000,  // gpa
    size: 0x10_0000,             // modify size accordingly
};
const ROOT_ZONE_ACPI_REGION_ID: usize = 0x6;

const ROOT_ZONE_UEFI_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x1a00_0000,
    virtual_start: 0x1500_0000,
    size: 0x1_0000,
};
const ROOT_ZONE_UEFI_REGION_ID: usize = 0x3;

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_CMDLINE: &str = "video=vesafb console=tty0 nointremap no_timer_check efi=noruntime pci=pcie_scan_all root=/dev/sda2 rw init=/init rootwait\0";
// pub const ROOT_ZONE_CMDLINE: &str = "video=vesafb console=ttyS0 earlyprintk=serial nointremap no_timer_check pci=pcie_scan_all root=/dev/vda rw init=/init\0";
//"console=ttyS0 earlyprintk=serial rdinit=/init nokaslr nointremap\0"; // noapic
// video=vesafb

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 15] = [
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
    ROOT_ZONE_UEFI_REGION, // uefi
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1a01_0000,
        virtual_start: 0x1501_0000,
        size: 0x2f_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x1a30_0000,
        virtual_start: 0x1530_0000,
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
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0x1_0000_0000,
        virtual_start: 0x1_0000_0000,
        size: 0x2000_0000,
    }, // zone 1
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0x6ed7_f000,
        virtual_start: 0x6ed7_f000,
        size: 0x10_e000,
    }, // ACPI non-volatile storage
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0xfeda_0000,
        virtual_start: 0xfeda_0000,
        size: 0x2_8000,
    }, // pnp 00:05
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0xfe01_1000,
        virtual_start: 0xfe01_1000,
        size: 0x40_0000,
    }, // reserved
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0x677a_b000,
        virtual_start: 0x677a_b000,
        size: 0x74d_3000,
    }, // reserved
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0xfd69_0000,
        virtual_start: 0xfd69_0000,
        size: 0x6_0000,
    }, // INTC1057:00
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RESERVED,
        physical_start: 0xfb00_0000,
        virtual_start: 0xfb00_0000,
        size: 0x100_0000,
    }, // reserved
];

const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0x9000;
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xa000;
const ROOT_ZONE_VMLINUX_ENTRY_ADDR: GuestPhysAddr = 0x10_0000;
const ROOT_ZONE_SCREEN_BASE_ADDR: GuestPhysAddr = 0x8000_0000;

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x6;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[0; 32]);
pub const ROOT_ZONE_IOAPIC_BASE: usize = 0xfec0_0000;
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    ioapic_base: ROOT_ZONE_IOAPIC_BASE,
    ioapic_size: 0x1000,
    kernel_entry_gpa: ROOT_ZONE_VMLINUX_ENTRY_ADDR,
    cmdline_load_gpa: ROOT_ZONE_CMDLINE_ADDR,
    setup_load_gpa: ROOT_ZONE_SETUP_ADDR,
    initrd_load_gpa: 0, // 0x1500_0000,
    initrd_size: 0,     // 0x26_b000,
    rsdp_memory_region_id: ROOT_ZONE_RSDP_REGION_ID,
    acpi_memory_region_id: ROOT_ZONE_ACPI_REGION_ID,
    uefi_memory_region_id: ROOT_ZONE_UEFI_REGION_ID,
    // not longer than 32 bits
    screen_base: ROOT_ZONE_SCREEN_BASE_ADDR,
};

pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    ecam_base: 0xc0000000,
    ecam_size: 0x300000,
    io_base: 0x0,
    io_size: 0x0,
    pci_io_base: 0x0,
    mem32_base: 0x0,
    mem32_size: 0x0,
    pci_mem32_base: 0x0,
    mem64_base: 0x0,
    mem64_size: 0x0,
    pci_mem64_base: 0x0,
}];

pub const ROOT_PCI_MAX_BUS: usize = 2;
pub const ROOT_PCI_DEVS: [HvPciDevConfig; 18] = [
    pci_dev!(0x0, 0x0, 0x0, VpciDevType::Physical), // host bridge
    pci_dev!(0x0, 0x2, 0x0, VpciDevType::Physical), // VGA controller
    pci_dev!(0x0, 0x4, 0x0, VpciDevType::Physical),
    pci_dev!(0x0, 0x8, 0x0, VpciDevType::Physical),
    pci_dev!(0x0, 0xa, 0x0, VpciDevType::Physical),
    // pci_dev!(0x0, 0xd, 0x0), // USB controller
    pci_dev!(0x0, 0x12, 0x0, VpciDevType::Physical), // serial controller
    pci_dev!(0x0, 0x14, 0x0, VpciDevType::Physical), // USB controller
    pci_dev!(0x0, 0x14, 0x2, VpciDevType::Physical), // RAM memory
    pci_dev!(0x0, 0x14, 0x3, VpciDevType::Physical), // network controller
    pci_dev!(0x0, 0x16, 0x0, VpciDevType::Physical), // communication controller
    pci_dev!(0x0, 0x1c, 0x0, VpciDevType::Physical), // PCI bridge
    pci_dev!(0x0, 0x1d, 0x0, VpciDevType::Physical), // PCI bridge
    pci_dev!(0x0, 0x1f, 0x0, VpciDevType::Physical), // ISA bridge
    pci_dev!(0x0, 0x1f, 0x3, VpciDevType::Physical), // audio controller
    pci_dev!(0x0, 0x1f, 0x4, VpciDevType::Physical), // SMBus
    pci_dev!(0x0, 0x1f, 0x5, VpciDevType::Physical), // serial bus controller
    pci_dev!(0x1, 0x0, 0x0, VpciDevType::Physical),  // ethernet controller
    pci_dev!(0x2, 0x0, 0x0, VpciDevType::Physical),  // memory controller
];

#[cfg(all(feature = "graphics"))]
pub const GRAPHICS_FONT: &[u8] =
    include_bytes!("../../platform/x86_64/qemu/image/font/spleen-6x12.psf");
