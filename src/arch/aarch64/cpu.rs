use crate::{
    arch::sysreg::write_sysreg,
    consts::{INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::VirtAddr,
    percpu::this_zone,
};
use aarch64_cpu::registers::{Readable, Writeable, MPIDR_EL1, SCTLR_EL1};

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    psci::cpu_on(cpuid as u64 | 0x80000000, start_addr as _, opaque as _).unwrap_or_else(|err| {
        if let psci::error::Error::AlreadyOn = err {
        } else {
            panic!("can't wake up cpu {}", cpuid);
        }
    });
}

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub exit_reason: usize,
    pub x: [usize; 31], //x0~x31
    pub esr: usize,
    pub elr: usize,
    pub spsr: usize,
    pub cpuid: usize,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        ArchCpu {
            exit_reason: 0,
            x: [0; 31],
            esr: 0,
            spsr: 0,
            elr: INVALID_ADDRESS,
            cpuid,
        }
    }
    pub fn get_cpuid(&self) -> usize {
        self.cpuid
    }
    pub fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.get_cpuid() + 1) as usize * PER_CPU_SIZE - 8
    }
    pub fn init(&mut self, entry: usize, cpu_id: usize, dtb: usize) {
        //self.sepc = guest_test as usize as u64;
        write_sysreg!(tpidr_el2, self as *const _ as u64); // arch cpu pointer
        self.elr = entry;
        self.esr = 0;
        self.spsr = 0x5 | 1 << 6 | 1 << 7 | 1 << 8 | 1 << 9; // SPSR_EL1h | SPSR_D | SPSR_A | SPSR_I | SPSR_F
        self.x[0] = cpu_id; // cpu id
        self.x[1] = dtb; // dtb addr
        self.reset();
    }

    fn reset(&self) {
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

        this_zone().read().gpm_activate();

        SCTLR_EL1.set((1 << 11) | (1 << 20) | (3 << 22) | (3 << 28));
    }

    pub fn run(&mut self) {
        todo!("run");
        // extern "C" {
        //     fn vcpu_arch_entry();
        // }
        // unsafe {
        //     vcpu_arch_entry();
        // }
    }

    pub fn idle(&self) {
        unsafe {
            core::arch::asm!("wfi");
        }
        println!("CPU{} weakup!", self.cpuid);
    }
}

pub fn mpidr_to_cpuid(mpidr: u64) -> u64 {
    mpidr & 0xff00ffffff
}

pub fn this_cpu_id() -> usize {
    mpidr_to_cpuid(MPIDR_EL1.get()) as _
}
