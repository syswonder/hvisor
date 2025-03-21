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
