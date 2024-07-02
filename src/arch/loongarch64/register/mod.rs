// File:          mod.rs
// Description:   this is the register file of loongarch64's LVZ extension
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-20

#![allow(unused)]

use bit_field::BitField;
use log::*;
use loongArch64::register::{tcfg, tval};

#[macro_use]
mod macros;

// LVZ registers
pub mod gcfg;
pub mod gcntc;
pub mod gintc;
pub mod gstat;
pub mod gtlbc;
pub mod trgp;

// ras
pub mod merrctl;
pub mod merrentry;
pub mod merrera;
pub mod merrsave;

// offset of all GCSR available registers
pub const GCSR_CRMD: usize = 0x0;
pub const GCSR_PRMD: usize = 0x1;
pub const GCSR_EUEN: usize = 0x2;
pub const GCSR_MISC: usize = 0x3;
pub const GCSR_ECTL: usize = 0x4;
pub const GCSR_ESTAT: usize = 0x5;
pub const GCSR_ERA: usize = 0x6;
pub const GCSR_BADV: usize = 0x7;
pub const GCSR_BADI: usize = 0x8;
pub const GCSR_EENTRY: usize = 0xc;
pub const GCSR_TLBIDX: usize = 0x10;
pub const GCSR_TLBEHI: usize = 0x11;
pub const GCSR_TLBELO0: usize = 0x12;
pub const GCSR_TLBELO1: usize = 0x13;
pub const GCSR_ASID: usize = 0x18;
pub const GCSR_PGDL: usize = 0x19;
pub const GCSR_PGDH: usize = 0x1a;
pub const GCSR_PGD: usize = 0x1b;
pub const GCSR_PWCL: usize = 0x1c;
pub const GCSR_PWCH: usize = 0x1d;
pub const GCSR_STLBPS: usize = 0x1e;
pub const GCSR_RAVCFG: usize = 0x1f;
pub const GCSR_CPUID: usize = 0x20;
pub const GCSR_PRCFG1: usize = 0x21;
pub const GCSR_PRCFG2: usize = 0x22;
pub const GCSR_PRCFG3: usize = 0x23;
pub const GCSR_SAVE0: usize = 0x30;
pub const GCSR_SAVE1: usize = 0x31;
pub const GCSR_SAVE2: usize = 0x32;
pub const GCSR_SAVE3: usize = 0x33;
pub const GCSR_SAVE4: usize = 0x34;
pub const GCSR_SAVE5: usize = 0x35;
pub const GCSR_SAVE6: usize = 0x36;
pub const GCSR_SAVE7: usize = 0x37;
pub const GCSR_SAVE8: usize = 0x38;
pub const GCSR_SAVE9: usize = 0x39;
pub const GCSR_SAVE10: usize = 0x3a;
pub const GCSR_SAVE11: usize = 0x3b;
pub const GCSR_SAVE12: usize = 0x3c;
pub const GCSR_SAVE13: usize = 0x3d;
pub const GCSR_SAVE14: usize = 0x3e;
pub const GCSR_SAVE15: usize = 0x3f;
pub const GCSR_TID: usize = 0x40;
pub const GCSR_TCFG: usize = 0x41;
pub const GCSR_TVAL: usize = 0x42;
pub const GCSR_CNTC: usize = 0x43;
pub const GCSR_TICLR: usize = 0x44;
pub const GCSR_LLBCTL: usize = 0x60;
pub const GCSR_TLBRENTRY: usize = 0x88;
pub const GCSR_TLBRBADV: usize = 0x89;
pub const GCSR_TLBRERA: usize = 0x8a;
pub const GCSR_TLBRSAVE: usize = 0x8b;
pub const GCSR_TLBRELO0: usize = 0x8c;
pub const GCSR_TLBRELO1: usize = 0x8d;
pub const GCSR_TLBREHI: usize = 0x8e;
pub const GCSR_TLBRPRMD: usize = 0x8f;
pub const GCSR_DMW0: usize = 0x180;
pub const GCSR_DMW1: usize = 0x181;
pub const GCSR_DMW2: usize = 0x182;
pub const GCSR_DMW3: usize = 0x183;
// and some more, which are performance monitoring related

