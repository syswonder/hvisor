use core::sync::atomic::{fence, Ordering};

use alloc::vec::Vec;

use crate::{
    device::virtio_trampoline::{mmio_virtio_handler, VIRTIO_BRIDGE},
    error::HvResult,
    memory::{
        addr::align_up, mmio_generic_handler, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion,
    },
    zone::Zone,
};

impl Zone {
    pub fn pt_init(
        &mut self,
        vm_paddr_start: usize,
        fdt: &fdt::Fdt,
        guest_dtb: usize,
        dtb_ipa: usize,
    ) -> HvResult {
        //debug!("fdt: {:?}", fdt);
        // The first memory region is used to map the guest physical memory.
        let mem_region = fdt.memory().regions().next().unwrap();
        info!("map mem_region: {:#x?}", mem_region);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            mem_region.starting_address as GuestPhysAddr,
            vm_paddr_start as HostPhysAddr,
            mem_region.size.unwrap(),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // map guest dtb
        info!("map guest dtb: {:#x?}", dtb_ipa);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            dtb_ipa as GuestPhysAddr,
            guest_dtb as HostPhysAddr,
            align_up(fdt.total_size()),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;

        // probe virtio mmio device
        {
            let mut mapped_virtio = Vec::new();
            let dev = VIRTIO_BRIDGE.lock();
            let region = if dev.is_enable {
                Some(dev.immut_region())
            } else {
                None
            };
            for node in fdt.find_all_nodes("/virtio_mmio") {
                if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                    let paddr = reg.starting_address as HostPhysAddr;
                    let size = reg.size.unwrap();
                    if !mapped_virtio.contains(&paddr) {
                        info!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                        if region.is_some() {
                            let dev_region = region.clone().unwrap();
                            while dev_region.mmio_avail == 0 {}
                            fence(Ordering::Acquire);
                            if dev_region.mmio_addrs.contains(&(paddr as u64)) {
                                self.mmio_region_register(paddr, size, mmio_virtio_handler, paddr);
                            } else {
                                self.mmio_region_register(paddr, size, mmio_generic_handler, paddr);
                            }
                        } else {
                            self.mmio_region_register(paddr, size, mmio_generic_handler, paddr);
                        }
                        mapped_virtio.push(paddr);
                    }
                }
            }
            drop(dev);
        }

        // probe uart device
        for node in fdt.find_all_nodes("/pl011") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = align_up(reg.size.unwrap());
                info!("map uart addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe pcie device
        for node in fdt.find_all_nodes("/pcie") {
            if let Some(reg_iter) = node.reg() {
                for reg in reg_iter {
                    let paddr = reg.starting_address as HostPhysAddr;
                    let size = reg.size.unwrap();
                    info!("map pcie addr: {:#x}, size: {:#x}", paddr, size);
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        paddr as GuestPhysAddr,
                        paddr,
                        size,
                        MemFlags::READ | MemFlags::WRITE,
                    )).ok();
                }
            }
            if let Some(ranges_iter) = node.ranges() {
                for ranges in ranges_iter {
                    let paddr = ranges.starting_address as HostPhysAddr;
                    let size = ranges.size.unwrap();
                    info!("map pcie addr: {:#x}, size: {:#x}", paddr, size);
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        paddr as GuestPhysAddr,
                        paddr,
                        size,
                        MemFlags::READ | MemFlags::WRITE,
                    )).ok();
                }
            }
        }


        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())

    }

    pub fn mmio_init(&mut self, fdt: &fdt::Fdt) {
        self.vgicv3_mmio_init(fdt);
    }
    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        //nothing to do
    }
}
