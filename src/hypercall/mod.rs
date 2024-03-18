#![allow(dead_code)]
use crate::cell::{add_cell, find_cell_by_id, remove_cell, root_cell, Cell, CommRegion};
use crate::config::{CellConfig, HvCellDesc, HvMemoryRegion, HvSystemConfig};
use crate::consts::{INVALID_ADDRESS, PAGE_SIZE};
use crate::control::{park_cpu, reset_cpu, resume_cpu, send_event};
use crate::device::pci::mmio_pci_handler;
use crate::device::virtio_trampoline::VIRTIO_RESULT_MAP;
use crate::device::virtio_trampoline::{HVISOR_DEVICE, MAX_REQ};
use crate::error::HvResult;
use crate::memory::addr::{align_down, align_up, is_aligned};
use crate::memory::{self, MemFlags, MemoryRegion, EMU_SHARED_REGION_BASE};
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
        HypervisorInitVirtio = 9,
        HypervisorFinishReq = 10,
    }
}

pub const SGI_INJECT_ID: u64 = 0;
pub const SGI_VIRTIO_RES_ID: u64 = 9;
pub const SGI_RESUME_ID: u64 = 14;
pub const SGI_EVENT_ID: u64 = 15;
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
            HyperCallCode::HypervisorInitVirtio => self.hypervisor_init_virtio(arg0),
            HyperCallCode::HypervisorFinishReq => self.hypervisor_finish_req(),
        }
    }

    // Send virtio req result to non root. Only root cell calls.
    fn hypervisor_finish_req(&self) -> HyperCallResult {
        let cell = self.cpu_data.cell.clone().unwrap();
        if !Arc::ptr_eq(&cell, &root_cell()) {
            return hv_result_err!(
                EPERM,
                "Virtio finish operation over non-root cells: unsupported!"
            );
        }
        let dev = HVISOR_DEVICE.lock();
        let mut map = VIRTIO_RESULT_MAP.lock();
        let region = dev.region();
        while !dev.is_res_list_empty() {
            let res_front = region.res_front as usize;
            let value = region.res_list[res_front].value;
            let target = region.res_list[res_front].target;
            let res_type = region.res_list[res_front].res_type;
            match res_type {
                0 => {
                    map.insert(target, value);
                    resume_cpu(target);
                    debug!("res_type: 0, value is {}", value);
                }
                1 => {
                    map.insert(target, value);
                    send_event(target, SGI_VIRTIO_RES_ID);
                    debug!("res_type: 1, value is {}", value);
                }
                2 => {
                    let cell = find_cell_by_id(target as u32).unwrap();
                    let tar_cpu = cell.read().cpu_set.first_cpu().unwrap();
                    map.insert(tar_cpu, value);
                    send_event(tar_cpu, SGI_VIRTIO_RES_ID);
                    debug!("res_type: 2, value is {}", value);
                }
                _ => panic!("res_type is invalid"),
            }
            region.res_front = (region.res_front + 1) & (MAX_REQ - 1);
        }
        drop(dev);
        HyperCallResult::Ok(0)
    }

    // only root cell calls the function and set virtio shared region between el1 and el2.
    fn hypervisor_init_virtio(&mut self, shared_region_addr: u64) -> HyperCallResult {
        debug!(
            "handle hvc init virtio, shared_region_addr = {:#x?}",
            shared_region_addr
        );
        let cell = self.cpu_data.cell.clone().unwrap();
        if !Arc::ptr_eq(&cell, &root_cell()) {
            return hv_result_err!(EPERM, "Init virtio over non-root cells: unsupported!");
        }
        let shared_region_addr_pa = cell.read().gpm_query(shared_region_addr as _);
        let offset = shared_region_addr_pa & (PAGE_SIZE - 1);
        memory::hv_page_table()
            .write()
            .insert(MemoryRegion::new_with_offset_mapper(
                EMU_SHARED_REGION_BASE,
                align_down(shared_region_addr_pa),
                PAGE_SIZE,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        HVISOR_DEVICE
            .lock()
            .set_base_addr(EMU_SHARED_REGION_BASE + offset);
        HyperCallResult::Ok(0)
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

        let desc = unsafe {
            ((cfg_mapping + cfg_pages_offs) as *const HvCellDesc)
                .as_ref()
                .unwrap()
        };
        let config = CellConfig::new(desc);
        let config_total_size = config.total_size();
        memory::hv_page_table().write().map_temporary(
            align_down(config_address),
            align_up(cfg_pages_offs + config_total_size),
            MemFlags::READ,
        )?;
        trace!("cell.desc = {:#x?}", desc);

        // we create the new cell here
        let cell = Cell::new(config, false)?;

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

        let mut cell = cell_p.write();

        let mem_regs: Vec<HvMemoryRegion> = cell.config().mem_regions().to_vec();
        // cell.comm_page.comm_region.cell_state = CELL_SHUT_DOWN;

        let comm_page_pa = cell.comm_page.start_paddr();
        assert!(is_aligned(comm_page_pa));

        // let mut rc_w = root_cell.write();
        // 为什么这里需要unmap
        mem_regs.iter().for_each(|mem| {
            // if !(mem.flags.contains(MemFlags::COMMUNICATION)
            //     || mem.flags.contains(MemFlags::ROOTSHARED))
            // {
            //     rc_w.mem_region_unmap_partial(&MemoryRegion::new_with_offset_mapper(
            //         mem.phys_start as _,
            //         mem.phys_start as _,
            //         mem.size as _,
            //         mem.flags,
            //     ));
            // }

            cell.mem_region_insert(MemoryRegion::from_hv_memregion(mem, Some(comm_page_pa)))
        });

        // drop(rc_w);
        // add pci mapping
        let sys_config = HvSystemConfig::get();
        let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
        let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;
        cell.mmio_region_register(mmcfg_start as _, mmcfg_size, mmio_pci_handler, mmcfg_start);

        cell.gicv3_config_commit();
        drop(cell);

        add_cell(cell_p);
        root_cell.read().resume();

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
        trace!("cell.mem_regions() = {:#x?}", cell_w.config().mem_regions());
        let mem_regs: Vec<HvMemoryRegion> = cell_w.config().mem_regions().to_vec();

        // remap to rootcell
        let root_cell = root_cell();
        let root_cell_w = root_cell.write();

        mem_regs.iter().for_each(|mem| {
            if mem.flags.contains(MemFlags::LOADABLE) {
                // root_cell_w.mem_region_map_partial(&MemoryRegion::new_with_offset_mapper(
                //     mem.phys_start as GuestPhysAddr,
                //     mem.phys_start as HostPhysAddr,
                //     mem.size as _,
                //     mem.flags,
                // ));
            }
        });
        root_cell_w.resume();
        info!("set loadbable done!");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_start(&mut self, cell_id: u64) -> HyperCallResult {
        info!("handle hvc cell start");
        let cell = cell_management_prologue(self.cpu_data, cell_id)?;
        let mut cell_w = cell.write();
        // set cell.comm_page
        {
            cell_w.comm_page.fill(0);

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
        if cell_w.loadable {
            let mem_regs: Vec<HvMemoryRegion> = cell_w.config().mem_regions().to_vec();
            // let root_cell = root_cell();
            // let root_cell_w = root_cell.write();
            mem_regs.iter().for_each(|mem| {
                if mem.flags.contains(MemFlags::LOADABLE) {
                    // root_cell_w.mem_region_unmap_partial(&MemoryRegion::new_with_offset_mapper(
                    //     mem.phys_start as GuestPhysAddr,
                    //     mem.phys_start as HostPhysAddr,
                    //     mem.size as _,
                    //     mem.flags,
                    // ));
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
        root_cell().read().resume();
        info!("start cell done!");
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
        cell_w.gicv3_config_commit();
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

/// check and suspend root_cell and new_cell.
fn cell_management_prologue(cpu_data: &mut PerCpu, cell_id: u64) -> HvResult<Arc<RwLock<Cell>>> {
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
