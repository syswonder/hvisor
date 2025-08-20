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
//      Yulong Han <wheatfox17@icloud.com>
//
use crate::{
    arch::s1pt::Stage1PageTable,
    arch::s2pt::Stage2PageTable,
    consts::PAGE_SIZE,
    error::HvResult,
    memory::{
        addr::{align_down, align_up},
        GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet,
    },
};
use spin::*;

pub const LOONGARCH64_CACHED_DMW_PREFIX: u64 = 0x9000_0000_0000_0000;
pub const LOONGARCH64_UNCACHED_DMW_PREFIX: u64 = 0x8000_0000_0000_0000;

pub fn init_hv_page_table(fdt: &fdt::Fdt) -> HvResult {
    todo!();
    // let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new(4);
    // // let mem_region = fdt.memory().regions().next().unwrap();
    // // info!("loongarch64: mm: mem_region: {:#x?}", mem_region);
    // // find all serial
    // for node in fdt.find_all_nodes("/platform/serial") {
    //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
    //         let paddr = reg.starting_address as HostPhysAddr;
    //         let size = reg.size.unwrap();
    //         info!(
    //             "loongarch64: mm: map serial addr: {:#x}, size: {:#x}",
    //             paddr, size
    //         );
    //         let paddr = align_down(paddr);
    //         let size = align_up(size);
    //         hv_pt.insert(MemoryRegion::new_with_offset_mapper(
    //             paddr as GuestPhysAddr,
    //             paddr,
    //             size,
    //             MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    //         ))?;
    //     }
    // }
    // info!("loongarch64: mm: init_hv_page_table done");
    // debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

    // HV_PT.call_once(|| RwLock::new(hv_pt));
    // Ok(())
}

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}

pub fn arch_setup_parange() {
    // LoongArch64 does not have a parange setup like AArch64.
    // This function can be used to set up any architecture-specific parameters if needed.
    // Currently, it does nothing.
}

pub fn arch_post_heap_init(host_dtb: usize) {
    // LoongArch64 does not need to do some setup work after heap init like x86_64.
    // This function can be used to set up any architecture-specific parameters if needed.
    // Currently, it does nothing.
}
