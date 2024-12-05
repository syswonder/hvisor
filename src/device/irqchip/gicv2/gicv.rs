use spin::Once;
use crate::device::irqchip::gicv2::device_refs::DeviceRef;
use crate::device::irqchip::gicv2::gicc::GicCpuInterface;
use crate::device::irqchip::gicv2::GICV2;

// Each CPU holds one GICV, and it has the same register layout as GICC.
pub static GICV: DeviceRef<GicCpuInterface> = unsafe { DeviceRef::new(GICV2.gicv_base as *const GicCpuInterface) };