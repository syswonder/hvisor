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

use alloc::sync::Arc;
use bit_field::BitField;

use super::{PciConfigAccessor, PciConfigMmio, PciRegion};

use crate::{
    error::HvResult,
    pci::{
        pci_struct::{Bdf, RootComplex},
        PciConfigAddress,
    },
};

impl RootComplex {
    pub fn new_ecam(mmio_base: PciConfigAddress) -> Self {
        let accessor = Arc::new(EcamConfigAccessor::new(mmio_base));

        Self {
            mmio_base,
            accessor,
        }
    }
}

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
    fn get_pci_addr_base(&self, bdf: Bdf, parent_bus: u8) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus() as PciConfigAddress;
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;

        let address = self.ecam_base + (bus << 20) + (device << 15) + (function << 12);
        Ok(address)
    }

    fn get_physical_address(
        &self,
        bdf: Bdf,
        offset: PciConfigAddress,
        _parent_bus: u8,
    ) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus() as PciConfigAddress;
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;

        // ECAM standard address calculation:
        // base + (bus << 20) + (device << 15) + (function << 12) + offset
        let address = self.ecam_base + (bus << 20) + (device << 15) + (function << 12) + offset;
        Ok(address)
    }
}

impl PciConfigMmio {
    pub(crate) fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
        (self.base + offset) as *mut T
    }
}
