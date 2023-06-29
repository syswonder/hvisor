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
            ::core::arch::asm!(
                concat!("mrs {value:x}, ", ::core::stringify!($name)),
                value = out(reg) value,
                options(nomem, nostack),
            );
            value
        }
    }
}
pub(crate) use read_sysreg;

/// Writes the given value to the given aarch64 system register.
macro_rules! write_sysreg {
    ($name:ident, $value:expr) => {
        {
            let v: u64 = $value;
            ::core::arch::asm!(
                concat!("msr ", ::core::stringify!($name), ", {value:x}"),
                value = in(reg) v,
                options(nomem, nostack),
            )
        }
    }
}
pub(crate) use write_sysreg;
