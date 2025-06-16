// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
use crate::{
    arch::{mm::new_s2_memory_set, sysreg::write_sysreg},
    consts::{PAGE_SIZE, PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::{
        addr::PHYS_VIRT_OFFSET, mm::PARKING_MEMORY_SET, GuestPhysAddr, HostPhysAddr, MemFlags,
        MemoryRegion, VirtAddr, PARKING_INST_PAGE,
    },
    percpu::this_cpu_data,
};
use aarch64_cpu::registers::{
    Readable, Writeable, ELR_EL2, HCR_EL2, MPIDR_EL1, SCTLR_EL1, SPSR_EL2, VTCR_EL2,
};
use core::ptr::addr_of;

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
        debug!(
            "cpu {} reset, entry: {:#x}, dtb: {:#x}",
            self.cpuid, entry, dtb
        );
        ELR_EL2.set(entry as _);
        SPSR_EL2.set(0x3c5);
        let regs = self.guest_reg();
        regs.clear();
        regs.usr[0] = dtb as _; // dtb addr
        self.reset_vm_regs();
        self.activate_vmm();
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
        info!(
            "cpu {} started at {:#x?}",
            self.cpuid,
            this_cpu_data().cpu_on_entry
        );
        unsafe {
            vmreturn(self.guest_reg() as *mut _ as usize);
        }
    }

    pub fn idle(&mut self) -> ! {
        debug!("cpu {} begin to be idle", self.cpuid);
        assert!(this_cpu_id() == self.cpuid);
        let cpu_data = this_cpu_data();
        let _lock = cpu_data.ctrl_lock.lock();
        self.power_on = false;
        drop(_lock);

        // reset current cpu -> pc = 0x0 (wfi)
        PARKING_MEMORY_SET.call_once(|| {
            let parking_code: [u8; 8] = [0x7f, 0x20, 0x03, 0xd5, 0xff, 0xff, 0xff, 0x17]; // 1: wfi; b 1b
            unsafe {
                PARKING_INST_PAGE[..8].copy_from_slice(&parking_code);
            }

            let mut gpm = new_s2_memory_set();
            gpm.insert(MemoryRegion::new_with_offset_mapper(
                0 as GuestPhysAddr,
                unsafe {
                    addr_of!(PARKING_INST_PAGE) as *const _ as HostPhysAddr - PHYS_VIRT_OFFSET
                },
                PAGE_SIZE,
                MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
            ))
            .unwrap();
            gpm
        });
        self.reset(0, this_cpu_data().dtb_ipa);
        unsafe {
            PARKING_MEMORY_SET.get().unwrap().activate();
            info!("cpu {} start parking", self.cpuid);
            vmreturn(self.guest_reg() as *mut _ as usize);
        }
    }
}

pub fn mpidr_to_cpuid(mpidr: u64) -> u64 {
    if cfg!(feature = "mpidr_rockchip") {
        (mpidr >> 8) & 0xff
    } else {
        mpidr & 0xff00ffffff
    }
}

pub fn this_cpu_id() -> usize {
    mpidr_to_cpuid(MPIDR_EL1.get()) as _
}

pub fn activate_vmm() {
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
