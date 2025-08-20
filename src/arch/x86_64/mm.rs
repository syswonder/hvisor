use crate::{
    arch::{acpi, boot, s1pt::Stage1PageTable, s2pt::Stage2PageTable},
    error::HvResult,
    memory::MemorySet,
};

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}

pub fn init_hv_page_table() -> HvResult {
    Ok(())
}

pub fn arch_setup_parange() {
    // x86_64 does not have a parange setup like AArch64.
    // The parange is determined by the memory regions defined in the device tree.
    // So we do not need to do anything here.
}

pub fn arch_post_heap_init(host_dtb: usize) {
    boot::module_init(host_dtb);
    acpi::root_init();
}
