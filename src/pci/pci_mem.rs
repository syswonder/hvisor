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
use crate::pci::PciConfigAddress;

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
    fn access<T>(&self, offset: PciConfigAddress) -> *mut T {
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
