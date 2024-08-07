use spin::RwLock;

use crate::{
    arch::Stage2PageTable,
    error::HvResult,
    memory::{
        addr::align_up, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet, HV_PT,
    },
};

pub fn init_hv_page_table() -> HvResult {
    todo!();
    // let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new();
    // // let _ = hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    // //     0x8000_0000 as HostVirtAddr,
    // //     hv_phys_start as HostPhysAddr,
    // //     (hv_phys_end - hv_phys_start) as usize,
    // //     MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    // // ));
    // trace!("fdt: {:?}", fdt);
    // // The first memory region is used to map the guest physical memory.
    // let mem_region = fdt.memory().regions().next().unwrap();
    // debug!("map mem_region: {:#x?}", mem_region);
    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     mem_region.starting_address as GuestPhysAddr,
    //     mem_region.starting_address as HostPhysAddr,
    //     mem_region.size.unwrap(),
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    // ))?;
    // // probe virtio mmio device
    // for node in fdt.find_all_nodes("/soc/virtio_mmio") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap();
    //         debug!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE,
    //         ))?;
    //     }
    // }

    // // probe virt test
    // for node in fdt.find_all_nodes("/soc/test") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap() + 0x1000;
    //         debug!("map test addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    //         ))?;
    //     }
    // }

    // // probe uart device
    // for node in fdt.find_all_nodes("/soc/uart") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = align_up(reg.size.unwrap());
    //         debug!("map uart addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE,
    //         ))?;
    //     }
    // }

    // // probe clint(core local interrupter)
    // for node in fdt.find_all_nodes("/soc/clint") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap();
    //         debug!("map clint addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE,
    //         ))?;
    //     }
    // }

    // // probe plic
    // for node in fdt.find_all_nodes("/soc/plic") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap();
    //         debug!("map plic addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE,
    //         ))?;
    //     }
    // }

    // for node in fdt.find_all_nodes("/soc/pci") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap();
    //         debug!("map pci addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE,
    //         ))?;
    //     }
    // }
    // info!("Hypervisor page table init end.");
    // debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    // HV_PT.call_once(|| RwLock::new(hv_pt));
    // Ok(())
}

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(3)
}
