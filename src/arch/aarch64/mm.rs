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
use core::sync::atomic::AtomicU32;

use spin::RwLock;

use crate::{
    arch::Stage2PageTable, consts::MAX_CPU_NUM, error::HvResult, memory::MemorySet, wait_for,
};

use super::sysreg::read_sysreg;

pub fn init_hv_page_table() -> HvResult {
    todo!();
    // let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new(4);
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
    // let mut last_mmio_addr: Option<usize> = None;
    // for node in fdt.find_all_nodes("/virtio_mmio") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap();
    //         if last_mmio_addr.is_none() || last_mmio_addr.unwrap() != align_down(paddr) {
    //             debug!(
    //                 "map virtio mmio addr: {:#x}, size: {:#x}",
    //                 paddr,
    //                 size.max(PAGE_SIZE)
    //             );
    //             hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //                 paddr as GuestPhysAddr,
    //                 paddr,
    //                 size.max(PAGE_SIZE),
    //                 MemFlags::READ | MemFlags::WRITE,
    //             ))?;
    //         }
    //         last_mmio_addr = Some(align_down(paddr));
    //     }
    // }

    // // probe virt test
    // // for node in fdt.find_all_nodes("/soc/test") {
    // //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    // //         let paddr = reg.starting_address as HostPhysAddr;
    // //         let size = reg.size.unwrap() + 0x1000;
    // //         debug!("map test addr: {:#x}, size: {:#x}", paddr, size);
    // //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    // //             paddr as GuestPhysAddr,
    // //             paddr,
    // //             size,
    // //             MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
    // //         ))?;
    // //     }
    // // }

    // // probe uart device
    // for node in fdt.find_all_nodes("/pl011") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = align_up(reg.size.unwrap());
    //         debug!("map uart addr: {:#x}, size: {:#x}", paddr, size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    //         ))?;
    //     }
    // }

    // // probe gic...

    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     host_gicd_base(),
    //     host_gicd_base(),
    //     host_gicd_size(),
    //     MemFlags::READ | MemFlags::WRITE,
    // ))?;

    // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //     host_gicr_base(0),
    //     host_gicr_base(0),
    //     host_gicr_size(),
    //     MemFlags::READ | MemFlags::WRITE,
    // ))?;

    // info!("Hypervisor page table initialization completed.");
    // debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    // HV_PT.call_once(|| RwLock::new(hv_pt));
    // Ok(())
}

const PARANGE_TABLE: [usize; 6] = [32, 36, 40, 42, 44, 48];
static MIN_PARANGE: RwLock<u64> = RwLock::new(0x7);
static PARANGE_OK_CPUS: AtomicU32 = AtomicU32::new(0);

static mut NCPU: usize = 0;

pub fn setup_parange(ncpu: usize) {
    unsafe {
        NCPU = ncpu;
    }
    let temp_parange = read_sysreg!(id_aa64mmfr0_el1) & 0xf;
    let mut p = MIN_PARANGE.write();
    *p = p.min(temp_parange);
    drop(p);

    PARANGE_OK_CPUS.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    wait_for(|| PARANGE_OK_CPUS.load(core::sync::atomic::Ordering::SeqCst) < unsafe { NCPU } as _);
}

pub fn get_parange() -> u64 {
    assert!(PARANGE_OK_CPUS.load(core::sync::atomic::Ordering::SeqCst) == unsafe { NCPU } as _);
    *MIN_PARANGE.read()
}

pub fn get_parange_bits() -> usize {
    assert!(PARANGE_OK_CPUS.load(core::sync::atomic::Ordering::SeqCst) == unsafe { NCPU } as _);
    PARANGE_TABLE[*MIN_PARANGE.read() as usize]
}

pub fn is_s2_pt_level3() -> bool {
    get_parange_bits() < 44
}

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(if is_s2_pt_level3() { 3 } else { 4 })
}
