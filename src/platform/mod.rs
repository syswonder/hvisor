#[cfg(target_arch = "riscv64")]
pub mod qemu_riscv64;

#[cfg(target_arch = "aarch64")]
pub mod qemu_aarch64;
