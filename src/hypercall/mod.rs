#![allow(dead_code)]
use crate::cell::{find_cell_by_id, remove_cell, root_cell};
use crate::config::{HvCellDesc, HvMemoryRegion};
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE};
use crate::control::{cell_management_prologue, do_cell_create, park_cpu, prepare_cell_start};
use crate::error::HvResult;
use crate::memory::addr::{align_down, align_up};
use crate::memory::{self, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion};
use crate::percpu::{get_cpu_data, PerCpu};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::mem::size_of;
use core::sync::atomic::{AtomicU32, Ordering};
use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum HyperCallCode {
        HypervisorDisable = 0,
        HypervisorCellCreate = 1,
        HypervisorCellStart = 2,
        HypervisorCellSetLoadable = 3,
        HypervisorCellDestroy = 4,
    }
}

pub const SGI_INJECT_ID: u64 = 0;
pub const SGI_EVENT_ID: u64 = 15;
pub const SGI_RESUME_ID: u64 = 14;
pub const COMM_REGION_ABI_REVISION: u16 = 1;

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
        match code {
            HyperCallCode::HypervisorDisable => self.hypervisor_disable(),
            HyperCallCode::HypervisorCellCreate => self.hypervisor_cell_create(arg0),
            HyperCallCode::HypervisorCellSetLoadable => self.hypervisor_cell_set_loadable(arg0),
            HyperCallCode::HypervisorCellStart => self.hypervisor_cell_start(arg0),
            HyperCallCode::HypervisorCellDestroy => self.hypervisor_cell_destroy(arg0),
        }
    }

    fn hypervisor_disable(&mut self) -> HyperCallResult {
        let cpus = PerCpu::activated_cpus();

        static TRY_DISABLE_CPUS: AtomicU32 = AtomicU32::new(0);
        TRY_DISABLE_CPUS.fetch_add(1, Ordering::SeqCst);
        while TRY_DISABLE_CPUS.load(Ordering::Acquire) < cpus {
            core::hint::spin_loop();
        }
        info!("Handle hvc disable");
        self.cpu_data.deactivate_vmm(0)?;
        unreachable!()
    }

    fn hypervisor_cell_create(&mut self, config_address: u64) -> HyperCallResult {
        info!(
            "handle hvc cell create, config_address = {:#x?}",
            config_address
        );

        let cell = self.cpu_data.cell.clone().unwrap();
        if !Arc::ptr_eq(&cell, &root_cell()) {
            return hv_result_err!(EPERM, "Creation over non-root cells: unsupported!");
        }
        info!("prepare to suspend root_cell");

        let root_cell = root_cell().clone();
        root_cell.read().suspend();

        // todo: 检查新cell是否和已有cell同id或同名
        let config_address = cell.write().gpm_query(config_address as _);

        let cfg_pages_offs = config_address as usize & (PAGE_SIZE - 1);
        let cfg_mapping = memory::hv_page_table().write().map_temporary(
            align_down(config_address),
            align_up(cfg_pages_offs + size_of::<HvCellDesc>()),
            MemFlags::READ,
        )?;

        let desc: &HvCellDesc = unsafe {
            ((cfg_mapping + cfg_pages_offs) as *const HvCellDesc)
                .as_ref()
                .unwrap()
        };

        do_cell_create(desc)?;

        info!("cell create done!");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_set_loadable(&mut self, cell_id: u64) -> HyperCallResult {
        info!("handle hvc cell set loadable");
        let cell = cell_management_prologue(self.cpu_data, cell_id)?;
        let mut cell_w = cell.write();
        if cell_w.loadable {
            root_cell().read().resume();
            return HyperCallResult::Ok(0);
        }

        cell_w.cpu_set.iter().for_each(|cpu_id| park_cpu(cpu_id));
        cell_w.loadable = true;
        info!("cell.mem_regions() = {:#x?}", cell_w.config().mem_regions());
        let mem_regs: Vec<HvMemoryRegion> = cell_w.config().mem_regions().to_vec();

        // remap to rootcell
        let root_cell = root_cell();
        let mut root_cell_w = root_cell.write();

        mem_regs.iter().for_each(|mem| {
            if mem.flags.contains(MemFlags::LOADABLE) {
                root_cell_w.mem_region_map_partial(&MemoryRegion::new_with_offset_mapper(
                    mem.phys_start as GuestPhysAddr,
                    mem.phys_start as HostPhysAddr,
                    mem.size as _,
                    mem.flags,
                ));
            }
        });
        root_cell_w.resume();
        info!("set loadbable done!");
        HyperCallResult::Ok(0)
    }

    pub fn hypervisor_cell_start(&mut self, cell_id: u64) -> HyperCallResult {
        info!("handle hvc cell start");
        prepare_cell_start(find_cell_by_id(cell_id as _).unwrap())?;
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_destroy(&mut self, cell_id: u64) -> HyperCallResult {
        info!("handle hvc cell destroy");
        let cell = cell_management_prologue(self.cpu_data, cell_id)?;
        let mut cell_w = cell.write();
        let root_cell = root_cell();
        let mut root_cell_w = root_cell.write();
        // return cell's cpus to root_cell
        cell_w.cpu_set.iter().for_each(|cpu_id| {
            park_cpu(cpu_id);
            root_cell_w.cpu_set.set_bit(cpu_id);
            get_cpu_data(cpu_id).cell = Some(root_cell.clone());
        });
        // return loadable ram memory to root_cell
        let mem_regs: Vec<HvMemoryRegion> = cell_w.config().mem_regions().to_vec();
        mem_regs.iter().for_each(|mem| {
            if !(mem.flags.contains(MemFlags::COMMUNICATION)
                || mem.flags.contains(MemFlags::ROOTSHARED))
            {
                root_cell_w.mem_region_map_partial(&MemoryRegion::new_with_offset_mapper(
                    mem.phys_start as _,
                    mem.phys_start as _,
                    mem.size as _,
                    mem.flags,
                ));
            }
        });
        // TODO：arm_cell_dcaches_flush， invalidate cell mems in cache
        cell_w.cpu_set.iter().for_each(|id| {
            get_cpu_data(id).cpu_on_entry = INVALID_ADDRESS;
        });
        drop(root_cell_w);
        cell_w.gicv3_exit();
        cell_w.adjust_irq_mappings();
        drop(cell_w);
        // Drop the cell will destroy cell's MemorySet so that all page tables will free
        drop(cell);
        remove_cell(cell_id as _);
        root_cell.read().resume();
        // TODO: config commit
        info!("cell destroy succeed");
        HyperCallResult::Ok(0)
    }
}
