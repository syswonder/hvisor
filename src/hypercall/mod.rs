#![allow(dead_code)]
use crate::cell::{add_cell, find_cell_by_id, root_cell, Cell, CommRegion};
use crate::config::{CellConfig, HvCellDesc, HvMemoryRegion, HvSystemConfig};
use crate::consts::{PAGE_SIZE, INVALID_ADDRESS};
use crate::control::{park_cpu, reset_cpu, send_event};
use crate::error::HvResult;
use crate::memory::{self, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion};
use crate::percpu::{get_cpu_data, this_cpu_data, PerCpu};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::mem::size_of;
use core::sync::atomic::{AtomicU32, Ordering};
use numeric_enum_macro::numeric_enum;
use spin::RwLock;
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
const CELL_SHUT_DOWN: u32 = 2;
const COMM_REGION_ABI_REVISION: u16 = 1;
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
        info!("Handel hvc disable");
        self.cpu_data.deactivate_vmm(0)?;
        unreachable!()
    }

    fn hypervisor_cell_create(&mut self, config_address: u64) -> HyperCallResult {
        info!(
            "handle hvc cell create, config_address = {:#x?}",
            config_address
        );
        //TODO should be read from config files

        let cell = self.cpu_data.cell.clone().unwrap();
        if !Arc::ptr_eq(&cell, &root_cell()) {
            return hv_result_err!(EPERM, "Creation over non-root cells: unsupported!");
        }
        info!("prepare to suspend root_cell");

        let root_cell = root_cell().clone();
        root_cell.read().suspend();

        // todo: 检查新cell是否和已有cell同id或同名
        let config_address = unsafe {
            cell.write()
                .gpm
                .page_table_query(config_address as usize)
                .unwrap()
                .0
        };
        let cfg_pages_offs = config_address as usize & (PAGE_SIZE - 1);
        let cfg_mapping = memory::hv_page_table().write().map_temporary(
            config_address,
            cfg_pages_offs + size_of::<HvCellDesc>(),
            MemFlags::READ,
        )?;

        let desc = unsafe {
            ((cfg_mapping + cfg_pages_offs) as *const HvCellDesc)
                .as_ref()
                .unwrap()
        };
        let config = CellConfig::new(desc);
        let config_total_size = config.total_size();
        memory::hv_page_table().write().map_temporary(
            config_address,
            cfg_pages_offs + config_total_size,
            MemFlags::READ,
        )?;
        info!("cell.desc = {:#x?}", desc);

        // we create the new cell here
        let cell = Cell::new(config)?;

        if cell.owns_cpu(this_cpu_data().id) {
            panic!("error: try to assign the CPU we are currently running on");
        }

        {
            let root_cell_r = root_cell.read();
            for id in cell.cpu_set.iter() {
                if !root_cell_r.owns_cpu(id) {
                    panic!("error: the root cell's cpu set must be super-set of new cell's set")
                }
            }
        }

        // todo: arch_cell_create

        let cpu_set = cell.cpu_set;
        info!("cell.cpu_set = {:#x?}", cell.cpu_set);
        let cell_p = Arc::new(RwLock::new(cell));
        {
            let mut root_cell_w = root_cell.write();
            cpu_set.iter().for_each(|cpu| {
                park_cpu(cpu);
                root_cell_w.cpu_set.clear_bit(cpu);
                get_cpu_data(cpu).cell = Some(cell_p.clone());
            });
        }

        // memory mapping

        {
            let mem_regs: Vec<HvMemoryRegion> = cell_p.read().config().mem_regions().to_vec();
            let mut cell = cell_p.write();
            // cell.comm_page.comm_region.cell_state = CELL_SHUT_DOWN;
            // let comm_page_pa = virt_to_phys(&cell.comm_page as *const _ as usize);
            let comm_page_pa = cell.comm_page.start_paddr();
            assert_eq!(comm_page_pa % 4096, 0);
            let root_gpm = &mut root_cell.write().gpm;

            mem_regs.iter().for_each(|mem| {
                if !(mem.flags.contains(MemFlags::COMMUNICATION)
                    || mem.flags.contains(MemFlags::ROOTSHARED))
                {
                    root_gpm
                        .unmap_partial(&MemoryRegion::new_with_offset_mapper(
                            mem.phys_start as _,
                            mem.phys_start as _,
                            mem.size as _,
                            mem.flags,
                        ))
                        .unwrap();
                }

                cell.gpm
                    .insert(MemoryRegion::from_hv_memregion(mem, Some(comm_page_pa)))
                    .unwrap();
            });
            // TODO: We should't add gic mapping to a cell, when mmio is finished, remove this.
            // add gicd & gicr mapping here
            cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
                0x8000000 as GuestPhysAddr,
                0x8000000 as HostPhysAddr,
                0x0200000 as usize,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
            // /* "physical" PCI ECAM */
            // cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
            //     0x7fb00000 as GuestPhysAddr,
            //     0x7fb00000 as HostPhysAddr,
            //     0x100000 as usize,
            //     MemFlags::READ | MemFlags::WRITE,
            // ))?;
        }

        add_cell(cell_p);
        root_cell.read().resume();

        info!("cell create done!");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_set_loadable(&mut self, cell_id: u64) -> HyperCallResult {
        let this_cpu_cell = self.cpu_data.cell.clone().unwrap();
        if !Arc::ptr_eq(&this_cpu_cell, &root_cell()) {
            return hv_result_err!(EPERM, "Operation over non-root cells: unsupported!");
        }
        let cell = match find_cell_by_id(cell_id as _) {
            Some(cell) => cell,
            None => return hv_result_err!(ENOENT),
        };
        if Arc::ptr_eq(&cell, &root_cell()) {
            return hv_result_err!(EINVAL, "Setting root-cell as loadable is not allowed!");
        }

        let mut cell_w = cell.write();
        if cell_w.loadable {
            return HyperCallResult::Ok(0);
        }

        cell_w.suspend();
        cell_w.cpu_set.iter().for_each(|cpu_id| park_cpu(cpu_id));
        cell_w.loadable = true;
        info!("cell.mem_regions() = {:#x?}", cell_w.config().mem_regions());
        let mem_regs: Vec<HvMemoryRegion> = cell_w.config().mem_regions().to_vec();

        // remap to rootcell
        let root_cell = root_cell();
        let root_gpm = &mut root_cell.write().gpm;
        mem_regs.iter().for_each(|mem| {
            if mem.flags.contains(MemFlags::LOADABLE) {
                root_gpm
                    .map_partial(&MemoryRegion::new_with_offset_mapper(
                        mem.phys_start as GuestPhysAddr,
                        mem.phys_start as HostPhysAddr,
                        mem.size as _,
                        mem.flags,
                    ))
                    .unwrap();
            }
        });
        info!("set loadbable done!");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_start(&mut self, cell_id: u64) -> HyperCallResult {
        info!("handle hvc cell start");

        let cell = match find_cell_by_id(cell_id as _) {
            Some(cell) => cell,
            None => return hv_result_err!(ENOENT),
        };
        // unsafe { assert!(*(0x7faf0000 as *mut u8) != 0x00) }
        // warn!("image = {:x?}", unsafe { *(0x7faf0000 as *const [u8; 64]) });
        cell.read().suspend();

        // set cell.comm_page
        unsafe {
            let mut cell_w = cell.write();
            cell_w.comm_page.fill(0);

            let flags = cell_w.config().flags();
            let console = cell_w.config().console();
            let comm_region = (cell_w.comm_page.as_mut_ptr() as *mut CommRegion)
                .as_mut()
                .unwrap();

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

        // todo: unmap from root cell

        // todo: set pc to `cpu_on_entry`
        // reset_cpu(cell.read().cpu_set.first_cpu().unwrap());

        let mut is_first = true;
        cell.read().cpu_set.iter().for_each(|cpu_id| {
            get_cpu_data(cpu_id).cpu_on_entry = if is_first {
                cell.read().config().cpu_reset_address()
            } else {
                INVALID_ADDRESS
            };
            is_first = false;
            reset_cpu(cpu_id);
        });
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_destroy(&mut self, _id: u64) -> HyperCallResult {
        info!("handle hvc cell destroy");
        let target_cpu = 3;
        send_event(target_cpu, SGI_RESUME_ID);
        HyperCallResult::Ok(0)
    }
}
