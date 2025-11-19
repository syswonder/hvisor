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
use crate::pci::{pci_access::{PciRW, PciRWBase}, pci_struct::{Bdf, RootComplex}, PciConfigAddress};
use crate::config::HvDwcAtuConfig;
use alloc::sync::Arc;
use super::{PciConfigAccessor, PciRegion, PciConfigMmio, PciRegionMmio, BdfAddressConversion};
use super::dwc_atu::{AtuUnroll, AtuConfig, ATU_UNUSED};
use bit_field::BitField;

impl RootComplex {
    pub fn new_dwc(
        ecam_base: u64,
        atu_config: &HvDwcAtuConfig, 
        root_bus: u8
    ) -> Self {
        let accessor = Arc::new(DwcConfigAccessor::new(atu_config, root_bus));
        
        Self { 
            mmio_base: ecam_base,
            accessor,
        }
    }
}

impl PciRegion for PciConfigMmio {
    fn read_u8(&self, offset: PciConfigAddress) -> HvResult<u8> {
        unsafe { Ok(self.access::<u8>(offset).read_volatile() as u8) }
    }
    fn write_u8(&self, offset: PciConfigAddress, value: u8) -> HvResult {
        unsafe { self.access::<u8>(offset).write_volatile(value) }
        Ok(())
    }
    fn read_u16(&self, offset: PciConfigAddress) -> HvResult<u16> {
        unsafe { Ok(self.access::<u16>(offset).read_volatile() as u16) }
    }
    fn write_u16(&self, offset: PciConfigAddress, value: u16) -> HvResult {
        unsafe { self.access::<u16>(offset).write_volatile(value) }
        Ok(())
    }
    fn read_u32(&self, offset: PciConfigAddress) -> HvResult<u32> {
        unsafe { Ok(self.access::<u32>(offset).read_volatile() as u32) }
    }
    fn write_u32(&self, offset: PciConfigAddress, value: u32) -> HvResult {
        unsafe { self.access::<u32>(offset).write_volatile(value) }
        Ok(())
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DwcConfigRegion {
    pub atu_index: usize,
    pub atu_type: u32,
    pub base: PciConfigAddress,
    pub size: u64,
}

#[derive(Debug)]
pub struct DwcConfigRegionBackend(PciRegionMmio);

impl PciRWBase for DwcConfigRegionBackend {
    fn backend(&self) -> &dyn PciRegion {
        &self.0
    }
}
impl PciRW for DwcConfigRegionBackend {}

#[derive(Debug)]
pub struct DwcConfigAccessor {
    dbi_backend: Arc<DwcConfigRegionBackend>,
    dbi: DwcConfigRegion,
    cfg0: DwcConfigRegion,
    cfg1: DwcConfigRegion,
    root_bus: u8,
}

impl DwcConfigAccessor {
    pub fn new(
        atu_config: &HvDwcAtuConfig,
        root_bus: u8
    ) -> Self {
        let cfg_size_half = atu_config.cfg_size / 2;
        let cfg0_base = atu_config.cfg_base;
        let cfg1_base = atu_config.cfg_base + cfg_size_half;
        
        // Create DBI backend for ATU configuration
        let dbi_base = atu_config.dbi_base as PciConfigAddress;
        let dbi_size = atu_config.dbi_size;
        let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
        let dbi_backend = Arc::new(DwcConfigRegionBackend(dbi_region));
        
        let dbi = DwcConfigRegion {
            atu_index: ATU_UNUSED as usize,
            atu_type: 0,
            base: dbi_base,
            size: dbi_size,
        };
        let cfg0 = DwcConfigRegion {
            atu_index: atu_config.cfg0_atu_index,
            atu_type: atu_config.cfg0_atu_type,
            base: cfg0_base,
            size: cfg_size_half,
        };
        let cfg1 = DwcConfigRegion {
            atu_index: atu_config.cfg1_atu_index,
            atu_type: atu_config.cfg1_atu_type,
            base: cfg1_base,
            size: cfg_size_half,
        };
        
        Self {
            dbi_backend,
            dbi,
            cfg0,
            cfg1,
            root_bus,
        }
    }
}

impl PciConfigAccessor for DwcConfigAccessor {
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress, parent_bus: u8) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus();
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;

        warn!("parent_bus {} self.root_bus {}", parent_bus, self.root_bus);
        
        // Calculate address without bus field (bus is handled by different config regions)
        // Address format: (device << 15) + (function << 12) + offset
        let offset_without_bus = (device << 15) + (function << 12) + offset;
        
        let address = if bus == self.root_bus {
            warn!("1");
            // Root bus: use DBI directly, no ATU configuration needed
            self.dbi.base + offset_without_bus
        } else if parent_bus == self.root_bus {
            // Check if cfg0 ATU is configured (not ATU_UNUSED)
            warn!("2");
            if self.cfg0.atu_index == ATU_UNUSED as usize {
                return hv_result_err!(EINVAL, "CFG0 ATU is not configured");
            }
            let atu_config = AtuConfig::new_with_dwc_config_region(
                &self.cfg0,
            );
            AtuUnroll::dw_pcie_prog_outbound_atu_unroll(self.dbi_backend.as_ref(), &atu_config)?;
            
            self.cfg0.base + offset_without_bus
        } else {
            // Check if cfg1 ATU is configured (not ATU_UNUSED)
            warn!("3");
            if self.cfg1.atu_index == ATU_UNUSED as usize {
                return hv_result_err!(EINVAL, "CFG1 ATU is not configured");
            }
            let atu_config = AtuConfig::new_with_dwc_config_region(
                &self.cfg1,
            );
            AtuUnroll::dw_pcie_prog_outbound_atu_unroll(self.dbi_backend.as_ref(), &atu_config)?;
            
            self.cfg1.base + offset_without_bus
        };
        
        Ok(address)
    }

    fn skip_device(&self, bdf: Bdf) -> bool {
        // On root bus, only device 0 (slot 0) is valid, devices with dev > 0 should be skipped
        // This matches Linux kernel's dw_pcie_valid_device() behavior
        if bdf.bus() == self.root_bus && bdf.device() > 0 {
            warn!("skip_device {:#?}", bdf);
            return true;
        }
        false
    }
}

#[cfg(feature = "dwc_pcie")]
impl BdfAddressConversion for Bdf {
    fn from_address(address: PciConfigAddress) -> Bdf {
        let bdf = address >> 12;
        let function = (bdf & 0b111) as u8;
        let device = ((bdf >> 3) & 0b11111) as u8;
        let bus = (bdf >> 8) as u8;
        Bdf {
            bus,
            device,
            function,
        }
    }
}

