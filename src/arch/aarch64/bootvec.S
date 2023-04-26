.macro virt2phys, register
    add	\register, \register, x11
.endm

.macro phys2virt, register
    sub	\register, \register, x11
.endm

.macro	ventry	label
	.align	7
	b	\label
.endm
	.align 11
bootstrap_vectors:
	ventry	.
	ventry	.
	ventry	.
	ventry	.

	ventry	.
	ventry	.
	ventry	.
	ventry	.

	ventry	{0}
	ventry	.
	ventry	.
	ventry	.

	ventry	.
	ventry	.
	ventry	.
	ventry	.