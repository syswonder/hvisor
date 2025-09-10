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
use crate::consts::{IPI_EVENT_SEND_IPI, IPI_EVENT_UPDATE_HART_LINE};
use sbi_rt::HartMask;
use sbi_rt::SbiRet;

// arch_send_event
pub fn arch_send_event(cpu_id: u64, _sgi_num: u64) {
    debug!("arch_send_event: cpu_id: {}", cpu_id);
    #[cfg(feature = "aclint")]
    crate::device::irqchip::aclint::aclint_send_ipi(cpu_id as usize);
    #[cfg(not(feature = "aclint"))]
    {
        let sbi_ret: SbiRet = sbi_rt::send_ipi(HartMask::from_mask_base(1 << cpu_id, 0));
        if sbi_ret.is_err() {
            error!("arch_send_event: send_ipi failed: {:?}", sbi_ret);
        }
    }
}

/// Handle send_ipi event.
pub fn arch_ipi_handler() {
    unsafe {
        riscv_h::register::hvip::set_vssip();
    }
}

pub fn arch_check_events(event: Option<usize>) {
    match event {
        #[cfg(feature = "plic")]
        Some(IPI_EVENT_UPDATE_HART_LINE) => {
            use crate::device::irqchip::plic::update_hart_line;
            update_hart_line();
        }
        Some(IPI_EVENT_SEND_IPI) => {
            use crate::arch::riscv64::ipi::arch_ipi_handler;
            arch_ipi_handler();
        }
        _ => {
            panic!("arch_check_events: unhandled event: {:?}", event);
        }
    }
}

pub fn arch_prepare_send_event(cpu_id: usize, ipi_int_id: usize, event_id: usize) {
    debug!("risc-v arch_prepare_send_event: do nothing now.")
}
