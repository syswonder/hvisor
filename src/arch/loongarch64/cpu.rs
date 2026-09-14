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
//      Yulong Han <wheatfox17@icloud.com>
//
use super::ipi::*;
use super::zone::ZoneContext;
use crate::arch::zone::disable_hwi_through;
use crate::cpu_data::{this_cpu_data, PerCpu, VcpuState};
use crate::zone::find_zone;
use core::arch::asm;
use core::fmt::{self, Debug, Formatter};
use loongArch64::register::crmd;
use loongArch64::register::crmd::Crmd;
use loongArch64::register::pgdl;

use crate::{
    consts::{MAX_CPU_NUM, PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::VirtAddr,
};

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub ctx: ZoneContext,
    pub stack_top: usize,
    pub cpuid: usize,
    pub init: bool,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        let mut ret = ArchCpu {
            ctx: super::trap::dump_reset_gcsrs(),
            stack_top: 0,
            cpuid,
            init: false,
        };
        return ret;
    }
    pub fn get_cpuid(&self) -> usize {
        self.cpuid
    }
    pub fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.get_cpuid() + 1) as usize * PER_CPU_SIZE
    }
    pub fn init(&mut self, entry: usize, cpu_id: usize, dtb: usize) {
        self.ctx.sepc = entry;
        self.stack_top = self.stack_top() as usize;
    }
    pub fn run(&mut self) -> ! {
        assert!(this_cpu_id() == self.get_cpuid());
        this_cpu_data().activate_gpm();
        this_cpu_data().vcpu_state.store(VcpuState::Running);
        if !self.init {
            self.init(this_cpu_data().cpu_on_entry, this_cpu_data().id, 0);
            self.init = true;
        }
        // set x[] to all 0
        for i in 0..32 {
            self.ctx.x[i] = 0;
        }
        let physical_cpu = this_cpu_data().id;
        self.ctx.gcsr_cpuid = this_cpu_data()
            .zone
            .as_ref()
            .and_then(|zone| zone.read().phys_to_guest_cpu(physical_cpu))
            .unwrap_or(0);
        info!(
            "[[CPU virtualization]] CPU{} run@{:#x}",
            self.get_cpuid(),
            self.ctx.sepc
        );
        debug!("loongarch64: @{:#x?}", self);
        // step 1: enable guest mode
        // step 2: set guest entry to era
        // step 3: run ertn and enter guest mode
        let ctx_addr = &mut self.ctx as *mut ZoneContext;
        debug!(
            "loongarch64: ArchCpu::run: percpu_s={:#x}",
            self.stack_top() - PER_CPU_SIZE
        );
        debug!(
            "loongarch64: ArchCpu::run: ctx_addr={:#x}, size={}",
            ctx_addr as usize,
            core::mem::size_of::<ZoneContext>()
        );
        debug!(
            "loongarch64: ArchCpu::run: stack_tp={:#x}",
            self.stack_top()
        );

        unsafe {
            asm!(
                "csrwr {}, {LOONGARCH_CSR_SAVE3}",
                "csrwr {}, {LOONGARCH_CSR_SAVE4}",
                in(reg) (ctx_addr as usize + core::mem::size_of::<ZoneContext>()),
                in(reg) self.stack_top(),
                LOONGARCH_CSR_SAVE3 = const 0x33,
                LOONGARCH_CSR_SAVE4 = const 0x34,
            );
        }

        unsafe {
            asm!("invtlb 0, $r0, $r0"); // flush TLBs
        }

        super::trap::_vcpu_return(ctx_addr as usize);
    }
    pub fn idle(&mut self) -> ! {
        let ctx_addr = &mut self.ctx as *mut ZoneContext;
        unsafe {
            asm!(
                "csrwr {}, {LOONGARCH_CSR_SAVE3}",
                "csrwr {}, {LOONGARCH_CSR_SAVE4}",
                in(reg) (ctx_addr as usize + core::mem::size_of::<ZoneContext>()),
                in(reg) self.stack_top(),
                LOONGARCH_CSR_SAVE3 = const 0x33,
                LOONGARCH_CSR_SAVE4 = const 0x34,
            );
        }
        info!("loongarch64: ArchCpu::idle: cpuid={}", self.get_cpuid());
        this_cpu_data().vcpu_state.store(VcpuState::Stopped);
        // enable ipi on ecfg
        ecfg_ipi_enable();
        // The trap vector is installed with interrupts disabled. Enable them only
        // after this CPU has valid trap context and stack pointers in SAVE3/SAVE4.
        super::trap::enable_global_interrupt();
        loop {}
    }
}

pub fn this_cpu_id() -> usize {
    // SAVE0 caches the PerCpu slot base (written once per core in
    // PerCpu::new); the id sits at slot offset 0. cpuid::read() (CSR 0x20)
    // stays available for code that wants the raw core id.
    unsafe { (*(this_cpu_pointer() as *const PerCpu)).id }
}

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    if cpuid >= MAX_CPU_NUM {
        error!("loongarch64: cpu_start: invalid cpuid={}", cpuid);
        return;
    }

    let _ = opaque;
    let start_addr = start_addr & 0x0000_ffff_ffff_ffff;
    mail_send_percore(start_addr, cpuid, 0);
    ipi_write_action_percore(cpuid, SMP_BOOT_CPU);
}

/// Free root CSR used to cache the PerCpu slot base of the current core.
///
/// CSR 0x21 (PRCFG1) is read-only and SAVE3/SAVE4 are the active trap
/// handoff (run/idle write the ctx/stack pointers), so the root SAVE0 is
/// the pragmatic free slot. Guest-state SAVE0 lives in the separate GCSR
/// file (trap.rs gcsrrd/gcsrwr) and is unaffected.
const CSR_SAVE0: usize = 0x30;

/// Cache the PerCpu slot base of the current core in root CSR SAVE0.
pub fn set_this_cpu_pointer(slot_base: usize) {
    unsafe {
        asm!(
            "csrwr {}, {LOONGARCH_CSR_SAVE0}",
            in(reg) slot_base,
            LOONGARCH_CSR_SAVE0 = const CSR_SAVE0,
        );
    }
}

/// PerCpu slot base of the current core, cached in SAVE0 by
/// `set_this_cpu_pointer` at `PerCpu::new` time.
pub fn this_cpu_pointer() -> usize {
    let ptr: usize;
    unsafe {
        asm!(
            "csrrd {}, {LOONGARCH_CSR_SAVE0}",
            out(reg) ptr,
            LOONGARCH_CSR_SAVE0 = const CSR_SAVE0,
        );
    }
    ptr
}

pub fn get_target_cpu(irq: usize, zone_id: usize) -> usize {
    find_zone(zone_id)
        .unwrap()
        .read()
        .cpu_set()
        .first_cpu()
        .unwrap()
}
