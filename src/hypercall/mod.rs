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
#![allow(dead_code)]
#![allow(unreachable_patterns)]

use crate::arch::cpu::get_target_cpu;
use crate::config::HvZoneConfig;
use crate::consts::{INVALID_ADDRESS, MAX_CPU_NUM, MAX_WAIT_TIMES, PAGE_SIZE};
use crate::device::virtio_trampoline::{MAX_DEVS, VIRTIO_BRIDGE, VIRTIO_IRQS};
use crate::error::HvResult;
use crate::percpu::{get_cpu_data, PerCpu};
use crate::zone::{
    add_zone, all_zones_info, find_zone, is_this_root_zone, remove_zone, zone_create, ZoneInfo,
};

use crate::event::{send_event, IPI_EVENT_SHUTDOWN, IPI_EVENT_VIRTIO_INJECT_IRQ, IPI_EVENT_WAKEUP};
use core::convert::TryFrom;
use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum HyperCallCode {
        HvVirtioInit = 0,
        HvVirtioInjectIrq = 1,
        HvVirtioGetIrq = 86,
        HvZoneStart = 2,
        HvZoneShutdown = 3,
        HvZoneList = 4,
        HvClearInjectIrq = 20,
        HvIvcInfo = 5,
        HvConfigCheck = 6,
    }
}
pub const SGI_IPI_ID: u64 = 7;

pub type HyperCallResult = HvResult<usize>;

pub struct HyperCall<'a> {
    cpu_data: &'a mut PerCpu,
}

impl<'a> HyperCall<'a> {
    pub fn new(cpu_data: &'a mut PerCpu) -> Self {
        Self { cpu_data }
    }

    pub fn hypercall(&mut self, code: u64, arg0: u64, arg1: u64) -> HyperCallResult {
        let code = match HyperCallCode::try_from(code) {
            Ok(code) => code,
            Err(_) => {
                warn!("hypercall id={} unsupported!", code);
                return Ok(0);
            }
        };
        debug!(
            "hypercall: code={:?}, arg0={:#x}, arg1={:#x}",
            code, arg0, arg1
        );
        unsafe {
            match code {
                HyperCallCode::HvVirtioInit => self.hv_virtio_init(arg0),
                HyperCallCode::HvVirtioInjectIrq => self.hv_virtio_inject_irq(),
                HyperCallCode::HvVirtioGetIrq => self.hv_virtio_get_irq(arg0 as *mut u32),
                HyperCallCode::HvZoneStart => {
                    self.hv_zone_start(&*(arg0 as *const HvZoneConfig), arg1)
                }
                HyperCallCode::HvZoneShutdown => self.hv_zone_shutdown(arg0),
                HyperCallCode::HvZoneList => self.hv_zone_list(&mut *(arg0 as *mut ZoneInfo), arg1),
                HyperCallCode::HvClearInjectIrq => {
                    use crate::consts::IPI_EVENT_CLEAR_INJECT_IRQ;
                    for i in 1..MAX_CPU_NUM {
                        // if target cpu status is not running, we skip it
                        if !get_cpu_data(i).arch_cpu.power_on {
                            continue;
                        }
                        send_event(i, SGI_IPI_ID as _, IPI_EVENT_CLEAR_INJECT_IRQ);
                    }
                    HyperCallResult::Ok(0)
                }
                HyperCallCode::HvIvcInfo => self.hv_ivc_info(arg0),
                HyperCallCode::HvConfigCheck => self.hv_zone_config_check(arg0 as *mut u64),
                _ => {
                    warn!("hypercall id={} unsupported!", code as u64);
                    Ok(0)
                }
            }
        }
    }

    // only root zone calls the function and set virtio shared region between el1 and el2.
    fn hv_virtio_init(&mut self, shared_region_addr: u64) -> HyperCallResult {
        info!(
            "handle hvc init virtio, shared_region_addr = {:#x?}",
            shared_region_addr
        );
        if !is_this_root_zone() {
            return hv_result_err!(EPERM, "Init virtio over non-root zones: unsupported!");
        }

        let shared_region_addr_pa = self.hv_get_real_pa(shared_region_addr) as usize;

        assert!(shared_region_addr_pa % PAGE_SIZE == 0);
        // let offset = shared_region_addr_pa & (PAGE_SIZE - 1);
        // memory::hv_page_table()
        // 	.write()
        // 	.insert(MemoryRegion::new_with_offset_mapper(
        // 		HVISOR_DEVICE_REGION_BASE,
        // 		shared_region_addr as _,
        // 		PAGE_SIZE,
        // 		MemFlags::READ | MemFlags::WRITE,
        // 	))?;
        // TODO: flush tlb
        VIRTIO_BRIDGE.set_base_addr(shared_region_addr_pa as _);
        info!("hvisor device region base is {:#x?}", shared_region_addr_pa);

        HyperCallResult::Ok(0)
    }

    // Inject virtio device's irq to non root when a virtio device finishes one IO request. Only root zone calls.
    fn hv_virtio_inject_irq(&mut self) -> HyperCallResult {
        trace!("hv_virtio_inject_irq: hypercall for trigger target cpu to inject irq");
        if !is_this_root_zone() {
            return hv_result_err!(
                EPERM,
                "Virtio send irq operation over non-root zones: unsupported!"
            );
        }
        let mut res_agent = VIRTIO_BRIDGE.res_agent();
        let mut map_irq = VIRTIO_IRQS.lock();
        while !res_agent.is_empty() {
            let (_res_front, irq_id, target_zone) = res_agent.peek_front();
            let target_cpu = match find_zone(target_zone as _) {
                Some(_zone) => get_target_cpu(irq_id as _, target_zone as _),
                _ => {
                    res_agent.advance_front();
                    continue;
                }
            };

            let irq_list = map_irq.entry(target_cpu).or_insert([0; MAX_DEVS + 1]);

            self.wait_for_interrupt(irq_list);
            if !irq_list[1..=irq_list[0] as usize].contains(&irq_id) {
                let len = irq_list[0] as usize;
                assert!(len + 1 < MAX_DEVS);
                irq_list[len + 1] = irq_id;
                irq_list[0] += 1;
                send_event(
                    target_cpu as _,
                    SGI_IPI_ID as _,
                    IPI_EVENT_VIRTIO_INJECT_IRQ,
                );
            }

            res_agent.advance_front();
        }
        drop(res_agent);
        HyperCallResult::Ok(0)
    }

