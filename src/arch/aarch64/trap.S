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

.macro	ventry	label
	.align	7
	b	\label
.endm

.macro handle_vmexit exit_reason
	.align	7
	mrs x0, TPIDR_EL2
    stp x0, x1, [x0]
    stp x2, x3, [x0, #0x10]
    stp x4, x5, [x0, #0x20]
    stp x6, x7, [x0, #0x30]
    stp x8, x9, [x0, #0x40]
    stp x10, x11, [x0, #0x50]
    stp x12, x13, [x0, #0x60]
    stp x14, x15, [x0, #0x70]
    stp x16, x17, [x0, #0x80]
    stp x18, x19, [x0, #0x90]
    stp x20, x21, [x0, #0xa0]
    stp x22, x23, [x0, #0xb0]
    stp x24, x25, [x0, #0xc0]
    stp x26, x27, [x0, #0xd0]
    stp x28, x29, [x0, #0xe0]
    str x30, [x0, #0xf0]

    mrs x1, ELR_EL2
    str x1, [x0, #0xf8]
    mrs x1, SPSR_EL2
    str x1, [x0, #0x100]

    mov x1, #\exit_reason
    bl {0}
	bl .
.endm

EXIT_REASON_EL2_ABORT	=0x0
EXIT_REASON_EL2_IRQ		=0x1
EXIT_REASON_EL1_ABORT	=0x2
EXIT_REASON_EL1_IRQ		=0x3

.global _hyp_trap_vector
	.align 11
_hyp_trap_vector:
	ventry	.
	ventry	.
	ventry	.
	ventry	.

	handle_vmexit EXIT_REASON_EL2_ABORT
	handle_vmexit EXIT_REASON_EL2_IRQ
	ventry	.
	ventry	.

	handle_vmexit EXIT_REASON_EL1_ABORT
	handle_vmexit EXIT_REASON_EL1_IRQ
	ventry	.
	ventry	.

	ventry	.
	ventry	.
	ventry	.
	ventry	.