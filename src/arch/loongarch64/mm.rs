use crate::{
    arch::s1pt::Stage1PageTable,
    consts::PAGE_SIZE,
    error::HvResult,
    memory::{
        addr::{align_down, align_up},
        GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet, HV_PT,
    },
};
use spin::*;

pub fn init_hv_page_table(fdt: &fdt::Fdt) -> HvResult {
    let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new();
    // let mem_region = fdt.memory().regions().next().unwrap();
    // info!("loongarch64: mm: mem_region: {:#x?}", mem_region);
    // find all serial
    for node in fdt.find_all_nodes("/platform/serial") {
        if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
            let paddr = reg.starting_address as HostPhysAddr;
            let size = reg.size.unwrap();
            info!(
                "loongarch64: mm: map serial addr: {:#x}, size: {:#x}",
                paddr, size
            );
            let paddr = align_down(paddr);
            let size = align_up(size);
            hv_pt.insert(MemoryRegion::new_with_offset_mapper(
                paddr as GuestPhysAddr,
                paddr,
                size,
                MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
            ))?;
        }
    }
    info!("loongarch64: mm: init_hv_page_table done");
    info!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    HV_PT.call_once(|| RwLock::new(hv_pt));
    Ok(())
}
