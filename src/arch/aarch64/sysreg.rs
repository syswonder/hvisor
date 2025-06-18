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

macro_rules! read_sysreg {
    ($name:ident) => {
        {
            let mut value: u64;
            unsafe{::core::arch::asm!(
                concat!("mrs {value:x}, ", ::core::stringify!($name)),
                value = out(reg) value,
                options(nomem, nostack),
            );}
            value
        }
    }
}
use psci::smccc::smc64;
pub(crate) use read_sysreg;

/// Writes the given value to the given aarch64 system register.
macro_rules! write_sysreg {
    ($name:ident, $value:expr) => {
        {
            let v: u64 = $value;
            unsafe{::core::arch::asm!(
                concat!("msr ", ::core::stringify!($name), ", {value:x}"),
                value = in(reg) v,
                options(nomem, nostack),
            )}
        }
    }
}
pub(crate) use write_sysreg;

macro_rules! smc_arg1 {
    ($value:expr) => {{
        let mut v: i64 = $value;
        ::core::arch::asm!("smc #0", inout("x0") v, options(nomem, nostack),);
        v
    }};
}
pub(crate) use smc_arg1;

pub fn smc_call(function: u64, args: &[u64]) -> [u64; 4] {
    let args: [u64; 17] = args.try_into().expect("args length should be 17");
    smc64(function as _, args)[0..4]
        .try_into()
        .expect("smc64 ret err")
}

// macro_rules! read_lrreg {
//     ($lr:expr) => {
//         {
//             let mut value: u64;
//             ::core::arch::asm!(
//                 concat!("mrs {value:x}, ","ich_lr",::core::stringify!($id),"_el2"),
//                 value = out(reg) value,
//                 options(nomem, nostack),
//             );
//             value
//         }
//     }
// }
// pub(crate) use read_lrreg;
