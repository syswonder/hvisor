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
//  ForeverYolo <2572131118@qq.com>

use crate::arch::cpu::this_cpu_id;
use crate::config::HvZoneConfig;
use crate::config::CONFIG_MAGIC_VERSION;
use crate::device::virtio_trampoline::MAX_DEVS;
use crate::hypercall::HyperCall;
use crate::hypercall::HyperCallResult;
impl<'a> HyperCall<'a> {
    pub fn hv_ivc_info(&mut self, ivc_info_ipa: u64) -> HyperCallResult {
        warn!("hv_ivc_info is not implemented for LoongArch64");
        HyperCallResult::Ok(0)
    }

    pub fn wait_for_interrupt(&mut self, irq_list: &mut [u64; MAX_DEVS + 1]) {
        use crate::device::irqchip::ls7a2000::*;
        let status = GLOBAL_IRQ_INJECT_STATUS.lock();
        drop(status);
        irq_list[0] = 0; // CAUTION: this is a workaround for loongarch64
    }

    pub fn hv_zone_config_check(&self, magic_version: *mut u64) -> HyperCallResult {
        let magic_version_raw = magic_version as u64;
        let magic_version_hva = magic_version_raw | crate::arch::mm::LOONGARCH64_CACHED_DMW_PREFIX;
        let magic_version_hva = magic_version_hva as *mut u64;
        debug!(
            "hv_zone_config_check: magic_version target addr to write = {:#x?}",
            magic_version_hva
        );
        unsafe {
            core::ptr::write(magic_version_hva, CONFIG_MAGIC_VERSION as _);
        }
        HyperCallResult::Ok(0)
    }

    pub fn hv_get_real_pa(&mut self, config_addr: u64) -> u64 {
        // LoongArch64 uses a specific prefix for cached memory addresses.
        let config_addr = config_addr as u64 | crate::arch::mm::LOONGARCH64_CACHED_DMW_PREFIX;
        return config_addr;
    }

    pub fn hv_get_real_list_pa(&mut self, list_addr: u64) -> u64 {
        // LoongArch64 does not have a specific prefix for cached memory, so we return the address as is.
        return list_addr;
    }

    pub fn check_cpu_id(&self) {
        let cpuid = this_cpu_id();
        assert_eq!(cpuid, 0);
    }

    pub fn hv_virtio_get_irq(&self, virtio_irq: *mut u32) -> HyperCallResult {
        trace!("hv_virtio_get_irq is not need for LoongArch64");
        HyperCallResult::Ok(0)
    }
}
