impl_define_csr!(MerrEra,"Machine Error Exception Data Save Register\n\
                          This register is used to record the PC of the instruction that triggered the machine error exception.");
impl_read_csr!(0x94, MerrEra);

impl MerrEra {
    pub fn pc(&self) -> usize {
        self.bits
    }
}

pub fn set_pc(pc: usize) {
    write_csr_loong!(0x94, pc);
}
