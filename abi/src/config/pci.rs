// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

//! PCI device assignment ABI.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum VpciDevType {
    #[default]
    Physical = 0,
    StandardVdev = 1,
    VirtioRng = 2,
    VirtioBlk = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct HvPciDevConfig {
    pub domain: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub v_bus: u8,
    pub v_device: u8,
    pub v_function: u8,
    pub dev_type: VpciDevType,
}
