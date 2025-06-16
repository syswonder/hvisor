use core::sync::atomic::AtomicUsize;

use alloc::sync::Weak;
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
    pub zone: Weak<RwLock<Zone>>,
    pub arch: ArchVCpu,
}

impl VCpu {
    pub fn new(zone: Weak<RwLock<Zone>>) -> Self {
        Self {
            id: free_vcpu_id(),
            zone,
            arch: ArchVCpu::new(),
        }
    }

    pub fn activate_gpm(&self) {
        unsafe {
            self.zone.upgrade().unwrap().read().gpm.activate();
        }
    }
}

pub fn current_vcpu() -> Weak<RwLock<VCpu>> {
    this_cpu_data().vcpu()
}

pub fn set_current_vcpu(cpu: Weak<RwLock<VCpu>>) {
    this_cpu_data().set_vcpu(cpu)
}

pub fn switch_to_vcpu(vcpu: Weak<RwLock<VCpu>>) {
    set_current_vcpu(vcpu.clone());
    crate::arch::vcpu::arch_switch_to_vcpu(vcpu);
}
