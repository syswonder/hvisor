use loongArch64::register::cpuid;

use crate::percpu::this_cpu_data;
use crate::{
    consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE},
    memory::VirtAddr,
};

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub r: [usize; 32], // r0~r31
    pub sepc: usize,
    pub stack_top: usize,
    pub cpuid: usize,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        ArchCpu {
            r: [0; 32],
            sepc: 0,
            stack_top: 0,
            cpuid,
        }
    }
    pub fn get_cpuid(&self) -> usize {
        self.cpuid
    }
    pub fn stack_top(&self) -> VirtAddr {
        PER_CPU_ARRAY_PTR as VirtAddr + (self.get_cpuid() + 1) as usize * PER_CPU_SIZE - 8
    }
    pub fn init(&mut self, entry: usize, cpu_id: usize, dtb: usize) {
        todo!("loongarch64 archcpu init");
    }
    pub fn run(&self) {
        todo!("loongarch64 archcpu run");
    }
    pub fn idle(&self) {
        todo!("loongarch64 archcpu idle");
    }
}

pub fn this_cpu_id() -> usize {
    cpuid::read().core_id()
}
