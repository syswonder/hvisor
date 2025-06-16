use aarch64_cpu::registers::SCTLR_EL1;
use alloc::boxed::Box;
use alloc::sync::Weak;
use spin::RwLock;

use crate::arch::sysreg::write_sysreg;
use crate::consts::VCPU_STACK_SIZE;
use crate::vcpu::{current_vcpu, VCpu};
use aarch64_cpu::registers::Writeable;
use core::fmt::{Debug, Formatter};
use core::ptr::addr_of;

#[repr(C, align(4096))]
struct AArch64VCpuStack {
    _st: [u8; VCPU_STACK_SIZE],
}

impl Default for AArch64VCpuStack {
    fn default() -> Self {
        Self {
            _st: [0; VCPU_STACK_SIZE],
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct AArch64VCpuHyper {
    x: [u64; 31],
    sp: u64,
    _stack: Box<AArch64VCpuStack>,
}

impl Debug for AArch64VCpuHyper {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("AArch64VCpuHyper")
            .field("x", &self.x)
            .field("sp", &self.sp)
            .field("stack", &addr_of!(self._stack._st))
            .field("stack_size", &VCPU_STACK_SIZE)
            .finish()
    }
}

impl AArch64VCpuHyper {
    fn stack_bottom(&self) -> usize {
        addr_of!(self._stack._st) as usize + VCPU_STACK_SIZE
    }

    #[naked]
    #[no_mangle]
    unsafe extern "C" fn __switch_to(_hyper_regs: *const Self) -> ! {
        core::arch::asm!(
            "ldp x2, x3, [x0, #0x10]",
            "ldp x4, x5, [x0, #0x20]",
            "ldp x6, x7, [x0, #0x30]",
            "ldp x8, x9, [x0, #0x40]",
            "ldp x10, x11, [x0, #0x50]",
            "ldp x12, x13, [x0, #0x60]",
            "ldp x14, x15, [x0, #0x70]",
            "ldp x16, x17, [x0, #0x80]",
            "ldp x18, x19, [x0, #0x90]",
            "ldp x20, x21, [x0, #0xa0]",
            "ldp x22, x23, [x0, #0xb0]",
            "ldp x24, x25, [x0, #0xc0]",
            "ldp x26, x27, [x0, #0xd0]",
            "ldp x28, x29, [x0, #0xe0]",
            "ldp x30, x1, [x0, #0xf0]", // x1 = saved sp
            "mov sp, x1",
            "ldp x0, x1, [x0]",
            "ret",
            options(noreturn)
        );
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct AArch64VCpuGuest {
    x: [u64; 31],
    elr: u64,
    spsr: u64,
}

impl AArch64VCpuGuest {
    #[naked]
    #[no_mangle]
    unsafe extern "C" fn __vm_return(_guest_regs: *const Self, hyp_stack_bottom: u64) -> ! {
        core::arch::asm!(
            "mov sp, x1", // We need to set the stack pointer to the end of the stack here
            "msr TPIDR_EL2, x0",
            "ldp x2, x3, [x0, #0x10]",
            "ldp x4, x5, [x0, #0x20]",
            "ldp x6, x7, [x0, #0x30]",
            "ldp x8, x9, [x0, #0x40]",
            "ldp x10, x11, [x0, #0x50]",
            "ldp x12, x13, [x0, #0x60]",
            "ldp x14, x15, [x0, #0x70]",
            "ldp x16, x17, [x0, #0x80]",
            "ldp x18, x19, [x0, #0x90]",
            "ldp x20, x21, [x0, #0xa0]",
            "ldp x22, x23, [x0, #0xb0]",
            "ldp x24, x25, [x0, #0xc0]",
            "ldp x26, x27, [x0, #0xd0]",
            "ldp x28, x29, [x0, #0xe0]",
            "ldr x30, [x0, #0xf0]",
            "ldr x1, [x0, #0xf8]",
            "msr ELR_EL2, x1",
            "ldr x1, [x0, #0x100]",
            "msr SPSR_EL2, x1",
            "ldp x0, x1, [x0]",
            "eret",
            options(noreturn)
        );
    }

    unsafe extern "C" fn __vm_exit() -> ! {
        core::arch::asm!(
            "mrs x0, TPIDR_EL2",
            "stp x0, x1, [x0]",
            "stp x2, x3, [x0, #0x10]",
            "stp x4, x5, [x0, #0x20]",
            "stp x6, x7, [x0, #0x30]",
            "stp x8, x9, [x0, #0x40]",
            "stp x10, x11, [x0, #0x50]",
            "stp x12, x13, [x0, #0x60]",
            "stp x14, x15, [x0, #0x70]",
            "stp x16, x17, [x0, #0x80]",
            "stp x18, x19, [x0, #0x90]",
            "stp x20, x21, [x0, #0xa0]",
            "stp x22, x23, [x0, #0xb0]",
            "stp x24, x25, [x0, #0xc0]",
            "stp x26, x27, [x0, #0xd0]",
            "stp x28, x29, [x0, #0xe0]",
            "str x30, [x0, #0xf0]",
            "mrs x1, ELR_EL2",
            "str x1, [x0, #0xf8]",
            "mrs x1, SPSR_EL2",
            "str x1, [x0, #0x100]",
            "stp x0, x1, [x0]",
            "bl testtrap",
            options(noreturn)
        );
    }
}

#[no_mangle]
fn testtrap() {
    println!("testtrap!\n");
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct AArch64VCpu {
    hyper_regs: AArch64VCpuHyper,
    guest_regs: AArch64VCpuGuest,
}

impl AArch64VCpu {
    pub fn new() -> Self {
        let mut vcpu = Self::default();
        // set stack pointer to the end of the stack
        vcpu.hyper_regs.sp = vcpu.hyper_regs.stack_bottom() as _;
        vcpu.hyper_regs.x[30] = arch_vcpu_hyp_entry as _;
        vcpu
    }

    fn reset_vm_regs() {
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
        write_sysreg!(CNTVOFF_EL2, 0);
        write_sysreg!(CNTP_CTL_EL0, 0);
        write_sysreg!(CNTP_CVAL_EL0, 0);
        write_sysreg!(CNTP_TVAL_EL0, 0);
        write_sysreg!(CNTV_CTL_EL0, 0);
        write_sysreg!(CNTV_CVAL_EL0, 0);
        write_sysreg!(CNTV_TVAL_EL0, 0);
        // //disable stage 1
        // write_sysreg!(SCTLR_EL1, 0);

        SCTLR_EL1.set((1 << 11) | (1 << 20) | (3 << 22) | (3 << 28));
    }
}

fn arch_vcpu_hyp_entry() -> ! {
    let vcpu = current_vcpu().upgrade().unwrap().clone();
    let mut vcpu_lock = vcpu.write();
    info!("AArch64 VCPU {} is running", vcpu_lock.id);

    vcpu_lock.arch.guest_regs.elr = 0xa0400000;
    vcpu_lock.arch.guest_regs.x[0] = 0xa0000000;
    vcpu_lock.arch.guest_regs.spsr = 0x3c5;

    vcpu_lock.activate_gpm();
    AArch64VCpu::reset_vm_regs();
    let guest_regs = &vcpu_lock.arch.guest_regs as *const _;
    let stack_bottom = vcpu_lock.arch.hyper_regs.stack_bottom();

    drop(vcpu_lock);
    drop(vcpu);

    unsafe {
        AArch64VCpuGuest::__vm_return(guest_regs, stack_bottom as _);
    }
}

pub fn arch_switch_to_vcpu(vcpu: Weak<RwLock<VCpu>>) {
    let vcpu = vcpu.upgrade().unwrap();
    let vcpu_lock = vcpu.read();
    let hyper_regs = &vcpu_lock.arch.hyper_regs as *const _;
    drop(vcpu_lock);
    drop(vcpu);

    unsafe {
        AArch64VCpuHyper::__switch_to(hyper_regs);
    }
}

pub type ArchVCpu = AArch64VCpu;
