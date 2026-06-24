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
    arch::zone::HvArchZoneConfig, config::*, memory::GuestPhysAddr, pci::vpci_dev::VpciDevType,
};

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
    size: 0xf000,
};
const ROOT_ZONE_ACPI_REGION_ID: usize = 0x6;

const ROOT_ZONE_UEFI_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x1a00_0000,
    virtual_start: 0x1500_0000,
    size: 0x1_0000,
};
const ROOT_ZONE_UEFI_REGION_ID: usize = 0x3;

pub const ROOT_ZONE_NAME: &str = "asterinas";
// Asterinas reads its console and init configuration from the Linux command
// line that the hypervisor places at ROOT_ZONE_CMDLINE_ADDR.
pub const ROOT_ZONE_CMDLINE: &str = "console=ttyS0 ostd.log_level=info init=/init\0";

// The protected-mode kernel is relocated to 0x10_0000 and the initramfs to
// ROOT_ZONE_INITRD_ADDR; both fall inside the two RAM windows below. grub
// decompresses the gzip module, so the size must cover the expanded cpio
// archive (the kernel stops at its TRAILER record).
const ROOT_ZONE_INITRD_ADDR: GuestPhysAddr = 0x1530_0000;
const ROOT_ZONE_INITRD_SIZE: usize = 0x30_0000;

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x500_0000,
        virtual_start: 0x0,
        size: 0xe_0000,
    }, // low ram: boot stack, cmdline, setup/zeropage
    ROOT_ZONE_RSDP_REGION, // rsdp
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x510_0000,
        virtual_start: 0x10_0000,
        size: 0x14f0_0000,
    }, // main ram: protected-mode kernel entry at 0x10_0000
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
    }, // ram: initramfs
    ROOT_ZONE_ACPI_REGION, // acpi
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfed0_0000,
        virtual_start: 0xfed0_0000,
        size: 0x1000,
    }, // hpet
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfe00_0000,
        virtual_start: 0xfe00_0000,
        size: 0xc0_0000,
    }, // pci mmio window: passthrough device BARs (up to the ioapic at 0xfec00000)
];

const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0x9000;
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xa000;
const ROOT_ZONE_VMLINUX_ENTRY_ADDR: GuestPhysAddr = 0x10_0000;

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x6;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[0; 32]);
pub const ROOT_ZONE_IOAPIC_BASE: usize = 0xfec0_0000;
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    ioapic_base: ROOT_ZONE_IOAPIC_BASE,
    ioapic_size: 0x1000,
    kernel_entry_gpa: ROOT_ZONE_VMLINUX_ENTRY_ADDR,
    cmdline_load_gpa: ROOT_ZONE_CMDLINE_ADDR,
    setup_load_gpa: ROOT_ZONE_SETUP_ADDR,
    initrd_load_gpa: ROOT_ZONE_INITRD_ADDR,
    initrd_size: ROOT_ZONE_INITRD_SIZE,
    rsdp_memory_region_id: ROOT_ZONE_RSDP_REGION_ID,
    acpi_memory_region_id: ROOT_ZONE_ACPI_REGION_ID,
    uefi_memory_region_id: ROOT_ZONE_UEFI_REGION_ID,
    // A framebuffer console is not used; Asterinas logs over ttyS0.
    screen_base: 0,
};

pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    // Buses 0..=1: ecam_size covers (bus_range_end + 1) buses at 0x100000 each.
    bus_range_begin: 0x0,
    bus_range_end: 0x1,
    // Must match the ECAM base the firmware advertises in the ACPI MCFG table,
    // which the guest uses to reach PCI configuration space over MMIO.
    ecam_base: 0xb0000000,
    ecam_size: 0x200000,
    io_base: 0x0,
    io_size: 0x0,
    pci_io_base: 0x0,
    mem32_base: 0x0,
    mem32_size: 0x0,
    pci_mem32_base: 0x0,
    mem64_base: 0x0,
    mem64_size: 0x0,
    pci_mem64_base: 0x0,
    domain: 0x0,
}];

pub const ROOT_PCI_MAX_BUS: usize = 1;
// The devices the q35 machine presents with `-nodefaults -device intel-iommu`.
pub const ROOT_PCI_DEVS: [HvPciDevConfig; 4] = [
    pci_dev!(0x0, 0x0, 0x0, 0x0 => 0x0, 0x0, 0x0, VpciDevType::Physical), // host bridge
    pci_dev!(0x0, 0x0, 0x1f, 0x0 => 0x0, 0x1f, 0x0, VpciDevType::Physical), // ISA bridge
    pci_dev!(0x0, 0x0, 0x1f, 0x2 => 0x0, 0x1f, 0x2, VpciDevType::Physical), // SATA controller
    pci_dev!(0x0, 0x0, 0x1f, 0x3 => 0x0, 0x1f, 0x3, VpciDevType::Physical), // SMBus
];

#[cfg(feature = "graphics")]
pub const GRAPHICS_FONT: &[u8] =
    include_bytes!("../../platform/x86_64/qemu/image/font/spleen-6x12.psf");
