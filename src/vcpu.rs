use core::sync::atomic::AtomicUsize;

use alloc::sync::Arc;
use spin::RwLock;

use crate::arch::vcpu::ArchVCpu;
use crate::percpu::this_cpu_data;
use crate::zone::Zone;

static VCPU_ID: AtomicUsize = AtomicUsize::new(0);

fn free_vcpu_id() -> usize {
    VCPU_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

pub struct VCpu {
    pub id: usize,
    pub zone: Arc<RwLock<Zone>>,
    pub arch: ArchVCpu,
}

impl VCpu {
    pub fn new(zone: Arc<RwLock<Zone>>) -> Self {
        Self {
            id: free_vcpu_id(),
            zone,
            arch: ArchVCpu::new(),
        }
    }

    pub fn activate_gpm(&self) {
        unsafe {
            self.zone.read().gpm.activate();
        }
    }
}

pub fn current_vcpu() -> Arc<VCpu> {
    this_cpu_data().vcpu().unwrap()
}

pub fn set_current_vcpu(cpu: Arc<VCpu>) {
    this_cpu_data().set_vcpu(cpu)
}

pub fn switch_to_vcpu(vcpu: Arc<VCpu>) {
    set_current_vcpu(vcpu.clone());
    crate::arch::vcpu::arch_switch_to_vcpu(vcpu);
}
