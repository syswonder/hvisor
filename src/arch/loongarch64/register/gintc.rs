// File:          gintc.rs
// Description:   this is the register definition of GINTC
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-25
use bit_field::BitField;

impl_define_csr!(
  Gintc,
  "Guest Interrupt Control Register (GINTC)
  this register is used to compensate the guest counter
  "
);
impl_read_csr!(0x52, Gintc);

impl Gintc {
  /// RW, for host to inject interrupt to guest
  /// guest will see 8 hardware interrupt sources
  pub fn hwis(&self) -> usize {
    self.bits.get_bits(0..=7)
  }
  /// RW, for host to directly route 8 hardware interrupt sources to guest
  /// hight bit means ENABLE
  pub fn hwip(&self) -> usize {
    self.bits.get_bits(8..=15)
  }
  /// RW, for host to directly clear 8 hardware interrupt sources to guest
  /// high bit means CLEAR
  pub fn hwic(&self) -> usize {
    self.bits.get_bits(16..=23)
  }
}

pub fn set_hwis(hwis: usize) {
  set_csr_loong_bits!(0x52, 0..=7, hwis);
}

pub fn set_hwip(hwip: usize) {
  set_csr_loong_bits!(0x52, 8..=15, hwip);
}

pub fn set_hwic(hwic: usize) {
  set_csr_loong_bits!(0x52, 16..=23, hwic);
}
