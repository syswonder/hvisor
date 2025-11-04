#![allow(unused)]

use aarch64_cpu::registers::Writeable;
use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

use crate::arch::mpam::mem::{MPAMCFG_MBW_MAX, MPAMCFG_MBW_MIN};

use super::mem::MPAMF_MBW_IDR;

use super::cache::{MPAMCFG_CMAX, MPAMF_CCAP_IDR, MPAMF_CPOR_IDR};

register_structs! {
    pub MpamRegisters {
        (0x0000 => pub mpamf_idr: ReadOnly<u64, MPAMF_IDR::Register>),
        (0x0008 => _reserved0),
        (0x0030 => pub mpamf_cpor_idr: ReadOnly<u32, MPAMF_CPOR_IDR::Register>),
        (0x0034 => _reserved_cpor),
        (0x0038 => pub mpamf_ccap_idr: ReadOnly<u32, MPAMF_CCAP_IDR::Register>),
        (0x003C => _reserved_ccap),
        (0x0040 => pub mpamf_mbw_idr: ReadOnly<u32, MPAMF_MBW_IDR::Register>),
        (0x0044 => _reserved_mbw),
        (0x0100 => pub mpamcfg_part_sel: ReadWrite<u32, MPAMCFG_PART_SEL::Register>),
        (0x0104 => _reserved1),
        (0x0108 => pub mpamcfg_cmax: ReadWrite<u32, MPAMCFG_CMAX::Register>),
        (0x010c => _reserved2),
        (0x0200 => pub mpamcfg_mbw_min: ReadWrite<u32, MPAMCFG_MBW_MIN::Register>),
        (0x0204 => _reserved_mbw_min),
        (0x0208 => pub mpamcfg_mbw_max: ReadWrite<u32, MPAMCFG_MBW_MAX::Register>),
        (0x020c => _reserved_mbw_prop),
        (0x1000 => pub cpbm_n: [ReadWrite<u32>; 1024]),
        (0x2000 => pub mbwpbm_n: [ReadWrite<u32>; 128]),
        (0x2200 => @END),
    }
}

impl MpamRegisters {
    pub fn new(base_addr: u64) -> &'static Self {
        unsafe { &*(base_addr as *const MpamRegisters) }
    }
    /// Select a PARTID and RIS for subsequent MPAM register accesses.
    /// if RIS is not enabled, ris should be 0.
    /// If `internal` is true, select the internal resource instance.
    pub fn select_partid_ris(&self, partid: u16, ris: u8, internal: bool) {
        let part_sel_value = MPAMCFG_PART_SEL::PARTID_SEL.val(partid as u32)
            + MPAMCFG_PART_SEL::RIS.val(ris as u32)
            + if internal {
                MPAMCFG_PART_SEL::INTERNAL.val(1)
            } else {
                MPAMCFG_PART_SEL::INTERNAL.val(0)
            };
        self.mpamcfg_part_sel.write(part_sel_value);
    }
}

register_bitfields! [
    u64,
    pub MPAMF_IDR [
        Reserved    OFFSET(60) NUMBITS(4) [],
        RIS_MAX     OFFSET(56) NUMBITS(4) [],
        Reserved1   OFFSET(44) NUMBITS(12) [],
        HAS_NFU     OFFSET(43) NUMBITS(1) [],
        HAS_ENDIS   OFFSET(42) NUMBITS(1) [],
        SP4         OFFSET(41) NUMBITS(1) [],
        HAS_ERR_MSI OFFSET(40) NUMBITS(1) [],
        HAS_ESR     OFFSET(39) NUMBITS(1) [],
        HAS_EXTD_ESR OFFSET(38) NUMBITS(1) [],
        NO_IMPL_MSMON OFFSET(37) NUMBITS(1) [],
        NO_IMPL_PART OFFSET(36) NUMBITS(1) [],
        Reserved2   OFFSET(33) NUMBITS(3) [],
        HAS_RIS     OFFSET(32) NUMBITS(1) [],
        PARTID_NRW  OFFSET(31) NUMBITS(1) [],   // bit 31
        MSMON       OFFSET(30) NUMBITS(1) [],   // bit 30
        IMPL_IDR    OFFSET(29) NUMBITS(1) [],   // bit 29
        EXT         OFFSET(28) NUMBITS(1) [],   // bit 28
        PRI_PART    OFFSET(27) NUMBITS(1) [],   // bit 27
        MBW_PART    OFFSET(26) NUMBITS(1) [],   // bit 26
        CPOR_PART   OFFSET(25) NUMBITS(1) [],   // bit 25
        CCAP_PART   OFFSET(24) NUMBITS(1) [],   // bit 24
        PMG_MAX     OFFSET(16) NUMBITS(8)  [],  // bits 16–23
        PARTID_MAX  OFFSET(0)  NUMBITS(16) []   // bits 0–15
    ]
];

