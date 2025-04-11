use crate::{
    config::*,
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion},
    platform::MEM_TYPE_OTHER_ZONES,
    zone::Zone,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {
    pub ioapic_base: usize,
    pub ioapic_size: usize,
    pub kernel_entry_gpa: usize,
    pub cmdline_load_gpa: usize,
    pub setup_load_gpa: usize,
    pub initrd_load_gpa: usize,
    pub initrd_size: usize,
    pub rsdp_memory_region_id: usize,
    pub acpi_memory_region_id: usize,
    pub initrd_memory_region_id: usize,
}

impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
        for mem_region in mem_regions.iter() {
            let mut flags = MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE;
            if mem_region.mem_type == MEM_TYPE_IO {
                flags |= MemFlags::IO;
            }
            match mem_region.mem_type {
                MEM_TYPE_RAM | MEM_TYPE_IO | MEM_TYPE_OTHER_ZONES => {
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        mem_region.virtual_start as GuestPhysAddr,
                        mem_region.physical_start as HostPhysAddr,
                        mem_region.size as _,
                        flags,
                    ))?
                }
                /*
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

        // info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {}

    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {}
}
