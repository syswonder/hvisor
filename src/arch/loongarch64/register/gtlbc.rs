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

impl_define_csr!(Gtlbc, "GTLBC");
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
