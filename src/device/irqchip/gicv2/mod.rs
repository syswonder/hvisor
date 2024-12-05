use crate::config::root_zone_config;
use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::{GicDistributer, GICD};
use crate::device::irqchip::gicv2::gich::{GicHypervisorInterface, GICH};
use crate::device::irqchip::gicv2::gicv::GICV;
use crate::platform::ROOT_ARCH_ZONE_CONFIG;
use crate::zone::Zone;

// GIC Distributor Definition.
pub mod gicd;

// GIC CPU Interface Definition.
pub mod gicc;

// Virtual GIC realization.
pub mod vgic;

// Physical GIC realization.
pub mod gic;

// GIC Hypervisor Interface Definition.
pub mod gich;

// GIC Virtual CPU Interface Definition.
pub mod gicv;
mod device_refs;

pub struct Gicv2 {
    gicd_base: usize,
    gicc_base: usize,
    gich_base: usize,
    gicv_base: usize,
}

pub static GICV2: Gicv2 = Gicv2 {
    gicd_base: ROOT_ARCH_ZONE_CONFIG.gicd_base,
    gicc_base: ROOT_ARCH_ZONE_CONFIG.gicc_base,
    gich_base: ROOT_ARCH_ZONE_CONFIG.gich_base,
    gicv_base: ROOT_ARCH_ZONE_CONFIG.gicv_base,
};
// pub static GICV2: Once<Gicv2> = Once::new();

// get base address of GIC and initialize GIC Structs.
pub fn primary_init_early() {
    let root_config = root_zone_config();
    // GICV2.call_once(|| {
    //     Gicv2 {
    //         gicd_base: root_config.arch_config.gicd_base,
    //         gicc_base: root_config.arch_config.gicc_base,
    //         gich_base: root_config.arch_config.gich_base,
    //         gicv_base: root_config.arch_config.gicv_base,
    //     }
    // });
    // GICD.call_once(|| {
    //     unsafe { &*root_config.arch_config.gicd_base as *const GicDistributer }
    // });
    // GICH.call_once(|| {
    //     unsafe { &*root_config.arch_config.gich_base as *const GicHypervisorInterface }
    // });
    // GICC.call_once(|| {
    //     unsafe { &*root_config.arch_config.gicc_base as *const GicCpuInterface }
    // });
    // GICV.call_once(|| {
    //     unsafe { &*root_config.arch_config.gicv_base as *const GicCpuInterface }
    // });
    info!("GicDistributer = {:#x?}", GICV2.gicd_base);
    info!("GicCpuInterface = {:#x?}", GICV2.gicc_base);
    info!("GicHypervisorInterface = {:#x?}", GICV2.gich_base);
    info!("GicVCpuInterface = {:#x?}", GICV2.gicv_base);
}

pub fn percpu_init() {
    GICC.init();
}

pub fn primary_init_late() {
    GICD.global_init();
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        // todo
        info!("todo: arch_irqchip_reset")
    }
}