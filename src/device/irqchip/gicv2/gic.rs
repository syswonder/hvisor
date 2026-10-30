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
//      Hangqi Ren <2572131118@qq.com>
use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICV2_SGIS_NUM;
use crate::device::irqchip::gicv2::lr::{
    handle_maintenace_interrupt, inject_irq, MAINTENACE_INTERRUPT,
};
use crate::event::{check_events, dispatch_host_events};
use crate::hypercall::SGI_IPI_ID;

pub fn gicv2_handle_irq() {
    if let Some(irq_id) = get_pending_irq() {
        if irq_id < GICV2_SGIS_NUM {
            deactivate_irq(irq_id);
            if irq_id == SGI_IPI_ID as _ {
                dispatch_host_events(|| while check_events() {});
            } else {
                warn!("unexpected physical SGI {}", irq_id);
            }
        } else if irq_id == MAINTENACE_INTERRUPT as usize {
            handle_maintenace_interrupt();
        } else {
            deactivate_irq(irq_id);
            inject_irq(irq_id, false);
        }
    }
}

pub fn get_pending_irq() -> Option<usize> {
    let iar = GICC.get().unwrap().get_iar() as usize;
    let irq = iar & 0x3ff;
    if irq >= 1023 {
        None
    } else {
        Some(irq)
    }
}

// deactivate irq: GIC doesn't care CPU ID.
pub fn deactivate_irq(irq_id: usize) {
    GICC.get().unwrap().set_eoir(irq_id as u32);
    if irq_id < GICV2_SGIS_NUM {
        GICC.get().unwrap().set_dir(irq_id as u32);
    }
}
