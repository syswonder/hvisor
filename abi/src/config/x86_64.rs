// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! x86_64-specific zone configuration.

use super::common::HvZoneConfigCommon;
use super::zone::impl_zone_config;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct X86_64ArchConfig {
    /// Base address of IOAPIC MMIO registers, usually 0xfec00000.
    pub ioapic_base: usize,
    /// Size of IOAPIC MMIO registers, usually 0x1000.
    pub ioapic_size: usize,
    /// Start GPA of vmlinux.bin, usually 0x100000.
    pub kernel_entry_gpa: usize,
    /// GPA of the Linux boot command line.
    pub cmdline_load_gpa: usize,
    /// Start GPA of setup.bin, address length no bigger than 16 bits.
    pub setup_load_gpa: usize,
    /// Start GPA of initrd, or zero when unused.
    pub initrd_load_gpa: usize,
    /// Size of initrd.
    pub initrd_size: usize,
    /// Memory region ID where the RSDP table is copied.
    pub rsdp_memory_region_id: usize,
    /// Memory region ID where other ACPI tables are copied.
    pub acpi_memory_region_id: usize,
    pub uefi_memory_region_id: usize,
    /// Start GPA of the framebuffer, or zero when unused.
    pub screen_base: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct X86_64ZoneConfig {
    pub common: HvZoneConfigCommon,
    pub arch_config: X86_64ArchConfig,
}

impl_zone_config!(X86_64ZoneConfig, X86_64ArchConfig);
