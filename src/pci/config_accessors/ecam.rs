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

use crate::error::HvResult;
use crate::pci::{pci_struct::Bdf, PciConfigAddress};
use super::{PciConfigAccessor, PciRegion, PciConfigMmio};

// ECAM (Enhanced Configuration Access Mechanism) accessor implementation
// Standard ECAM mechanism used by most PCIe platforms
#[derive(Debug)]
pub struct EcamConfigAccessor {
    ecam_base: PciConfigAddress,
}

impl EcamConfigAccessor {
    pub fn new(ecam_base: PciConfigAddress) -> Self {
        Self { ecam_base }
    }
}

impl PciConfigAccessor for EcamConfigAccessor {
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus() as PciConfigAddress;
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;
        
        // ECAM standard address calculation:
        // base + (bus << 20) + (device << 15) + (function << 12) + offset
        let address = self.ecam_base 
            + (bus << 20)
            + (device << 15)
            + (function << 12)
            + offset;
        Ok(address)
    }
    
    fn prepare_access(&self, _bdf: Bdf) -> HvResult {
        Ok(()) // ECAM doesn't need special preparation
    }
    
    fn get_base_address(&self) -> PciConfigAddress {
        self.ecam_base
    }
}

#[cfg(any(feature = "ecam_pcie", all(not(feature = "dwc_pcie"), not(feature = "loongarch64_pcie"))))]
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

