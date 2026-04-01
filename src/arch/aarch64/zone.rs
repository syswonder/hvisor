// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
use core::panic;

use super::cache::invalidate_dcache_range;
use crate::{
    config::*,
    device::virtio_trampoline::mmio_virtio_handler,
    error::HvResult,
    memory::{addr::phys_to_virt, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion},
    zone::Zone,
};

impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
        let mut inner = self.write();
        // The first memory region is used to map the guest physical memory.

        for mem_region in mem_regions.iter() {
            let mut flags = MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE;
            if mem_region.mem_type == MEM_TYPE_IO {
                flags |= MemFlags::IO;
            }
            match mem_region.mem_type {
                MEM_TYPE_RAM | MEM_TYPE_IO => {
                    inner
                        .gpm_mut()
                        .insert(MemoryRegion::new_with_offset_mapper(
                            mem_region.virtual_start as GuestPhysAddr,
                            mem_region.physical_start as HostPhysAddr,
                            mem_region.size as _,
                            flags,
                        ))?
                }
                MEM_TYPE_VIRTIO => {
                    inner.mmio_region_register(
                        mem_region.physical_start as _,
                        mem_region.size as _,
                        mmio_virtio_handler,
                        mem_region.physical_start as _,
                    );
                }
                _ => {
                    // hvisor-tool should check memory type in advance.
                    if self.id() == 0 {
                        panic!("Unsupported memory type: {}", mem_region.mem_type);
                    }
                    return hv_result_err!(
                        EINVAL,
                        format!(
                            "zone {} has unsupported memory type: {}",
                            self.id(),
                            mem_region.mem_type
                        )
                    );
                }
            }
        }

        info!("VM stage 2 memory set: {:#x?}", inner.gpm());
        Ok(())
    }

    pub fn iommu_pt_init(
        &mut self,
        mem_regions: &[HvConfigMemoryRegion],
        hv_config: &HvArchZoneConfig,
    ) -> HvResult {
        let mut inner = self.write();
        // Create a new stage 2 page table for iommu.
        // Only map the memory regions that are possible to be accessed by devices as DMA buffer.

        let pt = inner.iommu_pt_mut().unwrap();
        let flags = MemFlags::READ | MemFlags::WRITE;
        for mem_region in mem_regions.iter() {
            match mem_region.mem_type {
                MEM_TYPE_RAM => {
                    pt.insert(MemoryRegion::new_with_offset_mapper(
                        mem_region.virtual_start as GuestPhysAddr,
                        mem_region.physical_start as HostPhysAddr,
                        mem_region.size as _,
                        flags,
                    ))?;
                    info!(
                        "iommu map: vaddr:{} - paddr:{}",
                        mem_region.virtual_start, mem_region.physical_start
                    );
                }
                _ => {
                    // pass
                }
            }
        }

        match hv_config.gic_config {
            GicConfig::Gicv3(ref gicv3_config) => {
                if gicv3_config.gits_size != 0 {
                    // map gits
                    pt.insert(MemoryRegion::new_with_offset_mapper(
                        gicv3_config.gits_base as GuestPhysAddr,
                        gicv3_config.gits_base as HostPhysAddr,
                        gicv3_config.gits_size as _,
                        flags | MemFlags::IO,
                    ))?;
                    info!(
                        "iommu map: vaddr:{} - paddr:{}",
                        gicv3_config.gits_base, gicv3_config.gits_base
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn arch_zone_pre_configuration(&mut self, config: &HvZoneConfig) -> HvResult {
        self.ivc_init(config.ivc_config())
    }

    pub fn arch_zone_post_configuration(&mut self, _config: &HvZoneConfig) -> HvResult {
        Ok(())
    }

    pub fn arch_zone_reset(&mut self, _config: &HvZoneConfig) -> HvResult {
        // This operation serves as an insurance to ensure that
        //      there is no relevant d-cache line in the cache;
        //      since the VA has the inner-shareable attribute,
        //      the cores within the inner-shareable domain will
        //      be affected by the broadcast invalidation.
        unsafe {
            // Get cache line size from CTR_EL0[16:19] (min line size, in words of 4 bytes)
            let ctr_el0: u64;
            core::arch::asm!("mrs {0}, ctr_el0", out(reg) ctr_el0, options(nostack, preserves_flags));
            let dcache_line_size = (1 << ((ctr_el0 >> 16 & 0xF) as usize)) * 4;
            let inner = self.read();
            inner.gpm().for_each_region(|region| {
                // Invalidate all RAM regions of the guest
                if !region.flags.contains(MemFlags::IO) { // TODO: need to enrich the types and exercise more precise control
                    // Calculate the physical start address of the region
                    let phys_start = region.mapper.map_fn(region.start);
                    // Map phys_start to hvisor virtual address
                    let hva_start = phys_to_virt(phys_start);
                    info!("Invalidate Guest related cache, region.start: {:#x}, region.size: {:#x}, phys_start: {:#x}, hva_start: {:#x}", region.start, region.size, phys_start, hva_start);
                    // D-cache invalid operation will broadcast to all cores, just do it once. There is no need to do it on each core.
                    invalidate_dcache_range(hva_start, region.size, dcache_line_size);
                }
            });
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {
    pub is_aarch32: u8,
    pub gic_config: GicConfig,
}

#[repr(C, usize)]
#[derive(Debug, Clone)]
#[allow(unused)]
pub enum GicConfig {
    Gicv2(Gicv2Config),
    Gicv3(Gicv3Config),
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Gicv2Config {
    pub gicd_base: usize,
    pub gicd_size: usize,
    pub gicc_base: usize,
    pub gicc_size: usize,
    pub gicc_offset: usize,
    pub gich_base: usize,
    pub gich_size: usize,
    pub gicv_base: usize,
    pub gicv_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Gicv3Config {
    pub gicd_base: usize,
    pub gicd_size: usize,
    pub gicr_base: usize,
    pub gicr_size: usize,
    pub gits_base: usize,
    pub gits_size: usize,
}
