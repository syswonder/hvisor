#[cfg(target_arch = "aarch64")]
pub mod gicv3;

#[cfg(target_arch = "riscv64")]
pub mod plic;

#[cfg(target_arch = "aarch64")]
pub use gicv3::{irqchip_init_early, irqchip_init_late};

#[cfg(target_arch = "riscv64")]
pub use plic::{irqchip_init_early, irqchip_init_late};