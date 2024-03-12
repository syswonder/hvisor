#[cfg(target_arch = "aarch64")]
mod pl011;

#[cfg(target_arch = "aarch64")]
pub use pl011::{console_putchar, console_getchar};

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv64::sbi::{console_putchar, console_getchar};