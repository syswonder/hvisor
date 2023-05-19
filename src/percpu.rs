//use crate::arch::vcpu::Vcpu;
use crate::arch::entry::vmreturn;
use crate::consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::error::HvResult;
use crate::header::HvHeader;
use crate::memory::addr::VirtAddr;
use core::fmt::{Debug, Formatter, Result};
use core::sync::atomic::{AtomicU32, Ordering};
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static ACTIVATED_CPUS: AtomicU32 = AtomicU32::new(0);
#[repr(C)]
#[derive(Debug, Default)]
pub struct GeneralRegisters {
    pub exit_reason: u64,
    pub usr: [u64; 31],
}
#[repr(C, align(4096))]
pub struct PerCpu {
    /// Referenced by arch::cpu::thread_pointer() for x86_64.
    self_vaddr: VirtAddr,
    //guest_regs: GeneralRegisters, //should be in vcpu
    pub id: u32,
    // Stack will be placed here.
}

impl PerCpu {
    pub fn new<'a>() -> HvResult<&'a mut Self> {
        let cpu_id = ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = unsafe { &mut *(vaddr as *mut Self) };
        ret.id = cpu_id;
        ret.self_vaddr = vaddr;
        Ok(ret)
    }

    pub fn stack_top(&self) -> VirtAddr {
        self as *const _ as VirtAddr + PER_CPU_SIZE - 8
    }
    /*
    pub fn guest_reg(&self) -> VirtAddr {
        &self.guest_regs as *const _ as VirtAddr
    }
    */
    pub fn activate_vmm(&mut self) -> HvResult {
        ACTIVATED_CPUS.fetch_add(1, Ordering::SeqCst);
        self.return_linux()?;
        unreachable!()
    }
    pub fn return_linux(&mut self) -> HvResult {
        unsafe {
            vmreturn(self.stack_top());
        }
        Ok(())
    }
}
