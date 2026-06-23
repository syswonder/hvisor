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

// The Asterinas bzImage setup header is loaded at gpa 0xff000, the last page of
// this window. Stop the ACPI/RSDP region one page short there so that page is
// reported as ordinary RAM (the boot-params page below), not ACPI-reclaim.
#[cfg(not(feature = "aster_guest"))]
const ROOT_ZONE_RSDP_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x50e_0000,
    virtual_start: 0xe_0000,
    size: 0x2_0000,
};
#[cfg(feature = "aster_guest")]
const ROOT_ZONE_RSDP_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x50e_0000,
    virtual_start: 0xe_0000,
    size: 0x1_f000,
};
const ROOT_ZONE_RSDP_REGION_ID: usize = 0x1;

// The boot-params page carved out of the RSDP window above (gpa 0xff000), mapped
// as RAM so the setup header is not advertised as ACPI-reclaim memory.
#[cfg(feature = "aster_guest")]
const ROOT_ZONE_BOOTPARAMS_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM,
    physical_start: 0x50f_f000,
    virtual_start: 0xf_f000,
    size: 0x1000,
};

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

#[cfg(not(feature = "aster_guest"))]
pub const ROOT_ZONE_NAME: &str = "root-linux";
#[cfg(feature = "aster_guest")]
pub const ROOT_ZONE_NAME: &str = "root-asterinas";

#[cfg(not(feature = "aster_guest"))]
pub const ROOT_ZONE_CMDLINE: &str =
    "console=tty0 console=ttyS0 earlycon=efifb earlyprintk=serial nointremap no_timer_check efi=noruntime pci=pcie_scan_all,lastbus=1 root=/dev/vda rw init=/init\0";

// Asterinas consumes the Linux/x86 boot protocol: kernel options precede `--`
// and the userspace init argv follows it. `sh /init` runs the bring-up script
// from the initramfs (mount the pseudo filesystems, then hand off to a shell).
#[cfg(feature = "aster_guest")]
pub const ROOT_ZONE_CMDLINE: &str =
    "init=/bin/busybox SHELL=/bin/sh LOGNAME=root HOME=/ USER=root PATH=/bin ostd.log_level=info console=ttyS0 -- sh /init\0";

// Common regions shared by both boot profiles. The Asterinas profile appends one
// extra region (the boot-params page); appending keeps every existing region
// index -- and thus the *_memory_region_id constants below -- valid.
macro_rules! root_zone_common_regions {
    () => {
        [
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
                physical_start: 0x4030_0000,
                virtual_start: 0x4030_0000,
                size: 0x2000_0000,
            }, // zone 1
            HvConfigMemoryRegion {
                mem_type: MEM_TYPE_RESERVED,
                physical_start: 0x1_0000_0000,
                virtual_start: 0x1_0000_0000,
                size: 0x7000_0000,
            }, // zone 1
        ]
    };
}

#[cfg(not(feature = "aster_guest"))]
pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 10] = root_zone_common_regions!();

#[cfg(feature = "aster_guest")]
pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 11] = {
    let common = root_zone_common_regions!();
    [
        common[0],
        common[1],
        common[2],
        common[3],
        common[4],
        common[5],
        common[6],
        common[7],
        common[8],
        common[9],
        ROOT_ZONE_BOOTPARAMS_REGION,
    ]
};

const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0x9000;

// Linux is loaded as a split setup.bin/vmlinux.bin pair with the zeropage low in
// memory. An Asterinas bzImage is loaded whole, so its setup header lands just
// below the 1 MiB protected-mode entry: the image has setup_sects = 7, i.e. a
// 0x1000-byte setup (8 sectors counting the boot sector), placing it at 0xff000.
#[cfg(not(feature = "aster_guest"))]
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xa000;
#[cfg(feature = "aster_guest")]
const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xf_f000;

const ROOT_ZONE_VMLINUX_ENTRY_ADDR: GuestPhysAddr = 0x10_0000;

// The Asterinas image ships an initramfs as a separate module; GRUB inflates it
// on load, so the window must cover the decompressed cpio.
#[cfg(not(feature = "aster_guest"))]
const ROOT_ZONE_INITRD_GPA: GuestPhysAddr = 0;
#[cfg(feature = "aster_guest")]
const ROOT_ZONE_INITRD_GPA: GuestPhysAddr = 0x1530_0000;
#[cfg(not(feature = "aster_guest"))]
const ROOT_ZONE_INITRD_SIZE: usize = 0;
#[cfg(feature = "aster_guest")]
const ROOT_ZONE_INITRD_SIZE: usize = 0x40_0000;

// QEMU's q35 advertises the PCI ECAM base in its MCFG table; keep hvisor's view
// in sync (Asterinas reads it from MCFG, the Linux config uses the legacy base).
#[cfg(not(feature = "aster_guest"))]
const ROOT_ZONE_ECAM_BASE: usize = 0xe000_0000;
#[cfg(feature = "aster_guest")]
const ROOT_ZONE_ECAM_BASE: usize = 0xb000_0000;
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
    initrd_load_gpa: ROOT_ZONE_INITRD_GPA,
    initrd_size: ROOT_ZONE_INITRD_SIZE,
    rsdp_memory_region_id: ROOT_ZONE_RSDP_REGION_ID,
    acpi_memory_region_id: ROOT_ZONE_ACPI_REGION_ID,
    uefi_memory_region_id: ROOT_ZONE_UEFI_REGION_ID,
    // screen_base must fit in 32 bits
    screen_base: ROOT_ZONE_SCREEN_BASE_ADDR,
};

pub const ROOT_PCI_CONFIG: [HvPciConfig; 1] = [HvPciConfig {
    bus_range_begin: 0x0,
    bus_range_end: 0x1f,
    ecam_base: ROOT_ZONE_ECAM_BASE as u64,
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
