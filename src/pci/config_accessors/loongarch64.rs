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
use super::{PciConfigAccessor, PciRegion, PciConfigMmio};
use alloc::sync::Arc;
use bit_field::BitField;
use crate::arch::loongarch64::mm::LOONGARCH64_UNCACHED_DMW_PREFIX;

impl RootComplex {
    pub fn new_loongarch(mmio_base: PciConfigAddress, cfg_size: u64, root_bus: u8) -> Self {
        let accessor = Arc::new(LoongArchConfigAccessor::new(mmio_base, cfg_size, root_bus));
        
        Self { 
            mmio_base,
            accessor,
        }
    }
}

// LoongArch PCIe accessor implementation
// Similar to ECAM, uses standard address calculation
#[derive(Debug)]
pub struct LoongArchConfigAccessor {
    cfg0: PciConfigAddress,
    cfg1: PciConfigAddress,
    root_bus: u8,
}

impl LoongArchConfigAccessor {
    pub fn new(cfg_base: PciConfigAddress, cfg_size: u64, root_bus: u8) -> Self {
        let cfg_size_half = cfg_size / 2;
        let cfg0 = cfg_base;
        let cfg1 = cfg_base + cfg_size_half;
        Self { cfg0, cfg1, root_bus }
    }
}

impl PciConfigAccessor for LoongArchConfigAccessor {
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress, _parent_bus: u8) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus() as PciConfigAddress;
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;
        
        let address = if bus == self.root_bus as PciConfigAddress{
            self.cfg0 + (offset >> 8) << 23 + device << 10 + function << 7 + (offset & 0xff)
        } else {
            self.cfg1 + (offset >> 8) << 23 + bus << 15 + device << 10 + function << 7 + (offset & 0xff)
        };

        Ok(address)
    }

    fn skip_device(&self, _bdf: Bdf) -> bool {
        false
    }
}

impl PciConfigMmio {
    /* TODO: may here need check whether length exceeds*/
    pub(crate) fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
        (self.base + offset | LOONGARCH64_UNCACHED_DMW_PREFIX) as *mut T
    }
}