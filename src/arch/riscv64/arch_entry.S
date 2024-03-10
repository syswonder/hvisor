    .section .text.entry
    .globl arch_entry
arch_entry:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:

.global __rootcfg
__rootcfg: