// File:          gstat.rs
// Description:   this is the register definition of GSTAT
// Authors:       wheatfox(enkerewpo@hotmail.com)
// Created:       2023-12-20
use bit_field::BitField;

impl_define_csr!(
  Gstat,
  "Guest Status Register (GSTAT)
  this register is used to indicate the status of the guest machine when running LVZ extension.
  "
);
impl_read_csr!(0x50, Gstat);

impl Gstat {
  /// RW, when CPU changed from guest mode to host mode
  /// this bit will be set to 1
  /// when CSR.MERRCTL.IsMERR and CSR.TLBRPRMD.IsTLBR are both 0,
  /// and this bit is 1, then CPU will enter guest mode after running "ertn"
  pub fn pgm(&self) -> bool {
    self.bits.get_bit(1)
  }
  /// RO, the number of bits of the guest machine id
  pub fn gidbits(&self) -> usize {
    self.bits.get_bits(4..=9)
  }
  /// RW, current guest machine id
  pub fn gid(&self) -> usize {
    self.bits.get_bits(16..=23)
  }
}

pub fn set_gid(gid: usize) {
  set_csr_loong_bits!(0x50, 16..=23, gid);
}

pub fn set_pgm(pgm: bool) {
  set_csr_loong_bit!(0x50, 1, pgm);
}
