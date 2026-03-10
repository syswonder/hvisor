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

use crate::device::irqchip::gic_send_event;
pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    gic_send_event(cpu_id, sgi_num);
}

pub fn arch_check_events(event: Option<usize>) {
    match event {
        _ => {
            info!(
                "aarch64: arch_check_events: event={:#x?} do nothing now",
                event
            );
        }
    }
}

pub fn arch_prepare_send_event(_cpu_id: usize, _ipi_int_id: usize, _event_id: usize) {
    debug!("aarch64 arch_prepare_send_event: do nothing now.")
}
