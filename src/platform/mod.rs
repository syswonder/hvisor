#[cfg(target_arch = "riscv64")]
pub mod qemu_riscv64;
#[cfg(target_arch = "riscv64")]
pub use qemu_riscv64::ROOT_ZONE_DTB_ADDR;

#[cfg(target_arch = "aarch64")]
pub mod qemu_aarch64;
#[cfg(target_arch = "aarch64")]
pub use qemu_aarch64::ROOT_ZONE_DTB_ADDR;
