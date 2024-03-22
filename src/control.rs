#![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    config::HvZoneDesc,
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

pub fn prepare_zone_start(zone: Arc<RwLock<Zone>>) -> HvResult<()> {
    todo!();
    // let cpu_data = this_cpu_data();

    // let mut zone_w = zone.write();

    // {
    //     zone_w.comm_page.clear();

    //     let flags = zone_w.config().flags();
    //     let console = zone_w.config().console();
    //     let comm_region = unsafe {
    //         (zone_w.comm_page.as_mut_ptr() as *mut CommRegion)
    //             .as_mut()
    //             .unwrap()
    //     };

    //     comm_region.revision = COMM_REGION_ABI_REVISION;
    //     comm_region.signature.copy_from_slice("JHCOMM".as_bytes());

    //     // set virtual debug console
    //     if flags & 0x40000000 > 0 {
    //         comm_region.flags |= 0x0001;
    //     }
    //     if flags & 0x80000000 > 0 {
    //         comm_region.flags |= 0x0002;
    //     }
    //     comm_region.console = console;
    //     let system_config = HvSystemConfig::get();
    //     comm_region.gic_version = system_config.platform_info.arch.gic_version;
    //     comm_region.gicd_base = system_config.platform_info.arch.gicd_base;
    //     comm_region.gicc_base = system_config.platform_info.arch.gicc_base;
    //     comm_region.gicr_base = system_config.platform_info.arch.gicr_base;
    // }

    // zone_w
    //     .cpu_set
    //     .iter()
    //     .enumerate()
    //     .for_each(|(index, cpu_id)| {
    //         get_cpu_data(cpu_id).cpu_on_entry = if index == 0 {
    //             zone_w.config().cpu_reset_address()
    //         } else {
    //             INVALID_ADDRESS
    //         };
    //     });

    // todo!();
    // // zone_w.irqchip_reset();

    // info!("start zone done!");
    // Ok(())
}

pub fn do_zone_create(desc: &HvZoneDesc) -> HvResult<Arc<RwLock<Zone>>> {
    todo!();
    // let config = ZoneConfig::new(desc);
    // let config_total_size = config.total_size();

    // // we create the new zone here
    // let zone = Zone::new(config, false)?;

    // if zone.owns_cpu(this_cpu_data().id) {
    //     panic!("error: try to assign the CPU we are currently running on");
    // }
    // // todo: arch_zone_create

    // let cpu_set = zone.cpu_set;

    // let new_zone_pointer = Arc::new(RwLock::new(zone));
    // {
    //     cpu_set.iter().for_each(|cpu| {
    //         get_cpu_data(cpu).zone = Some(new_zone_pointer.clone());
    //     });
    // }

    // // memory mapping
    // {
    //     let mut zone = new_zone_pointer.write();

    //     let mem_regs: Vec<HvMemoryRegion> = zone.config().mem_regions().to_vec();
    //     // zone.comm_page.comm_region.zone_state = CELL_SHUT_DOWN;

    //     let comm_page_pa = zone.comm_page.start_paddr();
    //     assert!(is_aligned(comm_page_pa));

    //     mem_regs.iter().for_each(|mem| {
    //         zone.mem_region_insert(MemoryRegion::from_hv_memregion(mem, Some(comm_page_pa)))
    //     });

    //     // add pci mapping
    //     let sys_config = HvSystemConfig::get();
    //     let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
    //     let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;
    //     zone.mmio_region_register(mmcfg_start as _, mmcfg_size, mmio_pci_handler, mmcfg_start);

    //     zone.adjust_irq_mappings();
    // }

    // add_zone(new_zone_pointer.clone());

    // Ok(new_zone_pointer)
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
