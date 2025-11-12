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

use crate::{
    arch::Stage2PageTable,
    config::*,
    device::virtio_trampoline::mmio_virtio_handler,
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet},
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
                    // hvisor-tool will check memory type. So only root linux can reach here.
                    panic!("Unsupported memory type: {}", mem_region.mem_type)
                }
            }
        }

        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    pub fn iommu_pt_init(
        &mut self,
        mem_regions: &[HvConfigMemoryRegion],
        hv_config: &HvArchZoneConfig,
    ) -> HvResult {
        // Create a new stage 2 page table for iommu.
        // Only map the memory regions that are possible to be accessed by devices as DMA buffer.

        let pt = self.iommu_pt.as_mut().unwrap();
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
        self.ivc_init(config.ivc_config());
        Ok(())
    }

    pub fn arch_zone_post_configuration(&mut self, config: &HvZoneConfig) -> HvResult {
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
