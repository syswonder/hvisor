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

// SiFive composable cache controller Driver

// Note: this driver doesn't leverage full capability of hardware.
// This is needed for eic7700x soc(1 die), which has dma-noncoherent device, such as mmc, sata.
// In hvisor, L3 cache can't be used as Loosely-Integrated Memory(LIM).

// Reference:
//  Linux - drivers/soc/sifive/sifive_ccache.c

pub mod ccache;

use crate::arch::zone::HvArchZoneConfig;
use crate::error::{self, HvResult};
use crate::memory::mmio::MMIOAccess;
use crate::platform::__board::{SIFIVE_CCACHE_BASE, SIFIVE_CCACHE_SIZE};
use crate::zone::Zone;
use ccache::*;
use spin::Once;

pub static SIFIVE_CCACHE: Once<SifiveCcache> = Once::new();

pub fn init_sifive_ccache() {
    SIFIVE_CCACHE.call_once(|| SifiveCcache::new(SIFIVE_CCACHE_BASE));
    // info!("Sifive composable cache controller initialized at 0x{:x}", SIFIVE_CCACHE_BASE);
    // host_sifive_ccache().init();
}

pub fn host_sifive_ccache<'a>() -> &'a SifiveCcache {
    SIFIVE_CCACHE.get().expect("Uninitialized sifive ccache!")
}

/// Handle Zone's sifive composable cache controller mmio access.
pub fn virtual_sifive_ccache_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    match mmio.address {
        SIFIVE_CCACHE_CONFIG => {
            if mmio.size != 4 || mmio.is_write {
                error!("virtual_sifive_ccache_handler: Invalid access to SIFIVE_CCACHE_CONFIG");
                return hv_result_err!(EINVAL);
            }
            // Return config to guest
            mmio.value = host_sifive_ccache().get_config() as _;
        }
        SIFIVE_CCACHE_WAYENABLE => {
            if mmio.is_write {
                info!("Hvisor doesn't support guest to configure ccache wayenable");
                info!("Hvisor defaultly configs L3 cache ways enable.");
            } else {
                info!("virtual_sifive_ccache_handler: Reading SIFIVE_CCACHE_WAYENABLE");
                mmio.value = host_sifive_ccache().get_wayenable() as _;
            }
        }
        SIFIVE_CCACHE_FLUSH64 => {
            if mmio.is_write {
                host_sifive_ccache().flush_range(mmio.value as _);
            } else {
                return hv_result_err!(EPERM, "Guest wants to read SIFIVE_CCACHE_FLUSH64.");
            }
        }
        _ => {
            error!(
                "virtual_sifive_ccache_handler: Unknown address 0x{:x}",
                mmio.address
            );
            return hv_result_err!(EFAULT);
        }
    };
    Ok(())
}

impl Zone {
    /// Initialize cache controller MMIO region.
    pub fn virtual_sifive_ccache_mmio_init(&mut self) {
        self.mmio_region_register(
            SIFIVE_CCACHE_BASE,
            SIFIVE_CCACHE_SIZE,
            virtual_sifive_ccache_handler,
            0,
        );
    }
}
