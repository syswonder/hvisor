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

use crate::arch::cpu;

#[cfg(feature = "gicv3")]
use crate::arch::sysreg::write_sysreg;
#[cfg(feature = "gicv2")]
use crate::device::irqchip::set_sgi_irq;
pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    #[cfg(feature = "gicv3")]
    {
        /*Actually, the value passed to ICC_SGI1R_EL1 should be derived from
        the MPIDR of the target CPU. However, since we cannot access this
        register on the sender side, we have reverse-engineered a value
        here using the cpu_id.
        Due to differences in how some CPU implementations (e.g., RK3568 and RK3588)
        encode affinity values in MPIDR, we use conditional compilation to handle
        platform-specific mappings between cpu_id and interrupt target affinity.
        */
        let aff3: u64 = 0 << 48;
        let aff2: u64 = 0 << 32;
        let aff1: u64;
        let target_list: u64;

        if cfg!(feature = "mpidr_rockchip") {
            aff1 = cpu_id << 16;
            target_list = 1 << 0;
        } else {
            aff1 = 0 << 16;
            target_list = 1 << cpu_id;
        }
        let irm: u64 = 0 << 40;
        let sgi_id: u64 = sgi_num << 24;
        let val: u64 = aff1 | aff2 | aff3 | irm | sgi_id | target_list;
        write_sysreg!(icc_sgi1r_el1, val);
        debug!("write sgi sys value = {:#x}", val);
    }
    #[cfg(feature = "gicv2")]
    {
        let sgi_id: u64 = sgi_num;
        let target_list: u64 = 1 << cpu_id;
        set_sgi_irq(sgi_id as usize, target_list as usize, 0);
    }
}
