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
//
#[derive(Debug)]
pub struct PhantomCfg {
    pub bdf: usize,
    command: u16,
}

impl PhantomCfg {
    pub fn new(bdf: usize, command: u16) -> Self {
        Self {
            bdf,
            command: command & !0x400, // set disable-intx to 0, the origin state
        }
    }

    pub fn set_cmd(&mut self, command: u16) {
        self.command = command;
    }

    pub fn get_cmd(&self) -> u16 {
        self.command
    }
}
