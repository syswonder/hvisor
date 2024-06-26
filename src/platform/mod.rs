#[cfg(all(feature = "platform_qemu", target_arch = "riscv64"))]
pub mod qemu_riscv64;

#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
pub mod qemu_aarch64;

#[cfg(all(feature = "platform_imx8mp", target_arch = "aarch64"))]
pub mod imx8mp_aarch64;