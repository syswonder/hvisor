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

pub const SIFIVE_CCACHE_CONFIG: usize = 0x00;
pub const SIFIVE_CCACHE_CONFIG_BANK_MASK: u32 = 0xFF; // config[7:0]
pub const SIFIVE_CCACHE_CONFIG_WAYS_MASK: u32 = 0xFF << 8; // config[15:8]
pub const SIFIVE_CCACHE_CONFIG_SETS_MASK: u32 = 0xFF << 16; // config[23:16]
pub const SIFIVE_CCACHE_CONFIG_BLKS_MASK: u32 = 0xFF << 24; // config[31:24]

pub const SIFIVE_CCACHE_WAYENABLE: usize = 0x08;
pub const SIFIVE_CCACHE_FLUSH64: usize = 0x200;
pub const SIFIVE_CCACHE_FLUSH64_LINE_LEN: usize = 64;

/// SifiveCcache struct
pub struct SifiveCcache {
    base: usize,
}

#[allow(unused)]
impl SifiveCcache {
    /// Create a new SifiveCcache instance according to the base address
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    /// Enable all cache ways
    pub fn init(&self) {
        let cfg: u32 =
            unsafe { core::ptr::read_volatile((self.base + SIFIVE_CCACHE_CONFIG) as *const u32) };
        // Note: once all L3 cache ways are enabled, L3 cache can't be transferred to L3 LIM except restart
        // Enable whole L3 cache ways
        unsafe {
            let val = cfg & SIFIVE_CCACHE_CONFIG_WAYS_MASK >> 8;
            core::ptr::write_volatile((self.base + SIFIVE_CCACHE_WAYENABLE) as *mut u32, val - 1);
        }
        // Get configuration
        info!(
            "banks = {}, ways = {}, sets/bank = {}, bytes/block = {}",
            cfg & SIFIVE_CCACHE_CONFIG_BANK_MASK,
            (cfg & SIFIVE_CCACHE_CONFIG_WAYS_MASK) >> 8,
            1u64 << ((cfg & SIFIVE_CCACHE_CONFIG_SETS_MASK) >> 16),
            1u64 << ((cfg & SIFIVE_CCACHE_CONFIG_BLKS_MASK) >> 24)
        );
        // Get largest way enabled
        let cfg = unsafe {
            core::ptr::read_volatile((self.base + SIFIVE_CCACHE_WAYENABLE) as *const u32)
        };
        info!("Node 0, index of the largest way enabled: {}", cfg);
    }

    pub fn get_config(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + SIFIVE_CCACHE_CONFIG) as *const u32) }
    }

    pub fn get_wayenable(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + SIFIVE_CCACHE_WAYENABLE) as *const u32) }
    }

    /// Flush the related cache line to Memory
    pub fn flush_range(&self, paddr: u64) {
        // Handle page-fault for flush.
        unsafe {
            core::ptr::write_volatile(
                (self.base + SIFIVE_CCACHE_FLUSH64) as *mut usize,
                paddr as _,
            );
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        }
    }
}
