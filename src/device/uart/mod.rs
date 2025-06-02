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
// we specified the driver for each platform in the FEATURES environment variable.

#[cfg(all(feature = "pl011", target_arch = "aarch64"))]
mod pl011;
#[cfg(all(feature = "pl011", target_arch = "aarch64"))]
pub use pl011::{console_getchar, console_putchar};

#[cfg(all(feature = "imx_uart", target_arch = "aarch64"))]
mod imx_uart;
#[cfg(all(feature = "imx_uart", target_arch = "aarch64"))]
pub use imx_uart::{console_getchar, console_putchar};

#[cfg(all(feature = "ok62xx_uart", target_arch = "aarch64"))]
mod ok62xx_uart;
#[cfg(all(feature = "ok62xx_uart", target_arch = "aarch64"))]
pub use ok62xx_uart::{console_getchar, console_putchar};


#[cfg(all(feature = "xuartps", target_arch = "aarch64"))]
mod xuartps;
#[cfg(all(feature = "xuartps", target_arch = "aarch64"))]
pub use xuartps::{console_getchar, console_putchar};

#[cfg(target_arch = "riscv64")]
pub use crate::arch::riscv64::sbi::{
    sbi_console_getchar as console_getchar, sbi_console_putchar as console_putchar,
};

#[cfg(all(feature = "loongson_uart", target_arch = "loongarch64"))]
mod loongson_uart;
#[cfg(all(feature = "loongson_uart", target_arch = "loongarch64"))]
pub use loongson_uart::{console_getchar, console_putchar};

#[cfg(all(feature = "uart_16550", target_arch = "aarch64"))]
mod uart_16550;
#[cfg(all(feature = "uart_16550", target_arch = "aarch64"))]
pub use uart_16550::{console_getchar, console_putchar};