// READ GCSR
pub fn read_gcsr_crmd() -> usize {
    read_gcsr_loong!(0x0)
}
pub fn read_gcsr_prmd() -> usize {
    read_gcsr_loong!(0x1)
}
pub fn read_gcsr_euen() -> usize {
    read_gcsr_loong!(0x2)
}
pub fn read_gcsr_misc() -> usize {
    read_gcsr_loong!(0x3)
}
pub fn read_gcsr_ectl() -> usize {
    read_gcsr_loong!(0x4)
}
pub fn read_gcsr_estat() -> usize {
    read_gcsr_loong!(0x5)
}
pub fn read_gcsr_era() -> usize {
    read_gcsr_loong!(0x6)
}
pub fn read_gcsr_badv() -> usize {
    read_gcsr_loong!(0x7)
}
pub fn read_gcsr_badi() -> usize {
    read_gcsr_loong!(0x8)
}
pub fn read_gcsr_eentry() -> usize {
    read_gcsr_loong!(0xc)
}
pub fn read_gcsr_tcfg() -> usize {
    read_gcsr_loong!(0x41)
}
pub fn read_gcsr_tval() -> usize {
    read_gcsr_loong!(0x42)
}

// WRITE GCSR
pub fn write_gcsr_ectl(range: core::ops::RangeInclusive<usize>, value: usize) {
    set_gcsr_loong_bits!(0x4, range, value);
}

pub fn write_gcsr_eentry(range: core::ops::RangeInclusive<usize>, value: usize) {
    set_gcsr_loong_bits!(0xc, range, value);
}

use core::arch::asm;

// pub fn current_sp() -> usize{
//   let mut sp: usize;
//   unsafe {
//     asm!("addi.d {}, $sp, 0", out(reg) sp);
//   }
//   sp
// }

