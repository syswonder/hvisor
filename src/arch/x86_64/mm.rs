use crate::{
    arch::s1pt::Stage1PageTable,
    arch::s2pt::Stage2PageTable,
    consts::PAGE_SIZE,
    error::HvResult,
    memory::{
        addr::{align_down, align_up},
        GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet, HV_PT,
    },
};

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}
