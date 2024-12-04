use core::arch::global_asm;

use x86::msr::IA32_EFER;
use x86_64::registers::control::{Cr0Flags, Cr4Flags};
use x86_64::registers::model_specific::EferFlags;

const PHYS_VIRT_OFFSET: usize = 0xffff_ff80_0000_0000;
const BOOT_KERNEL_STACK_SIZE: usize = 4096 * 4;

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
    main_entry = sym crate::rust_main,
    offset = const PHYS_VIRT_OFFSET,
    boot_stack_size = const BOOT_KERNEL_STACK_SIZE,
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
        jmp entry32
        ",
        options(noreturn),
    );
}
