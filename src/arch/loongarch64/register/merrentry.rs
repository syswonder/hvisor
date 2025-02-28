impl_define_csr!(MerrEntry,"Machine Error Exception Entry Base Address (MERRENTRY)\n\
                            This register is used to configure the entry base address of the machine error exception.\n\
                            Since the processor core enters the direct address translation mode once the machine error exception is triggered,\n\
                            the entry base address filled here should be the physical address.");

impl_read_csr!(0x93, MerrEntry);

impl MerrEntry {
    pub fn addr(&self) -> usize {
        self.bits
    }
}

pub fn set_merrentry(addr: usize) {
    debug_assert_eq!(addr & 0xFFF, 0);
    write_csr_loong!(0x93, addr);
}
