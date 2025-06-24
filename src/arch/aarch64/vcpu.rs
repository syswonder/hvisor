use crate::arch::sysreg::write_sysreg;
use crate::consts::VCPU_STACK_SIZE;
use crate::scheduler::add_vcpu_to_scheduler;
use crate::vcpu::{current_vcpu, VCpu};
use aarch64_cpu::registers::Writeable;
use aarch64_cpu::registers::SCTLR_EL1;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::fmt::{self, Debug, Formatter};
use core::ops::{Index, IndexMut, Range};
use core::ptr::addr_of;
use core::sync::atomic::AtomicBool;

#[repr(C, align(4096))]
struct AArch64VCpuStack {
    _st: [u8; VCPU_STACK_SIZE],
}

impl AArch64VCpuStack {
    fn lower_bound(&self) -> *const u8 {
        addr_of!(self._st) as *const u8
    }

    fn upper_bound(&self) -> *const u8 {
        unsafe { self.lower_bound().add(VCPU_STACK_SIZE) }
    }
}

impl Default for AArch64VCpuStack {
    fn default() -> Self {
        Self {
            _st: [0; VCPU_STACK_SIZE],
        }
    }
}

impl Debug for AArch64VCpuStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        // write stack addr range as format [a..b]
        let addr_range = format!(
            "[0x{:x}..0x{:x}]",
            self.lower_bound() as usize,
            self.upper_bound() as usize
        );
        f.debug_struct("AArch64VCpuHyperStack")
            .field("stack_addr_range", &addr_range)
            .finish()
    }
}

#[repr(C)]
#[derive(Default)]
pub struct AArch64GenericRegs {
    x: [u64; 31],
}

impl Index<usize> for AArch64GenericRegs {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= 31 {
            panic!("Index out of bounds: {} (valid range 0-30)", index);
        }
        &self.x[index]
    }
}

impl IndexMut<usize> for AArch64GenericRegs {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= 31 {
            panic!("Index out of bounds: {} (valid range 0-30)", index);
        }
        &mut self.x[index]
    }
}

impl Index<Range<usize>> for AArch64GenericRegs {
    type Output = [u64];
    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.x[range]
    }
}

impl fmt::Debug for AArch64GenericRegs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Collect non-zero registers
        let mut non_zero = Vec::new();
        let mut all_zero = true;

        for (i, &val) in self.x.iter().enumerate() {
            if val != 0 {
                non_zero.push((i, val));
                all_zero = false;
            }
        }

        // Print non-zero registers
        for (i, val) in &non_zero {
            write!(f, "x{}: 0x{:016x} ", i, val)?;
        }

        // Handle zero registers
        if !all_zero {
            let zero_regs: Vec<_> = (0..31)
                .filter(|i| !non_zero.iter().any(|(j, _)| *j == *i))
                .collect();

            if !zero_regs.is_empty() {
                write!(
                    f,
                    "(x{} all zero)",
                    zero_regs
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
        } else {
            write!(f, "All x registers are zero")?;
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Default)]
struct AArch64VCpuContext {
    x: AArch64GenericRegs,
    sp: u64,
}

impl Debug for AArch64VCpuContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("AArch64VCpuHyper")
            .field("x", &self.x)
            .field("sp", &self.sp)
            .finish()
    }
}

