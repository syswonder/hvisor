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
