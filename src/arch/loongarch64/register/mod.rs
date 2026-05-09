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
// File:          mod.rs
// Description:   this is the register file of loongarch64's LVZ extension
// Authors:       wheatfox(wheatfox17@icloud.com)
// Created:       2023-12-20
// Updated:       2026-04-05

#![allow(unused)]

use bit_field::BitField;
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

// more interfaces... boneinscri 2026.04
// write GCSR
pub fn write_gcsr_crmd(value: usize) {
    write_gcsr_loong!(0x0, value);
}
pub fn write_gcsr_prmd(value: usize) {
    write_gcsr_loong!(0x1, value);
}
pub fn write_gcsr_euen(value: usize) {
    write_gcsr_loong!(0x2, value);
}
pub fn write_gcsr_misc(value: usize) {
    write_gcsr_loong!(0x3, value);
}
pub fn write_gcsr_ecfg(value: usize) {
    write_gcsr_loong!(0x4, value);
}
pub fn write_gcsr_estat(value: usize) {
    write_gcsr_loong!(0x5, value);
}
pub fn write_gcsr_era(value: usize) {
    write_gcsr_loong!(0x6, value);
}
pub fn write_gcsr_badv(value: usize) {
    write_gcsr_loong!(0x7, value);
}
pub fn write_gcsr_badi(value: usize) {
    write_gcsr_loong!(0x8, value);
}
pub fn write_gcsr_eentry(value: usize) {
    write_gcsr_loong!(0xc, value);
}
pub fn write_gcsr_tlbidx(value: usize) {
    write_gcsr_loong!(0x10, value);
}
pub fn write_gcsr_tlbehi(value: usize) {
    write_gcsr_loong!(0x11, value);
}
pub fn write_gcsr_tlbelo0(value: usize) {
    write_gcsr_loong!(0x12, value);
}
pub fn write_gcsr_tlbelo1(value: usize) {
    write_gcsr_loong!(0x13, value);
}
pub fn write_gcsr_asid(value: usize) {
    write_gcsr_loong!(0x18, value);
}
pub fn write_gcsr_pgdl(value: usize) {
    write_gcsr_loong!(0x19, value);
}
pub fn write_gcsr_pgdh(value: usize) {
    write_gcsr_loong!(0x1a, value);
}
pub fn write_gcsr_pwcl(value: usize) {
    write_gcsr_loong!(0x1c, value);
}
pub fn write_gcsr_pwch(value: usize) {
    write_gcsr_loong!(0x1d, value);
}
pub fn write_gcsr_stlbps(value: usize) {
    write_gcsr_loong!(0x1e, value);
}
pub fn write_gcsr_rvacfg(value: usize) {
    write_gcsr_loong!(0x1f, value);
}
pub fn write_gcsr_cpuid(value: usize) {
    write_gcsr_loong!(0x20, value);
}
pub fn write_gcsr_prcfg1(value: usize) {
    write_gcsr_loong!(0x21, value);
}
pub fn write_gcsr_prcfg2(value: usize) {
    write_gcsr_loong!(0x22, value);
}
pub fn write_gcsr_prcfg3(value: usize) {
    write_gcsr_loong!(0x23, value);
}
pub fn write_gcsr_save0(value: usize) {
    write_gcsr_loong!(0x30, value);
}
pub fn write_gcsr_save1(value: usize) {
    write_gcsr_loong!(0x31, value);
}
pub fn write_gcsr_save2(value: usize) {
    write_gcsr_loong!(0x32, value);
}
pub fn write_gcsr_save3(value: usize) {
    write_gcsr_loong!(0x33, value);
}
pub fn write_gcsr_save4(value: usize) {
    write_gcsr_loong!(0x34, value);
}
pub fn write_gcsr_save5(value: usize) {
    write_gcsr_loong!(0x35, value);
}
pub fn write_gcsr_save6(value: usize) {
    write_gcsr_loong!(0x36, value);
}
pub fn write_gcsr_save7(value: usize) {
    write_gcsr_loong!(0x37, value);
}
pub fn write_gcsr_tid(value: usize) {
    write_gcsr_loong!(0x40, value);
}
pub fn write_gcsr_tcfg(value: usize) {
    write_gcsr_loong!(0x41, value);
}
pub fn write_gcsr_tval(value: usize) {
    write_gcsr_loong!(0x42, value);
}
pub fn write_gcsr_cntc(value: usize) {
    write_gcsr_loong!(0x43, value);
}
pub fn write_gcsr_ticlr(value: usize) {
    write_gcsr_loong!(0x44, value);
}
pub fn write_gcsr_tlbrentry(value: usize) {
    write_gcsr_loong!(0x88, value);
}
pub fn write_gcsr_tlbrbadv(value: usize) {
    write_gcsr_loong!(0x89, value);
}
pub fn write_gcsr_tlbrera(value: usize) {
    write_gcsr_loong!(0x8a, value);
}
pub fn write_gcsr_tlbrsave(value: usize) {
    write_gcsr_loong!(0x8b, value);
}
pub fn write_gcsr_tlbrelo0(value: usize) {
    write_gcsr_loong!(0x8c, value);
}
pub fn write_gcsr_tlbrelo1(value: usize) {
    write_gcsr_loong!(0x8d, value);
}
pub fn write_gcsr_tlbrehi(value: usize) {
    write_gcsr_loong!(0x8e, value);
}
pub fn write_gcsr_tlbrprmd(value: usize) {
    write_gcsr_loong!(0x8f, value);
}
pub fn write_gcsr_dmw0(value: usize) {
    write_gcsr_loong!(0x180, value);
}
pub fn write_gcsr_dmw1(value: usize) {
    write_gcsr_loong!(0x181, value);
}
pub fn write_gcsr_dmw2(value: usize) {
    write_gcsr_loong!(0x182, value);
}
pub fn write_gcsr_dmw3(value: usize) {
    write_gcsr_loong!(0x183, value);
}

