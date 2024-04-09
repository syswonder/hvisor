#[cfg(target_arch = "aarch64")]
pub mod gicv3;

#[cfg(target_arch = "riscv64")]
pub mod plic;

#[cfg(target_arch = "aarch64")]
pub use gicv3::{primary_init_early, primary_init_late, percpu_init};

#[cfg(target_arch = "riscv64")]
pub use plic::{init_early, init_late, irqchip_cpu_init, per_cpu_init};
