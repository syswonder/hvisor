use crate::{
    config::*,
    error::HvResult,
    memory::{
        addr::align_down, addr::align_up, mmio_generic_handler, GuestPhysAddr, HostPhysAddr,
        MemFlags, MemoryRegion,
    },
    zone::Zone,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {}

impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
        Ok(())
    }

    pub fn mmio_init(&mut self, hv_config: &HvArchZoneConfig) {}

    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {}

    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {}
}