pub fn gcsr_dump() {
    unsafe {
        info!("==============CORE DUMP=============");
        let mut gcsr_r: usize;
        asm!("gcsrrd {}, 0x0", out(reg) gcsr_r);
        info!("gcsr_crmd : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1", out(reg) gcsr_r);
        info!("gcsr_prmd : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x2", out(reg) gcsr_r);
        info!("gcsr_euen : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3", out(reg) gcsr_r);
        info!("gcsr_misc : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x4", out(reg) gcsr_r);
        info!("gcsr_ectl : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x5", out(reg) gcsr_r);
        info!("gcsr_estat : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x6", out(reg) gcsr_r);
        info!("gcsr_era : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x7", out(reg) gcsr_r);
        info!("gcsr_badv : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8", out(reg) gcsr_r);
        info!("gcsr_badi : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0xc", out(reg) gcsr_r);
        info!("gcsr_eentry : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x10", out(reg) gcsr_r);
        info!("gcsr_tlbidx : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x11", out(reg) gcsr_r);
        info!("gcsr_tlbehi : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x12", out(reg) gcsr_r);
        info!("gcsr_tlbelo0 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x13", out(reg) gcsr_r);
        info!("gcsr_tlbelo1 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x18", out(reg) gcsr_r);
        info!("gcsr_asid : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x19", out(reg) gcsr_r);
        info!("gcsr_pgdl : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1a", out(reg) gcsr_r);
        info!("gcsr_pgdh : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1b", out(reg) gcsr_r);
        info!("gcsr_pgd : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1c", out(reg) gcsr_r);
        info!("gcsr_pwcl : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1d", out(reg) gcsr_r);
        info!("gcsr_pwch : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1e", out(reg) gcsr_r);
        info!("gcsr_stlbps : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x1f", out(reg) gcsr_r);
        info!("gcsr_ravcfg : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x20", out(reg) gcsr_r);
        info!("gcsr_cpuid : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x21", out(reg) gcsr_r);
        info!("gcsr_prcfg1 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x22", out(reg) gcsr_r);
        info!("gcsr_prcfg2 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x23", out(reg) gcsr_r);
        info!("gcsr_prcfg3 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x30", out(reg) gcsr_r);
        info!("gcsr_save0 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x31", out(reg) gcsr_r);
        info!("gcsr_save1 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x32", out(reg) gcsr_r);
        info!("gcsr_save2 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x33", out(reg) gcsr_r);
        info!("gcsr_save3 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x34", out(reg) gcsr_r);
        info!("gcsr_save4 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x35", out(reg) gcsr_r);
        info!("gcsr_save5 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x36", out(reg) gcsr_r);
        info!("gcsr_save6 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x37", out(reg) gcsr_r);
        info!("gcsr_save7 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x38", out(reg) gcsr_r);
        info!("gcsr_save8 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x39", out(reg) gcsr_r);
        info!("gcsr_save9 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3a", out(reg) gcsr_r);
        info!("gcsr_save10 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3b", out(reg) gcsr_r);
        info!("gcsr_save11 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3c", out(reg) gcsr_r);
        info!("gcsr_save12 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3d", out(reg) gcsr_r);
        info!("gcsr_save13 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3e", out(reg) gcsr_r);
        info!("gcsr_save14 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x3f", out(reg) gcsr_r);
        info!("gcsr_save15 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x40", out(reg) gcsr_r);
        info!("gcsr_tid : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x41", out(reg) gcsr_r);
        info!("gcsr_tcfg : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x42", out(reg) gcsr_r);
        info!("gcsr_tval : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x43", out(reg) gcsr_r);
        info!("gcsr_cntc : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x44", out(reg) gcsr_r);
        info!("gcsr_ticlr : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x60", out(reg) gcsr_r);
        info!("gcsr_llbctl : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x88", out(reg) gcsr_r);
        info!("gcsr_tlbreentry : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x89", out(reg) gcsr_r);
        info!("gcsr_tlbrbadv : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8a", out(reg) gcsr_r);
        info!("gcsr_tlbrera : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8b", out(reg) gcsr_r);
        info!("gcsr_tlbrsave : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8c", out(reg) gcsr_r);
        info!("gcsr_tlbrelo0 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8d", out(reg) gcsr_r);
        info!("gcsr_tlbrelo1 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8e", out(reg) gcsr_r);
        info!("gcsr_tlbrehi : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x8f", out(reg) gcsr_r);
        info!("gcsr_tlbrprmd : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x180", out(reg) gcsr_r);
        info!("gcsr_dmw0 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x181", out(reg) gcsr_r);
        info!("gcsr_dmw1 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x182", out(reg) gcsr_r);
        info!("gcsr_dmw2 : 0x{:#x}", gcsr_r);
        asm!("gcsrrd {}, 0x183", out(reg) gcsr_r);
        info!("gcsr_dmw3 : 0x{:#x}", gcsr_r);
    }
}

