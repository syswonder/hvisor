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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>
//

use crate::consts::PER_CPU_SIZE;
use crate::platform::BOARD_HARTID_MAP;
use crate::platform::BOARD_NCPUS;

extern "C" {
    fn __core_end();
    fn sbss();
    fn ebss();
}

#[link_section = ".data"]
static mut CLEAR_BSS_DONE: usize = 0;

#[link_section = ".data"]
static mut ENTER_CPU: usize = usize::MAX; // i.e., -1

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    // a0=hart_id, a1=dtb_addr
    // Note: hart_id may not be continuous, we should map it to continuous cpu_id.
    core::arch::asm!(
        "
        // ================================================
        // Step1: Map hart_id to cpu_id, store cpu_id in a0
        // ================================================
        la t0, {board_hartid_map}      // t0 = &BOARD_HARTID_MAP
        li t1, {num_cpus}              // t1 = num_cpus
        li t2, 0                       // t2 = 0
    0: 
        ld t3, 0(t0)                   // t3 = BOARD_HARTID_MAP[t2]
        beq a0, t3, 1f                 // if hartid == BOARD_HARTID_MAP[t2], found
        addi t2, t2, 1                 // t2++
        addi t0, t0, 8                 // t0 += sizeof(usize)
        bltu t2, t1, 0b                // if t2 < num_cpus, continue
        j .                            // not found, hang
    1:
        mv a0, t2                      // store logical cpu_id in a0

        // ===================================
        // Step2: Prepare stack used by hvisor 
        // ===================================
        la t0, {__core_end}            // t0 = core_end
        li t1, {per_cpu_size}          // t1 = per_cpu_size
        mul t2, a0, t1                 // t2 = cpu_id * per_cpu_size
        add t2, t1, t2                 // t2 = cpu_id * per_cpu_size + per_cpu_size
        add sp, t0, t2                 // sp = core_end + cpu_id * per_cpu_size + per_cpu_size

        // ==================================================
        // Step3: Record the master CPU (first entered hart)
        // ==================================================
        la      t0, {enter_cpu}        // t0 = &ENTER_CPU
        la      t3, {clear_bss_done}   // t3 = &CLEAR_BSS_DONE
        li      t1, -1                 // t1 = -1
        amoswap.w t2, a0, (t0)         // t2 = ENTER_CPU; ENTER_CPU = a0 (atomic)
        bne     t2, t1, 5f             // only master cpu can get -1

        // ===================================
        // Step4: Master CPU clear bss section
        // ===================================
    2:
        la    a3, {sbss}              // a3 = bss's start addr
        la    a4, {ebss}              // a4 = bss's end addr
    3:
        bgeu  a3, a4, 4f              // master cpu clear bss segment
        sb    zero, 0(a3)
        addi  a3, a3, 1
        j     3b
    4:
        li    t4, 1
        fence rw, w
        sd    t4, 0(t3)               // Master cpu: clear bss done
        j     6f
    5:    
        ld    t6, 0(t3)               // Secondary cpus: wait for master cpu to clear bss
        fence r, rw
        beq   t6, zero, 5b
    6:

        // ========================================================================================
        // Step5: Jump to rust, with a0=cpu_id, a1=dtb_addr, sp=stack_addr, and bss segment cleared
        // ========================================================================================
        call {rust_main}
        ",
        rust_main = sym crate::rust_main,
        __core_end = sym __core_end,
        sbss = sym sbss,
        ebss = sym ebss,
        enter_cpu = sym ENTER_CPU,
        clear_bss_done = sym CLEAR_BSS_DONE,
        board_hartid_map = sym BOARD_HARTID_MAP,
        per_cpu_size = const PER_CPU_SIZE,
        num_cpus = const BOARD_NCPUS,
        options(noreturn),
    );
}
