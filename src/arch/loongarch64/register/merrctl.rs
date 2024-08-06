use bit_field::BitField;
impl_define_csr!(MerrCtl, "Machine Error Controller\n\
                           Since the timing of machine error exceptions cannot be predicted and controlled by the software,\n\
                           a separate set of CSRs is defined for machine error exceptions to preserve other registers when machine error exceptions are triggered,\n\
                           which is used by the system software to save and restore other sites.\n\
                           This set of independent CSRs except MERRERA and MERRSAVE, the rest are concentrated in MERRCTL register.");

impl_read_csr!(0x90, MerrCtl);

impl MerrCtl {
    /// Returns whether the machine error exception is triggered.
    pub fn is_merr(&self) -> bool {
        self.bits.get_bit(0)
    }
    /// Returns whether the machine error exception is repairable.
    ///
    /// If the machine error exception is repairable, the system software can ignore the error and continue to execute the program;
    pub fn repairable(&self) -> bool {
        self.bits.get_bit(1)
    }
    pub fn pplv(&self) -> usize {
        self.bits.get_bits(2..=3)
    }

    pub fn pie(&self) -> bool {
        self.bits.get_bit(4)
    }
    pub fn pwe(&self) -> bool {
        self.bits.get_bit(6)
    }

    pub fn pda(&self) -> bool {
        self.bits.get_bit(7)
    }

    pub fn ppg(&self) -> bool {
        self.bits.get_bit(8)
    }

    pub fn pdatf(&self) -> usize {
        self.bits.get_bits(9..=10)
    }

    pub fn pdatm(&self) -> usize {
        self.bits.get_bits(11..=12)
    }

    pub fn cause(&self) -> MachineError {
        self.bits.get_bits(13..=15).into()
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(usize)]
pub enum MachineError {
    CacheCheckError,
}

impl From<usize> for MachineError {
    fn from(code: usize) -> Self {
        match code {
            0 => MachineError::CacheCheckError,
            _ => panic!("Unknown MachineError code: {}", code),
        }
    }
}

pub fn set_pplv(pplv: usize) {
    set_csr_loong_bits!(0x90, 2..=3, pplv);
}

pub fn set_pie(pie: bool) {
    set_csr_loong_bit!(0x90, 4, pie);
}

pub fn set_pwe(pwe: bool) {
    set_csr_loong_bit!(0x90, 6, pwe);
}

pub fn set_pda(pda: bool) {
    set_csr_loong_bit!(0x90, 7, pda);
}

pub fn set_ppg(ppg: bool) {
    set_csr_loong_bit!(0x90, 8, ppg);
}

pub fn set_pdatf(pdatf: usize) {
    set_csr_loong_bits!(0x90, 9..=10, pdatf);
}

pub fn set_pdatm(pdatm: usize) {
    set_csr_loong_bits!(0x90, 11..=12, pdatm);
}
