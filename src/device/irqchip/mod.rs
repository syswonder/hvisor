#[cfg(target_arch = "aarch64")]
pub mod gicv3;

#[cfg(target_arch = "riscv64")]
pub mod plic;

#[cfg(target_arch = "loongarch64")]
pub mod ls7a2000;

#[cfg(target_arch = "x86_64")]
pub mod pic;

#[cfg(target_arch = "aarch64")]
pub use gicv3::{inject_irq, percpu_init, primary_init_early, primary_init_late};

#[cfg(target_arch = "riscv64")]
pub use plic::{inject_irq, percpu_init, primary_init_early, primary_init_late};

#[cfg(target_arch = "loongarch64")]
pub use ls7a2000::{inject_irq, percpu_init, primary_init_early, primary_init_late};

#[cfg(target_arch = "x86_64")]
pub use pic::{inject_irq, percpu_init, primary_init_early, primary_init_late};