#[inline]
pub fn core_dump(sp: usize) {
    // #[cfg(target_arch = "dump_gprs")]
    {
        extern "C" {
            fn save_cpu_regs_state();
        }

        const REGISTER_ALIASES: [&str; 32] = [
            "zero", "ra", "tp", "sp", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "t0", "t1",
            "t2", "t3", "t4", "t5", "t6", "t7", "t8", "gp", "fp", "s0", "s1", "s2", "s3", "s4",
            "s5", "s6", "s7", "s8",
        ];

        unsafe {
            save_cpu_regs_state();
        }

        // Placeholder for a logging macro or function
        macro_rules! _info {
      ($($arg:tt)*) => (println!($($arg)*));
    }

        unsafe {
            // GPR
            let mut gpr = [0; 32];
            let save0_ = read_csr_loong!(0x30); // Placeholder: get the addr in CSR.SAVE0
            let mut ptr = save0_ as *mut usize;

            for i in 0..32 {
                gpr[i] = *ptr; // return a usize value
                ptr = ptr.add(1);
            }
            info!("performing full core dump:");

            println!("[GPR]");
            // Width of usize is 8 bytes, print leading zeros
            for i in 0..32 {
                if i == 3 {
                    println!("  {} = {:#016x}", REGISTER_ALIASES[i], sp);
                } else {
                    println!("  {} = {:#016x}", REGISTER_ALIASES[i], gpr[i]);
                }
            }
        }
    }
    // CSR
    let cpuid_ = loongArch64::register::cpuid::read();
    let eentry_ = loongArch64::register::eentry::read();
    let estat_ = loongArch64::register::estat::read();
    let badi_ = loongArch64::register::badi::read();
    let badv_ = loongArch64::register::badv::read();
    let crmd_ = loongArch64::register::crmd::read();
    let prmd_ = loongArch64::register::prmd::read();
    let era_ = loongArch64::register::era::read();
    let tcfg_ = loongArch64::register::tcfg::read();
    let tval_ = loongArch64::register::tval::read();
    let ecfg_ = loongArch64::register::ecfg::read();
    let mut desave_r: usize;
    unsafe {
        asm!("csrrd {}, 0x502", out(reg) desave_r);
    }

    println!("[CSR]");
    print!(
        "  csr.cpuid={:#x}, csr.eentry={:#x}\n  \
      csr.estat.is={:#x}, csr.estat.ecode={:#x}, csr.estat.esubcode={:#x}\n  \
      csr.badi={:#x}, csr.badv={:#x}\n  \
      csr.crmd={:?}\n  \
      csr.prmd.pplv={:#x}, csr.prmd.pie={}, csr.prmd.pwe={}\n  \
      csr.era.pc={:#x}, csr.tcfg.en={} csr.tcfg.periodic={} csr.tcfg.init_val={:#x}\n  \
      csr.tval.time_val={:#x}, csr.ecfg.lie={:#x}\n \
      csr.desave = {:#x}\n",
        cpuid_.core_id(),
        eentry_.eentry(),
        estat_.is(),
        estat_.ecode(),
        estat_.esubcode(),
        badi_.inst(),
        badv_.vaddr(),
        crmd_,
        prmd_.pplv(),
        prmd_.pie(),
        prmd_.pwe(),
        era_.pc(),
        tcfg_.en(),
        tcfg_.periodic(),
        tcfg_.init_val(),
        tval_.time_val(),
        ecfg_.lie(),
        desave_r
    );

    // GCSR
    let gcsr_crmd = read_gcsr_crmd();
    let gcsr_prmd = read_gcsr_prmd();
    let gcsr_ectl = read_gcsr_ectl();
    let gcsr_estat = read_gcsr_estat();
    let gcsr_era = read_gcsr_era();
    let gcsr_badv = read_gcsr_badv();
    let gcsr_badi = read_gcsr_badi();
    let gcsr_eentry = read_gcsr_eentry();
    let gcsr_tcfg = read_gcsr_tcfg();
    let gcsr_tval = read_gcsr_tval();
    println!("[GCSR]");
    print!(
        "  gcsr.crmd={:#x}, gcsr.prmd={:#x}, gcsr.ectl={:#x}\n  \
      gcsr.estat={:#x}, gcsr.era={:#x}, gcsr.badv={:#x}\n  \
      gcsr.badi={:#x}, gcsr.eentry={:#x}, gcsr.tcfg={:#x}, gcsr.tval={:#x}\n",
        gcsr_crmd,
        gcsr_prmd,
        gcsr_ectl,
        gcsr_estat,
        gcsr_era,
        gcsr_badv,
        gcsr_badi,
        gcsr_eentry,
        gcsr_tcfg,
        gcsr_tval
    );

    // LVZ registers
    let gstat = gstat::read();
    println!("[LVZ]");
    print!(
        "  csr.gstat.pgm={}, csr.gstat.gid={:#x}\n",
        gstat.pgm(),
        gstat.gid()
    );

    info!("core dump finished");
}
