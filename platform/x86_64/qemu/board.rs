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
    size: 0xf000,                // modify size accordingly
};
const ROOT_ZONE_ACPI_REGION_ID: usize = 0x6;

const ROOT_ZONE_UEFI_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x1a00_0000,
    virtual_start: 0x1500_0000,
    size: 0x1_0000,
};
const ROOT_ZONE_UEFI_REGION_ID: usize = 0x3;

#[cfg(not(feature = "asterinas"))]
pub const ROOT_ZONE_NAME: &str = "root-linux";
#[cfg(feature = "asterinas")]
pub const ROOT_ZONE_NAME: &str = "root-asterinas";

#[cfg(not(feature = "asterinas"))]
pub const ROOT_ZONE_CMDLINE: &str =
    "console=tty0 console=ttyS0 earlycon=efifb earlyprintk=serial nointremap no_timer_check efi=noruntime pci=pcie_scan_all,lastbus=1 root=/dev/vda rw init=/init\0";
// Asterinas parses the part before "--" as kernel arguments and the part after
// it as the init process argv (see the OSDK Linux boot protocol).
#[cfg(feature = "asterinas")]
pub const ROOT_ZONE_CMDLINE: &str =
    "SHELL=/bin/sh LOGNAME=root HOME=/ USER=root PATH=/bin ostd.log_level=info console=ttyS0 -- /init\0";

const ROOT_ZONE_RAM_LOW: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x500_0000,
    virtual_start: 0x0,
    size: 0xe_0000,
};
const ROOT_ZONE_RAM_MAIN: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x510_0000,
    virtual_start: 0x10_0000,
    size: 0x14f0_0000,
};
const ROOT_ZONE_RAM_AUX: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x1a01_0000,
    virtual_start: 0x1501_0000,
    size: 0x2f_0000,
};
const ROOT_ZONE_RAM_HIGH: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x1a30_0000,
    virtual_start: 0x1530_0000,
    size: 0x2000_0000,
};
const ROOT_ZONE_HPET: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_IO,
    physical_start: 0xfed0_0000,
    virtual_start: 0xfed0_0000,
    size: 0x1000,
};
const ROOT_ZONE_ZONE1_LOW: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RESERVED,
    physical_start: 0x4030_0000,
    virtual_start: 0x4030_0000,
    size: 0x2000_0000,
};
const ROOT_ZONE_ZONE1_HIGH: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RESERVED,
    physical_start: 0x1_0000_0000,
    virtual_start: 0x1_0000_0000,
    size: 0x7000_0000,
};

// PCI 32-bit MMIO window (device BARs) just below the I/O APIC. Linux programs
// device BARs through the legacy config ports, so hvisor maps each BAR lazily
// when it sees the write. Asterinas instead consumes the firmware-assigned BARs
// it reads over ECAM and never re-programs them, so the window is mapped through
// up front as identity-mapped device memory.
#[cfg(feature = "asterinas")]
const ROOT_ZONE_PCI_MMIO: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_IO,
    physical_start: 0xfe00_0000,
    virtual_start: 0xfe00_0000,
    size: 0xc0_0000,
};

#[cfg(not(feature = "asterinas"))]
pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 10] = [
    ROOT_ZONE_RAM_LOW,
    ROOT_ZONE_RSDP_REGION,
    ROOT_ZONE_RAM_MAIN,
    ROOT_ZONE_UEFI_REGION,
    ROOT_ZONE_RAM_AUX,
    ROOT_ZONE_RAM_HIGH,
    ROOT_ZONE_ACPI_REGION,
    ROOT_ZONE_HPET,
    ROOT_ZONE_ZONE1_LOW,
    ROOT_ZONE_ZONE1_HIGH,
];

#[cfg(feature = "asterinas")]
pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 11] = [
    ROOT_ZONE_RAM_LOW,
    ROOT_ZONE_RSDP_REGION,
    ROOT_ZONE_RAM_MAIN,
    ROOT_ZONE_UEFI_REGION,
    ROOT_ZONE_RAM_AUX,
    ROOT_ZONE_RAM_HIGH,
    ROOT_ZONE_ACPI_REGION,
    ROOT_ZONE_HPET,
    ROOT_ZONE_PCI_MMIO,
    ROOT_ZONE_ZONE1_LOW,
    ROOT_ZONE_ZONE1_HIGH,
];

