use crate::{
    error::HvResult,
    memory::{
        addr::align_up, mmio_generic_handler, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion,
    },
    zone::Zone,
};
use alloc::vec::Vec;
use core::sync::atomic::{fence, Ordering};

impl Zone {
    pub fn pt_init(
        &mut self,
        vm_paddr_start: usize,
        fdt: &fdt::Fdt,
        guest_dtb: usize,
        dtb_ipa: usize,
    ) -> HvResult {
        info!("loongarch64: mm: pt_init");
        Ok(())
    }

    pub fn mmio_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: mmio_init do nothing");
    }
    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: isa_init do nothing");
    }
    pub fn irq_bitmap_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: irq_bitmap_init do nothing");
    }
}
