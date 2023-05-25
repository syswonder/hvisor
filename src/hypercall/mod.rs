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
                return Ok(());
            }
        };
        let ret = match code {
            HyperCallCode::HypervisorDisable => self.hypervisor_disable(),
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

        self.cpu_data.deactivate_vmm(0)?;
        unreachable!()
    }
}
