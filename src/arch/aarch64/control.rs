use aarch64_cpu::registers::SCTLR_EL1;
use aarch64_cpu::registers::Writeable;
use crate::percpu::this_cell;
use super::sysreg::write_sysreg;

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

pub fn set_hcr_flags() {
    HCR_EL2.write(
        HCR_EL2::RW::EL1IsAarch64
            + HCR_EL2::TSC::EnableTrapEl1SmcToEl2
            + HCR_EL2::VM::SET
            + HCR_EL2::IMO::SET
            + HCR_EL2::FMO::SET,
    );
}

fn reset_current_cpu(entry: u64) {
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

    this_cell().read().gpm_activate();

    SCTLR_EL1.set((1 << 11) | (1 << 20) | (3 << 22) | (3 << 28));
    //SCTLR_EL1.modify(SCTLR_EL1::M::Disable);
    //HCR_EL2.modify(HCR_EL2::VM::Disable);
    unsafe {
        //isb();
        set_vm_pc(entry);
    }
    //disable stage2
    //HCR_EL2.modify(HCR_EL2::VM::Disable);
}

pub unsafe extern "C" fn set_vm_pc(_entry: u64) {
    //info!("Hello World! from el1");
    //set el1 pc
    // x0:entry
    unsafe {
        core::arch::asm!(
            "
            mov	x1, #965
            msr	SPSR_EL2, x1
            tlbi alle1is
            tlbi alle2is
            msr	ELR_EL2, x0
        "
        );
    }
}

pub fn send_event(cpu_id: u64, sgi_num: u64) {
    // TODO: add more info
    let aff3: u64 = 0 << 48;
    let aff2: u64 = 0 << 32;
    let aff1: u64 = 0 << 16;
    let irm: u64 = 0 << 40;
    let sgi_id: u64 = sgi_num << 24;
    let target_list: u64 = 1 << cpu_id;
    let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
    write_sysreg!(icc_sgi1r_el1, val);
    info!("write sgi sys value = {:#x}", val);
}
