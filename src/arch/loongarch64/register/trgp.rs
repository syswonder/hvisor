use bit_field::BitField;

impl_define_csr!(
  Trgp,
  "TRGP"
);
impl_read_csr!(0x16, Trgp);

impl Trgp {
  pub fn gtep(&self) -> bool {
    self.bits.get_bit(0)
  }
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
