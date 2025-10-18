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
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{acpi, boot, s1pt::Stage1PageTable, s2pt::Stage2PageTable},
    error::HvResult,
    memory::MemorySet,
};

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}

pub fn init_hv_page_table() -> HvResult {
    Ok(())
}

pub fn arch_setup_parange() {
    // x86_64 does not have a parange setup like AArch64.
    // The parange is determined by the memory regions defined in the device tree.
    // So we do not need to do anything here.
}

pub fn arch_post_heap_init(host_dtb: usize) {
    boot::module_init(host_dtb);
    acpi::root_init();
}
