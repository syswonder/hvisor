use aarch64_cpu::registers::{Readable, Writeable};
use tock_registers::register_bitfields;

use crate::arch::mpam::simple_round;

use super::{
    error::{MpamError, MpamResult},
    regs::MpamRegisters,
};

register_bitfields![
    u32,
    pub MPAMF_MBW_IDR [
        Reserved OFFSET(29) NUMBITS(2) [],
        BWPBM_WD OFFSET(16) NUMBITS(13) [], // bits 16-28
        WINDWR   OFFSET(14) NUMBITS(1)  [], // bit 14
        HAS_PROP OFFSET(13) NUMBITS(1)  [], // bit 13
        HAS_PBM  OFFSET(12) NUMBITS(1)  [], // bit 12
        HAS_MAX  OFFSET(11) NUMBITS(1)  [], // bit 11
        HAS_MIN  OFFSET(10) NUMBITS(1)  [], // bit 10
        MAX_LIM  OFFSET(8)  NUMBITS(2)  [], // bits 8-9
        BWA_WD   OFFSET(0)  NUMBITS(6)  []
    ]
];

register_bitfields![
    u32,
    pub MPAMCFG_MBW_PROP [
        EN      OFFSET(31) NUMBITS(1)  [], // bit 31
        Reserved OFFSET(16) NUMBITS(15) [], // bits 16-30
        STRIDEM1 OFFSET(0)  NUMBITS(16) []  // bits 0-15
    ]
];

register_bitfields![
    u32,
    pub MPAMCFG_MBW_MIN [
        Reserved OFFSET(16) NUMBITS(16) [], // bits 16-31
        MBW_MIN  OFFSET(0)  NUMBITS(16) []  // bits 0-15
    ]
];

register_bitfields![
    u32,
    pub MPAMCFG_MBW_MAX [
        HARDLIM OFFSET(31) NUMBITS(1)  [], // bit 31
        Reserved OFFSET(16) NUMBITS(15) [], // bits 16-30
        MAX      OFFSET(0)  NUMBITS(16) []  // bits 0-15
    ]
];

#[allow(unused)]
impl MpamRegisters {
    fn bwpbm_wd(&self) -> u16 {
        self.mpamf_mbw_idr.read(MPAMF_MBW_IDR::BWPBM_WD) as u16
    }
    fn write_mbwpbm(&self, bitmap_words: &[u32]) -> MpamResult<()> {
        let wd = self.bwpbm_wd();
        if wd == 0 || wd > 4096 {
            return Err(MpamError::InvalidValue);
        }
        let needed_words = ((wd as usize) + 31) / 32;
        if bitmap_words.len() < needed_words {
            return Err(MpamError::InvalidValue);
        }
        for i in 0..needed_words {
            self.mbwpbm_n[i].set(bitmap_words[i]);
        }
        Ok(())
    }
    pub fn set_part_mbwpbm(
        &self,
        partid: u16,
        ris: u8,
        internal: bool,
        bitmap_words: &[u32],
    ) -> MpamResult<()> {
        self.select_partid_ris(partid, ris, internal);
        self.write_mbwpbm(bitmap_words)
    }
}

impl MpamRegisters {
    fn mbw_info(
        &self,
    ) -> (
        bool, /* has_min */
        bool, /* has_max */
        u8,   /* bwa_wd */
    ) {
        let idr = &self.mpamf_mbw_idr;
        let has_min = idr.is_set(MPAMF_MBW_IDR::HAS_MIN);
        let has_max = idr.is_set(MPAMF_MBW_IDR::HAS_MAX);
        let bwa_wd = idr.read(MPAMF_MBW_IDR::BWA_WD) as u8;
        (has_min, has_max, bwa_wd)
    }
    fn write_mbw_min_raw(&self, value: u32) -> MpamResult<()> {
        let (has_min, _, bwa_wd) = self.mbw_info();
        if !has_min {
            return Err(MpamError::Unsupported);
        }
        if bwa_wd == 0 || bwa_wd > 16 {
            return Err(MpamError::InvalidValue);
        }
        let mask = (1u32 << bwa_wd) - 1;
        self.mpamcfg_mbw_min
            .write(MPAMCFG_MBW_MIN::MBW_MIN.val(value & mask));
        Ok(())
    }

    fn write_mbw_max_raw(&self, value: u32, hard_limit: bool) -> MpamResult<()> {
        let (_, has_max, bwa_wd) = self.mbw_info();
        if !has_max {
            return Err(MpamError::Unsupported);
        }
        if bwa_wd == 0 || bwa_wd > 16 {
            return Err(MpamError::InvalidValue);
        }
        let mask = (1u32 << bwa_wd) - 1;
        self.mpamcfg_mbw_max.write(
            MPAMCFG_MBW_MAX::MAX.val(value & mask)
                + MPAMCFG_MBW_MAX::HARDLIM.val(hard_limit as u32),
        );
        Ok(())
    }
    fn ratio_to_mbw_value(&self, ratio: f64) -> MpamResult<u32> {
        let (_, _, bwa_wd) = self.mbw_info();
        if bwa_wd == 0 || bwa_wd > 16 {
            return Err(MpamError::InvalidValue);
        }
        if !(0.0..=1.0).contains(&ratio) {
            return Err(MpamError::InvalidValue);
        }
        let max_value = (1u32 << bwa_wd) - 1;
        let value = simple_round(ratio * (max_value as f64)) as u32;
        Ok(value)
    }
    pub fn set_part_mbw_min_ratio(
        &self,
        partid: u16,
        ris: u8,
        internal: bool,
        ratio: f64,
    ) -> MpamResult<()> {
        let value = self.ratio_to_mbw_value(ratio)?;
        self.select_partid_ris(partid, ris, internal);
        self.write_mbw_min_raw(value)
    }
    pub fn set_part_mbw_max_ratio(
        &self,
        partid: u16,
        ris: u8,
        internal: bool,
        ratio: f64,
        hard_limit: bool,
    ) -> MpamResult<()> {
        let value = self.ratio_to_mbw_value(ratio)?;
        self.select_partid_ris(partid, ris, internal);
        self.write_mbw_max_raw(value, hard_limit)
    }
}
