// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! LoongArch-specific zone configuration.

use super::common::HvZoneConfigCommon;
use super::zone::impl_zone_config;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Loongarch64ArchConfig {
    pub dummy: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Loongarch64ZoneConfig {
    pub common: HvZoneConfigCommon,
    pub arch_config: Loongarch64ArchConfig,
}

impl_zone_config!(Loongarch64ZoneConfig, Loongarch64ArchConfig);
