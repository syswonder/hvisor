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

use core::{any::Any, fmt::Debug};

use crate::error::HvResult;
use crate::pci::{pci_struct::Bdf, PciConfigAddress};

// PCIe region trait for memory-mapped I/O access
pub trait PciRegion: Debug + Sync + Send + Any {
    fn read_u8(&self, offset: PciConfigAddress) -> HvResult<u8>;
    fn write_u8(&self, offset: PciConfigAddress, value: u8) -> HvResult;
    fn read_u16(&self, offset: PciConfigAddress) -> HvResult<u16>;
    fn write_u16(&self, offset: PciConfigAddress, value: u16) -> HvResult;
    fn read_u32(&self, offset: PciConfigAddress) -> HvResult<u32>;
    fn write_u32(&self, offset: PciConfigAddress, value: u32) -> HvResult;
}

/* in aarch64, config space just like a normal mem space */
#[derive(Debug, Clone, Copy)]
pub struct PciConfigMmio {
    base: PciConfigAddress,
    #[allow(dead_code)]
    length: u64,
}

impl PciConfigMmio {
    pub fn new(base: PciConfigAddress, length: u64) -> Self {
        Self { base, length }
    }

    /// Check if this is a placeholder (dummy) mmio with base address 0
    pub fn is_placeholder(&self) -> bool {
        self.base == 0 && self.length == 0
    }
}

/* PCIe region MMIO for general memory-mapped I/O regions */
#[derive(Debug, Clone, Copy)]
pub struct PciRegionMmio {
    base: PciConfigAddress,
    #[allow(dead_code)]
    length: u64,
}

impl PciRegionMmio {
    pub fn new(base: PciConfigAddress, length: u64) -> Self {
        Self { base, length }
    }
    /* TODO: may here need check whether length exceeds*/
    pub(crate) fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
        (self.base + offset) as *mut T
    }
}

impl PciRegion for PciRegionMmio {
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

pub trait PciConfigAccessor: Send + Sync + core::fmt::Debug {
    // Get physical address from BDF and offset
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress, _parent_bus: u8) -> HvResult<PciConfigAddress>;

    fn skip_device(&self, _bdf: Bdf) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PciAccessorType {
    Ecam,
    Dwc,
    LoongArch,
}

// Export accessor implementations
#[cfg(feature = "ecam_pcie")]
pub mod ecam;

#[cfg(feature = "dwc_pcie")]
pub mod dwc;
#[cfg(feature = "dwc_pcie")]
pub mod dwc_atu;

#[cfg(feature = "loongarch64_pcie")]
pub mod loongarch64;

