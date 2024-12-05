use crate::device::irqchip::gicv2::gic_ref::GicRef;
use crate::device::irqchip::gicv2::gicc::GicCpuInterface;
use crate::device::irqchip::gicv2::GICV2;

// Each CPU holds one GICV, and it has the same register layout as GICC.
pub static GICV: GicRef<GicCpuInterface> = unsafe { GicRef::new(GICV2.gicv_base as *const GicCpuInterface) };