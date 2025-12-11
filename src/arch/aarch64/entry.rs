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
use crate::consts::PER_CPU_SIZE;
use crate::platform::BOARD_MPIDR_MAPPINGS;

const INVALID_CPUID: usize = (-1) as _;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> ! {
    unsafe {
        core::arch::asm!(
            "
            // x0 = dtbaddr
            mov x18, x0

            /* Insert nop instruction to ensure byte at offset 10 in hvisor binary is non-zero.
            * Rockchip U-Boot (arch_preboot_os@arch/arm/mach-rockchip/board.c:670) performs 
            * forced relocation if this byte is zero, causing boot failure. This padding
            * prevents unintended relocation by maintaining non-zero value at this critical
            * offset in the binary layout. */

            nop
            nop
            bl {boot_cpuid_get}        // x17 = cpuid

            adrp x2, __core_end        // x2 = &__core_end
            mov x3, {per_cpu_size}     // x3 = per_cpu_size
            madd x4, x19, x3, x3       // x4 = cpuid * per_cpu_size
            add x5, x2, x4
            mov sp, x5                 // sp = &__core_end + (cpuid + 1) * per_cpu_size

            // disable cache and MMU
            mrs x1, sctlr_el2
            bic x1, x1, #0xf
            msr sctlr_el2, x1

            // cache_invalidate(0): clear dl1$
            mov x0, #0
            bl  {cache_invalidate}

            ic  iallu

            cmp x19, 0
            b.ne 1f

            // if (cpu_id == 0) cache_invalidate(2): clear l2$
            mov x0, #2
            bl  {cache_invalidate}

            // ic  iallu

            bl {clear_bss}
            bl {boot_pt_init}
        1:
            bl {mmu_enable}

            mov x1, x18
            mov x0, x19
            mov x18, #0
            mov x19, #0
            bl {rust_main}            // x0 = cpuid, x1 = dtbaddr
            ",
            options(noreturn),
            boot_cpuid_get = sym boot_cpuid_get,
            cache_invalidate = sym cache_invalidate,
            per_cpu_size = const PER_CPU_SIZE,
            rust_main = sym crate::rust_main,
            clear_bss = sym crate::clear_bss,
            boot_pt_init = sym super::mmu::boot_pt_init,
            mmu_enable = sym super::mmu::mmu_enable,
        );
    }
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn boot_cpuid_get() {
    use crate::arch::cpu;

    core::arch::asm!(
        "
        mrs x19, mpidr_el1
        ldr x2, ={mpidr_mask}
        and x19, x19, x2
        adr x2, {mpidr_mappings}
        mov x4, #0
    1:
        // search for the mpidr_el1 mapping in BOARD_MPIDR_MAPPINGS.
        ldr x3, [x2]
        cmp x19, x3
        b.eq 3f
        add x2, x2, #8
        add x4, x4, #1
        cmp x4, {ncpus}
        b.ne 1b
    2: 
        // failed to get cpuid, return an invalid id, and spin in an infinite loop.
        mov x19, {inv_id}
        wfi
        b 2b
    3:
        // found cpuid, return it.
        mov x19, x4
        ret
    ",
        mpidr_mask = const cpu::MPIDR_MASK,
        mpidr_mappings = sym BOARD_MPIDR_MAPPINGS,
        ncpus = const crate::consts::MAX_CPU_NUM,
        inv_id = const INVALID_CPUID,
        options(noreturn)
    )
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
