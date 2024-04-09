#![allow(dead_code)]
use crate::consts::{DTB_IPA, INVALID_ADDRESS};
use crate::error::HvResult;
use crate::event::{send_event, IPI_EVENT_SHUTDOWN, IPI_EVENT_WAKEUP};
use crate::percpu::{get_cpu_data, PerCpu};
use crate::zone::{find_zone, remove_zone, zone_create};

use core::convert::TryFrom;
use numeric_enum_macro::numeric_enum;

#[repr(C)]
#[derive(Debug)]
pub struct ZoneInfo {
    id: u64,
    image_phys_addr: u64,
    dtb_phys_addr: u64,
}

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum HyperCallCode {
        HvZoneStart = 11,
        HvZoneShutdown = 12,
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

    pub fn hypercall(&mut self, code: u64, arg0: u64, _arg1: u64) -> HyperCallResult {
        let code = match HyperCallCode::try_from(code) {
            Ok(code) => code,
            Err(_) => {
                warn!("hypercall id={} unsupported!", code);
                return Ok(0);
            }
        };
        unsafe {
            match code {
                HyperCallCode::HvZoneStart => self.hv_zone_start(&*(arg0 as *const ZoneInfo)),
                HyperCallCode::HvZoneShutdown => self.hv_zone_shutdown(arg0),
            }
        }
    }


    pub fn hv_zone_start(&mut self, zone_info: &ZoneInfo) -> HyperCallResult {
        info!("handle hvc zone start");
        let zone = zone_create(zone_info.id as _, zone_info.dtb_phys_addr as _, DTB_IPA)?;
        let boot_cpu = zone.read().cpu_set.first_cpu().unwrap();

        let target_data = get_cpu_data(boot_cpu as _);
        let _lock = target_data.ctrl_lock.lock();

        if !target_data.arch_cpu.psci_on {
            send_event(boot_cpu, SGI_IPI_ID as _, IPI_EVENT_WAKEUP);
        } else {
            error!("hv_zone_start: cpu {} already on", boot_cpu);
            return hv_result_err!(EBUSY);
        };
        drop(_lock);
        HyperCallResult::Ok(0)
    }

    fn hv_zone_shutdown(&mut self, zone_id: u64) -> HyperCallResult {
        info!("handle hvc zone shutdown, id={}", zone_id);
        if zone_id == 0 {
            return hv_result_err!(EINVAL);
        }
        let zone = match find_zone(zone_id as _) {
            Some(zone) => zone,
            _ => return hv_result_err!(EEXIST),
        };
        let zone_r = zone.read();

        // // return zone's cpus to root_zone
        zone_r.cpu_set.iter().for_each(|cpu_id| {
            let _lock = get_cpu_data(cpu_id).ctrl_lock.lock();
            get_cpu_data(cpu_id).zone = None;
            get_cpu_data(cpu_id).cpu_on_entry = INVALID_ADDRESS;
            send_event(cpu_id, SGI_IPI_ID as _, IPI_EVENT_SHUTDOWN);
        });

        zone_r.arch_irqchip_reset();

        drop(zone_r);
        drop(zone);
        remove_zone(zone_id as _);

        HyperCallResult::Ok(0)
    }
}
