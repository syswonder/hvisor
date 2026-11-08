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

#[repr(C, usize)]
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

#[repr(C, usize)]
#[derive(Debug, Clone)]
#[allow(unused)]
pub enum GicConfig {
    Gicv2(Gicv2Config),
    Gicv3(Gicv3Config),
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Gicv2Config {
    pub gicd_base: usize,
    pub gicd_size: usize,
    pub gicc_base: usize,
    pub gicc_size: usize,
    pub gicc_offset: usize,
    pub gich_base: usize,
    pub gich_size: usize,
    pub gicv_base: usize,
    pub gicv_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Gicv3Config {
    pub gicd_base: usize,
    pub gicd_size: usize,
    pub gicr_base: usize,
    pub gicr_size: usize,
    pub gits_base: usize,
    pub gits_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Aarch64ZoneConfig {
    pub common: HvZoneConfigCommon,
    pub arch_config: Aarch64ArchConfig,
}

impl_zone_config!(Aarch64ZoneConfig, Aarch64ArchConfig);
