use spin::RwLock;

use crate::{
    arch::{s1pt::Stage1PageTable, smmuv3::{smmuv3_base, smmuv3_size}},
    consts::PAGE_SIZE,
    device::irqchip::gicv3::{host_gicd_base, host_gicd_size, host_gicr_base, host_gicr_size},
    error::HvResult,
    memory::{
        addr::{align_down, align_up},
        GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet, HV_PT,
    },
};

pub fn init_hv_page_table(fdt: &fdt::Fdt) -> HvResult {
    let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new();
    // let _ = hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     0x8000_0000 as HostVirtAddr,
    //     hv_phys_start as HostPhysAddr,
    //     (hv_phys_end - hv_phys_start) as usize,
    //     MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    // ));
    trace!("fdt: {:?}", fdt);
    // The first memory region is used to map the guest physical memory.
    let mem_region = fdt.memory().regions().next().unwrap();
    debug!("map mem_region: {:#x?}", mem_region);
    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        mem_region.starting_address as GuestPhysAddr,
        mem_region.starting_address as HostPhysAddr,
        mem_region.size.unwrap(),
        MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    ))?;

    // probe virtio mmio device
    let mut last_mmio_addr: Option<usize> = None;
    for node in fdt.find_all_nodes("/virtio_mmio") {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let paddr = reg.starting_address as HostPhysAddr;
            let size = reg.size.unwrap();
            if last_mmio_addr.is_none() || last_mmio_addr.unwrap() != align_down(paddr) {
                debug!(
                    "map virtio mmio addr: {:#x}, size: {:#x}",
                    paddr,
                    size.max(PAGE_SIZE)
                );
                hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size.max(PAGE_SIZE),
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
            last_mmio_addr = Some(align_down(paddr));
        }
    }

    // probe virt test
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

    // probe uart device
    for node in fdt.find_all_nodes("/pl011") {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let paddr = reg.starting_address as HostPhysAddr;
            let size = align_up(reg.size.unwrap());
            debug!("map uart addr: {:#x}, size: {:#x}", paddr, size);
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }
    }

    // probe gic...

    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        host_gicd_base(),
        host_gicd_base(),
        host_gicd_size(),
        MemFlags::READ | MemFlags::WRITE,
    ))?;

    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        host_gicr_base(0),
        host_gicr_base(0),
        host_gicr_size(),
        MemFlags::READ | MemFlags::WRITE,
    ))?;

    hv_pt.insert(MemoryRegion::new_with_offset_mapper(
        smmuv3_base(),
        smmuv3_base(),
        smmuv3_size(),
        MemFlags::READ | MemFlags::WRITE,
    ))?;
    

    info!("Hypervisor page table initialization completed.");
    debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    HV_PT.call_once(|| RwLock::new(hv_pt));
    Ok(())
}
