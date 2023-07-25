use crate::arch::sysreg::write_sysreg;
use crate::error::HvResult;
use crate::percpu::PerCpu;
use bit_field::BitField;
use core::convert::TryFrom;
use core::sync::atomic::{AtomicU32, Ordering};
use numeric_enum_macro::numeric_enum;
numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum HyperCallCode {
        HypervisorDisable = 0,
        HypervisorCellCreate = 1,
    }
}

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
            HyperCallCode::HypervisorCellCreate => self.hypervisor_cell_create(),
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
    fn hypervisor_cell_create(&mut self) -> HyperCallResult {
        info!("handle hvc cell create");
        //TODO should be read from config files
        let target_cpu = 3;
        arch_send_event(target_cpu);
        HyperCallResult::Ok(0)
    }
}
pub const SGI_HV_ID: u64 = 15;
pub fn arch_send_event(cpuid: u64) -> HvResult {
    let aff3: u64 = 0 << 48;
    let aff2: u64 = 0 << 32;
    let aff1: u64 = 0 << 16;
    let irm: u64 = 0 << 40;
    let sgi_id: u64 = SGI_HV_ID << 24;
    let target_list: u64 = 1 << cpuid;
    let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
    unsafe {
        write_sysreg!(icc_sgi1r_el1, val);
    }
    Ok(())
}
