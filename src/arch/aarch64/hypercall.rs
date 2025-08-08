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
use crate::arch::ivc::{IvcInfo, IVC_INFOS};
use crate::config::{HvZoneConfig, CONFIG_MAGIC_VERSION};
use crate::device::virtio_trampoline::MAX_DEVS;
use crate::hypercall::HyperCall;
use crate::hypercall::HyperCallResult;
use crate::percpu::this_zone;
use crate::zone::this_zone_id;

impl<'a> HyperCall<'a> {
    pub fn hv_ivc_info(&mut self, ivc_info_ipa: u64) -> HyperCallResult {
        let zone_id = this_zone_id();
        let zone = this_zone();
        // ipa->hpa->hva
        let hpa = unsafe {
            zone.read()
                .gpm
                .page_table_query(ivc_info_ipa as _)
                .unwrap()
                .0
        };
        // hva == hpa
        let ivc_info = unsafe { &mut *(hpa as *mut IvcInfo) };
        let ivc_infos = IVC_INFOS.lock();
        let zone_ivc_info = ivc_infos.get(&(zone_id as _));
        match zone_ivc_info {
            Some(zone_ivc_info) => *ivc_info = *zone_ivc_info,
            None => return hv_result_err!(ENODEV, "Zone {zone_id} has no ivc config!"),
        }
        HyperCallResult::Ok(0)
    }

    pub fn translate_ipa_to_hva(&mut self, ipa: u64) -> u64 {
        return ipa;
    }

    pub fn wait_for_interrupt(&mut self, irq_list: &mut [u64; MAX_DEVS + 1]) {
        trace!("wait_for_interrupt is not need for AArch64");
    }

    pub fn hv_zone_config_check(&self, magic_version: *mut u64) -> HyperCallResult {
        unsafe {
            *magic_version = CONFIG_MAGIC_VERSION as _;
        }
        debug!(
            "hv_zone_config_check: finished writing current magic version ({:#x})",
            CONFIG_MAGIC_VERSION
        );
        HyperCallResult::Ok(0)
    }

    pub fn hv_get_real_pa(&mut self, config_addr: u64) -> u64 {
        // AArch64 does not have a specific prefix for cached memory, so we return the address as is.
        return config_addr;
    }

    pub fn hv_get_real_list_pa(&mut self, list_addr: u64) -> u64 {
        // RISC-V does not have a specific prefix for cached memory, so we return the address as is.
        return list_addr;
    }

    pub fn check_cpu_id(&self) {
        let cpuid = this_cpu_id();
        trace!("CPU ID: {} Start Zone", cpuid);
    }
}
