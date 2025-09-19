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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

// ESWIN Eic7700x SysCrg Emulation

pub const EIC7700_SYSCRG_BASE: usize = 0x51828000;
pub const EIC7700_SYSCRG_SIZE: usize = 0x80000;
pub const EIC7700_HSP_SP_TOP_CSR_BASE: usize = 0x50440000;
pub const EIC7700_HSP_SP_TOP_CSR_SIZE: usize = 0x2000;

use crate::arch::zone::HvArchZoneConfig;
use crate::error::{self, HvResult};
use crate::memory::mmio::MMIOAccess;
use crate::zone::Zone;

pub fn virtual_syscrg_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    let addr = mmio.address + EIC7700_SYSCRG_BASE;
    use core::ptr::{read_volatile, write_volatile};
    unsafe {
        if mmio.is_write {
            info!(
                "Syscrg MMIO write: addr=0x{:x}, value=0x{:x}, size={}",
                mmio.address, mmio.value, mmio.size
            );
            match mmio.size {
                1 => write_volatile(addr as *mut u8, mmio.value as u8),
                2 => write_volatile(addr as *mut u16, mmio.value as u16),
                4 => write_volatile(addr as *mut u32, mmio.value as u32),
                8 => write_volatile(addr as *mut u64, mmio.value as u64),
                _ => panic!("Unsupported MMIO write size: {}", mmio.size),
            }
        } else {
            mmio.value = match mmio.size {
                1 => read_volatile(addr as *const u8) as _,
                2 => read_volatile(addr as *const u16) as _,
                4 => read_volatile(addr as *const u32) as _,
                8 => read_volatile(addr as *const u64) as _,
                _ => panic!("Unsupported MMIO read size: {}", mmio.size),
            };
            debug!(
                "Syscrg MMIO read: addr=0x{:x}, value=0x{:x}, size={}",
                mmio.address, mmio.value, mmio.size
            );
        }
    }

    Ok(())
}

pub fn virtual_hsp_sp_top_csr_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    let addr = mmio.address + EIC7700_HSP_SP_TOP_CSR_BASE;
    use core::ptr::{read_volatile, write_volatile};
    unsafe {
        if mmio.is_write {
            info!(
                "Hsp_sp_top_csr MMIO write: addr=0x{:x}, value=0x{:x}, size={}",
                mmio.address, mmio.value, mmio.size
            );
            match mmio.size {
                1 => write_volatile(addr as *mut u8, mmio.value as u8),
                2 => write_volatile(addr as *mut u16, mmio.value as u16),
                4 => write_volatile(addr as *mut u32, mmio.value as u32),
                8 => write_volatile(addr as *mut u64, mmio.value as u64),
                _ => panic!("Unsupported MMIO write size: {}", mmio.size),
            }
        } else {
            mmio.value = match mmio.size {
                1 => read_volatile(addr as *const u8) as _,
                2 => read_volatile(addr as *const u16) as _,
                4 => read_volatile(addr as *const u32) as _,
                8 => read_volatile(addr as *const u64) as _,
                _ => panic!("Unsupported MMIO read size: {}", mmio.size),
            };
            debug!(
                "Hsp_sp_top_csr MMIO read: addr=0x{:x}, value=0x{:x}, size={}",
                mmio.address, mmio.value, mmio.size
            );
        }
    }

    Ok(())
}

impl Zone {
    /// Initialize syscon MMIO region.
    pub fn virtual_syscon_mmio_init(&mut self) {
        self.mmio_region_register(
            EIC7700_SYSCRG_BASE,
            EIC7700_SYSCRG_SIZE,
            virtual_syscrg_handler,
            0,
        );
        self.mmio_region_register(
            EIC7700_HSP_SP_TOP_CSR_BASE,
            EIC7700_HSP_SP_TOP_CSR_SIZE,
            virtual_hsp_sp_top_csr_handler,
            0,
        );
    }
}
