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
