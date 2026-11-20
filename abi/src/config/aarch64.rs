// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! AArch64-specific zone configuration.

use super::common::HvZoneConfigCommon;
use super::zone::impl_zone_config;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Aarch64ArchConfig {
    pub is_aarch32: u8,
    pub gic_config: GicConfig,
    pub uefi_config: UefiConfig,
}

#[repr(C, u64)]
#[derive(Debug, Clone)]
#[allow(unused)]
pub enum UefiConfig {
    Uefi(HvUefiConfig),
    NoUefi,
}

#[repr(C)]
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct HvUefiConfig {
    pub memory_map_addr: u64,
    pub memory_map_size: u64,
    pub sys_map_addr: u64,
}

#[repr(C, u64)]
#[derive(Debug, Clone)]
#[allow(unused)]
pub enum GicConfig {
    Gicv2(Gicv2Config),
    Gicv3(Gicv3Config),
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Gicv2Config {
    pub gicd_base: u64,
    pub gicd_size: u64,
    pub gicc_base: u64,
    pub gicc_size: u64,
    pub gicc_offset: u64,
    pub gich_base: u64,
    pub gich_size: u64,
    pub gicv_base: u64,
    pub gicv_size: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Gicv3Config {
    pub gicd_base: u64,
    pub gicd_size: u64,
    pub gicr_base: u64,
    pub gicr_size: u64,
    pub gits_base: u64,
    pub gits_size: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Aarch64ZoneConfig {
    pub common: HvZoneConfigCommon,
    pub arch_config: Aarch64ArchConfig,
}

impl_zone_config!(Aarch64ZoneConfig, Aarch64ArchConfig);