register_bitfields![
    u32,
    pub MPAMCFG_PART_SEL [
        Reserved OFFSET(28) NUMBITS(4) [],
        RIS OFFSET(24) NUMBITS(4) [],
        Reserved2 OFFSET(17) NUMBITS(7) [],
        INTERNAL OFFSET(16) NUMBITS(1) [],
        PARTID_SEL OFFSET(0) NUMBITS(16) [],
    ]
];

register_bitfields![
    u32,
    pub MPAM_PMG_IDR [
        RES0         OFFSET(16) NUMBITS(16) [], // bits 16-31
        PMG_WD       OFFSET(0)  NUMBITS(16) []  // bits 0-15
    ]
];

// register_bitfields! {
//     u64,
//     pub MPAMIDR_EL1 [
//         HAS_SDEFLT OFFSET(61) NUMBITS(1),
//         HAS_FORCE_NS OFFSET(60) NUMBITS(1),
//         HAS_TIDR OFFSET(58) NUMBITS(1),
//         PMG_MAX OFFSET(32) NUMBITS(8),
//         VPMR_MAX OFFSET(18) NUMBITS(3),
//         HAS_HCR OFFSET(17) NUMBITS(1),
//         PARTID_MAX OFFSET(0) NUMBITS(16),
//     ]
// }

#[derive(Debug, Clone, Copy)]
pub struct MpamIdrEL1(u64);

impl MpamIdrEL1 {
    const PARTID_MAX_MASK: u64 = 0xFFFF;
    const HAS_HCR_BIT: u64 = 1 << 17;
    const VPMR_MAX_SHIFT: u32 = 18;
    const VPMR_MAX_MASK: u64 = 0x7;
    const PMG_MAX_SHIFT: u32 = 32;
    const PMG_MAX_MASK: u64 = 0xFF;
    const HAS_TIDR_BIT: u64 = 1 << 58;
    const HAS_FORCE_NS_BIT: u64 = 1 << 60;
    const HAS_SDEFLT_BIT: u64 = 1 << 61;

    #[inline(always)]
    pub fn read() -> Self {
        let value: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, MPAMIDR_EL1",
                out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        Self(value)
    }
    #[inline(always)]
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
    #[inline(always)]
    pub fn raw(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    pub fn partid_max(&self) -> u16 {
        (self.0 & Self::PARTID_MAX_MASK) as u16
    }
    #[inline(always)]
    pub fn has_hcr(&self) -> bool {
        self.0 & Self::HAS_HCR_BIT != 0
    }
    #[inline(always)]
    pub fn vpmr_max(&self) -> u8 {
        ((self.0 >> Self::VPMR_MAX_SHIFT) & Self::VPMR_MAX_MASK) as u8
    }
    #[inline(always)]
    pub fn pmg_max(&self) -> u8 {
        ((self.0 >> Self::PMG_MAX_SHIFT) & Self::PMG_MAX_MASK) as u8
    }
    #[inline(always)]
    pub fn has_tidr(&self) -> bool {
        self.0 & Self::HAS_TIDR_BIT != 0
    }
    #[inline(always)]
    pub fn has_force_ns(&self) -> bool {
        self.0 & Self::HAS_FORCE_NS_BIT != 0
    }
    #[inline(always)]
    pub fn has_sdeflt(&self) -> bool {
        self.0 & Self::HAS_SDEFLT_BIT != 0
    }
}
