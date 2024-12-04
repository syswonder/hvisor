pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {}

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub cpuid: usize,
    pub power_on: bool,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        Self {
            cpuid,
            power_on: false,
        }
    }

    pub fn reset(&mut self, entry: usize, dtb: usize) {}

    pub fn run(&mut self) -> ! {
        loop {}
    }

    pub fn idle(&mut self) -> ! {
        loop {}
    }
}

pub fn this_cpu_id() -> usize {
    0
}
