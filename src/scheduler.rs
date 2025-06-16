use crate::vcpu::{switch_to_vcpu, VCpu};
use alloc::{sync::Weak, vec::Vec};
use spin::{Once, RwLock};

struct SchedulerInner {
    vcpus: Vec<Weak<RwLock<VCpu>>>,
}
pub struct Scheduler {
    inner: RwLock<SchedulerInner>,
}

static SCHEDULER: Once<Scheduler> = Once::new();

impl SchedulerInner {
    fn new() -> Self {
        Self { vcpus: Vec::new() }
    }

    fn add_vcpu(&mut self, vcpu: Weak<RwLock<VCpu>>) {
        self.vcpus.push(vcpu);
    }

    fn run_next(&mut self) {
        if let Some(v) = self.vcpus.pop() {
            match v.upgrade() {
                Some(_) => switch_to_vcpu(v),
                None => todo!("Weak reference expired..."),
            }
        } else {
            todo!("No vcpu available...")
        }
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(SchedulerInner::new()),
        }
    }

    pub fn add_vcpu(&self, vcpu: Weak<RwLock<VCpu>>) {
        let mut inner = self.inner.write();
        inner.add_vcpu(vcpu);
    }

    pub fn run_next(&self) {
        let mut inner = self.inner.write();
        inner.run_next();
    }
}

pub fn init() {
    SCHEDULER.call_once(|| Scheduler::new());
}

fn scheduler() -> &'static Scheduler {
    SCHEDULER.get().unwrap()
}

pub fn add_vcpu(vcpu: Weak<RwLock<VCpu>>) {
    scheduler().add_vcpu(vcpu);
}

pub fn run_next() {
    scheduler().run_next();
}
