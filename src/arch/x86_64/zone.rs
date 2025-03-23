use crate::{
    arch::mmio::mmio_handler,
    config::*,
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion},
    platform::qemu_x86_64::MEM_TYPE_ROM,
    zone::Zone,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {
    pub ioapic_base: usize,
    pub ioapic_size: usize,
}

impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
        // The first memory region is used to map the guest physical memory.

        for mem_region in mem_regions.iter() {
            let mut flags = MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE;
            if mem_region.mem_type == MEM_TYPE_IO {
                flags |= MemFlags::IO;
            } else if mem_region.mem_type == MEM_TYPE_ROM {
                flags &= !MemFlags::WRITE;
            }
            match mem_region.mem_type {
                MEM_TYPE_RAM | MEM_TYPE_ROM | MEM_TYPE_IO => {
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        mem_region.virtual_start as GuestPhysAddr,
                        mem_region.physical_start as HostPhysAddr,
                        mem_region.size as _,
                        flags,
                    ))?
                }
                /*MEM_TYPE_IO => {
                    self.mmio_region_register(
                        mem_region.physical_start as _,
                        mem_region.size as _,
                        mmio_handler,
                        mem_region.physical_start as _,
                    );
                }
                MEM_TYPE_VIRTIO => {
                    self.mmio_region_register(
                        mem_region.physical_start as _,
                        mem_region.size as _,
                        mmio_virtio_handler,
                        mem_region.physical_start as _,
                    );
                }*/
                _ => {
                    panic!("Unsupported memory type: {}", mem_region.mem_type)
                }
            }
        }

        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    pub fn mmio_init(&mut self, hv_config: &HvArchZoneConfig) {
        self.ioapic_mmio_init(hv_config);
    }

    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {}

    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {}
}
