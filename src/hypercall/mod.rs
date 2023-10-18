use crate::cell::{add_cell, find_cell_by_id, root_cell, Cell};
use crate::config::{CellConfig, HvCellDesc, HvMemoryRegion};
use crate::control::{park_cpu, reset_cpu, send_event};
use crate::error::HvResult;
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion};
use crate::percpu::{get_cpu_data, this_cpu_data, PerCpu};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryFrom;
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

        // 根据 Jailhouse 的方法，这里应该将 config_address（一个客户机的物理地址）映射到当前 CPU 的虚拟地址空间中。
        // 然而，我发现可以直接访问这个物理地址，所以没有进行映射操作。不过目前还不确定这样做是否会引起问题。
        let desc = unsafe { (config_address as *const HvCellDesc).as_ref().unwrap() };

        // todo: 检查新cell是否和已有cell同id或同名

        // let cell_w = cell.write();
        // cell_w.gpm.insert(MemoryRegion::new_with_empty_mapper(
        //     config_address as usize,
        //     cfg_pages_offs + size_of::<HvCellDesc>(),
        //     MemFlags::READ,
        // ))?;
        let config = CellConfig::new(desc);
        let config_total_size = config.total_size();
        info!("cell.desc = {:#x?}", desc);

        // we create the new cell here
        let cell = Cell::new(config)?;

        if cell.owns_cpu(this_cpu_data().id) {
            panic!("error: try to assign the CPU we are currently running on");
        }

        {
            let cpu_set = cell.cpu_set;
            let root_cell_lock = root_cell.read();
            for id in cell.cpu_set.iter() {
                if !root_cell_lock.owns_cpu(id) {
                    panic!("error: the root cell's cpu set must be super-set of new cell's set")
                }
            }
        }

        // todo: arch_cell_create

        let cpu_set = cell.cpu_set;
        info!("cell.cpu_set = {:#x?}", cell.cpu_set);
        let cell_p = Arc::new(RwLock::new(cell));
        {
            let mut root_cell_lock = root_cell.write();
            cpu_set.iter().for_each(|cpu| {
                park_cpu(cpu);
                root_cell_lock.cpu_set.clear_bit(cpu);
                get_cpu_data(cpu).cell = Some(cell_p.clone());
            });
        }

        // memory mapping

        {
            let mem_regs: Vec<HvMemoryRegion> = cell_p.read().config().mem_regions().to_vec();
            let mut cell = cell_p.write();
            let comm_page_pa = cell.comm_page.start_paddr();
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
                    .unwrap()
            });
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
        let mut cell_lock = cell.write();
        cell_lock.suspend();
        cell_lock.cpu_set.iter().for_each(|cpu_id| park_cpu(cpu_id));
        cell_lock.loadable = true;
        info!(
            "cell.mem_regions() = {:#x?}",
            cell_lock.config().mem_regions()
        );
        let mem_regs: Vec<HvMemoryRegion> = cell_lock.config().mem_regions().to_vec();

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
        unsafe { assert!(*(0x7faf0000 as *mut u8) != 0x00) }
        warn!("image = {:x?}", unsafe { *(0x7faf0000 as *const [u8; 64]) });
        cell.read().suspend();

        // set cell.comm_page
        {
            let mut cell_lock = cell.write();
            cell_lock.comm_page.fill(0);
            cell_lock.comm_page.copy_data_from("JHCOMM".as_bytes());
            cell_lock.comm_page.as_slice_mut()[6] = 0x01;
        }

        // todo: unmap from root cell

        // todo: set pc to `cpu_on_entry`
        cell.read().cpu_set.iter().for_each(|cpu_id| {
            reset_cpu(cpu_id);
        });
        cell.read().resume();
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_destroy(&mut self, _id: u64) -> HyperCallResult {
        info!("handle hvc cell destroy");
        let target_cpu = 3;
        send_event(target_cpu, SGI_RESUME_ID);
        HyperCallResult::Ok(0)
    }
}