// WRITE CSR
pub fn write_csr_crmd(value: usize) {
    write_csr_loong!(0x0, value);
}
pub fn write_csr_prmd(value: usize) {
    write_csr_loong!(0x1, value);
}
pub fn write_csr_euen(value: usize) {
    write_csr_loong!(0x2, value);
}
pub fn write_csr_misc(value: usize) {
    write_csr_loong!(0x3, value);
}
pub fn write_csr_ecfg(value: usize) {
    write_csr_loong!(0x4, value);
}
pub fn write_csr_estat(value: usize) {
    write_csr_loong!(0x5, value);
}
pub fn write_csr_era(value: usize) {
    write_csr_loong!(0x6, value);
}
pub fn write_csr_badv(value: usize) {
    write_csr_loong!(0x7, value);
}
pub fn write_csr_badi(value: usize) {
    write_csr_loong!(0x8, value);
}
pub fn write_csr_eentry(value: usize) {
    write_csr_loong!(0xc, value);
}
pub fn write_csr_tlbidx(value: usize) {
    write_csr_loong!(0x10, value);
}
pub fn write_csr_tlbehi(value: usize) {
    write_csr_loong!(0x11, value);
}
pub fn write_csr_tlbelo0(value: usize) {
    write_csr_loong!(0x12, value);
}
pub fn write_csr_tlbelo1(value: usize) {
    write_csr_loong!(0x13, value);
}
pub fn write_csr_asid(value: usize) {
    write_csr_loong!(0x18, value);
}
pub fn write_csr_pgdl(value: usize) {
    write_csr_loong!(0x19, value);
}
pub fn write_csr_pgdh(value: usize) {
    write_csr_loong!(0x1a, value);
}
pub fn write_csr_pwcl(value: usize) {
    write_csr_loong!(0x1c, value);
}
pub fn write_csr_pwch(value: usize) {
    write_csr_loong!(0x1d, value);
}
pub fn write_csr_stlbps(value: usize) {
    write_csr_loong!(0x1e, value);
}
pub fn write_csr_rvacfg(value: usize) {
    write_csr_loong!(0x1f, value);
}
pub fn write_csr_cpuid(value: usize) {
    write_csr_loong!(0x20, value);
}
pub fn write_csr_prcfg1(value: usize) {
    write_csr_loong!(0x21, value);
}
pub fn write_csr_prcfg2(value: usize) {
    write_csr_loong!(0x22, value);
}
pub fn write_csr_prcfg3(value: usize) {
    write_csr_loong!(0x23, value);
}
pub fn write_csr_save0(value: usize) {
    write_csr_loong!(0x30, value);
}
pub fn write_csr_save1(value: usize) {
    write_csr_loong!(0x31, value);
}
pub fn write_csr_save2(value: usize) {
    write_csr_loong!(0x32, value);
}
pub fn write_csr_save3(value: usize) {
    write_csr_loong!(0x33, value);
}
pub fn write_csr_save4(value: usize) {
    write_csr_loong!(0x34, value);
}
pub fn write_csr_save5(value: usize) {
    write_csr_loong!(0x35, value);
}
pub fn write_csr_save6(value: usize) {
    write_csr_loong!(0x36, value);
}
pub fn write_csr_save7(value: usize) {
    write_csr_loong!(0x37, value);
}
pub fn write_csr_tid(value: usize) {
    write_csr_loong!(0x40, value);
}
pub fn write_csr_tcfg(value: usize) {
    write_csr_loong!(0x41, value);
}
pub fn write_csr_tval(value: usize) {
    write_csr_loong!(0x42, value);
}
pub fn write_csr_cntc(value: usize) {
    write_csr_loong!(0x43, value);
}
pub fn write_csr_ticlr(value: usize) {
    write_csr_loong!(0x44, value);
}
pub fn write_csr_tlbrentry(value: usize) {
    write_csr_loong!(0x88, value);
}
pub fn write_csr_tlbrbadv(value: usize) {
    write_csr_loong!(0x89, value);
}
pub fn write_csr_tlbrera(value: usize) {
    write_csr_loong!(0x8a, value);
}
pub fn write_csr_tlbrsave(value: usize) {
    write_csr_loong!(0x8b, value);
}
pub fn write_csr_tlbrelo0(value: usize) {
    write_csr_loong!(0x8c, value);
}
pub fn write_csr_tlbrelo1(value: usize) {
    write_csr_loong!(0x8d, value);
}
pub fn write_csr_tlbrehi(value: usize) {
    write_csr_loong!(0x8e, value);
}
pub fn write_csr_tlbrprmd(value: usize) {
    write_csr_loong!(0x8f, value);
}
pub fn write_csr_dmw0(value: usize) {
    write_csr_loong!(0x180, value);
}
pub fn write_csr_dmw1(value: usize) {
    write_csr_loong!(0x181, value);
}
pub fn write_csr_dmw2(value: usize) {
    write_csr_loong!(0x182, value);
}
pub fn write_csr_dmw3(value: usize) {
    write_csr_loong!(0x183, value);
}
pub fn write_csr_gintc(value: usize) {
    write_csr_loong!(0x52, value);
}

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
pub fn read_gcsr_tlbidx() -> usize {
    read_gcsr_loong!(0x10)
}
pub fn read_gcsr_tlbehi() -> usize {
    read_gcsr_loong!(0x11)
}
pub fn read_gcsr_tlbelo0() -> usize {
    read_gcsr_loong!(0x12)
}
pub fn read_gcsr_tlbelo1() -> usize {
    read_gcsr_loong!(0x13)
}
pub fn read_gcsr_asid() -> usize {
    read_gcsr_loong!(0x18)
}
pub fn read_gcsr_pgdl() -> usize {
    read_gcsr_loong!(0x19)
}
pub fn read_gcsr_pgdh() -> usize {
    read_gcsr_loong!(0x1a)
}
pub fn read_gcsr_pgd() -> usize {
    read_gcsr_loong!(0x1b)
}
pub fn read_gcsr_pwcl() -> usize {
    read_gcsr_loong!(0x1c)
}
pub fn read_gcsr_pwch() -> usize {
    read_gcsr_loong!(0x1d)
}
pub fn read_gcsr_stlbps() -> usize {
    read_gcsr_loong!(0x1e)
}
pub fn read_gcsr_ravcfg() -> usize {
    read_gcsr_loong!(0x1f)
}
pub fn read_gcsr_cpuid() -> usize {
    read_gcsr_loong!(0x20)
}
pub fn read_gcsr_prcfg1() -> usize {
    read_gcsr_loong!(0x21)
}
pub fn read_gcsr_prcfg2() -> usize {
    read_gcsr_loong!(0x22)
}
pub fn read_gcsr_prcfg3() -> usize {
    read_gcsr_loong!(0x23)
}
pub fn read_gcsr_save0() -> usize {
    read_gcsr_loong!(0x30)
}
pub fn read_gcsr_save1() -> usize {
    read_gcsr_loong!(0x31)
}
pub fn read_gcsr_save2() -> usize {
    read_gcsr_loong!(0x32)
}
pub fn read_gcsr_save3() -> usize {
    read_gcsr_loong!(0x33)
}
pub fn read_gcsr_save4() -> usize {
    read_gcsr_loong!(0x34)
}
pub fn read_gcsr_save5() -> usize {
    read_gcsr_loong!(0x35)
}
pub fn read_gcsr_save6() -> usize {
    read_gcsr_loong!(0x36)
}
pub fn read_gcsr_save7() -> usize {
    read_gcsr_loong!(0x37)
}
pub fn read_gcsr_save8() -> usize {
    read_gcsr_loong!(0x38)
}
pub fn read_gcsr_save9() -> usize {
    read_gcsr_loong!(0x39)
}
pub fn read_gcsr_save10() -> usize {
    read_gcsr_loong!(0x3a)
}
pub fn read_gcsr_save11() -> usize {
    read_gcsr_loong!(0x3b)
}
pub fn read_gcsr_save12() -> usize {
    read_gcsr_loong!(0x3c)
}
pub fn read_gcsr_save13() -> usize {
    read_gcsr_loong!(0x3d)
}
pub fn read_gcsr_save14() -> usize {
    read_gcsr_loong!(0x3e)
}
pub fn read_gcsr_save15() -> usize {
    read_gcsr_loong!(0x3f)
}
pub fn read_gcsr_tid() -> usize {
    read_gcsr_loong!(0x40)
}
pub fn read_gcsr_tcfg() -> usize {
    read_gcsr_loong!(0x41)
}
pub fn read_gcsr_tval() -> usize {
    read_gcsr_loong!(0x42)
}
pub fn read_gcsr_cntc() -> usize {
    read_gcsr_loong!(0x43)
}
pub fn read_gcsr_ticlr() -> usize {
    read_gcsr_loong!(0x44)
}
pub fn read_gcsr_llbctl() -> usize {
    read_gcsr_loong!(0x60)
}
pub fn read_gcsr_tlbrentry() -> usize {
    read_gcsr_loong!(0x88)
}
pub fn read_gcsr_tlbrbadv() -> usize {
    read_gcsr_loong!(0x89)
}
pub fn read_gcsr_tlbrera() -> usize {
    read_gcsr_loong!(0x8a)
}
pub fn read_gcsr_tlbrsave() -> usize {
    read_gcsr_loong!(0x8b)
}
pub fn read_gcsr_tlbrrelo0() -> usize {
    read_gcsr_loong!(0x8c)
}
pub fn read_gcsr_tlbrrelo1() -> usize {
    read_gcsr_loong!(0x8d)
}
pub fn read_gcsr_tlbrrehi() -> usize {
    read_gcsr_loong!(0x8e)
}
pub fn read_gcsr_tlbrprmd() -> usize {
    read_gcsr_loong!(0x8f)
}
pub fn read_gcsr_dmw0() -> usize {
    read_gcsr_loong!(0x180)
}
pub fn read_gcsr_dmw1() -> usize {
    read_gcsr_loong!(0x181)
}
pub fn read_gcsr_dmw2() -> usize {
    read_gcsr_loong!(0x182)
}
pub fn read_gcsr_dmw3() -> usize {
    read_gcsr_loong!(0x183)
}

