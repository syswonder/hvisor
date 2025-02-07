#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
mod pl011;

#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
pub use pl011::{console_getchar, console_putchar};

#[cfg(all(feature = "platform_imx8mp", target_arch = "aarch64"))]
mod imx_uart;
#[cfg(all(feature = "platform_zcu102", target_arch = "aarch64"))]
mod xuartps;

#[cfg(all(feature = "platform_zcu102", target_arch = "aarch64"))]
pub use xuartps::{console_getchar, console_putchar};

#[cfg(all(feature = "platform_imx8mp", target_arch = "aarch64"))]
pub use imx_uart::{console_getchar, console_putchar};

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv64::sbi::{console_getchar, console_putchar};

#[cfg(target_arch = "loongarch64")]
mod loongson_uart;

#[cfg(target_arch = "loongarch64")]
pub use loongson_uart::{console_getchar, console_putchar};
