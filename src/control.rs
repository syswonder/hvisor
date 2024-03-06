#![allow(unused)]
use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    cell::{add_cell, find_cell_by_id, root_cell, Cell, CommRegion},
    config::{CellConfig, HvCellDesc, HvMemoryRegion, HvSystemConfig},
    consts::INVALID_ADDRESS,
    device::pci::mmio_pci_handler,
    error::HvResult,
    hypercall::{COMM_REGION_ABI_REVISION, SGI_EVENT_ID},
    memory::{
        self,
        addr::{align_down, is_aligned, GuestPhysAddr, HostPhysAddr},
        MemFlags, MemoryRegion,
    },
    percpu::{get_cpu_data, this_cell, this_cpu_data, PerCpu},
};

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

pub fn prepare_cell_start(cell: Arc<RwLock<Cell>>) -> HvResult<()> {
    let cpu_data = this_cpu_data();

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

    cell_w
        .cpu_set
        .iter()
        .enumerate()
        .for_each(|(index, cpu_id)| {
            get_cpu_data(cpu_id).cpu_on_entry = if index == 0 {
                cell_w.config().cpu_reset_address()
            } else {
                INVALID_ADDRESS
            };
        });

    cell_w.irqchip_reset();

    info!("start cell done!");
    Ok(())
}

pub fn do_cell_create(desc: &HvCellDesc) -> HvResult<Arc<RwLock<Cell>>> {
    let config = CellConfig::new(desc);
    let config_total_size = config.total_size();

    // we create the new cell here
    let cell = Cell::new(config, false)?;

    if cell.owns_cpu(this_cpu_data().id) {
        panic!("error: try to assign the CPU we are currently running on");
    }
    // todo: arch_cell_create

    let cpu_set = cell.cpu_set;

    let new_cell_pointer = Arc::new(RwLock::new(cell));
    {
        cpu_set.iter().for_each(|cpu| {
            get_cpu_data(cpu).cell = Some(new_cell_pointer.clone());
        });
    }

    // memory mapping
    {
        let mut cell = new_cell_pointer.write();

        let mem_regs: Vec<HvMemoryRegion> = cell.config().mem_regions().to_vec();
        // cell.comm_page.comm_region.cell_state = CELL_SHUT_DOWN;

        let comm_page_pa = cell.comm_page.start_paddr();
        assert!(is_aligned(comm_page_pa));

        mem_regs.iter().for_each(|mem| {
            cell.mem_region_insert(MemoryRegion::from_hv_memregion(mem, Some(comm_page_pa)))
        });

        // add pci mapping
        let sys_config = HvSystemConfig::get();
        let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
        let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;
        cell.mmio_region_register(mmcfg_start as _, mmcfg_size, mmio_pci_handler, mmcfg_start);

        cell.adjust_irq_mappings();
    }

    add_cell(new_cell_pointer.clone());

    Ok(new_cell_pointer)
}
pub fn wait_for_poweron() -> ! {
    let cpu_data = this_cpu_data();
    let mut _lock = Some(cpu_data.ctrl_lock.lock());
    cpu_data.wait_for_poweron = true;
    while !cpu_data.reset {
        _lock = None;
        while !cpu_data.reset {}
        _lock = Some(cpu_data.ctrl_lock.lock());
    }
    cpu_data.reset = false;
    cpu_data.wait_for_poweron = false;
    drop(_lock);

    cpu_data.start_zone();
}