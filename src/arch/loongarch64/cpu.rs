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
//      Ming Shen  <boneinscri@163.com>
use super::eiointc::LoongArch64Eiointc;
use super::ipi::*;
use super::register::*;
use super::zone::ZoneContext;
use crate::arch::trap::enable_global_interrupt;
use crate::arch::zone::disable_hwi_through;
use crate::cpu_data::this_zone;
use crate::cpu_data::{get_vcpuid_from_pcpuid, this_cpu_data, VcpuState};
use crate::device::common::MMIODerefWrapper;
use crate::zone::{find_zone, this_zone_id};
use core::arch::asm;
use core::fmt::{self, Debug, Formatter};
use loongArch64::register::crmd::Crmd;
use loongArch64::register::pgdl;
use loongArch64::register::{cpuid, crmd};
use spin::Mutex;
use tock_registers::interfaces::Writeable;

use crate::{
    consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::VirtAddr,
    platform::CPU_BOOT_CONTEXT_ADDRESS,
};

// CPU_BOOT_CONTEXT_ADDRESS is defined in board.rs (platform-specific)

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootContext {
    pub ra: usize, // return address
    pub sp: usize, // stack pointer
    pub tp: usize, // threads pointer
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub fp: usize,
    pub start_image: usize,
    pub image_handle: usize,
    pub efi_system_table: usize,
    pub cmd_line_ptr: usize,
    // pub t0: usize,
    // pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub crmd: usize,
    pub prmd: usize,
    pub euen: usize,
    pub misc: usize,
    pub ecfg: usize,
    pub estat: usize,
    pub era: usize,
    pub badv: usize,
    pub badi: usize,
    pub eentry: usize,
    pub tlbidx: usize,
    pub tlbehi: usize,
    pub tlbelo0: usize,
    pub tlbelo1: usize,
    pub asid: usize,
    pub pgdl: usize,
    pub pgdh: usize,
    pub pwcl: usize,
    pub pwch: usize,
    pub stlbps: usize,
    pub rvacfg: usize,
    pub cpuid: usize,
    pub prcfg1: usize,
    pub prcfg2: usize,
    pub prcfg3: usize,
    pub save0: usize,
    pub save1: usize,
    pub save2: usize,
    pub save3: usize,
    pub save4: usize,
    pub save5: usize,
    pub save6: usize,
    pub save7: usize,
    pub tid: usize,
    pub tcfg: usize,
    pub tval: usize,
    pub cntc: usize,
    pub ticlr: usize,
    pub tlbrentry: usize,
    pub tlbrbadv: usize,
    pub tlbrera: usize,
    pub tlbrsave: usize,
    pub tlbrelo0: usize,
    pub tlbrelo1: usize,
    pub tlbrehi: usize,
    pub tlbrprmd: usize,
    pub dmw0: usize,
    pub dmw1: usize,
    pub dmw2: usize,
    pub dmw3: usize,

    pub second_crmd: usize,
    pub second_prmd: usize,
    pub second_euen: usize,
    pub second_misc: usize,
    pub second_ecfg: usize,
    pub second_estat: usize,
    pub second_era: usize,
    pub second_badv: usize,
    pub second_badi: usize,
    pub second_eentry: usize,
    pub second_tlbidx: usize,
    pub second_tlbehi: usize,
    pub second_tlbelo0: usize,
    pub second_tlbelo1: usize,
    pub second_asid: usize,
    pub second_pgdl: usize,
    pub second_pgdh: usize,
    pub second_pwcl: usize,
    pub second_pwch: usize,
    pub second_stlbps: usize,
    pub second_rvacfg: usize,
    pub second_cpuid: usize,
    pub second_prcfg1: usize,
    pub second_prcfg2: usize,
    pub second_prcfg3: usize,
    pub second_save0: usize,
    pub second_save1: usize,
    pub second_save2: usize,
    pub second_save3: usize,
    pub second_save4: usize,
    pub second_save5: usize,
    pub second_save6: usize,
    pub second_save7: usize,
    pub second_tid: usize,
    pub second_tcfg: usize,
    pub second_tval: usize,
    pub second_cntc: usize,
    pub second_ticlr: usize,
    pub second_tlbrentry: usize,
    pub second_tlbrbadv: usize,
    pub second_tlbrera: usize,
    pub second_tlbrsave: usize,
    pub second_tlbrelo0: usize,
    pub second_tlbrelo1: usize,
    pub second_tlbrehi: usize,
    pub second_tlbrprmd: usize,
    pub second_dmw0: usize,
    pub second_dmw1: usize,
    pub second_dmw2: usize,
    pub second_dmw3: usize,
}

