use crate::arch::gdt::GdtStruct;
use crate::arch::lapic::{busy_wait, local_apic};
use crate::consts::{core_end, PER_CPU_SIZE};
use crate::memory::{addr::phys_to_virt, PhysAddr, PAGE_SIZE};
use alloc::boxed::Box;
use core::arch::global_asm;
use core::time::Duration;
use raw_cpuid::CpuId;
use x86_64::structures::tss::TaskStateSegment;

const AP_START_PAGE_IDX: u8 = 6;
const AP_START_PAGE_PADDR: PhysAddr = AP_START_PAGE_IDX as usize * PAGE_SIZE;

global_asm!(
    include_str!("ap_start.S"),
    ap_start_page_paddr = const AP_START_PAGE_PADDR,
);

unsafe fn setup_ap_start_page(cpuid: usize) {
    extern "C" {
        fn ap_start16();
        fn ap_end();
        fn ap_entry32();
    }
    const U64_PER_PAGE: usize = PAGE_SIZE / 8;

    let ap_start_page_ptr = phys_to_virt(AP_START_PAGE_PADDR) as *mut usize;
    let ap_start_page = core::slice::from_raw_parts_mut(ap_start_page_ptr, U64_PER_PAGE);
    core::ptr::copy_nonoverlapping(
        ap_start16 as *const usize,
        ap_start_page_ptr,
        (ap_end as usize - ap_start16 as usize) / 8,
    );
    ap_start_page[U64_PER_PAGE - 2] = core_end() as usize + (cpuid + 1) * PER_CPU_SIZE;
    ap_start_page[U64_PER_PAGE - 1] = ap_entry32 as usize;
}

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    unsafe { setup_ap_start_page(cpuid) };

    let lapic = local_apic();

    // Intel SDM Vol 3C, Section 8.4.4, MP Initialization Example
    unsafe { lapic.send_init_ipi(cpuid as u32) };
    busy_wait(Duration::from_millis(10)); // 10ms
    unsafe { lapic.send_sipi(AP_START_PAGE_IDX, cpuid as u32) };
    busy_wait(Duration::from_micros(200)); // 200us
    unsafe { lapic.send_sipi(AP_START_PAGE_IDX, cpuid as u32) };
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // pushed by 'trap.S'
    pub vector: u64,
    pub error_code: u64,

    // pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    pub cpuid: usize,
    pub power_on: bool,
    pub gdt: GdtStruct,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        let boxed = Box::new(TaskStateSegment::new());
        let tss = Box::leak(boxed);
        Self {
            cpuid,
            power_on: false,
            gdt: GdtStruct::new(tss),
        }
    }

    pub fn per_cpu_init(&'static self) {
        self.gdt.load();
        self.gdt.load_tss();
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
    match CpuId::new().get_feature_info() {
        Some(info) => info.initial_local_apic_id() as usize,
        None => 0,
    }
}
