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
use crate::pci::{pci_access::{PciRW, PciRWBase}, pci_struct::Bdf, PciConfigAddress};
use alloc::sync::Arc;
use spin::Mutex;
use super::{PciConfigAccessor, PciRegion, PciConfigMmio, PciRegionMmio};
use super::dwc_atu::{AtuUnroll, AtuConfig, ATU_TYPE_CFG0, ATU_TYPE_CFG1};

#[cfg(feature = "dwc_pcie")]
impl PciRegion for PciConfigMmio {
    fn read_u8(&self, _offset: PciConfigAddress) -> HvResult<u8> {
        unimplemented!()
    }
    fn write_u8(&self, _offset: PciConfigAddress, _value: u8) -> HvResult {
        unimplemented!()
    }
    fn read_u16(&self, _offset: PciConfigAddress) -> HvResult<u16> {
        unimplemented!()
    }
    fn write_u16(&self, _offset: PciConfigAddress, _value: u16) -> HvResult {
        unimplemented!()
    }
    fn read_u32(&self, _offset: PciConfigAddress) -> HvResult<u32> {
        unimplemented!()
    }
    fn write_u32(&self, _offset: PciConfigAddress, _value: u32) -> HvResult {
        unimplemented!()
    }
}

// DWC PCIe region types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwcRegionType {
    Dbi = 0,
    Apb = 1,
    Cfg0 = 2,
    Cfg1 = 3,
}

impl DwcRegionType {
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

// DWC PCIe configuration region structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DwcConfigRegion {
    pub region_type: DwcRegionType,
    pub physical_start: u64,
    pub size: u64,
}

pub struct DwcConfigRegionBackend(PciRegionMmio);

impl PciRWBase for DwcConfigRegionBackend {
    fn backend(&self) -> &dyn PciRegion {
        &self.0
    }
}
impl PciRW for DwcConfigRegionBackend {}

#[derive(Debug)]
pub struct DwcConfigAccessor {
    dbi_backend: Arc<dyn PciRW>,
    cfg0_backend: Arc<dyn PciRW>,
    cfg1_backend: Arc<dyn PciRW>,
    apb_backend: Option<Arc<dyn PciRW>>,
}

impl DwcConfigAccessor {
    pub fn new(regions: [DwcConfigRegion; 4]) -> Self {
        let mut dbi_backend = None;
        let mut cfg0_backend = None;
        let mut cfg1_backend = None;
        let mut apb_backend = None;
        
        for region in regions.iter() {
            let mmio = PciRegionMmio::new(region.physical_start as PciConfigAddress, region.size);
            let backend = Arc::new(DwcConfigRegionBackend(mmio)) as Arc<dyn PciRW>;
            
            match region.region_type {
                DwcRegionType::Dbi => {
                    dbi_backend = Some(backend);
                }
                DwcRegionType::Cfg0 => {
                    cfg0_backend = Some(backend);
                }
                DwcRegionType::Cfg1 => {
                    cfg1_backend = Some(backend);
                }
                DwcRegionType::Apb => {
                    apb_backend = Some(backend);
                }
            }
        }
        
        Self {
            dbi_backend: dbi_backend.expect("DBI backend is required"),
            cfg0_backend: cfg0_backend.expect("CFG0 backend is required"),
            cfg1_backend: cfg1_backend.expect("CFG1 backend is required"),
            apb_backend,
        }
    }
}

impl PciConfigAccessor for DwcConfigAccessor {
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress) -> HvResult<PciConfigAddress> {

    }
    
    fn prepare_access(&self, bdf: Bdf) -> HvResult {

    }
    
    fn get_base_address(&self) -> PciConfigAddress {

    }
}

