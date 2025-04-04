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

impl_define_csr!(Gcntc, "GCNTC");
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
