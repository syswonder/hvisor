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
//  Solicey <lzoi_lth@163.com>

pub mod ioapic;
mod ioapic_state;
pub mod lapic;

use crate::{
    arch::{acpi, cpu::this_cpu_id, idt, ipi, msr, pio, vmcs::Vmcs},
    consts::{MAX_CPU_NUM, MAX_ZONE_NUM},
    device::iommu,
    zone::Zone,
};
use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use core::arch::asm;
use ioapic::ioapic_inject_irq;
use spin::{Mutex, Once};

static PENDING_VECTORS: Once<PendingVectors> = Once::new();

struct InnerPendingVectors {
    pub queue: VecDeque<(u8, Option<u32>, bool)>,
    pub has_eoi: bool,
    pub in_service: Option<(u8, bool)>,
}

struct PendingVectors {
    inner: Vec<Mutex<InnerPendingVectors>>,
}

impl PendingVectors {
    fn new(max_cpus: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_cpus {
            let v = Mutex::new(InnerPendingVectors {
                queue: VecDeque::new(),
                has_eoi: true,
                in_service: None,
            });
            vs.push(v);
        }
        Self { inner: vs }
    }

    fn add_vector(
        &self,
        cpu_id: usize,
        vector: u8,
        err_code: Option<u32>,
        allow_repeat: bool,
        level: bool,
    ) {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();
        if vectors.queue.len() > 10 {
            warn!("too many pending vectors! cnt: {:x?}", vectors.queue.len());
        }
        if allow_repeat || !vectors.queue.contains(&(vector, err_code, level)) {
            vectors.queue.push_back((vector, err_code, level));
        }
    }

    fn check_pending_vectors(&self, cpu_id: usize) -> bool {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();

        if let Some(&vector) = vectors.queue.front() {
            let allow_interrupt = Vmcs::allow_interrupt().unwrap() && vectors.has_eoi;
            if vector.0 < 32 || allow_interrupt {
                if vectors.queue.len() > 10 {
                    warn!("too many pending vectors!");
                }
                // if it's an exception, or an interrupt that is not blocked, inject it directly.
                Vmcs::inject_interrupt(vector.0, vector.1).unwrap();
                if vector.0 >= 32 {
                    vectors.has_eoi = false;
                    vectors.in_service = Some((vector.0, vector.2));
                }
                vectors.queue.pop_front();
                return true;
            } else if vectors.has_eoi {
                // interrupts are blocked, enable interrupt-window exiting.
                Vmcs::set_interrupt_window(true).unwrap();
            }
        }
        false
    }

    fn pop_vector(&self, cpu_id: usize) -> Option<u8> {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();
        vectors.has_eoi = true;
        vectors.in_service.take().map(|(vector, _)| vector)
    }

    fn tmr(&self, cpu_id: usize, register: usize) -> u32 {
        let vectors = self.inner.get(cpu_id).unwrap().lock();
        let Some((vector, true)) = vectors.in_service else {
            return 0;
        };
        if vector as usize / 32 == register {
            1 << (vector % 32)
        } else {
            0
        }
    }

    fn isr(&self, cpu_id: usize, register: usize) -> u32 {
        let vectors = self.inner.get(cpu_id).unwrap().lock();
        let Some((vector, _)) = vectors.in_service else {
            return 0;
        };
        if vector as usize / 32 == register {
            1 << (vector % 32)
        } else {
            0
        }
    }

    fn clear_vectors(&self, cpu_id: usize) {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();
        vectors.queue.clear();
        vectors.has_eoi = true;
        vectors.in_service = None;
    }
}

pub fn inject_vector(
    cpu_id: usize,
    vector: u8,
    err_code: Option<u32>,
    allow_repeat: bool,
    level: bool,
) {
    PENDING_VECTORS
        .get()
        .unwrap()
        .add_vector(cpu_id, vector, err_code, allow_repeat, level);
    if cpu_id != this_cpu_id() {
        // wake up dest
        ipi::arch_send_event(cpu_id as _, 0);
    }
}

pub fn check_pending_vectors(cpu_id: usize) -> bool {
    PENDING_VECTORS.get().unwrap().check_pending_vectors(cpu_id)
}

pub fn pop_vector(cpu_id: usize) -> Option<u8> {
    PENDING_VECTORS.get().unwrap().pop_vector(cpu_id)
}

pub fn tmr(cpu_id: usize, register: usize) -> u32 {
    PENDING_VECTORS.get().unwrap().tmr(cpu_id, register)
}

pub fn isr(cpu_id: usize, register: usize) -> u32 {
    PENDING_VECTORS.get().unwrap().isr(cpu_id, register)
}

pub fn clear_vectors(cpu_id: usize) {
    PENDING_VECTORS.get().unwrap().clear_vectors(cpu_id);
}

pub fn enable_irq() {
    unsafe { asm!("sti") };
}

pub fn disable_irq() {
    unsafe { asm!("cli") };
}

pub fn inject_irq(_irq: usize, allow_repeat: bool) {
    ioapic_inject_irq(_irq as _, allow_repeat);
}

pub fn percpu_init() {}

pub fn primary_init_early() {
    ipi::init(MAX_CPU_NUM);
    PENDING_VECTORS.call_once(|| PendingVectors::new(MAX_CPU_NUM));
    ioapic::init_ioapic();
    ioapic::init_virt_ioapic(MAX_ZONE_NUM);
    msr::init_msr_bitmap_map();
    pio::init_pio_bitmap_map();
}

pub fn primary_init_late() {}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        #[cfg(intel_vtd)]
        iommu::clear_dma_translation_tables(self.id());
    }
}
