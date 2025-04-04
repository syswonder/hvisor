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
#[cfg(test)]
use crate::tests::*;

use core::panic::PanicInfo;

#[panic_handler]

fn on_panic(info: &PanicInfo) -> ! {
    error!("panic occurred: {:#?}", info);
    #[cfg(test)]
    {
        error!("panic occurred when running cargo test, quitting qemu");
        crate::tests::quit_qemu(HvUnitTestResult::Failed);
    }
    loop {}
}
