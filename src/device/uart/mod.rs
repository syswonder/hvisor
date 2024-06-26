#[cfg(target_arch = "aarch64")]
mod pl011;

#[cfg(target_arch = "aarch64")]
pub use pl011::{console_getchar, console_putchar};

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv64::sbi::{console_getchar, console_putchar};

#[cfg(target_arch = "loongarch64")]
mod ns16440a;

#[cfg(target_arch = "loongarch64")]
pub use ns16440a::{console_getchar, console_putchar};