impl AArch64VCpuContext {
    #[naked]
    #[no_mangle]
    unsafe extern "C" fn __switch_to(_context: *const Self) -> ! {
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
pub struct AArch64TrapFrame {
    pub x: AArch64GenericRegs,
    pub elr: u64,
    pub spsr: u64,
}

impl AArch64TrapFrame {
    #[naked]
    #[no_mangle]
    unsafe extern "C" fn __vm_entry(_trapframe: *const Self) -> ! {
        core::arch::asm!(
            "mov sp, x0", // We need to set the stack pointer to current trapframe here
            "ldp x2, x3, [sp, #0x10]",
            "ldp x4, x5, [sp, #0x20]",
            "ldp x6, x7, [sp, #0x30]",
            "ldp x8, x9, [sp, #0x40]",
            "ldp x10, x11, [sp, #0x50]",
            "ldp x12, x13, [sp, #0x60]",
            "ldp x14, x15, [sp, #0x70]",
            "ldp x16, x17, [sp, #0x80]",
            "ldp x18, x19, [sp, #0x90]",
            "ldp x20, x21, [sp, #0xa0]",
            "ldp x22, x23, [sp, #0xb0]",
            "ldp x24, x25, [sp, #0xc0]",
            "ldp x26, x27, [sp, #0xd0]",
            "ldp x28, x29, [sp, #0xe0]",
            "ldp x30, x1, [sp, #0xf0]",
            "msr ELR_EL2, x1",
            "ldr x1, [x0, #0x100]",
            "msr SPSR_EL2, x1",
            "ldp x0, x1, [sp]",
            "eret",
            options(noreturn)
        );
    }
}

#[repr(C)]
#[derive(Default)]
pub struct AArch64VCpu {
    context: UnsafeCell<Box<AArch64VCpuContext>>,
    stack: Box<AArch64VCpuStack>,
    pub psci_on: AtomicBool,
}

impl Debug for AArch64VCpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("AArch64VCpu")
            .field("context", &self.context())
            .field("trapframe", &self.trapframe())
            .field("stack", &self.stack)
            .finish()
    }
}

impl AArch64VCpu {
    pub fn new() -> Self {
        let arch = Self::default();
        let context = arch.context();
        context.sp = arch.trapframe() as *const _ as _;
        context.x[30] = arch_vcpu_hyp_entry as _;
        info!("New AArch64 VCPU created: {:#x?}", arch);
        arch
    }

    fn trapframe(&self) -> &mut AArch64TrapFrame {
        unsafe {
            &mut *((self
                .stack
                .upper_bound()
                .sub(core::mem::size_of::<AArch64TrapFrame>()))
                as *mut AArch64TrapFrame)
        }
    }

    fn context(&self) -> &mut AArch64VCpuContext {
        unsafe { (*self.context.get()).as_mut() }
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
    {
        let vcpu = current_vcpu();
        info!("AArch64 VCPU {} is running", vcpu.id);
        let trapframe = vcpu.arch.trapframe();
        trapframe.elr = 0xa0400000;
        trapframe.x[0] = 0xa0000000;
        trapframe.spsr = 0x3c5;
    }
    AArch64VCpu::reset_vm_regs();
    vmreturn();
}

pub fn vmreturn() -> ! {
    let mut _trapframe_ptr;
    {
        let vcpu = current_vcpu();
        vcpu.activate_gpm();
        let trapframe = vcpu.arch.trapframe();
        _trapframe_ptr = &*trapframe as *const _;
    }
    unsafe {
        AArch64TrapFrame::__vm_entry(_trapframe_ptr as _);
    }
}

pub fn arch_switch_to_vcpu(vcpu: Arc<VCpu>) -> ! {
    let context = vcpu.arch.context();
    let _context_ptr = &*context as *const _;
    drop(vcpu);

    unsafe {
        AArch64VCpuContext::__switch_to(_context_ptr as _);
    }
}

pub fn arch_wakeup_vcpu(vcpu: Arc<VCpu>) -> isize {
    if vcpu.arch.psci_on.load(core::sync::atomic::Ordering::SeqCst) {
        error!("PSCI: vcpu {} is already on", vcpu.id);
        return -4;
    }

    add_vcpu_to_scheduler(vcpu.clone());
    vcpu.arch
        .psci_on
        .store(true, core::sync::atomic::Ordering::SeqCst);

    0
}

unsafe impl Sync for AArch64VCpu {}

pub type ArchVCpu = AArch64VCpu;