/// Flattened GCSR values extracted from BootContext (primary or secondary).
struct GcsrSnapshot {
    crmd: usize,
    prmd: usize,
    euen: usize,
    misc: usize,
    ecfg: usize,
    estat: usize,
    era: usize,
    badv: usize,
    badi: usize,
    eentry: usize,
    tlbidx: usize,
    tlbehi: usize,
    tlbelo0: usize,
    tlbelo1: usize,
    asid: usize,
    pgdl: usize,
    pgdh: usize,
    pwcl: usize,
    pwch: usize,
    stlbps: usize,
    rvacfg: usize,
    cpuid: usize,
    prcfg1: usize,
    prcfg2: usize,
    prcfg3: usize,
    save0: usize,
    save1: usize,
    save2: usize,
    save3: usize,
    save4: usize,
    save5: usize,
    save6: usize,
    save7: usize,
    tid: usize,
    tcfg: usize,
    tval: usize,
    cntc: usize,
    ticlr: usize,
    tlbrentry: usize,
    tlbrbadv: usize,
    tlbrera: usize,
    tlbrsave: usize,
    tlbrelo0: usize,
    tlbrelo1: usize,
    tlbrehi: usize,
    tlbrprmd: usize,
    dmw0: usize,
    dmw1: usize,
    dmw2: usize,
    dmw3: usize,
}

impl GcsrSnapshot {
    fn from_primary(b: &BootContext, vcpu_id: usize) -> Self {
        Self {
            crmd: b.crmd,
            prmd: b.prmd,
            euen: b.euen,
            misc: b.misc,
            ecfg: b.ecfg,
            estat: b.estat,
            era: b.era,
            badv: b.badv,
            badi: b.badi,
            eentry: b.eentry,
            tlbidx: b.tlbidx,
            tlbehi: b.tlbehi,
            tlbelo0: b.tlbelo0,
            tlbelo1: b.tlbelo1,
            asid: b.asid,
            pgdl: b.pgdl,
            pgdh: b.pgdh,
            pwcl: b.pwcl,
            pwch: b.pwch,
            stlbps: b.stlbps,
            rvacfg: b.rvacfg,
            cpuid: vcpu_id,
            prcfg1: b.prcfg1,
            prcfg2: b.prcfg2,
            prcfg3: b.prcfg3,
            save0: b.save0,
            save1: b.save1,
            save2: b.save2,
            save3: b.save3,
            save4: b.save4,
            save5: b.save5,
            save6: b.save6,
            save7: b.save7,
            tid: vcpu_id,
            tcfg: b.tcfg,
            tval: b.tval,
            cntc: b.cntc,
            ticlr: b.ticlr,
            tlbrentry: b.tlbrentry,
            tlbrbadv: b.tlbrbadv,
            tlbrera: b.tlbrera,
            tlbrsave: b.tlbrsave,
            tlbrelo0: b.tlbrelo0,
            tlbrelo1: b.tlbrelo1,
            tlbrehi: b.tlbrehi,
            tlbrprmd: b.tlbrprmd,
            dmw0: b.dmw0,
            dmw1: b.dmw1,
            dmw2: b.dmw2,
            dmw3: b.dmw3,
        }
    }

