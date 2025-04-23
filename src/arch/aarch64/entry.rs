// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
use core::arch::global_asm;

use crate::consts::PER_CPU_SIZE;

//global_asm!(include_str!("boot_pt.S"));

#[cfg(feature = "a55")]
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    unsafe {
        core::arch::asm!(
            "
            // x0 = dtbaddr
            mov x18, x0
            mrs x17, mpidr_el1
            lsr x17, x17, #0x8
            and x17, x17, #0xff
            adrp x2, __core_end          // x2 = &__core_end
            mov x3, {per_cpu_size}      // x3 = per_cpu_size
            madd x4, x17, x3, x3       // x4 = cpuid * per_cpu_size
            add x5, x2, x4
            mov sp, x5                // sp = &__core_end + (cpuid + 1) * per_cpu_size

            // disable cache and MMU
            mrs x1, sctlr_el2
            bic x1, x1, #0xf
            msr sctlr_el2, x1

            // cache_invalidate(0): clear dl1$
            mov x0, #0
            bl  {cache_invalidate}

            ic  iallu

            cmp x17, 0
            b.ne 1f

            // if (cpu_id == 0) cache_invalidate(2): clear l2$
            mov x0, #2
            bl  {cache_invalidate}

            // ic  iallu

            bl {clear_bss}

            //bl boot_pt_init
            adrp x0, {BOOT_PT_L0}
            adrp x1, {BOOT_PT_L1}
            bl {boot_pt_init}
        1:
            adrp x0, {BOOT_PT_L0}
            bl {mmu_init}
            bl {mmu_enable}

            tlbi alle2
            dsb	nsh
            isb

            mov x1, x18
            mov x0, x17
            mov x18, 0
            mov x17, 0
            bl {rust_main}            // x0 = cpuid, x1 = dtbaddr
            ",
            options(noreturn),
            cache_invalidate = sym cache_invalidate,
            per_cpu_size=const PER_CPU_SIZE,
            rust_main = sym crate::rust_main,
            clear_bss = sym crate::clear_bss,
            BOOT_PT_L0 = sym super::mmu::BOOT_PT_L0,
            BOOT_PT_L1 = sym super::mmu::BOOT_PT_L1,
            boot_pt_init = sym super::mmu::boot_pt_init,
            mmu_init = sym super::mmu::mmu_init,
            mmu_enable = sym super::mmu::mmu_enable,
            // boot_cpuid_get = sym boot_cpuid_get,
        );
    }
}

#[cfg(not(feature = "a55"))]
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    unsafe {
        core::arch::asm!(
            "
            // x0 = dtbaddr
            mov x18, x0
            mrs x17, mpidr_el1
            and x17, x17, #0xff
            adrp x2, __core_end          // x2 = &__core_end
            mov x3, {per_cpu_size}      // x3 = per_cpu_size
            madd x4, x17, x3, x3       // x4 = cpuid * per_cpu_size
            add x5, x2, x4
            mov sp, x5                // sp = &__core_end + (cpuid + 1) * per_cpu_size

            // disable cache and MMU
            mrs x1, sctlr_el2
            bic x1, x1, #0xf
            msr sctlr_el2, x1

            // cache_invalidate(0): clear dl1$
            mov x0, #0
            bl  {cache_invalidate}

            ic  iallu

            cmp x17, 0
            b.ne 1f
           // if (cpu_id == 0) cache_invalidate(2): clear l2$
            mov x0, #2
            bl  {cache_invalidate}

            // ic  iallu

            bl {clear_bss}

            cmp x17, 0
            b.ne 1f

            bl {clear_bss}
            //bl boot_pt_init
            adrp x0, {BOOT_PT_L0}
            adrp x1, {BOOT_PT_L1}
            bl {boot_pt_init}
        1:
            adrp x0, {BOOT_PT_L0}
            bl {mmu_init}
            bl {mmu_enable}

            tlbi alle2
            dsb	nsh
            isb

            mov x1, x18
            mov x0, x17
            mov x18, 0
            mov x17, 0
            bl {rust_main}            // x0 = cpuid, x1 = dtbaddr
            ",
            options(noreturn),
            cache_invalidate = sym cache_invalidate,
            per_cpu_size=const PER_CPU_SIZE,
            rust_main = sym crate::rust_main,
            clear_bss = sym crate::clear_bss,
            BOOT_PT_L0 = sym super::mmu::BOOT_PT_L0,
            BOOT_PT_L1 = sym super::mmu::BOOT_PT_L1,
            boot_pt_init = sym super::mmu::boot_pt_init,
            mmu_init = sym super::mmu::mmu_init,
            mmu_enable = sym super::mmu::mmu_enable,
            // boot_cpuid_get = sym boot_cpuid_get,
        );
    }
}

// #[naked]
// #[no_mangle]
// #[link_section = ".text.entry"]
// pub unsafe extern "C" fn boot_cpuid_get() {
//     core::arch::asm!("
//         mrs x17, mpidr_el1
//         lsr x17, x17, #0x8
//         and x17, x17, #0xff
//         ret
//     "
//     ,options(noreturn)
//     )
// }
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

unsafe extern "C" fn cache_invalidate(cache_level: usize) {
    core::arch::asm!(
        r#"
        msr csselr_el1, {0}
        mrs x4, ccsidr_el1 // read cache size id.
        and x1, x4, #0x7
        add x1, x1, #0x4 // x1 = cache line size.
        ldr x3, =0x7fff
        and x2, x3, x4, lsr #13 // x2 = cache set number - 1.
        ldr x3, =0x3ff
        and x3, x3, x4, lsr #3 // x3 = cache associativity number - 1.
        clz w4, w3 // x4 = way position in the cisw instruction.
        mov x5, #0 // x5 = way counter way_loop.
    // way_loop:
    1:
        mov x6, #0 // x6 = set counter set_loop.
    // set_loop:
    2:
        lsl x7, x5, x4
        orr x7, {0}, x7 // set way.
        lsl x8, x6, x1  
        orr x7, x7, x8 // set set.
        dc cisw, x7 // clean and invalidate cache line.
        add x6, x6, #1 // increment set counter.
        cmp x6, x2 // last set reached yet?
        ble 2b // if not, iterate set_loop,
        add x5, x5, #1 // else, next way.
        cmp x5, x3 // last way reached yet?
        ble 1b // if not, iterate way_loop
        "#,
        in(reg) cache_level,
        options(nostack)
    );
}