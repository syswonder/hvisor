use crate::{
    arch::s1pt::Stage1PageTable, arch::s2pt::Stage2PageTable, error::HvResult, memory::MemorySet,
    platform::qemu_x86_64::*,
};

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}

pub fn init_hv_page_table() -> HvResult {
    Ok(())
}
