pub mod ioapic;
pub mod lapic;

use crate::{
    arch::{acpi, idt, ipi, vmcs::Vmcs, vtd},
    consts::{MAX_CPU_NUM, MAX_ZONE_NUM},
    zone::Zone,
};
use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use core::arch::asm;
use ioapic::ioapic_inject_irq;
use spin::{Mutex, Once};

static PENDING_VECTORS: Once<PendingVectors> = Once::new();

struct PendingVectors {
    inner: Vec<Mutex<VecDeque<(u8, Option<u32>)>>>,
}

impl PendingVectors {
    fn new(max_cpus: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_cpus {
            let v = Mutex::new(VecDeque::new());
            vs.push(v)
        }
        Self { inner: vs }
    }

    fn add_vector(&self, cpu_id: usize, vector: u8, err_code: Option<u32>, allow_repeat: bool) {
        match self.inner.get(cpu_id) {
            Some(pending_vectors) => {
                let mut vectors = pending_vectors.lock();
                /*if vectors.len() > 2 {
                    info!("len: {:x}", vectors.len());
                }*/
                if allow_repeat || !vectors.contains(&(vector, err_code)) {
                    vectors.push_back((vector, err_code));
                }
            }
            _ => {}
        }
    }

    fn check_pending_vectors(&self, cpu_id: usize) {
        match self.inner.get(cpu_id) {
            Some(pending_vectors) => {
                let mut vectors = pending_vectors.lock();
                if let Some(vector) = vectors.front() {
                    let allow_interrupt = Vmcs::allow_interrupt().unwrap();
                    if vector.0 < 32 || allow_interrupt {
                        if vectors.len() == 10 {
                            warn!("too many pending vectors!");
                        }
                        // if it's an exception, or an interrupt that is not blocked, inject it directly.
                        Vmcs::inject_interrupt(vector.0, vector.1).unwrap();
                        vectors.pop_front();
                    } else {
                        // interrupts are blocked, enable interrupt-window exiting.
                        Vmcs::set_interrupt_window(true).unwrap();
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn inject_vector(cpu_id: usize, vector: u8, err_code: Option<u32>, allow_repeat: bool) {
    PENDING_VECTORS
        .get()
        .unwrap()
        .add_vector(cpu_id, vector, err_code, allow_repeat);
}

pub fn check_pending_vectors(cpu_id: usize) {
    PENDING_VECTORS.get().unwrap().check_pending_vectors(cpu_id);
}

pub fn enable_irq() {
    unsafe { asm!("sti") };
}

pub fn disable_irq() {
    unsafe { asm!("cli") };
}

pub fn inject_irq(_irq: usize, _is_hardware: bool) {
    ioapic_inject_irq(_irq as _);
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
