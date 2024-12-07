use bit_field::BitField;

impl_define_csr!(
  Gintc,
  "GINTC"
);
impl_read_csr!(0x52, Gintc);

impl Gintc {
  pub fn hwis(&self) -> usize {
    self.bits.get_bits(0..=7)
  }
  pub fn hwip(&self) -> usize {
    self.bits.get_bits(8..=15)
  }
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
