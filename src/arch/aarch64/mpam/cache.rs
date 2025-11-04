use aarch64_cpu::registers::{Readable, Writeable};
use tock_registers::register_bitfields;

use crate::arch::mpam::simple_round;

use super::{
    error::{MpamError, MpamResult},
    regs::MpamRegisters,
};

register_bitfields![
    u32,
    pub MPAMF_CPOR_IDR [
        RES0      OFFSET(16) NUMBITS(16) [], // bits 16-31
        CPBM_WD   OFFSET(0)  NUMBITS(16) []  // bits 0-15
    ]
];

register_bitfields![
    u32,
    pub MPAMF_CCAP_IDR [
        HAS_CMAX_SOFTLIM OFFSET(31) NUMBITS(1) [], // bit 31
        NO_CMAX          OFFSET(30) NUMBITS(1) [], // bit 30
        HAS_CMIN         OFFSET(29) NUMBITS(1) [], // bit 29
        HAS_CASSOC       OFFSET(28) NUMBITS(1) [], // bit 28
        CASSOC_WD       OFFSET(8)  NUMBITS(5) [], // bits 8-12
        CMAX_WD         OFFSET(0)  NUMBITS(6) []  // bits 0-5
    ]
];

register_bitfields![
    u32,
    pub MPAMCFG_CMAX [
        SOFTLIM OFFSET(31) NUMBITS(1) [], // bit 31
        Reserved OFFSET(16) NUMBITS(15) [], // bits 16-30
        CMAX     OFFSET(0)  NUMBITS(16) [], // bits 0-15
    ]
];

/* MPAMF_CPOR_IDR, MPAMF_CPMBn : Cache Partition Bitmap */
#[allow(unused)]
impl MpamRegisters {
    fn cpbm_wd(&self) -> u16 {
        self.mpamf_cpor_idr.read(MPAMF_CPOR_IDR::CPBM_WD) as u16
    }
    fn write_cpbm(&self, bitmap_words: &[u32]) -> MpamResult<()> {
        let wd = self.cpbm_wd();
        if wd == 0 || wd > 32768 {
            return Err(MpamError::InvalidValue);
        }
        let needed_words = ((wd as usize) + 31) / 32;
        if bitmap_words.len() < needed_words {
            return Err(MpamError::InvalidValue);
        }
        for i in 0..needed_words {
            self.cpbm_n[i].set(bitmap_words[i]);
        }
        Ok(())
    }
    pub fn set_part_cpbm(
        &self,
        partid: u16,
        ris: u8,
        internal: bool,
        bitmap_words: &[u32],
    ) -> MpamResult<()> {
        self.select_partid_ris(partid, ris, internal);
        self.write_cpbm(bitmap_words)
    }
    // CPAM
}

// CPAM
impl MpamRegisters {
    fn ccap_info(&self) -> (bool /* no_cmax */, u8 /* cmax_wd */) {
        let mpamf_ccap_idr = &self.mpamf_ccap_idr;
        let no_cmax = mpamf_ccap_idr.is_set(MPAMF_CCAP_IDR::NO_CMAX);
        let cmax_wd = mpamf_ccap_idr.read(MPAMF_CCAP_IDR::CMAX_WD) as u8;
        (no_cmax, cmax_wd)
    }
    fn write_cmax_raw(&self, value: u32) -> MpamResult<()> {
        let (no_cmax, cmax_wd) = self.ccap_info();
        if no_cmax {
            return Err(MpamError::Unsupported);
        }
        if cmax_wd == 0 || cmax_wd > 16 {
            return Err(MpamError::InvalidValue);
        }
        let mask = (1u32 << cmax_wd) - 1;
        self.mpamcfg_cmax.set(value & mask);
        Ok(())
    }
    pub fn set_part_cmax_ratio(
        &self,
        partid: u16,
        ris: u8,
        internal: bool,
        ratio: f64,
    ) -> MpamResult<()> {
        let value = self.ratio_to_cmax_value(ratio)?;
        self.select_partid_ris(partid, ris, internal);
        self.write_cmax_raw(value)
    }
    /// convert 0.0 ~ 1.0 ratio to cmax fix-point fration format
    fn ratio_to_cmax_value(&self, ratio: f64) -> MpamResult<u32> {
        let (_, cmax_wd) = self.ccap_info();
        if cmax_wd == 0 || cmax_wd > 16 {
            return Err(MpamError::InvalidValue);
        }
        if ratio < 0.0 || ratio > 1.0 {
            return Err(MpamError::InvalidValue);
        }
        let max_value = (1u32 << cmax_wd) - 1;
        let value = simple_round(ratio * (max_value as f64)) as u32;
        Ok(value)
    }
}
