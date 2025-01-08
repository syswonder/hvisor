/// The outer layer is defined using gicv2.
/// author: ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf

use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICD;
use crate::platform::ROOT_ARCH_ZONE_CONFIG;
use crate::device::irqchip::gicv2::gic::MAX_CPU_NUM;
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
    /*
     * Some boards have the GIC CPU interface registers alias, which will overlap DIR register, so we need to add an offset to
     * find the last gic cpu alias region.
     * ref: https://github.com/Xilinx/qemu-devicetrees/commit/09d4c3200538dc90082fbda9289e2af9794b9a28
     */
    gicc_base: ROOT_ARCH_ZONE_CONFIG.gicc_base + ROOT_ARCH_ZONE_CONFIG.gicc_offset,
    gich_base: ROOT_ARCH_ZONE_CONFIG.gich_base,
    gicv_base: ROOT_ARCH_ZONE_CONFIG.gicv_base,
};

// get base address of GIC and initialize GIC Structs.
pub fn primary_init_early() {
    info!("GicDistributer = {:#x?}", GICV2.gicd_base);
    info!("GicCpuInterface = {:#x?}", GICV2.gicc_base);
    info!("GicHypervisorInterface = {:#x?}", GICV2.gich_base);
    info!("GicVCpuInterface = {:#x?}", GICV2.gicv_base);
    gic::PENDING_VIRQS.call_once(|| gic::PendingIrqs::new(MAX_CPU_NUM));
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