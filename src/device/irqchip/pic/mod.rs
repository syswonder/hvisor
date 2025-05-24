pub mod ioapic;
pub mod lapic;

use crate::{
    arch::{acpi, cpu::this_cpu_id, idt, ipi, vmcs::Vmcs, vtd},
    consts::{MAX_CPU_NUM, MAX_ZONE_NUM},
    zone::Zone,
};
use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use core::arch::asm;
use ioapic::ioapic_inject_irq;
use spin::{Mutex, Once};

static PENDING_VECTORS: Once<PendingVectors> = Once::new();

struct InnerPendingVectors {
    pub queue: VecDeque<(u8, Option<u32>)>,
    pub has_eoi: bool,
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
            });
            vs.push(v);
        }
        Self { inner: vs }
    }

    fn add_vector(&self, cpu_id: usize, vector: u8, err_code: Option<u32>, allow_repeat: bool) {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();
        if vectors.queue.len() > 10 {
            warn!("too many pending vectors! cnt: {:x?}", vectors.queue.len());
        }
        if allow_repeat || !vectors.queue.contains(&(vector, err_code)) {
            vectors.queue.push_back((vector, err_code));
        }
    }

    fn check_pending_vectors(&self, cpu_id: usize) -> bool {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();

        if let Some(vector) = vectors.queue.front() {
            let allow_interrupt = Vmcs::allow_interrupt().unwrap() && vectors.has_eoi;
            if vector.0 < 32 || allow_interrupt {
                if vectors.queue.len() > 10 {
                    warn!("too many pending vectors!");
                }
                // if it's an exception, or an interrupt that is not blocked, inject it directly.
                Vmcs::inject_interrupt(vector.0, vector.1).unwrap();
                vectors.has_eoi = false;
                vectors.queue.pop_front();
                return true;
            } else if vectors.has_eoi {
                // interrupts are blocked, enable interrupt-window exiting.
                Vmcs::set_interrupt_window(true).unwrap();
            }
        }
        false
    }

    fn pop_vector(&self, cpu_id: usize) {
        let mut vectors = self.inner.get(cpu_id).unwrap().lock();
        vectors.has_eoi = true;
    }
}

pub fn inject_vector(cpu_id: usize, vector: u8, err_code: Option<u32>, allow_repeat: bool) {
    PENDING_VECTORS
        .get()
        .unwrap()
        .add_vector(cpu_id, vector, err_code, allow_repeat);
    if cpu_id != this_cpu_id() {
        // wake up dest
        ipi::arch_send_event(cpu_id as _, 0);
    }
}

pub fn check_pending_vectors(cpu_id: usize) -> bool {
    PENDING_VECTORS.get().unwrap().check_pending_vectors(cpu_id)
}

pub fn pop_vector(cpu_id: usize) {
    PENDING_VECTORS.get().unwrap().pop_vector(cpu_id);
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
    idt::init(MAX_ZONE_NUM);
    ioapic::init_virt_ioapic(MAX_ZONE_NUM);
    acpi::root_init();
    vtd::init();
}

pub fn primary_init_late() {}

impl Zone {
    pub fn arch_irqchip_reset(&self) {}
}
