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

use super::dwc::DwcConfigRegion;
use core::{fmt, fmt::Debug};

use crate::{
    error::{HvErrorNum::*, HvResult},
    pci::{pci_access::PciRW, PciConfigAddress},
};

// DWC PCIe ATU (Address Translation Unit) register offsets
pub const ATU_BASE: usize = 0x300000;
pub const ATU_REGION_SIZE: usize = 0x200;
pub const PCIE_ATU_UNR_REGION_CTRL1: usize = 0x00;
pub const PCIE_ATU_UNR_REGION_CTRL2: usize = 0x04;
pub const PCIE_ATU_UNR_LOWER_BASE: usize = 0x08;
pub const PCIE_ATU_UNR_UPPER_BASE: usize = 0x0C;
pub const PCIE_ATU_UNR_LIMIT: usize = 0x10;
pub const PCIE_ATU_UNR_LOWER_TARGET: usize = 0x14;
pub const PCIE_ATU_UNR_UPPER_TARGET: usize = 0x18;
pub const PCIE_ATU_UNR_UPPER_LIMIT: usize = 0x20;

// ATU enable bit
pub const ATU_ENABLE_BIT: u32 = 0x80000000;

// the flag is for dbi just
pub const ATU_UNUSED: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, Default)]
pub enum AtuType {
    #[default]
    Unused = 0xFF,
    Cfg0 = 0x4,
    Cfg1 = 0x5,
    Mem = 0x0,
    Io = 0x2,
}

impl AtuType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x0 => AtuType::Mem,
            0x2 => AtuType::Io,
            0x4 => AtuType::Cfg0,
            0x5 => AtuType::Cfg1,
            0xFF => AtuType::Unused,
            _ => AtuType::Unused,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct AtuConfig {
    index: usize,
    atu_type: AtuType,
    cpu_base: PciConfigAddress,
    cpu_limit: PciConfigAddress,
    pci_target: PciConfigAddress,
    limit_hw_value: u32,
    upper_limit_hw_value: u32,
}

impl Debug for AtuConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtuConfig {{ index: {},\n atu_type: {:?},\n cpu_base: {:#x},\n cpu_limit: {:#x},\n pci_target: {:#x} }}", self.index(), self.atu_type(), self.cpu_base(), self.cpu_limit(), self.pci_target())
    }
}

impl AtuConfig {
    pub fn new(
        index: usize,
        atu_type: AtuType,
        cpu_base: PciConfigAddress,
        cpu_size: PciConfigAddress,
        pci_target: PciConfigAddress,
    ) -> Self {
        let cpu_limit = cpu_base + cpu_size - 1;
        Self {
            index,
            atu_type,
            cpu_base,
            cpu_limit,
            pci_target,
            limit_hw_value: 0,
            upper_limit_hw_value: 0,
        }
    }

    pub fn new_with_dwc_config_region(config_region: &DwcConfigRegion, atu_type: AtuType, pci_addr: PciConfigAddress) -> Self {
        Self::new(
            config_region.atu_index,
            atu_type,
            config_region.base,
            config_region.size,
            pci_addr,
        )
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn atu_type(&self) -> AtuType {
        self.atu_type
    }

    pub fn cpu_base(&self) -> PciConfigAddress {
        self.cpu_base
    }

    pub fn cpu_limit(&self) -> PciConfigAddress {
        self.cpu_limit
    }

    pub fn pci_target(&self) -> PciConfigAddress {
        self.pci_target
    }

    pub fn set_atu_type(&mut self, atu_type: AtuType) {
        self.atu_type = atu_type;
    }

    pub fn set_cpu_base(&mut self, cpu_base: PciConfigAddress) {
        self.cpu_base = cpu_base;
    }

    pub fn set_cpu_limit(&mut self, cpu_limit: PciConfigAddress) {
        self.cpu_limit = cpu_limit;
    }

    pub fn set_pci_target(&mut self, pci_target: PciConfigAddress) {
        self.pci_target = pci_target;
    }

    pub fn limit_hw_value(&self) -> u32 {
        self.limit_hw_value
    }

    pub fn upper_limit_hw_value(&self) -> u32 {
        self.upper_limit_hw_value
    }

    pub fn init_limit_hw_value(&mut self, dbi_backend: &dyn PciRW) -> HvResult {
        let atu_base = (ATU_BASE + self.index() * ATU_REGION_SIZE) as PciConfigAddress;

        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_LIMIT as PciConfigAddress,
            4,
            0,
        )?;
        self.limit_hw_value = dbi_backend.read(
            atu_base + PCIE_ATU_UNR_LIMIT as PciConfigAddress,
            4,
        )? as u32;

        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_UPPER_LIMIT as PciConfigAddress,
            4,
            0xffffffff,
        )?;
        self.upper_limit_hw_value = dbi_backend.read(
            atu_base + PCIE_ATU_UNR_UPPER_LIMIT as PciConfigAddress,
            4,
        )? as u32;

        Ok(())
    }
}

// ATU unroll configuration functions
pub struct AtuUnroll;

impl AtuUnroll {
    // Configure ATU region with unroll registers
    // Follows the same order as Linux kernel dw_pcie_prog_outbound_atu_unroll
    pub fn dw_pcie_prog_outbound_atu_unroll(
        dbi_backend: &dyn PciRW,
        config: &AtuConfig,
    ) -> HvResult {
        let atu_base = (ATU_BASE + config.index() * ATU_REGION_SIZE) as PciConfigAddress;

        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_LOWER_BASE as PciConfigAddress,
            4,
            (config.cpu_base() & 0xffffffff) as usize,
        )?;
        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_UPPER_BASE as PciConfigAddress,
            4,
            (config.cpu_base() >> 32) as usize,
        )?;
        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_LIMIT as PciConfigAddress,
            4,
            (config.cpu_limit() & 0xffffffff) as usize,
        )?;
        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_LOWER_TARGET as PciConfigAddress,
            4,
            (config.pci_target() & 0xffffffff) as usize,
        )?;
        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_UPPER_TARGET as PciConfigAddress,
            4,
            (config.pci_target() >> 32) as usize,
        )?;
        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_REGION_CTRL1 as PciConfigAddress,
            4,
            config.atu_type() as usize,
        )?;
        dbi_backend.write(
            atu_base + PCIE_ATU_UNR_REGION_CTRL2 as PciConfigAddress,
            4,
            ATU_ENABLE_BIT as usize,
        )?;

        // Verify that ATU enable takes effect before any subsequent config and I/O accesses
        const MAX_RETRIES: usize = 5;
        const RETRY_DELAY_ITERATIONS: usize = 10;

        for _ in 0..MAX_RETRIES {
            let val =
                dbi_backend.read(atu_base + PCIE_ATU_UNR_REGION_CTRL2 as PciConfigAddress, 4)?;
            if (val as u32) & ATU_ENABLE_BIT != 0 {
                return Ok(());
            }
            for _ in 0..RETRY_DELAY_ITERATIONS {
                core::hint::spin_loop();
            }
        }

        hv_result_err!(EBUSY, "Outbound iATU is not being enabled")
    }
}
