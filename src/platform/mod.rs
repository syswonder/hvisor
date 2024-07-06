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

#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
use qemu_aarch64::*;

#[cfg(all(feature = "platform_imx8mp", target_arch = "aarch64"))]
pub mod imx8mp_aarch64;

#[cfg(all(feature = "platform_imx8mp", target_arch = "aarch64"))]
use imx8mp_aarch64::*;

pub fn platform_root_zone_config() -> HvZoneConfig {
    // fill zero for memory regions and interrupts

    let mut memory_regions = [HvConfigMemoryRegion {
        mem_type: 0,
        physical_start: 0,
        virtual_start: 0,
        size: 0,
    }; CONFIG_MAX_MEMORY_REGIONS];

    memory_regions[..ROOT_ZONE_MEMORY_REGIONS.len()].copy_from_slice(&ROOT_ZONE_MEMORY_REGIONS);

    let mut interrupts = [0; CONFIG_MAX_INTERRUPTS];
    interrupts[..ROOT_ZONE_IRQS.len()].copy_from_slice(&ROOT_ZONE_IRQS);

    HvZoneConfig::new(
        0,
        ROOT_ZONE_CPUS,
        ROOT_ZONE_MEMORY_REGIONS.len() as u32,
        memory_regions,
        ROOT_ZONE_IRQS.len() as u32,
        interrupts,
        ROOT_ZONE_ENTRY,
        ROOT_ZONE_KERNEL_ADDR,
        INVALID_ADDRESS as _,
        ROOT_ZONE_DTB_ADDR,
        INVALID_ADDRESS as _,
        ROOT_ARCH_ZONE_CONFIG,
    )
}
