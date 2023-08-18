use aarch64_cpu::registers::MPIDR_EL1;

//use crate::arch::vcpu::Vcpu;
use crate::arch::entry::{shutdown_el2, virt2phys_el2, vmreturn};
use crate::arch::sysreg::{read_sysreg, smc_arg1, write_sysreg};
use crate::consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::device::gicv3::gicv3_cpu_init;
use crate::device::gicv3::gicv3_cpu_shutdown;
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
use core::arch::global_asm; // 支持内联汇编
global_asm!(include_str!("./arch/aarch64/page_table.S"),);
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
        info!("activating cpu {}", self.id);
        set_vtcr_flags();
        HCR_EL2.write(
            HCR_EL2::RW::EL1IsAarch64
                + HCR_EL2::TSC::EnableTrapSmcToEl2
                + HCR_EL2::VM::SET
                + HCR_EL2::IMO::SET
                + HCR_EL2::FMO::SET,
        );
        if (self.id == 3) {
            info!("test el1!");
            test_cpu_el1();
        } else {
            info!("stop here!");
            loop {}
        }
        self.return_linux()?;
        unreachable!()
    }
    pub fn deactivate_vmm(&mut self, ret_code: usize) -> HvResult {
        ACTIVATED_CPUS.fetch_sub(1, Ordering::SeqCst);
        info!("Disabling cpu {}", self.id);
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
        /*irqchip reset*/
        gicv3_cpu_shutdown();
        /* Free the guest */
        HCR_EL2.set(0x80000000);
        VTCR_EL2.set(0x80000000);
        /* Remove stage-2 mappings */
        unsafe {
            isb();
            arm_paging_vcpu_flush_tlbs();
        }
        /* TLB flush needs the cell's VMID */
        VTTBR_EL2.set(0);
        /* we will restore the root cell state with the MMU turned off,
         * so we need to make sure it has been committed to memory */

        /* hand over control of EL2 back to Linux */
        let linux_hyp_vec: u64 =
            unsafe { core::ptr::read_volatile(&HEADER_STUFF.arm_linux_hyp_vectors as *const _) };
        VBAR_EL2.set(linux_hyp_vec);
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

pub fn set_vtcr_flags() {
    let vtcr_flags = VTCR_EL2::TG0::Granule4KB
        + VTCR_EL2::PS::PA_44B_16TB
        + VTCR_EL2::SH0::Inner
        + VTCR_EL2::HA::Enabled
        + VTCR_EL2::SL0.val(2)
        + VTCR_EL2::ORGN0::NormalWBRAWA
        + VTCR_EL2::IRGN0::NormalWBRAWA
        + VTCR_EL2::T0SZ.val(20);

    VTCR_EL2.write(vtcr_flags);
}

pub unsafe extern "C" fn arm_paging_vcpu_flush_tlbs() {
    core::arch::asm!(
        "
            tlbi vmalls12e1is
        ",
    );
}

pub unsafe extern "C" fn isb() {
    core::arch::asm!(
        "
            isb
        ",
    );
}

pub fn test_cpu_el1() {
    info!("hello from el2");
    cpu_reset();
}
#[no_mangle]
fn cpu_reset() {
    /* put the cpu in a reset state */
    /* AARCH64_TODO: handle big endian support */
    write_sysreg!(CNTKCTL_EL1, 0);
    write_sysreg!(PMCR_EL0, 0);

    // /* AARCH64_TODO: wipe floating point registers */
    // /* wipe special registers */
    write_sysreg!(SP_EL0, 0);
    write_sysreg!(SP_EL1, 0);
    write_sysreg!(SPSR_EL1, 0);

    // /* wipe the system registers */
    write_sysreg!(AFSR0_EL1, 0);
    write_sysreg!(AFSR1_EL1, 0);
    write_sysreg!(AMAIR_EL1, 0);
    write_sysreg!(CONTEXTIDR_EL1, 0);
    write_sysreg!(CPACR_EL1, 0);
    write_sysreg!(CSSELR_EL1, 0);
    write_sysreg!(ESR_EL1, 0);
    write_sysreg!(FAR_EL1, 0);
    write_sysreg!(MAIR_EL1, 0);
    write_sysreg!(PAR_EL1, 0);
    write_sysreg!(TCR_EL1, 0);
    write_sysreg!(TPIDRRO_EL0, 0);
    write_sysreg!(TPIDR_EL0, 0);
    write_sysreg!(TPIDR_EL1, 0);
    write_sysreg!(TTBR0_EL1, 0);
    write_sysreg!(TTBR1_EL1, 0);
    write_sysreg!(VBAR_EL1, 0);

    /* wipe timer registers */
    write_sysreg!(CNTP_CTL_EL0, 0);
    write_sysreg!(CNTP_CVAL_EL0, 0);
    write_sysreg!(CNTP_TVAL_EL0, 0);
    write_sysreg!(CNTV_CTL_EL0, 0);
    write_sysreg!(CNTV_CVAL_EL0, 0);
    write_sysreg!(CNTV_TVAL_EL0, 0);
    // //disable stage 1
    // write_sysreg!(SCTLR_EL1, 0);
    //SCTLR_EL1.set(((1 << 11) | (1 << 20) | (3 << 22) | (3 << 28)));
    //SCTLR_EL1.modify(SCTLR_EL1::M::Disable);
    //HCR_EL2.modify(HCR_EL2::VM::Disable);
    unsafe {
        //isb();
        set_el1_pc();
    }
    //disable stage2
    //HCR_EL2.modify(HCR_EL2::VM::Disable);
}
#[no_mangle]
pub unsafe extern "C" fn set_el1_pc() -> i32 {
    //info!("Hello World! from el1");
    //set el1 pc
    unsafe {
        core::arch::asm!(
            "
            // mov	x0, #965
            // msr	SPSR_EL2, x0
            tlbi alle1is
            tlbi alle2is
            //x3 hva-hpa
            ldr x1,=0xffffc0200000
            ldr x2,=0x7fc00000
            sub	x3, x1, x2 
            adr x0,{entry} //hva
            //sub	x0, x0, x3 //hpa
            //add x0,x0,0x100000
            //ldr x0,=0x80100000

            msr	ELR_EL2, x0
            // mrs x4,vbar_el2
            // msr vbar_el1,x4
            //change stage 1 el1 mm
            mrs x4,ttbr0_el2
            msr ttbr0_el1,x4
            msr ttbr1_el1,x4
            eret
        ",
            entry = sym el1_test,
            options(noreturn),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn el1_test() -> i32 {
    //info!("Hello World! from el1");
    core::arch::asm!(
        "
        mov x0,#9
        ldr x10,[x2,#100]
        ldr x2,=0x7fd00000
        ldr x10,[x2,#100]
        ldr x2,=0xf7fd00000
        ldr x10,[x2,#100]
        ldr x2,=0xff7fd00000
        ldr x10,[x2,#100]
        ldr x2,=0xff7fd00000
        ldr x10,[x2,#100]
        ldr x2,=0xfff7fd00000
        ldr x10,[x2,#100]
        ldr x2,=0xfffffe00000
        ldr x10,[x2,#100]

        //stage2 phy addr over 44bit
        // ldr x2,=0x100000000000
        // ldr x10,[x2,#100]
        // ldr x2,=0x1fffffffffff
        // ldr x10,[x2,#100]
        // ldr x2,=0xffff00000000
        // ldr x10,[x2,#100]
        ldr x2,=0xffffc0200000
        ldr x10,[x2,#100]
        ldr x10,[x1,#100]
        hvc #0
        wfi
    ",
        options(noreturn),
    );
}
