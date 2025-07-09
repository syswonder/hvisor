use crate::{
    arch::{boot, cpu::this_cpu_id, graphics::font_init},
    consts::PER_CPU_SIZE,
    memory::addr::PHYS_VIRT_OFFSET,
    platform::__board,
    rust_main,
};
use core::arch::global_asm;
use x86::msr::IA32_EFER;
use x86_64::registers::{
    control::{Cr0Flags, Cr4Flags},
    model_specific::EferFlags,
};

const MULTIBOOT_HEADER_MAGIC: u32 = 0x1bad_b002;
const MULTIBOOT_HEADER_FLAGS: u32 = 0x0001_0002;
const MULTIBOOT2_HEADER_MAGIC: u32 = 0xe852_50d6;
const MULTIBOOT2_ARCH_I386: u32 = 0;
const X86_PHYS_VIRT_OFFSET: usize = 0xffff_ff80_0000_0000;

const CR0: u64 = Cr0Flags::PROTECTED_MODE_ENABLE.bits()
    | Cr0Flags::MONITOR_COPROCESSOR.bits()
    | Cr0Flags::TASK_SWITCHED.bits()
    | Cr0Flags::NUMERIC_ERROR.bits()
    | Cr0Flags::WRITE_PROTECT.bits()
    | Cr0Flags::PAGING.bits();
const CR4: u64 = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits() | Cr4Flags::PAGE_GLOBAL.bits();
const EFER: u64 = EferFlags::LONG_MODE_ENABLE.bits() | EferFlags::NO_EXECUTE_ENABLE.bits();

global_asm!(
    include_str!("multiboot.S"),
    multiboot_header_magic = const MULTIBOOT_HEADER_MAGIC,
    multiboot_header_flags = const MULTIBOOT_HEADER_FLAGS,
    multiboot2_header_magic = const MULTIBOOT2_HEADER_MAGIC,
    multiboot2_arch_i386 = const MULTIBOOT2_ARCH_I386,
    rust_entry = sym rust_entry,
    rust_entry_secondary = sym rust_entry_secondary,
    offset = const X86_PHYS_VIRT_OFFSET,
    per_cpu_size = const PER_CPU_SIZE,
    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const IA32_EFER,
    efer = const EFER,
);

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    core::arch::asm!(
        "
        .code32
        cli
        mov edi, eax    // magic
        mov esi, ebx    // multiboot info
        jmp bsp_entry32
        ",
        options(noreturn),
    );
}

extern "C" fn rust_entry(magic: u32, info_addr: usize) {
    unsafe { fill_page_table() };
    crate::clear_bss();
    unsafe { PHYS_VIRT_OFFSET = X86_PHYS_VIRT_OFFSET };
    boot::multiboot_init(info_addr);
    #[cfg(all(feature = "graphics", target_arch = "x86_64"))]
    font_init(__board::GRAPHICS_FONT);
    boot::print_memory_map();
    rust_main(this_cpu_id(), info_addr);
}

fn rust_entry_secondary() {
    // println!("CPUID: {}", this_cpu_id());
    rust_main(this_cpu_id(), 0);
}

extern "C" {
    #[link_name = "Ltmp_pdpt_low"]
    static mut PDPT_LOW: [u64; 512];
    #[link_name = "Ltmp_pdpt_high"]
    static mut PDPT_HIGH: [u64; 512];
}

unsafe fn fill_page_table() {
    let mut addr: usize = 0;
    for i in 0..512 {
        // paddr | PRESENT | WRITABLE | HUGE_PAGE
        PDPT_LOW[i] = (addr | 0x83) as _;
        PDPT_HIGH[i] = (addr | 0x83) as _;
        addr += 0x4000_0000;
    }
}
