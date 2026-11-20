// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! RISC-V-specific zone configuration.

use super::common::HvZoneConfigCommon;
use super::zone::impl_zone_config;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Riscv64ArchConfig {
    pub plic_base: u64,
    pub plic_size: u64,
    pub aplic_base: u64,
    pub aplic_size: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Riscv64ZoneConfig {
    pub common: HvZoneConfigCommon,
    pub arch_config: Riscv64ArchConfig,
}

impl_zone_config!(Riscv64ZoneConfig, Riscv64ArchConfig);
