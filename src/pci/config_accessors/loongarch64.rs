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
    pub fn new_loongarch(mmio_base: PciConfigAddress, cfg_size: u64, root_bus: u8) -> Self {
        let accessor = Arc::new(LoongArchConfigAccessor::new(mmio_base, cfg_size, root_bus));

        Self {
            mmio_base,
            accessor,
        }
    }
}

// LoongArch PCIe accessor implementation
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
        Self {
            cfg0,
            cfg1,
            root_bus,
        }
    }
}

impl PciConfigAccessor for LoongArchConfigAccessor {
    fn get_pci_addr_base(&self, bdf: Bdf, parent_bus: u8) -> HvResult<PciConfigAddress> {
        let bus = bdf.bus() as PciConfigAddress;
        let device = bdf.device() as PciConfigAddress;
        let function = bdf.function() as PciConfigAddress;
        let offset = 0;

        // Extract Offset[11:8] (bits 11-8 of offset) for bits 31-28
        let offset_high = (offset >> 8) & 0xf;
        // Extract Offset[7:0] (bits 7-0 of offset) for bits 7-0
        let offset_low = offset & 0xff;

        let address = if bus == self.root_bus as PciConfigAddress {
            // Type 0 format (Root Bus):
            // Bits 31-28: Offset[11:8]
            // Bits 27-16: Reserved (0)
            // Bits 15-11: Device Number
            // Bits 10-8:  Function Number
            // Bits 7-0:   Offset[7:0]
            self.cfg0
                + ((offset_high << 24) | (device << 11) | (function << 8) | offset_low)
        } else {
            // Type 1 format (Other Bus):
            // Bits 31-28: Offset[11:8]
            // Bits 27-16: Bus Number
            // Bits 15-11: Device Number
            // Bits 10-8:  Function Number
            // Bits 7-0:   Offset[7:0]
            self.cfg1
                + ((offset_high << 24) | (bus << 16) | (device << 11) | (function << 8) | offset_low)
        };

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

        // Extract Offset[11:8] (bits 11-8 of offset) for bits 31-28
        let offset_high = (offset >> 8) & 0xf;
        // Extract Offset[7:0] (bits 7-0 of offset) for bits 7-0
        let offset_low = offset & 0xff;

        let address = if bus == self.root_bus as PciConfigAddress {
            // Type 0 format (Root Bus):
            // Bits 31-28: Offset[11:8]
            // Bits 27-16: Reserved (0)
            // Bits 15-11: Device Number
            // Bits 10-8:  Function Number
            // Bits 7-0:   Offset[7:0]
            self.cfg0
                + ((offset_high << 24) | (device << 11) | (function << 8) | offset_low)
        } else {
            // Type 1 format (Other Bus):
            // Bits 31-28: Offset[11:8]
            // Bits 27-16: Bus Number
            // Bits 15-11: Device Number
            // Bits 10-8:  Function Number
            // Bits 7-0:   Offset[7:0]
            self.cfg1
                + ((offset_high << 24) | (bus << 16) | (device << 11) | (function << 8) | offset_low)
        };

        Ok(address)
    }

    fn skip_device(&self, _bdf: Bdf) -> bool {
        false
    }
}

pub const HV_ADDR_PREFIX: u64 = 0x8000_0000_0000_0000;
pub const LOONG_HT_PREFIX: u64 = 0xe00_0000_0000;

impl PciConfigMmio {
    pub(crate) fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
        (self.base + offset | HV_ADDR_PREFIX | LOONG_HT_PREFIX) as *mut T
    }
}
