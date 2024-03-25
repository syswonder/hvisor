#![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    error::HvResult,
    percpu::PerCpu,
    zone::{find_zone_by_id, root_zone, Zone},
};

pub fn suspend_cpu(cpu_id: usize) {
    todo!();
    // trace!("suspending cpu {:#x?}", cpu_id);
    // let cpu_data = get_cpu_data(cpu_id);
    // let _lock = cpu_data.ctrl_lock.lock();
    // cpu_data.need_suspend = true;
    // let target_suspended = cpu_data.suspended;
    // drop(_lock);

    // if !target_suspended {
    //     send_event(cpu_id, SGI_EVENT_ID);
    // }
}

pub fn resume_cpu(cpu_id: usize) {
    todo!();
    // trace!("resuming cpu {:#x?}", cpu_id);
    // let cpu_data = get_cpu_data(cpu_id);
    // let _lock = cpu_data.ctrl_lock.lock();
    // cpu_data.need_suspend = false;
}

pub fn park_cpu(cpu_id: usize) {
    todo!();
    // trace!("parking cpu {:#x?}", cpu_id);
    // let cpu_data = get_cpu_data(cpu_id);
    // let _lock = cpu_data.ctrl_lock.lock();
    // cpu_data.park = true;
    // cpu_data.need_suspend = false;
}

pub fn reset_cpu(cpu_id: u64) {
    todo!();
    // trace!("resetting cpu {:#x?}", cpu_id);
    // let cpu_data = get_cpu_data(cpu_id);
    // let _lock = cpu_data.ctrl_lock.lock();
    // cpu_data.reset = true;
    // cpu_data.need_suspend = false;
}

/// check and suspend root_zone and new_zone.
pub fn zone_management_prologue(
    cpu_data: &mut PerCpu,
    zone_id: u64,
) -> HvResult<Arc<RwLock<Zone>>> {
    let this_cpu_zone = cpu_data.zone.clone().unwrap();
    let root_zone = root_zone();
    if !Arc::ptr_eq(&this_cpu_zone, &root_zone) {
        return hv_result_err!(EPERM, "Manage over non-root zones: unsupported!");
    }
    let zone = match find_zone_by_id(zone_id as _) {
        Some(zone) => zone,
        None => return hv_result_err!(ENOENT),
    };
    if Arc::ptr_eq(&zone, &root_zone) {
        return hv_result_err!(EINVAL, "Manage root-zone is not allowed!");
    }
    root_zone.read().suspend();
    zone.read().suspend();
    HvResult::Ok(zone)
}

pub fn wait_for_poweron() -> ! {
    todo!();
    // let cpu_data = this_cpu_data();
    // let mut _lock = Some(cpu_data.ctrl_lock.lock());
    // cpu_data.wait_for_poweron = true;
    // while !cpu_data.reset {
    //     _lock = None;
    //     while !cpu_data.reset {}
    //     _lock = Some(cpu_data.ctrl_lock.lock());
    // }
    // cpu_data.reset = false;
    // cpu_data.wait_for_poweron = false;
    // drop(_lock);

    // cpu_data.start_zone();
}
