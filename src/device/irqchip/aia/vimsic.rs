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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use crate::consts::PAGE_SIZE;
use crate::memory::GuestPhysAddr;
use crate::memory::HostPhysAddr;
use crate::memory::MemFlags;
use crate::memory::MemoryRegion;
use crate::platform::__board::{IMSIC_GUEST_INDEX, IMSIC_GUEST_NUM, IMSIC_S_BASE};
use crate::zone::Zone;

/**
 * For imsic's guest_num = 1
 * its interrupt files:
 *     S-file CPU0      base + 0x0000
 *    VS-file CPU0      base + 0x1000
 *     S-file CPU1      base + 0x2000
 *    VS-file CPU1      base + 0x3000
 *     ...
 */

pub fn vimsic_init(zone: &mut Zone, imsic_base: usize, guest_num: usize) {
    let paddr = imsic_base as HostPhysAddr;
    let size = crate::memory::PAGE_SIZE;
    zone.cpu_set.iter().for_each(|cpu_id| {
        let vcpu_id = cpu_id; // In hvisor, vcpu_id == cpu_id.
        let imsic_hpa = imsic_base + PAGE_SIZE * ((1 + guest_num) * cpu_id + IMSIC_GUEST_INDEX);
        // For VM, it couldn't see VS-files.
        let imsic_gpa = imsic_base + PAGE_SIZE * vcpu_id; // In hvisor, vcpu_id == cpu_id.
        info!(
            "Zone {} vIMSIC map hart {} imsic hpa {:#x} gpa {:#x}",
            zone.id, cpu_id, imsic_hpa, imsic_gpa
        );
        zone.gpm.insert(MemoryRegion::new_with_offset_mapper(
            imsic_gpa as GuestPhysAddr,
            imsic_hpa,
            size,
            MemFlags::READ | MemFlags::WRITE,
        ));
    });
}

pub fn imsic_vs_file_addr(hart_id: usize) -> usize {
    IMSIC_S_BASE + PAGE_SIZE * ((1 + IMSIC_GUEST_NUM) * hart_id + IMSIC_GUEST_INDEX)
}
