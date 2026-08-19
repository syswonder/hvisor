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
    arch::cpu::this_cpu_id,
    arch::ipi::{arch_check_events, arch_notify_event},
    consts::{
        IPI_EVENT_CLEAR_INJECT_IRQ, IPI_EVENT_DWC_MSI_INJECT, IPI_EVENT_SEND_IPI,
        IPI_EVENT_UPDATE_HART_LINE, IPI_EVENT_VCPU_SUSPEND, MAX_CPU_NUM,
    },
    cpu_data::{this_cpu_data, vcpu_suspend, CpuSet},
    device::irqchip::{handle_virtio_irq_event, inject_irq},
    platform::IRQ_WAKEUP_VIRTIO_DEVICE,
};
#[cfg(virtio_pci)]
use crate::{
    pci::msix::activate_msix,
    platform::{IRQ_WAKEUP_VIRTIO_PCI_CONFIG, IRQ_WAKEUP_VIRTIO_PCI_DATA},
};
use alloc::{collections::VecDeque, vec::Vec};
use spin::Mutex;

pub const IPI_EVENT_WAKEUP: usize = 0;
pub const IPI_EVENT_SHUTDOWN: usize = 1;
pub const IPI_EVENT_VIRTIO_INJECT_IRQ: usize = 2;
pub const IPI_EVENT_WAKEUP_VIRTIO_DEVICE: usize = 3;
pub const IPI_EVENT_VIRTIO_PCI_CONFIG: usize = 7;
pub const IPI_EVENT_VIRTIO_PCI_DATA: usize = 8;
pub const IPI_EVENT_VIRTIO_PCI_DONE: usize = 9;

#[percpu::def_percpu]
static PERCPU_EVENTS: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

// The caller ensures the cpu_id is valid
#[inline(always)]
fn get_percpu_events(cpu: usize) -> &'static Mutex<VecDeque<usize>> {
    unsafe { PERCPU_EVENTS.remote_ref_raw(cpu) }
}

/// Enqueue an event and report whether the target queue was previously empty.
/// Architectures can use the transition to coalesce their notification mechanism.
fn add_event(cpu: usize, event_id: usize) -> Option<bool> {
    if cpu >= MAX_CPU_NUM {
        return None;
    }
    let mut e = get_percpu_events(cpu).lock();
    let was_empty = e.is_empty();
    if event_id == IPI_EVENT_SHUTDOWN {
        // If the event is shutdown, we need to clear all previous events, because shutdown will make cpu idle and won't process any events.
        e.clear();
    }
    e.push_back(event_id);
    Some(was_empty)
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

fn handle_event(event: Option<usize>) -> bool {
    let cpu_data = this_cpu_data();
    match event {
        Some(IPI_EVENT_WAKEUP) => {
            cpu_data.arch_cpu.run();
            false
        }
        Some(IPI_EVENT_SHUTDOWN) => {
            #[cfg(target_arch = "loongarch64")]
            {
                // Shutdown coalescing may have discarded the queued IRQ-sync
                // event. Apply the cleared software bitmap before idling so the
                // next zone cannot inherit stale GINTC.VIP bits.
                crate::device::irqchip::ls7a2000::sync_guest_irqs();
            }
            cpu_data.arch_cpu.idle();
            false
        }
        Some(IPI_EVENT_VIRTIO_INJECT_IRQ) => {
            handle_virtio_irq_event();
            true
        }
        Some(IPI_EVENT_WAKEUP_VIRTIO_DEVICE) => {
            inject_irq(IRQ_WAKEUP_VIRTIO_DEVICE, false);
            true
        }
        Some(IPI_EVENT_DWC_MSI_INJECT) => {
            #[cfg(all(target_arch = "aarch64", irq_gicv3, dwc_pcie, dwc_msi))]
            {
                crate::pci::dwc_msi::handle_dwc_msi_inject_event();
            }
            true
        }
        #[cfg(virtio_pci)]
        Some(IPI_EVENT_VIRTIO_PCI_CONFIG) => {
            inject_irq(IRQ_WAKEUP_VIRTIO_PCI_CONFIG, false);
            true
        }
        #[cfg(virtio_pci)]
        Some(IPI_EVENT_VIRTIO_PCI_DATA) => {
            inject_irq(IRQ_WAKEUP_VIRTIO_PCI_DATA, false);
            true
        }
        #[cfg(virtio_pci)]
        Some(IPI_EVENT_VIRTIO_PCI_DONE) => {
            // Virtio PCI notice
            // unsafe {
            //     VIRTIO_MSIX_MANAGER.write().activate_all_pending_irq();
            // }
            activate_msix();
            true
        }
        Some(IPI_EVENT_CLEAR_INJECT_IRQ)
        | Some(IPI_EVENT_UPDATE_HART_LINE)
        | Some(IPI_EVENT_SEND_IPI) => {
            arch_check_events(event);
            true
        }
        Some(IPI_EVENT_VCPU_SUSPEND) => {
            vcpu_suspend();
            true
        }
        _ => false,
    }
}

pub fn check_events() -> bool {
    handle_event(fetch_event(this_cpu_data().id))
}

/// Handle one queued event, returning whether an event was present.
pub fn handle_next_event() -> bool {
    let event = fetch_event(this_cpu_data().id);
    if event.is_none() {
        return false;
    }
    handle_event(event);
    true
}

pub fn send_event(cpu_id: usize, ipi_int_id: usize, event_id: usize) {
    if let Some(queue_was_empty) = add_event(cpu_id, event_id) {
        arch_notify_event(cpu_id as _, ipi_int_id as _, event_id, queue_was_empty);
    }
}

/// Send event to a cpu set (except self).
pub fn send_event_to_all(cpu_set: CpuSet, ipi_int_id: usize, event_id: usize) {
    let this_cpu_id = this_cpu_id();
    for target_cpu_id in cpu_set.iter() {
        if target_cpu_id == this_cpu_id {
            continue;
        }
        info!(
            "send_event_to_all: send event {} to cpu {}",
            event_id, target_cpu_id
        );
        send_event(target_cpu_id, ipi_int_id, event_id);
    }
}
