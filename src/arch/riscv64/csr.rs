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
#![allow(dead_code)]
pub const CSR_SCAUSE: u64 = 0x142;
pub const CSR_STVEC: u64 = 0x105;
pub const CSR_SCOUNTEREN: u64 = 0x106;
pub const CSR_SEPC: u64 = 0x141;
pub const CSR_SSTATUS: u64 = 0x100;
pub const CSR_SSCRATCH: u64 = 0x140;
pub const CSR_SIE: u64 = 0x104;
pub const CSR_SIP: u64 = 0x144;
pub const CSR_VSSTATUS: u64 = 0x200;
pub const CSR_VSIE: u64 = 0x204;
pub const CSR_VSTVEC: u64 = 0x205;
pub const CSR_VSSCRATCH: u64 = 0x240;
pub const CSR_VSEPC: u64 = 0x241;
pub const CSR_VSCAUSE: u64 = 0x242;
pub const CSR_VSTVAL: u64 = 0x243;
pub const CSR_VSIP: u64 = 0x244;
pub const CSR_VSATP: u64 = 0x280;
/* Sstc Extension */
pub const CSR_VSTIMECMP: u64 = 0x24D;
pub const CSR_VSTIMECMPH: u64 = 0x25D;

pub const CSR_HSTATUS: u64 = 0x600;
pub const CSR_HEDELEG: u64 = 0x602;
pub const CSR_HIDELEG: u64 = 0x603;
pub const CSR_HIE: u64 = 0x604;
pub const CSR_HTIMEDELTA: u64 = 0x605;
pub const CSR_HTIMEDELTAH: u64 = 0x615;
pub const CSR_HCOUNTEREN: u64 = 0x606;
pub const CSR_HGEIE: u64 = 0x607;
pub const CSR_HTVAL: u64 = 0x643;
pub const CSR_HIP: u64 = 0x644;
pub const CSR_HVIP: u64 = 0x645;
pub const CSR_HTINST: u64 = 0x64A;
pub const CSR_HGATP: u64 = 0x680;
pub const CSR_HGEIP: u64 = 0xE07;
/* Hypervisor Configuration */
pub const CSR_HENVCFG: u64 = 0x60A;
pub const CSR_HENVCFGH: u64 = 0x61A;
pub const CSR_HSTATEEN0: u64 = 0x60C;

/* Sstc Extension */
pub const CSR_STIMECMP: u64 = 0x14D;
pub const CSR_STIMECMPH: u64 = 0x15D;

pub const HSTATUS_SPV: u64 = 1 << 7;

// sstatus/vsstatus state fields.
pub const SSTATUS_SPP: usize = 1 << 8;
pub const SSTATUS_FS_DIRTY: usize = 0b11 << 13;
// vsstatus.VS: vector state is dirty and available to VS/VU-mode.
pub const SSTATUS_VS_SHIFT: usize = 9;
pub const SSTATUS_VS_DIRTY: usize = 0b11 << SSTATUS_VS_SHIFT;

// HENVCFG.STCE: enable vstimecmp access from VS-mode (Sstc).
pub const HENVCFG_STCE: usize = 1 << 63;
// HENVCFG.PBMTE: enable page-based memory types from Svpbmt in VS-mode.
pub const HENVCFG_PBMTE: usize = 1 << 62;
// HENVCFG controls for cache-block operations advertised to VS-mode.
// Zicbom uses CBIE[5:4]=01 for CBO.INVAL (performed as a flush) and CBCFE for
// CBO.CLEAN/CBO.FLUSH. Zicboz uses CBZE for CBO.ZERO. Zicbop prefetch hints
// do not require an HENVCFG enable bit.
pub const HENVCFG_CBIE: usize = 1 << 4;
pub const HENVCFG_CBCFE: usize = 1 << 6;
pub const HENVCFG_CBZE: usize = 1 << 7;

// HSTATEEN0 controls access to extension state from VS/VU-mode.
// These fields are defined by Smstateen/Ssstateen and AIA:
//   bit 58: IMSIC guest interrupt-file state, including vstopei.
//   bit 59: AIA state not covered by CSRIND or IMSIC.
//   bit 60: indirect CSR state (siselect/sireg, exposed as vsiselect/vsireg).
//   bit 62: senvcfg state.
//   bit 63: sstateen* CSRs, i.e. the supervisor state-enable view for VS-mode.
pub const HSTATEEN0_IMSIC: usize = 1 << 58;
pub const HSTATEEN0_AIA: usize = 1 << 59;
pub const HSTATEEN0_CSRIND: usize = 1 << 60;
pub const HSTATEEN0_SENVCFG: usize = 1 << 62;
pub const HSTATEEN0_SSTATEEN: usize = 1 << 63;

macro_rules! read_csr {
    ($csr_number:expr) => {
        {
            let mut value: usize;
            unsafe{
                ::core::arch::asm!(
                "csrr {value},  {csr}",
                value = out(reg) value,
                csr=const $csr_number,
                options(nomem, nostack),
            );}
            value
        }
    }
}
pub(crate) use read_csr;

macro_rules! write_csr {
    ($csr_number:expr, $value: expr) => {
        unsafe{
            let v: usize = $value;
            ::core::arch::asm!(
                "csrw {csr}, {value}",
                value = in(reg) v,
                csr = const $csr_number,
                options(nomem, nostack),)}
    };
}
pub(crate) use write_csr;

#[allow(unused)]
macro_rules! clear_csr {
    ($csr_number:expr, $value: expr) => {
        unsafe{
            let v: usize = $value;
            ::core::arch::asm!(
                "csrc {csr}, {value}",
                value = in(reg) v,
                csr = const $csr_number,
                options(nomem, nostack),)}
    };
}
#[allow(unused)]
pub(crate) use clear_csr;

macro_rules! set_csr {
    ($csr_number:expr, $value: expr) => {
        unsafe{
            let v: usize = $value;
            ::core::arch::asm!(
                "csrs {csr}, {value}",
                value = in(reg) v,
                csr = const $csr_number,
                options(nomem, nostack),)}
    };
}
pub(crate) use set_csr;
