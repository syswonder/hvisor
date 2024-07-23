use core::arch::global_asm;

use crate::consts::PER_CPU_SIZE;

global_asm!(include_str!("boot_pt.S"));

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    unsafe {
        core::arch::asm!(
            "
            // x0 = dtbaddr
            mov x18, x0
            mrs x0, mpidr_el1
            and x0, x0, #0xff
            adrp x2, __core_end          // x2 = &__core_end
            mov x3, {per_cpu_size}      // x3 = per_cpu_size
            madd x4, x0, x3, x3       // x4 = cpuid * per_cpu_size
            add x5, x2, x4
            mov sp, x5                // sp = &__core_end + (cpuid + 1) * per_cpu_size

            cmp x0, 0
            b.ne 1f
            bl {clear_bss}
            bl boot_pt_init
        1:
            bl enable_boot_pt
            mov x1, x18
            mov x18, 0
            bl {rust_main}            // x0 = cpuid, x1 = dtbaddr
            ",
            options(noreturn),
            per_cpu_size=const PER_CPU_SIZE,
            rust_main = sym crate::rust_main,
            clear_bss = sym crate::clear_bss,
        );
    }
}

#[naked]
#[no_mangle]
#[link_section = ".trampoline"]
pub unsafe extern "C" fn shutdown_el2(_gu_regs: usize) -> i32 {
    core::arch::asm!(
        "
        /* x0: guest registers */
        /* Disable mmu */
        SCTLR_FLAG2=0x00001005
        mrs	x1, sctlr_el2
        ldr	x2, =SCTLR_FLAG2
        bic	x1, x1, x2
        msr	sctlr_el2, x1
        isb

        msr	mair_el2, xzr
        msr	ttbr0_el2, xzr
        msr	tcr_el2, xzr
        isb
        /*vmreturn */
        mov	sp, x0
        ldp	x1, x0, [sp], #16	/* x1 is the exit_reason */
        ldp	x1, x2, [sp], #16
        ldp	x3, x4, [sp], #16
        ldp	x5, x6, [sp], #16
        ldp	x7, x8, [sp], #16
        ldp	x9, x10, [sp], #16
        ldp	x11, x12, [sp], #16
        ldp	x13, x14, [sp], #16
        ldp	x15, x16, [sp], #16
        ldp	x17, x18, [sp], #16
        ldp	x19, x20, [sp], #16
        ldp	x21, x22, [sp], #16
        ldp	x23, x24, [sp], #16
        ldp	x25, x26, [sp], #16
        ldp	x27, x28, [sp], #16
        ldp	x29, x30, [sp], #16
        /*now el2 sp point to per cpu stack top*/
        eret                            //ret to el2_entry hvc #0 now,depend on ELR_EL2
        
    ",
        options(noreturn),
    );
}
