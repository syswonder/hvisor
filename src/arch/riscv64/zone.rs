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
use crate::{
    config::*,
    device::virtio_trampoline::{mmio_virtio_handler, VIRTIO_BRIDGE},
    error::HvResult,
    memory::{addr::align_up, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion},
    percpu::get_cpu_data,
    zone::Zone,
};
impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
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
        #[cfg(feature = "aia")]
        {
            use crate::memory::PAGE_SIZE;
            let paddr = 0x2800_0000 as HostPhysAddr;
            let size = PAGE_SIZE;
            self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr + PAGE_SIZE * 1,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;

            let paddr = 0x2800_1000 as HostPhysAddr;
            let size = PAGE_SIZE;
            self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr + PAGE_SIZE * 2,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;

            let paddr = 0x2800_2000 as HostPhysAddr;
            let size = PAGE_SIZE;
            self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr + PAGE_SIZE * 3,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;

            let paddr = 0x2800_3000 as HostPhysAddr;
            let size = PAGE_SIZE;
            self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr + PAGE_SIZE * 4,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }
        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        let cpu_set = self.cpu_set;
        cpu_set.iter().for_each(|cpuid| {
            let cpu_data = get_cpu_data(cpuid);
            let cpu_isa = fdt
                .cpus()
                .find(|cpu| cpu.ids().all().next().unwrap() == cpuid)
                .unwrap()
                .properties()
                .find(|p| p.name == "riscv,isa")
                .unwrap();
            if cpu_isa.as_str().unwrap().contains("sstc") {
                println!("cpu{} support sstc", cpuid);
                cpu_data.arch_cpu.sstc = true;
            }
        })
    }

    pub fn arch_zone_configuration(&mut self, config: &HvZoneConfig) -> HvResult {
        // We do not have any specific architecture configuration for RISC-V.
        // If needed, this function can be extended in the future.
        Ok(())
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {
    pub plic_base: usize,
    pub plic_size: usize,
    pub aplic_base: usize,
    pub aplic_size: usize,
}