    fn from_secondary(b: &BootContext, vcpu_id: usize) -> Self {
        Self {
            crmd: b.second_crmd,
            prmd: b.second_prmd,
            euen: b.second_euen,
            misc: b.second_misc,
            ecfg: b.second_ecfg,
            estat: b.second_estat,
            era: b.second_era,
            badv: b.second_badv,
            badi: b.second_badi,
            eentry: b.second_eentry,
            tlbidx: b.second_tlbidx,
            tlbehi: b.second_tlbehi,
            tlbelo0: b.second_tlbelo0,
            tlbelo1: b.second_tlbelo1,
            asid: b.second_asid,
            pgdl: b.second_pgdl,
            pgdh: b.second_pgdh,
            pwcl: b.second_pwcl,
            pwch: b.second_pwch,
            stlbps: b.second_stlbps,
            rvacfg: b.second_rvacfg,
            cpuid: vcpu_id,
            prcfg1: b.second_prcfg1,
            prcfg2: b.second_prcfg2,
            prcfg3: b.second_prcfg3,
            save0: b.second_save0,
            save1: b.second_save1,
            save2: b.second_save2,
            save3: b.second_save3,
            save4: b.second_save4,
            save5: b.second_save5,
            save6: b.second_save6,
            save7: b.second_save7,
            tid: vcpu_id,
            tcfg: b.second_tcfg,
            tval: b.second_tval,
            cntc: b.second_cntc,
            ticlr: b.second_ticlr,
            tlbrentry: b.second_tlbrentry,
            tlbrbadv: b.second_tlbrbadv,
            tlbrera: b.second_tlbrera,
            tlbrsave: b.second_tlbrsave,
            tlbrelo0: b.second_tlbrelo0,
            tlbrelo1: b.second_tlbrelo1,
            tlbrehi: b.second_tlbrehi,
            tlbrprmd: b.second_tlbrprmd,
            dmw0: b.second_dmw0,
            dmw1: b.second_dmw1,
            dmw2: b.second_dmw2,
            dmw3: b.second_dmw3,
        }
    }

    fn write_all(&self) {
        write_gcsr_crmd(self.crmd);
        write_gcsr_prmd(self.prmd);
        write_gcsr_pgdh(self.euen);
        write_gcsr_pgdl(self.misc);
        write_gcsr_tval(self.ecfg);
        write_gcsr_estat(self.estat);
        write_gcsr_era(self.era);
        write_gcsr_badv(self.badv);
        write_gcsr_badi(self.badi);
        write_gcsr_eentry(self.eentry);
        write_gcsr_tlbidx(self.tlbidx);
        write_gcsr_tlbehi(self.tlbehi);
        write_gcsr_tlbelo0(self.tlbelo0);
        write_gcsr_tlbelo1(self.tlbelo1);
        write_gcsr_asid(self.asid);
        write_gcsr_pgdl(self.pgdl);
        write_gcsr_pgdh(self.pgdh);
        write_gcsr_pwcl(self.pwcl);
        write_gcsr_pwch(self.pwch);
        write_gcsr_stlbps(self.stlbps);
        write_gcsr_rvacfg(self.rvacfg);
        write_gcsr_cpuid(self.cpuid);
        write_gcsr_prcfg1(self.prcfg1);
        write_gcsr_prcfg2(self.prcfg2);
        write_gcsr_prcfg3(self.prcfg3);
        write_gcsr_save0(self.save0);
        write_gcsr_save1(self.save1);
        write_gcsr_save2(self.save2);
        write_gcsr_save3(self.save3);
        write_gcsr_save4(self.save4);
        write_gcsr_save5(self.save5);
        write_gcsr_save6(self.save6);
        write_gcsr_save7(self.save7);
        write_gcsr_tid(self.tid);
        write_gcsr_tcfg(self.tcfg);
        write_gcsr_tval(self.tval);
        write_gcsr_cntc(self.cntc);
        write_gcsr_ticlr(self.ticlr);
        write_gcsr_tlbrentry(self.tlbrentry);
        write_gcsr_tlbrbadv(self.tlbrbadv);
        write_gcsr_tlbrera(self.tlbrera);
        write_gcsr_tlbrsave(self.tlbrsave);
        write_gcsr_tlbrelo0(self.tlbrelo0);
        write_gcsr_tlbrelo1(self.tlbrelo1);
        write_gcsr_tlbrehi(self.tlbrehi);
        write_gcsr_tlbrprmd(self.tlbrprmd);
        write_gcsr_dmw0(self.dmw0);
        write_gcsr_dmw1(self.dmw1);
        write_gcsr_dmw2(self.dmw2);
        write_gcsr_dmw3(self.dmw3);
    }
}

