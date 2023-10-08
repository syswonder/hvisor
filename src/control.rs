use crate::{arch::sysreg::write_sysreg, error::HvResult, percpu::get_cpu_data, hypercall::SGI_HV_ID};

pub fn arch_send_event(cpu_id: u64, sgi_num: u64) -> HvResult {
    //TODO: add more info
    let aff3: u64 = 0 << 48;
    let aff2: u64 = 0 << 32;
    let aff1: u64 = 0 << 16;
    let irm: u64 = 0 << 40;
    let sgi_id: u64 = sgi_num << 24;
    let target_list: u64 = 1 << cpu_id;
    let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
    write_sysreg!(icc_sgi1r_el1, val);
    info!("write sgi sys value = {:#x}", val);
    Ok(())
}

pub fn suspend_cpu(cpu_id: u64) {
    let cpu_data = get_cpu_data(cpu_id);
    let _lock = cpu_data.ctrl_lock.lock();
    cpu_data.need_suspend = true;
    let target_suspended = cpu_data.suspended;
    drop(_lock);

    if !target_suspended {
        arch_send_event(cpu_id, SGI_HV_ID);
    }
}