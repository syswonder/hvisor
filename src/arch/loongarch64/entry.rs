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
//      Ming Shen  <boneinscri@163.com>
//
use crate::consts::PER_CPU_SIZE;
use crate::platform::CPU_BOOT_CONTEXT_ADDRESS;

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


#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_secondary_entry() -> i32 {
    core::arch::asm!(
       "li.d $r12, {CPU_BOOT_CONTEXT_ADDRESS}
        csrrd $r13, {LOONGARCH_CSR_CRMD}
        st.d $r13, $r12, (21+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PRMD}
        st.d $r13, $r12, (22+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_EUEN}
        st.d $r13, $r12, (23+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_MISC}
        st.d $r13, $r12, (24+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_ECFG}
        st.d $r13, $r12, (25+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_ESTAT}
        st.d $r13, $r12, (26+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_ERA}
        st.d $r13, $r12, (27+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_BADV}
        st.d $r13, $r12, (28+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_BADI}
        st.d $r13, $r12, (29+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_EENTRY}
        st.d $r13, $r12, (30+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBIDX}
        st.d $r13, $r12, (31+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBEHI}
        st.d $r13, $r12, (32+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBELO0}
        st.d $r13, $r12, (33+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBELO1}
        st.d $r13, $r12, (34+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_ASID}
        st.d $r13, $r12, (35+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PGDL}
        st.d $r13, $r12, (36+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PGDH}
        st.d $r13, $r12, (37+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PWCL}
        st.d $r13, $r12, (38+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PWCH}
        st.d $r13, $r12, (39+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_STLBPS}
        st.d $r13, $r12, (40+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_RVACFG}
        st.d $r13, $r12, (41+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_CPUID}
        st.d $r13, $r12, (42+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PRCFG1}
        st.d $r13, $r12, (43+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PRCFG2}
        st.d $r13, $r12, (44+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_PRCFG3}
        st.d $r13, $r12, (45+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE0}
        st.d $r13, $r12, (46+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE1}
        st.d $r13, $r12, (47+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE2}
        st.d $r13, $r12, (48+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE3}
        st.d $r13, $r12, (49+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE4}
        st.d $r13, $r12, (50+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE5}
        st.d $r13, $r12, (51+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE6}
        st.d $r13, $r12, (52+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_SAVE7}
        st.d $r13, $r12, (53+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TMID}
        st.d $r13, $r12, (54+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TCFG}
        st.d $r13, $r12, (55+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TVAL}
        st.d $r13, $r12, (56+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_CNTC}
        st.d $r13, $r12, (57+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TICLR}
        st.d $r13, $r12, (58+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRENTRY}
        st.d $r13, $r12, (59+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRBADV}
        st.d $r13, $r12, (60+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRERA}
        st.d $r13, $r12, (61+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRSAVE}
        st.d $r13, $r12, (62+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRELO0}
        st.d $r13, $r12, (63+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRELO1}
        st.d $r13, $r12, (64+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBREHI}
        st.d $r13, $r12, (65+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_TLBRPRMD}
        st.d $r13, $r12, (66+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_DMW0}
        st.d $r13, $r12, (67+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_DMW1}
        st.d $r13, $r12, (68+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_DMW2}
        st.d $r13, $r12, (69+3+50)*8
        csrrd $r13, {LOONGARCH_CSR_DMW3}
        st.d $r13, $r12, (70+3+50)*8
        bl     arch_entry",
        CPU_BOOT_CONTEXT_ADDRESS = const CPU_BOOT_CONTEXT_ADDRESS,
        LOONGARCH_CSR_CRMD = const 0x0,
        LOONGARCH_CSR_PRMD= const 0x1,
        LOONGARCH_CSR_EUEN= const 0x2,
        LOONGARCH_CSR_MISC = const 0x3,
        LOONGARCH_CSR_ECFG = const 0x4,
        LOONGARCH_CSR_ESTAT = const 0x5,
        LOONGARCH_CSR_ERA = const 0x6,
        LOONGARCH_CSR_BADV = const 0x7,
        LOONGARCH_CSR_BADI = const 0x8,
        LOONGARCH_CSR_EENTRY = const 0xc,
        LOONGARCH_CSR_TLBIDX = const 0x10,
        LOONGARCH_CSR_TLBEHI = const 0x11,
        LOONGARCH_CSR_TLBELO0 = const 0x12,
        LOONGARCH_CSR_TLBELO1 = const 0x13,
        LOONGARCH_CSR_ASID = const 0x18,
        LOONGARCH_CSR_PGDL = const 0x19,
        LOONGARCH_CSR_PGDH = const 0x1a,
        LOONGARCH_CSR_PWCL = const 0x1c,
        LOONGARCH_CSR_PWCH = const 0x1d,
        LOONGARCH_CSR_STLBPS = const 0x1e,
        LOONGARCH_CSR_RVACFG = const 0x1f,
        LOONGARCH_CSR_CPUID = const 0x20,
        LOONGARCH_CSR_PRCFG1 = const 0x21,
        LOONGARCH_CSR_PRCFG2 = const 0x22,
        LOONGARCH_CSR_PRCFG3 = const 0x23,
        LOONGARCH_CSR_SAVE0 = const 0x30,
        LOONGARCH_CSR_SAVE1 = const 0x31,
        LOONGARCH_CSR_SAVE2 = const 0x32,
        LOONGARCH_CSR_SAVE3 = const 0x33,
        LOONGARCH_CSR_SAVE4 = const 0x34,
        LOONGARCH_CSR_SAVE5 = const 0x35,
        LOONGARCH_CSR_SAVE6 = const 0x36,
        LOONGARCH_CSR_SAVE7 = const 0x37,
        LOONGARCH_CSR_TMID = const 0x40,
        LOONGARCH_CSR_TCFG = const 0x41,
        LOONGARCH_CSR_TVAL = const 0x42,
        LOONGARCH_CSR_CNTC = const 0x43,
        LOONGARCH_CSR_TICLR = const 0x44,
        LOONGARCH_CSR_TLBRENTRY = const 0x88,
        LOONGARCH_CSR_TLBRBADV = const 0x89,
        LOONGARCH_CSR_TLBRERA = const 0x8a,
        LOONGARCH_CSR_TLBRSAVE = const 0x8b,
        LOONGARCH_CSR_TLBRELO0 = const 0x8c,
        LOONGARCH_CSR_TLBRELO1 = const 0x8d,
        LOONGARCH_CSR_TLBREHI = const 0x8e,
        LOONGARCH_CSR_TLBRPRMD = const 0x8f,
        LOONGARCH_CSR_DMW0 = const 0x180,
        LOONGARCH_CSR_DMW1 = const 0x181,
        LOONGARCH_CSR_DMW2 = const 0x182,
        LOONGARCH_CSR_DMW3 = const 0x183,
        options(noreturn),
        
    );
}