#![allow(unused)]

use crate::{
    arch::ipi::arch_send_event,
    device::{
        irqchip::inject_irq,
        virtio_trampoline::{handle_virtio_irq, IRQ_WAKEUP_VIRTIO_DEVICE},
    },
    percpu::this_cpu_data,
};
use alloc::{collections::VecDeque, vec::Vec};
use spin::{Mutex, Once};

#[cfg(test)]
mod tests;

pub const IPI_EVENT_WAKEUP: usize = 0;
pub const IPI_EVENT_SHUTDOWN: usize = 1;
pub const IPI_EVENT_VIRTIO_INJECT_IRQ: usize = 2;
pub const IPI_EVENT_WAKEUP_VIRTIO_DEVICE: usize = 3;
pub const IPI_EVENT_CLEAR_INJECT_IRQ: usize = 4;

static EVENT_MANAGER: Once<EventManager> = Once::new();

struct EventManager {
    pub inner: Vec<Mutex<VecDeque<usize>>>,
}

impl EventManager {
    fn new(max_cpus: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_cpus {
            let v = Mutex::new(VecDeque::new());
            vs.push(v)
        }
        Self { inner: vs }
    }

    fn add_event(&self, cpu: usize, event_id: usize) -> Option<()> {
        match self.inner.get(cpu) {
            Some(events) => {
                let mut e = events.lock();
                e.push_back(event_id);
                Some(())
            }
            None => None,
        }
    }

    fn fetch_event(&self, cpu: usize) -> Option<usize> {
        match self.inner.get(cpu) {
            Some(events) => {
                let mut e = events.lock();
                e.pop_front()
            }
            None => None,
        }
    }

    fn dump(&self) {
        for (cpu, events) in self.inner.iter().enumerate() {
            let e = events.lock();
            debug!("event manager: cpu: {}, events: {:?}", cpu, e);
        }
    }

    fn dump_cpu(&self, cpu: usize) -> Vec<usize> {
        let mut res = Vec::new();
        let e = self.inner[cpu].lock();
        for i in e.iter() {
            res.push(*i);
        }
        res
    }
}

fn add_event(cpu: usize, event_id: usize) -> Option<()> {
    EVENT_MANAGER.get().unwrap().add_event(cpu, event_id)
}

fn fetch_event(cpu: usize) -> Option<usize> {
    EVENT_MANAGER.get().unwrap().fetch_event(cpu)
}

pub fn init(max_cpus: usize) {
    EVENT_MANAGER.call_once(|| EventManager::new(max_cpus));
}

pub fn dump_events() {
    EVENT_MANAGER.get().unwrap().dump();
}

pub fn dump_cpu_events(cpu: usize) -> Vec<usize> {
    EVENT_MANAGER.get().unwrap().dump_cpu(cpu)
}

pub fn check_events() -> bool {
    trace!("check_events");
    let cpu_data = this_cpu_data();
    match fetch_event(cpu_data.id) {
        Some(IPI_EVENT_WAKEUP) => {
            info!("cpu {} wakeup", cpu_data.id);
            cpu_data.arch_cpu.run();
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
        #[cfg(target_arch = "loongarch64")]
        Some(IPI_EVENT_CLEAR_INJECT_IRQ) => {
            irqchip::ls7a2000::clear_hwi_injected_irq();
            true
        }
        _ => false,
    }
}

pub fn send_event(cpu_id: usize, ipi_int_id: usize, event_id: usize) {
    #[cfg(target_arch = "loongarch64")]
    {
        // block until the previous event is processed, which means
        // the target queue is empty
        while !fetch_event(cpu_id).is_none() {}
        debug!(
            "loongarch64:: send_event: cpu_id: {}, ipi_int_id: {}, event_id: {}",
            cpu_id, ipi_int_id, event_id
        );
    }
    add_event(cpu_id, event_id);
    arch_send_event(cpu_id as _, ipi_int_id as _);
}
