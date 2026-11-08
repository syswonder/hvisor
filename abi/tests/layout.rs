// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

use core::mem::{align_of, offset_of, size_of};
use hvisor_abi::*;

#[test]
fn memory_region_layout() {
    assert_eq!(size_of::<HvConfigMemoryRegion>(), 32);
    assert_eq!(align_of::<HvConfigMemoryRegion>(), 8);
    assert_eq!(offset_of!(HvConfigMemoryRegion, physical_start), 8);
    assert_eq!(offset_of!(HvConfigMemoryRegion, virtual_start), 16);
    assert_eq!(offset_of!(HvConfigMemoryRegion, size), 24);
}

#[test]
fn pci_config_layout() {
    assert_eq!(size_of::<HvPciConfig>(), 104);
    assert_eq!(align_of::<HvPciConfig>(), 8);
    assert_eq!(offset_of!(HvPciConfig, bus_range_begin), 88);
    assert_eq!(offset_of!(HvPciConfig, domain), 96);
}

#[test]
fn ivc_and_boot_layout() {
    assert_eq!(size_of::<HvIvcConfig>(), 40);
    assert_eq!(align_of::<HvIvcConfig>(), 8);
    assert_eq!(size_of::<HvZoneBootMode>(), 16);
    assert_eq!(offset_of!(HvZoneBootMode, multiboot_info_paddr), 8);
}

#[test]
fn pci_device_layout() {
    use hvisor_abi::HvPciDevConfig;
    assert_eq!(size_of::<HvPciDevConfig>(), 12);
    assert_eq!(align_of::<HvPciDevConfig>(), 4);
}

#[test]
fn zone_config_layouts() {
    use hvisor_abi::config::aarch64::Aarch64ZoneConfig;
    use hvisor_abi::config::loongarch64::Loongarch64ZoneConfig;
    use hvisor_abi::config::riscv64::Riscv64ZoneConfig;
    use hvisor_abi::config::x86_64::X86_64ZoneConfig;

    assert_eq!(size_of::<Aarch64ZoneConfig>(), 3296);
    assert_eq!(align_of::<Aarch64ZoneConfig>(), 8);
    assert_eq!(offset_of!(Aarch64ZoneConfig, arch_config), 3176);

    assert_eq!(size_of::<Riscv64ZoneConfig>(), 3208);
    assert_eq!(align_of::<Riscv64ZoneConfig>(), 8);
    assert_eq!(offset_of!(Riscv64ZoneConfig, arch_config), 3176);

    assert_eq!(size_of::<Loongarch64ZoneConfig>(), 3184);
    assert_eq!(align_of::<Loongarch64ZoneConfig>(), 8);
    assert_eq!(offset_of!(Loongarch64ZoneConfig, arch_config), 3176);

    assert_eq!(size_of::<X86_64ZoneConfig>(), 3264);
    assert_eq!(align_of::<X86_64ZoneConfig>(), 8);
    assert_eq!(offset_of!(X86_64ZoneConfig, arch_config), 3176);
}
