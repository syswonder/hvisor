use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICD;
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

// GIC Reference warp.
mod gic_ref;

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

// get base address of GIC and initialize GIC Structs.
pub fn primary_init_early() {
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
        panic!("todo: arch_irqchip_reset")
    }
}