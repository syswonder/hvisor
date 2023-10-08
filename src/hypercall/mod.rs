use crate::cell::root_cell;
use crate::config::HvCellDesc;
use crate::control::arch_send_event;
use crate::error::HvResult;
use crate::percpu::PerCpu;
use alloc::sync::Arc;
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
        HypervisorCellLoad = 3,
        HypervisorCellDestroy = 4,
    }
}

pub const SGI_HV_ID: u64 = 15;
pub const SGI_RESUME_ID: u64 = 14;

pub type HyperCallResult = HvResult<usize>;

pub struct HyperCall<'a> {
    cpu_data: &'a mut PerCpu,
}

impl<'a> HyperCall<'a> {
    pub fn new(cpu_data: &'a mut PerCpu) -> Self {
        Self { cpu_data }
    }

    pub fn hypercall(&mut self, code: u64, arg0: u64, _arg1: u64) -> HvResult {
        let code = match HyperCallCode::try_from(code) {
            Ok(code) => code,
            Err(_) => {
                warn!("hypercall unsupported!");
                return Ok(());
            }
        };
        let _ret = match code {
            HyperCallCode::HypervisorDisable => self.hypervisor_disable(),
            HyperCallCode::HypervisorCellCreate => self.hypervisor_cell_create(arg0),
            HyperCallCode::HypervisorCellLoad => self.hypervisor_cell_load(arg0),
            HyperCallCode::HypervisorCellStart => self.hypervisor_cell_start(arg0),
            HyperCallCode::HypervisorCellDestroy => self.hypervisor_cell_destroy(arg0),
        };
        Ok(())
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
        info!("handle hvc cell create, config_address = {:#x?}", config_address);
        //TODO should be read from config files

        let cell = self.cpu_data.cell.clone().unwrap();
        if !Arc::ptr_eq(&cell, &root_cell()) {
            warn!("Creation over non-root cells: unsupported!");
            return HyperCallResult::Err(hv_err!(EPERM));
        }
        warn!("prepare to suspend root_cell");

        let root_cell = root_cell().clone();
        root_cell.read().suspend();
        let cfg_page_offs = config_address as usize & 0xfff;

        // 根据 Jailhouse 的方法，这里应该将 config_address（一个客户机的物理地址）映射到当前 CPU 的虚拟地址空间中。
        // 然而，我发现可以直接访问这个物理地址，所以没有进行映射操作。不过目前还不确定这样做是否会引起问题。
        let cfg = unsafe { *(config_address as *const HvCellDesc) };

        // todo: 检查新cell是否和已有cell同id或同名
        
        // let cell_w = cell.write();
        // cell_w.gpm.insert(MemoryRegion::new_with_empty_mapper(
        //     config_address as usize,
        //     cfg_pages_offs + size_of::<HvCellDesc>(),
        //     MemFlags::READ,
        // ))?;

        let cfg_total_size = cfg.config_size() + size_of::<HvCellDesc>();
        warn!("cfg = {:#x?}", cfg);
        warn!("cfg_total_size = {:#x?}", cfg_total_size);
        
        // let target_cpu = 3;
        // arch_send_event(target_cpu, SGI_HV_ID);
        info!("prepare to return!");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_load(&mut self, _id: u64) -> HyperCallResult {
        info!("handle hvc cell load");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_start(&mut self, _id: u64) -> HyperCallResult {
        info!("handle hvc cell start");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_destroy(&mut self, _id: u64) -> HyperCallResult {
        info!("handle hvc cell destroy");
        let target_cpu = 3;
        arch_send_event(target_cpu, SGI_RESUME_ID);
        HyperCallResult::Ok(0)
    }
}