const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0x9000;
// Linux loads a standalone setup.bin (boot params) at 0xa000. Asterinas is a
// single bzImage whose first setup_sects+1 sectors hold the boot params; loaded
// just below the protected-mode entry so the kernel body lands at 0x10_0000.
#[cfg(not(feature = "asterinas"))]
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xa000;
#[cfg(feature = "asterinas")]
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xf_f000;
const ROOT_ZONE_VMLINUX_ENTRY_ADDR: GuestPhysAddr = 0x10_0000;
const ROOT_ZONE_SCREEN_BASE_ADDR: GuestPhysAddr = 0x7000_0000;

pub const IRQ_WAKEUP_VIRTIO_DEVICE: usize = 0x6;
pub const ROOT_ZONE_IRQS_BITMAP: &[BitmapWord] = &get_irqs_bitmap(&[0; 32]);
pub const ROOT_ZONE_IOAPIC_BASE: usize = 0xfec0_0000;
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    ioapic_base: ROOT_ZONE_IOAPIC_BASE,
    ioapic_size: 0x1000,
    kernel_entry_gpa: ROOT_ZONE_VMLINUX_ENTRY_ADDR,
    cmdline_load_gpa: ROOT_ZONE_CMDLINE_ADDR,
    setup_load_gpa: ROOT_ZONE_SETUP_ADDR,
    // Linux mounts a root disk; Asterinas boots from the initramfs supplied as a
    // separate module at GPA 0x1530_0000 (HPA 0x1a30_0000).
    #[cfg(not(feature = "asterinas"))]
    initrd_load_gpa: 0,
    #[cfg(feature = "asterinas")]
    initrd_load_gpa: 0x1530_0000,
    #[cfg(not(feature = "asterinas"))]
    initrd_size: 0,
    // GRUB gunzips the module, so this bounds the decompressed cpio. The cpio
    // reader stops at the archive trailer, so a ceiling above the real image is
    // fine; it stays inside the 0x2000_0000-byte RAM region at 0x1530_0000.
    #[cfg(feature = "asterinas")]
    initrd_size: 0x0400_0000,
    rsdp_memory_region_id: ROOT_ZONE_RSDP_REGION_ID,
    acpi_memory_region_id: ROOT_ZONE_ACPI_REGION_ID,
    uefi_memory_region_id: ROOT_ZONE_UEFI_REGION_ID,
    // not longer than 32 bits
    screen_base: ROOT_ZONE_SCREEN_BASE_ADDR,
};

pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    bus_range_begin: 0x0,
    bus_range_end: 0x1f,
    // Linux on this board reaches PCI config space through the legacy 0xcf8/0xcfc
    // ports, so the ECAM window is unused. Asterinas uses ECAM exclusively and
    // takes its base from the firmware MCFG table, which on QEMU q35 is at
    // 0xb000_0000; the virtual ECAM must be registered at the same address.
    #[cfg(not(feature = "asterinas"))]
    ecam_base: 0xe0000000,
    #[cfg(feature = "asterinas")]
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
pub const ROOT_PCI_DEVS: [HvPciDevConfig; 7] = [
    pci_dev!(0x0, 0x0, 0x0, 0x0 => 0x0, 0x0, 0x0, VpciDevType::Physical), // host bridge
    pci_dev!(0x0, 0x0, 0x1, 0x0 => 0x0, 0x1, 0x0, VpciDevType::Physical), // VGA controller
    pci_dev!(0x0, 0x0, 0x2, 0x0 => 0x0, 0x2, 0x0, VpciDevType::Physical), // Ethernet controller
    pci_dev!(0x0, 0x0, 0x3, 0x0 => 0x0, 0x3, 0x0, VpciDevType::Physical), // PCI bridge
    pci_dev!(0x0, 0x0, 0x1f, 0x0 => 0x0, 0x1f, 0x0, VpciDevType::Physical), // ISA bridge
    pci_dev!(0x0, 0x0, 0x1f, 0x2 => 0x0, 0x1f, 0x2, VpciDevType::Physical), // SATA controller
    // pci_dev!(0x0, 0x0, 0x1f, 0x3 => 0x0, 0x1f, 0x3, VpciDevType::Physical), // SMBus
    pci_dev!(0x0, 0x1, 0x0, 0x0 => 0x1, 0x0, 0x0, VpciDevType::Physical), // SCSI controller
];

#[cfg(all(graphics))]
pub const GRAPHICS_FONT: &[u8] =
    include_bytes!("../../platform/x86_64/qemu/image/font/spleen-6x12.psf");
