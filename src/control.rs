use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    arch::sysreg::write_sysreg,
    cell::{find_cell_by_id, root_cell, Cell, CommRegion},
    config::{HvMemoryRegion, HvSystemConfig},
    consts::INVALID_ADDRESS,
    error::HvResult,
    hypercall::{COMM_REGION_ABI_REVISION, SGI_EVENT_ID},
    memory::{
        addr::{GuestPhysAddr, HostPhysAddr},
        MemFlags, MemoryRegion,
    },
    percpu::{get_cpu_data, this_cpu_data, PerCpu},
};

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
    info!("write sgi sys value = {:#x}", val);
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

/// check and suspend root_cell and new_cell.
pub fn cell_management_prologue(
    cpu_data: &mut PerCpu,
    cell_id: u64,
) -> HvResult<Arc<RwLock<Cell>>> {
    let this_cpu_cell = cpu_data.cell.clone().unwrap();
    let root_cell = root_cell();
    if !Arc::ptr_eq(&this_cpu_cell, &root_cell) {
        return hv_result_err!(EPERM, "Manage over non-root cells: unsupported!");
    }
    let cell = match find_cell_by_id(cell_id as _) {
        Some(cell) => cell,
        None => return hv_result_err!(ENOENT),
    };
    if Arc::ptr_eq(&cell, &root_cell) {
        return hv_result_err!(EINVAL, "Manage root-cell is not allowed!");
    }
    root_cell.read().suspend();
    cell.read().suspend();
    HvResult::Ok(cell)
}

pub fn cell_start(id: u64) -> HvResult<()> {
    let cpu_data = this_cpu_data();
    let cell = if id != 0 {
        cell_management_prologue(cpu_data, id)?
    } else {
        root_cell()
    };
    let mut cell_w = cell.write();

    {
        cell_w.comm_page.clear();

        let flags = cell_w.config().flags();
        let console = cell_w.config().console();
        let comm_region = unsafe {
            (cell_w.comm_page.as_mut_ptr() as *mut CommRegion)
                .as_mut()
                .unwrap()
        };

        comm_region.revision = COMM_REGION_ABI_REVISION;
        comm_region.signature.copy_from_slice("JHCOMM".as_bytes());

        // set virtual debug console
        if flags & 0x40000000 > 0 {
            comm_region.flags |= 0x0001;
        }
        if flags & 0x80000000 > 0 {
            comm_region.flags |= 0x0002;
        }
        comm_region.console = console;
        let system_config = HvSystemConfig::get();
        comm_region.gic_version = system_config.platform_info.arch.gic_version;
        comm_region.gicd_base = system_config.platform_info.arch.gicd_base;
        comm_region.gicc_base = system_config.platform_info.arch.gicc_base;
        comm_region.gicr_base = system_config.platform_info.arch.gicr_base;
    }

    if cell_w.loadable && id != 0 {
        let mem_regs: Vec<HvMemoryRegion> = cell_w.config().mem_regions().to_vec();
        let root_cell = root_cell();
        let mut root_cell_w = root_cell.write();
        mem_regs.iter().for_each(|mem| {
            if mem.flags.contains(MemFlags::LOADABLE) {
                root_cell_w.mem_region_unmap_partial(&MemoryRegion::new_with_offset_mapper(
                    mem.phys_start as GuestPhysAddr,
                    mem.phys_start as HostPhysAddr,
                    mem.size as _,
                    mem.flags,
                ));
            }
        });
        cell_w.loadable = false;
    }

    let mut is_first = true;
    cell_w.cpu_set.iter().for_each(|cpu_id| {
        get_cpu_data(cpu_id).cpu_on_entry = if is_first {
            cell_w.config().cpu_reset_address()
        } else {
            INVALID_ADDRESS
        };
        is_first = false;
        reset_cpu(cpu_id);
    });

    cell_w.irqchip_reset();

    if id != 0 {
        root_cell().read().resume();
    }

    info!("start cell done!");
    Ok(())
}
