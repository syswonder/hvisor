// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//      Yulong Han <wheatfox17@icloud.com>
//
use bit_field::BitField;

impl_define_csr!(Gcfg, "GCFG");
impl_read_csr!(0x51, Gcfg);

impl Gcfg {
    pub fn matp(&self) -> usize {
        self.bits.get_bits(0..=3)
    }
    pub fn matc(&self) -> usize {
        self.bits.get_bits(4..=5)
    }
    pub fn topip(&self) -> bool {
        self.bits.get_bit(6)
    }
    pub fn topi(&self) -> bool {
        self.bits.get_bit(7)
    }
    pub fn totip(&self) -> bool {
        self.bits.get_bit(8)
    }
    pub fn toti(&self) -> bool {
        self.bits.get_bit(9)
    }
    pub fn toep(&self) -> bool {
        self.bits.get_bit(10)
    }
    pub fn toe(&self) -> bool {
        self.bits.get_bit(11)
    }
    pub fn topp(&self) -> bool {
        self.bits.get_bit(12)
    }
    pub fn top(&self) -> bool {
        self.bits.get_bit(13)
    }
    pub fn tohup(&self) -> bool {
        self.bits.get_bit(14)
    }
    pub fn tohu(&self) -> bool {
        self.bits.get_bit(15)
    }
    pub fn tocip(&self) -> usize {
        self.bits.get_bits(16..=19)
    }
    pub fn toci(&self) -> usize {
        self.bits.get_bits(20..=21)
    }
    pub fn gpmp(&self) -> bool {
        self.bits.get_bit(23)
    }
    pub fn gpm_num(&self) -> usize {
        self.bits.get_bits(24..=26)
    }
}

pub fn set_matc(matc: usize) {
    set_csr_loong_bits!(0x51, 4..=5, matc);
}

pub fn set_topi(topi: bool) {
    set_csr_loong_bit!(0x51, 7, topi);
}

pub fn set_toti(toti: bool) {
    set_csr_loong_bit!(0x51, 9, toti);
}

pub fn set_toe(toe: bool) {
    set_csr_loong_bit!(0x51, 11, toe);
}

pub fn set_top(top: bool) {
    set_csr_loong_bit!(0x51, 13, top);
}

pub fn set_tohu(tohu: bool) {
    set_csr_loong_bit!(0x51, 15, tohu);
}

pub fn set_toci(toci: usize) {
    set_csr_loong_bits!(0x51, 20..=21, toci);
}

pub fn set_gpm_num(gpm_num: usize) {
    set_csr_loong_bits!(0x51, 24..=26, gpm_num);
}
