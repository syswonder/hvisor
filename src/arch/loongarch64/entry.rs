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
//      Yulong Han <wheatfox17@icloud.com>
//
use crate::consts::PER_CPU_SIZE;

const DMW_DA_BITS: usize = 48;
const CSR_DMW0_PLV0: usize = 1 << 0;
const CSR_DMW0_VSEG: usize = 0x8000;
const CSR_DMW0_BASE: usize = CSR_DMW0_VSEG << DMW_DA_BITS;
const CSR_DMW0_INIT: usize = CSR_DMW0_BASE | CSR_DMW0_PLV0;

const CSR_DMW1_PLV0: usize = 1 << 0;
const CSR_DMW1_MAT: usize = 1 << 4;
const CSR_DMW1_VSEG: usize = 0x9000;
const CSR_DMW1_BASE: usize = CSR_DMW1_VSEG << DMW_DA_BITS;
const CSR_DMW1_INIT: usize = CSR_DMW1_BASE | CSR_DMW1_PLV0 | CSR_DMW1_MAT;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    // a0/r4: CPU_ID read from CSR 0x20 CPUID

    // .macro JUMP_VIRT_ADDR temp1 temp2 (r12, r13)
    //      li.d	\temp1, CACHE_BASE (0x9000_0000_0000_0000)
    //      pcaddi	\temp2, 0
    //      or		\temp1, \temp1, \temp2
    //      jirl    zero, \temp1, 0xc // 0xc is beacuse the above pcaddi + 0xc will jump to exacly the next instruction after jirl - wheatfox
    // .endm
    core::arch::asm!(
        "
        0:
            li.d        $r12, {CSR_DMW0_INIT} // 0x8
            csrwr       $r12, {LOONGARCH_CSR_DMW0}
            li.d        $r12, {CSR_DMW1_INIT} // 0x9
            csrwr       $r12, {LOONGARCH_CSR_DMW1}

            // first JUMP_VIRT_ADDR
            li.d        $r12, {CSR_DMW1_BASE}
            pcaddi      $r13, 0
            or          $r12, $r12, $r13
            jirl        $zero, $r12, 0xc
            // end of JUMP_VIRT_ADDR

            li.w		$r12, 0xb0		    // PLV=0, IE=0, PG=1
            csrwr		$r12, {LOONGARCH_CSR_CRMD}
            li.w		$r12, 0x04		    // PLV=0, PIE=1, PWE=0
            csrwr		$r12, {LOONGARCH_CSR_PRMD}
            li.w		$r12, 0x00		    // FPE=0, SXE=0, ASXE=0, BTE=0
            csrwr		$r12, {LOONGARCH_CSR_EUEN}

            csrrd       $r4, {CSR_CPUID}
            la.pcrel    $r12, __core_end
            li.d        $r13, {per_cpu_size}
            mul.d       $r14, $r4, $r13
            add.d       $r14, $r13, $r14
            add.d       $r15, $r12, $r14    // this is the stack top of CPU[CPU_ID]
            addi.d      $sp, $r15, 0        // set sp
        //     la.pcrel    $r12, sbss          // bss start, t0 as ptr
        //     la.pcrel    $r13, ebss          // bss end
        // 1:
        //     st.d        $zero, $r12, 0      // MEM[ptr] = 0
        //     beq         $r12, $r13, 2f      // break if ptr == ebss, and we just set MEM[ebss] = 0
        //     addi.d      $r12, $r12, 8       // ptr += 8
        //     b           1b
        2:
            ibar        0
            dbar        0
            bl          {rust_main}
        ",
        CSR_DMW0_INIT = const CSR_DMW0_INIT,
        CSR_DMW1_INIT = const CSR_DMW1_INIT,
        LOONGARCH_CSR_CRMD = const 0x0,
        LOONGARCH_CSR_PRMD = const 0x1,
        LOONGARCH_CSR_EUEN = const 0x2,
        LOONGARCH_CSR_DMW0 = const 0x180,
        LOONGARCH_CSR_DMW1 = const 0x181,
        CSR_DMW1_BASE = const 0x9000000000000000usize,
        rust_main = sym crate::rust_main,
        per_cpu_size = const PER_CPU_SIZE,
        CSR_CPUID = const 0x20,
        options(noreturn),
    );
}

pub fn check_and_do_clear_bss() {
    // Loongarch clear bss in arch_entry, so we don't need to do it again.
    trace!("check_and_do_clear_bss: Loongarch does not need to clear bss again.");
    return;
}
