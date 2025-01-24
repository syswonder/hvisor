use crate::{
    arch::s1pt::Stage1PageTable, arch::s2pt::Stage2PageTable, error::HvResult, memory::MemorySet,
    platform::qemu_x86_64::*,
};

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(4)
}

pub fn init_hv_page_table() -> HvResult {
    use x86_64::structures::paging::{PageTable, PageTableFlags as PTF};
    let pt1 = unsafe { &mut *(gpa_as_mut_ptr(GUEST_PT1) as *mut PageTable) };
    let pt2 = unsafe { &mut *(gpa_as_mut_ptr(GUEST_PT2) as *mut PageTable) };
    // identity mapping
    pt1[0].set_addr(
        x86_64::PhysAddr::new(GUEST_PT2 as _),
        PTF::PRESENT | PTF::WRITABLE,
    );
    pt2[0].set_addr(
        x86_64::PhysAddr::new(0),
        PTF::PRESENT | PTF::WRITABLE | PTF::HUGE_PAGE,
    );

    unsafe {
        core::ptr::copy_nonoverlapping(
            crate::platform::qemu_x86_64::test_guest_2 as usize as *const u8,
            gpa_as_mut_ptr(GUEST_ENTRY),
            0x1000,
        );
    }

    Ok(())
}
