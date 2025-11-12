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

#[cfg(feature = "loongarch64_pcie")]
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

// LoongArch PCIe accessor implementation
// Similar to ECAM, uses standard address calculation
#[derive(Debug)]
pub struct LoongArchConfigAccessor {
    cfg_base: PciConfigAddress,
}

impl LoongArchConfigAccessor {
    pub fn new(cfg_base: PciConfigAddress) -> Self {
        Self { cfg_base }
    }
}

impl PciConfigAccessor for LoongArchConfigAccessor {
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress) -> HvResult<PciConfigAddress> {

    }
    
    fn prepare_access(&self, _bdf: Bdf) -> HvResult {

    }
    
    fn get_base_address(&self) -> PciConfigAddress {

    }
}

