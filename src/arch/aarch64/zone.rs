use core::panic;

use alloc::vec::Vec;

use crate::{
    config::*,
    device::virtio_trampoline::{mmio_virtio_handler, VIRTIO_BRIDGE},
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion},
    zone::Zone,
};

impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
        // The first memory region is used to map the guest physical memory.

        for mem_region in mem_regions.iter() {
            let mut flags = MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE;
            if mem_region.mem_type == MEM_TYPE_IO {
                flags |= MemFlags::IO;
            }
            match mem_region.mem_type {
                MEM_TYPE_RAM | MEM_TYPE_IO => {
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        mem_region.virtual_start as GuestPhysAddr,
                        mem_region.physical_start as HostPhysAddr,
                        mem_region.size as _,
                        flags,
                    ))?
                }
                MEM_TYPE_VIRTIO => {
                    self.mmio_region_register(
                        mem_region.physical_start as _,
                        mem_region.size as _,
                        mmio_virtio_handler,
                        mem_region.physical_start as _,
                    );
                }
                _ => {
                    panic!("Unsupported memory type: {}", mem_region.mem_type)
                }
            }
        }

        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    pub fn mmio_init(&mut self, hv_config: &HvArchZoneConfig) {
        self.vgicv3_mmio_init(hv_config);
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {
    pub gicd_base: usize,
    pub gicr_base: usize,
    pub gicd_size: usize,
    pub gicr_size: usize,
    pub gits_base: usize,
    pub gits_size: usize,
}
