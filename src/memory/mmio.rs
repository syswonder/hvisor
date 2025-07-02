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
use core::{ptr, usize};

use crate::{error::HvResult, percpu::this_zone, zone::zone_error};

use super::GuestPhysAddr;

pub type MMIOHandler = fn(&mut MMIOAccess, usize) -> HvResult;

#[derive(Copy, Clone, Debug)]
pub struct MMIOAccess {
    /** Address to access, depending on the context, an absolute address or
     * relative offset to region start. */
    pub address: GuestPhysAddr,
    /** Size of the access. */
    pub size: usize,
    /** True if write access. */
    pub is_write: bool,
    /** The value to be written or the read value to return. */
    pub value: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct MMIORegion {
    pub start: GuestPhysAddr,
    pub size: usize,
}

#[derive(Debug)]
pub struct MMIOConfig {
    pub region: MMIORegion,
    pub handler: MMIOHandler,
    pub arg: usize,
}

impl MMIORegion {
    pub fn contains_region(&self, addr: GuestPhysAddr, sz: usize) -> bool {
        addr >= self.start && addr + (sz as usize) <= self.start + (self.size as usize)
    }
}

pub fn mmio_perform_access(base: usize, mmio: &mut MMIOAccess) {
    let addr = base as usize + mmio.address;

    unsafe {
        if mmio.is_write {
            match mmio.size {
                1 => ptr::write_volatile(addr as *mut u8, mmio.value as u8),
                2 => ptr::write_volatile(addr as *mut u16, mmio.value as u16),
                4 => ptr::write_volatile(addr as *mut u32, mmio.value as u32),
                8 => ptr::write_volatile(addr as *mut u64, mmio.value as u64),
                _ => {
                    zone_error!("invalid mmio size: {}", mmio.size);
                }
            }
        } else {
            mmio.value = match mmio.size {
                1 => ptr::read_volatile(addr as *mut u8) as _,
                2 => ptr::read_volatile(addr as *mut u16) as _,
                4 => ptr::read_volatile(addr as *mut u32) as _,
                8 => ptr::read_volatile(addr as *mut u64) as _,
                _ => {
                    zone_error!("invalid mmio size: {}", mmio.size);
                    usize::MAX
                }
            }
        }
    }
}

pub fn mmio_handle_access(mmio: &mut MMIOAccess) -> HvResult {
    let zone = this_zone();
    let res = zone.read().find_mmio_region(mmio.address, mmio.size);
    let zone_id = zone.read().id;
    drop(zone);
    match res {
        Some((region, handler, arg)) => {
            mmio.address -= region.start;
            match handler(mmio, arg) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("mmio handler returned error: {:#x?}", e);
                    Err(e)
                }
            }
        }
        None => {
            warn!("Zone {} unhandled mmio fault {:#x?}", zone_id, mmio);
            hv_result_err!(EINVAL)
        }
    }
}

#[allow(dead_code)]
pub fn mmio_generic_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    mmio_perform_access(base, mmio);
    Ok(())
}
