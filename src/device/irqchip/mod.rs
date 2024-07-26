#[cfg(target_arch = "aarch64")]
pub mod gicv3;

#[cfg(target_arch = "riscv64")]
#[cfg(feature = "plic")]
pub mod plic;

#[cfg(target_arch = "riscv64")]
#[cfg(feature = "aia")]
pub mod aia;

#[cfg(target_arch = "aarch64")]
pub use gicv3::{inject_irq, percpu_init, primary_init_early, primary_init_late};

#[cfg(target_arch = "riscv64")]
#[cfg(feature = "plic")]
pub use plic::{inject_irq, percpu_init, primary_init_early, primary_init_late, 
    host_plic, vplic_global_emul_handler, vplic_hart_emul_handler};

#[cfg(target_arch = "riscv64")]
#[cfg(feature = "aia")]
pub use aia::aplic::{inject_irq, percpu_init, primary_init_early, primary_init_late, 
    host_aplic, vaplic_emul_handler};
