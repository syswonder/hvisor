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
//  Jingyu Liu <liujingyu24s@ict.ac.cn>
//

pub unsafe fn invalidate_dcache_range(start: usize, size: usize) {
    // Get cache line size from CTR_EL0[16:19] (min line size, in words of 4 bytes)
    let ctr_el0: u64;
    core::arch::asm!("mrs {0}, ctr_el0", out(reg) ctr_el0, options(nostack, preserves_flags));
    let line_size = (1 << ((ctr_el0 >> 16 & 0xF) as usize)) * 4;

    let mut addr = start & !(line_size - 1);
    let end = start + size;
    while addr < end {
        core::arch::asm!("dc civac, {0}", in(reg) addr, options(nostack, preserves_flags));
        addr += line_size;
    }
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}
