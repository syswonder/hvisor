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

use crate::{cpu_data::this_zone, error::HvResult, zone::zone_error};

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
        // The start must be strictly inside the region: a zero-sized probe (used
        // by the x86 EPT-violation lookup) at exactly `start + size` belongs to
        // the next region, not this one. Both ends are computed with checked
        // arithmetic so a high-address region or access cannot wrap.
        match (
            self.start.checked_add(self.size as usize),
            addr.checked_add(sz),
        ) {
            (Some(end), Some(acc_end)) => addr >= self.start && addr < end && acc_end <= end,
            _ => false,
        }
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
    let zone_id = zone.id();
    drop(zone);
    match res {
        Some((region, handler, arg)) => {
            mmio.address -= region.start;

            // x86_64 requires instruction emulation for mmio access
            #[cfg(target_arch = "x86_64")]
            if mmio.size == 0 {
                return crate::arch::mmio::instruction_emulator(
                    &handler,
                    mmio,
                    region.start,
                    region.size,
                    arg,
                );
            }

            match handler(mmio, arg) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("mmio handler returned error: {:#x?}", e);
                    Err(e)
                }
            }
        }
        None => {
            // No device is mapped here. Real hardware returns all-ones when a CPU
            // reads from an empty MMIO/PCI aperture and discards writes; mirror that
            // so guest device probing (e.g. PCI BAR sizing into an unpopulated hole)
            // makes progress instead of taking an unrecoverable fault.
            #[cfg(target_arch = "x86_64")]
            if mmio.size == 0 {
                debug!(
                    "Zone {} access to unmapped mmio {:#x}",
                    zone_id, mmio.address
                );
                return crate::arch::mmio::instruction_emulator(
                    &(mmio_absent_handler as _),
                    mmio,
                    0,
                    usize::MAX,
                    0,
                );
            }
            warn!("Zone {} unhandled mmio fault {:#x?}", zone_id, mmio);
            hv_result_err!(EINVAL)
        }
    }
}

/// Handler for accesses with no backing device: reads return all-ones (the
/// bus-idle value), writes are discarded.
pub fn mmio_absent_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    if !mmio.is_write {
        mmio.value = usize::MAX;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn mmio_generic_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    mmio_perform_access(base, mmio);
    Ok(())
}