pub type IpiState = LoongArch64IpiState;
pub type Eiointc = LoongArch64Eiointc;

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub ctx: ZoneContext,
    pub stack_top: usize,
    pub cpuid: usize,
    pub init: bool,

    // boneinscri 2026.04
    // ipi/eiointc/timer according to kvm
    pub irq_pending: usize,
    pub irq_clear: usize,
    pub ipi_state: Mutex<IpiState>,
    pub eiointc: Mutex<Eiointc>,
    pub expire: isize,
    pub csr: [usize; 0x513],
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        let mut ret = ArchCpu {
            ctx: super::trap::dump_reset_gcsrs(),
            stack_top: 0,
            cpuid,
            init: false,
            irq_pending: 0,
            irq_clear: 0,
            ipi_state: Mutex::new(LoongArch64IpiState::new()),
            eiointc: Mutex::new(LoongArch64Eiointc::new()),
            expire: 0,
            csr: [0; 0x513],
        };
        return ret;
    }

    // ================
    // boneinscri : 2026.04 (for smp)
    pub fn add_irq(&mut self, irq: usize) {
        let mask = 1 << irq;
        self.irq_pending = self.irq_pending | mask;
        self.irq_clear = self.irq_clear & (!mask);
    }
    pub fn remove_irq(&mut self, irq: usize) {
        let mask = 1 << irq;
        self.irq_pending = self.irq_pending & (!mask);
        self.irq_clear = self.irq_clear | mask;
    }
    // ================

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
    /// Common vcpu entry: write SAVE3/SAVE4, flush TLB, jump to guest.
    fn vcpu_enter(&mut self) -> ! {
        let ctx_addr = &mut self.ctx as *mut ZoneContext;
        debug!(
            "loongarch64: ArchCpu::vcpu_enter: percpu_s={:#x}",
            self.stack_top() - PER_CPU_SIZE
        );
        debug!(
            "loongarch64: ArchCpu::vcpu_enter: ctx_addr={:#x}, size={}",
            ctx_addr as usize,
            core::mem::size_of::<ZoneContext>()
        );
        debug!(
            "loongarch64: ArchCpu::vcpu_enter: stack_tp={:#x}",
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
            asm!("invtlb 0, $r0, $r0");
        }
        super::trap::_vcpu_return(ctx_addr as usize);
        panic!("loongarch64: ArchCpu::vcpu_enter: unreachable");
    }

    pub fn run(&mut self) -> ! {
        assert!(this_cpu_id() == self.get_cpuid());
        this_cpu_data().activate_gpm();
        this_cpu_data().vcpu_state.store(VcpuState::Running);
        self.power_on = true;

        let boot_ctx = unsafe { &mut *(CPU_BOOT_CONTEXT_ADDRESS as *mut BootContext) };
        info!("boot_ctx_addr={:#x}", CPU_BOOT_CONTEXT_ADDRESS);
        info!("loongarch64: ArchCpu::run: boot_ctx={:#x?}", boot_ctx);

        let vcpu_id = get_vcpuid_from_pcpuid(self.get_cpuid());
        info!("gcsr_cpu_id : {}", vcpu_id);

        for i in 0..32 {
            self.ctx.x[i] = 0;
        }

        let mut snap = GcsrSnapshot::from_primary(boot_ctx, vcpu_id);

        let zone_id = this_zone_id();
        if zone_id == 0 {
            this_cpu_data().cpu_on_entry = boot_ctx.start_image;
            info!(
                "boot_ctx.efi_system_table: {:#x?}",
                boot_ctx.efi_system_table
            );
            self.ctx.x[4] = boot_ctx.image_handle;
            self.ctx.x[5] = 0;
            self.ctx.x[6] = 0;
            info!(
                "a0={:#x?} a1={:#x?} a2={:#x?}",
                self.ctx.x[4], self.ctx.x[5], self.ctx.x[6]
            );
        } else {
            let is_acpi = {
                let zone = this_zone();
                zone.boot_method.starts_with(b"acpi")
            };
            if is_acpi {
                self.ctx.x[4] = 1;
                self.ctx.x[5] = boot_ctx.cmd_line_ptr;
                self.ctx.x[6] = boot_ctx.efi_system_table;
            } else {
                self.ctx.x[4] = 0;
                self.ctx.x[5] = 0;
                self.ctx.x[6] = 0;
            }
            info!("zone: {} boot_method: acpi={}, x[4](a0)={:#x}, x[5](a1/cmd_line_ptr)={:#x}, x[6](a2/efi_system_table)={:#x}",
                zone_id, is_acpi, self.ctx.x[4], self.ctx.x[5], self.ctx.x[6]);

            snap.dmw0 = read_csr_dmw0();
            snap.dmw1 = read_csr_dmw1();
            snap.dmw2 = read_csr_dmw2();
            snap.dmw3 = read_csr_dmw3();
            if this_cpu_data().cpu_on_entry & 0xffff_0000_0000_0000 == 0 {
                snap.dmw1 &= 0x0000_ffff_ffff_ffff; // for npucore
            }
        }

        if !self.init {
            self.init(this_cpu_data().cpu_on_entry, this_cpu_data().id, 0);
            self.init = true;
        }

        self.ctx.x[1] = boot_ctx.ra;
        self.ctx.x[2] = boot_ctx.tp;
        self.ctx.x[3] = boot_ctx.sp;
        self.ctx.x[22] = boot_ctx.fp;
        self.ctx.x[23] = boot_ctx.s0;
        self.ctx.x[24] = boot_ctx.s1;
        self.ctx.x[25] = boot_ctx.s2;
        self.ctx.x[26] = boot_ctx.s3;
        self.ctx.x[27] = boot_ctx.s4;
        self.ctx.x[28] = boot_ctx.s5;
        self.ctx.x[29] = boot_ctx.s6;
        self.ctx.x[30] = boot_ctx.s7;
        self.ctx.x[31] = boot_ctx.s8;
        self.ctx.x[14] = boot_ctx.t2;
        self.ctx.x[15] = boot_ctx.t3;
        self.ctx.x[16] = boot_ctx.t4;
        self.ctx.x[17] = boot_ctx.t5;
        self.ctx.x[18] = boot_ctx.t6;
        self.ctx.x[19] = boot_ctx.t7;
        self.ctx.x[20] = boot_ctx.t8;

        snap.write_all();
        self.vcpu_enter();
    }

    pub fn run_secondary(&mut self, smpboot_entry: usize) -> ! {
        assert!(this_cpu_id() == self.get_cpuid());
        this_cpu_data().activate_gpm();
        self.power_on = true;
        this_cpu_data().cpu_on_entry = smpboot_entry;

        if !self.init {
            self.init(this_cpu_data().cpu_on_entry, this_cpu_data().id, 0);
            self.init = true;
        }
        for i in 0..32 {
            self.ctx.x[i] = 0;
        }

        let vcpu_id = get_vcpuid_from_pcpuid(self.get_cpuid());
        let boot_ctx = unsafe { &mut *(CPU_BOOT_CONTEXT_ADDRESS as *mut BootContext) };
        info!("boot_ctx_addr={:#x}", CPU_BOOT_CONTEXT_ADDRESS);

        let snap = GcsrSnapshot::from_secondary(boot_ctx, vcpu_id);
        snap.write_all();
        self.vcpu_enter();
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
        enable_global_interrupt();
        self.power_on = false;
        loop {}
    }
}

