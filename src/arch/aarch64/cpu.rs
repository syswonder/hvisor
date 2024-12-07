use crate::{
    arch::{mm::new_s2_memory_set, sysreg::write_sysreg}, consts::{PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE}, device::irqchip::gicv3::{gicr::{GICR_ICACTIVER, GICR_ICENABLER, GICR_ISENABLER, GICR_SGI_BASE}, host_gicr_base, MAINTENACE_INTERRUPT}, memory::{
        addr::PHYS_VIRT_OFFSET, mm::PARKING_MEMORY_SET, GuestPhysAddr, HostPhysAddr, MemFlags,
        MemoryRegion, VirtAddr, PARKING_INST_PAGE,
    }, percpu::this_cpu_data
};
use aarch64_cpu::registers::{
    Readable, Writeable, ELR_EL2, HCR_EL2, MPIDR_EL1, SCTLR_EL1, SPSR_EL2, VTCR_EL2,
};

use super::{
    mm::{get_parange, get_parange_bits, is_s2_pt_level3},
    trap::vmreturn,
};

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
pub struct GeneralRegisters {
    pub exit_reason: u64,
    pub usr: [u64; 31],
}

impl GeneralRegisters {
    pub fn clear(&mut self) {
        self.exit_reason = 0;
        self.usr.fill(0);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub cpuid: usize,
    pub power_on: bool,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        Self {
            cpuid,
            power_on: false,
        }
    }

    pub fn reset(&mut self, entry: usize, dtb: usize) {
        ELR_EL2.set(entry as _);
        SPSR_EL2.set(0x3c5);
        let regs = self.guest_reg();
        regs.clear();
        regs.usr[0] = dtb as _; // dtb addr
        self.reset_vm_regs();
        self.activate_vmm();
        // unsafe {
        //     let base = host_gicr_base(this_cpu_id()) + GICR_SGI_BASE;
        //     let gicr_isenabler0 = (base + GICR_ISENABLER) as *mut u32;
        //     gicr_isenabler0.write_volatile(0xffff | 1 << MAINTENACE_INTERRUPT);
        //     let gicr_icenabler0 = (base + GICR_ICENABLER) as *mut u32;
        //     gicr_icenabler0.write_volatile(0xffff0000 & !(1 << MAINTENACE_INTERRUPT));
        // }
    }

    fn activate_vmm(&self) {
        VTCR_EL2.write(
            VTCR_EL2::TG0::Granule4KB
                + VTCR_EL2::PS.val(get_parange() as _)
                + VTCR_EL2::SH0::Inner
                + VTCR_EL2::HA::Enabled
                + VTCR_EL2::SL0.val(if is_s2_pt_level3() { 1 } else { 2 })
                + VTCR_EL2::ORGN0::NormalWBRAWA
                + VTCR_EL2::IRGN0::NormalWBRAWA
                + VTCR_EL2::T0SZ.val(
                    64 - if is_s2_pt_level3() {
                        39
                    } else {
                        get_parange_bits() as _
                    },
                ),
        );
        HCR_EL2.write(
            HCR_EL2::RW::EL1IsAarch64
                + HCR_EL2::TSC::EnableTrapEl1SmcToEl2
                + HCR_EL2::VM::SET
                + HCR_EL2::IMO::SET
                + HCR_EL2::FMO::SET,
        );
    }

    fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.cpuid + 1) as usize * PER_CPU_SIZE
    }

    fn guest_reg(&self) -> &mut GeneralRegisters {
        unsafe { &mut *((self.stack_top() - 32 * 8) as *mut GeneralRegisters) }
    }

    fn reset_vm_regs(&self) {
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

    pub fn run(&mut self) -> ! {
        assert!(this_cpu_id() == self.cpuid);
        this_cpu_data().activate_gpm();
        self.reset(this_cpu_data().cpu_on_entry, this_cpu_data().dtb_ipa);
        self.power_on = true;
        info!("cpu {} started", self.cpuid);
        unsafe {
            vmreturn(self.guest_reg() as *mut _ as usize);
        }
    }

    pub fn idle(&mut self) -> ! {
        assert!(this_cpu_id() == self.cpuid);
        let cpu_data = this_cpu_data();
        let _lock = cpu_data.ctrl_lock.lock();
        self.power_on = false;
        drop(_lock);

        info!("cpu {} idle", self.cpuid);
        // reset current cpu -> pc = 0x0 (wfi)
        PARKING_MEMORY_SET.call_once(|| {
            let parking_code: [u8; 8] = [0x7f, 0x20, 0x03, 0xd5, 0xff, 0xff, 0xff, 0x17]; // 1: wfi; b 1b
            unsafe {
                PARKING_INST_PAGE[..8].copy_from_slice(&parking_code);
            }

            let mut gpm = new_s2_memory_set();
            gpm.insert(MemoryRegion::new_with_offset_mapper(
                0 as GuestPhysAddr,
                unsafe { &PARKING_INST_PAGE as *const _ as HostPhysAddr - PHYS_VIRT_OFFSET },
                PAGE_SIZE,
                MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
            ))
            .unwrap();
            gpm
        });
        self.reset(0, this_cpu_data().dtb_ipa);
        unsafe {
            PARKING_MEMORY_SET.get().unwrap().activate();
            info!("cpu {} started from parking", self.cpuid);
            vmreturn(self.guest_reg() as *mut _ as usize);
        }
    }
}

pub fn mpidr_to_cpuid(mpidr: u64) -> u64 {
    mpidr & 0xff00ffffff
}

pub fn this_cpu_id() -> usize {
    mpidr_to_cpuid(MPIDR_EL1.get()) as _
}

pub unsafe fn enable_mmu() {
    const MAIR_FLAG: usize = 0x004404ff; //10001000000010011111111
    const SCTLR_FLAG: usize = 0x30c51835; //110000110001010001100000110101
    const TCR_FLAG: usize = 0x80853510; //10000000100001010011010100010000

    core::arch::asm!(
        "
        /* setup the MMU for EL2 hypervisor mappings */
        ldr	x1, ={MAIR_FLAG}     
        msr	mair_el2, x1       // memory attributes for pagetable
        ldr	x1, ={TCR_FLAG}
	    msr	tcr_el2, x1        // translate control, virt range = [0, 2^48)

	    /* Enable MMU, allow cacheability for instructions and data */
	    ldr	x1, ={SCTLR_FLAG}
	    msr	sctlr_el2, x1      // system control register

	    isb
	    tlbi alle2
	    dsb	nsh
    ",
        MAIR_FLAG = const MAIR_FLAG,
        TCR_FLAG = const TCR_FLAG,
        SCTLR_FLAG = const SCTLR_FLAG,
    );
}
