// File:          gtlbc.rs
// Description:   this is the register definition of GTLBC
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-20
use bit_field::BitField;

impl_define_csr!(
  Gtlbc,
  "Guest TLB Control Register (GTLBC)
  this register is used to control the TLB of the guest machine when running LVZ extension.
  "
);
impl_read_csr!(0x15, Gtlbc);

impl Gtlbc {
  /// RW, guest mode MTLB entry count
  pub fn gmtlb_num(&self) -> usize {
    self.bits.get_bits(0..=5)
  }
  /// RW, TGID function enable
  /// if set to 1
  /// host mode tlb instruction will use GTLBC.TGID as GID
  /// if set to 0
  /// if PGM is 1, then host mode tlb instruction will use GSTAT.GID as GID
  /// if PGM is 0, then host mode tlb instruction will use 0 as GID
  pub fn use_tgid(&self) -> bool {
    self.bits.get_bit(12)
  }
  /// RW, trap on TLB invalid
  /// if set to 1
  /// in guest mode when running tlb instruction
  /// or access tlb related CSR
  /// will throw GSPR exception
  pub fn totlbinv(&self) -> bool {
    self.bits.get_bit(13)
  }
  /// if use_tgid is 0, RO
  /// if use_tgid is 1, RW
  pub fn tgid(&self) -> usize {
    self.bits.get_bits(16..=23)
  }
}

pub fn set_gmtlb_num(gmtlb_num: usize) {
  set_csr_loong_bits!(0x15, 0..=5, gmtlb_num);
}

pub fn set_use_tgid(use_tgid: bool) {
  set_csr_loong_bit!(0x15, 12, use_tgid);
}

pub fn set_totlbinv(totlbinv: bool) {
  set_csr_loong_bit!(0x15, 13, totlbinv);
}

pub fn set_tgid(tgid: usize) {
  set_csr_loong_bits!(0x15, 16..=23, tgid);
}
