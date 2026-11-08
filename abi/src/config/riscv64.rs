// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! RISC-V-specific zone configuration.

use super::common::HvZoneConfigCommon;
use super::zone::impl_zone_config;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Riscv64ArchConfig {
    pub plic_base: usize,
    pub plic_size: usize,
    pub aplic_base: usize,
    pub aplic_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Riscv64ZoneConfig {
    pub common: HvZoneConfigCommon,
    pub arch_config: Riscv64ArchConfig,
}

impl_zone_config!(Riscv64ZoneConfig, Riscv64ArchConfig);
