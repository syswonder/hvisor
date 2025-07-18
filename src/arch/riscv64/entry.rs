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

#[no_mangle]
#[link_section = ".data"]
pub static mut CPU0_BSS_LOCK: u32 = 1;

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

        bnez  a0, 1f             // if cpuid != 0, jump to wait
        la    a3, sbss           // a3 = bss's start addr
        la    a4, ebss           // a4 = bss's end addr
    0:
        blt   a4, a3, 2f         // cpu0 clear bss
        sb    zero, 0(a3)
        addi  a3, a3, 1
        j     0b
    2:
        la    t3, CPU0_BSS_LOCK  // complete bss clear
        sw    zero, 0(t3)
        j     3f
    1:
        la    t3, CPU0_BSS_LOCK
    3:
        lw    t4, 0(t3)          // wait for CPU0 to clear bss
        bnez  t4, 3b

        fence rw, rw             // ensure all cores can see the bss cleared 

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
