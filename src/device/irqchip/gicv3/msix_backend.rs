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

use alloc::{sync::Arc, vec::Vec};
use spin::rwlock::RwLock;

use crate::{
    device::irqchip::inject_irq,
    pci::msix::{MsixBackend, MsixTableEntry},
};

type MsixEventID = u64;
type MsixDeviceID = u64;
type MsixIrqID = u64;

const INACTIVE_IDX: usize = 114514;

static mut GICV3_MSIX_BACKEND_IDX: usize = INACTIVE_IDX;
static mut GICV3_MSIX_BACKEND_LIST: Vec<Arc<RwLock<Gicv3MsixBackend>>> = Vec::new();

pub fn get_gicv3_backend() -> Arc<RwLock<Gicv3MsixBackend>> {
    unsafe {
        let backend = Arc::new(RwLock::new(Gicv3MsixBackend::new(
            GICV3_MSIX_BACKEND_LIST.len(),
        )));
        GICV3_MSIX_BACKEND_LIST.push(backend.clone());
        backend
    }
}

pub fn get_current_msix_backend() -> Option<Arc<RwLock<Gicv3MsixBackend>>> {
    unsafe {
        if GICV3_MSIX_BACKEND_IDX == INACTIVE_IDX {
            return None;
        }
        Some(GICV3_MSIX_BACKEND_LIST[GICV3_MSIX_BACKEND_IDX].clone())
    }
}

pub fn msix_intercept_its(device_id: MsixDeviceID, event_id: MsixEventID, irq_id: MsixIrqID) {
    // let device_id = device_id >> 32;
    match get_current_msix_backend().clone() {
        Some(x) => {
            x.write().intercept_its(device_id, event_id, irq_id);
        }
        None => {
            //do nothing
        }
    }
}

#[derive(Debug)]
pub struct Gicv3MsixBackend {
    is_enable: bool,
    idx: usize,
    irq_info_list: Vec<(MsixDeviceID, MsixEventID, MsixIrqID)>,
}

impl MsixBackend for Gicv3MsixBackend {
    fn activate_irq(&self, device_id: usize, vector: &MsixTableEntry) {
        // info!(
        //     "irq_info_list:{:x?},device_id:0x{:x},vector:{:x?}",
        //     self.irq_info_list, device_id, vector
        // );
        // inject_irq(0x2001, false);
        for i in self.irq_info_list.iter() {
            if i.0 == device_id as u64 && i.1 == vector.get_msg_data() as u64 {
                inject_irq(i.2 as usize, false);
                return;
            }
        }
        warn!("can't find corresponding Msix vector irq:{:x?}", vector)
    }

    fn disable(&mut self) {
        if !self.is_enable {
            return;
        }
        unsafe { GICV3_MSIX_BACKEND_IDX = INACTIVE_IDX }
        self.is_enable = false
    }

    fn enable(&mut self) {
        unsafe { GICV3_MSIX_BACKEND_IDX = self.idx }
        self.is_enable = true
    }
}

impl Gicv3MsixBackend {
    pub fn new(idx: usize) -> Self {
        Self {
            is_enable: false,
            idx,
            irq_info_list: Vec::new(),
        }
    }

    pub fn intercept_its(
        &mut self,
        device_id: MsixDeviceID,
        event_id: MsixEventID,
        irq_id: MsixIrqID,
    ) {
        if self.is_enable {
            self.irq_info_list.push((device_id, event_id, irq_id));
        }
    }
}
