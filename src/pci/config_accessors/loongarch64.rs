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
    cfg_base: PciConfigAddress,
    root_bus: u8,
}

impl LoongArchConfigAccessor {
    pub fn new(cfg_base: PciConfigAddress, _cfg_size: u64, root_bus: u8) -> Self {
        Self { cfg_base, root_bus }
    }

    fn cfg_map(&self, bdf: Bdf, offset: PciConfigAddress) -> PciConfigAddress {
        let devfn = ((bdf.device() as PciConfigAddress) << 3) | bdf.function() as PciConfigAddress;
        let mut addroff = (devfn << 8) | (offset & 0xff) | ((offset & 0xf00) << 16);
        if bdf.bus() != self.root_bus {
            addroff |= 1 << 28;
            addroff |= (bdf.bus() as PciConfigAddress) << 16;
        }
        self.cfg_base + addroff
    }
}

impl PciConfigAccessor for LoongArchConfigAccessor {
    fn get_pci_addr_base(&self, bdf: Bdf) -> HvResult<PciConfigAddress> {
        Ok(self.cfg_map(bdf, 0))
    }

    fn get_physical_address(
        &self,
        bdf: Bdf,
        offset: PciConfigAddress,
        _parent_bus: u8,
    ) -> HvResult<PciConfigAddress> {
        Ok(self.cfg_map(bdf, offset))
    }

    fn skip_device(&self, _bdf: Bdf) -> bool {
        false
    }
}

pub const HV_ADDR_PREFIX: u64 = 0x8000_0000_0000_0000;
pub const LOONG_HT_PREFIX: u64 = 0xe00_0000_0000;

impl PciConfigMmio {
    pub(crate) fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
        let addroff = (offset & 0xff) | ((offset & 0xf00) << 16);
        (self.base + addroff | HV_ADDR_PREFIX | LOONG_HT_PREFIX) as *mut T
    }

    fn read_u32_aligned(&self, offset: PciConfigAddress) -> u32 {
        unsafe { self.access::<u32>(offset & !0x3).read_volatile() }
    }

    fn write_u32_aligned(&self, offset: PciConfigAddress, value: u32) {
        unsafe { self.access::<u32>(offset & !0x3).write_volatile(value) }
    }

    fn read_aligned_part(&self, offset: PciConfigAddress, size: u32) -> u32 {
        let value = self.read_u32_aligned(offset);
        let shift = ((offset & 0x3) * 8) as u32;
        let mask = (1u32 << (size * 8)) - 1;

        (value >> shift) & mask
    }

    fn write_aligned_part(&self, offset: PciConfigAddress, size: u32, value: u32) {
        let shift = ((offset & 0x3) * 8) as u32;
        let mask = !(((1u32 << (size * 8)) - 1) << shift);
        let old = self.read_u32_aligned(offset);
        let new = (old & mask) | (value << shift);

        self.write_u32_aligned(offset, new);
    }
}

impl PciRegion for PciConfigMmio {
    fn read_u8(&self, offset: PciConfigAddress) -> HvResult<u8> {
        Ok(self.read_aligned_part(offset, 1) as u8)
    }

    fn write_u8(&self, offset: PciConfigAddress, value: u8) -> HvResult {
        self.write_aligned_part(offset, 1, value as u32);
        Ok(())
    }

    fn read_u16(&self, offset: PciConfigAddress) -> HvResult<u16> {
        Ok(self.read_aligned_part(offset, 2) as u16)
    }

    fn write_u16(&self, offset: PciConfigAddress, value: u16) -> HvResult {
        self.write_aligned_part(offset, 2, value as u32);
        Ok(())
    }

    fn read_u32(&self, offset: PciConfigAddress) -> HvResult<u32> {
        Ok(self.read_u32_aligned(offset))
    }

    fn write_u32(&self, offset: PciConfigAddress, value: u32) -> HvResult {
        self.write_u32_aligned(offset, value);
        Ok(())
    }
}
