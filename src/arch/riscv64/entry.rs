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
use crate::clear_bss;
use crate::consts::PER_CPU_SIZE;
#[no_mangle]
#[link_section = ".data"]
pub static mut CPU_BSS_LOCK: u32 = 1;

#[no_mangle]
#[link_section = ".data"]
pub static mut ENTER_CPU: u32 = u32::MAX; // the first entered cpuid will be written.

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    //a0=cpuid,a1=dtb addr
    core::arch::asm!(
        "
        la t0, __core_end        // t0 = core_end
        li t1, {per_cpu_size}    // t1 = per_cpu_size
        mul t2, a0, t1           // t2 = cpuid * per_cpu_size
        add t2, t1, t2           // t2 = cpuid * per_cpu_size+per_cpu_size
        add sp, t0, t2           // sp = core_end + cpuid * per_cpu_size + per_cpu_size

        # The first entered CPU will be stored in ENTER_CPU.
        # And the first CPU will clear the bss.

        la      t0, ENTER_CPU     # t0 = &ENTER_CPU
        la      t3, CPU_BSS_LOCK
        li      t1, -1            # t1 = initial expected value (-1)
        amoswap.w.aq t2, a0, (t0) # t2 = old value; swap a0(cpuid) into ENTER_CPU
        bne     t2, t1, 2f        # if old != -1, someone else already wrote

    0:
        la    a3, sbss           // a3 = bss's start addr
        la    a4, ebss           // a4 = bss's end addr
    1:
        blt   a4, a3, 2f         // first entered cpu clear bss
        sb    zero, 0(a3)
        addi  a3, a3, 1
        j     1b
    2:
        fence w, w
        sw    zero, 0(t3)        // clear bss done
        j     4f
    3:  
        lw    t4, 0(t3)          // wait for ENTER_CPU to clear bss
        bnez  t4, 3b
        fence r, rw

    4:
        # All CPUs could see the bss cleared.
        call {rust_main}         // a0, a1, sp are certain values
        ",
        rust_main = sym crate::rust_main,
        per_cpu_size=const PER_CPU_SIZE,
        options(noreturn),
    );
}

// global_asm!("
//     .section \".rootcfg\", \"a\"
//     .incbin \"imgs/config/qemu-riscv64.zone\"

//     // .section \".nrcfg1\", \"a\"
//     // .incbin \"imgs/config/qemu-arm64-linux-demo.zone\"
// ");

