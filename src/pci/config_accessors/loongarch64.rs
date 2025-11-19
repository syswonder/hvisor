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
use crate::pci::{pci_struct::{Bdf, RootComplex}, PciConfigAddress};
use super::{PciConfigAccessor, PciRegion, PciConfigMmio, BdfAddressConversion};
use alloc::sync::Arc;
use bit_field::BitField;

impl RootComplex {
    pub fn new_loongarch(mmio_base: PciConfigAddress) -> Self {
        let accessor = Arc::new(LoongArchConfigAccessor::new(mmio_base));
        
        Self { 
            mmio_base,
            accessor,
        }
    }
}

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
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress, _parent_bus: u8) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus() as PciConfigAddress;
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;
        
        // LoongArch PCIe uses similar address calculation to ECAM
        // base + (bus << 20) + (device << 15) + (function << 12) + offset
        let address = self.cfg_base 
            + (bus << 20)
            + (device << 15)
            + (function << 12)
            + offset;
        Ok(address)
    }

    fn skip_device(&self, _bdf: Bdf) -> bool {
        false
    }
}

#[cfg(feature = "loongarch64_pcie")]
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

