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
#![allow(unused)]

pub mod clock;
pub mod consts;
pub mod cpu;
pub mod eiointc;
pub mod entry;
pub mod hypercall;
pub mod ipi;
pub mod mm;
pub mod paging;
pub mod register;
pub mod s1pt;
pub mod s2pt;
pub mod time;
pub mod timer;
pub mod trap;
pub mod zone;

pub use s1pt::Stage1PageTable;
pub use s2pt::stage2_mode_detect;
pub use s2pt::Stage2PageTable;

/// Print LoongArch64 specific logo
pub fn print_logo() {
    println!(
        r"
  _            _                   _
 | |          (_)                 | |
 | |__  __   ___ ___  ___  _ __   | | __ _
 | '_ \ \ \ / / / __|/ _ \| '__|  | |/ _` |
 | | | | \ V /| \__ \ (_) | |    _| | (_| |
 |_| |_|  \_/ |_|___/\___/|_|   (_)_|\__,_|
"
    );
}
