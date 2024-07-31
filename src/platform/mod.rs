use crate::{
    config::{
        HvConfigMemoryRegion, HvZoneConfig, CONFIG_MAX_INTERRUPTS, CONFIG_MAX_MEMORY_REGIONS,
    },
    consts::INVALID_ADDRESS,
};

#[cfg(all(feature = "platform_qemu", target_arch = "riscv64"))]
pub mod qemu_riscv64;

#[cfg(all(feature = "platform_qemu", target_arch = "riscv64"))]
use qemu_riscv64::*;

#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
pub mod qemu_aarch64;
#[cfg(target_arch = "aarch64")]
pub use qemu_aarch64::ROOT_ZONE_DTB_ADDR;
#[cfg(target_arch = "aarch64")]
pub use qemu_aarch64::ROOT_ENTRY;

#[cfg(target_arch = "loongarch64")]
pub mod ls3a5000_loongarch64;
#[cfg(target_arch = "loongarch64")]
pub use ls3a5000_loongarch64::*;