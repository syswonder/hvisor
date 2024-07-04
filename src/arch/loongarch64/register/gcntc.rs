// File:          gcntc.rs
// Description:   this is the register definition of GCNTC
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-25
use bit_field::BitField;

impl_define_csr!(
  Gcntc,
  "Guest Counter Compensate Register (GCNTC)
  this register is used to compensate the guest counter
  "
);
impl_read_csr!(0x53, Gcntc);

const GRLEN: usize = 64; // not sure what is GRLEN, set to 64 for now...

impl Gcntc {
  /// RW, the counter compensate value
  /// [GRLEN-1:0]
  pub fn compensation(&self) -> usize {
    self.bits.get_bits(0..=GRLEN - 1)
  }
}

pub fn set_compensation(value: usize) {
  set_csr_loong_bits!(0x53, 0..=GRLEN - 1, value);
}