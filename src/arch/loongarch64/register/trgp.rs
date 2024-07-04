// File:          trgp.rs
// Description:   this is the register definition of TRGP
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-25
use bit_field::BitField;

impl_define_csr!(
  Trgp,
  "TLB Read Guest Page Register (TRGP)
  this register is used to show status when reading guest page entry
  "
);
impl_read_csr!(0x16, Trgp);

impl Trgp {
  /// RO, when host run "tlbrd" and read the guest page entry
  /// this bit will be set to 1
  /// otherwise will be 0
  pub fn gtep(&self) -> bool {
    self.bits.get_bit(0)
  }
  /// RO, when host run "tlbrd" and read the guest page entry
  /// the corresponding GID will be written here
  /// otherwise will be 0
  pub fn trgid(&self) -> usize {
    self.bits.get_bits(16..=23)
  }
}

pub fn set_gtep(gtep: bool) {
  set_csr_loong_bit!(0x16, 0, gtep);
}

pub fn set_trgid(trgid: usize) {
  set_csr_loong_bits!(0x16, 16..=23, trgid);
}
