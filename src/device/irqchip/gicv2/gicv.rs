#![allow(unused_variables)]
#![allow(dead_code)]
/// gicv layout definition and functions for gicv operations.
/// author : ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
/// note: We don't actually use it. This is to ensure the integrity of GICV2.

use crate::device::irqchip::gicv2::gic_ref::GicRef;
use crate::device::irqchip::gicv2::gicc::GicCpuInterface;
use crate::device::irqchip::gicv2::GICV2;

// Each CPU holds one GICV, and it has the same register layout as GICC.
pub static GICV: GicRef<GicCpuInterface> = unsafe { GicRef::new(GICV2.gicv_base as *const GicCpuInterface) };