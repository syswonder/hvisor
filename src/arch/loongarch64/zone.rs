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
        todo!()
    }

    pub fn mmio_init(&mut self, fdt: &fdt::Fdt) {
        todo!()
    }
    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        todo!()
    }
    pub fn irq_bitmap_init(&mut self, fdt: &fdt::Fdt) {
        todo!()
    }
}