// READ CSR
pub fn read_csr_crmd() -> usize {
    read_csr_loong!(0x0)
}
pub fn read_csr_prmd() -> usize {
    read_csr_loong!(0x1)
}
pub fn read_csr_euen() -> usize {
    read_csr_loong!(0x2)
}
pub fn read_csr_misc() -> usize {
    read_csr_loong!(0x3)
}
pub fn read_csr_ectl() -> usize {
    read_csr_loong!(0x4)
}
pub fn read_csr_estat() -> usize {
    read_csr_loong!(0x5)
}
pub fn read_csr_era() -> usize {
    read_csr_loong!(0x6)
}
pub fn read_csr_badv() -> usize {
    read_csr_loong!(0x7)
}
pub fn read_csr_badi() -> usize {
    read_csr_loong!(0x8)
}
pub fn read_csr_eentry() -> usize {
    read_csr_loong!(0xc)
}
pub fn read_csr_tlbidx() -> usize {
    read_csr_loong!(0x10)
}
pub fn read_csr_tlbehi() -> usize {
    read_csr_loong!(0x11)
}
pub fn read_csr_tlbelo0() -> usize {
    read_csr_loong!(0x12)
}
pub fn read_csr_tlbelo1() -> usize {
    read_csr_loong!(0x13)
}
pub fn read_csr_asid() -> usize {
    read_csr_loong!(0x18)
}
pub fn read_csr_pgdl() -> usize {
    read_csr_loong!(0x19)
}
pub fn read_csr_pgdh() -> usize {
    read_csr_loong!(0x1a)
}
pub fn read_csr_pgd() -> usize {
    read_csr_loong!(0x1b)
}
pub fn read_csr_pwcl() -> usize {
    read_csr_loong!(0x1c)
}
pub fn read_csr_pwch() -> usize {
    read_csr_loong!(0x1d)
}
pub fn read_csr_stlbps() -> usize {
    read_csr_loong!(0x1e)
}
pub fn read_csr_ravcfg() -> usize {
    read_csr_loong!(0x1f)
}
pub fn read_csr_cpuid() -> usize {
    read_csr_loong!(0x20)
}
pub fn read_csr_prcfg1() -> usize {
    read_csr_loong!(0x21)
}
pub fn read_csr_prcfg2() -> usize {
    read_csr_loong!(0x22)
}
pub fn read_csr_prcfg3() -> usize {
    read_csr_loong!(0x23)
}
pub fn read_csr_save0() -> usize {
    read_csr_loong!(0x30)
}
pub fn read_csr_save1() -> usize {
    read_csr_loong!(0x31)
}
pub fn read_csr_save2() -> usize {
    read_csr_loong!(0x32)
}
pub fn read_csr_save3() -> usize {
    read_csr_loong!(0x33)
}
pub fn read_csr_save4() -> usize {
    read_csr_loong!(0x34)
}
pub fn read_csr_save5() -> usize {
    read_csr_loong!(0x35)
}
pub fn read_csr_save6() -> usize {
    read_csr_loong!(0x36)
}
pub fn read_csr_save7() -> usize {
    read_csr_loong!(0x37)
}
pub fn read_csr_save8() -> usize {
    read_csr_loong!(0x38)
}
pub fn read_csr_save9() -> usize {
    read_csr_loong!(0x39)
}
pub fn read_csr_save10() -> usize {
    read_csr_loong!(0x3a)
}
pub fn read_csr_save11() -> usize {
    read_csr_loong!(0x3b)
}
pub fn read_csr_save12() -> usize {
    read_csr_loong!(0x3c)
}
pub fn read_csr_save13() -> usize {
    read_csr_loong!(0x3d)
}
pub fn read_csr_save14() -> usize {
    read_csr_loong!(0x3e)
}
pub fn read_csr_save15() -> usize {
    read_csr_loong!(0x3f)
}
pub fn read_csr_tid() -> usize {
    read_csr_loong!(0x40)
}
pub fn read_csr_tcfg() -> usize {
    read_csr_loong!(0x41)
}
pub fn read_csr_tval() -> usize {
    read_csr_loong!(0x42)
}
pub fn read_csr_cntc() -> usize {
    read_csr_loong!(0x43)
}
pub fn read_csr_ticlr() -> usize {
    read_csr_loong!(0x44)
}
pub fn read_csr_llbctl() -> usize {
    read_csr_loong!(0x60)
}
pub fn read_csr_tlbrentry() -> usize {
    read_csr_loong!(0x88)
}
pub fn read_csr_tlbrbadv() -> usize {
    read_csr_loong!(0x89)
}
pub fn read_csr_tlbrera() -> usize {
    read_csr_loong!(0x8a)
}
pub fn read_csr_tlbrsave() -> usize {
    read_csr_loong!(0x8b)
}
pub fn read_csr_tlbrrelo0() -> usize {
    read_csr_loong!(0x8c)
}
pub fn read_csr_tlbrrelo1() -> usize {
    read_csr_loong!(0x8d)
}
pub fn read_csr_tlbrrehi() -> usize {
    read_csr_loong!(0x8e)
}
pub fn read_csr_tlbrprmd() -> usize {
    read_csr_loong!(0x8f)
}
pub fn read_csr_dmw0() -> usize {
    read_csr_loong!(0x180)
}
pub fn read_csr_dmw1() -> usize {
    read_csr_loong!(0x181)
}
pub fn read_csr_dmw2() -> usize {
    read_csr_loong!(0x182)
}
pub fn read_csr_dmw3() -> usize {
    read_csr_loong!(0x183)
}
pub fn read_csr_gintc() -> usize {
    read_csr_loong!(0x52)
}

pub fn set_gcsr_estat(val: usize) {
    gcsr_xchg!(val, val, 0x5);
}

pub fn clear_gcsr_estat(val: usize) {
    gcsr_xchg!(!(val), val, 0x5);
}

pub fn set_csr_gintc(val: usize) {
    csr_xchg!(val, val, 0x52);
}

pub fn clear_csr_gintc(val: usize) {
    csr_xchg!(!(val), val, 0x52);
}