use crate::arch::cpu::{self, this_cpu_id};
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use spin::Mutex;
use x86_64::instructions::tables::{lgdt, load_tss};
use x86_64::registers::segmentation::{Segment, SegmentSelector, CS};
use x86_64::structures::gdt::{Descriptor, DescriptorFlags};
use x86_64::structures::{tss::TaskStateSegment, DescriptorTablePointer};
use x86_64::{addr::VirtAddr, PrivilegeLevel};

#[repr(align(16))]
#[derive(Debug)]
pub struct GdtStruct {
    table: [u64; 16],
    tss: &'static TaskStateSegment,
}

/*lazy_static! {
    static ref TSS: Mutex<BTreeMap<usize, TaskStateSegment>> = Mutex::new(BTreeMap::new());
    static ref GDT: Mutex<BTreeMap<usize, GdtStruct>> = Mutex::new(BTreeMap::new());
}*/

impl GdtStruct {
    pub const KCODE32_SELECTOR: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);
    pub const KCODE64_SELECTOR: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);
    pub const KDATA_SELECTOR: SegmentSelector = SegmentSelector::new(3, PrivilegeLevel::Ring0);
    pub const TSS_SELECTOR: SegmentSelector = SegmentSelector::new(4, PrivilegeLevel::Ring0);

    pub fn new(tss: &'static TaskStateSegment) -> Self {
        let mut table: [u64; 16] = [0; 16];
        table[1] = DescriptorFlags::KERNEL_CODE32.bits();
        table[2] = DescriptorFlags::KERNEL_CODE64.bits();
        table[3] = DescriptorFlags::KERNEL_DATA.bits();
        if let Descriptor::SystemSegment(low, high) = Descriptor::tss_segment(&tss) {
            table[4] = low;
            table[5] = high;
        }
        Self { table, tss }
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (core::mem::size_of_val(&self.table) - 1) as u16,
        }
    }

    pub fn load(&self) {
        unsafe {
            lgdt(&self.pointer());
            CS::set_reg(GdtStruct::KCODE64_SELECTOR);
            load_tss(GdtStruct::TSS_SELECTOR);
        }
    }
}

/*fn new_static_tss() -> &'static TaskStateSegment {
    let mut boxed = Box::new(TaskStateSegment::new());
    Box::leak(boxed)
}

pub fn load_gdt_tss() {
    println!("Initializing GDT and TSS...");

    let tss = &mut TSS.lock();
    let gdt = &mut GDT.lock();
    let cpuid = this_cpu_id();

    tss.insert(cpuid, TaskStateSegment::new());
    gdt.insert(cpuid, GdtStruct::new(tss.get(&cpuid).unwrap()));
    if let Some(tss) = tss.get(&cpuid) {
        gdt.insert(cpuid, GdtStruct::new(tss));
    }
}*/
