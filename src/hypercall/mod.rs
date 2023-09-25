use crate::arch::sysreg::write_sysreg;
use crate::cell::Cell;
use crate::config::HvCellDesc;
use crate::error::HvResult;
use crate::memory::{MemoryRegion, MemFlags};
use crate::percpu::PerCpu;
use bit_field::BitField;
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
        let ret = match code {
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
        info!("handle hvc cell create");
        //TODO should be read from config files

        // todo: check if at root cell

        // todo: cell_suspend(root cell)

        // let cfg_pages_offs = config_address as usize & PAGE_OFFS_MASK;
        // let cfg_pages = pages(cfg_pages_offs + size_of::<HvCellDesc>());
        // let cell_some = self.cpu_data.cell;
        
        // let cell = match self.cpu_data.cell {
        //     Some(cell_ptr) => cell_ptr,
        //     None => {
        //         warn!("null cell point!");
        //         todo!();
        //     }
        // };
        // cell.gpm.insert(MemoryRegion::new_with_empty_mapper(
        //     config_address as usize,
        //     cfg_pages_offs + size_of::<HvCellDesc>(), 
        //     MemFlags::READ,
        // ))?;

        // let cfg_total_size: 

        let target_cpu = 3;
        arch_send_event(target_cpu, SGI_HV_ID);
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_load(&mut self, id: u64) -> HyperCallResult {
        info!("handle hvc cell load");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_start(&mut self, id: u64) -> HyperCallResult {
        info!("handle hvc cell start");
        HyperCallResult::Ok(0)
    }

    fn hypervisor_cell_destroy(&mut self, id: u64) -> HyperCallResult {
        info!("handle hvc cell destroy");
        let target_cpu = 3;
        arch_send_event(target_cpu, SGI_RESUME_ID);
        HyperCallResult::Ok(0)
    }
}

pub fn arch_send_event(cpuid: u64, sgi_num: u64) -> HvResult {
    //TODO: add more info
    let aff3: u64 = 0 << 48;
    let aff2: u64 = 0 << 32;
    let aff1: u64 = 0 << 16;
    let irm: u64 = 0 << 40;
    let sgi_id: u64 = sgi_num << 24;
    let target_list: u64 = 1 << cpuid;
    let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
    unsafe {
        write_sysreg!(icc_sgi1r_el1, val);
    }
    info!("write sgi sys value = {:#x}", val);
    Ok(())
}