    pub fn hv_zone_start(&mut self, config: &HvZoneConfig, config_size: u64) -> HyperCallResult {
        let config_ipa = config as *const HvZoneConfig as u64;
        let config_pa = self.hv_get_real_pa(config_ipa);
        let config = unsafe { &*(config_pa as *const HvZoneConfig) };

        debug!("hv_zone_start: config: {:#x?}", config);
        if !is_this_root_zone() {
            return hv_result_err!(
                EPERM,
                "Start zone operation over non-root zones: unsupported!"
            );
        }
        if config_size != core::mem::size_of::<HvZoneConfig>() as _ {
            return hv_result_err!(
                EINVAL,
                format!(
                    "hv_zone_start: config size should be {} bytes, but got {}",
                    core::mem::size_of::<HvZoneConfig>(),
                    config_size
                )
            );
        }
        let zone = zone_create(config)?;
        let boot_cpu = zone.read().cpu_set.first_cpu().unwrap();

        let target_data = get_cpu_data(boot_cpu as _);
        let _lock = target_data.ctrl_lock.lock();

        if !target_data.arch_cpu.power_on {
            info!("boot_cpu: {}", boot_cpu);
            send_event(boot_cpu, SGI_IPI_ID as _, IPI_EVENT_WAKEUP);
        } else {
            error!("hv_zone_start: cpu {} already on", boot_cpu);
            return hv_result_err!(EBUSY);
        };
        self.check_cpu_id();
        add_zone(zone);
        drop(_lock);
        HyperCallResult::Ok(0)
    }

    fn hv_zone_shutdown(&mut self, zone_id: u64) -> HyperCallResult {
        info!("handle hvc zone shutdown, id={}", zone_id);
        if !is_this_root_zone() {
            return hv_result_err!(
                EPERM,
                "Shutdown zone operation over non-root zones: unsupported!"
            );
        }
        if zone_id == 0 {
            return hv_result_err!(EINVAL);
        }
        // avoid virtio daemon send sgi to the shutdowning zone
        let mut map_irq = VIRTIO_IRQS.lock();

        let zone = match find_zone(zone_id as _) {
            Some(zone) => zone,
            _ => {
                return hv_result_err!(
                    EINVAL,
                    format!("Shutdown zone: zone {} not found!", zone_id)
                )
            }
        };
        let zone_w = zone.write();

        zone_w.cpu_set.iter().for_each(|cpu_id| {
            let _lock = get_cpu_data(cpu_id).ctrl_lock.lock();
            get_cpu_data(cpu_id).cpu_on_entry = INVALID_ADDRESS;
            send_event(cpu_id, SGI_IPI_ID as _, IPI_EVENT_SHUTDOWN);
            // set the virtio irq list's len to 0
            if let Some(irq_list) = map_irq.get_mut(&cpu_id) {
                irq_list[0] = 0;
            }
        });

        let mut count: usize = 0;

        // wait all zone's cpus shutdown
        while zone_w.cpu_set.iter().any(|cpu_id| {
            let _lock = get_cpu_data(cpu_id).ctrl_lock.lock();
            let power_on = get_cpu_data(cpu_id).arch_cpu.power_on;
            count += 1;
            if count > MAX_WAIT_TIMES {
                if power_on {
                    error!("cpu {} cannot be shut down", cpu_id);
                    return false;
                }
            }
            power_on
        }) {}

        zone_w.cpu_set.iter().for_each(|cpu_id| {
            let _lock = get_cpu_data(cpu_id).ctrl_lock.lock();
            get_cpu_data(cpu_id).zone = None;
        });
        zone_w.arch_irqchip_reset();

        drop(zone_w);
        drop(zone);
        remove_zone(zone_id as _);
        info!("zone {} has been shutdown", zone_id);
        HyperCallResult::Ok(0)
    }

    fn hv_zone_list(&mut self, zones: *mut ZoneInfo, cnt: u64) -> HyperCallResult {
        if zones.is_null() {
            return hv_result_err!(EINVAL, "hv_zone_list: zones is null");
        }
        let zones_info = all_zones_info();
        let zones_ipa = zones as u64;
        let zones_pa = self.hv_get_real_pa(zones_ipa);
        let zones = zones_pa as *mut ZoneInfo;
        let slice = unsafe { core::slice::from_raw_parts_mut(zones, cnt as usize) };

        // #[cfg(target_arch = "loongarch64")]
        // let slice = unsafe {
        //     core::slice::from_raw_parts_mut(
        //         (zones as u64 | crate::arch::mm::LOONGARCH64_CACHED_DMW_PREFIX) as *mut ZoneInfo,
        //         cnt as usize,
        //     )
        // };

        for (i, zone_info) in slice.iter_mut().enumerate() {
            if i < zones_info.len() {
                *zone_info = zones_info[i].clone();
            } else {
                break;
            }
        }

        HyperCallResult::Ok(core::cmp::min(cnt as _, zones_info.len()))
    }
}
