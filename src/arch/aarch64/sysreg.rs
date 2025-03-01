#![allow(unused_imports)]

// Copyright 2023 The arm-gic Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

/// Reads and returns the value of the given aarch64 system register.
/// use crate::arch::sysreg::write_sysreg;
/// unsafe {write_sysreg!(icc_sgi1r_el1, val);}
/// let intid = unsafe { read_sysreg!(icc_iar1_el1) } as u32;
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

macro_rules! smc_call {
    ($x0:expr, $x1:expr, $x2:expr, $x3:expr) => {{
        let mut x0_val: u64 = $x0;
        let mut x1_val: u64 = $x1;
        let mut x2_val: u64 = $x2;
        let mut x3_val: u64 = $x3;

        ::core::arch::asm!(
            "smc #0",
            inout("x0") x0_val,
            inout("x1") x1_val,
            inout("x2") x2_val,
            inout("x3") x3_val,
            options(nomem, nostack),
        );
        (x0_val,x1_val,x2_val,x3_val)
    }};
}
pub(crate) use smc_call;

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
