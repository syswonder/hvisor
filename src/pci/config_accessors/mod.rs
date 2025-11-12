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
use alloc::sync::Arc;

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
    /* TODO: may here need check whether length exceeds*/
    pub(crate) fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
        (self.base + offset) as *mut T
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

// PCIe config space accessor trait
// Unified interface for different PCIe mechanisms (ECAM, DWC, LoongArch)
pub trait PciConfigAccessor: Send + Sync + core::fmt::Debug {
    // Get physical address from BDF and offset
    fn get_physical_address(&self, bdf: Bdf, offset: PciConfigAddress) -> HvResult<PciConfigAddress>;
    
    // Prepare access before reading/writing (e.g., configure ATU)
    fn prepare_access(&self, bdf: Bdf) -> HvResult;
    
    // Get base address of config space
    fn get_base_address(&self) -> PciConfigAddress;
}

// Accessor type enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PciAccessorType {
    Ecam,
    Dwc,
    LoongArch,
}

// Factory function to create accessor based on feature flag
pub fn create_accessor(
    accessor_type: PciAccessorType,
    ecam_base: PciConfigAddress,
) -> Arc<dyn PciConfigAccessor> {
    match accessor_type {
        PciAccessorType::Ecam => Arc::new(ecam::EcamConfigAccessor::new(ecam_base)),
        PciAccessorType::Dwc => Arc::new(ecam::EcamConfigAccessor::new(ecam_base)),
        PciAccessorType::LoongArch => Arc::new(ecam::EcamConfigAccessor::new(ecam_base)),
    }
}

// Get default accessor type based on current features
pub fn get_default_accessor_type() -> PciAccessorType {
    #[cfg(feature = "dwc_pcie")]
    return PciAccessorType::Dwc;
    
    #[cfg(feature = "loongarch64_pcie")]
    return PciAccessorType::LoongArch;
    
    #[cfg(feature = "ecam_pcie")]
    return PciAccessorType::Ecam;
    
    #[cfg(all(not(feature = "ecam_pcie"), not(feature = "dwc_pcie"), not(feature = "loongarch64_pcie")))]
    return PciAccessorType::Ecam; // Default to ECAM
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

// Default ECAM implementation when no feature is specified
#[cfg(all(not(feature = "ecam_pcie"), not(feature = "dwc_pcie"), not(feature = "loongarch64_pcie")))]
pub mod ecam;

