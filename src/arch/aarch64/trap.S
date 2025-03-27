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
	/* Fill the union registers. Should comply with NUM_USR_REGS */
	/*now sp =>per_cpu stack top*/
	stp	x29, x30, [sp, #-16]!
	stp	x27, x28, [sp, #-16]!
	stp	x25, x26, [sp, #-16]!
	stp	x23, x24, [sp, #-16]!
	stp	x21, x22, [sp, #-16]!
	stp	x19, x20, [sp, #-16]!
	stp	x17, x18, [sp, #-16]!
	stp	x15, x16, [sp, #-16]!
	stp	x13, x14, [sp, #-16]!
	stp	x11, x12, [sp, #-16]!
	stp	x9, x10, [sp, #-16]!
	stp	x7, x8, [sp, #-16]!
	stp	x5, x6, [sp, #-16]!
	stp	x3, x4, [sp, #-16]!
	stp	x1, x2, [sp, #-16]!

	mov	x1, #\exit_reason
	stp	x1, x0, [sp, #-16]!

	mov	x29, xzr	/* reset fp,lr */
	mov	x30, xzr
	mov	x0, sp		/*x0 guest regs*/
	bl	{0}
	b	.
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