pub fn this_cpu_id() -> usize {
    cpuid::read().core_id()
}

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    let start_addr = start_addr & 0x0000_ffff_ffff_ffff;
    let ipi: &MMIODerefWrapper<IpiRegisters> = match cpuid {
        1 => &CORE1_IPI,
        2 => &CORE2_IPI,
        3 => &CORE3_IPI,
        // boneinscri, for 3a6000 smp
        4 => &CORE4_IPI,
        5 => &CORE5_IPI,
        6 => &CORE6_IPI,
        7 => &CORE7_IPI,
        _ => {
            panic!("loongarch64: cpu_start: invalid cpuid={}", cpuid);
        }
    };
    ipi.ipi_enable.write(IpiEnable::IPIENABLE.val(0xffffffff));
    let entry_addr = start_addr;
    mail_send(entry_addr, cpuid, 0);
    ipi_write_action(cpuid, SMP_BOOT_CPU);
}

pub fn store_cpu_pointer_to_reg(pointer: usize) {
    // println!("loongarch64 doesn't support store cpu pointer to reg, pointer: {:#x}", pointer);
    return;
}

pub fn get_target_cpu(irq: usize, zone_id: usize) -> usize {
    find_zone(zone_id)
        .unwrap()
        .read()
        .cpu_set()
        .first_cpu()
        .unwrap()
}
