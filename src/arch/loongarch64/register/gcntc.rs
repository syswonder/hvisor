use bit_field::BitField;

impl_define_csr!(
  Gcntc,
  "GCNTC"
);
impl_read_csr!(0x53, Gcntc);

const GRLEN: usize = 64; // not sure what is GRLEN, set to 64 for now...

impl Gcntc {
  pub fn compensation(&self) -> usize {
    self.bits.get_bits(0..=GRLEN - 1)
  }
}

pub fn set_compensation(value: usize) {
  set_csr_loong_bits!(0x53, 0..=GRLEN - 1, value);
}