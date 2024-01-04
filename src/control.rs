use crate::{
    arch::sysreg::write_sysreg,
    hypercall::SGI_EVENT_ID,
    percpu::{get_cpu_data, this_cpu_data},
};

/// send sgi to cpu
pub fn send_event(cpu_id: u64, sgi_num: u64) {
    // TODO: add more info
    let aff3: u64 = 0 << 48;
    let aff2: u64 = 0 << 32;
    let aff1: u64 = 0 << 16;
    let irm: u64 = 0 << 40;
    let sgi_id: u64 = sgi_num << 24;
    let target_list: u64 = 1 << cpu_id;
    let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
    write_sysreg!(icc_sgi1r_el1, val);
    // info!("write sgi sys value = {:#x}", val);
}
/// suspend current cpu
pub fn suspend_self() {
    let cpu_data = this_cpu_data();
    let mut _lock = Some(cpu_data.ctrl_lock.lock());
    cpu_data.need_suspend = true;
    cpu_data.suspended = true;
    _lock = None;
    while cpu_data.need_suspend {}
    _lock = Some(cpu_data.ctrl_lock.lock());
    cpu_data.suspended = false;
    drop(_lock);
}

pub fn suspend_cpu(cpu_id: u64) {
    trace!("suspending cpu {:#x?}", cpu_id);
    let cpu_data = get_cpu_data(cpu_id);
    let _lock = cpu_data.ctrl_lock.lock();
    cpu_data.need_suspend = true;
    let target_suspended = cpu_data.suspended;
    drop(_lock);

    if !target_suspended {
        send_event(cpu_id, SGI_EVENT_ID);
    }
}

pub fn resume_cpu(cpu_id: u64) {
    trace!("resuming cpu {:#x?}", cpu_id);
    let cpu_data = get_cpu_data(cpu_id);
    let _lock = cpu_data.ctrl_lock.lock();
    cpu_data.need_suspend = false;
}

pub fn park_cpu(cpu_id: u64) {
    trace!("parking cpu {:#x?}", cpu_id);
    let cpu_data = get_cpu_data(cpu_id);
    let _lock = cpu_data.ctrl_lock.lock();
    cpu_data.park = true;
    cpu_data.need_suspend = false;
}

pub fn reset_cpu(cpu_id: u64) {
    trace!("resetting cpu {:#x?}", cpu_id);
    let cpu_data = get_cpu_data(cpu_id);
    let _lock = cpu_data.ctrl_lock.lock();
    cpu_data.reset = true;
    cpu_data.need_suspend = false;
}
