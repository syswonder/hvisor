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

use crate::error::{HvResult, HvErrorNum::*};
use crate::pci::{pci_access::PciRW, PciConfigAddress};
use super::dwc::DwcConfigRegion;

// DWC PCIe ATU (Address Translation Unit) register offsets
// Unroll registers are used for unrolled ATU regions
pub const ATU_BASE: usize = 0x300000;
pub const ATU_REGION_SIZE: usize = 0x200;
pub const PCIE_ATU_UNR_REGION_CTRL1: usize = 0x00; // Type of the region
pub const PCIE_ATU_UNR_REGION_CTRL2: usize = 0x04; // Write 0x80000000 to enable
pub const PCIE_ATU_UNR_LOWER_BASE: usize = 0x08;   // CPU lower address (32bit)
pub const PCIE_ATU_UNR_UPPER_BASE: usize = 0x0C;   // CPU upper address (32bit)
pub const PCIE_ATU_UNR_LIMIT: usize = 0x10;        // Region limit (lower 32 bit of (cpu_addr + size - 1))
pub const PCIE_ATU_UNR_LOWER_TARGET: usize = 0x14; // PCI lower address
pub const PCIE_ATU_UNR_UPPER_TARGET: usize = 0x18; // PCI upper address

// ATU region type constants
pub const ATU_TYPE_CFG0: u32 = 0x4; // CFG0 Type
pub const ATU_TYPE_CFG1: u32 = 0x5; // CFG1 Type
pub const ATU_TYPE_MEM: u32 = 0x0;  // Memory Type
pub const ATU_TYPE_IO: u32 = 0x2;   // IO Type

// ATU enable bit
pub const ATU_ENABLE_BIT: u32 = 0x80000000;

pub const ATU_UNUSED: u32 = u32::MAX;

// ATU configuration parameters
#[derive(Debug, Clone, Copy)]
pub struct AtuConfig {
    pub index: usize,
    pub atu_type: u32,
    pub cpu_base: PciConfigAddress,
    pub cpu_limit: PciConfigAddress,
    pub pci_target: PciConfigAddress,
}

impl AtuConfig {
    pub fn new(
        index: usize,
        atu_type: u32,
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
        }
    }

    pub fn new_with_dwc_config_region(
        config_region: &DwcConfigRegion,
    ) -> Self {
        Self::new(
            config_region.atu_index,
            config_region.atu_type,
            config_region.base,
            config_region.size,
            config_region.base,
        )
    }
}

// ATU unroll configuration functions
pub struct AtuUnroll;

impl AtuUnroll {
    // Configure ATU region using unroll registers
    // Follows the same order as Linux kernel dw_pcie_prog_outbound_atu_unroll
    pub fn dw_pcie_prog_outbound_atu_unroll(
        dbi_backend: &dyn PciRW,
        config: &AtuConfig,
    ) -> HvResult {
        let atu_base = (ATU_BASE + config.index * ATU_REGION_SIZE) as PciConfigAddress;
        
        dbi_backend.write(atu_base + PCIE_ATU_UNR_LOWER_BASE as PciConfigAddress, 4, (config.cpu_base & 0xffffffff) as usize)?;
        dbi_backend.write(atu_base + PCIE_ATU_UNR_UPPER_BASE as PciConfigAddress, 4, (config.cpu_base >> 32) as usize)?;
        dbi_backend.write(atu_base + PCIE_ATU_UNR_LIMIT as PciConfigAddress, 4, (config.cpu_limit & 0xffffffff) as usize)?;
        dbi_backend.write(atu_base + PCIE_ATU_UNR_LOWER_TARGET as PciConfigAddress, 4, (config.pci_target & 0xffffffff) as usize)?;
        dbi_backend.write(atu_base + PCIE_ATU_UNR_UPPER_TARGET as PciConfigAddress, 4, (config.pci_target >> 32) as usize)?;
        dbi_backend.write(atu_base + PCIE_ATU_UNR_REGION_CTRL1 as PciConfigAddress, 4, config.atu_type as usize)?;
        dbi_backend.write(atu_base + PCIE_ATU_UNR_REGION_CTRL2 as PciConfigAddress, 4, ATU_ENABLE_BIT as usize)?;
        
        // Verify that ATU enable takes effect before any subsequent config and I/O accesses
        const MAX_RETRIES: usize = 5;
        const RETRY_DELAY_ITERATIONS: usize = 10;
        
        for _ in 0..MAX_RETRIES {
            let val = dbi_backend.read(atu_base + PCIE_ATU_UNR_REGION_CTRL2 as PciConfigAddress, 4)?;
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

