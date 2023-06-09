use aarch64_cpu::registers::MPIDR_EL1;

//use crate::arch::vcpu::Vcpu;
use crate::arch::entry::{shutdown_el2, virt2phys_el2, vmreturn};
use crate::consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::error::HvResult;
use crate::header::HvHeader;
use crate::header::{HvHeaderStuff, HEADER_STUFF};
use crate::memory::addr::VirtAddr;
use aarch64_cpu::{asm, registers::*};
use core::fmt::{Debug, Formatter, Result};
use core::sync::atomic::{AtomicU32, Ordering};
use tock_registers::interfaces::*;
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static ACTIVATED_CPUS: AtomicU32 = AtomicU32::new(0);
#[repr(C)]
#[derive(Debug, Default)]
pub struct GeneralRegisters {
    pub exit_reason: u64,
    pub usr: [u64; 31],
}
#[repr(C)]
pub struct PerCpu {
    pub id: u64,
    /// Referenced by arch::cpu::thread_pointer() for x86_64.
    pub self_vaddr: VirtAddr,
    //guest_regs: GeneralRegisters, //should be in vcpu
    pub wait_for_poweron: bool,
    // Stack will be placed here.
}

impl PerCpu {
    pub fn new<'a>(cpu_id: u64) -> HvResult<&'a mut Self> {
        let _cpu_rank = ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = unsafe { &mut *(vaddr as *mut Self) };
        ret.id = cpu_id;
        ret.self_vaddr = vaddr;
        ret.wait_for_poweron = false;
        Ok(ret)
    }

    pub fn stack_top(&self) -> VirtAddr {
        self as *const _ as VirtAddr + PER_CPU_SIZE - 8
    }

    pub fn guest_reg(&self) -> VirtAddr {
        self as *const _ as VirtAddr + PER_CPU_SIZE - 8 - 32 * 8
    }
    pub fn entered_cpus() -> u32 {
        ENTERED_CPUS.load(Ordering::Acquire)
    }
    pub fn activated_cpus() -> u32 {
        ACTIVATED_CPUS.load(Ordering::Acquire)
    }
    pub fn activate_vmm(&mut self) -> HvResult {
        ACTIVATED_CPUS.fetch_add(1, Ordering::SeqCst);
        HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64 + HCR_EL2::TSC::EnableTrapSmcToEl2);
        self.return_linux()?;
        unreachable!()
    }
    pub fn deactivate_vmm(&mut self, ret_code: usize) -> HvResult {
        ACTIVATED_CPUS.fetch_sub(1, Ordering::SeqCst);
        info!("Disabling cpu{}", self.id);
        self.arch_shutdown_self();
        Ok(())
    }
    pub fn return_linux(&mut self) -> HvResult {
        unsafe {
            vmreturn(self.guest_reg());
        }
        Ok(())
    }
    /*should be in vcpu*/
    pub fn arch_shutdown_self(&mut self) -> HvResult {
        /* Free the guest */

        /* Remove stage-2 mappings */

        /* TLB flush needs the cell's VMID */

        /* we will restore the root cell state with the MMU turned off,
         * so we need to make sure it has been committed to memory */

        /* hand over control of EL2 back to Linux */
        let linux_hyp_vec: u64 =
            unsafe { core::ptr::read_volatile(&HEADER_STUFF.arm_linux_hyp_vectors as *const _) };
        unsafe {
            core::arch::asm!(
                "
                msr vbar_el2,{linux_hyp_vec}
        ",
                linux_hyp_vec= in(reg) linux_hyp_vec,
            );
        }

        /* Return to EL1 */
        /* Disable mmu */

        unsafe {
            let page_offset: u64 = 0xffff_4060_0000;
            virt2phys_el2(self.guest_reg(), page_offset);
        }
        Ok(())
    }
}

pub fn this_cpu_data<'a>() -> &'a mut PerCpu {
    /*per cpu data should be handled after final el2 paging init
    now just only cpu 0*/
    /*arm_read_sysreg(MPIDR_EL1, mpidr);
    return mpidr & MPIDR_CPUID_MASK;*/
    let mpidr = MPIDR_EL1.get();

    let cpu_id = mpidr & 0xff00ffffff;
    let cpu_data: usize = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
    unsafe { &mut *(cpu_data as *mut PerCpu) }
}
pub fn get_cpu_data<'a>(cpu_id: u64) -> &'a mut PerCpu {
    let cpu_data: usize = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
    unsafe { &mut *(cpu_data as *mut PerCpu) }
}
