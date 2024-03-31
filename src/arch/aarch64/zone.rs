use alloc::vec::Vec;

use crate::{
    consts::PAGE_SIZE,
    device::irqchip::gicv3::Gic,
    error::HvResult,
    memory::{
        addr::{align_down, align_up},
        GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion,
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
            for node in fdt.find_all_nodes("/virtio_mmio") {
                if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                    let paddr = align_down(reg.starting_address as _) as HostPhysAddr;
                    let size = reg.size.unwrap().max(PAGE_SIZE);
                    if !mapped_virtio.contains(&paddr) {
                        debug!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                            paddr as GuestPhysAddr,
                            paddr,
                            size,
                            MemFlags::READ | MemFlags::WRITE,
                        ))?;
                        mapped_virtio.push(paddr);
                    }
                }
            }
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
        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    pub fn mmio_init(&mut self, fdt: &fdt::Fdt) {
        self.vgicv3_mmio_init(fdt);
    }
}
