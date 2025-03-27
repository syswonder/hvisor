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

.text 

.macro VM_EXIT

    csrrw   x31, sscratch, x31

    sd   x0, 0*8(x31)
    sd   x1, 1*8(x31)
    sd   x2, 2*8(x31)
    sd   x3, 3*8(x31)
    sd   x4, 4*8(x31)
    sd   x5, 5*8(x31)
    sd   x6, 6*8(x31)
    sd   x7, 7*8(x31)
    sd   x8, 8*8(x31)
    sd   x9, 9*8(x31)
    sd   x10, 10*8(x31)
    sd   x11, 11*8(x31)
    sd   x12, 12*8(x31)
    sd   x13, 13*8(x31)
    sd   x14, 14*8(x31)
    sd   x15, 15*8(x31)
    sd   x16, 16*8(x31)
    sd   x17, 17*8(x31)
    sd   x18, 18*8(x31)
    sd   x19, 19*8(x31)
    sd   x20, 20*8(x31)
    sd   x21, 21*8(x31)
    sd   x22, 22*8(x31)
    sd   x23, 23*8(x31)
    sd   x24, 24*8(x31)
    sd   x25, 25*8(x31)
    sd   x26, 26*8(x31)
    sd   x27, 27*8(x31)
    sd   x28, 28*8(x31)
    sd   x29, 29*8(x31)
    sd   x30, 30*8(x31)
    
    mv      sp, x31
    csrrw   x31, sscratch, x31
    sd   x31, 31*8(sp)

    csrr    t0, hstatus
    sd   t0, 32*8(sp)
    csrr    t0, sstatus
    sd   t0, 33*8(sp)
    csrr    t0, sepc
    sd   t0, 34*8(sp)

    ld   sp, 35*8(sp)
    csrr      a0, sscratch
    # .option push
    # .option norelax
    # la  gp, __global_pointer$
    # .option pop   

.endm



.macro VM_ENTRY

    csrr   x31, sscratch

    ld   x1, 32*8(x31)
    csrw   hstatus, x1
    ld   x1, 33*8(x31)
    csrw   sstatus, x1
    ld   x1, 34*8(x31)
    csrw   sepc, x1

    ld   x0, 0*8(x31)
    ld   x1, 1*8(x31)
    ld   x2, 2*8(x31)
    ld   x3, 3*8(x31)
    ld   x4, 4*8(x31)
    ld   x5, 5*8(x31)
    ld   x6, 6*8(x31)
    ld   x7, 7*8(x31)
    ld   x8, 8*8(x31)
    ld   x9, 9*8(x31)
    ld   x10, 10*8(x31)
    ld   x11, 11*8(x31)
    ld   x12, 12*8(x31)
    ld   x13, 13*8(x31)
    ld   x14, 14*8(x31)
    ld   x15, 15*8(x31)
    ld   x16, 16*8(x31)
    ld   x17, 17*8(x31)
    ld   x18, 18*8(x31)
    ld   x19, 19*8(x31)
    ld   x20, 20*8(x31)
    ld   x21, 21*8(x31)
    ld   x22, 22*8(x31)
    ld   x23, 23*8(x31)
    ld   x24, 24*8(x31)
    ld   x25, 25*8(x31)
    ld   x26, 26*8(x31)
    ld   x27, 27*8(x31)
    ld   x28, 28*8(x31)
    ld   x29, 29*8(x31)
    ld   x30, 30*8(x31)
    ld   x31, 31*8(x31)

    sret
    j   .
.endm

.balign 0x4
.global _hyp_trap_vector	
_hyp_trap_vector:
    VM_EXIT
    csrr    t0, scause
    bltz    t0, 1f
    call    {sync_exception_handler}
    j       2f
1:
    call    {interrupts_arch_handle}
2:
.global vcpu_arch_entry
vcpu_arch_entry:
    VM_ENTRY
