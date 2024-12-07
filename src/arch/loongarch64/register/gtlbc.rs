use bit_field::BitField;

impl_define_csr!(
  Gtlbc,
  "GTLBC"
);
impl_read_csr!(0x15, Gtlbc);

impl Gtlbc {
  pub fn gmtlb_num(&self) -> usize {
    self.bits.get_bits(0..=5)
  }
  pub fn use_tgid(&self) -> bool {
    self.bits.get_bit(12)
  }
  pub fn totlbinv(&self) -> bool {
    self.bits.get_bit(13)
  }
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
