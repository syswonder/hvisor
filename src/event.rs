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
#![allow(unused)]
use crate::{
    arch::ipi::{arch_check_events, arch_prepare_send_event, arch_send_event},
    consts::{
        IPI_EVENT_CLEAR_INJECT_IRQ, IPI_EVENT_SEND_IPI, IPI_EVENT_UPDATE_HART_LINE, MAX_CPU_NUM,
    },
    cpu_data::this_cpu_data,
    device::{
        irqchip::inject_irq,
        virtio_trampoline::{handle_virtio_irq, IRQ_WAKEUP_VIRTIO_DEVICE},
    },
};
use alloc::{collections::VecDeque, vec::Vec};
use percpu::def_percpu;
use spin::{Mutex, Once};

pub const IPI_EVENT_WAKEUP: usize = 0;
pub const IPI_EVENT_SHUTDOWN: usize = 1;
pub const IPI_EVENT_VIRTIO_INJECT_IRQ: usize = 2;
pub const IPI_EVENT_WAKEUP_VIRTIO_DEVICE: usize = 3;

#[percpu::def_percpu]
static PERCPU_EVENTS: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

// The caller ensures the cpu_id is vaild
#[inline(always)]
fn get_percpu_events(cpu: usize) -> &'static Mutex<VecDeque<usize>> {
    unsafe { PERCPU_EVENTS.remote_ref_raw(cpu) }
}

fn add_event(cpu: usize, event_id: usize) -> Option<()> {
    if cpu >= MAX_CPU_NUM {
        return None;
    }
    let mut e = get_percpu_events(cpu).lock();
    if event_id == IPI_EVENT_SHUTDOWN {
        // If the event is shutdown, we need to clear all previous events, because shutdown will make cpu idle and won't process any events.
        e.clear();
    }
    e.push_back(event_id);
    Some(())
}

pub fn fetch_event(cpu: usize) -> Option<usize> {
    if cpu >= MAX_CPU_NUM {
        return None;
    }
    get_percpu_events(cpu).lock().pop_front()
}

pub fn dump_events() {
    for cpu in 0..MAX_CPU_NUM {
        let events = get_percpu_events(cpu).lock();
        if !events.is_empty() {
            debug!("cpu {} events: {:?}", cpu, *events);
        }
    }
}

pub fn dump_cpu_events(cpu: usize) -> Vec<usize> {
    if cpu >= MAX_CPU_NUM {
        return Vec::new();
    }
    get_percpu_events(cpu).lock().iter().cloned().collect()
}

pub fn clear_events(cpu: usize) {
    if cpu >= MAX_CPU_NUM {
        return;
    }
    get_percpu_events(cpu).lock().clear();
}

pub fn check_events() -> bool {
    let cpu_data = this_cpu_data();
    let event = fetch_event(cpu_data.id);
    match event {
        Some(IPI_EVENT_WAKEUP) => {
            cpu_data.arch_cpu.run();
            false
        }
        Some(IPI_EVENT_SHUTDOWN) => {
            cpu_data.arch_cpu.idle();
        }
        Some(IPI_EVENT_VIRTIO_INJECT_IRQ) => {
            handle_virtio_irq();
            true
        }
        Some(IPI_EVENT_WAKEUP_VIRTIO_DEVICE) => {
            inject_irq(IRQ_WAKEUP_VIRTIO_DEVICE, false);
            true
        }
        Some(IPI_EVENT_CLEAR_INJECT_IRQ)
        | Some(IPI_EVENT_UPDATE_HART_LINE)
        | Some(IPI_EVENT_SEND_IPI) => {
            arch_check_events(event);
            true
        }
        // #[cfg(target_arch = "loongarch64")]
        // Some(IPI_EVENT_CLEAR_INJECT_IRQ) => {
        //     use crate::device::irqchip;
        //     irqchip::ls7a2000::clear_hwi_injected_irq();
        //     true
        // }
        // #[cfg(all(target_arch = "riscv64", feature = "plic"))]
        // Some(IPI_EVENT_UPDATE_HART_LINE) => {
        //     use crate::device::irqchip;
        //     info!("cpu {} update hart line", cpu_data.id);
        //     irqchip::plic::update_hart_line();
        //     true
        // }
        // #[cfg(target_arch = "riscv64")]
        // Some(IPI_EVENT_SEND_IPI) => {
        //     // This event is different from events above, it is used to inject software interrupt.
        //     // While events above will inject external interrupt.
        //     use crate::arch::ipi::arch_ipi_handler;
        //     arch_ipi_handler();
        //     true
        // }
        _ => false,
    }
}

pub fn send_event(cpu_id: usize, ipi_int_id: usize, event_id: usize) {
    // #[cfg(target_arch = "loongarch64")]
    // {
    //     // block until the previous event is processed, which means
    //     // the target queue is empty
    //     while !fetch_event(cpu_id).is_none() {}
    //     debug!(
    //         "loongarch64:: send_event: cpu_id: {}, ipi_int_id: {}, event_id: {}",
    //         cpu_id, ipi_int_id, event_id
    //     );
    // }
    /// Some arch need do something before send event.
    /// Currently, we are not passing parameters, and we will modify the function signature later as needed.
    arch_prepare_send_event(cpu_id, ipi_int_id, event_id);
    add_event(cpu_id, event_id);
    arch_send_event(cpu_id as _, ipi_int_id as _);
}
