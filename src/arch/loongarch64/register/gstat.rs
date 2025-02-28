use bit_field::BitField;

impl_define_csr!(Gstat, "GSTAT");
impl_read_csr!(0x50, Gstat);

impl Gstat {
    pub fn pgm(&self) -> bool {
        self.bits.get_bit(1)
    }
    pub fn gidbits(&self) -> usize {
        self.bits.get_bits(4..=9)
    }
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
