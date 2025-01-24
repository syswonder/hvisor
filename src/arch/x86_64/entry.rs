use crate::{
    arch::cpu::this_cpu_id, consts::PER_CPU_SIZE, memory::addr::PHYS_VIRT_OFFSET, rust_main,
};
use core::arch::global_asm;
use x86::msr::IA32_EFER;
use x86_64::registers::{
    control::{Cr0Flags, Cr4Flags},
    model_specific::EferFlags,
};

const MULTIBOOT_HEADER_MAGIC: i32 = 0x1BADB002;
const MULTIBOOT_HEADER_FLAGS: i32 = 0x00010002;
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
        mov edi, eax    // magic
        mov esi, ebx    // multiboot info
        jmp bsp_entry32
        ",
        options(noreturn),
    );
}

fn rust_entry() {
    crate::clear_bss();
    unsafe { PHYS_VIRT_OFFSET = X86_PHYS_VIRT_OFFSET };
    println!("");
    rust_main(this_cpu_id(), 0);
}

fn rust_entry_secondary() {
    // println!("CPUID: {}", this_cpu_id());
    rust_main(this_cpu_id(), 0);
}
