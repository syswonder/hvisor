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
    pci::pcibar::BarRegion,
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
    pub gicd_base: usize,
    pub gicd_size: usize,
    pub gicr_base: usize,
    pub gicr_size: usize,
    pub gits_base: usize,
    pub gits_size: usize,
    pub gicc_base: usize,
    pub gicc_offset: usize,
    pub gicc_size: usize,
    pub gich_base: usize,
    pub gich_size: usize,
    pub gicv_base: usize,
    pub gicv_size: usize,
    pub is_aarch32: u8,
}

impl BarRegion {
    pub fn arch_set_bar_region_start(&mut self, cpu_base: usize, pci_base: usize) {
        self.start = crate::memory::addr::align_down(cpu_base + self.start - pci_base);
    }

    pub fn arch_insert_bar_region(&self, gpm: &mut MemorySet<Stage2PageTable>, zone_id: usize) {
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            self.start as GuestPhysAddr,
            self.start,
            self.size,
            MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
        ))
        .ok();
    }
}
