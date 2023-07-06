
hypervisor/hypervisor.o:	file format elf64-littleaarch64

Disassembly of section .text:

0000ffffc0200800 <spin_lock>:
ffffc0200800: f9800011 	prfm	pstl1strm, [x0]
ffffc0200804: 885ffc01 	ldaxr	w1, [x0]
ffffc0200808: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc020080c: 88037c02 	stxr	w3, w2, [x0]
ffffc0200810: 35ffffa3 	cbnz	w3, 0xffffc0200804 <spin_lock+0x4>
ffffc0200814: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc0200818: 340000c2 	cbz	w2, 0xffffc0200830 <spin_lock+0x30>
ffffc020081c: d50320bf 	sevl
ffffc0200820: d503205f 	wfe
ffffc0200824: 485ffc03 	ldaxrh	w3, [x0]
ffffc0200828: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc020082c: 35ffffa2 	cbnz	w2, 0xffffc0200820 <spin_lock+0x20>
ffffc0200830: d65f03c0 	ret

0000ffffc0200834 <arm_cpu_park>:
ffffc0200834: d2868d84 	mov	x4, #13420
ffffc0200838: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020083c: f2a00024 	movk	x4, #1, lsl #16
ffffc0200840: f2dfe004 	movk	x4, #65280, lsl #32
ffffc0200844: 910003fd 	mov	x29, sp
ffffc0200848: aa0403e0 	mov	x0, x4
ffffc020084c: 97ffffed 	bl	0xffffc0200800 <spin_lock>
ffffc0200850: d2840000 	mov	x0, #8192
ffffc0200854: 52800021 	mov	w1, #1
ffffc0200858: f2a00020 	movk	x0, #1, lsl #16
ffffc020085c: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0200860: b9147001 	str	w1, [x0, #5232]
ffffc0200864: b914781f 	str	wzr, [x0, #5240]
ffffc0200868: 7968d800 	ldrh	w0, [x0, #5228]
ffffc020086c: 0b010000 	add	w0, w0, w1
ffffc0200870: 489ffc80 	stlrh	w0, [x4]
ffffc0200874: d2800000 	mov	x0, #0
ffffc0200878: 94000c2c 	bl	0xffffc0203928 <arm_cpu_reset>
ffffc020087c: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0200880: 900000c0 	adrp	x0, 0xffffc0218000 <arch_reset_cpu+0x10>
ffffc0200884: 91014000 	add	x0, x0, #80
ffffc0200888: 14000a00 	b	0xffffc0203088 <arm_paging_vcpu_init>

0000ffffc020088c <arm_cpu_kick>:
ffffc020088c: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0200890: 910003fd 	mov	x29, sp
ffffc0200894: f9000bf3 	str	x19, [sp, #16]
ffffc0200898: 2a0003f3 	mov	w19, w0
ffffc020089c: 940007ec 	bl	0xffffc020284c <irqchip_get_cpu_target>
ffffc02008a0: 790073a0 	strh	w0, [x29, #56]
ffffc02008a4: 2a1303e0 	mov	w0, w19
ffffc02008a8: 940007ec 	bl	0xffffc0202858 <irqchip_get_cluster_target>
ffffc02008ac: f9001ba0 	str	x0, [x29, #48]
ffffc02008b0: 52800020 	mov	w0, #1
ffffc02008b4: 3900a3bf 	strb	wzr, [x29, #40]
ffffc02008b8: 790077a0 	strh	w0, [x29, #58]
ffffc02008bc: 9100a3a0 	add	x0, x29, #40
ffffc02008c0: 940007af 	bl	0xffffc020277c <irqchip_send_sgi>
ffffc02008c4: f9400bf3 	ldr	x19, [sp, #16]
ffffc02008c8: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc02008cc: d65f03c0 	ret

0000ffffc02008d0 <arch_reset_cpu>:
ffffc02008d0: d3727c01 	ubfiz	x1, x0, #14, #32
ffffc02008d4: b00000e2 	adrp	x2, 0xffffc021d000 <arch_handle_sgi+0x40>
ffffc02008d8: 91000042 	add	x2, x2, #0
ffffc02008dc: 8b020021 	add	x1, x1, x2
ffffc02008e0: 52800022 	mov	w2, #1
ffffc02008e4: b9347422 	str	w2, [x1, #13428]
ffffc02008e8: 14001d61 	b	0xffffc0207e6c <resume_cpu>

0000ffffc02008ec <arch_park_cpu>:
ffffc02008ec: d3727c01 	ubfiz	x1, x0, #14, #32
ffffc02008f0: b00000e2 	adrp	x2, 0xffffc021d000 <arch_handle_sgi+0x5c>
ffffc02008f4: 91000042 	add	x2, x2, #0
ffffc02008f8: 8b020021 	add	x1, x1, x2
ffffc02008fc: 52800022 	mov	w2, #1
ffffc0200900: b9347822 	str	w2, [x1, #13432]
ffffc0200904: 14001d5a 	b	0xffffc0207e6c <resume_cpu>

0000ffffc0200908 <arch_handle_sgi>:
ffffc0200908: 340000e0 	cbz	w0, 0xffffc0200924 <arch_handle_sgi+0x1c>
ffffc020090c: 7100041f 	cmp	w0, #1
ffffc0200910: 54000180 	b.eq	0xffffc0200940 <arch_handle_sgi+0x38>
ffffc0200914: 2a0003e1 	mov	w1, w0
ffffc0200918: f0000040 	adrp	x0, 0xffffc020b000 <arch_handle_sgi+0x3c>
ffffc020091c: 912dd000 	add	x0, x0, #2932
ffffc0200920: 14001881 	b	0xffffc0206b24 <printk>
ffffc0200924: d2840002 	mov	x2, #8192
ffffc0200928: f2a00022 	movk	x2, #1, lsl #16
ffffc020092c: f2dfe002 	movk	x2, #65280, lsl #32
ffffc0200930: b9502840 	ldr	w0, [x2, #4136]
ffffc0200934: 0b010001 	add	w1, w0, w1
ffffc0200938: b9102841 	str	w1, [x2, #4136]
ffffc020093c: 1400076a 	b	0xffffc02026e4 <irqchip_inject_pending>
ffffc0200940: d2840004 	mov	x4, #8192
ffffc0200944: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0200948: f2a00024 	movk	x4, #1, lsl #16
ffffc020094c: 2a0003e5 	mov	w5, w0
ffffc0200950: f2dfe004 	movk	x4, #65280, lsl #32
ffffc0200954: 910003fd 	mov	x29, sp
ffffc0200958: d2868d86 	mov	x6, #13420
ffffc020095c: b9501880 	ldr	w0, [x4, #4120]
ffffc0200960: f2a00026 	movk	x6, #1, lsl #16
ffffc0200964: f2dfe006 	movk	x6, #65280, lsl #32
ffffc0200968: 0b010001 	add	w1, w0, w1
ffffc020096c: b9101881 	str	w1, [x4, #4120]
ffffc0200970: aa0603e0 	mov	x0, x6
ffffc0200974: 97ffffa3 	bl	0xffffc0200800 <spin_lock>
ffffc0200978: b9503880 	ldr	w0, [x4, #4152]
ffffc020097c: 35000120 	cbnz	w0, 0xffffc02009a0 <arch_handle_sgi+0x98>
ffffc0200980: b9547880 	ldr	w0, [x4, #5240]
ffffc0200984: b9103c9f 	str	wzr, [x4, #4156]
ffffc0200988: 340001a0 	cbz	w0, 0xffffc02009bc <arch_handle_sgi+0xb4>
ffffc020098c: 52800020 	mov	w0, #1
ffffc0200990: b914789f 	str	wzr, [x4, #5240]
ffffc0200994: b9147080 	str	w0, [x4, #5232]
ffffc0200998: 52800000 	mov	w0, #0
ffffc020099c: 14000010 	b	0xffffc02009dc <arch_handle_sgi+0xd4>
ffffc02009a0: 7968d880 	ldrh	w0, [x4, #5228]
ffffc02009a4: b9103c85 	str	w5, [x4, #4156]
ffffc02009a8: 11000400 	add	w0, w0, #1
ffffc02009ac: 489ffcc0 	stlrh	w0, [x6]
ffffc02009b0: b9503880 	ldr	w0, [x4, #4152]
ffffc02009b4: 34fffde0 	cbz	w0, 0xffffc0200970 <arch_handle_sgi+0x68>
ffffc02009b8: 17fffffe 	b	0xffffc02009b0 <arch_handle_sgi+0xa8>
ffffc02009bc: b9547480 	ldr	w0, [x4, #5236]
ffffc02009c0: 340000e0 	cbz	w0, 0xffffc02009dc <arch_handle_sgi+0xd4>
ffffc02009c4: f94a4080 	ldr	x0, [x4, #5248]
ffffc02009c8: b914749f 	str	wzr, [x4, #5236]
ffffc02009cc: b100041f 	cmn	x0, #1
ffffc02009d0: 52800020 	mov	w0, #1
ffffc02009d4: 54fffe00 	b.eq	0xffffc0200994 <arch_handle_sgi+0x8c>
ffffc02009d8: b914709f 	str	wzr, [x4, #5232]
ffffc02009dc: b9504081 	ldr	w1, [x4, #4160]
ffffc02009e0: 34000061 	cbz	w1, 0xffffc02009ec <arch_handle_sgi+0xe4>
ffffc02009e4: b910409f 	str	wzr, [x4, #4160]
ffffc02009e8: d50c83df 	tlbi	vmalls12e1is
ffffc02009ec: 7968d881 	ldrh	w1, [x4, #5228]
ffffc02009f0: 11000421 	add	w1, w1, #1
ffffc02009f4: 489ffcc1 	stlrh	w1, [x6]
ffffc02009f8: b9547081 	ldr	w1, [x4, #5232]
ffffc02009fc: 34000061 	cbz	w1, 0xffffc0200a08 <arch_handle_sgi+0x100>
ffffc0200a00: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0200a04: 17ffff8c 	b	0xffffc0200834 <arm_cpu_park>
ffffc0200a08: 34000080 	cbz	w0, 0xffffc0200a18 <arch_handle_sgi+0x110>
ffffc0200a0c: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0200a10: f94a4080 	ldr	x0, [x4, #5248]
ffffc0200a14: 14000bc5 	b	0xffffc0203928 <arm_cpu_reset>
ffffc0200a18: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0200a1c: d65f03c0 	ret

0000ffffc0200a20 <arch_handle_phys_irq>:
ffffc0200a20: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0200a24: 2a0003e3 	mov	w3, w0
ffffc0200a28: b00000c0 	adrp	x0, 0xffffc0219000 <arch_cell_reset+0x4>
ffffc0200a2c: 910003fd 	mov	x29, sp
ffffc0200a30: f9400000 	ldr	x0, [x0]
ffffc0200a34: 39416000 	ldrb	w0, [x0, #88]
ffffc0200a38: 6b03001f 	cmp	w0, w3
ffffc0200a3c: d2840000 	mov	x0, #8192
ffffc0200a40: f2a00020 	movk	x0, #1, lsl #16
ffffc0200a44: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0200a48: 54000101 	b.ne	0xffffc0200a68 <arch_handle_phys_irq+0x48>
ffffc0200a4c: b9502002 	ldr	w2, [x0, #4128]
ffffc0200a50: 0b010041 	add	w1, w2, w1
ffffc0200a54: b9102001 	str	w1, [x0, #4128]
ffffc0200a58: 94000723 	bl	0xffffc02026e4 <irqchip_inject_pending>
ffffc0200a5c: 52800020 	mov	w0, #1
ffffc0200a60: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0200a64: d65f03c0 	ret
ffffc0200a68: b9502402 	ldr	w2, [x0, #4132]
ffffc0200a6c: 0b010041 	add	w1, w2, w1
ffffc0200a70: b9102401 	str	w1, [x0, #4132]
ffffc0200a74: 2a0303e1 	mov	w1, w3
ffffc0200a78: 9400077b 	bl	0xffffc0202864 <irqchip_set_pending>
ffffc0200a7c: 52800000 	mov	w0, #0
ffffc0200a80: 17fffff8 	b	0xffffc0200a60 <arch_handle_phys_irq+0x40>

0000ffffc0200a84 <arch_cell_create>:
ffffc0200a84: 14000956 	b	0xffffc0202fdc <arm_paging_cell_init>

0000ffffc0200a88 <arch_cell_reset>:
ffffc0200a88: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0200a8c: 12800002 	mov	w2, #-1
ffffc0200a90: 910003fd 	mov	x29, sp
ffffc0200a94: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0200a98: aa0003f3 	mov	x19, x0
ffffc0200a9c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0200aa0: 92800016 	mov	x22, #-1
ffffc0200aa4: f9485401 	ldr	x1, [x0, #4264]
ffffc0200aa8: 2a0203e0 	mov	w0, w2
ffffc0200aac: 94001c67 	bl	0xffffc0207c48 <next_cpu>
ffffc0200ab0: 2a0003f5 	mov	w21, w0
ffffc0200ab4: b00000c0 	adrp	x0, 0xffffc0219000 <arch_cell_reset+0x90>
ffffc0200ab8: f9400000 	ldr	x0, [x0]
ffffc0200abc: 39416401 	ldrb	w1, [x0, #89]
ffffc0200ac0: 3900e261 	strb	w1, [x19, #56]
ffffc0200ac4: f845a001 	ldur	x1, [x0, #90]
ffffc0200ac8: f9002261 	str	x1, [x19, #64]
ffffc0200acc: f8462001 	ldur	x1, [x0, #98]
ffffc0200ad0: f9002661 	str	x1, [x19, #72]
ffffc0200ad4: b00000e1 	adrp	x1, 0xffffc021d000 <arch_cell_reset+0xc0>
ffffc0200ad8: 91000021 	add	x1, x1, #0
ffffc0200adc: f847a000 	ldur	x0, [x0, #122]
ffffc0200ae0: aa0103f4 	mov	x20, x1
ffffc0200ae4: f9002a60 	str	x0, [x19, #80]
ffffc0200ae8: f9485260 	ldr	x0, [x19, #4256]
ffffc0200aec: f9402802 	ldr	x2, [x0, #80]
ffffc0200af0: d3727ea0 	ubfiz	x0, x21, #14, #32
ffffc0200af4: 8b010000 	add	x0, x0, x1
ffffc0200af8: f91a4002 	str	x2, [x0, #13440]
ffffc0200afc: 12800000 	mov	w0, #-1
ffffc0200b00: f9485661 	ldr	x1, [x19, #4264]
ffffc0200b04: 2a1503e2 	mov	w2, w21
ffffc0200b08: 94001c50 	bl	0xffffc0207c48 <next_cpu>
ffffc0200b0c: 2a0003e1 	mov	w1, w0
ffffc0200b10: f9485662 	ldr	x2, [x19, #4264]
ffffc0200b14: f9400042 	ldr	x2, [x2]
ffffc0200b18: eb02003f 	cmp	x1, x2
ffffc0200b1c: 54000129 	b.ls	0xffffc0200b40 <arch_cell_reset+0xb8>
ffffc0200b20: aa1303e0 	mov	x0, x19
ffffc0200b24: 52800021 	mov	w1, #1
ffffc0200b28: 940008f6 	bl	0xffffc0202f00 <arm_cell_dcaches_flush>
ffffc0200b2c: aa1303e0 	mov	x0, x19
ffffc0200b30: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0200b34: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0200b38: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0200b3c: 140007f9 	b	0xffffc0202b20 <irqchip_cell_reset>
ffffc0200b40: 8b013a81 	add	x1, x20, x1, lsl #14
ffffc0200b44: f91a4036 	str	x22, [x1, #13440]
ffffc0200b48: 17ffffee 	b	0xffffc0200b00 <arch_cell_reset+0x78>

0000ffffc0200b4c <arch_cell_destroy>:
ffffc0200b4c: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0200b50: 52800021 	mov	w1, #1
ffffc0200b54: 910003fd 	mov	x29, sp
ffffc0200b58: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0200b5c: aa0003f3 	mov	x19, x0
ffffc0200b60: f90013f5 	str	x21, [sp, #32]
ffffc0200b64: b00000f4 	adrp	x20, 0xffffc021d000 <arch_flush_cell_vcpu_caches+0x20>
ffffc0200b68: 940008e6 	bl	0xffffc0202f00 <arm_cell_dcaches_flush>
ffffc0200b6c: 91000294 	add	x20, x20, #0
ffffc0200b70: 12800000 	mov	w0, #-1
ffffc0200b74: 92800015 	mov	x21, #-1
ffffc0200b78: f9485661 	ldr	x1, [x19, #4264]
ffffc0200b7c: 12800002 	mov	w2, #-1
ffffc0200b80: 94001c32 	bl	0xffffc0207c48 <next_cpu>
ffffc0200b84: 2a0003e1 	mov	w1, w0
ffffc0200b88: f9485662 	ldr	x2, [x19, #4264]
ffffc0200b8c: f9400042 	ldr	x2, [x2]
ffffc0200b90: eb02003f 	cmp	x1, x2
ffffc0200b94: 540000c9 	b.ls	0xffffc0200bac <arch_cell_destroy+0x60>
ffffc0200b98: aa1303e0 	mov	x0, x19
ffffc0200b9c: f94013f5 	ldr	x21, [sp, #32]
ffffc0200ba0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0200ba4: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0200ba8: 1400092c 	b	0xffffc0203058 <arm_paging_cell_destroy>
ffffc0200bac: 8b013a81 	add	x1, x20, x1, lsl #14
ffffc0200bb0: f91a4035 	str	x21, [x1, #13440]
ffffc0200bb4: 17fffff1 	b	0xffffc0200b78 <arch_cell_destroy+0x2c>

0000ffffc0200bb8 <arch_flush_cell_vcpu_caches>:
ffffc0200bb8: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0200bbc: 910003fd 	mov	x29, sp
ffffc0200bc0: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0200bc4: d2840015 	mov	x21, #8192
ffffc0200bc8: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0200bcc: f2a00035 	movk	x21, #1, lsl #16
ffffc0200bd0: b00000f3 	adrp	x19, 0xffffc021d000 <arch_prepare_shutdown>
ffffc0200bd4: 91000273 	add	x19, x19, #0
ffffc0200bd8: aa0003f4 	mov	x20, x0
ffffc0200bdc: f2dfe015 	movk	x21, #65280, lsl #32
ffffc0200be0: 12800000 	mov	w0, #-1
ffffc0200be4: 52800036 	mov	w22, #1
ffffc0200be8: f9485681 	ldr	x1, [x20, #4264]
ffffc0200bec: 12800002 	mov	w2, #-1
ffffc0200bf0: 94001c16 	bl	0xffffc0207c48 <next_cpu>
ffffc0200bf4: 2a0003e1 	mov	w1, w0
ffffc0200bf8: f9485682 	ldr	x2, [x20, #4264]
ffffc0200bfc: f9400042 	ldr	x2, [x2]
ffffc0200c00: eb02003f 	cmp	x1, x2
ffffc0200c04: 540000a9 	b.ls	0xffffc0200c18 <arch_flush_cell_vcpu_caches+0x60>
ffffc0200c08: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0200c0c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0200c10: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0200c14: d65f03c0 	ret
ffffc0200c18: b95002a2 	ldr	w2, [x21, #4096]
ffffc0200c1c: 6b02001f 	cmp	w0, w2
ffffc0200c20: 54000061 	b.ne	0xffffc0200c2c <arch_flush_cell_vcpu_caches+0x74>
ffffc0200c24: d50c83df 	tlbi	vmalls12e1is
ffffc0200c28: 17fffff0 	b	0xffffc0200be8 <arch_flush_cell_vcpu_caches+0x30>
ffffc0200c2c: 8b013a61 	add	x1, x19, x1, lsl #14
ffffc0200c30: b9304036 	str	w22, [x1, #12352]
ffffc0200c34: 17ffffed 	b	0xffffc0200be8 <arch_flush_cell_vcpu_caches+0x30>

0000ffffc0200c38 <arch_config_commit>:
ffffc0200c38: 14000815 	b	0xffffc0202c8c <irqchip_config_commit>

0000ffffc0200c3c <arch_panic_stop>:
ffffc0200c3c: d503207f 	wfi
ffffc0200c40: 17ffffff 	b	0xffffc0200c3c <arch_panic_stop>

0000ffffc0200c44 <arch_prepare_shutdown>:
ffffc0200c44: d65f03c0 	ret

0000ffffc0200c48 <arch_dbg_write_init>:
ffffc0200c48: b00000c0 	adrp	x0, 0xffffc0219000 <arch_dbg_write_init+0x64>
ffffc0200c4c: f9400002 	ldr	x2, [x0]
ffffc0200c50: 79407440 	ldrh	w0, [x2, #58]
ffffc0200c54: 36000700 	tbz	w0, #0, 0xffffc0200d34 <arch_dbg_write_init+0xec>
ffffc0200c58: 3940e041 	ldrb	w1, [x2, #56]
ffffc0200c5c: 900000e0 	adrp	x0, 0xffffc021c000 <arch_dbg_write_init+0x84>
ffffc0200c60: 71000c3f 	cmp	w1, #3
ffffc0200c64: 540002c1 	b.ne	0xffffc0200cbc <arch_dbg_write_init+0x74>
ffffc0200c68: d0000061 	adrp	x1, 0xffffc020e000 <arch_dbg_write_init+0x58>
ffffc0200c6c: 9102a021 	add	x1, x1, #168
ffffc0200c70: f9001401 	str	x1, [x0, #40]
ffffc0200c74: f9401401 	ldr	x1, [x0, #40]
ffffc0200c78: b40005e1 	cbz	x1, 0xffffc0200d34 <arch_dbg_write_init+0xec>
ffffc0200c7c: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0200c80: 9100b042 	add	x2, x2, #44
ffffc0200c84: 910003fd 	mov	x29, sp
ffffc0200c88: f9000422 	str	x2, [x1, #8]
ffffc0200c8c: 90000002 	adrp	x2, 0xffffc0200000 <arch_dbg_write_init+0x44>
ffffc0200c90: f9401c42 	ldr	x2, [x2, #56]
ffffc0200c94: f9000022 	str	x2, [x1]
ffffc0200c98: f9401400 	ldr	x0, [x0, #40]
ffffc0200c9c: f9401001 	ldr	x1, [x0, #32]
ffffc0200ca0: d63f0020 	blr	x1
ffffc0200ca4: d0000061 	adrp	x1, 0xffffc020e000 <arch_dbg_write_init+0x94>
ffffc0200ca8: d0000040 	adrp	x0, 0xffffc020a000 <arch_dbg_write_init+0x88>
ffffc0200cac: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0200cb0: 913d6000 	add	x0, x0, #3928
ffffc0200cb4: f900ac20 	str	x0, [x1, #344]
ffffc0200cb8: d65f03c0 	ret
ffffc0200cbc: 7100083f 	cmp	w1, #2
ffffc0200cc0: 54000081 	b.ne	0xffffc0200cd0 <arch_dbg_write_init+0x88>
ffffc0200cc4: d0000061 	adrp	x1, 0xffffc020e000 <arch_dbg_write_init+0xb4>
ffffc0200cc8: 91066021 	add	x1, x1, #408
ffffc0200ccc: 17ffffe9 	b	0xffffc0200c70 <arch_dbg_write_init+0x28>
ffffc0200cd0: 7100103f 	cmp	w1, #4
ffffc0200cd4: 54000081 	b.ne	0xffffc0200ce4 <arch_dbg_write_init+0x9c>
ffffc0200cd8: d0000061 	adrp	x1, 0xffffc020e000 <arch_dbg_write_init+0xc8>
ffffc0200cdc: 91046021 	add	x1, x1, #280
ffffc0200ce0: 17ffffe4 	b	0xffffc0200c70 <arch_dbg_write_init+0x28>
ffffc0200ce4: 7100143f 	cmp	w1, #5
ffffc0200ce8: 54000081 	b.ne	0xffffc0200cf8 <arch_dbg_write_init+0xb0>
ffffc0200cec: d0000061 	adrp	x1, 0xffffc020e000 <arch_dbg_write_init+0xdc>
ffffc0200cf0: 9101c021 	add	x1, x1, #112
ffffc0200cf4: 17ffffdf 	b	0xffffc0200c70 <arch_dbg_write_init+0x28>
ffffc0200cf8: 7100183f 	cmp	w1, #6
ffffc0200cfc: 54000081 	b.ne	0xffffc0200d0c <arch_dbg_write_init+0xc4>
ffffc0200d00: d0000061 	adrp	x1, 0xffffc020e000 <gicv2_targets_in_cell>
ffffc0200d04: 91000021 	add	x1, x1, #0
ffffc0200d08: 17ffffda 	b	0xffffc0200c70 <arch_dbg_write_init+0x28>
ffffc0200d0c: 71001c3f 	cmp	w1, #7
ffffc0200d10: 54000081 	b.ne	0xffffc0200d20 <arch_dbg_write_init+0xd8>
ffffc0200d14: d0000061 	adrp	x1, 0xffffc020e000 <gicv2_targets_in_cell+0x14>
ffffc0200d18: 91038021 	add	x1, x1, #224
ffffc0200d1c: 17ffffd5 	b	0xffffc0200c70 <arch_dbg_write_init+0x28>
ffffc0200d20: 7100203f 	cmp	w1, #8
ffffc0200d24: 54fffa81 	b.ne	0xffffc0200c74 <arch_dbg_write_init+0x2c>
ffffc0200d28: d0000061 	adrp	x1, 0xffffc020e000 <gicv2_targets_in_cell+0x28>
ffffc0200d2c: 9100e021 	add	x1, x1, #56
ffffc0200d30: 17ffffd0 	b	0xffffc0200c70 <arch_dbg_write_init+0x28>
ffffc0200d34: d65f03c0 	ret

0000ffffc0200d38 <gicv2_targets_in_cell>:
ffffc0200d38: b00000a3 	adrp	x3, 0xffffc0215000 <gicv2_cpu_shutdown+0x4>
ffffc0200d3c: b00000e4 	adrp	x4, 0xffffc021d000 <gicv2_cpu_shutdown+0x28>
ffffc0200d40: 12001c21 	and	w1, w1, #0xff
ffffc0200d44: 91000063 	add	x3, x3, #0
ffffc0200d48: 91000084 	add	x4, x4, #0
ffffc0200d4c: d2800002 	mov	x2, #0
ffffc0200d50: 38636845 	ldrb	w5, [x2, x3]
ffffc0200d54: 6a05003f 	tst	w1, w5
ffffc0200d58: 540000a0 	b.eq	0xffffc0200d6c <gicv2_targets_in_cell+0x34>
ffffc0200d5c: 8b023885 	add	x5, x4, x2, lsl #14
ffffc0200d60: f95804a5 	ldr	x5, [x5, #12296]
ffffc0200d64: eb0000bf 	cmp	x5, x0
ffffc0200d68: 540000c1 	b.ne	0xffffc0200d80 <gicv2_targets_in_cell+0x48>
ffffc0200d6c: 91000442 	add	x2, x2, #1
ffffc0200d70: f100205f 	cmp	x2, #8
ffffc0200d74: 54fffee1 	b.ne	0xffffc0200d50 <gicv2_targets_in_cell+0x18>
ffffc0200d78: 52800020 	mov	w0, #1
ffffc0200d7c: d65f03c0 	ret
ffffc0200d80: 52800000 	mov	w0, #0
ffffc0200d84: 17fffffe 	b	0xffffc0200d7c <gicv2_targets_in_cell+0x44>

0000ffffc0200d88 <gicv2_cpu_shutdown>:
ffffc0200d88: b9505000 	ldr	w0, [x0, #4176]
ffffc0200d8c: 34000320 	cbz	w0, 0xffffc0200df0 <gicv2_cpu_shutdown+0x68>
ffffc0200d90: b00000a1 	adrp	x1, 0xffffc0215000 <gicv2_cpu_shutdown+0x5c>
ffffc0200d94: 91000020 	add	x0, x1, #0
ffffc0200d98: f9400402 	ldr	x2, [x0, #8]
ffffc0200d9c: b00000a0 	adrp	x0, 0xffffc0215000 <gicv2_cpu_shutdown+0x68>
ffffc0200da0: b900005f 	str	wzr, [x2]
ffffc0200da4: f9407003 	ldr	x3, [x0, #224]
ffffc0200da8: b00000c0 	adrp	x0, 0xffffc0219000 <gicv2_eoi_irq>
ffffc0200dac: f9400000 	ldr	x0, [x0]
ffffc0200db0: 39416004 	ldrb	w4, [x0, #88]
ffffc0200db4: 52800020 	mov	w0, #1
ffffc0200db8: 1ac42000 	lsl	w0, w0, w4
ffffc0200dbc: b9018060 	str	w0, [x3, #384]
ffffc0200dc0: b9400840 	ldr	w0, [x2, #8]
ffffc0200dc4: 12000002 	and	w2, w0, #0x1
ffffc0200dc8: 36480040 	tbz	w0, #9, 0xffffc0200dd0 <gicv2_cpu_shutdown+0x48>
ffffc0200dcc: 32170042 	orr	w2, w2, #0x200
ffffc0200dd0: 91000021 	add	x1, x1, #0
ffffc0200dd4: 531b7c00 	lsr	w0, w0, #27
ffffc0200dd8: 531d7000 	lsl	w0, w0, #3
ffffc0200ddc: f9400821 	ldr	x1, [x1, #16]
ffffc0200de0: b9000022 	str	w2, [x1]
ffffc0200de4: b9000420 	str	w0, [x1, #4]
ffffc0200de8: 52800000 	mov	w0, #0
ffffc0200dec: d65f03c0 	ret
ffffc0200df0: 12800240 	mov	w0, #-19
ffffc0200df4: 17fffffe 	b	0xffffc0200dec <gicv2_cpu_shutdown+0x64>

0000ffffc0200df8 <gicv2_read_iar_irqn>:
ffffc0200df8: b00000a0 	adrp	x0, 0xffffc0215000 <gicv2_send_sgi+0x28>
ffffc0200dfc: f9400800 	ldr	x0, [x0, #16]
ffffc0200e00: b9400c00 	ldr	w0, [x0, #12]
ffffc0200e04: 12002400 	and	w0, w0, #0x3ff
ffffc0200e08: d65f03c0 	ret

0000ffffc0200e0c <gicv2_eoi_irq>:
ffffc0200e0c: b00000a2 	adrp	x2, 0xffffc0215000 <gicv2_send_sgi+0x3c>
ffffc0200e10: f9400842 	ldr	x2, [x2, #16]
ffffc0200e14: b9001040 	str	w0, [x2, #16]
ffffc0200e18: 34000041 	cbz	w1, 0xffffc0200e20 <gicv2_eoi_irq+0x14>
ffffc0200e1c: b9100040 	str	w0, [x2, #4096]
ffffc0200e20: d65f03c0 	ret

0000ffffc0200e24 <gicv2_send_sgi>:
ffffc0200e24: 79402402 	ldrh	w2, [x0, #18]
ffffc0200e28: 71003c5f 	cmp	w2, #15
ffffc0200e2c: 54000188 	b.hi	0xffffc0200e5c <gicv2_send_sgi+0x38>
ffffc0200e30: 39400001 	ldrb	w1, [x0]
ffffc0200e34: 79402000 	ldrh	w0, [x0, #16]
ffffc0200e38: 53080421 	ubfiz	w1, w1, #24, #2
ffffc0200e3c: 53101c00 	ubfiz	w0, w0, #16, #8
ffffc0200e40: 2a000020 	orr	w0, w1, w0
ffffc0200e44: b00000a1 	adrp	x1, 0xffffc0215000 <gicv2_inject_irq+0x34>
ffffc0200e48: 2a020000 	orr	w0, w0, w2
ffffc0200e4c: f9407021 	ldr	x1, [x1, #224]
ffffc0200e50: b90f0020 	str	w0, [x1, #3840]
ffffc0200e54: 52800000 	mov	w0, #0
ffffc0200e58: d65f03c0 	ret
ffffc0200e5c: 128002a0 	mov	w0, #-22
ffffc0200e60: 17fffffe 	b	0xffffc0200e58 <gicv2_send_sgi+0x34>

0000ffffc0200e64 <gicv2_inject_irq>:
ffffc0200e64: b00000a2 	adrp	x2, 0xffffc0215000 <gicv2_inject_irq+0x54>
ffffc0200e68: 91000042 	add	x2, x2, #0
ffffc0200e6c: d10043ff 	sub	sp, sp, #16
ffffc0200e70: 12003c00 	and	w0, w0, #0xffff
ffffc0200e74: 12003c21 	and	w1, w1, #0xffff
ffffc0200e78: f9400444 	ldr	x4, [x2, #8]
ffffc0200e7c: b9401846 	ldr	w6, [x2, #24]
ffffc0200e80: 12800002 	mov	w2, #-1
ffffc0200e84: b9403083 	ldr	w3, [x4, #48]
ffffc0200e88: 2a0303e3 	mov	w3, w3
ffffc0200e8c: f90003e3 	str	x3, [sp]
ffffc0200e90: b9403483 	ldr	w3, [x4, #52]
ffffc0200e94: 2a0303e3 	mov	w3, w3
ffffc0200e98: f90007e3 	str	x3, [sp, #8]
ffffc0200e9c: 52800003 	mov	w3, #0
ffffc0200ea0: 6b06007f 	cmp	w3, w6
ffffc0200ea4: 54000221 	b.ne	0xffffc0200ee8 <gicv2_inject_irq+0x84>
ffffc0200ea8: 3100045f 	cmn	w2, #1
ffffc0200eac: 54000420 	b.eq	0xffffc0200f30 <gicv2_inject_irq+0xcc>
ffffc0200eb0: 531e7442 	lsl	w2, w2, #2
ffffc0200eb4: 32040003 	orr	w3, w0, #0x10000000
ffffc0200eb8: 53160821 	ubfiz	w1, w1, #10, #3
ffffc0200ebc: 91040042 	add	x2, x2, #256
ffffc0200ec0: 2a030023 	orr	w3, w1, w3
ffffc0200ec4: 52b20001 	mov	w1, #-1879048192
ffffc0200ec8: 2a010001 	orr	w1, w0, w1
ffffc0200ecc: 71003c1f 	cmp	w0, #15
ffffc0200ed0: 2a002821 	orr	w1, w1, w0, lsl #10
ffffc0200ed4: 52800000 	mov	w0, #0
ffffc0200ed8: 1a838021 	csel	w1, w1, w3, hi
ffffc0200edc: b8226881 	str	w1, [x4, x2]
ffffc0200ee0: 910043ff 	add	sp, sp, #16
ffffc0200ee4: d65f03c0 	ret
ffffc0200ee8: 53067c65 	lsr	w5, w3, #6
ffffc0200eec: 12001467 	and	w7, w3, #0x3f
ffffc0200ef0: d37df0a5 	lsl	x5, x5, #3
ffffc0200ef4: f8656be5 	ldr	x5, [sp, x5]
ffffc0200ef8: 9ac724a5 	lsr	x5, x5, x7
ffffc0200efc: 360000a5 	tbz	w5, #0, 0xffffc0200f10 <gicv2_inject_irq+0xac>
ffffc0200f00: 3100045f 	cmn	w2, #1
ffffc0200f04: 1a831042 	csel	w2, w2, w3, ne
ffffc0200f08: 11000463 	add	w3, w3, #1
ffffc0200f0c: 17ffffe5 	b	0xffffc0200ea0 <gicv2_inject_irq+0x3c>
ffffc0200f10: 531e7465 	lsl	w5, w3, #2
ffffc0200f14: 910400a5 	add	x5, x5, #256
ffffc0200f18: b8656885 	ldr	w5, [x4, x5]
ffffc0200f1c: 120024a5 	and	w5, w5, #0x3ff
ffffc0200f20: 6b0000bf 	cmp	w5, w0
ffffc0200f24: 54ffff21 	b.ne	0xffffc0200f08 <gicv2_inject_irq+0xa4>
ffffc0200f28: 12800200 	mov	w0, #-17
ffffc0200f2c: 17ffffed 	b	0xffffc0200ee0 <gicv2_inject_irq+0x7c>
ffffc0200f30: 128001e0 	mov	w0, #-16
ffffc0200f34: 17ffffeb 	b	0xffffc0200ee0 <gicv2_inject_irq+0x7c>

0000ffffc0200f38 <gicv2_enable_maint_irq>:
ffffc0200f38: b00000a1 	adrp	x1, 0xffffc0215000 <gicv2_adjust_irq_target+0x10>
ffffc0200f3c: 7100001f 	cmp	w0, #0
ffffc0200f40: f9400422 	ldr	x2, [x1, #8]
ffffc0200f44: b9400041 	ldr	w1, [x2]
ffffc0200f48: 321f0023 	orr	w3, w1, #0x2
ffffc0200f4c: 121e7821 	and	w1, w1, #0xfffffffd
ffffc0200f50: 1a830021 	csel	w1, w1, w3, eq
ffffc0200f54: b9000041 	str	w1, [x2]
ffffc0200f58: d65f03c0 	ret

0000ffffc0200f5c <gicv2_handle_irq_route>:
ffffc0200f5c: 52800020 	mov	w0, #1
ffffc0200f60: d65f03c0 	ret

0000ffffc0200f64 <gicv2_get_cpu_target>:
ffffc0200f64: b00000a1 	adrp	x1, 0xffffc0215000 <gicv2_adjust_irq_target+0x3c>
ffffc0200f68: 91000021 	add	x1, x1, #0
ffffc0200f6c: 38604820 	ldrb	w0, [x1, w0, uxtw]
ffffc0200f70: d65f03c0 	ret

0000ffffc0200f74 <gicv2_get_cluster_target>:
ffffc0200f74: d2800000 	mov	x0, #0
ffffc0200f78: d65f03c0 	ret

0000ffffc0200f7c <gicv2_adjust_irq_target>:
ffffc0200f7c: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0200f80: 12003c21 	and	w1, w1, #0xffff
ffffc0200f84: b00000a2 	adrp	x2, 0xffffc0215000 <gicv2_adjust_irq_target+0x5c>
ffffc0200f88: aa0003e7 	mov	x7, x0
ffffc0200f8c: 910003fd 	mov	x29, sp
ffffc0200f90: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0200f94: 927e3435 	and	x21, x1, #0xfffc
ffffc0200f98: 912002b5 	add	x21, x21, #2048
ffffc0200f9c: f9407056 	ldr	x22, [x2, #224]
ffffc0200fa0: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0200fa4: 531d0433 	ubfiz	w19, w1, #3, #2
ffffc0200fa8: b8756ac6 	ldr	w6, [x22, x21]
ffffc0200fac: 1ad324c1 	lsr	w1, w6, w19
ffffc0200fb0: 97ffff62 	bl	0xffffc0200d38 <gicv2_targets_in_cell>
ffffc0200fb4: 350001c0 	cbnz	w0, 0xffffc0200fec <gicv2_adjust_irq_target+0x70>
ffffc0200fb8: f94854e1 	ldr	x1, [x7, #4264]
ffffc0200fbc: 52801ff4 	mov	w20, #255
ffffc0200fc0: 1ad32294 	lsl	w20, w20, w19
ffffc0200fc4: 12800002 	mov	w2, #-1
ffffc0200fc8: 0a3400d4 	bic	w20, w6, w20
ffffc0200fcc: 2a0203e0 	mov	w0, w2
ffffc0200fd0: 94001b1e 	bl	0xffffc0207c48 <next_cpu>
ffffc0200fd4: b00000a1 	adrp	x1, 0xffffc0215000 <gicv2_cell_init+0x10>
ffffc0200fd8: 91000021 	add	x1, x1, #0
ffffc0200fdc: 38604821 	ldrb	w1, [x1, w0, uxtw]
ffffc0200fe0: 1ad32033 	lsl	w19, w1, w19
ffffc0200fe4: 2a140273 	orr	w19, w19, w20
ffffc0200fe8: b8356ad3 	str	w19, [x22, x21]
ffffc0200fec: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0200ff0: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0200ff4: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0200ff8: d65f03c0 	ret

0000ffffc0200ffc <gicv2_cell_exit>:
ffffc0200ffc: b00000c1 	adrp	x1, 0xffffc0219000 <gicv2_init+0x24>
ffffc0201000: 52800003 	mov	w3, #0
ffffc0201004: d2840002 	mov	x2, #8192
ffffc0201008: 91400400 	add	x0, x0, #1, lsl #12     // =4096
ffffc020100c: f9400021 	ldr	x1, [x1]
ffffc0201010: f8462021 	ldur	x1, [x1, #98]
ffffc0201014: 140017f2 	b	0xffffc0206fdc <paging_destroy>

0000ffffc0201018 <gicv2_cell_init>:
ffffc0201018: 900000c1 	adrp	x1, 0xffffc0219000 <gicv2_init+0x3c>
ffffc020101c: 52800005 	mov	w5, #0
ffffc0201020: d28098a4 	mov	x4, #1221
ffffc0201024: d2840002 	mov	x2, #8192
ffffc0201028: f9400021 	ldr	x1, [x1]
ffffc020102c: 91400400 	add	x0, x0, #1, lsl #12     // =4096
ffffc0201030: f8462023 	ldur	x3, [x1, #98]
ffffc0201034: f8472021 	ldur	x1, [x1, #114]
ffffc0201038: 14001854 	b	0xffffc0207188 <paging_create>

0000ffffc020103c <gicv2_init>:
ffffc020103c: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_init+0x50>
ffffc0201040: f9407000 	ldr	x0, [x0, #224]
ffffc0201044: b94fe800 	ldr	w0, [x0, #4072]
ffffc0201048: d3441c00 	ubfx	x0, x0, #4, #4
ffffc020104c: 7100081f 	cmp	w0, #2
ffffc0201050: 54000321 	b.ne	0xffffc02010b4 <gicv2_init+0x78>
ffffc0201054: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0201058: d2840001 	mov	x1, #8192
ffffc020105c: 910003fd 	mov	x29, sp
ffffc0201060: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0201064: 900000d4 	adrp	x20, 0xffffc0219000 <gicv2_inject_phys_irq+0x8>
ffffc0201068: 900000b3 	adrp	x19, 0xffffc0215000 <gicv2_init+0x7c>
ffffc020106c: 91000273 	add	x19, x19, #0
ffffc0201070: f9400280 	ldr	x0, [x20]
ffffc0201074: f8462000 	ldur	x0, [x0, #98]
ffffc0201078: 940018e6 	bl	0xffffc0207410 <paging_map_device>
ffffc020107c: f9000a60 	str	x0, [x19, #16]
ffffc0201080: b50000a0 	cbnz	x0, 0xffffc0201094 <gicv2_init+0x58>
ffffc0201084: 12800160 	mov	w0, #-12
ffffc0201088: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020108c: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0201090: d65f03c0 	ret
ffffc0201094: f9400280 	ldr	x0, [x20]
ffffc0201098: d2840001 	mov	x1, #8192
ffffc020109c: f846a000 	ldur	x0, [x0, #106]
ffffc02010a0: 940018dc 	bl	0xffffc0207410 <paging_map_device>
ffffc02010a4: f9000660 	str	x0, [x19, #8]
ffffc02010a8: b4fffee0 	cbz	x0, 0xffffc0201084 <gicv2_init+0x48>
ffffc02010ac: 52800000 	mov	w0, #0
ffffc02010b0: 17fffff6 	b	0xffffc0201088 <gicv2_init+0x4c>
ffffc02010b4: 12800240 	mov	w0, #-19
ffffc02010b8: d65f03c0 	ret

0000ffffc02010bc <gicv2_inject_phys_irq>:
ffffc02010bc: 900000a2 	adrp	x2, 0xffffc0215000 <gicv2_inject_phys_irq+0x50>
ffffc02010c0: 12003c00 	and	w0, w0, #0xffff
ffffc02010c4: 71003c1f 	cmp	w0, #15
ffffc02010c8: f9407043 	ldr	x3, [x2, #224]
ffffc02010cc: 540000c8 	b.hi	0xffffc02010e4 <gicv2_inject_phys_irq+0x28>
ffffc02010d0: 92403c00 	and	x0, x0, #0xffff
ffffc02010d4: 52800021 	mov	w1, #1
ffffc02010d8: 913c8000 	add	x0, x0, #3872
ffffc02010dc: 38206861 	strb	w1, [x3, x0]
ffffc02010e0: d65f03c0 	ret
ffffc02010e4: 12001004 	and	w4, w0, #0x1f
ffffc02010e8: 53057c00 	lsr	w0, w0, #5
ffffc02010ec: 52800021 	mov	w1, #1
ffffc02010f0: d37e2800 	ubfiz	x0, x0, #2, #11
ffffc02010f4: 1ac42021 	lsl	w1, w1, w4
ffffc02010f8: 910e0004 	add	x4, x0, #896
ffffc02010fc: 91080000 	add	x0, x0, #512
ffffc0201100: b8246861 	str	w1, [x3, x4]
ffffc0201104: f9407042 	ldr	x2, [x2, #224]
ffffc0201108: b8206841 	str	w1, [x2, x0]
ffffc020110c: 17fffff5 	b	0xffffc02010e0 <gicv2_inject_phys_irq+0x24>

0000ffffc0201110 <gicv2_handle_dist_access>:
ffffc0201110: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0201114: 910003fd 	mov	x29, sp
ffffc0201118: f9400002 	ldr	x2, [x0]
ffffc020111c: f13f705f 	cmp	x2, #4060
ffffc0201120: 540002a8 	b.hi	0xffffc0201174 <gicv2_handle_dist_access+0x64>
ffffc0201124: f13f405f 	cmp	x2, #4048
ffffc0201128: 540000a2 	b.hs	0xffffc020113c <gicv2_handle_dist_access+0x2c>
ffffc020112c: f100105f 	cmp	x2, #4
ffffc0201130: 54000060 	b.eq	0xffffc020113c <gicv2_handle_dist_access+0x2c>
ffffc0201134: 54000128 	b.hi	0xffffc0201158 <gicv2_handle_dist_access+0x48>
ffffc0201138: b5000182 	cbnz	x2, 0xffffc0201168 <gicv2_handle_dist_access+0x58>
ffffc020113c: b9400c01 	ldr	w1, [x0, #12]
ffffc0201140: 35000141 	cbnz	w1, 0xffffc0201168 <gicv2_handle_dist_access+0x58>
ffffc0201144: aa0003e1 	mov	x1, x0
ffffc0201148: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_handle_dist_access+0x88>
ffffc020114c: f9407000 	ldr	x0, [x0, #224]
ffffc0201150: 94002061 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0201154: 14000005 	b	0xffffc0201168 <gicv2_handle_dist_access+0x58>
ffffc0201158: f100205f 	cmp	x2, #8
ffffc020115c: 54ffff00 	b.eq	0xffffc020113c <gicv2_handle_dist_access+0x2c>
ffffc0201160: f13c005f 	cmp	x2, #3840
ffffc0201164: 54000180 	b.eq	0xffffc0201194 <gicv2_handle_dist_access+0x84>
ffffc0201168: 52800020 	mov	w0, #1
ffffc020116c: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0201170: d65f03c0 	ret
ffffc0201174: f13f805f 	cmp	x2, #4064
ffffc0201178: 54ffff83 	b.lo	0xffffc0201168 <gicv2_handle_dist_access+0x58>
ffffc020117c: f13fb05f 	cmp	x2, #4076
ffffc0201180: 54fffde9 	b.ls	0xffffc020113c <gicv2_handle_dist_access+0x2c>
ffffc0201184: d13fc042 	sub	x2, x2, #4080
ffffc0201188: f100305f 	cmp	x2, #12
ffffc020118c: 54fffee8 	b.hi	0xffffc0201168 <gicv2_handle_dist_access+0x58>
ffffc0201190: 17ffffeb 	b	0xffffc020113c <gicv2_handle_dist_access+0x2c>
ffffc0201194: b9400c01 	ldr	w1, [x0, #12]
ffffc0201198: 34fffe81 	cbz	w1, 0xffffc0201168 <gicv2_handle_dist_access+0x58>
ffffc020119c: f9400800 	ldr	x0, [x0, #16]
ffffc02011a0: f90013bf 	str	xzr, [x29, #32]
ffffc02011a4: 53105c01 	ubfx	w1, w0, #16, #8
ffffc02011a8: 790053a1 	strh	w1, [x29, #40]
ffffc02011ac: 53186401 	ubfx	w1, w0, #24, #2
ffffc02011b0: 12000c00 	and	w0, w0, #0xf
ffffc02011b4: 390063a1 	strb	w1, [x29, #24]
ffffc02011b8: 790057a0 	strh	w0, [x29, #42]
ffffc02011bc: 910063a0 	add	x0, x29, #24
ffffc02011c0: 940005f4 	bl	0xffffc0202990 <gic_handle_sgir_write>
ffffc02011c4: 17ffffe9 	b	0xffffc0201168 <gicv2_handle_dist_access+0x58>

0000ffffc02011c8 <gicv2_handle_irq_target>:
ffffc02011c8: a9bb7bfd 	stp	x29, x30, [sp, #-80]!
ffffc02011cc: 910003fd 	mov	x29, sp
ffffc02011d0: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02011d4: aa0003f3 	mov	x19, x0
ffffc02011d8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02011dc: 51008020 	sub	w0, w1, #32
ffffc02011e0: a90363f7 	stp	x23, x24, [sp, #48]
ffffc02011e4: 710f6c1f 	cmp	w0, #987
ffffc02011e8: f90023f9 	str	x25, [sp, #64]
ffffc02011ec: 540000e9 	b.ls	0xffffc0201208 <gicv2_handle_irq_target+0x40>
ffffc02011f0: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_handle_irq_target+0x78>
ffffc02011f4: aa1303e1 	mov	x1, x19
ffffc02011f8: f9407000 	ldr	x0, [x0, #224]
ffffc02011fc: 94002036 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0201200: 52800020 	mov	w0, #1
ffffc0201204: 14000026 	b	0xffffc020129c <gicv2_handle_irq_target+0xd4>
ffffc0201208: d2840000 	mov	x0, #8192
ffffc020120c: 121e7437 	and	w23, w1, #0xfffffffc
ffffc0201210: f2a00020 	movk	x0, #1, lsl #16
ffffc0201214: 531d0421 	ubfiz	w1, w1, #3, #2
ffffc0201218: f2dfe000 	movk	x0, #65280, lsl #32
ffffc020121c: 52800014 	mov	w20, #0
ffffc0201220: 52800016 	mov	w22, #0
ffffc0201224: 52801ff9 	mov	w25, #255
ffffc0201228: f9480415 	ldr	x21, [x0, #4104]
ffffc020122c: f9400260 	ldr	x0, [x19]
ffffc0201230: 927ef400 	and	x0, x0, #0xfffffffffffffffc
ffffc0201234: f9000260 	str	x0, [x19]
ffffc0201238: f9400a60 	ldr	x0, [x19, #16]
ffffc020123c: 9ac12001 	lsl	x1, x0, x1
ffffc0201240: 52800080 	mov	w0, #4
ffffc0201244: b9000a60 	str	w0, [x19, #8]
ffffc0201248: f9000a61 	str	x1, [x19, #16]
ffffc020124c: 0b1402f8 	add	w24, w23, w20
ffffc0201250: aa1503e0 	mov	x0, x21
ffffc0201254: 2a1803e1 	mov	w1, w24
ffffc0201258: 9400047d 	bl	0xffffc020244c <irqchip_irq_in_cell>
ffffc020125c: 340002c0 	cbz	w0, 0xffffc02012b4 <gicv2_handle_irq_target+0xec>
ffffc0201260: 531d7280 	lsl	w0, w20, #3
ffffc0201264: 1ac02321 	lsl	w1, w25, w0
ffffc0201268: 2a0102d6 	orr	w22, w22, w1
ffffc020126c: b9400e61 	ldr	w1, [x19, #12]
ffffc0201270: 34000221 	cbz	w1, 0xffffc02012b4 <gicv2_handle_irq_target+0xec>
ffffc0201274: f9400a61 	ldr	x1, [x19, #16]
ffffc0201278: 9ac02421 	lsr	x1, x1, x0
ffffc020127c: aa1503e0 	mov	x0, x21
ffffc0201280: 97fffeae 	bl	0xffffc0200d38 <gicv2_targets_in_cell>
ffffc0201284: 35000180 	cbnz	w0, 0xffffc02012b4 <gicv2_handle_irq_target+0xec>
ffffc0201288: 2a1803e1 	mov	w1, w24
ffffc020128c: d0000040 	adrp	x0, 0xffffc020b000 <gicv2_handle_irq_target+0xec>
ffffc0201290: 912e4c00 	add	x0, x0, #2963
ffffc0201294: 94001624 	bl	0xffffc0206b24 <printk>
ffffc0201298: 12800000 	mov	w0, #-1
ffffc020129c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02012a0: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02012a4: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc02012a8: f94023f9 	ldr	x25, [sp, #64]
ffffc02012ac: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc02012b0: d65f03c0 	ret
ffffc02012b4: 11000694 	add	w20, w20, #1
ffffc02012b8: 7100129f 	cmp	w20, #4
ffffc02012bc: 54fffc81 	b.ne	0xffffc020124c <gicv2_handle_irq_target+0x84>
ffffc02012c0: b9400e61 	ldr	w1, [x19, #12]
ffffc02012c4: 2a1603f5 	mov	w21, w22
ffffc02012c8: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_handle_irq_target+0x150>
ffffc02012cc: 340003c1 	cbz	w1, 0xffffc0201344 <gicv2_handle_irq_target+0x17c>
ffffc02012d0: 900000b4 	adrp	x20, 0xffffc0215000 <gicv2_handle_irq_target+0x158>
ffffc02012d4: 9103a284 	add	x4, x20, #232
ffffc02012d8: f9800091 	prfm	pstl1strm, [x4]
ffffc02012dc: 885ffc81 	ldaxr	w1, [x4]
ffffc02012e0: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc02012e4: 88037c82 	stxr	w3, w2, [x4]
ffffc02012e8: 35ffffa3 	cbnz	w3, 0xffffc02012dc <gicv2_handle_irq_target+0x114>
ffffc02012ec: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc02012f0: 340000c2 	cbz	w2, 0xffffc0201308 <gicv2_handle_irq_target+0x140>
ffffc02012f4: d50320bf 	sevl
ffffc02012f8: d503205f 	wfe
ffffc02012fc: 485ffc83 	ldaxrh	w3, [x4]
ffffc0201300: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc0201304: 35ffffa2 	cbnz	w2, 0xffffc02012f8 <gicv2_handle_irq_target+0x130>
ffffc0201308: f9407000 	ldr	x0, [x0, #224]
ffffc020130c: 912002f7 	add	x23, x23, #2048
ffffc0201310: f9400a62 	ldr	x2, [x19, #16]
ffffc0201314: b8776801 	ldr	w1, [x0, x23]
ffffc0201318: 8a0202b5 	and	x21, x21, x2
ffffc020131c: 0a360036 	bic	w22, w1, w22
ffffc0201320: aa1303e1 	mov	x1, x19
ffffc0201324: aa1602b5 	orr	x21, x21, x22
ffffc0201328: f9000a75 	str	x21, [x19, #16]
ffffc020132c: 94001fea 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0201330: 7941d280 	ldrh	w0, [x20, #232]
ffffc0201334: 9103a281 	add	x1, x20, #232
ffffc0201338: 11000400 	add	w0, w0, #1
ffffc020133c: 489ffc20 	stlrh	w0, [x1]
ffffc0201340: 17ffffb0 	b	0xffffc0201200 <gicv2_handle_irq_target+0x38>
ffffc0201344: f9407000 	ldr	x0, [x0, #224]
ffffc0201348: aa1303e1 	mov	x1, x19
ffffc020134c: 94001fe2 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0201350: f9400a60 	ldr	x0, [x19, #16]
ffffc0201354: 8a150015 	and	x21, x0, x21
ffffc0201358: f9000a75 	str	x21, [x19, #16]
ffffc020135c: 17ffffa9 	b	0xffffc0201200 <gicv2_handle_irq_target+0x38>

0000ffffc0201360 <gicv2_has_pending_irqs>:
ffffc0201360: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_clear_pending_irqs+0xc>
ffffc0201364: 91000000 	add	x0, x0, #0
ffffc0201368: b9401802 	ldr	w2, [x0, #24]
ffffc020136c: f9400403 	ldr	x3, [x0, #8]
ffffc0201370: 52800000 	mov	w0, #0
ffffc0201374: 6b00005f 	cmp	w2, w0
ffffc0201378: 54000061 	b.ne	0xffffc0201384 <gicv2_has_pending_irqs+0x24>
ffffc020137c: 52800000 	mov	w0, #0
ffffc0201380: d65f03c0 	ret
ffffc0201384: 531e7401 	lsl	w1, w0, #2
ffffc0201388: 91040021 	add	x1, x1, #256
ffffc020138c: b8616861 	ldr	w1, [x3, x1]
ffffc0201390: 37e00061 	tbnz	w1, #28, 0xffffc020139c <gicv2_has_pending_irqs+0x3c>
ffffc0201394: 11000400 	add	w0, w0, #1
ffffc0201398: 17fffff7 	b	0xffffc0201374 <gicv2_has_pending_irqs+0x14>
ffffc020139c: 52800020 	mov	w0, #1
ffffc02013a0: 17fffff8 	b	0xffffc0201380 <gicv2_has_pending_irqs+0x20>

0000ffffc02013a4 <gicv2_clear_pending_irqs>:
ffffc02013a4: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_cpu_reset+0x18>
ffffc02013a8: 91000000 	add	x0, x0, #0
ffffc02013ac: b9401803 	ldr	w3, [x0, #24]
ffffc02013b0: f9400402 	ldr	x2, [x0, #8]
ffffc02013b4: 52800000 	mov	w0, #0
ffffc02013b8: 6b00007f 	cmp	w3, w0
ffffc02013bc: 54000061 	b.ne	0xffffc02013c8 <gicv2_clear_pending_irqs+0x24>
ffffc02013c0: b900f05f 	str	wzr, [x2, #240]
ffffc02013c4: d65f03c0 	ret
ffffc02013c8: 531e7401 	lsl	w1, w0, #2
ffffc02013cc: 11000400 	add	w0, w0, #1
ffffc02013d0: 91040021 	add	x1, x1, #256
ffffc02013d4: b821685f 	str	wzr, [x2, x1]
ffffc02013d8: 17fffff8 	b	0xffffc02013b8 <gicv2_clear_pending_irqs+0x14>

0000ffffc02013dc <gicv2_cpu_reset>:
ffffc02013dc: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc02013e0: 900000c0 	adrp	x0, 0xffffc0219000 <gicv2_cpu_init+0xc>
ffffc02013e4: 910003fd 	mov	x29, sp
ffffc02013e8: f9400000 	ldr	x0, [x0]
ffffc02013ec: 39416004 	ldrb	w4, [x0, #88]
ffffc02013f0: 97ffffed 	bl	0xffffc02013a4 <gicv2_clear_pending_irqs>
ffffc02013f4: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_cpu_init+0x10>
ffffc02013f8: f9407001 	ldr	x1, [x0, #224]
ffffc02013fc: 52800020 	mov	w0, #1
ffffc0201400: 1ac42000 	lsl	w0, w0, w4
ffffc0201404: 32003c02 	orr	w2, w0, #0xffff
ffffc0201408: 2a2003e0 	mvn	w0, w0
ffffc020140c: b9010022 	str	w2, [x1, #256]
ffffc0201410: 12103c00 	and	w0, w0, #0xffff0000
ffffc0201414: b9018020 	str	w0, [x1, #384]
ffffc0201418: 52bfffe0 	mov	w0, #-65536
ffffc020141c: b9038020 	str	w0, [x1, #896]
ffffc0201420: 900000a0 	adrp	x0, 0xffffc0215000 <gicv2_cpu_init+0x3c>
ffffc0201424: f9400400 	ldr	x0, [x0, #8]
ffffc0201428: b900081f 	str	wzr, [x0, #8]
ffffc020142c: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201430: d65f03c0 	ret

0000ffffc0201434 <gicv2_cpu_init>:
ffffc0201434: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0201438: 900000c1 	adrp	x1, 0xffffc0219000 <gicv2_cpu_init+0x64>
ffffc020143c: 900000a4 	adrp	x4, 0xffffc0215000 <gicv2_cpu_init+0x58>
ffffc0201440: aa0003e7 	mov	x7, x0
ffffc0201444: 910003fd 	mov	x29, sp
ffffc0201448: f9400021 	ldr	x1, [x1]
ffffc020144c: 52804026 	mov	w6, #513
ffffc0201450: f9407080 	ldr	x0, [x4, #224]
ffffc0201454: 39416022 	ldrb	w2, [x1, #88]
ffffc0201458: 52800021 	mov	w1, #1
ffffc020145c: 1ac22021 	lsl	w1, w1, w2
ffffc0201460: 900000a2 	adrp	x2, 0xffffc0215000 <gicv2_cpu_init+0x7c>
ffffc0201464: 91000045 	add	x5, x2, #0
ffffc0201468: 32003c21 	orr	w1, w1, #0xffff
ffffc020146c: b9010001 	str	w1, [x0, #256]
ffffc0201470: f94008a3 	ldr	x3, [x5, #16]
ffffc0201474: b9400060 	ldr	w0, [x3]
ffffc0201478: b9400461 	ldr	w1, [x3, #4]
ffffc020147c: b9000066 	str	w6, [x3]
ffffc0201480: 52801e06 	mov	w6, #240
ffffc0201484: b9000466 	str	w6, [x3, #4]
ffffc0201488: f94004a6 	ldr	x6, [x5, #8]
ffffc020148c: 53037c21 	lsr	w1, w1, #3
ffffc0201490: 53051021 	lsl	w1, w1, #27
ffffc0201494: b94004c3 	ldr	w3, [x6, #4]
ffffc0201498: 12001463 	and	w3, w3, #0x3f
ffffc020149c: 11000463 	add	w3, w3, #1
ffffc02014a0: b90018a3 	str	w3, [x5, #24]
ffffc02014a4: aa0403e5 	mov	x5, x4
ffffc02014a8: aa0203e4 	mov	x4, x2
ffffc02014ac: 36000040 	tbz	w0, #0, 0xffffc02014b4 <gicv2_cpu_init+0x80>
ffffc02014b0: 32000021 	orr	w1, w1, #0x1
ffffc02014b4: 36480040 	tbz	w0, #9, 0xffffc02014bc <gicv2_cpu_init+0x88>
ffffc02014b8: 32170021 	orr	w1, w1, #0x200
ffffc02014bc: b90008c1 	str	w1, [x6, #8]
ffffc02014c0: 52800028 	mov	w8, #1
ffffc02014c4: b90000c8 	str	w8, [x6]
ffffc02014c8: 97ffffb7 	bl	0xffffc02013a4 <gicv2_clear_pending_irqs>
ffffc02014cc: b97000e1 	ldr	w1, [x7, #12288]
ffffc02014d0: b93050e8 	str	w8, [x7, #12368]
ffffc02014d4: 71001c3f 	cmp	w1, #7
ffffc02014d8: 54000168 	b.hi	0xffffc0201504 <gicv2_cpu_init+0xd0>
ffffc02014dc: f94070a0 	ldr	x0, [x5, #224]
ffffc02014e0: 91000082 	add	x2, x4, #0
ffffc02014e4: b9480000 	ldr	w0, [x0, #2048]
ffffc02014e8: 12001c00 	and	w0, w0, #0xff
ffffc02014ec: 38214840 	strb	w0, [x2, w1, uxtw]
ffffc02014f0: 7100001f 	cmp	w0, #0
ffffc02014f4: 12800240 	mov	w0, #-19
ffffc02014f8: 1a9f0000 	csel	w0, w0, wzr, eq
ffffc02014fc: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201500: d65f03c0 	ret
ffffc0201504: 128002a0 	mov	w0, #-22
ffffc0201508: 17fffffd 	b	0xffffc02014fc <gicv2_cpu_init+0xc8>

0000ffffc020150c <gicv2_get_pending_irq>:
ffffc020150c: 900000a0 	adrp	x0, 0xffffc0215000 <gicv3_read_lr+0x4>
ffffc0201510: 91000000 	add	x0, x0, #0
ffffc0201514: 52800001 	mov	w1, #0
ffffc0201518: b9401803 	ldr	w3, [x0, #24]
ffffc020151c: f9400402 	ldr	x2, [x0, #8]
ffffc0201520: 6b01007f 	cmp	w3, w1
ffffc0201524: 54000061 	b.ne	0xffffc0201530 <gicv2_get_pending_irq+0x24>
ffffc0201528: 12800020 	mov	w0, #-2
ffffc020152c: 14000008 	b	0xffffc020154c <gicv2_get_pending_irq+0x40>
ffffc0201530: 531e7420 	lsl	w0, w1, #2
ffffc0201534: 91040000 	add	x0, x0, #256
ffffc0201538: 8b000044 	add	x4, x2, x0
ffffc020153c: b8606840 	ldr	w0, [x2, x0]
ffffc0201540: 36e00080 	tbz	w0, #28, 0xffffc0201550 <gicv2_get_pending_irq+0x44>
ffffc0201544: 12002400 	and	w0, w0, #0x3ff
ffffc0201548: b900009f 	str	wzr, [x4]
ffffc020154c: d65f03c0 	ret
ffffc0201550: 11000421 	add	w1, w1, #1
ffffc0201554: 17fffff3 	b	0xffffc0201520 <gicv2_get_pending_irq+0x14>

0000ffffc0201558 <gicv3_read_lr>:
ffffc0201558: 71003c1f 	cmp	w0, #15
ffffc020155c: 540004e8 	b.hi	0xffffc02015f8 <gicv3_read_lr+0xa0>
ffffc0201560: d0000041 	adrp	x1, 0xffffc020b000 <gicv3_read_lr+0x30>
ffffc0201564: 91072021 	add	x1, x1, #456
ffffc0201568: 38604820 	ldrb	w0, [x1, w0, uxtw]
ffffc020156c: 10000061 	adr	x1, #12
ffffc0201570: 8b208820 	add	x0, x1, w0, sxtb #2
ffffc0201574: d61f0000 	br	x0
ffffc0201578: d53ccc00 	mrs	x0, ICH_LR0_EL2
ffffc020157c: d65f03c0 	ret
ffffc0201580: d53ccc20 	mrs	x0, ICH_LR1_EL2
ffffc0201584: 17fffffe 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc0201588: d53ccc40 	mrs	x0, ICH_LR2_EL2
ffffc020158c: 17fffffc 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc0201590: d53ccc60 	mrs	x0, ICH_LR3_EL2
ffffc0201594: 17fffffa 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc0201598: d53ccc80 	mrs	x0, ICH_LR4_EL2
ffffc020159c: 17fffff8 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015a0: d53ccca0 	mrs	x0, ICH_LR5_EL2
ffffc02015a4: 17fffff6 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015a8: d53cccc0 	mrs	x0, ICH_LR6_EL2
ffffc02015ac: 17fffff4 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015b0: d53ccce0 	mrs	x0, ICH_LR7_EL2
ffffc02015b4: 17fffff2 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015b8: d53ccd00 	mrs	x0, ICH_LR8_EL2
ffffc02015bc: 17fffff0 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015c0: d53ccd20 	mrs	x0, ICH_LR9_EL2
ffffc02015c4: 17ffffee 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015c8: d53ccd40 	mrs	x0, ICH_LR10_EL2
ffffc02015cc: 17ffffec 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015d0: d53ccd60 	mrs	x0, ICH_LR11_EL2
ffffc02015d4: 17ffffea 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015d8: d53ccd80 	mrs	x0, ICH_LR12_EL2
ffffc02015dc: 17ffffe8 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015e0: d53ccda0 	mrs	x0, ICH_LR13_EL2
ffffc02015e4: 17ffffe6 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015e8: d53ccdc0 	mrs	x0, ICH_LR14_EL2
ffffc02015ec: 17ffffe4 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015f0: d53ccde0 	mrs	x0, ICH_LR15_EL2
ffffc02015f4: 17ffffe2 	b	0xffffc020157c <gicv3_read_lr+0x24>
ffffc02015f8: 92800000 	mov	x0, #-1
ffffc02015fc: 17ffffe0 	b	0xffffc020157c <gicv3_read_lr+0x24>

0000ffffc0201600 <gicv3_write_lr>:
ffffc0201600: 71003c1f 	cmp	w0, #15
ffffc0201604: 54000108 	b.hi	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201608: d0000042 	adrp	x2, 0xffffc020b000 <gicv3_write_lr+0x30>
ffffc020160c: 91076042 	add	x2, x2, #472
ffffc0201610: 38604840 	ldrb	w0, [x2, w0, uxtw]
ffffc0201614: 10000062 	adr	x2, #12
ffffc0201618: 8b208840 	add	x0, x2, w0, sxtb #2
ffffc020161c: d61f0000 	br	x0
ffffc0201620: d51ccc01 	msr	ICH_LR0_EL2, x1
ffffc0201624: d65f03c0 	ret
ffffc0201628: d51ccc21 	msr	ICH_LR1_EL2, x1
ffffc020162c: 17fffffe 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201630: d51ccc41 	msr	ICH_LR2_EL2, x1
ffffc0201634: 17fffffc 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201638: d51ccc61 	msr	ICH_LR3_EL2, x1
ffffc020163c: 17fffffa 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201640: d51ccc81 	msr	ICH_LR4_EL2, x1
ffffc0201644: 17fffff8 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201648: d51ccca1 	msr	ICH_LR5_EL2, x1
ffffc020164c: 17fffff6 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201650: d51cccc1 	msr	ICH_LR6_EL2, x1
ffffc0201654: 17fffff4 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201658: d51ccce1 	msr	ICH_LR7_EL2, x1
ffffc020165c: 17fffff2 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201660: d51ccd01 	msr	ICH_LR8_EL2, x1
ffffc0201664: 17fffff0 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201668: d51ccd21 	msr	ICH_LR9_EL2, x1
ffffc020166c: 17ffffee 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201670: d51ccd41 	msr	ICH_LR10_EL2, x1
ffffc0201674: 17ffffec 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201678: d51ccd61 	msr	ICH_LR11_EL2, x1
ffffc020167c: 17ffffea 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201680: d51ccd81 	msr	ICH_LR12_EL2, x1
ffffc0201684: 17ffffe8 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201688: d51ccda1 	msr	ICH_LR13_EL2, x1
ffffc020168c: 17ffffe6 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201690: d51ccdc1 	msr	ICH_LR14_EL2, x1
ffffc0201694: 17ffffe4 	b	0xffffc0201624 <gicv3_write_lr+0x24>
ffffc0201698: d51ccde1 	msr	ICH_LR15_EL2, x1
ffffc020169c: 17ffffe2 	b	0xffffc0201624 <gicv3_write_lr+0x24>

0000ffffc02016a0 <gicv3_read_iar_irqn>:
ffffc02016a0: d538cc00 	mrs	x0, ICC_IAR1_EL1
ffffc02016a4: 12005c00 	and	w0, w0, #0xffffff
ffffc02016a8: d65f03c0 	ret

0000ffffc02016ac <gicv3_eoi_irq>:
ffffc02016ac: 2a0003e0 	mov	w0, w0
ffffc02016b0: d518cc20 	msr	ICC_EOIR1_EL1, x0
ffffc02016b4: 34000041 	cbz	w1, 0xffffc02016bc <gicv3_eoi_irq+0x10>
ffffc02016b8: d518cb20 	msr	ICC_DIR_EL1, x0
ffffc02016bc: d65f03c0 	ret

0000ffffc02016c0 <gicv3_inject_irq>:
ffffc02016c0: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc02016c4: 12003c04 	and	w4, w0, #0xffff
ffffc02016c8: 910003fd 	mov	x29, sp
ffffc02016cc: d53ccba6 	mrs	x6, ICH_ELRSR_EL2
ffffc02016d0: 900000a5 	adrp	x5, 0xffffc0215000 <gicv3_inject_irq+0x60>
ffffc02016d4: 910080a5 	add	x5, x5, #32
ffffc02016d8: 12800003 	mov	w3, #-1
ffffc02016dc: 52800002 	mov	w2, #0
ffffc02016e0: b94000a0 	ldr	w0, [x5]
ffffc02016e4: 6b00005f 	cmp	w2, w0
ffffc02016e8: 540001e3 	b.lo	0xffffc0201724 <gicv3_inject_irq+0x64>
ffffc02016ec: 3100047f 	cmn	w3, #1
ffffc02016f0: 54000320 	b.eq	0xffffc0201754 <gicv3_inject_irq+0x94>
ffffc02016f4: 92403c80 	and	x0, x4, #0xffff
ffffc02016f8: 71003c9f 	cmp	w4, #15
ffffc02016fc: b2440802 	orr	x2, x0, #0x7000000000000000
ffffc0201700: d2ea0001 	mov	x1, #5764607523034234880
ffffc0201704: aa010001 	orr	x1, x0, x1
ffffc0201708: aa008040 	orr	x0, x2, x0, lsl #32
ffffc020170c: 9a818001 	csel	x1, x0, x1, hi
ffffc0201710: 2a0303e0 	mov	w0, w3
ffffc0201714: 97ffffbb 	bl	0xffffc0201600 <gicv3_write_lr>
ffffc0201718: 52800000 	mov	w0, #0
ffffc020171c: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201720: d65f03c0 	ret
ffffc0201724: 1ac224c0 	lsr	w0, w6, w2
ffffc0201728: 360000a0 	tbz	w0, #0, 0xffffc020173c <gicv3_inject_irq+0x7c>
ffffc020172c: 3100047f 	cmn	w3, #1
ffffc0201730: 1a821063 	csel	w3, w3, w2, ne
ffffc0201734: 11000442 	add	w2, w2, #1
ffffc0201738: 17ffffea 	b	0xffffc02016e0 <gicv3_inject_irq+0x20>
ffffc020173c: 2a0203e0 	mov	w0, w2
ffffc0201740: 97ffff86 	bl	0xffffc0201558 <gicv3_read_lr>
ffffc0201744: 6b00009f 	cmp	w4, w0
ffffc0201748: 54ffff61 	b.ne	0xffffc0201734 <gicv3_inject_irq+0x74>
ffffc020174c: 12800200 	mov	w0, #-17
ffffc0201750: 17fffff3 	b	0xffffc020171c <gicv3_inject_irq+0x5c>
ffffc0201754: 128001e0 	mov	w0, #-16
ffffc0201758: 17fffff1 	b	0xffffc020171c <gicv3_inject_irq+0x5c>

0000ffffc020175c <gicv3_enable_maint_irq>:
ffffc020175c: d53ccb01 	mrs	x1, ICH_HCR_EL2
ffffc0201760: 321f0022 	orr	w2, w1, #0x2
ffffc0201764: 7100001f 	cmp	w0, #0
ffffc0201768: 121e7821 	and	w1, w1, #0xfffffffd
ffffc020176c: 1a820021 	csel	w1, w1, w2, eq
ffffc0201770: d51ccb01 	msr	ICH_HCR_EL2, x1
ffffc0201774: d65f03c0 	ret

0000ffffc0201778 <gicv3_has_pending_irqs>:
ffffc0201778: 900000a3 	adrp	x3, 0xffffc0215000 <gicv3_has_pending_irqs+0x50>
ffffc020177c: 91008063 	add	x3, x3, #32
ffffc0201780: 52800002 	mov	w2, #0
ffffc0201784: b9400060 	ldr	w0, [x3]
ffffc0201788: 6b02001f 	cmp	w0, w2
ffffc020178c: 540000a8 	b.hi	0xffffc02017a0 <gicv3_has_pending_irqs+0x28>
ffffc0201790: 52800000 	mov	w0, #0
ffffc0201794: d65f03c0 	ret
ffffc0201798: 52800020 	mov	w0, #1
ffffc020179c: 1400000b 	b	0xffffc02017c8 <gicv3_has_pending_irqs+0x50>
ffffc02017a0: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc02017a4: 910003fd 	mov	x29, sp
ffffc02017a8: 2a0203e0 	mov	w0, w2
ffffc02017ac: 97ffff6b 	bl	0xffffc0201558 <gicv3_read_lr>
ffffc02017b0: b7f7ff40 	tbnz	x0, #62, 0xffffc0201798 <gicv3_has_pending_irqs+0x20>
ffffc02017b4: b9400060 	ldr	w0, [x3]
ffffc02017b8: 11000442 	add	w2, w2, #1
ffffc02017bc: 6b02001f 	cmp	w0, w2
ffffc02017c0: 54ffff48 	b.hi	0xffffc02017a8 <gicv3_has_pending_irqs+0x30>
ffffc02017c4: 52800000 	mov	w0, #0
ffffc02017c8: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc02017cc: d65f03c0 	ret

0000ffffc02017d0 <gicv3_get_pending_irq>:
ffffc02017d0: 900000a4 	adrp	x4, 0xffffc0215000 <gicv3_get_pending_irq+0x50>
ffffc02017d4: 91008084 	add	x4, x4, #32
ffffc02017d8: 52800002 	mov	w2, #0
ffffc02017dc: b9400080 	ldr	w0, [x4]
ffffc02017e0: 6b02001f 	cmp	w0, w2
ffffc02017e4: 54000128 	b.hi	0xffffc0201808 <gicv3_get_pending_irq+0x38>
ffffc02017e8: 12800020 	mov	w0, #-2
ffffc02017ec: d65f03c0 	ret
ffffc02017f0: b9400080 	ldr	w0, [x4]
ffffc02017f4: 11000442 	add	w2, w2, #1
ffffc02017f8: 6b02001f 	cmp	w0, w2
ffffc02017fc: 540000a8 	b.hi	0xffffc0201810 <gicv3_get_pending_irq+0x40>
ffffc0201800: 12800020 	mov	w0, #-2
ffffc0201804: 1400000b 	b	0xffffc0201830 <gicv3_get_pending_irq+0x60>
ffffc0201808: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020180c: 910003fd 	mov	x29, sp
ffffc0201810: 2a0203e0 	mov	w0, w2
ffffc0201814: 97ffff51 	bl	0xffffc0201558 <gicv3_read_lr>
ffffc0201818: aa0003e3 	mov	x3, x0
ffffc020181c: b6f7fea0 	tbz	x0, #62, 0xffffc02017f0 <gicv3_get_pending_irq+0x20>
ffffc0201820: 2a0203e0 	mov	w0, w2
ffffc0201824: d2800001 	mov	x1, #0
ffffc0201828: 97ffff76 	bl	0xffffc0201600 <gicv3_write_lr>
ffffc020182c: 2a0303e0 	mov	w0, w3
ffffc0201830: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201834: d65f03c0 	ret

0000ffffc0201838 <gicv3_inject_phys_irq>:
ffffc0201838: 12003c00 	and	w0, w0, #0xffff
ffffc020183c: 52800021 	mov	w1, #1
ffffc0201840: 12001002 	and	w2, w0, #0x1f
ffffc0201844: 1ac22021 	lsl	w1, w1, w2
ffffc0201848: 51008002 	sub	w2, w0, #32
ffffc020184c: 12003c42 	and	w2, w2, #0xffff
ffffc0201850: 710f6c5f 	cmp	w2, #987
ffffc0201854: 54000129 	b.ls	0xffffc0201878 <gicv3_inject_phys_irq+0x40>
ffffc0201858: d2840000 	mov	x0, #8192
ffffc020185c: f2a00020 	movk	x0, #1, lsl #16
ffffc0201860: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0201864: f9482800 	ldr	x0, [x0, #4176]
ffffc0201868: 91404000 	add	x0, x0, #16, lsl #12    // =65536
ffffc020186c: b9038001 	str	w1, [x0, #896]
ffffc0201870: b9020001 	str	w1, [x0, #512]
ffffc0201874: d65f03c0 	ret
ffffc0201878: 53057c00 	lsr	w0, w0, #5
ffffc020187c: 900000a2 	adrp	x2, 0xffffc0215000 <gicv3_get_cluster_target+0x4>
ffffc0201880: d37e2800 	ubfiz	x0, x0, #2, #11
ffffc0201884: f9407044 	ldr	x4, [x2, #224]
ffffc0201888: 910e0003 	add	x3, x0, #896
ffffc020188c: 91080000 	add	x0, x0, #512
ffffc0201890: b8236881 	str	w1, [x4, x3]
ffffc0201894: f9407042 	ldr	x2, [x2, #224]
ffffc0201898: b8206841 	str	w1, [x2, x0]
ffffc020189c: 17fffff6 	b	0xffffc0201874 <gicv3_inject_phys_irq+0x3c>

0000ffffc02018a0 <gicv3_handle_irq_target>:
ffffc02018a0: 52800020 	mov	w0, #1
ffffc02018a4: d65f03c0 	ret

0000ffffc02018a8 <gicv3_get_cpu_target>:
ffffc02018a8: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc02018ac: 900000e1 	adrp	x1, 0xffffc021d000 <gicv3_handle_redist_access+0x34>
ffffc02018b0: 91000021 	add	x1, x1, #0
ffffc02018b4: 8b010000 	add	x0, x0, x1
ffffc02018b8: 52800021 	mov	w1, #1
ffffc02018bc: f9582400 	ldr	x0, [x0, #12360]
ffffc02018c0: 1ac02020 	lsl	w0, w1, w0
ffffc02018c4: d65f03c0 	ret

0000ffffc02018c8 <gicv3_get_cluster_target>:
ffffc02018c8: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc02018cc: 900000e1 	adrp	x1, 0xffffc021d000 <gicv3_handle_redist_access+0x54>
ffffc02018d0: 91000021 	add	x1, x1, #0
ffffc02018d4: 8b010000 	add	x0, x0, x1
ffffc02018d8: f9582400 	ldr	x0, [x0, #12360]
ffffc02018dc: 92787c00 	and	x0, x0, #0xffffffff00
ffffc02018e0: 9260dc00 	and	x0, x0, #0xffffffff00ffffff
ffffc02018e4: d65f03c0 	ret

0000ffffc02018e8 <gicv3_handle_redist_access>:
ffffc02018e8: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc02018ec: d2802002 	mov	x2, #256
ffffc02018f0: f2a00022 	movk	x2, #1, lsl #16
ffffc02018f4: 910003fd 	mov	x29, sp
ffffc02018f8: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02018fc: aa0003f4 	mov	x20, x0
ffffc0201900: aa0103f3 	mov	x19, x1
ffffc0201904: f9400020 	ldr	x0, [x1]
ffffc0201908: eb02001f 	cmp	x0, x2
ffffc020190c: 54000140 	b.eq	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc0201910: 54000448 	b.hi	0xffffc0201998 <gicv3_handle_redist_access+0xb0>
ffffc0201914: f100401f 	cmp	x0, #16
ffffc0201918: 540000e0 	b.eq	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc020191c: 54000228 	b.hi	0xffffc0201960 <gicv3_handle_redist_access+0x78>
ffffc0201920: f100101f 	cmp	x0, #4
ffffc0201924: 54000160 	b.eq	0xffffc0201950 <gicv3_handle_redist_access+0x68>
ffffc0201928: f100201f 	cmp	x0, #8
ffffc020192c: 54000760 	b.eq	0xffffc0201a18 <gicv3_handle_redist_access+0x130>
ffffc0201930: b50002c0 	cbnz	x0, 0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201934: d2840000 	mov	x0, #8192
ffffc0201938: f9480681 	ldr	x1, [x20, #4104]
ffffc020193c: f2a00020 	movk	x0, #1, lsl #16
ffffc0201940: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0201944: f9480400 	ldr	x0, [x0, #4104]
ffffc0201948: eb00003f 	cmp	x1, x0
ffffc020194c: 540001e1 	b.ne	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201950: f9482a80 	ldr	x0, [x20, #4176]
ffffc0201954: aa1303e1 	mov	x1, x19
ffffc0201958: 94001e5f 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc020195c: 1400000b 	b	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201960: f103001f 	cmp	x0, #192
ffffc0201964: 54000700 	b.eq	0xffffc0201a44 <gicv3_handle_redist_access+0x15c>
ffffc0201968: 54000088 	b.hi	0xffffc0201978 <gicv3_handle_redist_access+0x90>
ffffc020196c: f100501f 	cmp	x0, #20
ffffc0201970: 540000c1 	b.ne	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201974: 17fffff0 	b	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc0201978: 929ff9e1 	mov	x1, #-65488
ffffc020197c: 8b010000 	add	x0, x0, x1
ffffc0201980: f100b01f 	cmp	x0, #44
ffffc0201984: 54fffe69 	b.ls	0xffffc0201950 <gicv3_handle_redist_access+0x68>
ffffc0201988: 52800020 	mov	w0, #1
ffffc020198c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0201990: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0201994: d65f03c0 	ret
ffffc0201998: d2807001 	mov	x1, #896
ffffc020199c: f2a00021 	movk	x1, #1, lsl #16
ffffc02019a0: eb01001f 	cmp	x0, x1
ffffc02019a4: 54fffc80 	b.eq	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc02019a8: 54000208 	b.hi	0xffffc02019e8 <gicv3_handle_redist_access+0x100>
ffffc02019ac: d2804001 	mov	x1, #512
ffffc02019b0: f2a00021 	movk	x1, #1, lsl #16
ffffc02019b4: eb01001f 	cmp	x0, x1
ffffc02019b8: 54fffbe0 	b.eq	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc02019bc: 540000a8 	b.hi	0xffffc02019d0 <gicv3_handle_redist_access+0xe8>
ffffc02019c0: d2803001 	mov	x1, #384
ffffc02019c4: f2a00021 	movk	x1, #1, lsl #16
ffffc02019c8: eb01001f 	cmp	x0, x1
ffffc02019cc: 17ffffe9 	b	0xffffc0201970 <gicv3_handle_redist_access+0x88>
ffffc02019d0: d2805001 	mov	x1, #640
ffffc02019d4: f2a00021 	movk	x1, #1, lsl #16
ffffc02019d8: eb01001f 	cmp	x0, x1
ffffc02019dc: 54fffac0 	b.eq	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc02019e0: 91020021 	add	x1, x1, #128
ffffc02019e4: 17fffff9 	b	0xffffc02019c8 <gicv3_handle_redist_access+0xe0>
ffffc02019e8: d2808001 	mov	x1, #1024
ffffc02019ec: f2a00021 	movk	x1, #1, lsl #16
ffffc02019f0: eb01001f 	cmp	x0, x1
ffffc02019f4: 54fffca3 	b.lo	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc02019f8: 91007021 	add	x1, x1, #28
ffffc02019fc: eb01001f 	cmp	x0, x1
ffffc0201a00: 54fff9a9 	b.ls	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc0201a04: d1404000 	sub	x0, x0, #16, lsl #12    // =65536
ffffc0201a08: d1300000 	sub	x0, x0, #3072
ffffc0201a0c: f100101f 	cmp	x0, #4
ffffc0201a10: 54fffbc8 	b.hi	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201a14: 17ffffc8 	b	0xffffc0201934 <gicv3_handle_redist_access+0x4c>
ffffc0201a18: f9482a80 	ldr	x0, [x20, #4176]
ffffc0201a1c: 94001e2e 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0201a20: 900000a0 	adrp	x0, 0xffffc0215000 <gicv3_handle_irq_route+0x24>
ffffc0201a24: b9500281 	ldr	w1, [x20, #4096]
ffffc0201a28: b9402400 	ldr	w0, [x0, #36]
ffffc0201a2c: 6b00003f 	cmp	w1, w0
ffffc0201a30: 54fffac1 	b.ne	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201a34: f9400a60 	ldr	x0, [x19, #16]
ffffc0201a38: b27c0000 	orr	x0, x0, #0x10
ffffc0201a3c: f9000a60 	str	x0, [x19, #16]
ffffc0201a40: 17ffffd2 	b	0xffffc0201988 <gicv3_handle_redist_access+0xa0>
ffffc0201a44: f900083f 	str	xzr, [x1, #16]
ffffc0201a48: 17ffffd0 	b	0xffffc0201988 <gicv3_handle_redist_access+0xa0>

0000ffffc0201a4c <gicv3_handle_irq_route>:
ffffc0201a4c: b9400802 	ldr	w2, [x0, #8]
ffffc0201a50: 7100105f 	cmp	w2, #4
ffffc0201a54: 540000c1 	b.ne	0xffffc0201a6c <gicv3_handle_irq_route+0x20>
ffffc0201a58: f9400002 	ldr	x2, [x0]
ffffc0201a5c: f240085f 	tst	x2, #0x7
ffffc0201a60: 54000060 	b.eq	0xffffc0201a6c <gicv3_handle_irq_route+0x20>
ffffc0201a64: 52800020 	mov	w0, #1
ffffc0201a68: d65f03c0 	ret
ffffc0201a6c: 51008022 	sub	w2, w1, #32
ffffc0201a70: 710f6c5f 	cmp	w2, #987
ffffc0201a74: 54ffff88 	b.hi	0xffffc0201a64 <gicv3_handle_irq_route+0x18>
ffffc0201a78: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0201a7c: 910003fd 	mov	x29, sp
ffffc0201a80: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0201a84: aa0003f4 	mov	x20, x0
ffffc0201a88: d2840000 	mov	x0, #8192
ffffc0201a8c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0201a90: f2a00020 	movk	x0, #1, lsl #16
ffffc0201a94: f9001bf7 	str	x23, [sp, #48]
ffffc0201a98: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0201a9c: 2a0103f3 	mov	w19, w1
ffffc0201aa0: f9480415 	ldr	x21, [x0, #4104]
ffffc0201aa4: aa1503e0 	mov	x0, x21
ffffc0201aa8: 94000269 	bl	0xffffc020244c <irqchip_irq_in_cell>
ffffc0201aac: 34000280 	cbz	w0, 0xffffc0201afc <gicv3_handle_irq_route+0xb0>
ffffc0201ab0: b9400e80 	ldr	w0, [x20, #12]
ffffc0201ab4: 35000280 	cbnz	w0, 0xffffc0201b04 <gicv3_handle_irq_route+0xb8>
ffffc0201ab8: 900000a0 	adrp	x0, 0xffffc0215000 <gicv3_handle_irq_route+0xbc>
ffffc0201abc: 531d7273 	lsl	w19, w19, #3
ffffc0201ac0: 91401a73 	add	x19, x19, #6, lsl #12   // =24576
ffffc0201ac4: f9407000 	ldr	x0, [x0, #224]
ffffc0201ac8: f8736800 	ldr	x0, [x0, x19]
ffffc0201acc: f9000a80 	str	x0, [x20, #16]
ffffc0201ad0: 1400000b 	b	0xffffc0201afc <gicv3_handle_irq_route+0xb0>
ffffc0201ad4: 8b013ac1 	add	x1, x22, x1, lsl #14
ffffc0201ad8: f9400a82 	ldr	x2, [x20, #16]
ffffc0201adc: f9582421 	ldr	x1, [x1, #12360]
ffffc0201ae0: 8a170021 	and	x1, x1, x23
ffffc0201ae4: eb02003f 	cmp	x1, x2
ffffc0201ae8: 54000181 	b.ne	0xffffc0201b18 <gicv3_handle_irq_route+0xcc>
ffffc0201aec: 900000a0 	adrp	x0, 0xffffc0215000 <gicv3_handle_irq_route+0xf0>
ffffc0201af0: aa1403e1 	mov	x1, x20
ffffc0201af4: f9407000 	ldr	x0, [x0, #224]
ffffc0201af8: 94001df7 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0201afc: 52800020 	mov	w0, #1
ffffc0201b00: 14000013 	b	0xffffc0201b4c <gicv3_handle_irq_route+0x100>
ffffc0201b04: 900000f6 	adrp	x22, 0xffffc021d000 <gicv3_send_sgi+0x14>
ffffc0201b08: b2405ff7 	mov	x23, #16777215
ffffc0201b0c: 910002d6 	add	x22, x22, #0
ffffc0201b10: 12800000 	mov	w0, #-1
ffffc0201b14: f2c01ff7 	movk	x23, #255, lsl #32
ffffc0201b18: f94856a1 	ldr	x1, [x21, #4264]
ffffc0201b1c: 12800002 	mov	w2, #-1
ffffc0201b20: 9400184a 	bl	0xffffc0207c48 <next_cpu>
ffffc0201b24: 2a0003e1 	mov	w1, w0
ffffc0201b28: f94856a2 	ldr	x2, [x21, #4264]
ffffc0201b2c: f9400042 	ldr	x2, [x2]
ffffc0201b30: eb02003f 	cmp	x1, x2
ffffc0201b34: 54fffd09 	b.ls	0xffffc0201ad4 <gicv3_handle_irq_route+0x88>
ffffc0201b38: 2a1303e1 	mov	w1, w19
ffffc0201b3c: d0000040 	adrp	x0, 0xffffc020b000 <gicv3_send_sgi+0x4>
ffffc0201b40: 912e4c00 	add	x0, x0, #2963
ffffc0201b44: 940013f8 	bl	0xffffc0206b24 <printk>
ffffc0201b48: 12800000 	mov	w0, #-1
ffffc0201b4c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0201b50: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0201b54: f9401bf7 	ldr	x23, [sp, #48]
ffffc0201b58: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0201b5c: d65f03c0 	ret

0000ffffc0201b60 <gicv3_send_sgi>:
ffffc0201b60: 79402401 	ldrh	w1, [x0, #18]
ffffc0201b64: 71003c3f 	cmp	w1, #15
ffffc0201b68: 540004e8 	b.hi	0xffffc0201c04 <gicv3_send_sgi+0xa4>
ffffc0201b6c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0201b70: 910003fd 	mov	x29, sp
ffffc0201b74: f9000bf3 	str	x19, [sp, #16]
ffffc0201b78: aa0003f3 	mov	x19, x0
ffffc0201b7c: 39400000 	ldrb	w0, [x0]
ffffc0201b80: 7100081f 	cmp	w0, #2
ffffc0201b84: 54000360 	b.eq	0xffffc0201bf0 <gicv3_send_sgi+0x90>
ffffc0201b88: 79402260 	ldrh	w0, [x19, #16]
ffffc0201b8c: f9400662 	ldr	x2, [x19, #8]
ffffc0201b90: 92403c00 	and	x0, x0, #0xffff
ffffc0201b94: 79402663 	ldrh	w3, [x19, #18]
ffffc0201b98: d348fc41 	lsr	x1, x2, #8
ffffc0201b9c: d3680c63 	ubfiz	x3, x3, #24, #4
ffffc0201ba0: d3701c21 	ubfiz	x1, x1, #16, #8
ffffc0201ba4: aa030021 	orr	x1, x1, x3
ffffc0201ba8: d360fc43 	lsr	x3, x2, #32
ffffc0201bac: d350fc42 	lsr	x2, x2, #16
ffffc0201bb0: d3501c63 	ubfiz	x3, x3, #48, #8
ffffc0201bb4: d3601c42 	ubfiz	x2, x2, #32, #8
ffffc0201bb8: aa020062 	orr	x2, x3, x2
ffffc0201bbc: aa020021 	orr	x1, x1, x2
ffffc0201bc0: aa000021 	orr	x1, x1, x0
ffffc0201bc4: 39400260 	ldrb	w0, [x19]
ffffc0201bc8: 7100041f 	cmp	w0, #1
ffffc0201bcc: 54000041 	b.ne	0xffffc0201bd4 <gicv3_send_sgi+0x74>
ffffc0201bd0: b2580021 	orr	x1, x1, #0x10000000000
ffffc0201bd4: d5033b9f 	dsb	ish
ffffc0201bd8: d518cba1 	msr	ICC_SGI1R_EL1, x1
ffffc0201bdc: d5033fdf 	isb
ffffc0201be0: 52800000 	mov	w0, #0
ffffc0201be4: f9400bf3 	ldr	x19, [sp, #16]
ffffc0201be8: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0201bec: d65f03c0 	ret
ffffc0201bf0: 94000480 	bl	0xffffc0202df0 <phys_processor_id>
ffffc0201bf4: 52800021 	mov	w1, #1
ffffc0201bf8: 1ac02020 	lsl	w0, w1, w0
ffffc0201bfc: 12003c00 	and	w0, w0, #0xffff
ffffc0201c00: 17ffffe3 	b	0xffffc0201b8c <gicv3_send_sgi+0x2c>
ffffc0201c04: 128002a0 	mov	w0, #-22
ffffc0201c08: d65f03c0 	ret

0000ffffc0201c0c <gicv3_adjust_irq_target>:
ffffc0201c0c: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0201c10: d37d3c21 	ubfiz	x1, x1, #3, #16
ffffc0201c14: 12800002 	mov	w2, #-1
ffffc0201c18: 910003fd 	mov	x29, sp
ffffc0201c1c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0201c20: aa0003f5 	mov	x21, x0
ffffc0201c24: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0201c28: 91401833 	add	x19, x1, #6, lsl #12    // =24576
ffffc0201c2c: 900000a0 	adrp	x0, 0xffffc0215000 <gicv3_adjust_irq_target+0x70>
ffffc0201c30: f94856a1 	ldr	x1, [x21, #4264]
ffffc0201c34: f9407014 	ldr	x20, [x0, #224]
ffffc0201c38: 2a0203e0 	mov	w0, w2
ffffc0201c3c: 94001803 	bl	0xffffc0207c48 <next_cpu>
ffffc0201c40: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc0201c44: 900000e1 	adrp	x1, 0xffffc021d000 <gicv3_cell_init+0xc>
ffffc0201c48: 91000021 	add	x1, x1, #0
ffffc0201c4c: 8b010000 	add	x0, x0, x1
ffffc0201c50: f8736a81 	ldr	x1, [x20, x19]
ffffc0201c54: 92409c21 	and	x1, x1, #0xffffffffff
ffffc0201c58: 9260dc21 	and	x1, x1, #0xffffffff00ffffff
ffffc0201c5c: f9582416 	ldr	x22, [x0, #12360]
ffffc0201c60: aa1503e0 	mov	x0, x21
ffffc0201c64: 94000467 	bl	0xffffc0202e00 <arm_cpu_by_mpidr>
ffffc0201c68: f94856a1 	ldr	x1, [x21, #4264]
ffffc0201c6c: f9400022 	ldr	x2, [x1]
ffffc0201c70: eb20405f 	cmp	x2, w0, uxtw
ffffc0201c74: 54000103 	b.lo	0xffffc0201c94 <gicv3_adjust_irq_target+0x88>
ffffc0201c78: 53067c02 	lsr	w2, w0, #6
ffffc0201c7c: 91002021 	add	x1, x1, #8
ffffc0201c80: 12001400 	and	w0, w0, #0x3f
ffffc0201c84: d37df042 	lsl	x2, x2, #3
ffffc0201c88: f8626821 	ldr	x1, [x1, x2]
ffffc0201c8c: 9ac02420 	lsr	x0, x1, x0
ffffc0201c90: 37000040 	tbnz	w0, #0, 0xffffc0201c98 <gicv3_adjust_irq_target+0x8c>
ffffc0201c94: f8336a96 	str	x22, [x20, x19]
ffffc0201c98: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0201c9c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0201ca0: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0201ca4: d65f03c0 	ret

0000ffffc0201ca8 <gicv3_cell_init>:
ffffc0201ca8: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0201cac: 910003fd 	mov	x29, sp
ffffc0201cb0: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0201cb4: 900000f5 	adrp	x21, 0xffffc021d000 <gicv3_cell_init+0x7c>
ffffc0201cb8: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0201cbc: 900000b6 	adrp	x22, 0xffffc0215000 <gicv3_cell_init+0x64>
ffffc0201cc0: aa0003f8 	mov	x24, x0
ffffc0201cc4: 910002b5 	add	x21, x21, #0
ffffc0201cc8: 910082d6 	add	x22, x22, #32
ffffc0201ccc: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0201cd0: 900000d7 	adrp	x23, 0xffffc0219000 <gicv3_cell_init+0x88>
ffffc0201cd4: 52800014 	mov	w20, #0
ffffc0201cd8: f94002e0 	ldr	x0, [x23]
ffffc0201cdc: 91080000 	add	x0, x0, #512
ffffc0201ce0: b853a000 	ldur	w0, [x0, #-198]
ffffc0201ce4: 6b000e9f 	cmp	w20, w0, lsl #3
ffffc0201ce8: 540000e3 	b.lo	0xffffc0201d04 <gicv3_cell_init+0x5c>
ffffc0201cec: 52800000 	mov	w0, #0
ffffc0201cf0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0201cf4: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0201cf8: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0201cfc: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0201d00: d65f03c0 	ret
ffffc0201d04: 2a1403f3 	mov	w19, w20
ffffc0201d08: aa1303e0 	mov	x0, x19
ffffc0201d0c: 94001847 	bl	0xffffc0207e28 <cpu_id_valid>
ffffc0201d10: 340001a0 	cbz	w0, 0xffffc0201d44 <gicv3_cell_init+0x9c>
ffffc0201d14: 8b133aa4 	add	x4, x21, x19, lsl #14
ffffc0201d18: b9400ac0 	ldr	w0, [x22, #8]
ffffc0201d1c: 91400884 	add	x4, x4, #2, lsl #12     // =8192
ffffc0201d20: 90000003 	adrp	x3, 0xffffc0201000 <gicv3_cell_init+0x78>
ffffc0201d24: 7100101f 	cmp	w0, #4
ffffc0201d28: d2a00040 	mov	x0, #131072
ffffc0201d2c: f9482c81 	ldr	x1, [x4, #4184]
ffffc0201d30: d2a00082 	mov	x2, #262144
ffffc0201d34: 9123a063 	add	x3, x3, #2280
ffffc0201d38: 9a800042 	csel	x2, x2, x0, eq
ffffc0201d3c: aa1803e0 	mov	x0, x24
ffffc0201d40: 94001cc8 	bl	0xffffc0209060 <mmio_region_register>
ffffc0201d44: 11000694 	add	w20, w20, #1
ffffc0201d48: 17ffffe4 	b	0xffffc0201cd8 <gicv3_cell_init+0x30>

0000ffffc0201d4c <gicv3_clear_pending_irqs>:
ffffc0201d4c: 900000a3 	adrp	x3, 0xffffc0215000 <gicv3_clear_pending_irqs+0x50>
ffffc0201d50: 91008063 	add	x3, x3, #32
ffffc0201d54: 52800004 	mov	w4, #0
ffffc0201d58: b9400060 	ldr	w0, [x3]
ffffc0201d5c: 6b04001f 	cmp	w0, w4
ffffc0201d60: 540001e8 	b.hi	0xffffc0201d9c <gicv3_clear_pending_irqs+0x50>
ffffc0201d64: b9400c61 	ldr	w1, [x3, #12]
ffffc0201d68: 7100103f 	cmp	w1, #4
ffffc0201d6c: 54000169 	b.ls	0xffffc0201d98 <gicv3_clear_pending_irqs+0x4c>
ffffc0201d70: d2800000 	mov	x0, #0
ffffc0201d74: d51cc900 	msr	ICH_AP1R0_EL2, x0
ffffc0201d78: 7100143f 	cmp	w1, #5
ffffc0201d7c: 540000e0 	b.eq	0xffffc0201d98 <gicv3_clear_pending_irqs+0x4c>
ffffc0201d80: d51cc920 	msr	ICH_AP1R1_EL2, x0
ffffc0201d84: 7100183f 	cmp	w1, #6
ffffc0201d88: 54000080 	b.eq	0xffffc0201d98 <gicv3_clear_pending_irqs+0x4c>
ffffc0201d8c: d51cc940 	msr	ICH_AP1R2_EL2, x0
ffffc0201d90: d51cc960 	msr	ICH_AP1R3_EL2, x0
ffffc0201d94: d65f03c0 	ret
ffffc0201d98: d65f03c0 	ret
ffffc0201d9c: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0201da0: 910003fd 	mov	x29, sp
ffffc0201da4: 2a0403e0 	mov	w0, w4
ffffc0201da8: d2800001 	mov	x1, #0
ffffc0201dac: 97fffe15 	bl	0xffffc0201600 <gicv3_write_lr>
ffffc0201db0: b9400060 	ldr	w0, [x3]
ffffc0201db4: 11000484 	add	w4, w4, #1
ffffc0201db8: 6b04001f 	cmp	w0, w4
ffffc0201dbc: 54ffff48 	b.hi	0xffffc0201da4 <gicv3_clear_pending_irqs+0x58>
ffffc0201dc0: b9400c61 	ldr	w1, [x3, #12]
ffffc0201dc4: 7100103f 	cmp	w1, #4
ffffc0201dc8: 54000149 	b.ls	0xffffc0201df0 <gicv3_clear_pending_irqs+0xa4>
ffffc0201dcc: d2800000 	mov	x0, #0
ffffc0201dd0: d51cc900 	msr	ICH_AP1R0_EL2, x0
ffffc0201dd4: 7100143f 	cmp	w1, #5
ffffc0201dd8: 540000c0 	b.eq	0xffffc0201df0 <gicv3_clear_pending_irqs+0xa4>
ffffc0201ddc: d51cc920 	msr	ICH_AP1R1_EL2, x0
ffffc0201de0: 7100183f 	cmp	w1, #6
ffffc0201de4: 54000060 	b.eq	0xffffc0201df0 <gicv3_clear_pending_irqs+0xa4>
ffffc0201de8: d51cc940 	msr	ICH_AP1R2_EL2, x0
ffffc0201dec: d51cc960 	msr	ICH_AP1R3_EL2, x0
ffffc0201df0: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201df4: d65f03c0 	ret

0000ffffc0201df8 <gicv3_cpu_reset>:
ffffc0201df8: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0201dfc: 900000c1 	adrp	x1, 0xffffc0219000 <gicv3_cpu_init+0x10>
ffffc0201e00: 910003fd 	mov	x29, sp
ffffc0201e04: f9400021 	ldr	x1, [x1]
ffffc0201e08: f9582805 	ldr	x5, [x0, #12368]
ffffc0201e0c: 39416026 	ldrb	w6, [x1, #88]
ffffc0201e10: 97ffffcf 	bl	0xffffc0201d4c <gicv3_clear_pending_irqs>
ffffc0201e14: 914040a1 	add	x1, x5, #16, lsl #12    // =65536
ffffc0201e18: 52800020 	mov	w0, #1
ffffc0201e1c: 1ac62000 	lsl	w0, w0, w6
ffffc0201e20: 32003c02 	orr	w2, w0, #0xffff
ffffc0201e24: 2a2003e0 	mvn	w0, w0
ffffc0201e28: b9010022 	str	w2, [x1, #256]
ffffc0201e2c: 12103c00 	and	w0, w0, #0xffff0000
ffffc0201e30: b9018020 	str	w0, [x1, #384]
ffffc0201e34: 52bfffe0 	mov	w0, #-65536
ffffc0201e38: b9038020 	str	w0, [x1, #896]
ffffc0201e3c: d2800000 	mov	x0, #0
ffffc0201e40: d51ccbe0 	msr	ICH_VMCR_EL2, x0
ffffc0201e44: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201e48: d65f03c0 	ret

0000ffffc0201e4c <gicv3_cpu_init>:
ffffc0201e4c: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0201e50: 900000c1 	adrp	x1, 0xffffc0219000 <gicv3_cpu_init+0x64>
ffffc0201e54: 900000a2 	adrp	x2, 0xffffc0215000 <gicv3_cpu_init+0x58>
ffffc0201e58: d2a00083 	mov	x3, #262144
ffffc0201e5c: 910003fd 	mov	x29, sp
ffffc0201e60: f9400021 	ldr	x1, [x1]
ffffc0201e64: d2a00044 	mov	x4, #131072
ffffc0201e68: f847a025 	ldur	x5, [x1, #122]
ffffc0201e6c: 39416029 	ldrb	w9, [x1, #88]
ffffc0201e70: 91008041 	add	x1, x2, #32
ffffc0201e74: b9400828 	ldr	w8, [x1, #8]
ffffc0201e78: f9400821 	ldr	x1, [x1, #16]
ffffc0201e7c: 7100111f 	cmp	w8, #4
ffffc0201e80: 9a831084 	csel	x4, x4, x3, ne
ffffc0201e84: f9582403 	ldr	x3, [x0, #12360]
ffffc0201e88: cb0100a5 	sub	x5, x5, x1
ffffc0201e8c: 12005c67 	and	w7, w3, #0xffffff
ffffc0201e90: d360fc66 	lsr	x6, x3, #32
ffffc0201e94: 2a0660e6 	orr	w6, w7, w6, lsl #24
ffffc0201e98: 91403027 	add	x7, x1, #12, lsl #12    // =49152
ffffc0201e9c: 8b05002a 	add	x10, x1, x5
ffffc0201ea0: b97fe8e7 	ldr	w7, [x7, #16360]
ffffc0201ea4: d3441ce7 	ubfx	x7, x7, #4, #4
ffffc0201ea8: 6b07011f 	cmp	w8, w7
ffffc0201eac: 540000c1 	b.ne	0xffffc0201ec4 <gicv3_cpu_init+0x78>
ffffc0201eb0: f9400427 	ldr	x7, [x1, #8]
ffffc0201eb4: eb4780df 	cmp	x6, x7, lsr #32
ffffc0201eb8: 54000181 	b.ne	0xffffc0201ee8 <gicv3_cpu_init+0x9c>
ffffc0201ebc: f9182801 	str	x1, [x0, #12368]
ffffc0201ec0: f9182c0a 	str	x10, [x0, #12376]
ffffc0201ec4: f9582804 	ldr	x4, [x0, #12368]
ffffc0201ec8: b5000164 	cbnz	x4, 0xffffc0201ef4 <gicv3_cpu_init+0xa8>
ffffc0201ecc: b9700001 	ldr	w1, [x0, #12288]
ffffc0201ed0: d0000040 	adrp	x0, 0xffffc020b000 <gicv3_cpu_init+0xac>
ffffc0201ed4: 912eec00 	add	x0, x0, #3003
ffffc0201ed8: 94001313 	bl	0xffffc0206b24 <printk>
ffffc0201edc: 12800240 	mov	w0, #-19
ffffc0201ee0: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0201ee4: d65f03c0 	ret
ffffc0201ee8: 8b040021 	add	x1, x1, x4
ffffc0201eec: 3627fd67 	tbz	w7, #4, 0xffffc0201e98 <gicv3_cpu_init+0x4c>
ffffc0201ef0: 17fffff5 	b	0xffffc0201ec4 <gicv3_cpu_init+0x78>
ffffc0201ef4: 92401c63 	and	x3, x3, #0xff
ffffc0201ef8: f1003c7f 	cmp	x3, #15
ffffc0201efc: 54000448 	b.hi	0xffffc0201f84 <gicv3_cpu_init+0x138>
ffffc0201f00: 91404021 	add	x1, x1, #16, lsl #12    // =65536
ffffc0201f04: 52800020 	mov	w0, #1
ffffc0201f08: 1ac92000 	lsl	w0, w0, w9
ffffc0201f0c: 32003c00 	orr	w0, w0, #0xffff
ffffc0201f10: b9010020 	str	w0, [x1, #256]
ffffc0201f14: d538cc86 	mrs	x6, ICC_CTLR_EL1
ffffc0201f18: d2800040 	mov	x0, #2
ffffc0201f1c: d518cc80 	msr	ICC_CTLR_EL1, x0
ffffc0201f20: d5384605 	mrs	x5, ICC_PMR_EL1
ffffc0201f24: d2801e00 	mov	x0, #240
ffffc0201f28: d5184600 	msr	ICC_PMR_EL1, x0
ffffc0201f2c: d538cce7 	mrs	x7, ICC_IGRPEN1_EL1
ffffc0201f30: d2800020 	mov	x0, #1
ffffc0201f34: d518cce0 	msr	ICC_IGRPEN1_EL1, x0
ffffc0201f38: d53ccb20 	mrs	x0, ICH_VTR_EL2
ffffc0201f3c: 12000c01 	and	w1, w0, #0xf
ffffc0201f40: 531d7c00 	lsr	w0, w0, #29
ffffc0201f44: 11000400 	add	w0, w0, #1
ffffc0201f48: 91008043 	add	x3, x2, #32
ffffc0201f4c: 11000421 	add	w1, w1, #1
ffffc0201f50: b9002041 	str	w1, [x2, #32]
ffffc0201f54: b9000c60 	str	w0, [x3, #12]
ffffc0201f58: 97ffff7d 	bl	0xffffc0201d4c <gicv3_clear_pending_irqs>
ffffc0201f5c: 53081ca0 	lsl	w0, w5, #24
ffffc0201f60: 36000047 	tbz	w7, #0, 0xffffc0201f68 <gicv3_cpu_init+0x11c>
ffffc0201f64: 321f0000 	orr	w0, w0, #0x2
ffffc0201f68: 36080046 	tbz	w6, #1, 0xffffc0201f70 <gicv3_cpu_init+0x124>
ffffc0201f6c: 32170000 	orr	w0, w0, #0x200
ffffc0201f70: d51ccbe0 	msr	ICH_VMCR_EL2, x0
ffffc0201f74: d2800020 	mov	x0, #1
ffffc0201f78: d51ccb00 	msr	ICH_HCR_EL2, x0
ffffc0201f7c: 52800000 	mov	w0, #0
ffffc0201f80: 17ffffd8 	b	0xffffc0201ee0 <gicv3_cpu_init+0x94>
ffffc0201f84: 12800080 	mov	w0, #-5
ffffc0201f88: 17ffffd6 	b	0xffffc0201ee0 <gicv3_cpu_init+0x94>

0000ffffc0201f8c <gicv3_cpu_shutdown>:
ffffc0201f8c: f9482800 	ldr	x0, [x0, #4176]
ffffc0201f90: b40002c0 	cbz	x0, 0xffffc0201fe8 <gicv3_cpu_shutdown+0x5c>
ffffc0201f94: d2800001 	mov	x1, #0
ffffc0201f98: d51ccb01 	msr	ICH_HCR_EL2, x1
ffffc0201f9c: 900000c1 	adrp	x1, 0xffffc0219000 <gicv3_handle_dist_access+0xc>
ffffc0201fa0: 91404000 	add	x0, x0, #16, lsl #12    // =65536
ffffc0201fa4: f9400021 	ldr	x1, [x1]
ffffc0201fa8: 39416022 	ldrb	w2, [x1, #88]
ffffc0201fac: 52800021 	mov	w1, #1
ffffc0201fb0: 1ac22021 	lsl	w1, w1, w2
ffffc0201fb4: b9018001 	str	w1, [x0, #384]
ffffc0201fb8: d53ccbe1 	mrs	x1, ICH_VMCR_EL2
ffffc0201fbc: 374800a1 	tbnz	w1, #9, 0xffffc0201fd0 <gicv3_cpu_shutdown+0x44>
ffffc0201fc0: d538cc80 	mrs	x0, ICC_CTLR_EL1
ffffc0201fc4: 92407c00 	and	x0, x0, #0xffffffff
ffffc0201fc8: 927ef800 	and	x0, x0, #0xfffffffffffffffd
ffffc0201fcc: d518cc80 	msr	ICC_CTLR_EL1, x0
ffffc0201fd0: 37080081 	tbnz	w1, #1, 0xffffc0201fe0 <gicv3_cpu_shutdown+0x54>
ffffc0201fd4: d538cce0 	mrs	x0, ICC_IGRPEN1_EL1
ffffc0201fd8: 927f7800 	and	x0, x0, #0xfffffffe
ffffc0201fdc: d518cce0 	msr	ICC_IGRPEN1_EL1, x0
ffffc0201fe0: 52800000 	mov	w0, #0
ffffc0201fe4: 14000002 	b	0xffffc0201fec <gicv3_cpu_shutdown+0x60>
ffffc0201fe8: 12800240 	mov	w0, #-19
ffffc0201fec: d65f03c0 	ret

0000ffffc0201ff0 <gicv3_handle_dist_access>:
ffffc0201ff0: f9400002 	ldr	x2, [x0]
ffffc0201ff4: d29ffb81 	mov	x1, #65500
ffffc0201ff8: eb01005f 	cmp	x2, x1
ffffc0201ffc: 54000288 	b.hi	0xffffc020204c <gicv3_handle_dist_access+0x5c>
ffffc0202000: d29ffa01 	mov	x1, #65488
ffffc0202004: eb01005f 	cmp	x2, x1
ffffc0202008: 540000c2 	b.hs	0xffffc0202020 <gicv3_handle_dist_access+0x30>
ffffc020200c: f100105f 	cmp	x2, #4
ffffc0202010: 54000080 	b.eq	0xffffc0202020 <gicv3_handle_dist_access+0x30>
ffffc0202014: f100205f 	cmp	x2, #8
ffffc0202018: 54000040 	b.eq	0xffffc0202020 <gicv3_handle_dist_access+0x30>
ffffc020201c: b50002c2 	cbnz	x2, 0xffffc0202074 <gicv3_handle_dist_access+0x84>
ffffc0202020: b9400c01 	ldr	w1, [x0, #12]
ffffc0202024: 35000281 	cbnz	w1, 0xffffc0202074 <gicv3_handle_dist_access+0x84>
ffffc0202028: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020202c: aa0003e1 	mov	x1, x0
ffffc0202030: f0000080 	adrp	x0, 0xffffc0215000 <gicv3_init>
ffffc0202034: 910003fd 	mov	x29, sp
ffffc0202038: f9407000 	ldr	x0, [x0, #224]
ffffc020203c: 94001ca6 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0202040: 52800020 	mov	w0, #1
ffffc0202044: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0202048: d65f03c0 	ret
ffffc020204c: d29ffc01 	mov	x1, #65504
ffffc0202050: eb01005f 	cmp	x2, x1
ffffc0202054: 54000103 	b.lo	0xffffc0202074 <gicv3_handle_dist_access+0x84>
ffffc0202058: d29ffd81 	mov	x1, #65516
ffffc020205c: eb01005f 	cmp	x2, x1
ffffc0202060: 54fffe09 	b.ls	0xffffc0202020 <gicv3_handle_dist_access+0x30>
ffffc0202064: 929ffde1 	mov	x1, #-65520
ffffc0202068: 8b010042 	add	x2, x2, x1
ffffc020206c: f100305f 	cmp	x2, #12
ffffc0202070: 54fffd89 	b.ls	0xffffc0202020 <gicv3_handle_dist_access+0x30>
ffffc0202074: 52800020 	mov	w0, #1
ffffc0202078: d65f03c0 	ret

0000ffffc020207c <gicv3_init>:
ffffc020207c: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202080: f0000080 	adrp	x0, 0xffffc0215000 <gicv3_init+0x50>
ffffc0202084: 910003fd 	mov	x29, sp
ffffc0202088: f9407002 	ldr	x2, [x0, #224]
ffffc020208c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202090: 91403040 	add	x0, x2, #12, lsl #12    // =49152
ffffc0202094: f90013f5 	str	x21, [sp, #32]
ffffc0202098: b97fe801 	ldr	w1, [x0, #16360]
ffffc020209c: f0000080 	adrp	x0, 0xffffc0215000 <gicv3_init+0x6c>
ffffc02020a0: 91008013 	add	x19, x0, #32
ffffc02020a4: d3441c21 	ubfx	x1, x1, #4, #4
ffffc02020a8: b9000a61 	str	w1, [x19, #8]
ffffc02020ac: 51000c21 	sub	w1, w1, #3
ffffc02020b0: 7100043f 	cmp	w1, #1
ffffc02020b4: 540004c8 	b.hi	0xffffc020214c <gicv3_init+0xd0>
ffffc02020b8: aa0003f4 	mov	x20, x0
ffffc02020bc: b9400040 	ldr	w0, [x2]
ffffc02020c0: 362004a0 	tbz	w0, #4, 0xffffc0202154 <gicv3_init+0xd8>
ffffc02020c4: f00000a1 	adrp	x1, 0xffffc0219000 <gicv3_init+0xa4>
ffffc02020c8: aa0103f5 	mov	x21, x1
ffffc02020cc: f9400020 	ldr	x0, [x1]
ffffc02020d0: 91080000 	add	x0, x0, #512
ffffc02020d4: b853a000 	ldur	w0, [x0, #-198]
ffffc02020d8: 531d7000 	lsl	w0, w0, #3
ffffc02020dc: 51000400 	sub	w0, w0, #1
ffffc02020e0: b9000660 	str	w0, [x19, #4]
ffffc02020e4: b9400660 	ldr	w0, [x19, #4]
ffffc02020e8: 94001750 	bl	0xffffc0207e28 <cpu_id_valid>
ffffc02020ec: 34000280 	cbz	w0, 0xffffc020213c <gicv3_init+0xc0>
ffffc02020f0: b9400a60 	ldr	w0, [x19, #8]
ffffc02020f4: 91008294 	add	x20, x20, #32
ffffc02020f8: d2a00041 	mov	x1, #131072
ffffc02020fc: 7100101f 	cmp	w0, #4
ffffc0202100: d2a00080 	mov	x0, #262144
ffffc0202104: 9a801021 	csel	x1, x1, x0, ne
ffffc0202108: b9400680 	ldr	w0, [x20, #4]
ffffc020210c: 1b010401 	madd	w1, w0, w1, w1
ffffc0202110: f94002a0 	ldr	x0, [x21]
ffffc0202114: f847a000 	ldur	x0, [x0, #122]
ffffc0202118: 940014be 	bl	0xffffc0207410 <paging_map_device>
ffffc020211c: f100001f 	cmp	x0, #0
ffffc0202120: f9000a80 	str	x0, [x20, #16]
ffffc0202124: 12800160 	mov	w0, #-12
ffffc0202128: 1a9f0000 	csel	w0, w0, wzr, eq
ffffc020212c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202130: f94013f5 	ldr	x21, [sp, #32]
ffffc0202134: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202138: d65f03c0 	ret
ffffc020213c: b9400660 	ldr	w0, [x19, #4]
ffffc0202140: 51000400 	sub	w0, w0, #1
ffffc0202144: b9000660 	str	w0, [x19, #4]
ffffc0202148: 17ffffe7 	b	0xffffc02020e4 <gicv3_init+0x68>
ffffc020214c: 12800240 	mov	w0, #-19
ffffc0202150: 17fffff7 	b	0xffffc020212c <gicv3_init+0xb0>
ffffc0202154: 12800080 	mov	w0, #-5
ffffc0202158: 17fffff5 	b	0xffffc020212c <gicv3_init+0xb0>

0000ffffc020215c <gicv3_handle_sgir_write>:
ffffc020215c: f0000081 	adrp	x1, 0xffffc0215000 <gicv3_handle_sgir_write+0x4c>
ffffc0202160: b9402821 	ldr	w1, [x1, #40]
ffffc0202164: 7100083f 	cmp	w1, #2
ffffc0202168: 540002c9 	b.ls	0xffffc02021c0 <gicv3_handle_sgir_write+0x64>
ffffc020216c: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202170: d368a001 	ubfx	x1, x0, #40, #1
ffffc0202174: d360fc02 	lsr	x2, x0, #32
ffffc0202178: 910003fd 	mov	x29, sp
ffffc020217c: d3701c42 	ubfiz	x2, x2, #16, #8
ffffc0202180: 390063a1 	strb	w1, [x29, #24]
ffffc0202184: d370fc01 	lsr	x1, x0, #48
ffffc0202188: 790053a0 	strh	w0, [x29, #40]
ffffc020218c: d3601c21 	ubfiz	x1, x1, #32, #8
ffffc0202190: aa020021 	orr	x1, x1, x2
ffffc0202194: d350fc02 	lsr	x2, x0, #16
ffffc0202198: 53186c00 	ubfx	w0, w0, #24, #4
ffffc020219c: 790057a0 	strh	w0, [x29, #42]
ffffc02021a0: d3781c42 	ubfiz	x2, x2, #8, #8
ffffc02021a4: 910063a0 	add	x0, x29, #24
ffffc02021a8: aa020021 	orr	x1, x1, x2
ffffc02021ac: f90013a1 	str	x1, [x29, #32]
ffffc02021b0: 940001f8 	bl	0xffffc0202990 <gic_handle_sgir_write>
ffffc02021b4: 52800020 	mov	w0, #1
ffffc02021b8: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc02021bc: d65f03c0 	ret
ffffc02021c0: 52800000 	mov	w0, #0
ffffc02021c4: d65f03c0 	ret

0000ffffc02021c8 <jailhouse_cell_irqchips>:
ffffc02021c8: b9403401 	ldr	w1, [x0, #52]
ffffc02021cc: 52800183 	mov	w3, #12
ffffc02021d0: d37be822 	lsl	x2, x1, #5
ffffc02021d4: b9403801 	ldr	w1, [x0, #56]
ffffc02021d8: 9ba30821 	umaddl	x1, w1, w3, x2
ffffc02021dc: b9403002 	ldr	w2, [x0, #48]
ffffc02021e0: 91020042 	add	x2, x2, #128
ffffc02021e4: 8b020021 	add	x1, x1, x2
ffffc02021e8: 8b010000 	add	x0, x0, x1
ffffc02021ec: d65f03c0 	ret

0000ffffc02021f0 <spin_lock>:
ffffc02021f0: f9800011 	prfm	pstl1strm, [x0]
ffffc02021f4: 885ffc01 	ldaxr	w1, [x0]
ffffc02021f8: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc02021fc: 88037c02 	stxr	w3, w2, [x0]
ffffc0202200: 35ffffa3 	cbnz	w3, 0xffffc02021f4 <spin_lock+0x4>
ffffc0202204: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc0202208: 340000c2 	cbz	w2, 0xffffc0202220 <spin_lock+0x30>
ffffc020220c: d50320bf 	sevl
ffffc0202210: d503205f 	wfe
ffffc0202214: 485ffc03 	ldaxrh	w3, [x0]
ffffc0202218: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc020221c: 35ffffa2 	cbnz	w2, 0xffffc0202210 <spin_lock+0x20>
ffffc0202220: d65f03c0 	ret

0000ffffc0202224 <irqchip_mmio_count_regions>:
ffffc0202224: f00000a0 	adrp	x0, 0xffffc0219000 <irqchip_cell_init+0x2c>
ffffc0202228: f9400000 	ldr	x0, [x0]
ffffc020222c: 39416400 	ldrb	w0, [x0, #89]
ffffc0202230: 7100081f 	cmp	w0, #2
ffffc0202234: 540000a9 	b.ls	0xffffc0202248 <irqchip_mmio_count_regions+0x24>
ffffc0202238: d0ffffe0 	adrp	x0, 0xffffc0200000 <irqchip_mmio_count_regions+0xc>
ffffc020223c: b9403400 	ldr	w0, [x0, #52]
ffffc0202240: 11000400 	add	w0, w0, #1
ffffc0202244: d65f03c0 	ret
ffffc0202248: 52800020 	mov	w0, #1
ffffc020224c: 17fffffe 	b	0xffffc0202244 <irqchip_mmio_count_regions+0x20>

0000ffffc0202250 <irqchip_shutdown>:
ffffc0202250: d65f03c0 	ret

0000ffffc0202254 <irqchip_cell_init>:
ffffc0202254: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202258: 910003fd 	mov	x29, sp
ffffc020225c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202260: f00000b4 	adrp	x20, 0xffffc0219000 <irqchip_cell_init+0x68>
ffffc0202264: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0202268: aa0003f3 	mov	x19, x0
ffffc020226c: f9400285 	ldr	x5, [x20]
ffffc0202270: f9485000 	ldr	x0, [x0, #4256]
ffffc0202274: 394160a6 	ldrb	w6, [x5, #88]
ffffc0202278: 97ffffd4 	bl	0xffffc02021c8 <jailhouse_cell_irqchips>
ffffc020227c: 52800004 	mov	w4, #0
ffffc0202280: f9485261 	ldr	x1, [x19, #4256]
ffffc0202284: b9403c21 	ldr	w1, [x1, #60]
ffffc0202288: 6b04003f 	cmp	w1, w4
ffffc020228c: 540004c8 	b.hi	0xffffc0202324 <irqchip_cell_init+0xd0>
ffffc0202290: f0000096 	adrp	x22, 0xffffc0215000 <irqchip_cell_init+0x88>
ffffc0202294: 9100e2d6 	add	x22, x22, #56
ffffc0202298: 52800020 	mov	w0, #1
ffffc020229c: 1ac62000 	lsl	w0, w0, w6
ffffc02022a0: 32000400 	orr	w0, w0, #0x3
ffffc02022a4: f94012c1 	ldr	x1, [x22, #32]
ffffc02022a8: 2a2003e0 	mvn	w0, w0
ffffc02022ac: b9101a60 	str	w0, [x19, #4120]
ffffc02022b0: aa1303e0 	mov	x0, x19
ffffc02022b4: d63f0020 	blr	x1
ffffc02022b8: 2a0003f5 	mov	w21, w0
ffffc02022bc: 350002a0 	cbnz	w0, 0xffffc0202310 <irqchip_cell_init+0xbc>
ffffc02022c0: f9400280 	ldr	x0, [x20]
ffffc02022c4: d2800004 	mov	x4, #0
ffffc02022c8: f94052c2 	ldr	x2, [x22, #160]
ffffc02022cc: 90000003 	adrp	x3, 0xffffc0202000 <irqchip_cell_init+0x78>
ffffc02022d0: 91175063 	add	x3, x3, #1492
ffffc02022d4: f845a001 	ldur	x1, [x0, #90]
ffffc02022d8: aa1303e0 	mov	x0, x19
ffffc02022dc: 94001b61 	bl	0xffffc0209060 <mmio_region_register>
ffffc02022e0: 900000c4 	adrp	x4, 0xffffc021a000 <irqchip_cell_init+0xec>
ffffc02022e4: 91000084 	add	x4, x4, #0
ffffc02022e8: eb04027f 	cmp	x19, x4
ffffc02022ec: 54000120 	b.eq	0xffffc0202310 <irqchip_cell_init+0xbc>
ffffc02022f0: f9485266 	ldr	x6, [x19, #4256]
ffffc02022f4: aa0603e0 	mov	x0, x6
ffffc02022f8: 97ffffb4 	bl	0xffffc02021c8 <jailhouse_cell_irqchips>
ffffc02022fc: f9400287 	ldr	x7, [x20]
ffffc0202300: 52800003 	mov	w3, #0
ffffc0202304: b9403cc1 	ldr	w1, [x6, #60]
ffffc0202308: 6b03003f 	cmp	w1, w3
ffffc020230c: 54000328 	b.hi	0xffffc0202370 <irqchip_cell_init+0x11c>
ffffc0202310: 2a1503e0 	mov	w0, w21
ffffc0202314: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202318: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020231c: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202320: d65f03c0 	ret
ffffc0202324: f9400002 	ldr	x2, [x0]
ffffc0202328: f845a0a1 	ldur	x1, [x5, #90]
ffffc020232c: eb01005f 	cmp	x2, x1
ffffc0202330: 540001a1 	b.ne	0xffffc0202364 <irqchip_cell_init+0x110>
ffffc0202334: b9400c01 	ldr	w1, [x0, #12]
ffffc0202338: f240103f 	tst	x1, #0x1f
ffffc020233c: 54000421 	b.ne	0xffffc02023c0 <irqchip_cell_init+0x16c>
ffffc0202340: 2a0103e2 	mov	w2, w1
ffffc0202344: 91020042 	add	x2, x2, #128
ffffc0202348: f110005f 	cmp	x2, #1024
ffffc020234c: 540003a8 	b.hi	0xffffc02023c0 <irqchip_cell_init+0x16c>
ffffc0202350: 53057c21 	lsr	w1, w1, #5
ffffc0202354: 91101821 	add	x1, x1, #1030
ffffc0202358: a9410c02 	ldp	x2, x3, [x0, #16]
ffffc020235c: 8b010a61 	add	x1, x19, x1, lsl #2
ffffc0202360: a9000c22 	stp	x2, x3, [x1]
ffffc0202364: 91008000 	add	x0, x0, #32
ffffc0202368: 11000484 	add	w4, w4, #1
ffffc020236c: 17ffffc5 	b	0xffffc0202280 <irqchip_cell_init+0x2c>
ffffc0202370: f9400002 	ldr	x2, [x0]
ffffc0202374: f845a0e1 	ldur	x1, [x7, #90]
ffffc0202378: eb01005f 	cmp	x2, x1
ffffc020237c: 540001c1 	b.ne	0xffffc02023b4 <irqchip_cell_init+0x160>
ffffc0202380: d2800002 	mov	x2, #0
ffffc0202384: b9400c01 	ldr	w1, [x0, #12]
ffffc0202388: 8b020805 	add	x5, x0, x2, lsl #2
ffffc020238c: 0b411441 	add	w1, w2, w1, lsr #5
ffffc0202390: b94010a5 	ldr	w5, [x5, #16]
ffffc0202394: 91101021 	add	x1, x1, #1028
ffffc0202398: 91000442 	add	x2, x2, #1
ffffc020239c: f100105f 	cmp	x2, #4
ffffc02023a0: 8b010881 	add	x1, x4, x1, lsl #2
ffffc02023a4: b9400828 	ldr	w8, [x1, #8]
ffffc02023a8: 0a250105 	bic	w5, w8, w5
ffffc02023ac: b9000825 	str	w5, [x1, #8]
ffffc02023b0: 54fffea1 	b.ne	0xffffc0202384 <irqchip_cell_init+0x130>
ffffc02023b4: 91008000 	add	x0, x0, #32
ffffc02023b8: 11000463 	add	w3, w3, #1
ffffc02023bc: 17ffffd2 	b	0xffffc0202304 <irqchip_cell_init+0xb0>
ffffc02023c0: 128002b5 	mov	w21, #-22
ffffc02023c4: 17ffffd3 	b	0xffffc0202310 <irqchip_cell_init+0xbc>

0000ffffc02023c8 <irqchip_init>:
ffffc02023c8: 900000c0 	adrp	x0, 0xffffc021a000 <irqchip_handle_irq+0x54>
ffffc02023cc: 91000000 	add	x0, x0, #0
ffffc02023d0: 17ffffa1 	b	0xffffc0202254 <irqchip_cell_init>

0000ffffc02023d4 <irqchip_handle_irq>:
ffffc02023d4: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc02023d8: 910003fd 	mov	x29, sp
ffffc02023dc: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02023e0: f0000093 	adrp	x19, 0xffffc0215000 <irqchip_handle_irq+0x58>
ffffc02023e4: 9100e273 	add	x19, x19, #56
ffffc02023e8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02023ec: aa1303f6 	mov	x22, x19
ffffc02023f0: 52800034 	mov	w20, #1
ffffc02023f4: f9402260 	ldr	x0, [x19, #64]
ffffc02023f8: d63f0000 	blr	x0
ffffc02023fc: 2a0003f5 	mov	w21, w0
ffffc0202400: 710ffc1f 	cmp	w0, #1023
ffffc0202404: 540001c0 	b.eq	0xffffc020243c <irqchip_handle_irq+0x68>
ffffc0202408: 2a1403e1 	mov	w1, w20
ffffc020240c: 71003c1f 	cmp	w0, #15
ffffc0202410: 54000128 	b.hi	0xffffc0202434 <irqchip_handle_irq+0x60>
ffffc0202414: 97fff93d 	bl	0xffffc0200908 <arch_handle_sgi>
ffffc0202418: 52800020 	mov	w0, #1
ffffc020241c: f94026c2 	ldr	x2, [x22, #72]
ffffc0202420: 2a0003e1 	mov	w1, w0
ffffc0202424: 52800014 	mov	w20, #0
ffffc0202428: 2a1503e0 	mov	w0, w21
ffffc020242c: d63f0040 	blr	x2
ffffc0202430: 17fffff1 	b	0xffffc02023f4 <irqchip_handle_irq+0x20>
ffffc0202434: 97fff97b 	bl	0xffffc0200a20 <arch_handle_phys_irq>
ffffc0202438: 17fffff9 	b	0xffffc020241c <irqchip_handle_irq+0x48>
ffffc020243c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202440: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202444: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202448: d65f03c0 	ret

0000ffffc020244c <irqchip_irq_in_cell>:
ffffc020244c: 710ffc3f 	cmp	w1, #1023
ffffc0202450: 54000148 	b.hi	0xffffc0202478 <irqchip_irq_in_cell+0x2c>
ffffc0202454: 53057c22 	lsr	w2, w1, #5
ffffc0202458: 12001021 	and	w1, w1, #0x1f
ffffc020245c: 8b020800 	add	x0, x0, x2, lsl #2
ffffc0202460: 52800022 	mov	w2, #1
ffffc0202464: 1ac12041 	lsl	w1, w2, w1
ffffc0202468: b9501800 	ldr	w0, [x0, #4120]
ffffc020246c: 6a00003f 	tst	w1, w0
ffffc0202470: 1a9f07e0 	cset	w0, ne
ffffc0202474: d65f03c0 	ret
ffffc0202478: 52800000 	mov	w0, #0
ffffc020247c: 17fffffe 	b	0xffffc0202474 <irqchip_irq_in_cell+0x28>

0000ffffc0202480 <restrict_bitmask_access>:
ffffc0202480: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202484: 2a0203e8 	mov	w8, w2
ffffc0202488: 910003fd 	mov	x29, sp
ffffc020248c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202490: aa0003f4 	mov	x20, x0
ffffc0202494: d2840000 	mov	x0, #8192
ffffc0202498: f90013f5 	str	x21, [sp, #32]
ffffc020249c: f2a00020 	movk	x0, #1, lsl #16
ffffc02024a0: fd0017e8 	str	d8, [sp, #40]
ffffc02024a4: f2dfe000 	movk	x0, #65280, lsl #32
ffffc02024a8: f948040a 	ldr	x10, [x0, #4104]
ffffc02024ac: 2a0203e0 	mov	w0, w2
ffffc02024b0: dac00000 	rbit	x0, x0
ffffc02024b4: dac01000 	clz	x0, x0
ffffc02024b8: 52800404 	mov	w4, #32
ffffc02024bc: 52800007 	mov	w7, #0
ffffc02024c0: 1ac02880 	asr	w0, w4, w0
ffffc02024c4: 52800024 	mov	w4, #1
ffffc02024c8: 1ac22084 	lsl	w4, w4, w2
ffffc02024cc: 51000484 	sub	w4, w4, #1
ffffc02024d0: 1b007c26 	mul	w6, w1, w0
ffffc02024d4: 93407c09 	sxtw	x9, w0
ffffc02024d8: 93407c84 	sxtw	x4, w4
ffffc02024dc: 9e6703e8 	fmov	d8, xzr
ffffc02024e0: 52800005 	mov	w5, #0
ffffc02024e4: eb25413f 	cmp	x9, w5, uxtw
ffffc02024e8: 54000248 	b.hi	0xffffc0202530 <restrict_bitmask_access+0xb0>
ffffc02024ec: b9400e80 	ldr	w0, [x20, #12]
ffffc02024f0: f0000093 	adrp	x19, 0xffffc0215000 <restrict_bitmask_access+0xbc>
ffffc02024f4: 35000320 	cbnz	w0, 0xffffc0202558 <restrict_bitmask_access+0xd8>
ffffc02024f8: 9100e273 	add	x19, x19, #56
ffffc02024fc: aa1403e1 	mov	x1, x20
ffffc0202500: f9405660 	ldr	x0, [x19, #168]
ffffc0202504: 94001b74 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0202508: 9e660101 	fmov	x1, d8
ffffc020250c: f9400a80 	ldr	x0, [x20, #16]
ffffc0202510: 8a010000 	and	x0, x0, x1
ffffc0202514: f9000a80 	str	x0, [x20, #16]
ffffc0202518: 52800020 	mov	w0, #1
ffffc020251c: f94013f5 	ldr	x21, [sp, #32]
ffffc0202520: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202524: fd4017e8 	ldr	d8, [sp, #40]
ffffc0202528: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc020252c: d65f03c0 	ret
ffffc0202530: 0b0600a1 	add	w1, w5, w6
ffffc0202534: aa0a03e0 	mov	x0, x10
ffffc0202538: 97ffffc5 	bl	0xffffc020244c <irqchip_irq_in_cell>
ffffc020253c: 34000080 	cbz	w0, 0xffffc020254c <restrict_bitmask_access+0xcc>
ffffc0202540: 9ac72080 	lsl	x0, x4, x7
ffffc0202544: 9e670000 	fmov	d0, x0
ffffc0202548: 0ea01d08 	orr	v8.8b, v8.8b, v0.8b
ffffc020254c: 110004a5 	add	w5, w5, #1
ffffc0202550: 0b0800e7 	add	w7, w7, w8
ffffc0202554: 17ffffe4 	b	0xffffc02024e4 <restrict_bitmask_access+0x64>
ffffc0202558: f9400a95 	ldr	x21, [x20, #16]
ffffc020255c: 350002c3 	cbnz	w3, 0xffffc02025b4 <restrict_bitmask_access+0x134>
ffffc0202560: 9100e273 	add	x19, x19, #56
ffffc0202564: 9102c260 	add	x0, x19, #176
ffffc0202568: 97ffff22 	bl	0xffffc02021f0 <spin_lock>
ffffc020256c: b9000e9f 	str	wzr, [x20, #12]
ffffc0202570: aa1403e1 	mov	x1, x20
ffffc0202574: f9405660 	ldr	x0, [x19, #168]
ffffc0202578: 94001b57 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc020257c: fd400a81 	ldr	d1, [x20, #16]
ffffc0202580: 9e6702a0 	fmov	d0, x21
ffffc0202584: 52800020 	mov	w0, #1
ffffc0202588: b9000e80 	str	w0, [x20, #12]
ffffc020258c: f9405660 	ldr	x0, [x19, #168]
ffffc0202590: aa1403e1 	mov	x1, x20
ffffc0202594: 2e611c08 	bsl	v8.8b, v0.8b, v1.8b
ffffc0202598: fd000a88 	str	d8, [x20, #16]
ffffc020259c: 94001b4e 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc02025a0: 79416260 	ldrh	w0, [x19, #176]
ffffc02025a4: 9102c261 	add	x1, x19, #176
ffffc02025a8: 11000400 	add	w0, w0, #1
ffffc02025ac: 489ffc20 	stlrh	w0, [x1]
ffffc02025b0: 17ffffda 	b	0xffffc0202518 <restrict_bitmask_access+0x98>
ffffc02025b4: 9e660100 	fmov	x0, d8
ffffc02025b8: 9100e273 	add	x19, x19, #56
ffffc02025bc: aa1403e1 	mov	x1, x20
ffffc02025c0: 8a150000 	and	x0, x0, x21
ffffc02025c4: f9000a80 	str	x0, [x20, #16]
ffffc02025c8: f9405660 	ldr	x0, [x19, #168]
ffffc02025cc: 94001b42 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc02025d0: 17ffffd2 	b	0xffffc0202518 <restrict_bitmask_access+0x98>

0000ffffc02025d4 <gic_handle_dist_access>:
ffffc02025d4: aa0103e0 	mov	x0, x1
ffffc02025d8: f9400021 	ldr	x1, [x1]
ffffc02025dc: f10df03f 	cmp	x1, #892
ffffc02025e0: 540002e8 	b.hi	0xffffc020263c <gic_handle_dist_access+0x68>
ffffc02025e4: f10c003f 	cmp	x1, #768
ffffc02025e8: 54000642 	b.hs	0xffffc02026b0 <gic_handle_dist_access+0xdc>
ffffc02025ec: f107f03f 	cmp	x1, #508
ffffc02025f0: 540001a8 	b.hi	0xffffc0202624 <gic_handle_dist_access+0x50>
ffffc02025f4: f106003f 	cmp	x1, #384
ffffc02025f8: 540005c2 	b.hs	0xffffc02026b0 <gic_handle_dist_access+0xdc>
ffffc02025fc: f102003f 	cmp	x1, #128
ffffc0202600: 540000c3 	b.lo	0xffffc0202618 <gic_handle_dist_access+0x44>
ffffc0202604: f103f03f 	cmp	x1, #252
ffffc0202608: 540005c9 	b.ls	0xffffc02026c0 <gic_handle_dist_access+0xec>
ffffc020260c: d1040022 	sub	x2, x1, #256
ffffc0202610: f101f05f 	cmp	x2, #124
ffffc0202614: 540004e9 	b.ls	0xffffc02026b0 <gic_handle_dist_access+0xdc>
ffffc0202618: f0000081 	adrp	x1, 0xffffc0215000 <gic_handle_dist_access+0x90>
ffffc020261c: f9406821 	ldr	x1, [x1, #208]
ffffc0202620: d61f0020 	br	x1
ffffc0202624: f108003f 	cmp	x1, #512
ffffc0202628: 54ffff83 	b.lo	0xffffc0202618 <gic_handle_dist_access+0x44>
ffffc020262c: f109f03f 	cmp	x1, #636
ffffc0202630: 54000409 	b.ls	0xffffc02026b0 <gic_handle_dist_access+0xdc>
ffffc0202634: d10a0022 	sub	x2, x1, #640
ffffc0202638: 17fffff6 	b	0xffffc0202610 <gic_handle_dist_access+0x3c>
ffffc020263c: f12ffc3f 	cmp	x1, #3071
ffffc0202640: 540001c8 	b.hi	0xffffc0202678 <gic_handle_dist_access+0xa4>
ffffc0202644: f120003f 	cmp	x1, #2048
ffffc0202648: 540002c2 	b.hs	0xffffc02026a0 <gic_handle_dist_access+0xcc>
ffffc020264c: f10e003f 	cmp	x1, #896
ffffc0202650: 54fffe43 	b.lo	0xffffc0202618 <gic_handle_dist_access+0x44>
ffffc0202654: f10ff03f 	cmp	x1, #1020
ffffc0202658: 540002c9 	b.ls	0xffffc02026b0 <gic_handle_dist_access+0xdc>
ffffc020265c: d1100022 	sub	x2, x1, #1024
ffffc0202660: f10fe05f 	cmp	x2, #1016
ffffc0202664: 54fffda8 	b.hi	0xffffc0202618 <gic_handle_dist_access+0x44>
ffffc0202668: 52800003 	mov	w3, #0
ffffc020266c: 52800102 	mov	w2, #8
ffffc0202670: 53022421 	ubfx	w1, w1, #2, #8
ffffc0202674: 14000012 	b	0xffffc02026bc <gic_handle_dist_access+0xe8>
ffffc0202678: f133f03f 	cmp	x1, #3324
ffffc020267c: 54000269 	b.ls	0xffffc02026c8 <gic_handle_dist_access+0xf4>
ffffc0202680: d1401821 	sub	x1, x1, #6, lsl #12     // =24576
ffffc0202684: d283ff02 	mov	x2, #8184
ffffc0202688: eb02003f 	cmp	x1, x2
ffffc020268c: 54fffc68 	b.hi	0xffffc0202618 <gic_handle_dist_access+0x44>
ffffc0202690: f0000082 	adrp	x2, 0xffffc0215000 <irqchip_has_pending_irqs+0x4>
ffffc0202694: d3438821 	ubfx	x1, x1, #3, #32
ffffc0202698: f9406042 	ldr	x2, [x2, #192]
ffffc020269c: d61f0040 	br	x2
ffffc02026a0: f0000082 	adrp	x2, 0xffffc0215000 <irqchip_inject_pending+0x8>
ffffc02026a4: 51200021 	sub	w1, w1, #2048
ffffc02026a8: f9406442 	ldr	x2, [x2, #200]
ffffc02026ac: 17fffffc 	b	0xffffc020269c <gic_handle_dist_access+0xc8>
ffffc02026b0: 52800023 	mov	w3, #1
ffffc02026b4: 53021821 	ubfx	w1, w1, #2, #5
ffffc02026b8: 52800022 	mov	w2, #1
ffffc02026bc: 17ffff71 	b	0xffffc0202480 <restrict_bitmask_access>
ffffc02026c0: 52800003 	mov	w3, #0
ffffc02026c4: 17fffffc 	b	0xffffc02026b4 <gic_handle_dist_access+0xe0>
ffffc02026c8: 52800003 	mov	w3, #0
ffffc02026cc: 52800042 	mov	w2, #2
ffffc02026d0: 53021c21 	ubfx	w1, w1, #2, #6
ffffc02026d4: 17fffffa 	b	0xffffc02026bc <gic_handle_dist_access+0xe8>

0000ffffc02026d8 <irqchip_has_pending_irqs>:
ffffc02026d8: f0000080 	adrp	x0, 0xffffc0215000 <irqchip_inject_pending+0x40>
ffffc02026dc: f9404c00 	ldr	x0, [x0, #152]
ffffc02026e0: d61f0000 	br	x0

0000ffffc02026e4 <irqchip_inject_pending>:
ffffc02026e4: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc02026e8: 910003fd 	mov	x29, sp
ffffc02026ec: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02026f0: f0000093 	adrp	x19, 0xffffc0215000 <irqchip_inject_pending+0x58>
ffffc02026f4: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02026f8: d2a00036 	mov	x22, #65536
ffffc02026fc: 9100e275 	add	x21, x19, #56
ffffc0202700: f2dfe016 	movk	x22, #65280, lsl #32
ffffc0202704: d2860c14 	mov	x20, #12384
ffffc0202708: f2a00034 	movk	x20, #1, lsl #16
ffffc020270c: f2dfe014 	movk	x20, #65280, lsl #32
ffffc0202710: b9440a81 	ldr	w1, [x20, #1032]
ffffc0202714: b9440680 	ldr	w0, [x20, #1028]
ffffc0202718: 6b01001f 	cmp	w0, w1
ffffc020271c: 540000a1 	b.ne	0xffffc0202730 <irqchip_inject_pending+0x4c>
ffffc0202720: 9100e273 	add	x19, x19, #56
ffffc0202724: 52800000 	mov	w0, #0
ffffc0202728: f9402e61 	ldr	x1, [x19, #88]
ffffc020272c: 1400000b 	b	0xffffc0202758 <irqchip_inject_pending+0x74>
ffffc0202730: 8b2046c0 	add	x0, x22, w0, uxtw #1
ffffc0202734: f9402aa2 	ldr	x2, [x21, #80]
ffffc0202738: 91400800 	add	x0, x0, #2, lsl #12     // =8192
ffffc020273c: 7964c801 	ldrh	w1, [x0, #4708]
ffffc0202740: 7960c800 	ldrh	w0, [x0, #4196]
ffffc0202744: d63f0040 	blr	x2
ffffc0202748: 3100401f 	cmn	w0, #16
ffffc020274c: 540000e1 	b.ne	0xffffc0202768 <irqchip_inject_pending+0x84>
ffffc0202750: f9402ea1 	ldr	x1, [x21, #88]
ffffc0202754: 52800020 	mov	w0, #1
ffffc0202758: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020275c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202760: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202764: d61f0020 	br	x1
ffffc0202768: b9440680 	ldr	w0, [x20, #1028]
ffffc020276c: 11000400 	add	w0, w0, #1
ffffc0202770: 12001c00 	and	w0, w0, #0xff
ffffc0202774: b9040680 	str	w0, [x20, #1028]
ffffc0202778: 17ffffe3 	b	0xffffc0202704 <irqchip_inject_pending+0x20>

0000ffffc020277c <irqchip_send_sgi>:
ffffc020277c: f0000081 	adrp	x1, 0xffffc0215000 <irqchip_cpu_init+0x40>
ffffc0202780: f9403821 	ldr	x1, [x1, #112]
ffffc0202784: d61f0020 	br	x1

0000ffffc0202788 <irqchip_cpu_init>:
ffffc0202788: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc020278c: 910003fd 	mov	x29, sp
ffffc0202790: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202794: f0000093 	adrp	x19, 0xffffc0215000 <irqchip_cpu_init+0x58>
ffffc0202798: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020279c: aa0003f6 	mov	x22, x0
ffffc02027a0: 9100e260 	add	x0, x19, #56
ffffc02027a4: b940b401 	ldr	w1, [x0, #180]
ffffc02027a8: 35000381 	cbnz	w1, 0xffffc0202818 <irqchip_cpu_init+0x90>
ffffc02027ac: f00000a1 	adrp	x1, 0xffffc0219000 <irqchip_cpu_init+0x80>
ffffc02027b0: f9400035 	ldr	x21, [x1]
ffffc02027b4: 394166a1 	ldrb	w1, [x21, #89]
ffffc02027b8: 7100083f 	cmp	w1, #2
ffffc02027bc: 54000100 	b.eq	0xffffc02027dc <irqchip_cpu_init+0x54>
ffffc02027c0: 71000c3f 	cmp	w1, #3
ffffc02027c4: 54000380 	b.eq	0xffffc0202834 <irqchip_cpu_init+0xac>
ffffc02027c8: 128002a0 	mov	w0, #-22
ffffc02027cc: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02027d0: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02027d4: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc02027d8: d65f03c0 	ret
ffffc02027dc: b0000041 	adrp	x1, 0xffffc020b000 <irqchip_cpu_init+0x78>
ffffc02027e0: 91048021 	add	x1, x1, #288
ffffc02027e4: d2801502 	mov	x2, #168
ffffc02027e8: 9100e274 	add	x20, x19, #56
ffffc02027ec: 940019a5 	bl	0xffffc0208e80 <memcpy>
ffffc02027f0: f845a2a0 	ldur	x0, [x21, #90]
ffffc02027f4: f9405281 	ldr	x1, [x20, #160]
ffffc02027f8: 94001306 	bl	0xffffc0207410 <paging_map_device>
ffffc02027fc: f9005680 	str	x0, [x20, #168]
ffffc0202800: b4000220 	cbz	x0, 0xffffc0202844 <irqchip_cpu_init+0xbc>
ffffc0202804: f9401e60 	ldr	x0, [x19, #56]
ffffc0202808: d63f0000 	blr	x0
ffffc020280c: 35fffe00 	cbnz	w0, 0xffffc02027cc <irqchip_cpu_init+0x44>
ffffc0202810: 52800020 	mov	w0, #1
ffffc0202814: b900b680 	str	w0, [x20, #180]
ffffc0202818: 9100e273 	add	x19, x19, #56
ffffc020281c: aa1603e0 	mov	x0, x22
ffffc0202820: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202824: f9400661 	ldr	x1, [x19, #8]
ffffc0202828: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020282c: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202830: d61f0020 	br	x1
ffffc0202834: b0000041 	adrp	x1, 0xffffc020b000 <irqchip_get_cluster_target>
ffffc0202838: d2801502 	mov	x2, #168
ffffc020283c: 9107a021 	add	x1, x1, #488
ffffc0202840: 17ffffea 	b	0xffffc02027e8 <irqchip_cpu_init+0x60>
ffffc0202844: 12800160 	mov	w0, #-12
ffffc0202848: 17ffffe1 	b	0xffffc02027cc <irqchip_cpu_init+0x44>

0000ffffc020284c <irqchip_get_cpu_target>:
ffffc020284c: f0000081 	adrp	x1, 0xffffc0215000 <irqchip_set_pending+0x34>
ffffc0202850: f9405821 	ldr	x1, [x1, #176]
ffffc0202854: d61f0020 	br	x1

0000ffffc0202858 <irqchip_get_cluster_target>:
ffffc0202858: f0000081 	adrp	x1, 0xffffc0215000 <irqchip_set_pending+0x40>
ffffc020285c: f9405c21 	ldr	x1, [x1, #184]
ffffc0202860: d61f0020 	br	x1

0000ffffc0202864 <irqchip_set_pending>:
ffffc0202864: a9bb7bfd 	stp	x29, x30, [sp, #-80]!
ffffc0202868: 910003fd 	mov	x29, sp
ffffc020286c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202870: 12003c34 	and	w20, w1, #0xffff
ffffc0202874: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0202878: b50001c0 	cbnz	x0, 0xffffc02028b0 <irqchip_set_pending+0x4c>
ffffc020287c: 53057e80 	lsr	w0, w20, #5
ffffc0202880: f0000081 	adrp	x1, 0xffffc0215000 <irqchip_set_pending+0x68>
ffffc0202884: 12001294 	and	w20, w20, #0x1f
ffffc0202888: d37e2800 	ubfiz	x0, x0, #2, #11
ffffc020288c: f9407022 	ldr	x2, [x1, #224]
ffffc0202890: 91080000 	add	x0, x0, #512
ffffc0202894: 52800021 	mov	w1, #1
ffffc0202898: 1ad42034 	lsl	w20, w1, w20
ffffc020289c: b8206854 	str	w20, [x2, x0]
ffffc02028a0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02028a4: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02028a8: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc02028ac: d65f03c0 	ret
ffffc02028b0: d2840015 	mov	x21, #8192
ffffc02028b4: aa0003f3 	mov	x19, x0
ffffc02028b8: f2a00035 	movk	x21, #1, lsl #16
ffffc02028bc: f2dfe015 	movk	x21, #65280, lsl #32
ffffc02028c0: eb15001f 	cmp	x0, x21
ffffc02028c4: 796002b6 	ldrh	w22, [x21, #4096]
ffffc02028c8: 540003e0 	b.eq	0xffffc0202944 <irqchip_set_pending+0xe0>
ffffc02028cc: d2820c00 	mov	x0, #4192
ffffc02028d0: 8b000264 	add	x4, x19, x0
ffffc02028d4: aa0403e0 	mov	x0, x4
ffffc02028d8: 97fffe46 	bl	0xffffc02021f0 <spin_lock>
ffffc02028dc: d2820c01 	mov	x1, #4192
ffffc02028e0: 8b010260 	add	x0, x19, x1
ffffc02028e4: b9440801 	ldr	w1, [x0, #1032]
ffffc02028e8: b9440402 	ldr	w2, [x0, #1028]
ffffc02028ec: 11000421 	add	w1, w1, #1
ffffc02028f0: 12001c21 	and	w1, w1, #0xff
ffffc02028f4: 6b01005f 	cmp	w2, w1
ffffc02028f8: 54000120 	b.eq	0xffffc020291c <irqchip_set_pending+0xb8>
ffffc02028fc: b9440802 	ldr	w2, [x0, #1032]
ffffc0202900: 8b224402 	add	x2, x0, w2, uxtw #1
ffffc0202904: 79000854 	strh	w20, [x2, #4]
ffffc0202908: b9440802 	ldr	w2, [x0, #1032]
ffffc020290c: 8b224402 	add	x2, x0, w2, uxtw #1
ffffc0202910: 79040856 	strh	w22, [x2, #516]
ffffc0202914: b9040801 	str	w1, [x0, #1032]
ffffc0202918: d5033bbf 	dmb	ish
ffffc020291c: 7960c260 	ldrh	w0, [x19, #4192]
ffffc0202920: 11000400 	add	w0, w0, #1
ffffc0202924: 489ffc80 	stlrh	w0, [x4]
ffffc0202928: eb15027f 	cmp	x19, x21
ffffc020292c: 540001c1 	b.ne	0xffffc0202964 <irqchip_set_pending+0x100>
ffffc0202930: f0000080 	adrp	x0, 0xffffc0215000 <irqchip_set_pending+0x118>
ffffc0202934: f9404801 	ldr	x1, [x0, #144]
ffffc0202938: 52800020 	mov	w0, #1
ffffc020293c: d63f0020 	blr	x1
ffffc0202940: 17ffffd8 	b	0xffffc02028a0 <irqchip_set_pending+0x3c>
ffffc0202944: f0000080 	adrp	x0, 0xffffc0215000 <gic_handle_sgir_write>
ffffc0202948: 2a1603e1 	mov	w1, w22
ffffc020294c: f9404402 	ldr	x2, [x0, #136]
ffffc0202950: 2a1403e0 	mov	w0, w20
ffffc0202954: d63f0040 	blr	x2
ffffc0202958: 3100401f 	cmn	w0, #16
ffffc020295c: 54fffa21 	b.ne	0xffffc02028a0 <irqchip_set_pending+0x3c>
ffffc0202960: 17ffffdb 	b	0xffffc02028cc <irqchip_set_pending+0x68>
ffffc0202964: b9500260 	ldr	w0, [x19, #4096]
ffffc0202968: 97ffffb9 	bl	0xffffc020284c <irqchip_get_cpu_target>
ffffc020296c: 790093a0 	strh	w0, [x29, #72]
ffffc0202970: b9500260 	ldr	w0, [x19, #4096]
ffffc0202974: 97ffffb9 	bl	0xffffc0202858 <irqchip_get_cluster_target>
ffffc0202978: 3900e3bf 	strb	wzr, [x29, #56]
ffffc020297c: f90023a0 	str	x0, [x29, #64]
ffffc0202980: 9100e3a0 	add	x0, x29, #56
ffffc0202984: 790097bf 	strh	wzr, [x29, #74]
ffffc0202988: 97ffff7d 	bl	0xffffc020277c <irqchip_send_sgi>
ffffc020298c: 17ffffc5 	b	0xffffc02028a0 <irqchip_set_pending+0x3c>

0000ffffc0202990 <gic_handle_sgir_write>:
ffffc0202990: 39400001 	ldrb	w1, [x0]
ffffc0202994: 7100083f 	cmp	w1, #2
ffffc0202998: 54000581 	b.ne	0xffffc0202a48 <gic_handle_sgir_write+0xb8>
ffffc020299c: 79402401 	ldrh	w1, [x0, #18]
ffffc02029a0: d2840000 	mov	x0, #8192
ffffc02029a4: f2a00020 	movk	x0, #1, lsl #16
ffffc02029a8: f2dfe000 	movk	x0, #65280, lsl #32
ffffc02029ac: 17ffffae 	b	0xffffc0202864 <irqchip_set_pending>
ffffc02029b0: 39400261 	ldrb	w1, [x19]
ffffc02029b4: 7100043f 	cmp	w1, #1
ffffc02029b8: 54000321 	b.ne	0xffffc0202a1c <gic_handle_sgir_write+0x8c>
ffffc02029bc: b95002c0 	ldr	w0, [x22, #4096]
ffffc02029c0: 6b14001f 	cmp	w0, w20
ffffc02029c4: 540000a0 	b.eq	0xffffc02029d8 <gic_handle_sgir_write+0x48>
ffffc02029c8: 79402661 	ldrh	w1, [x19, #18]
ffffc02029cc: 8b153ae0 	add	x0, x23, x21, lsl #14
ffffc02029d0: 91400800 	add	x0, x0, #2, lsl #12     // =8192
ffffc02029d4: 97ffffa4 	bl	0xffffc0202864 <irqchip_set_pending>
ffffc02029d8: f94806c0 	ldr	x0, [x22, #4104]
ffffc02029dc: 12800002 	mov	w2, #-1
ffffc02029e0: f9485401 	ldr	x1, [x0, #4264]
ffffc02029e4: 2a1403e0 	mov	w0, w20
ffffc02029e8: 94001498 	bl	0xffffc0207c48 <next_cpu>
ffffc02029ec: 2a0003f5 	mov	w21, w0
ffffc02029f0: f94806c1 	ldr	x1, [x22, #4104]
ffffc02029f4: aa1503f4 	mov	x20, x21
ffffc02029f8: f9485421 	ldr	x1, [x1, #4264]
ffffc02029fc: f9400021 	ldr	x1, [x1]
ffffc0202a00: eb0102bf 	cmp	x21, x1
ffffc0202a04: 54fffd69 	b.ls	0xffffc02029b0 <gic_handle_sgir_write+0x20>
ffffc0202a08: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202a0c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202a10: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0202a14: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0202a18: d65f03c0 	ret
ffffc0202a1c: 97ffff8c 	bl	0xffffc020284c <irqchip_get_cpu_target>
ffffc0202a20: 2a0003f8 	mov	w24, w0
ffffc0202a24: 2a1403e0 	mov	w0, w20
ffffc0202a28: 97ffff8c 	bl	0xffffc0202858 <irqchip_get_cluster_target>
ffffc0202a2c: f9400661 	ldr	x1, [x19, #8]
ffffc0202a30: eb00003f 	cmp	x1, x0
ffffc0202a34: 54fffd21 	b.ne	0xffffc02029d8 <gic_handle_sgir_write+0x48>
ffffc0202a38: 79402260 	ldrh	w0, [x19, #16]
ffffc0202a3c: 6a18001f 	tst	w0, w24
ffffc0202a40: 54fffc41 	b.ne	0xffffc02029c8 <gic_handle_sgir_write+0x38>
ffffc0202a44: 17ffffe5 	b	0xffffc02029d8 <gic_handle_sgir_write+0x48>
ffffc0202a48: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0202a4c: 910003fd 	mov	x29, sp
ffffc0202a50: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0202a54: d2840016 	mov	x22, #8192
ffffc0202a58: f2a00036 	movk	x22, #1, lsl #16
ffffc0202a5c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202a60: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0202a64: aa0003f3 	mov	x19, x0
ffffc0202a68: f00000d7 	adrp	x23, 0xffffc021d000 <irqchip_cpu_shutdown+0x44>
ffffc0202a6c: 12800014 	mov	w20, #-1
ffffc0202a70: f2dfe016 	movk	x22, #65280, lsl #32
ffffc0202a74: 910002f7 	add	x23, x23, #0
ffffc0202a78: 17ffffd8 	b	0xffffc02029d8 <gic_handle_sgir_write+0x48>

0000ffffc0202a7c <irqchip_cpu_reset>:
ffffc0202a7c: f0000081 	adrp	x1, 0xffffc0215000 <irqchip_cpu_shutdown+0x38>
ffffc0202a80: b934641f 	str	wzr, [x0, #13412]
ffffc0202a84: b934681f 	str	wzr, [x0, #13416]
ffffc0202a88: f9402421 	ldr	x1, [x1, #72]
ffffc0202a8c: d61f0020 	br	x1

0000ffffc0202a90 <irqchip_cpu_shutdown>:
ffffc0202a90: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202a94: 910003fd 	mov	x29, sp
ffffc0202a98: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202a9c: f0000094 	adrp	x20, 0xffffc0215000 <irqchip_cpu_shutdown+0x58>
ffffc0202aa0: f90013f5 	str	x21, [sp, #32]
ffffc0202aa4: 9100e295 	add	x21, x20, #56
ffffc0202aa8: aa0003f3 	mov	x19, x0
ffffc0202aac: f9400ea1 	ldr	x1, [x21, #24]
ffffc0202ab0: d63f0020 	blr	x1
ffffc0202ab4: 37f80160 	tbnz	w0, #31, 0xffffc0202ae0 <irqchip_cpu_shutdown+0x50>
ffffc0202ab8: f94036a0 	ldr	x0, [x21, #104]
ffffc0202abc: d63f0000 	blr	x0
ffffc0202ac0: 36f80180 	tbz	w0, #31, 0xffffc0202af0 <irqchip_cpu_shutdown+0x60>
ffffc0202ac4: d2820c00 	mov	x0, #4192
ffffc0202ac8: 9100e294 	add	x20, x20, #56
ffffc0202acc: 8b000273 	add	x19, x19, x0
ffffc0202ad0: b9440a61 	ldr	w1, [x19, #1032]
ffffc0202ad4: b9440660 	ldr	w0, [x19, #1028]
ffffc0202ad8: 6b01001f 	cmp	w0, w1
ffffc0202adc: 54000101 	b.ne	0xffffc0202afc <irqchip_cpu_shutdown+0x6c>
ffffc0202ae0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202ae4: f94013f5 	ldr	x21, [sp, #32]
ffffc0202ae8: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202aec: d65f03c0 	ret
ffffc0202af0: f9403aa1 	ldr	x1, [x21, #112]
ffffc0202af4: d63f0020 	blr	x1
ffffc0202af8: 17fffff0 	b	0xffffc0202ab8 <irqchip_cpu_shutdown+0x28>
ffffc0202afc: 8b204660 	add	x0, x19, w0, uxtw #1
ffffc0202b00: f9403a81 	ldr	x1, [x20, #112]
ffffc0202b04: 79400800 	ldrh	w0, [x0, #4]
ffffc0202b08: d63f0020 	blr	x1
ffffc0202b0c: b9440660 	ldr	w0, [x19, #1028]
ffffc0202b10: 11000400 	add	w0, w0, #1
ffffc0202b14: 12001c00 	and	w0, w0, #0xff
ffffc0202b18: b9040660 	str	w0, [x19, #1028]
ffffc0202b1c: 17ffffed 	b	0xffffc0202ad0 <irqchip_cpu_shutdown+0x40>

0000ffffc0202b20 <irqchip_cell_reset>:
ffffc0202b20: f0000082 	adrp	x2, 0xffffc0215000 <irqchip_cell_exit+0x14>
ffffc0202b24: 9100e042 	add	x2, x2, #56
ffffc0202b28: d2803081 	mov	x1, #388
ffffc0202b2c: 8b010003 	add	x3, x0, x1
ffffc0202b30: f9405444 	ldr	x4, [x2, #168]
ffffc0202b34: b94e9863 	ldr	w3, [x3, #3736]
ffffc0202b38: b8216883 	str	w3, [x4, x1]
ffffc0202b3c: 91080024 	add	x4, x1, #512
ffffc0202b40: 91001021 	add	x1, x1, #4
ffffc0202b44: f9405445 	ldr	x5, [x2, #168]
ffffc0202b48: f108003f 	cmp	x1, #512
ffffc0202b4c: b82468a3 	str	w3, [x5, x4]
ffffc0202b50: 54fffee1 	b.ne	0xffffc0202b2c <irqchip_cell_reset+0xc>
ffffc0202b54: d65f03c0 	ret

0000ffffc0202b58 <irqchip_cell_exit>:
ffffc0202b58: f0000086 	adrp	x6, 0xffffc0215000 <irqchip_cell_exit+0x4c>
ffffc0202b5c: 9100e0c1 	add	x1, x6, #56
ffffc0202b60: b940b421 	ldr	w1, [x1, #180]
ffffc0202b64: 34000921 	cbz	w1, 0xffffc0202c88 <irqchip_cell_exit+0x130>
ffffc0202b68: aa0003e7 	mov	x7, x0
ffffc0202b6c: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0202b70: 910003fd 	mov	x29, sp
ffffc0202b74: 97ffffeb 	bl	0xffffc0202b20 <irqchip_cell_reset>
ffffc0202b78: f9485009 	ldr	x9, [x0, #4256]
ffffc0202b7c: aa0903e0 	mov	x0, x9
ffffc0202b80: 97fffd92 	bl	0xffffc02021c8 <jailhouse_cell_irqchips>
ffffc0202b84: f00000a1 	adrp	x1, 0xffffc0219000 <irqchip_cell_exit+0x88>
ffffc0202b88: 900000c4 	adrp	x4, 0xffffc021a000 <irqchip_cell_exit+0x90>
ffffc0202b8c: 91000084 	add	x4, x4, #0
ffffc0202b90: 52800003 	mov	w3, #0
ffffc0202b94: f9400028 	ldr	x8, [x1]
ffffc0202b98: b9403d21 	ldr	w1, [x9, #60]
ffffc0202b9c: 6b01007f 	cmp	w3, w1
ffffc0202ba0: 54000203 	b.lo	0xffffc0202be0 <irqchip_cell_exit+0x88>
ffffc0202ba4: 900000c4 	adrp	x4, 0xffffc021a000 <irqchip_cell_exit+0xac>
ffffc0202ba8: 91000084 	add	x4, x4, #0
ffffc0202bac: f9485089 	ldr	x9, [x4, #4256]
ffffc0202bb0: aa0903e0 	mov	x0, x9
ffffc0202bb4: 97fffd85 	bl	0xffffc02021c8 <jailhouse_cell_irqchips>
ffffc0202bb8: 52800003 	mov	w3, #0
ffffc0202bbc: b9403d21 	ldr	w1, [x9, #60]
ffffc0202bc0: 6b01007f 	cmp	w3, w1
ffffc0202bc4: 54000363 	b.lo	0xffffc0202c30 <irqchip_cell_exit+0xd8>
ffffc0202bc8: 9100e0c6 	add	x6, x6, #56
ffffc0202bcc: f94014c1 	ldr	x1, [x6, #40]
ffffc0202bd0: b4000581 	cbz	x1, 0xffffc0202c80 <irqchip_cell_exit+0x128>
ffffc0202bd4: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0202bd8: aa0703e0 	mov	x0, x7
ffffc0202bdc: d61f0020 	br	x1
ffffc0202be0: f9400002 	ldr	x2, [x0]
ffffc0202be4: f845a101 	ldur	x1, [x8, #90]
ffffc0202be8: eb01005f 	cmp	x2, x1
ffffc0202bec: 540001c1 	b.ne	0xffffc0202c24 <irqchip_cell_exit+0xcc>
ffffc0202bf0: d2800002 	mov	x2, #0
ffffc0202bf4: b9400c01 	ldr	w1, [x0, #12]
ffffc0202bf8: 8b020805 	add	x5, x0, x2, lsl #2
ffffc0202bfc: 0b411441 	add	w1, w2, w1, lsr #5
ffffc0202c00: b94010a5 	ldr	w5, [x5, #16]
ffffc0202c04: 91101021 	add	x1, x1, #1028
ffffc0202c08: 91000442 	add	x2, x2, #1
ffffc0202c0c: f100105f 	cmp	x2, #4
ffffc0202c10: 8b010881 	add	x1, x4, x1, lsl #2
ffffc0202c14: b940082a 	ldr	w10, [x1, #8]
ffffc0202c18: 2a0a00a5 	orr	w5, w5, w10
ffffc0202c1c: b9000825 	str	w5, [x1, #8]
ffffc0202c20: 54fffea1 	b.ne	0xffffc0202bf4 <irqchip_cell_exit+0x9c>
ffffc0202c24: 91008000 	add	x0, x0, #32
ffffc0202c28: 11000463 	add	w3, w3, #1
ffffc0202c2c: 17ffffdb 	b	0xffffc0202b98 <irqchip_cell_exit+0x40>
ffffc0202c30: f9400002 	ldr	x2, [x0]
ffffc0202c34: f845a101 	ldur	x1, [x8, #90]
ffffc0202c38: eb01005f 	cmp	x2, x1
ffffc0202c3c: 540001c1 	b.ne	0xffffc0202c74 <irqchip_cell_exit+0x11c>
ffffc0202c40: d2800002 	mov	x2, #0
ffffc0202c44: b9400c01 	ldr	w1, [x0, #12]
ffffc0202c48: 8b020805 	add	x5, x0, x2, lsl #2
ffffc0202c4c: 0b411441 	add	w1, w2, w1, lsr #5
ffffc0202c50: b94010a5 	ldr	w5, [x5, #16]
ffffc0202c54: 91101021 	add	x1, x1, #1028
ffffc0202c58: 91000442 	add	x2, x2, #1
ffffc0202c5c: f100105f 	cmp	x2, #4
ffffc0202c60: 8b010881 	add	x1, x4, x1, lsl #2
ffffc0202c64: b940082a 	ldr	w10, [x1, #8]
ffffc0202c68: 0a0a00a5 	and	w5, w5, w10
ffffc0202c6c: b9000825 	str	w5, [x1, #8]
ffffc0202c70: 54fffea1 	b.ne	0xffffc0202c44 <irqchip_cell_exit+0xec>
ffffc0202c74: 91008000 	add	x0, x0, #32
ffffc0202c78: 11000463 	add	w3, w3, #1
ffffc0202c7c: 17ffffd0 	b	0xffffc0202bbc <irqchip_cell_exit+0x64>
ffffc0202c80: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0202c84: d65f03c0 	ret
ffffc0202c88: d65f03c0 	ret

0000ffffc0202c8c <irqchip_config_commit>:
ffffc0202c8c: b4000440 	cbz	x0, 0xffffc0202d14 <irqchip_config_commit+0x88>
ffffc0202c90: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202c94: 910003fd 	mov	x29, sp
ffffc0202c98: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202c9c: aa0003f4 	mov	x20, x0
ffffc0202ca0: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0202ca4: f0000096 	adrp	x22, 0xffffc0215000 <irqchip_config_commit+0x64>
ffffc0202ca8: 900000d5 	adrp	x21, 0xffffc021a000 <irqchip_config_commit+0x7c>
ffffc0202cac: 9100e2d6 	add	x22, x22, #56
ffffc0202cb0: 910002b5 	add	x21, x21, #0
ffffc0202cb4: 52800413 	mov	w19, #32
ffffc0202cb8: 2a1303e1 	mov	w1, w19
ffffc0202cbc: aa1403e0 	mov	x0, x20
ffffc0202cc0: 97fffde3 	bl	0xffffc020244c <irqchip_irq_in_cell>
ffffc0202cc4: 340000a0 	cbz	w0, 0xffffc0202cd8 <irqchip_config_commit+0x4c>
ffffc0202cc8: f9401ac2 	ldr	x2, [x22, #48]
ffffc0202ccc: 2a1303e1 	mov	w1, w19
ffffc0202cd0: aa1403e0 	mov	x0, x20
ffffc0202cd4: d63f0040 	blr	x2
ffffc0202cd8: 2a1303e1 	mov	w1, w19
ffffc0202cdc: aa1503e0 	mov	x0, x21
ffffc0202ce0: 97fffddb 	bl	0xffffc020244c <irqchip_irq_in_cell>
ffffc0202ce4: 340000a0 	cbz	w0, 0xffffc0202cf8 <irqchip_config_commit+0x6c>
ffffc0202ce8: f9401ac2 	ldr	x2, [x22, #48]
ffffc0202cec: 2a1303e1 	mov	w1, w19
ffffc0202cf0: aa1503e0 	mov	x0, x21
ffffc0202cf4: d63f0040 	blr	x2
ffffc0202cf8: 11000673 	add	w19, w19, #1
ffffc0202cfc: 7110027f 	cmp	w19, #1024
ffffc0202d00: 54fffdc1 	b.ne	0xffffc0202cb8 <irqchip_config_commit+0x2c>
ffffc0202d04: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202d08: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202d0c: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202d10: d65f03c0 	ret
ffffc0202d14: d65f03c0 	ret

0000ffffc0202d18 <arch_ivshmem_trigger_interrupt>:
ffffc0202d18: 79412801 	ldrh	w1, [x0, #148]
ffffc0202d1c: 34000061 	cbz	w1, 0xffffc0202d28 <arch_ivshmem_trigger_interrupt+0x10>
ffffc0202d20: d2800000 	mov	x0, #0
ffffc0202d24: 17fffed0 	b	0xffffc0202864 <irqchip_set_pending>
ffffc0202d28: d65f03c0 	ret

0000ffffc0202d2c <arch_ivshmem_update_msix>:
ffffc0202d2c: f9400001 	ldr	x1, [x0]
ffffc0202d30: 79404821 	ldrh	w1, [x1, #36]
ffffc0202d34: 34000381 	cbz	w1, 0xffffc0202da4 <arch_ivshmem_update_msix+0x78>
ffffc0202d38: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202d3c: 910003fd 	mov	x29, sp
ffffc0202d40: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202d44: aa0003f3 	mov	x19, x0
ffffc0202d48: f90013f5 	str	x21, [sp, #32]
ffffc0202d4c: f9402015 	ldr	x21, [x0, #64]
ffffc0202d50: aa1503e0 	mov	x0, x21
ffffc0202d54: 94001f3f 	bl	0xffffc020aa50 <ivshmem_is_msix_masked>
ffffc0202d58: 35000220 	cbnz	w0, 0xffffc0202d9c <arch_ivshmem_update_msix+0x70>
ffffc0202d5c: f9402a60 	ldr	x0, [x19, #80]
ffffc0202d60: b9400814 	ldr	w20, [x0, #8]
ffffc0202d64: 71007e9f 	cmp	w20, #31
ffffc0202d68: 540000c8 	b.hi	0xffffc0202d80 <arch_ivshmem_update_msix+0x54>
ffffc0202d6c: 12800000 	mov	w0, #-1
ffffc0202d70: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202d74: f94013f5 	ldr	x21, [sp, #32]
ffffc0202d78: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202d7c: d65f03c0 	ret
ffffc0202d80: f9400660 	ldr	x0, [x19, #8]
ffffc0202d84: 2a1403e1 	mov	w1, w20
ffffc0202d88: 97fffdb1 	bl	0xffffc020244c <irqchip_irq_in_cell>
ffffc0202d8c: 34ffff00 	cbz	w0, 0xffffc0202d6c <arch_ivshmem_update_msix+0x40>
ffffc0202d90: 79012ab4 	strh	w20, [x21, #148]
ffffc0202d94: 52800000 	mov	w0, #0
ffffc0202d98: 17fffff6 	b	0xffffc0202d70 <arch_ivshmem_update_msix+0x44>
ffffc0202d9c: 52800014 	mov	w20, #0
ffffc0202da0: 17fffffc 	b	0xffffc0202d90 <arch_ivshmem_update_msix+0x64>
ffffc0202da4: 52800000 	mov	w0, #0
ffffc0202da8: d65f03c0 	ret

0000ffffc0202dac <arch_ivshmem_update_intx>:
ffffc0202dac: f9403c01 	ldr	x1, [x0, #120]
ffffc0202db0: f9400022 	ldr	x2, [x1]
ffffc0202db4: 79404842 	ldrh	w2, [x2, #36]
ffffc0202db8: 35000162 	cbnz	w2, 0xffffc0202de4 <arch_ivshmem_update_intx+0x38>
ffffc0202dbc: b9409802 	ldr	w2, [x0, #152]
ffffc0202dc0: 36000142 	tbz	w2, #0, 0xffffc0202de8 <arch_ivshmem_update_intx+0x3c>
ffffc0202dc4: f9400421 	ldr	x1, [x1, #8]
ffffc0202dc8: 3940f402 	ldrb	w2, [x0, #61]
ffffc0202dcc: f9485021 	ldr	x1, [x1, #4256]
ffffc0202dd0: 11007c42 	add	w2, w2, #31
ffffc0202dd4: b9404c21 	ldr	w1, [x1, #76]
ffffc0202dd8: 0b020021 	add	w1, w1, w2
ffffc0202ddc: 12003c21 	and	w1, w1, #0xffff
ffffc0202de0: 79012801 	strh	w1, [x0, #148]
ffffc0202de4: d65f03c0 	ret
ffffc0202de8: 52800001 	mov	w1, #0
ffffc0202dec: 17fffffd 	b	0xffffc0202de0 <arch_ivshmem_update_intx+0x34>

0000ffffc0202df0 <phys_processor_id>:
ffffc0202df0: d53800a0 	mrs	x0, MPIDR_EL1
ffffc0202df4: 92409c00 	and	x0, x0, #0xffffffffff
ffffc0202df8: 9260dc00 	and	x0, x0, #0xffffffff00ffffff
ffffc0202dfc: d65f03c0 	ret

0000ffffc0202e00 <arm_cpu_by_mpidr>:
ffffc0202e00: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0202e04: 910003fd 	mov	x29, sp
ffffc0202e08: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202e0c: f00000d3 	adrp	x19, 0xffffc021d000 <arch_map_memory_region>
ffffc0202e10: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0202e14: 91000273 	add	x19, x19, #0
ffffc0202e18: b2405ff6 	mov	x22, #16777215
ffffc0202e1c: aa0103f5 	mov	x21, x1
ffffc0202e20: aa0003f4 	mov	x20, x0
ffffc0202e24: f2c01ff6 	movk	x22, #255, lsl #32
ffffc0202e28: 12800000 	mov	w0, #-1
ffffc0202e2c: f9485681 	ldr	x1, [x20, #4264]
ffffc0202e30: 12800002 	mov	w2, #-1
ffffc0202e34: 94001385 	bl	0xffffc0207c48 <next_cpu>
ffffc0202e38: 2a0003e1 	mov	w1, w0
ffffc0202e3c: f9485682 	ldr	x2, [x20, #4264]
ffffc0202e40: f9400042 	ldr	x2, [x2]
ffffc0202e44: eb02003f 	cmp	x1, x2
ffffc0202e48: 54000069 	b.ls	0xffffc0202e54 <arm_cpu_by_mpidr+0x54>
ffffc0202e4c: 12800000 	mov	w0, #-1
ffffc0202e50: 14000006 	b	0xffffc0202e68 <arm_cpu_by_mpidr+0x68>
ffffc0202e54: 8b013a61 	add	x1, x19, x1, lsl #14
ffffc0202e58: f9582421 	ldr	x1, [x1, #12360]
ffffc0202e5c: 8a160021 	and	x1, x1, x22
ffffc0202e60: eb15003f 	cmp	x1, x21
ffffc0202e64: 54fffe41 	b.ne	0xffffc0202e2c <arm_cpu_by_mpidr+0x2c>
ffffc0202e68: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202e6c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202e70: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0202e74: d65f03c0 	ret

0000ffffc0202e78 <arch_map_memory_region>:
ffffc0202e78: aa0103e6 	mov	x6, x1
ffffc0202e7c: 52808022 	mov	w2, #1025
ffffc0202e80: 52808824 	mov	w4, #1089
ffffc0202e84: f9400021 	ldr	x1, [x1]
ffffc0202e88: f9400cc3 	ldr	x3, [x6, #24]
ffffc0202e8c: f240007f 	tst	x3, #0x1
ffffc0202e90: 1a840042 	csel	w2, w2, w4, eq
ffffc0202e94: 36080043 	tbz	w3, #1, 0xffffc0202e9c <arch_map_memory_region+0x24>
ffffc0202e98: 32190042 	orr	w2, w2, #0x80
ffffc0202e9c: f27c007f 	tst	x3, #0x10
ffffc0202ea0: 321e0044 	orr	w4, w2, #0x4
ffffc0202ea4: 321e0c42 	orr	w2, w2, #0x3c
ffffc0202ea8: 9a840044 	csel	x4, x2, x4, eq
ffffc0202eac: 36280083 	tbz	w3, #5, 0xffffc0202ebc <arch_map_memory_region+0x44>
ffffc0202eb0: d00000a1 	adrp	x1, 0xffffc0218000 <arm_cell_dcaches_flush+0x8>
ffffc0202eb4: f9400422 	ldr	x2, [x1, #8]
ffffc0202eb8: cb020001 	sub	x1, x0, x2
ffffc0202ebc: a94088c3 	ldp	x3, x2, [x6, #8]
ffffc0202ec0: 52800005 	mov	w5, #0
ffffc0202ec4: 91400400 	add	x0, x0, #1, lsl #12     // =4096
ffffc0202ec8: 140010b0 	b	0xffffc0207188 <paging_create>

0000ffffc0202ecc <arch_unmap_memory_region>:
ffffc0202ecc: f9400822 	ldr	x2, [x1, #16]
ffffc0202ed0: 52800003 	mov	w3, #0
ffffc0202ed4: f9400421 	ldr	x1, [x1, #8]
ffffc0202ed8: 91400400 	add	x0, x0, #1, lsl #12     // =4096
ffffc0202edc: 14001040 	b	0xffffc0206fdc <paging_destroy>

0000ffffc0202ee0 <arch_paging_gphys2phys>:
ffffc0202ee0: d2840002 	mov	x2, #8192
ffffc0202ee4: f2a00022 	movk	x2, #1, lsl #16
ffffc0202ee8: f2dfe002 	movk	x2, #65280, lsl #32
ffffc0202eec: f9480443 	ldr	x3, [x2, #4104]
ffffc0202ef0: aa0103e2 	mov	x2, x1
ffffc0202ef4: aa0003e1 	mov	x1, x0
ffffc0202ef8: 91400460 	add	x0, x3, #1, lsl #12     // =4096
ffffc0202efc: 14001014 	b	0xffffc0206f4c <paging_virt2phys>

0000ffffc0202f00 <arm_cell_dcaches_flush>:
ffffc0202f00: a9ba7bfd 	stp	x29, x30, [sp, #-96]!
ffffc0202f04: 910003fd 	mov	x29, sp
ffffc0202f08: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0202f0c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0202f10: 52800016 	mov	w22, #0
ffffc0202f14: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0202f18: aa0003f8 	mov	x24, x0
ffffc0202f1c: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc0202f20: 2a0103f9 	mov	w25, w1
ffffc0202f24: f9002bfb 	str	x27, [sp, #80]
ffffc0202f28: d282001b 	mov	x27, #4096
ffffc0202f2c: f2a0003b 	movk	x27, #1, lsl #16
ffffc0202f30: d2a0003a 	mov	x26, #65536
ffffc0202f34: f9485013 	ldr	x19, [x0, #4256]
ffffc0202f38: f2dfe01b 	movk	x27, #65280, lsl #32
ffffc0202f3c: b9403260 	ldr	w0, [x19, #48]
ffffc0202f40: 91020000 	add	x0, x0, #128
ffffc0202f44: 8b000273 	add	x19, x19, x0
ffffc0202f48: f9485300 	ldr	x0, [x24, #4256]
ffffc0202f4c: b9403400 	ldr	w0, [x0, #52]
ffffc0202f50: 6b16001f 	cmp	w0, w22
ffffc0202f54: 54000128 	b.hi	0xffffc0202f78 <arm_cell_dcaches_flush+0x78>
ffffc0202f58: d5033bbf 	dmb	ish
ffffc0202f5c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0202f60: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0202f64: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0202f68: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc0202f6c: f9402bfb 	ldr	x27, [sp, #80]
ffffc0202f70: a8c67bfd 	ldp	x29, x30, [sp], #96
ffffc0202f74: d65f03c0 	ret
ffffc0202f78: f9400e60 	ldr	x0, [x19, #24]
ffffc0202f7c: f27c041f 	tst	x0, #0x30
ffffc0202f80: 54000080 	b.eq	0xffffc0202f90 <arm_cell_dcaches_flush+0x90>
ffffc0202f84: 91008273 	add	x19, x19, #32
ffffc0202f88: 110006d6 	add	w22, w22, #1
ffffc0202f8c: 17ffffef 	b	0xffffc0202f48 <arm_cell_dcaches_flush+0x48>
ffffc0202f90: f9400277 	ldr	x23, [x19]
ffffc0202f94: f9400a74 	ldr	x20, [x19, #16]
ffffc0202f98: b4ffff74 	cbz	x20, 0xffffc0202f84 <arm_cell_dcaches_flush+0x84>
ffffc0202f9c: f140429f 	cmp	x20, #16, lsl #12       // =65536
ffffc0202fa0: 52800025 	mov	w5, #1
ffffc0202fa4: 9a9a9295 	csel	x21, x20, x26, ls
ffffc0202fa8: d280e824 	mov	x4, #1857
ffffc0202fac: d2dfe003 	mov	x3, #280375465082880
ffffc0202fb0: aa1703e1 	mov	x1, x23
ffffc0202fb4: aa1503e2 	mov	x2, x21
ffffc0202fb8: aa1b03e0 	mov	x0, x27
ffffc0202fbc: 94001073 	bl	0xffffc0207188 <paging_create>
ffffc0202fc0: 8b1502f7 	add	x23, x23, x21
ffffc0202fc4: 2a1903e2 	mov	w2, w25
ffffc0202fc8: aa1503e1 	mov	x1, x21
ffffc0202fcc: d2dfe000 	mov	x0, #280375465082880
ffffc0202fd0: cb150294 	sub	x20, x20, x21
ffffc0202fd4: 94000240 	bl	0xffffc02038d4 <arm_dcaches_flush>
ffffc0202fd8: 17fffff0 	b	0xffffc0202f98 <arm_cell_dcaches_flush+0x98>

0000ffffc0202fdc <arm_paging_cell_init>:
ffffc0202fdc: f9485001 	ldr	x1, [x0, #4256]
ffffc0202fe0: b9402821 	ldr	w1, [x1, #40]
ffffc0202fe4: 7103fc3f 	cmp	w1, #255
ffffc0202fe8: 54000348 	b.hi	0xffffc0203050 <arm_paging_cell_init+0x74>
ffffc0202fec: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0202ff0: 52800021 	mov	w1, #1
ffffc0202ff4: 910003fd 	mov	x29, sp
ffffc0202ff8: f9000bf3 	str	x19, [sp, #16]
ffffc0202ffc: aa0003f3 	mov	x19, x0
ffffc0203000: d0000080 	adrp	x0, 0xffffc0215000 <arm_paging_cell_init+0x6c>
ffffc0203004: f9407c00 	ldr	x0, [x0, #248]
ffffc0203008: f9080660 	str	x0, [x19, #4104]
ffffc020300c: d0000080 	adrp	x0, 0xffffc0215000 <arm_paging_cell_init+0x78>
ffffc0203010: b940f002 	ldr	w2, [x0, #240]
ffffc0203014: 5100a040 	sub	w0, w2, #40
ffffc0203018: 51009c42 	sub	w2, w2, #39
ffffc020301c: 7100101f 	cmp	w0, #4
ffffc0203020: b00000a0 	adrp	x0, 0xffffc0218000 <arm_paging_cell_destroy+0x1c>
ffffc0203024: 1ac22022 	lsl	w2, w1, w2
ffffc0203028: 91004000 	add	x0, x0, #16
ffffc020302c: 1a813041 	csel	w1, w2, w1, lo
ffffc0203030: 94000f94 	bl	0xffffc0206e80 <page_alloc_aligned>
ffffc0203034: f9080a60 	str	x0, [x19, #4112]
ffffc0203038: f100001f 	cmp	x0, #0
ffffc020303c: 12800160 	mov	w0, #-12
ffffc0203040: f9400bf3 	ldr	x19, [sp, #16]
ffffc0203044: 1a9f0000 	csel	w0, w0, wzr, eq
ffffc0203048: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020304c: d65f03c0 	ret
ffffc0203050: 128000c0 	mov	w0, #-7
ffffc0203054: d65f03c0 	ret

0000ffffc0203058 <arm_paging_cell_destroy>:
ffffc0203058: d0000081 	adrp	x1, 0xffffc0215000 <arm_paging_vcpu_init+0x18>
ffffc020305c: 52800022 	mov	w2, #1
ffffc0203060: b940f021 	ldr	w1, [x1, #240]
ffffc0203064: 5100a023 	sub	w3, w1, #40
ffffc0203068: 51009c21 	sub	w1, w1, #39
ffffc020306c: 7100107f 	cmp	w3, #4
ffffc0203070: 1ac12041 	lsl	w1, w2, w1
ffffc0203074: 1a823022 	csel	w2, w1, w2, lo
ffffc0203078: f9480801 	ldr	x1, [x0, #4112]
ffffc020307c: b00000a0 	adrp	x0, 0xffffc0218000 <arm_paging_vcpu_init+0x48>
ffffc0203080: 91004000 	add	x0, x0, #16
ffffc0203084: 14000f81 	b	0xffffc0206e88 <page_free>

0000ffffc0203088 <arm_paging_vcpu_init>:
ffffc0203088: b00000a1 	adrp	x1, 0xffffc0218000 <arm_paging_vcpu_init+0x54>
ffffc020308c: f9400800 	ldr	x0, [x0, #16]
ffffc0203090: f9400421 	ldr	x1, [x1, #8]
ffffc0203094: cb010000 	sub	x0, x0, x1
ffffc0203098: d2840001 	mov	x1, #8192
ffffc020309c: f2a00021 	movk	x1, #1, lsl #16
ffffc02030a0: 92748c00 	and	x0, x0, #0xfffffffff000
ffffc02030a4: f2dfe001 	movk	x1, #65280, lsl #32
ffffc02030a8: f9480421 	ldr	x1, [x1, #4104]
ffffc02030ac: f9485021 	ldr	x1, [x1, #4256]
ffffc02030b0: b9402821 	ldr	w1, [x1, #40]
ffffc02030b4: aa01c000 	orr	x0, x0, x1, lsl #48
ffffc02030b8: d51c2100 	msr	VTTBR_EL2, x0
ffffc02030bc: d0000080 	adrp	x0, 0xffffc0215000 <arm_paging_vcpu_init+0x7c>
ffffc02030c0: 52800803 	mov	w3, #64
ffffc02030c4: 5286a002 	mov	w2, #13568
ffffc02030c8: b940f001 	ldr	w1, [x0, #240]
ffffc02030cc: 72b00002 	movk	w2, #32768, lsl #16
ffffc02030d0: 4b010060 	sub	w0, w3, w1
ffffc02030d4: 7100ac3f 	cmp	w1, #43
ffffc02030d8: 51009021 	sub	w1, w1, #36
ffffc02030dc: 2a020002 	orr	w2, w0, w2
ffffc02030e0: 52801000 	mov	w0, #128
ffffc02030e4: 1a838003 	csel	w3, w0, w3, hi
ffffc02030e8: 7100303f 	cmp	w1, #12
ffffc02030ec: 54000148 	b.hi	0xffffc0203114 <arm_paging_vcpu_init+0x8c>
ffffc02030f0: 90000040 	adrp	x0, 0xffffc020b000 <arm_paging_vcpu_init+0x88>
ffffc02030f4: 910a4000 	add	x0, x0, #656
ffffc02030f8: 38614800 	ldrb	w0, [x0, w1, uxtw]
ffffc02030fc: 2a004060 	orr	w0, w3, w0, lsl #16
ffffc0203100: 2a020000 	orr	w0, w0, w2
ffffc0203104: d51c2140 	msr	VTCR_EL2, x0
ffffc0203108: d5033fdf 	isb
ffffc020310c: d50c83df 	tlbi	vmalls12e1is
ffffc0203110: d65f03c0 	ret
ffffc0203114: 52800000 	mov	w0, #0
ffffc0203118: 17fffff9 	b	0xffffc02030fc <arm_paging_vcpu_init+0x74>

0000ffffc020311c <arm_entry_valid>:
ffffc020311c: f9400000 	ldr	x0, [x0]
ffffc0203120: 12000000 	and	w0, w0, #0x1
ffffc0203124: d65f03c0 	ret

0000ffffc0203128 <arm_get_entry_flags>:
ffffc0203128: f9400000 	ldr	x0, [x0]
ffffc020312c: 92402c00 	and	x0, x0, #0xfff
ffffc0203130: d65f03c0 	ret

0000ffffc0203134 <arm_clear_entry>:
ffffc0203134: f900001f 	str	xzr, [x0]
ffffc0203138: d65f03c0 	ret

0000ffffc020313c <arm_page_table_empty>:
ffffc020313c: d2800001 	mov	x1, #0
ffffc0203140: f8617802 	ldr	x2, [x0, x1, lsl #3]
ffffc0203144: 370000c2 	tbnz	w2, #0, 0xffffc020315c <arm_page_table_empty+0x20>
ffffc0203148: 91000421 	add	x1, x1, #1
ffffc020314c: f108003f 	cmp	x1, #512
ffffc0203150: 54ffff81 	b.ne	0xffffc0203140 <arm_page_table_empty+0x4>
ffffc0203154: 52800020 	mov	w0, #1
ffffc0203158: d65f03c0 	ret
ffffc020315c: 52800000 	mov	w0, #0
ffffc0203160: 17fffffe 	b	0xffffc0203158 <arm_page_table_empty+0x1c>

0000ffffc0203164 <arm_get_l0_entry>:
ffffc0203164: d367bc21 	ubfx	x1, x1, #39, #9
ffffc0203168: 8b010c00 	add	x0, x0, x1, lsl #3
ffffc020316c: d65f03c0 	ret

0000ffffc0203170 <arm_get_l0_phys>:
ffffc0203170: f9400000 	ldr	x0, [x0]
ffffc0203174: 92409821 	and	x1, x1, #0x7fffffffff
ffffc0203178: 92400402 	and	x2, x0, #0x3
ffffc020317c: 92592000 	and	x0, x0, #0xff8000000000
ffffc0203180: aa010000 	orr	x0, x0, x1
ffffc0203184: f1000c5f 	cmp	x2, #3
ffffc0203188: da9f1000 	csinv	x0, x0, xzr, ne
ffffc020318c: d65f03c0 	ret

0000ffffc0203190 <arm_get_l1_entry>:
ffffc0203190: d35e9821 	ubfx	x1, x1, #30, #9
ffffc0203194: 8b010c00 	add	x0, x0, x1, lsl #3
ffffc0203198: d65f03c0 	ret

0000ffffc020319c <arm_set_l1_block>:
ffffc020319c: 92624421 	and	x1, x1, #0xffffc0000000
ffffc02031a0: aa020021 	orr	x1, x1, x2
ffffc02031a4: f9000001 	str	x1, [x0]
ffffc02031a8: d65f03c0 	ret

0000ffffc02031ac <arm_get_l1_phys>:
ffffc02031ac: f9400000 	ldr	x0, [x0]
ffffc02031b0: 92407421 	and	x1, x1, #0x3fffffff
ffffc02031b4: 92400402 	and	x2, x0, #0x3
ffffc02031b8: 92624400 	and	x0, x0, #0xffffc0000000
ffffc02031bc: aa010000 	orr	x0, x0, x1
ffffc02031c0: f1000c5f 	cmp	x2, #3
ffffc02031c4: da9f1000 	csinv	x0, x0, xzr, ne
ffffc02031c8: d65f03c0 	ret

0000ffffc02031cc <arm_get_l1_alt_entry>:
ffffc02031cc: d35ec021 	ubfx	x1, x1, #30, #19
ffffc02031d0: 8b010c00 	add	x0, x0, x1, lsl #3
ffffc02031d4: d65f03c0 	ret

0000ffffc02031d8 <arm_get_l1_alt_phys>:
ffffc02031d8: f9400000 	ldr	x0, [x0]
ffffc02031dc: 92407421 	and	x1, x1, #0x3fffffff
ffffc02031e0: 92400402 	and	x2, x0, #0x3
ffffc02031e4: 92624800 	and	x0, x0, #0x1ffffc0000000
ffffc02031e8: aa010000 	orr	x0, x0, x1
ffffc02031ec: f1000c5f 	cmp	x2, #3
ffffc02031f0: da9f1000 	csinv	x0, x0, xzr, ne
ffffc02031f4: d65f03c0 	ret

0000ffffc02031f8 <arm_get_l2_entry>:
ffffc02031f8: d3557421 	ubfx	x1, x1, #21, #9
ffffc02031fc: 8b010c00 	add	x0, x0, x1, lsl #3
ffffc0203200: d65f03c0 	ret

0000ffffc0203204 <arm_get_l3_entry>:
ffffc0203204: d34c5021 	ubfx	x1, x1, #12, #9
ffffc0203208: 8b010c00 	add	x0, x0, x1, lsl #3
ffffc020320c: d65f03c0 	ret

0000ffffc0203210 <arm_set_l2_block>:
ffffc0203210: 926b6821 	and	x1, x1, #0xffffffe00000
ffffc0203214: aa020021 	orr	x1, x1, x2
ffffc0203218: f9000001 	str	x1, [x0]
ffffc020321c: d65f03c0 	ret

0000ffffc0203220 <arm_set_l3_page>:
ffffc0203220: b27f0042 	orr	x2, x2, #0x2
ffffc0203224: 92748c21 	and	x1, x1, #0xfffffffff000
ffffc0203228: aa020021 	orr	x1, x1, x2
ffffc020322c: f9000001 	str	x1, [x0]
ffffc0203230: d65f03c0 	ret

0000ffffc0203234 <arm_set_l12_table>:
ffffc0203234: 92748c21 	and	x1, x1, #0xfffffffff000
ffffc0203238: b2400421 	orr	x1, x1, #0x3
ffffc020323c: f9000001 	str	x1, [x0]
ffffc0203240: d65f03c0 	ret

0000ffffc0203244 <arm_get_l12_table>:
ffffc0203244: f9400000 	ldr	x0, [x0]
ffffc0203248: 92748c00 	and	x0, x0, #0xfffffffff000
ffffc020324c: d65f03c0 	ret

0000ffffc0203250 <arm_get_l2_phys>:
ffffc0203250: f9400000 	ldr	x0, [x0]
ffffc0203254: 92405021 	and	x1, x1, #0x1fffff
ffffc0203258: 92400402 	and	x2, x0, #0x3
ffffc020325c: 926b6800 	and	x0, x0, #0xffffffe00000
ffffc0203260: aa010000 	orr	x0, x0, x1
ffffc0203264: f1000c5f 	cmp	x2, #3
ffffc0203268: da9f1000 	csinv	x0, x0, xzr, ne
ffffc020326c: d65f03c0 	ret

0000ffffc0203270 <arm_get_l3_phys>:
ffffc0203270: f9400002 	ldr	x2, [x0]
ffffc0203274: 92402c21 	and	x1, x1, #0xfff
ffffc0203278: 92748c40 	and	x0, x2, #0xfffffffff000
ffffc020327c: aa010000 	orr	x0, x0, x1
ffffc0203280: f27f005f 	tst	x2, #0x2
ffffc0203284: da9f1000 	csinv	x0, x0, xzr, ne
ffffc0203288: d65f03c0 	ret

0000ffffc020328c <arch_paging_init>:
ffffc020328c: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0203290: 910003fd 	mov	x29, sp
ffffc0203294: 940009ab 	bl	0xffffc0205940 <get_cpu_parange>
ffffc0203298: 7100b01f 	cmp	w0, #44
ffffc020329c: d0000081 	adrp	x1, 0xffffc0215000 <arch_pci_suppress_msi>
ffffc02032a0: 9103c023 	add	x3, x1, #240
ffffc02032a4: b900f020 	str	w0, [x1, #240]
ffffc02032a8: 90000041 	adrp	x1, 0xffffc020b000 <arch_paging_init+0x3c>
ffffc02032ac: 910a8021 	add	x1, x1, #672
ffffc02032b0: b00000a0 	adrp	x0, 0xffffc0218000 <arch_pci_update_msi+0x1c>
ffffc02032b4: 9103c022 	add	x2, x1, #240
ffffc02032b8: 9a823021 	csel	x1, x1, x2, lo
ffffc02032bc: f9000461 	str	x1, [x3, #8]
ffffc02032c0: f9002002 	str	x2, [x0, #64]
ffffc02032c4: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc02032c8: d65f03c0 	ret

0000ffffc02032cc <arch_pci_read_config>:
ffffc02032cc: 12800000 	mov	w0, #-1
ffffc02032d0: d65f03c0 	ret

0000ffffc02032d4 <arch_pci_write_config>:
ffffc02032d4: d65f03c0 	ret

0000ffffc02032d8 <arch_pci_add_physical_device>:
ffffc02032d8: 52800000 	mov	w0, #0
ffffc02032dc: d65f03c0 	ret

0000ffffc02032e0 <arch_pci_remove_physical_device>:
ffffc02032e0: d65f03c0 	ret

0000ffffc02032e4 <arch_pci_suppress_msi>:
ffffc02032e4: d65f03c0 	ret

0000ffffc02032e8 <arch_pci_update_msi>:
ffffc02032e8: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc02032ec: 910003fd 	mov	x29, sp
ffffc02032f0: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02032f4: d2800033 	mov	x19, #1
ffffc02032f8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02032fc: aa0003f5 	mov	x21, x0
ffffc0203300: aa0103f6 	mov	x22, x1
ffffc0203304: f9400014 	ldr	x20, [x0]
ffffc0203308: 39408e80 	ldrb	w0, [x20, #35]
ffffc020330c: 7100001f 	cmp	w0, #0
ffffc0203310: 1a9f07e0 	cset	w0, ne
ffffc0203314: 11000c00 	add	w0, w0, #3
ffffc0203318: 6b13001f 	cmp	w0, w19
ffffc020331c: 540000c8 	b.hi	0xffffc0203334 <arch_pci_update_msi+0x4c>
ffffc0203320: 52800000 	mov	w0, #0
ffffc0203324: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0203328: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020332c: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0203330: d65f03c0 	ret
ffffc0203334: 8b130aa0 	add	x0, x21, x19, lsl #2
ffffc0203338: 794006c1 	ldrh	w1, [x22, #2]
ffffc020333c: 52800083 	mov	w3, #4
ffffc0203340: b9402802 	ldr	w2, [x0, #40]
ffffc0203344: 0b130821 	add	w1, w1, w19, lsl #2
ffffc0203348: 79400a80 	ldrh	w0, [x20, #4]
ffffc020334c: 91000673 	add	x19, x19, #1
ffffc0203350: 94001910 	bl	0xffffc0209790 <pci_write_config>
ffffc0203354: 17ffffed 	b	0xffffc0203308 <arch_pci_update_msi+0x20>

0000ffffc0203358 <arch_pci_update_msix_vector>:
ffffc0203358: a9448c02 	ldp	x2, x3, [x0, #72]
ffffc020335c: d37c7c21 	ubfiz	x1, x1, #4, #32
ffffc0203360: f8616863 	ldr	x3, [x3, x1]
ffffc0203364: f8216843 	str	x3, [x2, x1]
ffffc0203368: a9448002 	ldp	x2, x0, [x0, #72]
ffffc020336c: 8b010042 	add	x2, x2, x1
ffffc0203370: 8b010001 	add	x1, x0, x1
ffffc0203374: b9400820 	ldr	w0, [x1, #8]
ffffc0203378: b9000840 	str	w0, [x2, #8]
ffffc020337c: 52800000 	mov	w0, #0
ffffc0203380: d65f03c0 	ret

0000ffffc0203384 <psci_dispatch>:
ffffc0203384: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0203388: d2840002 	mov	x2, #8192
ffffc020338c: f2a00022 	movk	x2, #1, lsl #16
ffffc0203390: d2800144 	mov	x4, #10
ffffc0203394: 910003fd 	mov	x29, sp
ffffc0203398: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020339c: f2dfe002 	movk	x2, #65280, lsl #32
ffffc02033a0: f2b08004 	movk	x4, #33792, lsl #16
ffffc02033a4: b9502c41 	ldr	w1, [x2, #4140]
ffffc02033a8: 11000421 	add	w1, w1, #1
ffffc02033ac: b9102c41 	str	w1, [x2, #4140]
ffffc02033b0: f9400003 	ldr	x3, [x0]
ffffc02033b4: f9400061 	ldr	x1, [x3]
ffffc02033b8: eb04003f 	cmp	x1, x4
ffffc02033bc: 54000ea0 	b.eq	0xffffc0203590 <psci_dispatch+0x20c>
ffffc02033c0: aa0003f4 	mov	x20, x0
ffffc02033c4: 54000468 	b.hi	0xffffc0203450 <psci_dispatch+0xcc>
ffffc02033c8: d2800040 	mov	x0, #2
ffffc02033cc: f2b08000 	movk	x0, #33792, lsl #16
ffffc02033d0: eb00003f 	cmp	x1, x0
ffffc02033d4: 54000880 	b.eq	0xffffc02034e4 <psci_dispatch+0x160>
ffffc02033d8: 54000128 	b.hi	0xffffc02033fc <psci_dispatch+0x78>
ffffc02033dc: d2b08000 	mov	x0, #2214592512
ffffc02033e0: eb00003f 	cmp	x1, x0
ffffc02033e4: 540006a0 	b.eq	0xffffc02034b8 <psci_dispatch+0x134>
ffffc02033e8: 91000400 	add	x0, x0, #1
ffffc02033ec: eb00003f 	cmp	x1, x0
ffffc02033f0: 540006e0 	b.eq	0xffffc02034cc <psci_dispatch+0x148>
ffffc02033f4: 92800013 	mov	x19, #-1
ffffc02033f8: 14000031 	b	0xffffc02034bc <psci_dispatch+0x138>
ffffc02033fc: d2800060 	mov	x0, #3
ffffc0203400: f2b08000 	movk	x0, #33792, lsl #16
ffffc0203404: eb00003f 	cmp	x1, x0
ffffc0203408: 540003c0 	b.eq	0xffffc0203480 <psci_dispatch+0xfc>
ffffc020340c: 91000400 	add	x0, x0, #1
ffffc0203410: eb00003f 	cmp	x1, x0
ffffc0203414: 54ffff01 	b.ne	0xffffc02033f4 <psci_dispatch+0x70>
ffffc0203418: f9480440 	ldr	x0, [x2, #4104]
ffffc020341c: f9400461 	ldr	x1, [x3, #8]
ffffc0203420: 97fffe78 	bl	0xffffc0202e00 <arm_cpu_by_mpidr>
ffffc0203424: 3100041f 	cmn	w0, #1
ffffc0203428: 540003e0 	b.eq	0xffffc02034a4 <psci_dispatch+0x120>
ffffc020342c: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc0203430: d00000c1 	adrp	x1, 0xffffc021d000 <psci_dispatch+0x114>
ffffc0203434: 91000021 	add	x1, x1, #0
ffffc0203438: 8b010000 	add	x0, x0, x1
ffffc020343c: 91400800 	add	x0, x0, #2, lsl #12     // =8192
ffffc0203440: b9547000 	ldr	w0, [x0, #5232]
ffffc0203444: 7100001f 	cmp	w0, #0
ffffc0203448: 9a9f07f3 	cset	x19, ne
ffffc020344c: 1400001c 	b	0xffffc02034bc <psci_dispatch+0x138>
ffffc0203450: d2800020 	mov	x0, #1
ffffc0203454: f2b88000 	movk	x0, #50176, lsl #16
ffffc0203458: eb00003f 	cmp	x1, x0
ffffc020345c: 54000380 	b.eq	0xffffc02034cc <psci_dispatch+0x148>
ffffc0203460: 54000268 	b.hi	0xffffc02034ac <psci_dispatch+0x128>
ffffc0203464: d2974be0 	mov	x0, #47711
ffffc0203468: f2b2b820 	movk	x0, #38337, lsl #16
ffffc020346c: eb00003f 	cmp	x1, x0
ffffc0203470: 540003a0 	b.eq	0xffffc02034e4 <psci_dispatch+0x160>
ffffc0203474: 91000400 	add	x0, x0, #1
ffffc0203478: eb00003f 	cmp	x1, x0
ffffc020347c: 54fffbc1 	b.ne	0xffffc02033f4 <psci_dispatch+0x70>
ffffc0203480: f9480440 	ldr	x0, [x2, #4104]
ffffc0203484: f262003f 	tst	x1, #0x40000000
ffffc0203488: f9400461 	ldr	x1, [x3, #8]
ffffc020348c: b2407ff3 	mov	x19, #4294967295
ffffc0203490: da9f0273 	csinv	x19, x19, xzr, eq
ffffc0203494: 8a010261 	and	x1, x19, x1
ffffc0203498: 97fffe5a 	bl	0xffffc0202e00 <arm_cpu_by_mpidr>
ffffc020349c: 3100041f 	cmn	w0, #1
ffffc02034a0: 54000261 	b.ne	0xffffc02034ec <psci_dispatch+0x168>
ffffc02034a4: 92800053 	mov	x19, #-3
ffffc02034a8: 14000005 	b	0xffffc02034bc <psci_dispatch+0x138>
ffffc02034ac: d2800060 	mov	x0, #3
ffffc02034b0: f2b88000 	movk	x0, #50176, lsl #16
ffffc02034b4: 17ffffd4 	b	0xffffc0203404 <psci_dispatch+0x80>
ffffc02034b8: 320083f3 	mov	w19, #65537
ffffc02034bc: aa1303e0 	mov	x0, x19
ffffc02034c0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02034c4: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc02034c8: d65f03c0 	ret
ffffc02034cc: 97fffc83 	bl	0xffffc02026d8 <irqchip_has_pending_irqs>
ffffc02034d0: 35000060 	cbnz	w0, 0xffffc02034dc <psci_dispatch+0x158>
ffffc02034d4: d503207f 	wfi
ffffc02034d8: 97fffbbf 	bl	0xffffc02023d4 <irqchip_handle_irq>
ffffc02034dc: d2800013 	mov	x19, #0
ffffc02034e0: 17fffff7 	b	0xffffc02034bc <psci_dispatch+0x138>
ffffc02034e4: 97fff4d4 	bl	0xffffc0200834 <arm_cpu_park>
ffffc02034e8: 17fffffd 	b	0xffffc02034dc <psci_dispatch+0x158>
ffffc02034ec: d3727c01 	ubfiz	x1, x0, #14, #32
ffffc02034f0: d00000c2 	adrp	x2, 0xffffc021d000 <psci_dispatch+0x1d4>
ffffc02034f4: 91000042 	add	x2, x2, #0
ffffc02034f8: 8b020021 	add	x1, x1, x2
ffffc02034fc: d2828d82 	mov	x2, #5228
ffffc0203500: 91400821 	add	x1, x1, #2, lsl #12     // =8192
ffffc0203504: 8b020022 	add	x2, x1, x2
ffffc0203508: f9800051 	prfm	pstl1strm, [x2]
ffffc020350c: 885ffc43 	ldaxr	w3, [x2]
ffffc0203510: 11404064 	add	w4, w3, #16, lsl #12    // =65536
ffffc0203514: 88057c44 	stxr	w5, w4, [x2]
ffffc0203518: 35ffffa5 	cbnz	w5, 0xffffc020350c <psci_dispatch+0x188>
ffffc020351c: 4ac34064 	eor	w4, w3, w3, ror #16
ffffc0203520: 340000c4 	cbz	w4, 0xffffc0203538 <psci_dispatch+0x1b4>
ffffc0203524: d50320bf 	sevl
ffffc0203528: d503205f 	wfe
ffffc020352c: 485ffc45 	ldaxrh	w5, [x2]
ffffc0203530: 4a4340a4 	eor	w4, w5, w3, lsr #16
ffffc0203534: 35ffffa4 	cbnz	w4, 0xffffc0203528 <psci_dispatch+0x1a4>
ffffc0203538: b9547023 	ldr	w3, [x1, #5232]
ffffc020353c: 34000243 	cbz	w3, 0xffffc0203584 <psci_dispatch+0x200>
ffffc0203540: f9400283 	ldr	x3, [x20]
ffffc0203544: f9400863 	ldr	x3, [x3, #16]
ffffc0203548: 8a130063 	and	x3, x3, x19
ffffc020354c: f90a4023 	str	x3, [x1, #5248]
ffffc0203550: f9400283 	ldr	x3, [x20]
ffffc0203554: f9400c63 	ldr	x3, [x3, #24]
ffffc0203558: 8a130073 	and	x19, x3, x19
ffffc020355c: 52800023 	mov	w3, #1
ffffc0203560: b9147423 	str	w3, [x1, #5236]
ffffc0203564: f90a4433 	str	x19, [x1, #5256]
ffffc0203568: d2800013 	mov	x19, #0
ffffc020356c: 7968d821 	ldrh	w1, [x1, #5228]
ffffc0203570: 11000421 	add	w1, w1, #1
ffffc0203574: 489ffc41 	stlrh	w1, [x2]
ffffc0203578: 34fffa23 	cbz	w3, 0xffffc02034bc <psci_dispatch+0x138>
ffffc020357c: 97fff4c4 	bl	0xffffc020088c <arm_cpu_kick>
ffffc0203580: 17ffffcf 	b	0xffffc02034bc <psci_dispatch+0x138>
ffffc0203584: 92800073 	mov	x19, #-4
ffffc0203588: 52800003 	mov	w3, #0
ffffc020358c: 17fffff8 	b	0xffffc020356c <psci_dispatch+0x1e8>
ffffc0203590: f9400460 	ldr	x0, [x3, #8]
ffffc0203594: eb01001f 	cmp	x0, x1
ffffc0203598: 54fffa20 	b.eq	0xffffc02034dc <psci_dispatch+0x158>
ffffc020359c: 54000168 	b.hi	0xffffc02035c8 <psci_dispatch+0x244>
ffffc02035a0: d2b00001 	mov	x1, #2147483648
ffffc02035a4: eb01001f 	cmp	x0, x1
ffffc02035a8: 54fff9a0 	b.eq	0xffffc02034dc <psci_dispatch+0x158>
ffffc02035ac: 54fff243 	b.lo	0xffffc02033f4 <psci_dispatch+0x70>
ffffc02035b0: 929fffe1 	mov	x1, #-65536
ffffc02035b4: f2af8001 	movk	x1, #31744, lsl #16
ffffc02035b8: 8b010000 	add	x0, x0, x1
ffffc02035bc: f100101f 	cmp	x0, #4
ffffc02035c0: 54fff8e9 	b.ls	0xffffc02034dc <psci_dispatch+0x158>
ffffc02035c4: 17ffff8c 	b	0xffffc02033f4 <psci_dispatch+0x70>
ffffc02035c8: d2800021 	mov	x1, #1
ffffc02035cc: f2b88001 	movk	x1, #50176, lsl #16
ffffc02035d0: eb01001f 	cmp	x0, x1
ffffc02035d4: 54fff840 	b.eq	0xffffc02034dc <psci_dispatch+0x158>
ffffc02035d8: 54fff0e3 	b.lo	0xffffc02033f4 <psci_dispatch+0x70>
ffffc02035dc: 92800041 	mov	x1, #-3
ffffc02035e0: f2a77fe1 	movk	x1, #15359, lsl #16
ffffc02035e4: 8b010000 	add	x0, x0, x1
ffffc02035e8: f100041f 	cmp	x0, #1
ffffc02035ec: 17fffff5 	b	0xffffc02035c0 <psci_dispatch+0x23c>

0000ffffc02035f0 <arm_init_early>:
ffffc02035f0: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc02035f4: d0000081 	adrp	x1, 0xffffc0215000 <arm_init_early+0x4c>
ffffc02035f8: b00000a0 	adrp	x0, 0xffffc0218000 <arm_init_early+0x5c>
ffffc02035fc: 91014000 	add	x0, x0, #80
ffffc0203600: 910003fd 	mov	x29, sp
ffffc0203604: f9407c21 	ldr	x1, [x1, #248]
ffffc0203608: 52800005 	mov	w5, #0
ffffc020360c: f9000401 	str	x1, [x0, #8]
ffffc0203610: d2808fa4 	mov	x4, #1149
ffffc0203614: b00000a1 	adrp	x1, 0xffffc0218000 <arm_cpu_init+0x18>
ffffc0203618: d2800003 	mov	x3, #0
ffffc020361c: d2820002 	mov	x2, #4096
ffffc0203620: f9400426 	ldr	x6, [x1, #8]
ffffc0203624: d0000041 	adrp	x1, 0xffffc020d000 <arm_init_early+0x5c>
ffffc0203628: 91000021 	add	x1, x1, #0
ffffc020362c: cb060021 	sub	x1, x1, x6
ffffc0203630: 94000ed6 	bl	0xffffc0207188 <paging_create>
ffffc0203634: 350000a0 	cbnz	w0, 0xffffc0203648 <arm_init_early+0x58>
ffffc0203638: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc020363c: f00000a0 	adrp	x0, 0xffffc021a000 <handle_smc+0x14>
ffffc0203640: 91000000 	add	x0, x0, #0
ffffc0203644: 17fffe66 	b	0xffffc0202fdc <arm_paging_cell_init>
ffffc0203648: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc020364c: d65f03c0 	ret

0000ffffc0203650 <arm_cpu_init>:
ffffc0203650: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0203654: 910003fd 	mov	x29, sp
ffffc0203658: f9000bf3 	str	x19, [sp, #16]
ffffc020365c: aa0003f3 	mov	x19, x0
ffffc0203660: 97fffde4 	bl	0xffffc0202df0 <phys_processor_id>
ffffc0203664: f9182660 	str	x0, [x19, #12360]
ffffc0203668: 900000c0 	adrp	x0, 0xffffc021b000 <handle_smc+0x44>
ffffc020366c: 91000000 	add	x0, x0, #0
ffffc0203670: 97fffe86 	bl	0xffffc0203088 <arm_paging_vcpu_init>
ffffc0203674: aa1303e0 	mov	x0, x19
ffffc0203678: f9400bf3 	ldr	x19, [sp, #16]
ffffc020367c: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0203680: 17fffc42 	b	0xffffc0202788 <irqchip_cpu_init>

0000ffffc0203684 <handle_smc>:
ffffc0203684: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0203688: 910003fd 	mov	x29, sp
ffffc020368c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0203690: aa0003f4 	mov	x20, x0
ffffc0203694: f9400013 	ldr	x19, [x0]
ffffc0203698: f9400262 	ldr	x2, [x19]
ffffc020369c: d3587441 	ubfx	x1, x2, #24, #6
ffffc02036a0: f100083f 	cmp	x1, #2
ffffc02036a4: 540001e0 	b.eq	0xffffc02036e0 <handle_smc+0x5c>
ffffc02036a8: f100103f 	cmp	x1, #4
ffffc02036ac: 540001e0 	b.eq	0xffffc02036e8 <handle_smc+0x64>
ffffc02036b0: b5000201 	cbnz	x1, 0xffffc02036f0 <handle_smc+0x6c>
ffffc02036b4: 52b00000 	mov	w0, #-2147483648
ffffc02036b8: 6b02001f 	cmp	w0, w2
ffffc02036bc: 320083e0 	mov	w0, #65537
ffffc02036c0: da9f0000 	csinv	x0, x0, xzr, eq
ffffc02036c4: f9000260 	str	x0, [x19]
ffffc02036c8: aa1403e0 	mov	x0, x20
ffffc02036cc: 940009e9 	bl	0xffffc0205e70 <arch_skip_instruction>
ffffc02036d0: 52800020 	mov	w0, #1
ffffc02036d4: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02036d8: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc02036dc: d65f03c0 	ret
ffffc02036e0: 92800000 	mov	x0, #-1
ffffc02036e4: 17fffff8 	b	0xffffc02036c4 <handle_smc+0x40>
ffffc02036e8: 97ffff27 	bl	0xffffc0203384 <psci_dispatch>
ffffc02036ec: 17fffff6 	b	0xffffc02036c4 <handle_smc+0x40>
ffffc02036f0: 52800000 	mov	w0, #0
ffffc02036f4: 17fffff8 	b	0xffffc02036d4 <handle_smc+0x50>

0000ffffc02036f8 <uart_is_busy>:
ffffc02036f8: f9400000 	ldr	x0, [x0]
ffffc02036fc: 79402000 	ldrh	w0, [x0, #16]
ffffc0203700: d27b0000 	eor	x0, x0, #0x20
ffffc0203704: 53051400 	ubfx	w0, w0, #5, #1
ffffc0203708: d65f03c0 	ret

0000ffffc020370c <uart_write_char>:
ffffc020370c: f9400002 	ldr	x2, [x0]
ffffc0203710: 12001c21 	and	w1, w1, #0xff
ffffc0203714: 39003041 	strb	w1, [x2, #12]
ffffc0203718: 529ff3e2 	mov	w2, #65439
ffffc020371c: f9400001 	ldr	x1, [x0]
ffffc0203720: 79402020 	ldrh	w0, [x1, #16]
ffffc0203724: 0a020000 	and	w0, w0, w2
ffffc0203728: 79002020 	strh	w0, [x1, #16]
ffffc020372c: d65f03c0 	ret

0000ffffc0203730 <uart_init>:
ffffc0203730: f9400401 	ldr	x1, [x0, #8]
ffffc0203734: b9401021 	ldr	w1, [x1, #16]
ffffc0203738: 34000201 	cbz	w1, 0xffffc0203778 <uart_init+0x48>
ffffc020373c: f9400003 	ldr	x3, [x0]
ffffc0203740: 79401061 	ldrh	w1, [x3, #8]
ffffc0203744: 12003c21 	and	w1, w1, #0xffff
ffffc0203748: 121a7422 	and	w2, w1, #0xffffffcf
ffffc020374c: 321b0021 	orr	w1, w1, #0x20
ffffc0203750: 12003c42 	and	w2, w2, #0xffff
ffffc0203754: 79001062 	strh	w2, [x3, #8]
ffffc0203758: a9400c02 	ldp	x2, x3, [x0]
ffffc020375c: 39404063 	ldrb	w3, [x3, #16]
ffffc0203760: 39001043 	strb	w3, [x2, #4]
ffffc0203764: 52800803 	mov	w3, #64
ffffc0203768: f9400002 	ldr	x2, [x0]
ffffc020376c: 7900b043 	strh	w3, [x2, #88]
ffffc0203770: f9400000 	ldr	x0, [x0]
ffffc0203774: 79001001 	strh	w1, [x0, #8]
ffffc0203778: d65f03c0 	ret

0000ffffc020377c <uart_init>:
ffffc020377c: d65f03c0 	ret

0000ffffc0203780 <uart_is_busy>:
ffffc0203780: f9400000 	ldr	x0, [x0]
ffffc0203784: b940b400 	ldr	w0, [x0, #180]
ffffc0203788: d27a0000 	eor	x0, x0, #0x40
ffffc020378c: 53061800 	ubfx	w0, w0, #6, #1
ffffc0203790: d65f03c0 	ret

0000ffffc0203794 <uart_write_char>:
ffffc0203794: f9400000 	ldr	x0, [x0]
ffffc0203798: 12001c21 	and	w1, w1, #0xff
ffffc020379c: b9004001 	str	w1, [x0, #64]
ffffc02037a0: d65f03c0 	ret

0000ffffc02037a4 <uart_init>:
ffffc02037a4: d65f03c0 	ret

0000ffffc02037a8 <uart_is_busy>:
ffffc02037a8: f9400000 	ldr	x0, [x0]
ffffc02037ac: b9400c00 	ldr	w0, [x0, #12]
ffffc02037b0: d34b2c00 	ubfx	x0, x0, #11, #1
ffffc02037b4: d65f03c0 	ret

0000ffffc02037b8 <uart_write_char>:
ffffc02037b8: f9400000 	ldr	x0, [x0]
ffffc02037bc: 12001c21 	and	w1, w1, #0xff
ffffc02037c0: b9000401 	str	w1, [x0, #4]
ffffc02037c4: d65f03c0 	ret

0000ffffc02037c8 <uart_init>:
ffffc02037c8: f9400401 	ldr	x1, [x0, #8]
ffffc02037cc: b9401021 	ldr	w1, [x1, #16]
ffffc02037d0: 34000161 	cbz	w1, 0xffffc02037fc <uart_init+0x34>
ffffc02037d4: f9400000 	ldr	x0, [x0]
ffffc02037d8: 7900601f 	strh	wzr, [x0, #48]
ffffc02037dc: 39406002 	ldrb	w2, [x0, #24]
ffffc02037e0: 37180102 	tbnz	w2, #3, 0xffffc0203800 <uart_init+0x38>
ffffc02037e4: 12003c21 	and	w1, w1, #0xffff
ffffc02037e8: 79004801 	strh	w1, [x0, #36]
ffffc02037ec: 52800c01 	mov	w1, #96
ffffc02037f0: 3900b001 	strb	w1, [x0, #44]
ffffc02037f4: 52862021 	mov	w1, #12545
ffffc02037f8: 79006001 	strh	w1, [x0, #48]
ffffc02037fc: d65f03c0 	ret
ffffc0203800: 17fffff7 	b	0xffffc02037dc <uart_init+0x14>

0000ffffc0203804 <uart_is_busy>:
ffffc0203804: f9400000 	ldr	x0, [x0]
ffffc0203808: b9401801 	ldr	w1, [x0, #24]
ffffc020380c: 52800500 	mov	w0, #40
ffffc0203810: 6a00003f 	tst	w1, w0
ffffc0203814: 1a9f07e0 	cset	w0, ne
ffffc0203818: d65f03c0 	ret

0000ffffc020381c <uart_write_char>:
ffffc020381c: f9400000 	ldr	x0, [x0]
ffffc0203820: 12001c21 	and	w1, w1, #0xff
ffffc0203824: b9000001 	str	w1, [x0]
ffffc0203828: d65f03c0 	ret

0000ffffc020382c <uart_is_busy>:
ffffc020382c: f9400000 	ldr	x0, [x0]
ffffc0203830: 79402800 	ldrh	w0, [x0, #20]
ffffc0203834: d27b0000 	eor	x0, x0, #0x20
ffffc0203838: 53051400 	ubfx	w0, w0, #5, #1
ffffc020383c: d65f03c0 	ret

0000ffffc0203840 <uart_write_char>:
ffffc0203840: f9400002 	ldr	x2, [x0]
ffffc0203844: 12001c21 	and	w1, w1, #0xff
ffffc0203848: 39008041 	strb	w1, [x2, #32]
ffffc020384c: 529ff3e2 	mov	w2, #65439
ffffc0203850: f9400001 	ldr	x1, [x0]
ffffc0203854: 79402820 	ldrh	w0, [x1, #20]
ffffc0203858: 0a020000 	and	w0, w0, w2
ffffc020385c: 79002820 	strh	w0, [x1, #20]
ffffc0203860: d65f03c0 	ret

0000ffffc0203864 <uart_init>:
ffffc0203864: f9400401 	ldr	x1, [x0, #8]
ffffc0203868: b9401021 	ldr	w1, [x1, #16]
ffffc020386c: 340001e1 	cbz	w1, 0xffffc02038a8 <uart_init+0x44>
ffffc0203870: f9400003 	ldr	x3, [x0]
ffffc0203874: 79401061 	ldrh	w1, [x3, #8]
ffffc0203878: 12003c21 	and	w1, w1, #0xffff
ffffc020387c: 121a7422 	and	w2, w1, #0xffffffcf
ffffc0203880: 321b0021 	orr	w1, w1, #0x20
ffffc0203884: 12003c42 	and	w2, w2, #0xffff
ffffc0203888: 79001062 	strh	w2, [x3, #8]
ffffc020388c: a9400c02 	ldp	x2, x3, [x0]
ffffc0203890: 39404063 	ldrb	w3, [x3, #16]
ffffc0203894: 39001043 	strb	w3, [x2, #4]
ffffc0203898: f9400002 	ldr	x2, [x0]
ffffc020389c: 7900305f 	strh	wzr, [x2, #24]
ffffc02038a0: f9400000 	ldr	x0, [x0]
ffffc02038a4: 79001001 	strh	w1, [x0, #8]
ffffc02038a8: d65f03c0 	ret

0000ffffc02038ac <uart_init>:
ffffc02038ac: d65f03c0 	ret

0000ffffc02038b0 <uart_is_busy>:
ffffc02038b0: f9400000 	ldr	x0, [x0]
ffffc02038b4: b9402c00 	ldr	w0, [x0, #44]
ffffc02038b8: d27d0000 	eor	x0, x0, #0x8
ffffc02038bc: 53030c00 	ubfx	w0, w0, #3, #1
ffffc02038c0: d65f03c0 	ret

0000ffffc02038c4 <uart_write_char>:
ffffc02038c4: f9400000 	ldr	x0, [x0]
ffffc02038c8: 12001c21 	and	w1, w1, #0xff
ffffc02038cc: b9003001 	str	w1, [x0, #48]
ffffc02038d0: d65f03c0 	ret

0000ffffc02038d4 <arm_dcaches_flush>:
ffffc02038d4: d53b0024 	mrs	x4, CTR_EL0
ffffc02038d8: d3504c84 	ubfx	x4, x4, #16, #4
ffffc02038dc: d2800083 	mov	x3, #4
ffffc02038e0: 9ac42063 	lsl	x3, x3, x4
ffffc02038e4: 8b010001 	add	x1, x0, x1
ffffc02038e8: d1000464 	sub	x4, x3, #1
ffffc02038ec: 8a240000 	bic	x0, x0, x4
ffffc02038f0: f100005f 	cmp	x2, #0
ffffc02038f4: 54000061 	b.ne	0xffffc0203900 <arm_dcaches_flush+0x2c>
ffffc02038f8: d50b7a20 	dc	cvac, x0
ffffc02038fc: 14000006 	b	0xffffc0203914 <arm_dcaches_flush+0x40>
ffffc0203900: f100045f 	cmp	x2, #1
ffffc0203904: 54000061 	b.ne	0xffffc0203910 <arm_dcaches_flush+0x3c>
ffffc0203908: d5087620 	dc	ivac, x0
ffffc020390c: 14000002 	b	0xffffc0203914 <arm_dcaches_flush+0x40>
ffffc0203910: d50b7e20 	dc	civac, x0
ffffc0203914: 8b030000 	add	x0, x0, x3
ffffc0203918: eb01001f 	cmp	x0, x1
ffffc020391c: 54fffea3 	b.lo	0xffffc02038f0 <arm_dcaches_flush+0x1c>
ffffc0203920: d5033f9f 	dsb	sy
ffffc0203924: d65f03c0 	ret

0000ffffc0203928 <arm_cpu_reset>:
ffffc0203928: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc020392c: 910003fd 	mov	x29, sp
ffffc0203930: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0203934: aa0003f4 	mov	x20, x0
ffffc0203938: d28078a0 	mov	x0, #965
ffffc020393c: d51c4000 	msr	SPSR_EL2, x0
ffffc0203940: d2810000 	mov	x0, #2048
ffffc0203944: f2a61a00 	movk	x0, #12496, lsl #16
ffffc0203948: d5181000 	msr	SCTLR_EL1, x0
ffffc020394c: d2800013 	mov	x19, #0
ffffc0203950: d518e113 	msr	CNTKCTL_EL1, x19
ffffc0203954: d51b9c13 	msr	PMCR_EL0, x19
ffffc0203958: d281e000 	mov	x0, #3840
ffffc020395c: d2802002 	mov	x2, #256
ffffc0203960: f2a00020 	movk	x0, #1, lsl #16
ffffc0203964: 52800001 	mov	w1, #0
ffffc0203968: f2dfe000 	movk	x0, #65280, lsl #32
ffffc020396c: 94001533 	bl	0xffffc0208e38 <memset>
ffffc0203970: d5184113 	msr	SP_EL0, x19
ffffc0203974: d51c4113 	msr	SP_EL1, x19
ffffc0203978: d5184013 	msr	SPSR_EL1, x19
ffffc020397c: d5185113 	msr	AFSR0_EL1, x19
ffffc0203980: d5185133 	msr	AFSR1_EL1, x19
ffffc0203984: d518a313 	msr	AMAIR_EL1, x19
ffffc0203988: d518d033 	msr	CONTEXTIDR_EL1, x19
ffffc020398c: d5181053 	msr	CPACR_EL1, x19
ffffc0203990: d51a0013 	msr	CSSELR_EL1, x19
ffffc0203994: d5185213 	msr	ESR_EL1, x19
ffffc0203998: d5186013 	msr	FAR_EL1, x19
ffffc020399c: d518a213 	msr	MAIR_EL1, x19
ffffc02039a0: d5187413 	msr	PAR_EL1, x19
ffffc02039a4: d5182053 	msr	TCR_EL1, x19
ffffc02039a8: d51bd073 	msr	TPIDRRO_EL0, x19
ffffc02039ac: d51bd053 	msr	TPIDR_EL0, x19
ffffc02039b0: d518d093 	msr	TPIDR_EL1, x19
ffffc02039b4: d5182013 	msr	TTBR0_EL1, x19
ffffc02039b8: d5182033 	msr	TTBR1_EL1, x19
ffffc02039bc: d518c013 	msr	VBAR_EL1, x19
ffffc02039c0: d51be233 	msr	CNTP_CTL_EL0, x19
ffffc02039c4: d51be253 	msr	CNTP_CVAL_EL0, x19
ffffc02039c8: d51be213 	msr	CNTP_TVAL_EL0, x19
ffffc02039cc: d51be333 	msr	CNTV_CTL_EL0, x19
ffffc02039d0: d51be353 	msr	CNTV_CVAL_EL0, x19
ffffc02039d4: d51be313 	msr	CNTV_TVAL_EL0, x19
ffffc02039d8: d51c4034 	msr	ELR_EL2, x20
ffffc02039dc: d2a00033 	mov	x19, #65536
ffffc02039e0: f2dfe013 	movk	x19, #65280, lsl #32
ffffc02039e4: f95a4661 	ldr	x1, [x19, #13448]
ffffc02039e8: f9580660 	ldr	x0, [x19, #12296]
ffffc02039ec: f9078a61 	str	x1, [x19, #3856]
ffffc02039f0: 91400400 	add	x0, x0, #1, lsl #12     // =4096
ffffc02039f4: 97fffda5 	bl	0xffffc0203088 <arm_paging_vcpu_init>
ffffc02039f8: aa1303e0 	mov	x0, x19
ffffc02039fc: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0203a00: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0203a04: 17fffc1e 	b	0xffffc0202a7c <irqchip_cpu_reset>
		...

0000ffffc0204000 <arch_entry>:
ffffc0204000: aa0003f0 	mov	x16, x0
ffffc0204004: aa1e03f1 	mov	x17, x30
ffffc0204008: 10fdffc0 	adr	x0, #-16392
ffffc020400c: f9402412 	ldr	x18, [x0, #72]
ffffc0204010: 5800bbcf 	ldr	x15, 0xffffc0205788 <$d>
ffffc0204014: b00000c1 	adrp	x1, 0xffffc021d000 <el2_entry+0x8>
ffffc0204018: 79406002 	ldrh	w2, [x0, #48]
ffffc020401c: d2880003 	mov	x3, #16384
ffffc0204020: 9b030441 	madd	x1, x2, x3, x1
ffffc0204024: f842c02e 	ldur	x14, [x1, #44]
ffffc0204028: 5800bb4d 	ldr	x13, 0xffffc0205790 <$d+0x8>
ffffc020402c: f840c02c 	ldur	x12, [x1, #12]
ffffc0204030: cb0d018b 	sub	x11, x12, x13
ffffc0204034: 924051c1 	and	x1, x14, #0x1fffff
ffffc0204038: 8b0101ef 	add	x15, x15, x1
ffffc020403c: f9001c0f 	str	x15, [x0, #56]
ffffc0204040: f9400401 	ldr	x1, [x0, #8]
ffffc0204044: d2800042 	mov	x2, #2
ffffc0204048: 97fffe23 	bl	0xffffc02038d4 <arm_dcaches_flush>
ffffc020404c: 5800ba61 	ldr	x1, 0xffffc0205798 <$d+0x10>
ffffc0204050: 8b0b0021 	add	x1, x1, x11
ffffc0204054: d2800000 	mov	x0, #0
ffffc0204058: f100025f 	cmp	x18, #0
ffffc020405c: 54000041 	b.ne	0xffffc0204064 <arch_entry+0x64>
ffffc0204060: d2800020 	mov	x0, #1
ffffc0204064: d4000002 	hvc	#0
ffffc0204068: d4000002 	hvc	#0
ffffc020406c: 14000000 	b	0xffffc020406c <arch_entry+0x6c>

0000ffffc0204070 <el2_entry>:
ffffc0204070: d53c5201 	mrs	x1, ESR_EL2
ffffc0204074: d35afc21 	lsr	x1, x1, #26
ffffc0204078: f100583f 	cmp	x1, #22
ffffc020407c: 54000001 	b.ne	0xffffc020407c <el2_entry+0xc>
ffffc0204080: 94000019 	bl	0xffffc02040e4 <init_bootstrap_pt>
ffffc0204084: 10057be0 	adr	x0, #44924
ffffc0204088: 1000007e 	adr	x30, #12
ffffc020408c: cb0b03de 	sub	x30, x30, x11
ffffc0204090: 14003bdc 	b	0xffffc0213000 <enable_mmu_el2>
ffffc0204094: 10007b61 	adr	x1, #3948
ffffc0204098: d51cc001 	msr	VBAR_EL2, x1
ffffc020409c: aa1003e0 	mov	x0, x16
ffffc02040a0: b00000c1 	adrp	x1, 0xffffc021d000 <init_bootstrap_pt+0x20>
ffffc02040a4: d2880002 	mov	x2, #16384
ffffc02040a8: 9b000441 	madd	x1, x2, x0, x1
ffffc02040ac: 9140043f 	add	sp, x1, #1, lsl #12     // =4096
ffffc02040b0: a9bf47fd 	stp	x29, x17, [sp, #-16]!
ffffc02040b4: a9bf73fb 	stp	x27, x28, [sp, #-16]!
ffffc02040b8: a9bf6bf9 	stp	x25, x26, [sp, #-16]!
ffffc02040bc: a9bf63f7 	stp	x23, x24, [sp, #-16]!
ffffc02040c0: a9bf5bf5 	stp	x21, x22, [sp, #-16]!
ffffc02040c4: a9bf53f3 	stp	x19, x20, [sp, #-16]!
ffffc02040c8: d10283ff 	sub	sp, sp, #160
ffffc02040cc: d538071d 	mrs	x29, ID_AA64MMFR0_EL1
ffffc02040d0: f9080c3d 	str	x29, [x1, #4120]
ffffc02040d4: aa1f03fd 	mov	x29, xzr
ffffc02040d8: aa1f03fe 	mov	x30, xzr
ffffc02040dc: 940007da 	bl	0xffffc0206044 <entry>
ffffc02040e0: 14000000 	b	0xffffc02040e0 <el2_entry+0x70>

0000ffffc02040e4 <init_bootstrap_pt>:
ffffc02040e4: f0000060 	adrp	x0, 0xffffc0213000 <init_bootstrap_pt+0x3c>
ffffc02040e8: d367bda2 	ubfx	x2, x13, #39, #9
ffffc02040ec: 1005f8a8 	adr	x8, #48916
ffffc02040f0: 91000d08 	add	x8, x8, #3
ffffc02040f4: 10057869 	adr	x9, #44812
ffffc02040f8: 8b020d29 	add	x9, x9, x2, lsl #3
ffffc02040fc: f9000128 	str	x8, [x9]
ffffc0204100: d367bc03 	ubfx	x3, x0, #39, #9
ffffc0204104: 100677e8 	adr	x8, #52988
ffffc0204108: 91000d08 	add	x8, x8, #3
ffffc020410c: 100577a9 	adr	x9, #44788
ffffc0204110: 8b030d29 	add	x9, x9, x3, lsl #3
ffffc0204114: f9000128 	str	x8, [x9]
ffffc0204118: d35e99a2 	ubfx	x2, x13, #30, #9
ffffc020411c: 1006f728 	adr	x8, #57060
ffffc0204120: 91000d08 	add	x8, x8, #3
ffffc0204124: 1005f6e9 	adr	x9, #48860
ffffc0204128: 8b020d29 	add	x9, x9, x2, lsl #3
ffffc020412c: f9000128 	str	x8, [x9]
ffffc0204130: d35e9804 	ubfx	x4, x0, #30, #9
ffffc0204134: 92624408 	and	x8, x0, #0xffffc0000000
ffffc0204138: 911d0508 	add	x8, x8, #1857
ffffc020413c: 10067629 	adr	x9, #52932
ffffc0204140: 8b040d29 	add	x9, x9, x4, lsl #3
ffffc0204144: f9000128 	str	x8, [x9]
ffffc0204148: d35575a2 	ubfx	x2, x13, #21, #9
ffffc020414c: 926b6988 	and	x8, x12, #0xffffffe00000
ffffc0204150: 911d0508 	add	x8, x8, #1857
ffffc0204154: 1006f569 	adr	x9, #57004
ffffc0204158: 8b020d29 	add	x9, x9, x2, lsl #3
ffffc020415c: f9000128 	str	x8, [x9]
ffffc0204160: d35575e3 	ubfx	x3, x15, #21, #9
ffffc0204164: 926b69c8 	and	x8, x14, #0xffffffe00000
ffffc0204168: 911d1508 	add	x8, x8, #1861
ffffc020416c: 1006f4a9 	adr	x9, #56980
ffffc0204170: 8b030d29 	add	x9, x9, x3, lsl #3
ffffc0204174: f9000128 	str	x8, [x9]
ffffc0204178: f0000040 	adrp	x0, 0xffffc020f000 <init_bootstrap_pt+0xc0>
ffffc020417c: d2880001 	mov	x1, #16384
ffffc0204180: d2800022 	mov	x2, #1
ffffc0204184: 17fffdd4 	b	0xffffc02038d4 <arm_dcaches_flush>
ffffc0204188: d503201f 	nop
ffffc020418c: d503201f 	nop
ffffc0204190: d503201f 	nop
ffffc0204194: d503201f 	nop
ffffc0204198: d503201f 	nop
ffffc020419c: d503201f 	nop
ffffc02041a0: d503201f 	nop
ffffc02041a4: d503201f 	nop
ffffc02041a8: d503201f 	nop
ffffc02041ac: d503201f 	nop
ffffc02041b0: d503201f 	nop
ffffc02041b4: d503201f 	nop
ffffc02041b8: d503201f 	nop
ffffc02041bc: d503201f 	nop
ffffc02041c0: d503201f 	nop
ffffc02041c4: d503201f 	nop
ffffc02041c8: d503201f 	nop
ffffc02041cc: d503201f 	nop
ffffc02041d0: d503201f 	nop
ffffc02041d4: d503201f 	nop
ffffc02041d8: d503201f 	nop
ffffc02041dc: d503201f 	nop
ffffc02041e0: d503201f 	nop
ffffc02041e4: d503201f 	nop
ffffc02041e8: d503201f 	nop
ffffc02041ec: d503201f 	nop
ffffc02041f0: d503201f 	nop
ffffc02041f4: d503201f 	nop
ffffc02041f8: d503201f 	nop
ffffc02041fc: d503201f 	nop
ffffc0204200: d503201f 	nop
ffffc0204204: d503201f 	nop
ffffc0204208: d503201f 	nop
ffffc020420c: d503201f 	nop
ffffc0204210: d503201f 	nop
ffffc0204214: d503201f 	nop
ffffc0204218: d503201f 	nop
ffffc020421c: d503201f 	nop
ffffc0204220: d503201f 	nop
ffffc0204224: d503201f 	nop
ffffc0204228: d503201f 	nop
ffffc020422c: d503201f 	nop
ffffc0204230: d503201f 	nop
ffffc0204234: d503201f 	nop
ffffc0204238: d503201f 	nop
ffffc020423c: d503201f 	nop
ffffc0204240: d503201f 	nop
ffffc0204244: d503201f 	nop
ffffc0204248: d503201f 	nop
ffffc020424c: d503201f 	nop
ffffc0204250: d503201f 	nop
ffffc0204254: d503201f 	nop
ffffc0204258: d503201f 	nop
ffffc020425c: d503201f 	nop
ffffc0204260: d503201f 	nop
ffffc0204264: d503201f 	nop
ffffc0204268: d503201f 	nop
ffffc020426c: d503201f 	nop
ffffc0204270: d503201f 	nop
ffffc0204274: d503201f 	nop
ffffc0204278: d503201f 	nop
ffffc020427c: d503201f 	nop
ffffc0204280: d503201f 	nop
ffffc0204284: d503201f 	nop
ffffc0204288: d503201f 	nop
ffffc020428c: d503201f 	nop
ffffc0204290: d503201f 	nop
ffffc0204294: d503201f 	nop
ffffc0204298: d503201f 	nop
ffffc020429c: d503201f 	nop
ffffc02042a0: d503201f 	nop
ffffc02042a4: d503201f 	nop
ffffc02042a8: d503201f 	nop
ffffc02042ac: d503201f 	nop
ffffc02042b0: d503201f 	nop
ffffc02042b4: d503201f 	nop
ffffc02042b8: d503201f 	nop
ffffc02042bc: d503201f 	nop
ffffc02042c0: d503201f 	nop
ffffc02042c4: d503201f 	nop
ffffc02042c8: d503201f 	nop
ffffc02042cc: d503201f 	nop
ffffc02042d0: d503201f 	nop
ffffc02042d4: d503201f 	nop
ffffc02042d8: d503201f 	nop
ffffc02042dc: d503201f 	nop
ffffc02042e0: d503201f 	nop
ffffc02042e4: d503201f 	nop
ffffc02042e8: d503201f 	nop
ffffc02042ec: d503201f 	nop
ffffc02042f0: d503201f 	nop
ffffc02042f4: d503201f 	nop
ffffc02042f8: d503201f 	nop
ffffc02042fc: d503201f 	nop
ffffc0204300: d503201f 	nop
ffffc0204304: d503201f 	nop
ffffc0204308: d503201f 	nop
ffffc020430c: d503201f 	nop
ffffc0204310: d503201f 	nop
ffffc0204314: d503201f 	nop
ffffc0204318: d503201f 	nop
ffffc020431c: d503201f 	nop
ffffc0204320: d503201f 	nop
ffffc0204324: d503201f 	nop
ffffc0204328: d503201f 	nop
ffffc020432c: d503201f 	nop
ffffc0204330: d503201f 	nop
ffffc0204334: d503201f 	nop
ffffc0204338: d503201f 	nop
ffffc020433c: d503201f 	nop
ffffc0204340: d503201f 	nop
ffffc0204344: d503201f 	nop
ffffc0204348: d503201f 	nop
ffffc020434c: d503201f 	nop
ffffc0204350: d503201f 	nop
ffffc0204354: d503201f 	nop
ffffc0204358: d503201f 	nop
ffffc020435c: d503201f 	nop
ffffc0204360: d503201f 	nop
ffffc0204364: d503201f 	nop
ffffc0204368: d503201f 	nop
ffffc020436c: d503201f 	nop
ffffc0204370: d503201f 	nop
ffffc0204374: d503201f 	nop
ffffc0204378: d503201f 	nop
ffffc020437c: d503201f 	nop
ffffc0204380: d503201f 	nop
ffffc0204384: d503201f 	nop
ffffc0204388: d503201f 	nop
ffffc020438c: d503201f 	nop
ffffc0204390: d503201f 	nop
ffffc0204394: d503201f 	nop
ffffc0204398: d503201f 	nop
ffffc020439c: d503201f 	nop
ffffc02043a0: d503201f 	nop
ffffc02043a4: d503201f 	nop
ffffc02043a8: d503201f 	nop
ffffc02043ac: d503201f 	nop
ffffc02043b0: d503201f 	nop
ffffc02043b4: d503201f 	nop
ffffc02043b8: d503201f 	nop
ffffc02043bc: d503201f 	nop
ffffc02043c0: d503201f 	nop
ffffc02043c4: d503201f 	nop
ffffc02043c8: d503201f 	nop
ffffc02043cc: d503201f 	nop
ffffc02043d0: d503201f 	nop
ffffc02043d4: d503201f 	nop
ffffc02043d8: d503201f 	nop
ffffc02043dc: d503201f 	nop
ffffc02043e0: d503201f 	nop
ffffc02043e4: d503201f 	nop
ffffc02043e8: d503201f 	nop
ffffc02043ec: d503201f 	nop
ffffc02043f0: d503201f 	nop
ffffc02043f4: d503201f 	nop
ffffc02043f8: d503201f 	nop
ffffc02043fc: d503201f 	nop
ffffc0204400: d503201f 	nop
ffffc0204404: d503201f 	nop
ffffc0204408: d503201f 	nop
ffffc020440c: d503201f 	nop
ffffc0204410: d503201f 	nop
ffffc0204414: d503201f 	nop
ffffc0204418: d503201f 	nop
ffffc020441c: d503201f 	nop
ffffc0204420: d503201f 	nop
ffffc0204424: d503201f 	nop
ffffc0204428: d503201f 	nop
ffffc020442c: d503201f 	nop
ffffc0204430: d503201f 	nop
ffffc0204434: d503201f 	nop
ffffc0204438: d503201f 	nop
ffffc020443c: d503201f 	nop
ffffc0204440: d503201f 	nop
ffffc0204444: d503201f 	nop
ffffc0204448: d503201f 	nop
ffffc020444c: d503201f 	nop
ffffc0204450: d503201f 	nop
ffffc0204454: d503201f 	nop
ffffc0204458: d503201f 	nop
ffffc020445c: d503201f 	nop
ffffc0204460: d503201f 	nop
ffffc0204464: d503201f 	nop
ffffc0204468: d503201f 	nop
ffffc020446c: d503201f 	nop
ffffc0204470: d503201f 	nop
ffffc0204474: d503201f 	nop
ffffc0204478: d503201f 	nop
ffffc020447c: d503201f 	nop
ffffc0204480: d503201f 	nop
ffffc0204484: d503201f 	nop
ffffc0204488: d503201f 	nop
ffffc020448c: d503201f 	nop
ffffc0204490: d503201f 	nop
ffffc0204494: d503201f 	nop
ffffc0204498: d503201f 	nop
ffffc020449c: d503201f 	nop
ffffc02044a0: d503201f 	nop
ffffc02044a4: d503201f 	nop
ffffc02044a8: d503201f 	nop
ffffc02044ac: d503201f 	nop
ffffc02044b0: d503201f 	nop
ffffc02044b4: d503201f 	nop
ffffc02044b8: d503201f 	nop
ffffc02044bc: d503201f 	nop
ffffc02044c0: d503201f 	nop
ffffc02044c4: d503201f 	nop
ffffc02044c8: d503201f 	nop
ffffc02044cc: d503201f 	nop
ffffc02044d0: d503201f 	nop
ffffc02044d4: d503201f 	nop
ffffc02044d8: d503201f 	nop
ffffc02044dc: d503201f 	nop
ffffc02044e0: d503201f 	nop
ffffc02044e4: d503201f 	nop
ffffc02044e8: d503201f 	nop
ffffc02044ec: d503201f 	nop
ffffc02044f0: d503201f 	nop
ffffc02044f4: d503201f 	nop
ffffc02044f8: d503201f 	nop
ffffc02044fc: d503201f 	nop
ffffc0204500: d503201f 	nop
ffffc0204504: d503201f 	nop
ffffc0204508: d503201f 	nop
ffffc020450c: d503201f 	nop
ffffc0204510: d503201f 	nop
ffffc0204514: d503201f 	nop
ffffc0204518: d503201f 	nop
ffffc020451c: d503201f 	nop
ffffc0204520: d503201f 	nop
ffffc0204524: d503201f 	nop
ffffc0204528: d503201f 	nop
ffffc020452c: d503201f 	nop
ffffc0204530: d503201f 	nop
ffffc0204534: d503201f 	nop
ffffc0204538: d503201f 	nop
ffffc020453c: d503201f 	nop
ffffc0204540: d503201f 	nop
ffffc0204544: d503201f 	nop
ffffc0204548: d503201f 	nop
ffffc020454c: d503201f 	nop
ffffc0204550: d503201f 	nop
ffffc0204554: d503201f 	nop
ffffc0204558: d503201f 	nop
ffffc020455c: d503201f 	nop
ffffc0204560: d503201f 	nop
ffffc0204564: d503201f 	nop
ffffc0204568: d503201f 	nop
ffffc020456c: d503201f 	nop
ffffc0204570: d503201f 	nop
ffffc0204574: d503201f 	nop
ffffc0204578: d503201f 	nop
ffffc020457c: d503201f 	nop
ffffc0204580: d503201f 	nop
ffffc0204584: d503201f 	nop
ffffc0204588: d503201f 	nop
ffffc020458c: d503201f 	nop
ffffc0204590: d503201f 	nop
ffffc0204594: d503201f 	nop
ffffc0204598: d503201f 	nop
ffffc020459c: d503201f 	nop
ffffc02045a0: d503201f 	nop
ffffc02045a4: d503201f 	nop
ffffc02045a8: d503201f 	nop
ffffc02045ac: d503201f 	nop
ffffc02045b0: d503201f 	nop
ffffc02045b4: d503201f 	nop
ffffc02045b8: d503201f 	nop
ffffc02045bc: d503201f 	nop
ffffc02045c0: d503201f 	nop
ffffc02045c4: d503201f 	nop
ffffc02045c8: d503201f 	nop
ffffc02045cc: d503201f 	nop
ffffc02045d0: d503201f 	nop
ffffc02045d4: d503201f 	nop
ffffc02045d8: d503201f 	nop
ffffc02045dc: d503201f 	nop
ffffc02045e0: d503201f 	nop
ffffc02045e4: d503201f 	nop
ffffc02045e8: d503201f 	nop
ffffc02045ec: d503201f 	nop
ffffc02045f0: d503201f 	nop
ffffc02045f4: d503201f 	nop
ffffc02045f8: d503201f 	nop
ffffc02045fc: d503201f 	nop
ffffc0204600: d503201f 	nop
ffffc0204604: d503201f 	nop
ffffc0204608: d503201f 	nop
ffffc020460c: d503201f 	nop
ffffc0204610: d503201f 	nop
ffffc0204614: d503201f 	nop
ffffc0204618: d503201f 	nop
ffffc020461c: d503201f 	nop
ffffc0204620: d503201f 	nop
ffffc0204624: d503201f 	nop
ffffc0204628: d503201f 	nop
ffffc020462c: d503201f 	nop
ffffc0204630: d503201f 	nop
ffffc0204634: d503201f 	nop
ffffc0204638: d503201f 	nop
ffffc020463c: d503201f 	nop
ffffc0204640: d503201f 	nop
ffffc0204644: d503201f 	nop
ffffc0204648: d503201f 	nop
ffffc020464c: d503201f 	nop
ffffc0204650: d503201f 	nop
ffffc0204654: d503201f 	nop
ffffc0204658: d503201f 	nop
ffffc020465c: d503201f 	nop
ffffc0204660: d503201f 	nop
ffffc0204664: d503201f 	nop
ffffc0204668: d503201f 	nop
ffffc020466c: d503201f 	nop
ffffc0204670: d503201f 	nop
ffffc0204674: d503201f 	nop
ffffc0204678: d503201f 	nop
ffffc020467c: d503201f 	nop
ffffc0204680: d503201f 	nop
ffffc0204684: d503201f 	nop
ffffc0204688: d503201f 	nop
ffffc020468c: d503201f 	nop
ffffc0204690: d503201f 	nop
ffffc0204694: d503201f 	nop
ffffc0204698: d503201f 	nop
ffffc020469c: d503201f 	nop
ffffc02046a0: d503201f 	nop
ffffc02046a4: d503201f 	nop
ffffc02046a8: d503201f 	nop
ffffc02046ac: d503201f 	nop
ffffc02046b0: d503201f 	nop
ffffc02046b4: d503201f 	nop
ffffc02046b8: d503201f 	nop
ffffc02046bc: d503201f 	nop
ffffc02046c0: d503201f 	nop
ffffc02046c4: d503201f 	nop
ffffc02046c8: d503201f 	nop
ffffc02046cc: d503201f 	nop
ffffc02046d0: d503201f 	nop
ffffc02046d4: d503201f 	nop
ffffc02046d8: d503201f 	nop
ffffc02046dc: d503201f 	nop
ffffc02046e0: d503201f 	nop
ffffc02046e4: d503201f 	nop
ffffc02046e8: d503201f 	nop
ffffc02046ec: d503201f 	nop
ffffc02046f0: d503201f 	nop
ffffc02046f4: d503201f 	nop
ffffc02046f8: d503201f 	nop
ffffc02046fc: d503201f 	nop
ffffc0204700: d503201f 	nop
ffffc0204704: d503201f 	nop
ffffc0204708: d503201f 	nop
ffffc020470c: d503201f 	nop
ffffc0204710: d503201f 	nop
ffffc0204714: d503201f 	nop
ffffc0204718: d503201f 	nop
ffffc020471c: d503201f 	nop
ffffc0204720: d503201f 	nop
ffffc0204724: d503201f 	nop
ffffc0204728: d503201f 	nop
ffffc020472c: d503201f 	nop
ffffc0204730: d503201f 	nop
ffffc0204734: d503201f 	nop
ffffc0204738: d503201f 	nop
ffffc020473c: d503201f 	nop
ffffc0204740: d503201f 	nop
ffffc0204744: d503201f 	nop
ffffc0204748: d503201f 	nop
ffffc020474c: d503201f 	nop
ffffc0204750: d503201f 	nop
ffffc0204754: d503201f 	nop
ffffc0204758: d503201f 	nop
ffffc020475c: d503201f 	nop
ffffc0204760: d503201f 	nop
ffffc0204764: d503201f 	nop
ffffc0204768: d503201f 	nop
ffffc020476c: d503201f 	nop
ffffc0204770: d503201f 	nop
ffffc0204774: d503201f 	nop
ffffc0204778: d503201f 	nop
ffffc020477c: d503201f 	nop
ffffc0204780: d503201f 	nop
ffffc0204784: d503201f 	nop
ffffc0204788: d503201f 	nop
ffffc020478c: d503201f 	nop
ffffc0204790: d503201f 	nop
ffffc0204794: d503201f 	nop
ffffc0204798: d503201f 	nop
ffffc020479c: d503201f 	nop
ffffc02047a0: d503201f 	nop
ffffc02047a4: d503201f 	nop
ffffc02047a8: d503201f 	nop
ffffc02047ac: d503201f 	nop
ffffc02047b0: d503201f 	nop
ffffc02047b4: d503201f 	nop
ffffc02047b8: d503201f 	nop
ffffc02047bc: d503201f 	nop
ffffc02047c0: d503201f 	nop
ffffc02047c4: d503201f 	nop
ffffc02047c8: d503201f 	nop
ffffc02047cc: d503201f 	nop
ffffc02047d0: d503201f 	nop
ffffc02047d4: d503201f 	nop
ffffc02047d8: d503201f 	nop
ffffc02047dc: d503201f 	nop
ffffc02047e0: d503201f 	nop
ffffc02047e4: d503201f 	nop
ffffc02047e8: d503201f 	nop
ffffc02047ec: d503201f 	nop
ffffc02047f0: d503201f 	nop
ffffc02047f4: d503201f 	nop
ffffc02047f8: d503201f 	nop
ffffc02047fc: d503201f 	nop

0000ffffc0204800 <bootstrap_vectors>:
ffffc0204800: 14000000 	b	0xffffc0204800 <bootstrap_vectors>
ffffc0204804: d503201f 	nop
ffffc0204808: d503201f 	nop
ffffc020480c: d503201f 	nop
ffffc0204810: d503201f 	nop
ffffc0204814: d503201f 	nop
ffffc0204818: d503201f 	nop
ffffc020481c: d503201f 	nop
ffffc0204820: d503201f 	nop
ffffc0204824: d503201f 	nop
ffffc0204828: d503201f 	nop
ffffc020482c: d503201f 	nop
ffffc0204830: d503201f 	nop
ffffc0204834: d503201f 	nop
ffffc0204838: d503201f 	nop
ffffc020483c: d503201f 	nop
ffffc0204840: d503201f 	nop
ffffc0204844: d503201f 	nop
ffffc0204848: d503201f 	nop
ffffc020484c: d503201f 	nop
ffffc0204850: d503201f 	nop
ffffc0204854: d503201f 	nop
ffffc0204858: d503201f 	nop
ffffc020485c: d503201f 	nop
ffffc0204860: d503201f 	nop
ffffc0204864: d503201f 	nop
ffffc0204868: d503201f 	nop
ffffc020486c: d503201f 	nop
ffffc0204870: d503201f 	nop
ffffc0204874: d503201f 	nop
ffffc0204878: d503201f 	nop
ffffc020487c: d503201f 	nop
ffffc0204880: 14000000 	b	0xffffc0204880 <bootstrap_vectors+0x80>
ffffc0204884: d503201f 	nop
ffffc0204888: d503201f 	nop
ffffc020488c: d503201f 	nop
ffffc0204890: d503201f 	nop
ffffc0204894: d503201f 	nop
ffffc0204898: d503201f 	nop
ffffc020489c: d503201f 	nop
ffffc02048a0: d503201f 	nop
ffffc02048a4: d503201f 	nop
ffffc02048a8: d503201f 	nop
ffffc02048ac: d503201f 	nop
ffffc02048b0: d503201f 	nop
ffffc02048b4: d503201f 	nop
ffffc02048b8: d503201f 	nop
ffffc02048bc: d503201f 	nop
ffffc02048c0: d503201f 	nop
ffffc02048c4: d503201f 	nop
ffffc02048c8: d503201f 	nop
ffffc02048cc: d503201f 	nop
ffffc02048d0: d503201f 	nop
ffffc02048d4: d503201f 	nop
ffffc02048d8: d503201f 	nop
ffffc02048dc: d503201f 	nop
ffffc02048e0: d503201f 	nop
ffffc02048e4: d503201f 	nop
ffffc02048e8: d503201f 	nop
ffffc02048ec: d503201f 	nop
ffffc02048f0: d503201f 	nop
ffffc02048f4: d503201f 	nop
ffffc02048f8: d503201f 	nop
ffffc02048fc: d503201f 	nop
ffffc0204900: 14000000 	b	0xffffc0204900 <bootstrap_vectors+0x100>
ffffc0204904: d503201f 	nop
ffffc0204908: d503201f 	nop
ffffc020490c: d503201f 	nop
ffffc0204910: d503201f 	nop
ffffc0204914: d503201f 	nop
ffffc0204918: d503201f 	nop
ffffc020491c: d503201f 	nop
ffffc0204920: d503201f 	nop
ffffc0204924: d503201f 	nop
ffffc0204928: d503201f 	nop
ffffc020492c: d503201f 	nop
ffffc0204930: d503201f 	nop
ffffc0204934: d503201f 	nop
ffffc0204938: d503201f 	nop
ffffc020493c: d503201f 	nop
ffffc0204940: d503201f 	nop
ffffc0204944: d503201f 	nop
ffffc0204948: d503201f 	nop
ffffc020494c: d503201f 	nop
ffffc0204950: d503201f 	nop
ffffc0204954: d503201f 	nop
ffffc0204958: d503201f 	nop
ffffc020495c: d503201f 	nop
ffffc0204960: d503201f 	nop
ffffc0204964: d503201f 	nop
ffffc0204968: d503201f 	nop
ffffc020496c: d503201f 	nop
ffffc0204970: d503201f 	nop
ffffc0204974: d503201f 	nop
ffffc0204978: d503201f 	nop
ffffc020497c: d503201f 	nop
ffffc0204980: 14000000 	b	0xffffc0204980 <bootstrap_vectors+0x180>
ffffc0204984: d503201f 	nop
ffffc0204988: d503201f 	nop
ffffc020498c: d503201f 	nop
ffffc0204990: d503201f 	nop
ffffc0204994: d503201f 	nop
ffffc0204998: d503201f 	nop
ffffc020499c: d503201f 	nop
ffffc02049a0: d503201f 	nop
ffffc02049a4: d503201f 	nop
ffffc02049a8: d503201f 	nop
ffffc02049ac: d503201f 	nop
ffffc02049b0: d503201f 	nop
ffffc02049b4: d503201f 	nop
ffffc02049b8: d503201f 	nop
ffffc02049bc: d503201f 	nop
ffffc02049c0: d503201f 	nop
ffffc02049c4: d503201f 	nop
ffffc02049c8: d503201f 	nop
ffffc02049cc: d503201f 	nop
ffffc02049d0: d503201f 	nop
ffffc02049d4: d503201f 	nop
ffffc02049d8: d503201f 	nop
ffffc02049dc: d503201f 	nop
ffffc02049e0: d503201f 	nop
ffffc02049e4: d503201f 	nop
ffffc02049e8: d503201f 	nop
ffffc02049ec: d503201f 	nop
ffffc02049f0: d503201f 	nop
ffffc02049f4: d503201f 	nop
ffffc02049f8: d503201f 	nop
ffffc02049fc: d503201f 	nop
ffffc0204a00: 14000000 	b	0xffffc0204a00 <bootstrap_vectors+0x200>
ffffc0204a04: d503201f 	nop
ffffc0204a08: d503201f 	nop
ffffc0204a0c: d503201f 	nop
ffffc0204a10: d503201f 	nop
ffffc0204a14: d503201f 	nop
ffffc0204a18: d503201f 	nop
ffffc0204a1c: d503201f 	nop
ffffc0204a20: d503201f 	nop
ffffc0204a24: d503201f 	nop
ffffc0204a28: d503201f 	nop
ffffc0204a2c: d503201f 	nop
ffffc0204a30: d503201f 	nop
ffffc0204a34: d503201f 	nop
ffffc0204a38: d503201f 	nop
ffffc0204a3c: d503201f 	nop
ffffc0204a40: d503201f 	nop
ffffc0204a44: d503201f 	nop
ffffc0204a48: d503201f 	nop
ffffc0204a4c: d503201f 	nop
ffffc0204a50: d503201f 	nop
ffffc0204a54: d503201f 	nop
ffffc0204a58: d503201f 	nop
ffffc0204a5c: d503201f 	nop
ffffc0204a60: d503201f 	nop
ffffc0204a64: d503201f 	nop
ffffc0204a68: d503201f 	nop
ffffc0204a6c: d503201f 	nop
ffffc0204a70: d503201f 	nop
ffffc0204a74: d503201f 	nop
ffffc0204a78: d503201f 	nop
ffffc0204a7c: d503201f 	nop
ffffc0204a80: 14000000 	b	0xffffc0204a80 <bootstrap_vectors+0x280>
ffffc0204a84: d503201f 	nop
ffffc0204a88: d503201f 	nop
ffffc0204a8c: d503201f 	nop
ffffc0204a90: d503201f 	nop
ffffc0204a94: d503201f 	nop
ffffc0204a98: d503201f 	nop
ffffc0204a9c: d503201f 	nop
ffffc0204aa0: d503201f 	nop
ffffc0204aa4: d503201f 	nop
ffffc0204aa8: d503201f 	nop
ffffc0204aac: d503201f 	nop
ffffc0204ab0: d503201f 	nop
ffffc0204ab4: d503201f 	nop
ffffc0204ab8: d503201f 	nop
ffffc0204abc: d503201f 	nop
ffffc0204ac0: d503201f 	nop
ffffc0204ac4: d503201f 	nop
ffffc0204ac8: d503201f 	nop
ffffc0204acc: d503201f 	nop
ffffc0204ad0: d503201f 	nop
ffffc0204ad4: d503201f 	nop
ffffc0204ad8: d503201f 	nop
ffffc0204adc: d503201f 	nop
ffffc0204ae0: d503201f 	nop
ffffc0204ae4: d503201f 	nop
ffffc0204ae8: d503201f 	nop
ffffc0204aec: d503201f 	nop
ffffc0204af0: d503201f 	nop
ffffc0204af4: d503201f 	nop
ffffc0204af8: d503201f 	nop
ffffc0204afc: d503201f 	nop
ffffc0204b00: 14000000 	b	0xffffc0204b00 <bootstrap_vectors+0x300>
ffffc0204b04: d503201f 	nop
ffffc0204b08: d503201f 	nop
ffffc0204b0c: d503201f 	nop
ffffc0204b10: d503201f 	nop
ffffc0204b14: d503201f 	nop
ffffc0204b18: d503201f 	nop
ffffc0204b1c: d503201f 	nop
ffffc0204b20: d503201f 	nop
ffffc0204b24: d503201f 	nop
ffffc0204b28: d503201f 	nop
ffffc0204b2c: d503201f 	nop
ffffc0204b30: d503201f 	nop
ffffc0204b34: d503201f 	nop
ffffc0204b38: d503201f 	nop
ffffc0204b3c: d503201f 	nop
ffffc0204b40: d503201f 	nop
ffffc0204b44: d503201f 	nop
ffffc0204b48: d503201f 	nop
ffffc0204b4c: d503201f 	nop
ffffc0204b50: d503201f 	nop
ffffc0204b54: d503201f 	nop
ffffc0204b58: d503201f 	nop
ffffc0204b5c: d503201f 	nop
ffffc0204b60: d503201f 	nop
ffffc0204b64: d503201f 	nop
ffffc0204b68: d503201f 	nop
ffffc0204b6c: d503201f 	nop
ffffc0204b70: d503201f 	nop
ffffc0204b74: d503201f 	nop
ffffc0204b78: d503201f 	nop
ffffc0204b7c: d503201f 	nop
ffffc0204b80: 14000000 	b	0xffffc0204b80 <bootstrap_vectors+0x380>
ffffc0204b84: d503201f 	nop
ffffc0204b88: d503201f 	nop
ffffc0204b8c: d503201f 	nop
ffffc0204b90: d503201f 	nop
ffffc0204b94: d503201f 	nop
ffffc0204b98: d503201f 	nop
ffffc0204b9c: d503201f 	nop
ffffc0204ba0: d503201f 	nop
ffffc0204ba4: d503201f 	nop
ffffc0204ba8: d503201f 	nop
ffffc0204bac: d503201f 	nop
ffffc0204bb0: d503201f 	nop
ffffc0204bb4: d503201f 	nop
ffffc0204bb8: d503201f 	nop
ffffc0204bbc: d503201f 	nop
ffffc0204bc0: d503201f 	nop
ffffc0204bc4: d503201f 	nop
ffffc0204bc8: d503201f 	nop
ffffc0204bcc: d503201f 	nop
ffffc0204bd0: d503201f 	nop
ffffc0204bd4: d503201f 	nop
ffffc0204bd8: d503201f 	nop
ffffc0204bdc: d503201f 	nop
ffffc0204be0: d503201f 	nop
ffffc0204be4: d503201f 	nop
ffffc0204be8: d503201f 	nop
ffffc0204bec: d503201f 	nop
ffffc0204bf0: d503201f 	nop
ffffc0204bf4: d503201f 	nop
ffffc0204bf8: d503201f 	nop
ffffc0204bfc: d503201f 	nop
ffffc0204c00: 17fffd1c 	b	0xffffc0204070 <el2_entry>
ffffc0204c04: d503201f 	nop
ffffc0204c08: d503201f 	nop
ffffc0204c0c: d503201f 	nop
ffffc0204c10: d503201f 	nop
ffffc0204c14: d503201f 	nop
ffffc0204c18: d503201f 	nop
ffffc0204c1c: d503201f 	nop
ffffc0204c20: d503201f 	nop
ffffc0204c24: d503201f 	nop
ffffc0204c28: d503201f 	nop
ffffc0204c2c: d503201f 	nop
ffffc0204c30: d503201f 	nop
ffffc0204c34: d503201f 	nop
ffffc0204c38: d503201f 	nop
ffffc0204c3c: d503201f 	nop
ffffc0204c40: d503201f 	nop
ffffc0204c44: d503201f 	nop
ffffc0204c48: d503201f 	nop
ffffc0204c4c: d503201f 	nop
ffffc0204c50: d503201f 	nop
ffffc0204c54: d503201f 	nop
ffffc0204c58: d503201f 	nop
ffffc0204c5c: d503201f 	nop
ffffc0204c60: d503201f 	nop
ffffc0204c64: d503201f 	nop
ffffc0204c68: d503201f 	nop
ffffc0204c6c: d503201f 	nop
ffffc0204c70: d503201f 	nop
ffffc0204c74: d503201f 	nop
ffffc0204c78: d503201f 	nop
ffffc0204c7c: d503201f 	nop
ffffc0204c80: 14000000 	b	0xffffc0204c80 <bootstrap_vectors+0x480>
ffffc0204c84: d503201f 	nop
ffffc0204c88: d503201f 	nop
ffffc0204c8c: d503201f 	nop
ffffc0204c90: d503201f 	nop
ffffc0204c94: d503201f 	nop
ffffc0204c98: d503201f 	nop
ffffc0204c9c: d503201f 	nop
ffffc0204ca0: d503201f 	nop
ffffc0204ca4: d503201f 	nop
ffffc0204ca8: d503201f 	nop
ffffc0204cac: d503201f 	nop
ffffc0204cb0: d503201f 	nop
ffffc0204cb4: d503201f 	nop
ffffc0204cb8: d503201f 	nop
ffffc0204cbc: d503201f 	nop
ffffc0204cc0: d503201f 	nop
ffffc0204cc4: d503201f 	nop
ffffc0204cc8: d503201f 	nop
ffffc0204ccc: d503201f 	nop
ffffc0204cd0: d503201f 	nop
ffffc0204cd4: d503201f 	nop
ffffc0204cd8: d503201f 	nop
ffffc0204cdc: d503201f 	nop
ffffc0204ce0: d503201f 	nop
ffffc0204ce4: d503201f 	nop
ffffc0204ce8: d503201f 	nop
ffffc0204cec: d503201f 	nop
ffffc0204cf0: d503201f 	nop
ffffc0204cf4: d503201f 	nop
ffffc0204cf8: d503201f 	nop
ffffc0204cfc: d503201f 	nop
ffffc0204d00: 14000000 	b	0xffffc0204d00 <bootstrap_vectors+0x500>
ffffc0204d04: d503201f 	nop
ffffc0204d08: d503201f 	nop
ffffc0204d0c: d503201f 	nop
ffffc0204d10: d503201f 	nop
ffffc0204d14: d503201f 	nop
ffffc0204d18: d503201f 	nop
ffffc0204d1c: d503201f 	nop
ffffc0204d20: d503201f 	nop
ffffc0204d24: d503201f 	nop
ffffc0204d28: d503201f 	nop
ffffc0204d2c: d503201f 	nop
ffffc0204d30: d503201f 	nop
ffffc0204d34: d503201f 	nop
ffffc0204d38: d503201f 	nop
ffffc0204d3c: d503201f 	nop
ffffc0204d40: d503201f 	nop
ffffc0204d44: d503201f 	nop
ffffc0204d48: d503201f 	nop
ffffc0204d4c: d503201f 	nop
ffffc0204d50: d503201f 	nop
ffffc0204d54: d503201f 	nop
ffffc0204d58: d503201f 	nop
ffffc0204d5c: d503201f 	nop
ffffc0204d60: d503201f 	nop
ffffc0204d64: d503201f 	nop
ffffc0204d68: d503201f 	nop
ffffc0204d6c: d503201f 	nop
ffffc0204d70: d503201f 	nop
ffffc0204d74: d503201f 	nop
ffffc0204d78: d503201f 	nop
ffffc0204d7c: d503201f 	nop
ffffc0204d80: 14000000 	b	0xffffc0204d80 <bootstrap_vectors+0x580>
ffffc0204d84: d503201f 	nop
ffffc0204d88: d503201f 	nop
ffffc0204d8c: d503201f 	nop
ffffc0204d90: d503201f 	nop
ffffc0204d94: d503201f 	nop
ffffc0204d98: d503201f 	nop
ffffc0204d9c: d503201f 	nop
ffffc0204da0: d503201f 	nop
ffffc0204da4: d503201f 	nop
ffffc0204da8: d503201f 	nop
ffffc0204dac: d503201f 	nop
ffffc0204db0: d503201f 	nop
ffffc0204db4: d503201f 	nop
ffffc0204db8: d503201f 	nop
ffffc0204dbc: d503201f 	nop
ffffc0204dc0: d503201f 	nop
ffffc0204dc4: d503201f 	nop
ffffc0204dc8: d503201f 	nop
ffffc0204dcc: d503201f 	nop
ffffc0204dd0: d503201f 	nop
ffffc0204dd4: d503201f 	nop
ffffc0204dd8: d503201f 	nop
ffffc0204ddc: d503201f 	nop
ffffc0204de0: d503201f 	nop
ffffc0204de4: d503201f 	nop
ffffc0204de8: d503201f 	nop
ffffc0204dec: d503201f 	nop
ffffc0204df0: d503201f 	nop
ffffc0204df4: d503201f 	nop
ffffc0204df8: d503201f 	nop
ffffc0204dfc: d503201f 	nop
ffffc0204e00: 14000000 	b	0xffffc0204e00 <bootstrap_vectors+0x600>
ffffc0204e04: d503201f 	nop
ffffc0204e08: d503201f 	nop
ffffc0204e0c: d503201f 	nop
ffffc0204e10: d503201f 	nop
ffffc0204e14: d503201f 	nop
ffffc0204e18: d503201f 	nop
ffffc0204e1c: d503201f 	nop
ffffc0204e20: d503201f 	nop
ffffc0204e24: d503201f 	nop
ffffc0204e28: d503201f 	nop
ffffc0204e2c: d503201f 	nop
ffffc0204e30: d503201f 	nop
ffffc0204e34: d503201f 	nop
ffffc0204e38: d503201f 	nop
ffffc0204e3c: d503201f 	nop
ffffc0204e40: d503201f 	nop
ffffc0204e44: d503201f 	nop
ffffc0204e48: d503201f 	nop
ffffc0204e4c: d503201f 	nop
ffffc0204e50: d503201f 	nop
ffffc0204e54: d503201f 	nop
ffffc0204e58: d503201f 	nop
ffffc0204e5c: d503201f 	nop
ffffc0204e60: d503201f 	nop
ffffc0204e64: d503201f 	nop
ffffc0204e68: d503201f 	nop
ffffc0204e6c: d503201f 	nop
ffffc0204e70: d503201f 	nop
ffffc0204e74: d503201f 	nop
ffffc0204e78: d503201f 	nop
ffffc0204e7c: d503201f 	nop
ffffc0204e80: 14000000 	b	0xffffc0204e80 <bootstrap_vectors+0x680>
ffffc0204e84: d503201f 	nop
ffffc0204e88: d503201f 	nop
ffffc0204e8c: d503201f 	nop
ffffc0204e90: d503201f 	nop
ffffc0204e94: d503201f 	nop
ffffc0204e98: d503201f 	nop
ffffc0204e9c: d503201f 	nop
ffffc0204ea0: d503201f 	nop
ffffc0204ea4: d503201f 	nop
ffffc0204ea8: d503201f 	nop
ffffc0204eac: d503201f 	nop
ffffc0204eb0: d503201f 	nop
ffffc0204eb4: d503201f 	nop
ffffc0204eb8: d503201f 	nop
ffffc0204ebc: d503201f 	nop
ffffc0204ec0: d503201f 	nop
ffffc0204ec4: d503201f 	nop
ffffc0204ec8: d503201f 	nop
ffffc0204ecc: d503201f 	nop
ffffc0204ed0: d503201f 	nop
ffffc0204ed4: d503201f 	nop
ffffc0204ed8: d503201f 	nop
ffffc0204edc: d503201f 	nop
ffffc0204ee0: d503201f 	nop
ffffc0204ee4: d503201f 	nop
ffffc0204ee8: d503201f 	nop
ffffc0204eec: d503201f 	nop
ffffc0204ef0: d503201f 	nop
ffffc0204ef4: d503201f 	nop
ffffc0204ef8: d503201f 	nop
ffffc0204efc: d503201f 	nop
ffffc0204f00: 14000000 	b	0xffffc0204f00 <bootstrap_vectors+0x700>
ffffc0204f04: d503201f 	nop
ffffc0204f08: d503201f 	nop
ffffc0204f0c: d503201f 	nop
ffffc0204f10: d503201f 	nop
ffffc0204f14: d503201f 	nop
ffffc0204f18: d503201f 	nop
ffffc0204f1c: d503201f 	nop
ffffc0204f20: d503201f 	nop
ffffc0204f24: d503201f 	nop
ffffc0204f28: d503201f 	nop
ffffc0204f2c: d503201f 	nop
ffffc0204f30: d503201f 	nop
ffffc0204f34: d503201f 	nop
ffffc0204f38: d503201f 	nop
ffffc0204f3c: d503201f 	nop
ffffc0204f40: d503201f 	nop
ffffc0204f44: d503201f 	nop
ffffc0204f48: d503201f 	nop
ffffc0204f4c: d503201f 	nop
ffffc0204f50: d503201f 	nop
ffffc0204f54: d503201f 	nop
ffffc0204f58: d503201f 	nop
ffffc0204f5c: d503201f 	nop
ffffc0204f60: d503201f 	nop
ffffc0204f64: d503201f 	nop
ffffc0204f68: d503201f 	nop
ffffc0204f6c: d503201f 	nop
ffffc0204f70: d503201f 	nop
ffffc0204f74: d503201f 	nop
ffffc0204f78: d503201f 	nop
ffffc0204f7c: d503201f 	nop
ffffc0204f80: 14000000 	b	0xffffc0204f80 <bootstrap_vectors+0x780>
ffffc0204f84: d503201f 	nop
ffffc0204f88: d503201f 	nop
ffffc0204f8c: d503201f 	nop
ffffc0204f90: d503201f 	nop
ffffc0204f94: d503201f 	nop
ffffc0204f98: d503201f 	nop
ffffc0204f9c: d503201f 	nop
ffffc0204fa0: d503201f 	nop
ffffc0204fa4: d503201f 	nop
ffffc0204fa8: d503201f 	nop
ffffc0204fac: d503201f 	nop
ffffc0204fb0: d503201f 	nop
ffffc0204fb4: d503201f 	nop
ffffc0204fb8: d503201f 	nop
ffffc0204fbc: d503201f 	nop
ffffc0204fc0: d503201f 	nop
ffffc0204fc4: d503201f 	nop
ffffc0204fc8: d503201f 	nop
ffffc0204fcc: d503201f 	nop
ffffc0204fd0: d503201f 	nop
ffffc0204fd4: d503201f 	nop
ffffc0204fd8: d503201f 	nop
ffffc0204fdc: d503201f 	nop
ffffc0204fe0: d503201f 	nop
ffffc0204fe4: d503201f 	nop
ffffc0204fe8: d503201f 	nop
ffffc0204fec: d503201f 	nop
ffffc0204ff0: d503201f 	nop
ffffc0204ff4: d503201f 	nop
ffffc0204ff8: d503201f 	nop
ffffc0204ffc: d503201f 	nop

0000ffffc0205000 <hyp_vectors>:
ffffc0205000: 14000000 	b	0xffffc0205000 <hyp_vectors>
ffffc0205004: d503201f 	nop
ffffc0205008: d503201f 	nop
ffffc020500c: d503201f 	nop
ffffc0205010: d503201f 	nop
ffffc0205014: d503201f 	nop
ffffc0205018: d503201f 	nop
ffffc020501c: d503201f 	nop
ffffc0205020: d503201f 	nop
ffffc0205024: d503201f 	nop
ffffc0205028: d503201f 	nop
ffffc020502c: d503201f 	nop
ffffc0205030: d503201f 	nop
ffffc0205034: d503201f 	nop
ffffc0205038: d503201f 	nop
ffffc020503c: d503201f 	nop
ffffc0205040: d503201f 	nop
ffffc0205044: d503201f 	nop
ffffc0205048: d503201f 	nop
ffffc020504c: d503201f 	nop
ffffc0205050: d503201f 	nop
ffffc0205054: d503201f 	nop
ffffc0205058: d503201f 	nop
ffffc020505c: d503201f 	nop
ffffc0205060: d503201f 	nop
ffffc0205064: d503201f 	nop
ffffc0205068: d503201f 	nop
ffffc020506c: d503201f 	nop
ffffc0205070: d503201f 	nop
ffffc0205074: d503201f 	nop
ffffc0205078: d503201f 	nop
ffffc020507c: d503201f 	nop
ffffc0205080: 14000000 	b	0xffffc0205080 <hyp_vectors+0x80>
ffffc0205084: d503201f 	nop
ffffc0205088: d503201f 	nop
ffffc020508c: d503201f 	nop
ffffc0205090: d503201f 	nop
ffffc0205094: d503201f 	nop
ffffc0205098: d503201f 	nop
ffffc020509c: d503201f 	nop
ffffc02050a0: d503201f 	nop
ffffc02050a4: d503201f 	nop
ffffc02050a8: d503201f 	nop
ffffc02050ac: d503201f 	nop
ffffc02050b0: d503201f 	nop
ffffc02050b4: d503201f 	nop
ffffc02050b8: d503201f 	nop
ffffc02050bc: d503201f 	nop
ffffc02050c0: d503201f 	nop
ffffc02050c4: d503201f 	nop
ffffc02050c8: d503201f 	nop
ffffc02050cc: d503201f 	nop
ffffc02050d0: d503201f 	nop
ffffc02050d4: d503201f 	nop
ffffc02050d8: d503201f 	nop
ffffc02050dc: d503201f 	nop
ffffc02050e0: d503201f 	nop
ffffc02050e4: d503201f 	nop
ffffc02050e8: d503201f 	nop
ffffc02050ec: d503201f 	nop
ffffc02050f0: d503201f 	nop
ffffc02050f4: d503201f 	nop
ffffc02050f8: d503201f 	nop
ffffc02050fc: d503201f 	nop
ffffc0205100: 14000000 	b	0xffffc0205100 <hyp_vectors+0x100>
ffffc0205104: d503201f 	nop
ffffc0205108: d503201f 	nop
ffffc020510c: d503201f 	nop
ffffc0205110: d503201f 	nop
ffffc0205114: d503201f 	nop
ffffc0205118: d503201f 	nop
ffffc020511c: d503201f 	nop
ffffc0205120: d503201f 	nop
ffffc0205124: d503201f 	nop
ffffc0205128: d503201f 	nop
ffffc020512c: d503201f 	nop
ffffc0205130: d503201f 	nop
ffffc0205134: d503201f 	nop
ffffc0205138: d503201f 	nop
ffffc020513c: d503201f 	nop
ffffc0205140: d503201f 	nop
ffffc0205144: d503201f 	nop
ffffc0205148: d503201f 	nop
ffffc020514c: d503201f 	nop
ffffc0205150: d503201f 	nop
ffffc0205154: d503201f 	nop
ffffc0205158: d503201f 	nop
ffffc020515c: d503201f 	nop
ffffc0205160: d503201f 	nop
ffffc0205164: d503201f 	nop
ffffc0205168: d503201f 	nop
ffffc020516c: d503201f 	nop
ffffc0205170: d503201f 	nop
ffffc0205174: d503201f 	nop
ffffc0205178: d503201f 	nop
ffffc020517c: d503201f 	nop
ffffc0205180: 14000000 	b	0xffffc0205180 <hyp_vectors+0x180>
ffffc0205184: d503201f 	nop
ffffc0205188: d503201f 	nop
ffffc020518c: d503201f 	nop
ffffc0205190: d503201f 	nop
ffffc0205194: d503201f 	nop
ffffc0205198: d503201f 	nop
ffffc020519c: d503201f 	nop
ffffc02051a0: d503201f 	nop
ffffc02051a4: d503201f 	nop
ffffc02051a8: d503201f 	nop
ffffc02051ac: d503201f 	nop
ffffc02051b0: d503201f 	nop
ffffc02051b4: d503201f 	nop
ffffc02051b8: d503201f 	nop
ffffc02051bc: d503201f 	nop
ffffc02051c0: d503201f 	nop
ffffc02051c4: d503201f 	nop
ffffc02051c8: d503201f 	nop
ffffc02051cc: d503201f 	nop
ffffc02051d0: d503201f 	nop
ffffc02051d4: d503201f 	nop
ffffc02051d8: d503201f 	nop
ffffc02051dc: d503201f 	nop
ffffc02051e0: d503201f 	nop
ffffc02051e4: d503201f 	nop
ffffc02051e8: d503201f 	nop
ffffc02051ec: d503201f 	nop
ffffc02051f0: d503201f 	nop
ffffc02051f4: d503201f 	nop
ffffc02051f8: d503201f 	nop
ffffc02051fc: d503201f 	nop
ffffc0205200: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0205204: a9bf73fb 	stp	x27, x28, [sp, #-16]!
ffffc0205208: a9bf6bf9 	stp	x25, x26, [sp, #-16]!
ffffc020520c: a9bf63f7 	stp	x23, x24, [sp, #-16]!
ffffc0205210: a9bf5bf5 	stp	x21, x22, [sp, #-16]!
ffffc0205214: a9bf53f3 	stp	x19, x20, [sp, #-16]!
ffffc0205218: a9bf4bf1 	stp	x17, x18, [sp, #-16]!
ffffc020521c: a9bf43ef 	stp	x15, x16, [sp, #-16]!
ffffc0205220: a9bf3bed 	stp	x13, x14, [sp, #-16]!
ffffc0205224: a9bf33eb 	stp	x11, x12, [sp, #-16]!
ffffc0205228: a9bf2be9 	stp	x9, x10, [sp, #-16]!
ffffc020522c: a9bf23e7 	stp	x7, x8, [sp, #-16]!
ffffc0205230: a9bf1be5 	stp	x5, x6, [sp, #-16]!
ffffc0205234: a9bf13e3 	stp	x3, x4, [sp, #-16]!
ffffc0205238: a9bf0be1 	stp	x1, x2, [sp, #-16]!
ffffc020523c: d2800001 	mov	x1, #0
ffffc0205240: a9bf03e1 	stp	x1, x0, [sp, #-16]!
ffffc0205244: aa1f03fd 	mov	x29, xzr
ffffc0205248: aa1f03fe 	mov	x30, xzr
ffffc020524c: 910003e0 	mov	x0, sp
ffffc0205250: 94000332 	bl	0xffffc0205f18 <arch_handle_exit>
ffffc0205254: 14000000 	b	0xffffc0205254 <hyp_vectors+0x254>
ffffc0205258: d503201f 	nop
ffffc020525c: d503201f 	nop
ffffc0205260: d503201f 	nop
ffffc0205264: d503201f 	nop
ffffc0205268: d503201f 	nop
ffffc020526c: d503201f 	nop
ffffc0205270: d503201f 	nop
ffffc0205274: d503201f 	nop
ffffc0205278: d503201f 	nop
ffffc020527c: d503201f 	nop
ffffc0205280: 14000000 	b	0xffffc0205280 <hyp_vectors+0x280>
ffffc0205284: d503201f 	nop
ffffc0205288: d503201f 	nop
ffffc020528c: d503201f 	nop
ffffc0205290: d503201f 	nop
ffffc0205294: d503201f 	nop
ffffc0205298: d503201f 	nop
ffffc020529c: d503201f 	nop
ffffc02052a0: d503201f 	nop
ffffc02052a4: d503201f 	nop
ffffc02052a8: d503201f 	nop
ffffc02052ac: d503201f 	nop
ffffc02052b0: d503201f 	nop
ffffc02052b4: d503201f 	nop
ffffc02052b8: d503201f 	nop
ffffc02052bc: d503201f 	nop
ffffc02052c0: d503201f 	nop
ffffc02052c4: d503201f 	nop
ffffc02052c8: d503201f 	nop
ffffc02052cc: d503201f 	nop
ffffc02052d0: d503201f 	nop
ffffc02052d4: d503201f 	nop
ffffc02052d8: d503201f 	nop
ffffc02052dc: d503201f 	nop
ffffc02052e0: d503201f 	nop
ffffc02052e4: d503201f 	nop
ffffc02052e8: d503201f 	nop
ffffc02052ec: d503201f 	nop
ffffc02052f0: d503201f 	nop
ffffc02052f4: d503201f 	nop
ffffc02052f8: d503201f 	nop
ffffc02052fc: d503201f 	nop
ffffc0205300: 14000000 	b	0xffffc0205300 <hyp_vectors+0x300>
ffffc0205304: d503201f 	nop
ffffc0205308: d503201f 	nop
ffffc020530c: d503201f 	nop
ffffc0205310: d503201f 	nop
ffffc0205314: d503201f 	nop
ffffc0205318: d503201f 	nop
ffffc020531c: d503201f 	nop
ffffc0205320: d503201f 	nop
ffffc0205324: d503201f 	nop
ffffc0205328: d503201f 	nop
ffffc020532c: d503201f 	nop
ffffc0205330: d503201f 	nop
ffffc0205334: d503201f 	nop
ffffc0205338: d503201f 	nop
ffffc020533c: d503201f 	nop
ffffc0205340: d503201f 	nop
ffffc0205344: d503201f 	nop
ffffc0205348: d503201f 	nop
ffffc020534c: d503201f 	nop
ffffc0205350: d503201f 	nop
ffffc0205354: d503201f 	nop
ffffc0205358: d503201f 	nop
ffffc020535c: d503201f 	nop
ffffc0205360: d503201f 	nop
ffffc0205364: d503201f 	nop
ffffc0205368: d503201f 	nop
ffffc020536c: d503201f 	nop
ffffc0205370: d503201f 	nop
ffffc0205374: d503201f 	nop
ffffc0205378: d503201f 	nop
ffffc020537c: d503201f 	nop
ffffc0205380: 14000000 	b	0xffffc0205380 <hyp_vectors+0x380>
ffffc0205384: d503201f 	nop
ffffc0205388: d503201f 	nop
ffffc020538c: d503201f 	nop
ffffc0205390: d503201f 	nop
ffffc0205394: d503201f 	nop
ffffc0205398: d503201f 	nop
ffffc020539c: d503201f 	nop
ffffc02053a0: d503201f 	nop
ffffc02053a4: d503201f 	nop
ffffc02053a8: d503201f 	nop
ffffc02053ac: d503201f 	nop
ffffc02053b0: d503201f 	nop
ffffc02053b4: d503201f 	nop
ffffc02053b8: d503201f 	nop
ffffc02053bc: d503201f 	nop
ffffc02053c0: d503201f 	nop
ffffc02053c4: d503201f 	nop
ffffc02053c8: d503201f 	nop
ffffc02053cc: d503201f 	nop
ffffc02053d0: d503201f 	nop
ffffc02053d4: d503201f 	nop
ffffc02053d8: d503201f 	nop
ffffc02053dc: d503201f 	nop
ffffc02053e0: d503201f 	nop
ffffc02053e4: d503201f 	nop
ffffc02053e8: d503201f 	nop
ffffc02053ec: d503201f 	nop
ffffc02053f0: d503201f 	nop
ffffc02053f4: d503201f 	nop
ffffc02053f8: d503201f 	nop
ffffc02053fc: d503201f 	nop
ffffc0205400: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0205404: a9bf73fb 	stp	x27, x28, [sp, #-16]!
ffffc0205408: a9bf6bf9 	stp	x25, x26, [sp, #-16]!
ffffc020540c: a9bf63f7 	stp	x23, x24, [sp, #-16]!
ffffc0205410: a9bf5bf5 	stp	x21, x22, [sp, #-16]!
ffffc0205414: a9bf53f3 	stp	x19, x20, [sp, #-16]!
ffffc0205418: a9bf4bf1 	stp	x17, x18, [sp, #-16]!
ffffc020541c: a9bf43ef 	stp	x15, x16, [sp, #-16]!
ffffc0205420: a9bf3bed 	stp	x13, x14, [sp, #-16]!
ffffc0205424: a9bf33eb 	stp	x11, x12, [sp, #-16]!
ffffc0205428: a9bf2be9 	stp	x9, x10, [sp, #-16]!
ffffc020542c: a9bf23e7 	stp	x7, x8, [sp, #-16]!
ffffc0205430: a9bf1be5 	stp	x5, x6, [sp, #-16]!
ffffc0205434: a9bf13e3 	stp	x3, x4, [sp, #-16]!
ffffc0205438: a9bf0be1 	stp	x1, x2, [sp, #-16]!
ffffc020543c: d2800021 	mov	x1, #1
ffffc0205440: a9bf03e1 	stp	x1, x0, [sp, #-16]!
ffffc0205444: aa1f03fd 	mov	x29, xzr
ffffc0205448: aa1f03fe 	mov	x30, xzr
ffffc020544c: 910003e0 	mov	x0, sp
ffffc0205450: 940002b2 	bl	0xffffc0205f18 <arch_handle_exit>
ffffc0205454: 14000000 	b	0xffffc0205454 <hyp_vectors+0x454>
ffffc0205458: d503201f 	nop
ffffc020545c: d503201f 	nop
ffffc0205460: d503201f 	nop
ffffc0205464: d503201f 	nop
ffffc0205468: d503201f 	nop
ffffc020546c: d503201f 	nop
ffffc0205470: d503201f 	nop
ffffc0205474: d503201f 	nop
ffffc0205478: d503201f 	nop
ffffc020547c: d503201f 	nop
ffffc0205480: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0205484: a9bf73fb 	stp	x27, x28, [sp, #-16]!
ffffc0205488: a9bf6bf9 	stp	x25, x26, [sp, #-16]!
ffffc020548c: a9bf63f7 	stp	x23, x24, [sp, #-16]!
ffffc0205490: a9bf5bf5 	stp	x21, x22, [sp, #-16]!
ffffc0205494: a9bf53f3 	stp	x19, x20, [sp, #-16]!
ffffc0205498: a9bf4bf1 	stp	x17, x18, [sp, #-16]!
ffffc020549c: a9bf43ef 	stp	x15, x16, [sp, #-16]!
ffffc02054a0: a9bf3bed 	stp	x13, x14, [sp, #-16]!
ffffc02054a4: a9bf33eb 	stp	x11, x12, [sp, #-16]!
ffffc02054a8: a9bf2be9 	stp	x9, x10, [sp, #-16]!
ffffc02054ac: a9bf23e7 	stp	x7, x8, [sp, #-16]!
ffffc02054b0: a9bf1be5 	stp	x5, x6, [sp, #-16]!
ffffc02054b4: a9bf13e3 	stp	x3, x4, [sp, #-16]!
ffffc02054b8: a9bf0be1 	stp	x1, x2, [sp, #-16]!
ffffc02054bc: d2800041 	mov	x1, #2
ffffc02054c0: a9bf03e1 	stp	x1, x0, [sp, #-16]!
ffffc02054c4: aa1f03fd 	mov	x29, xzr
ffffc02054c8: aa1f03fe 	mov	x30, xzr
ffffc02054cc: 910003e0 	mov	x0, sp
ffffc02054d0: 94000292 	bl	0xffffc0205f18 <arch_handle_exit>
ffffc02054d4: 14000000 	b	0xffffc02054d4 <hyp_vectors+0x4d4>
ffffc02054d8: d503201f 	nop
ffffc02054dc: d503201f 	nop
ffffc02054e0: d503201f 	nop
ffffc02054e4: d503201f 	nop
ffffc02054e8: d503201f 	nop
ffffc02054ec: d503201f 	nop
ffffc02054f0: d503201f 	nop
ffffc02054f4: d503201f 	nop
ffffc02054f8: d503201f 	nop
ffffc02054fc: d503201f 	nop
ffffc0205500: 14000000 	b	0xffffc0205500 <hyp_vectors+0x500>
ffffc0205504: d503201f 	nop
ffffc0205508: d503201f 	nop
ffffc020550c: d503201f 	nop
ffffc0205510: d503201f 	nop
ffffc0205514: d503201f 	nop
ffffc0205518: d503201f 	nop
ffffc020551c: d503201f 	nop
ffffc0205520: d503201f 	nop
ffffc0205524: d503201f 	nop
ffffc0205528: d503201f 	nop
ffffc020552c: d503201f 	nop
ffffc0205530: d503201f 	nop
ffffc0205534: d503201f 	nop
ffffc0205538: d503201f 	nop
ffffc020553c: d503201f 	nop
ffffc0205540: d503201f 	nop
ffffc0205544: d503201f 	nop
ffffc0205548: d503201f 	nop
ffffc020554c: d503201f 	nop
ffffc0205550: d503201f 	nop
ffffc0205554: d503201f 	nop
ffffc0205558: d503201f 	nop
ffffc020555c: d503201f 	nop
ffffc0205560: d503201f 	nop
ffffc0205564: d503201f 	nop
ffffc0205568: d503201f 	nop
ffffc020556c: d503201f 	nop
ffffc0205570: d503201f 	nop
ffffc0205574: d503201f 	nop
ffffc0205578: d503201f 	nop
ffffc020557c: d503201f 	nop
ffffc0205580: 14000000 	b	0xffffc0205580 <hyp_vectors+0x580>
ffffc0205584: d503201f 	nop
ffffc0205588: d503201f 	nop
ffffc020558c: d503201f 	nop
ffffc0205590: d503201f 	nop
ffffc0205594: d503201f 	nop
ffffc0205598: d503201f 	nop
ffffc020559c: d503201f 	nop
ffffc02055a0: d503201f 	nop
ffffc02055a4: d503201f 	nop
ffffc02055a8: d503201f 	nop
ffffc02055ac: d503201f 	nop
ffffc02055b0: d503201f 	nop
ffffc02055b4: d503201f 	nop
ffffc02055b8: d503201f 	nop
ffffc02055bc: d503201f 	nop
ffffc02055c0: d503201f 	nop
ffffc02055c4: d503201f 	nop
ffffc02055c8: d503201f 	nop
ffffc02055cc: d503201f 	nop
ffffc02055d0: d503201f 	nop
ffffc02055d4: d503201f 	nop
ffffc02055d8: d503201f 	nop
ffffc02055dc: d503201f 	nop
ffffc02055e0: d503201f 	nop
ffffc02055e4: d503201f 	nop
ffffc02055e8: d503201f 	nop
ffffc02055ec: d503201f 	nop
ffffc02055f0: d503201f 	nop
ffffc02055f4: d503201f 	nop
ffffc02055f8: d503201f 	nop
ffffc02055fc: d503201f 	nop
ffffc0205600: 14000000 	b	0xffffc0205600 <hyp_vectors+0x600>
ffffc0205604: d503201f 	nop
ffffc0205608: d503201f 	nop
ffffc020560c: d503201f 	nop
ffffc0205610: d503201f 	nop
ffffc0205614: d503201f 	nop
ffffc0205618: d503201f 	nop
ffffc020561c: d503201f 	nop
ffffc0205620: d503201f 	nop
ffffc0205624: d503201f 	nop
ffffc0205628: d503201f 	nop
ffffc020562c: d503201f 	nop
ffffc0205630: d503201f 	nop
ffffc0205634: d503201f 	nop
ffffc0205638: d503201f 	nop
ffffc020563c: d503201f 	nop
ffffc0205640: d503201f 	nop
ffffc0205644: d503201f 	nop
ffffc0205648: d503201f 	nop
ffffc020564c: d503201f 	nop
ffffc0205650: d503201f 	nop
ffffc0205654: d503201f 	nop
ffffc0205658: d503201f 	nop
ffffc020565c: d503201f 	nop
ffffc0205660: d503201f 	nop
ffffc0205664: d503201f 	nop
ffffc0205668: d503201f 	nop
ffffc020566c: d503201f 	nop
ffffc0205670: d503201f 	nop
ffffc0205674: d503201f 	nop
ffffc0205678: d503201f 	nop
ffffc020567c: d503201f 	nop
ffffc0205680: 14000000 	b	0xffffc0205680 <hyp_vectors+0x680>
ffffc0205684: d503201f 	nop
ffffc0205688: d503201f 	nop
ffffc020568c: d503201f 	nop
ffffc0205690: d503201f 	nop
ffffc0205694: d503201f 	nop
ffffc0205698: d503201f 	nop
ffffc020569c: d503201f 	nop
ffffc02056a0: d503201f 	nop
ffffc02056a4: d503201f 	nop
ffffc02056a8: d503201f 	nop
ffffc02056ac: d503201f 	nop
ffffc02056b0: d503201f 	nop
ffffc02056b4: d503201f 	nop
ffffc02056b8: d503201f 	nop
ffffc02056bc: d503201f 	nop
ffffc02056c0: d503201f 	nop
ffffc02056c4: d503201f 	nop
ffffc02056c8: d503201f 	nop
ffffc02056cc: d503201f 	nop
ffffc02056d0: d503201f 	nop
ffffc02056d4: d503201f 	nop
ffffc02056d8: d503201f 	nop
ffffc02056dc: d503201f 	nop
ffffc02056e0: d503201f 	nop
ffffc02056e4: d503201f 	nop
ffffc02056e8: d503201f 	nop
ffffc02056ec: d503201f 	nop
ffffc02056f0: d503201f 	nop
ffffc02056f4: d503201f 	nop
ffffc02056f8: d503201f 	nop
ffffc02056fc: d503201f 	nop
ffffc0205700: 14000000 	b	0xffffc0205700 <hyp_vectors+0x700>
ffffc0205704: d503201f 	nop
ffffc0205708: d503201f 	nop
ffffc020570c: d503201f 	nop
ffffc0205710: d503201f 	nop
ffffc0205714: d503201f 	nop
ffffc0205718: d503201f 	nop
ffffc020571c: d503201f 	nop
ffffc0205720: d503201f 	nop
ffffc0205724: d503201f 	nop
ffffc0205728: d503201f 	nop
ffffc020572c: d503201f 	nop
ffffc0205730: d503201f 	nop
ffffc0205734: d503201f 	nop
ffffc0205738: d503201f 	nop
ffffc020573c: d503201f 	nop
ffffc0205740: d503201f 	nop
ffffc0205744: d503201f 	nop
ffffc0205748: d503201f 	nop
ffffc020574c: d503201f 	nop
ffffc0205750: d503201f 	nop
ffffc0205754: d503201f 	nop
ffffc0205758: d503201f 	nop
ffffc020575c: d503201f 	nop
ffffc0205760: d503201f 	nop
ffffc0205764: d503201f 	nop
ffffc0205768: d503201f 	nop
ffffc020576c: d503201f 	nop
ffffc0205770: d503201f 	nop
ffffc0205774: d503201f 	nop
ffffc0205778: d503201f 	nop
ffffc020577c: d503201f 	nop
ffffc0205780: 14000000 	b	0xffffc0205780 <hyp_vectors+0x780>
ffffc0205784: 00000000 	udf	#0

0000ffffc0205788 <$d>:
ffffc0205788: 00 00 00 c0      	.word	0xc0000000
ffffc020578c: ff ff 00 00      	.word	0x0000ffff
ffffc0205790: 00 00 20 c0      	.word	0xc0200000
ffffc0205794: ff ff 00 00      	.word	0x0000ffff
ffffc0205798: 00 48 20 c0      	.word	0xc0204800
ffffc020579c: ff ff 00 00      	.word	0x0000ffff

0000ffffc02057a0 <arch_handle_dabt>:
ffffc02057a0: a9b97bfd 	stp	x29, x30, [sp, #-112]!
ffffc02057a4: 910003fd 	mov	x29, sp
ffffc02057a8: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02057ac: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02057b0: aa0003f6 	mov	x22, x0
ffffc02057b4: a90363f7 	stp	x23, x24, [sp, #48]
ffffc02057b8: 52800035 	mov	w21, #1
ffffc02057bc: f90023f9 	str	x25, [sp, #64]
ffffc02057c0: f9400400 	ldr	x0, [x0, #8]
ffffc02057c4: 12006000 	and	w0, w0, #0x1ffffff
ffffc02057c8: d3565c14 	ubfx	x20, x0, #22, #2
ffffc02057cc: 53067c18 	lsr	w24, w0, #6
ffffc02057d0: 12000317 	and	w23, w24, #0x1
ffffc02057d4: 1ad422b5 	lsl	w21, w21, w20
ffffc02057d8: d53c6081 	mrs	x1, HPFAR_EL2
ffffc02057dc: d53c6002 	mrs	x2, FAR_EL2
ffffc02057e0: 92402c42 	and	x2, x2, #0xfff
ffffc02057e4: 6b4063ff 	cmp	wzr, w0, lsr #24
ffffc02057e8: aa012041 	orr	x1, x2, x1, lsl #8
ffffc02057ec: d2840002 	mov	x2, #8192
ffffc02057f0: f2a00022 	movk	x2, #1, lsl #16
ffffc02057f4: f9002fa1 	str	x1, [x29, #88]
ffffc02057f8: f2dfe002 	movk	x2, #65280, lsl #32
ffffc02057fc: b9501441 	ldr	w1, [x2, #4116]
ffffc0205800: 11000421 	add	w1, w1, #1
ffffc0205804: b9101441 	str	w1, [x2, #4116]
ffffc0205808: 54000820 	b.eq	0xffffc020590c <arch_handle_dabt+0x16c>
ffffc020580c: 53087c01 	lsr	w1, w0, #8
ffffc0205810: 2a402421 	orr	w1, w1, w0, lsr #9
ffffc0205814: 2a401c21 	orr	w1, w1, w0, lsr #7
ffffc0205818: 36000041 	tbz	w1, #0, 0xffffc0205820 <arch_handle_dabt+0x80>
ffffc020581c: 14000000 	b	0xffffc020581c <arch_handle_dabt+0x7c>
ffffc0205820: d3555419 	ubfx	x25, x0, #21, #1
ffffc0205824: d3505013 	ubfx	x19, x0, #16, #5
ffffc0205828: 340006f7 	cbz	w23, 0xffffc0205904 <arch_handle_dabt+0x164>
ffffc020582c: 71007e7f 	cmp	w19, #31
ffffc0205830: 54000660 	b.eq	0xffffc02058fc <arch_handle_dabt+0x15c>
ffffc0205834: d37d1260 	ubfiz	x0, x19, #3, #5
ffffc0205838: f94002c1 	ldr	x1, [x22]
ffffc020583c: f8606820 	ldr	x0, [x1, x0]
ffffc0205840: f90037a0 	str	x0, [x29, #104]
ffffc0205844: 34000179 	cbz	w25, 0xffffc0205870 <arch_handle_dabt+0xd0>
ffffc0205848: 52800101 	mov	w1, #8
ffffc020584c: 71000e9f 	cmp	w20, #3
ffffc0205850: 1ad42021 	lsl	w1, w1, w20
ffffc0205854: 540000c0 	b.eq	0xffffc020586c <arch_handle_dabt+0xcc>
ffffc0205858: 51000421 	sub	w1, w1, #1
ffffc020585c: d2800022 	mov	x2, #1
ffffc0205860: 9ac12041 	lsl	x1, x2, x1
ffffc0205864: ca010000 	eor	x0, x0, x1
ffffc0205868: cb010000 	sub	x0, x0, x1
ffffc020586c: f90037a0 	str	x0, [x29, #104]
ffffc0205870: 910163a0 	add	x0, x29, #88
ffffc0205874: 290c5fb5 	stp	w21, w23, [x29, #96]
ffffc0205878: 94000e77 	bl	0xffffc0209254 <mmio_handle_access>
ffffc020587c: 3100041f 	cmn	w0, #1
ffffc0205880: 54000320 	b.eq	0xffffc02058e4 <arch_handle_dabt+0x144>
ffffc0205884: 34000440 	cbz	w0, 0xffffc020590c <arch_handle_dabt+0x16c>
ffffc0205888: 71007e7f 	cmp	w19, #31
ffffc020588c: 1a9f07e0 	cset	w0, ne
ffffc0205890: 6a38001f 	bics	wzr, w0, w24
ffffc0205894: 54000220 	b.eq	0xffffc02058d8 <arch_handle_dabt+0x138>
ffffc0205898: f94037a0 	ldr	x0, [x29, #104]
ffffc020589c: 34000179 	cbz	w25, 0xffffc02058c8 <arch_handle_dabt+0x128>
ffffc02058a0: 52800101 	mov	w1, #8
ffffc02058a4: 71000e9f 	cmp	w20, #3
ffffc02058a8: 1ad42021 	lsl	w1, w1, w20
ffffc02058ac: 540000c0 	b.eq	0xffffc02058c4 <arch_handle_dabt+0x124>
ffffc02058b0: 51000421 	sub	w1, w1, #1
ffffc02058b4: d2800022 	mov	x2, #1
ffffc02058b8: 9ac12041 	lsl	x1, x2, x1
ffffc02058bc: ca000020 	eor	x0, x1, x0
ffffc02058c0: cb010000 	sub	x0, x0, x1
ffffc02058c4: f90037a0 	str	x0, [x29, #104]
ffffc02058c8: d37d1260 	ubfiz	x0, x19, #3, #5
ffffc02058cc: f94002c1 	ldr	x1, [x22]
ffffc02058d0: f94037a2 	ldr	x2, [x29, #104]
ffffc02058d4: f8206822 	str	x2, [x1, x0]
ffffc02058d8: aa1603e0 	mov	x0, x22
ffffc02058dc: 94000165 	bl	0xffffc0205e70 <arch_skip_instruction>
ffffc02058e0: 52800020 	mov	w0, #1
ffffc02058e4: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02058e8: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02058ec: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc02058f0: f94023f9 	ldr	x25, [sp, #64]
ffffc02058f4: a8c77bfd 	ldp	x29, x30, [sp], #112
ffffc02058f8: d65f03c0 	ret
ffffc02058fc: d2800000 	mov	x0, #0
ffffc0205900: 17ffffd0 	b	0xffffc0205840 <arch_handle_dabt+0xa0>
ffffc0205904: f90037bf 	str	xzr, [x29, #104]
ffffc0205908: 17ffffda 	b	0xffffc0205870 <arch_handle_dabt+0xd0>
ffffc020590c: f9402fa2 	ldr	x2, [x29, #88]
ffffc0205910: 710002ff 	cmp	w23, #0
ffffc0205914: d0000020 	adrp	x0, 0xffffc020b000 <arch_handle_dabt+0x18c>
ffffc0205918: d0000021 	adrp	x1, 0xffffc020b000 <arch_handle_dabt+0x190>
ffffc020591c: 912f8c00 	add	x0, x0, #3043
ffffc0205920: 912fa421 	add	x1, x1, #3049
ffffc0205924: 9a800021 	csel	x1, x1, x0, eq
ffffc0205928: 2a1503e3 	mov	w3, w21
ffffc020592c: d0000020 	adrp	x0, 0xffffc020b000 <get_cpu_parange+0x4>
ffffc0205930: 912fb800 	add	x0, x0, #3054
ffffc0205934: 940004af 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205938: 52800000 	mov	w0, #0
ffffc020593c: 17ffffea 	b	0xffffc02058e4 <arch_handle_dabt+0x144>

0000ffffc0205940 <get_cpu_parange>:
ffffc0205940: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0205944: 910003fd 	mov	x29, sp
ffffc0205948: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020594c: 900000d6 	adrp	x22, 0xffffc021d000 <get_cpu_parange+0x6c>
ffffc0205950: 910002d6 	add	x22, x22, #0
ffffc0205954: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0205958: f9001bf7 	str	x23, [sp, #48]
ffffc020595c: 52800014 	mov	w20, #0
ffffc0205960: 52800213 	mov	w19, #16
ffffc0205964: 900000b7 	adrp	x23, 0xffffc0219000 <get_cpu_parange+0x74>
ffffc0205968: f94002e0 	ldr	x0, [x23]
ffffc020596c: 91080000 	add	x0, x0, #512
ffffc0205970: b853a000 	ldur	w0, [x0, #-198]
ffffc0205974: 6b000e9f 	cmp	w20, w0, lsl #3
ffffc0205978: 54000163 	b.lo	0xffffc02059a4 <get_cpu_parange+0x64>
ffffc020597c: 7100167f 	cmp	w19, #5
ffffc0205980: 54000288 	b.hi	0xffffc02059d0 <get_cpu_parange+0x90>
ffffc0205984: d0000020 	adrp	x0, 0xffffc020b000 <get_cpu_parange+0x5c>
ffffc0205988: 91134000 	add	x0, x0, #1232
ffffc020598c: 38734800 	ldrb	w0, [x0, w19, uxtw]
ffffc0205990: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0205994: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0205998: f9401bf7 	ldr	x23, [sp, #48]
ffffc020599c: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc02059a0: d65f03c0 	ret
ffffc02059a4: 2a1403f5 	mov	w21, w20
ffffc02059a8: aa1503e0 	mov	x0, x21
ffffc02059ac: 9400091f 	bl	0xffffc0207e28 <cpu_id_valid>
ffffc02059b0: 340000c0 	cbz	w0, 0xffffc02059c8 <get_cpu_parange+0x88>
ffffc02059b4: 8b153ad5 	add	x21, x22, x21, lsl #14
ffffc02059b8: f9480ea0 	ldr	x0, [x21, #4120]
ffffc02059bc: 92400c00 	and	x0, x0, #0xf
ffffc02059c0: eb33401f 	cmp	x0, w19, uxtw
ffffc02059c4: 1a802273 	csel	w19, w19, w0, hs
ffffc02059c8: 11000694 	add	w20, w20, #1
ffffc02059cc: 17ffffe7 	b	0xffffc0205968 <get_cpu_parange+0x28>
ffffc02059d0: 52800000 	mov	w0, #0
ffffc02059d4: 17ffffef 	b	0xffffc0205990 <get_cpu_parange+0x50>

0000ffffc02059d8 <arch_init_early>:
ffffc02059d8: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc02059dc: f0000080 	adrp	x0, 0xffffc0218000 <arch_cpu_init+0x4>
ffffc02059e0: d0000063 	adrp	x3, 0xffffc0213000 <arch_init_early+0x40>
ffffc02059e4: 91000063 	add	x3, x3, #0
ffffc02059e8: 910003fd 	mov	x29, sp
ffffc02059ec: f9400400 	ldr	x0, [x0, #8]
ffffc02059f0: 52800025 	mov	w5, #1
ffffc02059f4: d280e824 	mov	x4, #1857
ffffc02059f8: d2820002 	mov	x2, #4096
ffffc02059fc: cb000063 	sub	x3, x3, x0
ffffc0205a00: f0000080 	adrp	x0, 0xffffc0218000 <arch_cpu_init+0x28>
ffffc0205a04: aa0303e1 	mov	x1, x3
ffffc0205a08: 9100e000 	add	x0, x0, #56
ffffc0205a0c: 940005df 	bl	0xffffc0207188 <paging_create>
ffffc0205a10: 35000060 	cbnz	w0, 0xffffc0205a1c <arch_init_early+0x44>
ffffc0205a14: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0205a18: 17fff6f6 	b	0xffffc02035f0 <arm_init_early>
ffffc0205a1c: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0205a20: d65f03c0 	ret

0000ffffc0205a24 <arch_cpu_init>:
ffffc0205a24: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0205a28: 910003fd 	mov	x29, sp
ffffc0205a2c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0205a30: f0000094 	adrp	x20, 0xffffc0218000 <arch_cpu_init+0x58>
ffffc0205a34: aa0003f3 	mov	x19, x0
ffffc0205a38: d0000060 	adrp	x0, 0xffffc0213000 <arch_cpu_init+0x4c>
ffffc0205a3c: f9400681 	ldr	x1, [x20, #8]
ffffc0205a40: 91000000 	add	x0, x0, #0
ffffc0205a44: cb010001 	sub	x1, x0, x1
ffffc0205a48: 91400660 	add	x0, x19, #1, lsl #12    // =4096
ffffc0205a4c: 940006a6 	bl	0xffffc02074e4 <paging_create_hvpt_link>
ffffc0205a50: 35000180 	cbnz	w0, 0xffffc0205a80 <arch_cpu_init+0x5c>
ffffc0205a54: f9480a61 	ldr	x1, [x19, #4112]
ffffc0205a58: f9400680 	ldr	x0, [x20, #8]
ffffc0205a5c: cb000020 	sub	x0, x1, x0
ffffc0205a60: 94003568 	bl	0xffffc0213000 <enable_mmu_el2>
ffffc0205a64: d2800320 	mov	x0, #25
ffffc0205a68: f2b00500 	movk	x0, #32808, lsl #16
ffffc0205a6c: d51c1100 	msr	HCR_EL2, x0
ffffc0205a70: aa1303e0 	mov	x0, x19
ffffc0205a74: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0205a78: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0205a7c: 17fff6f5 	b	0xffffc0203650 <arm_cpu_init>
ffffc0205a80: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0205a84: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0205a88: d65f03c0 	ret

0000ffffc0205a8c <arch_cpu_activate_vmm>:
ffffc0205a8c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0205a90: 900000c2 	adrp	x2, 0xffffc021d000 <arch_shutdown_self+0x14>
ffffc0205a94: 91000042 	add	x2, x2, #0
ffffc0205a98: 910003fd 	mov	x29, sp
ffffc0205a9c: f9000bf3 	str	x19, [sp, #16]
ffffc0205aa0: d2a00033 	mov	x19, #65536
ffffc0205aa4: f2dfe013 	movk	x19, #65280, lsl #32
ffffc0205aa8: b9700260 	ldr	w0, [x19, #12288]
ffffc0205aac: d3727c01 	ubfiz	x1, x0, #14, #32
ffffc0205ab0: 8b020021 	add	x1, x1, x2
ffffc0205ab4: cb010261 	sub	x1, x19, x1
ffffc0205ab8: 8b2163ff 	add	sp, sp, x1
ffffc0205abc: 52800001 	mov	w1, #0
ffffc0205ac0: 9400070d 	bl	0xffffc02076f4 <paging_map_all_per_cpu>
ffffc0205ac4: f947fe60 	ldr	x0, [x19, #4088]
ffffc0205ac8: d51c4020 	msr	ELR_EL2, x0
ffffc0205acc: d281e000 	mov	x0, #3840
ffffc0205ad0: f2a00020 	movk	x0, #1, lsl #16
ffffc0205ad4: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0205ad8: 94003562 	bl	0xffffc0213060 <vmreturn>

0000ffffc0205adc <arch_shutdown_self>:
ffffc0205adc: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0205ae0: 910003fd 	mov	x29, sp
ffffc0205ae4: f90013f5 	str	x21, [sp, #32]
ffffc0205ae8: f0000095 	adrp	x21, 0xffffc0218000 <arch_shutdown_self+0x58>
ffffc0205aec: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0205af0: aa0003f4 	mov	x20, x0
ffffc0205af4: f94006a0 	ldr	x0, [x21, #8]
ffffc0205af8: d0000073 	adrp	x19, 0xffffc0213000 <arch_shutdown_self+0x54>
ffffc0205afc: 9100e273 	add	x19, x19, #56
ffffc0205b00: cb000273 	sub	x19, x19, x0
ffffc0205b04: 91400a80 	add	x0, x20, #2, lsl #12    // =8192
ffffc0205b08: 97fff3e2 	bl	0xffffc0202a90 <irqchip_cpu_shutdown>
ffffc0205b0c: d2b00000 	mov	x0, #2147483648
ffffc0205b10: d51c1100 	msr	HCR_EL2, x0
ffffc0205b14: b26183e0 	mov	x0, #-2147483648
ffffc0205b18: d51c2140 	msr	VTCR_EL2, x0
ffffc0205b1c: d50c83df 	tlbi	vmalls12e1is
ffffc0205b20: d5033fdf 	isb
ffffc0205b24: d2800000 	mov	x0, #0
ffffc0205b28: d51c2100 	msr	VTTBR_EL2, x0
ffffc0205b2c: 52800002 	mov	w2, #0
ffffc0205b30: d2880001 	mov	x1, #16384
ffffc0205b34: aa1403e0 	mov	x0, x20
ffffc0205b38: 97fff767 	bl	0xffffc02038d4 <arm_dcaches_flush>
ffffc0205b3c: d5033b9f 	dsb	ish
ffffc0205b40: f0ffffc0 	adrp	x0, 0xffffc0200000 <arch_shutdown_self+0x50>
ffffc0205b44: f9402000 	ldr	x0, [x0, #64]
ffffc0205b48: d51cc000 	msr	VBAR_EL2, x0
ffffc0205b4c: f94006a0 	ldr	x0, [x21, #8]
ffffc0205b50: aa1303e1 	mov	x1, x19
ffffc0205b54: f94013f5 	ldr	x21, [sp, #32]
ffffc0205b58: cb000280 	sub	x0, x20, x0
ffffc0205b5c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0205b60: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0205b64: d61f0020 	br	x1

0000ffffc0205b68 <arch_cpu_restore>:
ffffc0205b68: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc0205b6c: 900000c2 	adrp	x2, 0xffffc021d000 <fill_trap_context+0x40>
ffffc0205b70: 91000042 	add	x2, x2, #0
ffffc0205b74: 8b020000 	add	x0, x0, x2
ffffc0205b78: f947fc02 	ldr	x2, [x0, #4088]
ffffc0205b7c: d51c4022 	msr	ELR_EL2, x2
ffffc0205b80: 93407c21 	sxtw	x1, w1
ffffc0205b84: f9078401 	str	x1, [x0, #3848]
ffffc0205b88: 17ffffd5 	b	0xffffc0205adc <arch_shutdown_self>

0000ffffc0205b8c <fill_trap_context>:
ffffc0205b8c: d53c4002 	mrs	x2, SPSR_EL2
ffffc0205b90: f9000802 	str	x2, [x0, #16]
ffffc0205b94: d3420c42 	ubfx	x2, x2, #2, #2
ffffc0205b98: f100045f 	cmp	x2, #1
ffffc0205b9c: 54000180 	b.eq	0xffffc0205bcc <fill_trap_context+0x40>
ffffc0205ba0: b4000122 	cbz	x2, 0xffffc0205bc4 <fill_trap_context+0x38>
ffffc0205ba4: 91040023 	add	x3, x1, #256
ffffc0205ba8: f100085f 	cmp	x2, #2
ffffc0205bac: 9a9f0062 	csel	x2, x3, xzr, eq
ffffc0205bb0: f9000c02 	str	x2, [x0, #24]
ffffc0205bb4: d53c5202 	mrs	x2, ESR_EL2
ffffc0205bb8: 91002021 	add	x1, x1, #8
ffffc0205bbc: a9000801 	stp	x1, x2, [x0]
ffffc0205bc0: d65f03c0 	ret
ffffc0205bc4: d5384102 	mrs	x2, SP_EL0
ffffc0205bc8: 17fffffa 	b	0xffffc0205bb0 <fill_trap_context+0x24>
ffffc0205bcc: d53c4102 	mrs	x2, SP_EL1
ffffc0205bd0: 17fffff8 	b	0xffffc0205bb0 <fill_trap_context+0x24>

0000ffffc0205bd4 <handle_iabt>:
ffffc0205bd4: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0205bd8: 910003fd 	mov	x29, sp
ffffc0205bdc: d53c6081 	mrs	x1, HPFAR_EL2
ffffc0205be0: d53c6000 	mrs	x0, FAR_EL2
ffffc0205be4: 92402c00 	and	x0, x0, #0xfff
ffffc0205be8: aa012001 	orr	x1, x0, x1, lsl #8
ffffc0205bec: d0000020 	adrp	x0, 0xffffc020b000 <dump_regs>
ffffc0205bf0: 91303800 	add	x0, x0, #3086
ffffc0205bf4: 940003ff 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205bf8: 12800000 	mov	w0, #-1
ffffc0205bfc: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0205c00: d65f03c0 	ret

0000ffffc0205c04 <dump_regs>:
ffffc0205c04: d10183ff 	sub	sp, sp, #96
ffffc0205c08: a9017bfd 	stp	x29, x30, [sp, #16]
ffffc0205c0c: 910043fd 	add	x29, sp, #16
ffffc0205c10: a90253f3 	stp	x19, x20, [sp, #32]
ffffc0205c14: aa0003f4 	mov	x20, x0
ffffc0205c18: a9035bf5 	stp	x21, x22, [sp, #48]
ffffc0205c1c: a90463f7 	stp	x23, x24, [sp, #64]
ffffc0205c20: f9002bf9 	str	x25, [sp, #80]
ffffc0205c24: d53c4021 	mrs	x1, ELR_EL2
ffffc0205c28: a9401800 	ldp	x0, x6, [x0]
ffffc0205c2c: f0000038 	adrp	x24, 0xffffc020c000 <dump_regs+0x44>
ffffc0205c30: a9411683 	ldp	x3, x5, [x20, #16]
ffffc0205c34: 924060c2 	and	x2, x6, #0x1ffffff
ffffc0205c38: f90003e2 	str	x2, [sp]
ffffc0205c3c: d0000039 	adrp	x25, 0xffffc020b000 <dump_regs+0x50>
ffffc0205c40: d0000036 	adrp	x22, 0xffffc020b000 <dump_regs+0x54>
ffffc0205c44: f0000037 	adrp	x23, 0xffffc020c000 <dump_regs+0x5c>
ffffc0205c48: f9407802 	ldr	x2, [x0, #240]
ffffc0205c4c: d0000035 	adrp	x21, 0xffffc020b000 <dump_regs+0x60>
ffffc0205c50: d35964c7 	ubfx	x7, x6, #25, #1
ffffc0205c54: d0000020 	adrp	x0, 0xffffc020b000 <dump_regs+0x68>
ffffc0205c58: d2800013 	mov	x19, #0
ffffc0205c5c: 91013718 	add	x24, x24, #77
ffffc0205c60: 91344b39 	add	x25, x25, #3346
ffffc0205c64: 913446d6 	add	x22, x22, #3345
ffffc0205c68: 910132f7 	add	x23, x23, #76
ffffc0205c6c: 913232b5 	add	x21, x21, #3212
ffffc0205c70: 531a7cc6 	lsr	w6, w6, #26
ffffc0205c74: d3420c64 	ubfx	x4, x3, #2, #2
ffffc0205c78: 9130c400 	add	x0, x0, #3121
ffffc0205c7c: 940003dd 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205c80: 12001e60 	and	w0, w19, #0xff
ffffc0205c84: 52800062 	mov	w2, #3
ffffc0205c88: f9400283 	ldr	x3, [x20]
ffffc0205c8c: f1002a7f 	cmp	x19, #10
ffffc0205c90: 9a983321 	csel	x1, x25, x24, lo
ffffc0205c94: 1ac20802 	udiv	w2, w0, w2
ffffc0205c98: f8737863 	ldr	x3, [x3, x19, lsl #3]
ffffc0205c9c: 0b020442 	add	w2, w2, w2, lsl #1
ffffc0205ca0: 4b020000 	sub	w0, w0, w2
ffffc0205ca4: 2a1303e2 	mov	w2, w19
ffffc0205ca8: 12001c00 	and	w0, w0, #0xff
ffffc0205cac: 91000673 	add	x19, x19, #1
ffffc0205cb0: 7100081f 	cmp	w0, #2
ffffc0205cb4: aa1503e0 	mov	x0, x21
ffffc0205cb8: 9a9602e4 	csel	x4, x23, x22, eq
ffffc0205cbc: 940003cd 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205cc0: f1007a7f 	cmp	x19, #30
ffffc0205cc4: 54fffde1 	b.ne	0xffffc0205c80 <dump_regs+0x7c>
ffffc0205cc8: a9417bfd 	ldp	x29, x30, [sp, #16]
ffffc0205ccc: f0000020 	adrp	x0, 0xffffc020c000 <dump_regs+0xe4>
ffffc0205cd0: a94253f3 	ldp	x19, x20, [sp, #32]
ffffc0205cd4: 91013000 	add	x0, x0, #76
ffffc0205cd8: a9435bf5 	ldp	x21, x22, [sp, #48]
ffffc0205cdc: a94463f7 	ldp	x23, x24, [sp, #64]
ffffc0205ce0: f9402bf9 	ldr	x25, [sp, #80]
ffffc0205ce4: 910183ff 	add	sp, sp, #96
ffffc0205ce8: 140003c2 	b	0xffffc0206bf0 <panic_printk>

0000ffffc0205cec <handle_hvc>:
ffffc0205cec: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0205cf0: d2894901 	mov	x1, #19016
ffffc0205cf4: 910003fd 	mov	x29, sp
ffffc0205cf8: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0205cfc: a9400013 	ldp	x19, x0, [x0]
ffffc0205d00: 92406000 	and	x0, x0, #0x1ffffff
ffffc0205d04: eb01001f 	cmp	x0, x1
ffffc0205d08: f9400274 	ldr	x20, [x19]
ffffc0205d0c: 54000281 	b.ne	0xffffc0205d5c <handle_hvc+0x70>
ffffc0205d10: a9408a61 	ldp	x1, x2, [x19, #8]
ffffc0205d14: aa1403e0 	mov	x0, x20
ffffc0205d18: 9400095b 	bl	0xffffc0208284 <hypercall>
ffffc0205d1c: f9000260 	str	x0, [x19]
ffffc0205d20: b50001b4 	cbnz	x20, 0xffffc0205d54 <handle_hvc+0x68>
ffffc0205d24: b5000180 	cbnz	x0, 0xffffc0205d54 <handle_hvc+0x68>
ffffc0205d28: d2840013 	mov	x19, #8192
ffffc0205d2c: 52800021 	mov	w1, #1
ffffc0205d30: f2a00033 	movk	x19, #1, lsl #16
ffffc0205d34: f2dfe013 	movk	x19, #65280, lsl #32
ffffc0205d38: b9500260 	ldr	w0, [x19, #4096]
ffffc0205d3c: 9400066e 	bl	0xffffc02076f4 <paging_map_all_per_cpu>
ffffc0205d40: b9500261 	ldr	w1, [x19, #4096]
ffffc0205d44: 900000c0 	adrp	x0, 0xffffc021d000 <arch_dump_exit+0x38>
ffffc0205d48: 91000000 	add	x0, x0, #0
ffffc0205d4c: 8b013800 	add	x0, x0, x1, lsl #14
ffffc0205d50: 97ffff63 	bl	0xffffc0205adc <arch_shutdown_self>
ffffc0205d54: 52800020 	mov	w0, #1
ffffc0205d58: 14000002 	b	0xffffc0205d60 <handle_hvc+0x74>
ffffc0205d5c: 12800000 	mov	w0, #-1
ffffc0205d60: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0205d64: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0205d68: d65f03c0 	ret

0000ffffc0205d6c <arch_dump_exit>:
ffffc0205d6c: a9b97bfd 	stp	x29, x30, [sp, #-112]!
ffffc0205d70: aa0103e4 	mov	x4, x1
ffffc0205d74: aa0003e1 	mov	x1, x0
ffffc0205d78: 910003fd 	mov	x29, sp
ffffc0205d7c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0205d80: 910143a0 	add	x0, x29, #80
ffffc0205d84: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0205d88: d2820014 	mov	x20, #4096
ffffc0205d8c: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0205d90: f2a00034 	movk	x20, #1, lsl #16
ffffc0205d94: f90023f9 	str	x25, [sp, #64]
ffffc0205d98: 97ffff7d 	bl	0xffffc0205b8c <fill_trap_context>
ffffc0205d9c: aa0403e1 	mov	x1, x4
ffffc0205da0: d0000020 	adrp	x0, 0xffffc020b000 <arch_dump_exit+0x4c>
ffffc0205da4: 91327000 	add	x0, x0, #3228
ffffc0205da8: 94000392 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205dac: 910143a0 	add	x0, x29, #80
ffffc0205db0: f2dfe014 	movk	x20, #65280, lsl #32
ffffc0205db4: 97ffff94 	bl	0xffffc0205c04 <dump_regs>
ffffc0205db8: d0000020 	adrp	x0, 0xffffc020b000 <arch_dump_exit+0x64>
ffffc0205dbc: 91330400 	add	x0, x0, #3265
ffffc0205dc0: 9400038c 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205dc4: f94037b5 	ldr	x21, [x29, #104]
ffffc0205dc8: eb1402bf 	cmp	x21, x20
ffffc0205dcc: 540003a2 	b.hs	0xffffc0205e40 <arch_dump_exit+0xd4>
ffffc0205dd0: d0000036 	adrp	x22, 0xffffc020b000 <arch_dump_exit+0x7c>
ffffc0205dd4: d0000037 	adrp	x23, 0xffffc020b000 <arch_dump_exit+0x80>
ffffc0205dd8: 927beab3 	and	x19, x21, #0xffffffffffffffe0
ffffc0205ddc: 9133eed6 	add	x22, x22, #3323
ffffc0205de0: 91342af7 	add	x23, x23, #3338
ffffc0205de4: d0000038 	adrp	x24, 0xffffc020b000 <arch_dump_exit+0x90>
ffffc0205de8: aa1403e2 	mov	x2, x20
ffffc0205dec: aa1503e1 	mov	x1, x21
ffffc0205df0: d0000020 	adrp	x0, 0xffffc020b000 <arch_dump_exit+0x9c>
ffffc0205df4: 91339000 	add	x0, x0, #3300
ffffc0205df8: 9400034b 	bl	0xffffc0206b24 <printk>
ffffc0205dfc: 91341319 	add	x25, x24, #3332
ffffc0205e00: 927b2a61 	and	x1, x19, #0xffe0
ffffc0205e04: aa1603e0 	mov	x0, x22
ffffc0205e08: 94000347 	bl	0xffffc0206b24 <printk>
ffffc0205e0c: eb1302bf 	cmp	x21, x19
ffffc0205e10: 54000248 	b.hi	0xffffc0205e58 <arch_dump_exit+0xec>
ffffc0205e14: b9400261 	ldr	w1, [x19]
ffffc0205e18: aa1903e0 	mov	x0, x25
ffffc0205e1c: 94000342 	bl	0xffffc0206b24 <printk>
ffffc0205e20: 91001273 	add	x19, x19, #4
ffffc0205e24: f240127f 	tst	x19, #0x1f
ffffc0205e28: 540001e0 	b.eq	0xffffc0205e64 <arch_dump_exit+0xf8>
ffffc0205e2c: eb13029f 	cmp	x20, x19
ffffc0205e30: 54fffee8 	b.hi	0xffffc0205e0c <arch_dump_exit+0xa0>
ffffc0205e34: f0000020 	adrp	x0, 0xffffc020c000 <arch_dump_exit+0xe4>
ffffc0205e38: 91013000 	add	x0, x0, #76
ffffc0205e3c: 9400033a 	bl	0xffffc0206b24 <printk>
ffffc0205e40: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0205e44: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0205e48: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0205e4c: f94023f9 	ldr	x25, [sp, #64]
ffffc0205e50: a8c77bfd 	ldp	x29, x30, [sp], #112
ffffc0205e54: d65f03c0 	ret
ffffc0205e58: aa1703e0 	mov	x0, x23
ffffc0205e5c: 94000332 	bl	0xffffc0206b24 <printk>
ffffc0205e60: 17fffff0 	b	0xffffc0205e20 <arch_dump_exit+0xb4>
ffffc0205e64: eb13029f 	cmp	x20, x19
ffffc0205e68: 54fffca8 	b.hi	0xffffc0205dfc <arch_dump_exit+0x90>
ffffc0205e6c: 17fffff2 	b	0xffffc0205e34 <arch_dump_exit+0xc8>

0000ffffc0205e70 <arch_skip_instruction>:
ffffc0205e70: d53c4021 	mrs	x1, ELR_EL2
ffffc0205e74: f9400400 	ldr	x0, [x0, #8]
ffffc0205e78: d2800042 	mov	x2, #2
ffffc0205e7c: f267001f 	tst	x0, #0x2000000
ffffc0205e80: d2800080 	mov	x0, #4
ffffc0205e84: 9a821000 	csel	x0, x0, x2, ne
ffffc0205e88: 8b010000 	add	x0, x0, x1
ffffc0205e8c: d51c4020 	msr	ELR_EL2, x0
ffffc0205e90: d65f03c0 	ret

0000ffffc0205e94 <handle_sysreg>:
ffffc0205e94: f9400401 	ldr	x1, [x0, #8]
ffffc0205e98: 360000e1 	tbz	w1, #0, 0xffffc0205eb4 <handle_sysreg+0x20>
ffffc0205e9c: 52800000 	mov	w0, #0
ffffc0205ea0: d65f03c0 	ret
ffffc0205ea4: 52800000 	mov	w0, #0
ffffc0205ea8: f9400bf3 	ldr	x19, [sp, #16]
ffffc0205eac: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0205eb0: d65f03c0 	ret
ffffc0205eb4: 927f5022 	and	x2, x1, #0x3ffffe
ffffc0205eb8: d28602c3 	mov	x3, #12310
ffffc0205ebc: 9276e842 	and	x2, x2, #0xfffffffffffffc1f
ffffc0205ec0: f2a00743 	movk	x3, #58, lsl #16
ffffc0205ec4: eb03005f 	cmp	x2, x3
ffffc0205ec8: 54fffea1 	b.ne	0xffffc0205e9c <handle_sysreg+0x8>
ffffc0205ecc: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0205ed0: 53052421 	ubfx	w1, w1, #5, #5
ffffc0205ed4: 71007c3f 	cmp	w1, #31
ffffc0205ed8: 910003fd 	mov	x29, sp
ffffc0205edc: f9000bf3 	str	x19, [sp, #16]
ffffc0205ee0: 54000180 	b.eq	0xffffc0205f10 <handle_sysreg+0x7c>
ffffc0205ee4: d37d1021 	ubfiz	x1, x1, #3, #5
ffffc0205ee8: f9400002 	ldr	x2, [x0]
ffffc0205eec: f8616841 	ldr	x1, [x2, x1]
ffffc0205ef0: aa0003f3 	mov	x19, x0
ffffc0205ef4: aa0103e0 	mov	x0, x1
ffffc0205ef8: 97fff099 	bl	0xffffc020215c <gicv3_handle_sgir_write>
ffffc0205efc: 34fffd40 	cbz	w0, 0xffffc0205ea4 <handle_sysreg+0x10>
ffffc0205f00: aa1303e0 	mov	x0, x19
ffffc0205f04: 97ffffdb 	bl	0xffffc0205e70 <arch_skip_instruction>
ffffc0205f08: 52800020 	mov	w0, #1
ffffc0205f0c: 17ffffe7 	b	0xffffc0205ea8 <handle_sysreg+0x14>
ffffc0205f10: d2800001 	mov	x1, #0
ffffc0205f14: 17fffff7 	b	0xffffc0205ef0 <handle_sysreg+0x5c>

0000ffffc0205f18 <arch_handle_exit>:
ffffc0205f18: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0205f1c: d2840002 	mov	x2, #8192
ffffc0205f20: f2a00022 	movk	x2, #1, lsl #16
ffffc0205f24: 910003fd 	mov	x29, sp
ffffc0205f28: f9000bf3 	str	x19, [sp, #16]
ffffc0205f2c: f2dfe002 	movk	x2, #65280, lsl #32
ffffc0205f30: aa0003f3 	mov	x19, x0
ffffc0205f34: b9501041 	ldr	w1, [x2, #4112]
ffffc0205f38: 11000421 	add	w1, w1, #1
ffffc0205f3c: b9101041 	str	w1, [x2, #4112]
ffffc0205f40: f9400001 	ldr	x1, [x0]
ffffc0205f44: f100043f 	cmp	x1, #1
ffffc0205f48: 540000e0 	b.eq	0xffffc0205f64 <arch_handle_exit+0x4c>
ffffc0205f4c: b4000501 	cbz	x1, 0xffffc0205fec <arch_handle_exit+0xd4>
ffffc0205f50: f100083f 	cmp	x1, #2
ffffc0205f54: 54000541 	b.ne	0xffffc0205ffc <arch_handle_exit+0xe4>
ffffc0205f58: 97fff11f 	bl	0xffffc02023d4 <irqchip_handle_irq>
ffffc0205f5c: aa1303e0 	mov	x0, x19
ffffc0205f60: 94003440 	bl	0xffffc0213060 <vmreturn>
ffffc0205f64: aa0003e1 	mov	x1, x0
ffffc0205f68: 910083a0 	add	x0, x29, #32
ffffc0205f6c: 97ffff08 	bl	0xffffc0205b8c <fill_trap_context>
ffffc0205f70: f94017a1 	ldr	x1, [x29, #40]
ffffc0205f74: d0000020 	adrp	x0, 0xffffc020b000 <arch_handle_exit+0x74>
ffffc0205f78: 91136000 	add	x0, x0, #1240
ffffc0205f7c: 531a7c21 	lsr	w1, w1, #26
ffffc0205f80: f8617801 	ldr	x1, [x0, x1, lsl #3]
ffffc0205f84: b4000301 	cbz	x1, 0xffffc0205fe4 <arch_handle_exit+0xcc>
ffffc0205f88: 910083a0 	add	x0, x29, #32
ffffc0205f8c: d63f0020 	blr	x1
ffffc0205f90: 11000401 	add	w1, w0, #1
ffffc0205f94: 7100043f 	cmp	w1, #1
ffffc0205f98: 54fffe28 	b.hi	0xffffc0205f5c <arch_handle_exit+0x44>
ffffc0205f9c: f94017a2 	ldr	x2, [x29, #40]
ffffc0205fa0: d0000021 	adrp	x1, 0xffffc020b000 <arch_handle_exit+0xa0>
ffffc0205fa4: 7100001f 	cmp	w0, #0
ffffc0205fa8: d0000023 	adrp	x3, 0xffffc020b000 <arch_handle_exit+0xa8>
ffffc0205fac: 91348c20 	add	x0, x1, #3363
ffffc0205fb0: 91345061 	add	x1, x3, #3348
ffffc0205fb4: 9a800021 	csel	x1, x1, x0, eq
ffffc0205fb8: 531a7c42 	lsr	w2, w2, #26
ffffc0205fbc: d0000020 	adrp	x0, 0xffffc020b000 <arch_handle_exit+0xbc>
ffffc0205fc0: 9134d000 	add	x0, x0, #3380
ffffc0205fc4: 9400030b 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205fc8: d0000020 	adrp	x0, 0xffffc020b000 <arch_handle_exit+0xc8>
ffffc0205fcc: 91356c00 	add	x0, x0, #3419
ffffc0205fd0: 94000308 	bl	0xffffc0206bf0 <panic_printk>
ffffc0205fd4: 910083a0 	add	x0, x29, #32
ffffc0205fd8: 97ffff0b 	bl	0xffffc0205c04 <dump_regs>
ffffc0205fdc: 94000b6b 	bl	0xffffc0208d88 <panic_park>
ffffc0205fe0: 17ffffdf 	b	0xffffc0205f5c <arch_handle_exit+0x44>
ffffc0205fe4: 52800000 	mov	w0, #0
ffffc0205fe8: 17ffffea 	b	0xffffc0205f90 <arch_handle_exit+0x78>
ffffc0205fec: d0000021 	adrp	x1, 0xffffc020b000 <arch_handle_exit+0xec>
ffffc0205ff0: 9135e421 	add	x1, x1, #3449
ffffc0205ff4: 97ffff5e 	bl	0xffffc0205d6c <arch_dump_exit>
ffffc0205ff8: 94000b47 	bl	0xffffc0208d14 <panic_stop>
ffffc0205ffc: d0000021 	adrp	x1, 0xffffc020b000 <spin_lock.constprop.1+0xc>
ffffc0206000: 91365021 	add	x1, x1, #3476
ffffc0206004: 17fffffc 	b	0xffffc0205ff4 <arch_handle_exit+0xdc>

0000ffffc0206008 <spin_lock.constprop.1>:
ffffc0206008: 90000080 	adrp	x0, 0xffffc0216000 <entry+0x4>
ffffc020600c: 91000000 	add	x0, x0, #0
ffffc0206010: f9800011 	prfm	pstl1strm, [x0]
ffffc0206014: 885ffc01 	ldaxr	w1, [x0]
ffffc0206018: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc020601c: 88037c02 	stxr	w3, w2, [x0]
ffffc0206020: 35ffffa3 	cbnz	w3, 0xffffc0206014 <spin_lock.constprop.1+0xc>
ffffc0206024: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc0206028: 340000c2 	cbz	w2, 0xffffc0206040 <spin_lock.constprop.1+0x38>
ffffc020602c: d50320bf 	sevl
ffffc0206030: d503205f 	wfe
ffffc0206034: 485ffc03 	ldaxrh	w3, [x0]
ffffc0206038: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc020603c: 35ffffa2 	cbnz	w2, 0xffffc0206030 <spin_lock.constprop.1+0x28>
ffffc0206040: d65f03c0 	ret

0000ffffc0206044 <entry>:
ffffc0206044: a9b87bfd 	stp	x29, x30, [sp, #-128]!
ffffc0206048: 910003fd 	mov	x29, sp
ffffc020604c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0206050: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0206054: 2a0003f6 	mov	w22, w0
ffffc0206058: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020605c: aa0103f5 	mov	x21, x1
ffffc0206060: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc0206064: f9002bfb 	str	x27, [sp, #80]
ffffc0206068: b9300020 	str	w0, [x1, #12288]
ffffc020606c: 97ffffe7 	bl	0xffffc0206008 <spin_lock.constprop.1>
ffffc0206070: d5033bbf 	dmb	ish
ffffc0206074: 90000093 	adrp	x19, 0xffffc0216000 <entry+0x70>
ffffc0206078: 91000274 	add	x20, x19, #0
ffffc020607c: b9400680 	ldr	w0, [x20, #4]
ffffc0206080: 11000400 	add	w0, w0, #1
ffffc0206084: b9000680 	str	w0, [x20, #4]
ffffc0206088: 79400260 	ldrh	w0, [x19]
ffffc020608c: 11000400 	add	w0, w0, #1
ffffc0206090: 489ffe80 	stlrh	w0, [x20]
ffffc0206094: d0ffffd8 	adrp	x24, 0xffffc0200000 <entry+0x38>
ffffc0206098: 91000304 	add	x4, x24, #0
ffffc020609c: b9400680 	ldr	w0, [x20, #4]
ffffc02060a0: b9403481 	ldr	w1, [x4, #52]
ffffc02060a4: 6b00003f 	cmp	w1, w0
ffffc02060a8: 540013a8 	b.hi	0xffffc020631c <entry+0x2d8>
ffffc02060ac: 97ffffd7 	bl	0xffffc0206008 <spin_lock.constprop.1>
ffffc02060b0: 90000042 	adrp	x2, 0xffffc020e000 <entry+0x8c>
ffffc02060b4: b9415040 	ldr	w0, [x2, #336]
ffffc02060b8: 3100041f 	cmn	w0, #1
ffffc02060bc: 540015e1 	b.ne	0xffffc0206378 <entry+0x334>
ffffc02060c0: f9400481 	ldr	x1, [x4, #8]
ffffc02060c4: d2b80403 	mov	x3, #3223322624
ffffc02060c8: b9403080 	ldr	w0, [x4, #48]
ffffc02060cc: f2dfffe3 	movk	x3, #65535, lsl #32
ffffc02060d0: 8b030021 	add	x1, x1, x3
ffffc02060d4: b9015056 	str	w22, [x2, #336]
ffffc02060d8: 52880002 	mov	w2, #16384
ffffc02060dc: f000009a 	adrp	x26, 0xffffc0219000 <entry+0xe4>
ffffc02060e0: d0000097 	adrp	x23, 0xffffc0218000 <entry+0xe4>
ffffc02060e4: 9ba20400 	umaddl	x0, w0, w2, x1
ffffc02060e8: f9000340 	str	x0, [x26]
ffffc02060ec: b9400800 	ldr	w0, [x0, #8]
ffffc02060f0: 12000000 	and	w0, w0, #0x1
ffffc02060f4: b90002e0 	str	w0, [x23]
ffffc02060f8: 97ffead4 	bl	0xffffc0200c48 <arch_dbg_write_init>
ffffc02060fc: 2a1603e2 	mov	w2, w22
ffffc0206100: b0000021 	adrp	x1, 0xffffc020b000 <entry+0xd0>
ffffc0206104: b0000020 	adrp	x0, 0xffffc020b000 <entry+0xd4>
ffffc0206108: 91367c21 	add	x1, x1, #3487
ffffc020610c: 9136e400 	add	x0, x0, #3513
ffffc0206110: 94000285 	bl	0xffffc0206b24 <printk>
ffffc0206114: d0ffffc1 	adrp	x1, 0xffffc0200000 <entry+0xb8>
ffffc0206118: b0000020 	adrp	x0, 0xffffc020b000 <entry+0xe8>
ffffc020611c: 91200021 	add	x1, x1, #2048
ffffc0206120: 9137a800 	add	x0, x0, #3562
ffffc0206124: 94000280 	bl	0xffffc0206b24 <printk>
ffffc0206128: 94000582 	bl	0xffffc0207730 <paging_init>
ffffc020612c: b9000a80 	str	w0, [x20, #8]
ffffc0206130: b9400a80 	ldr	w0, [x20, #8]
ffffc0206134: 35000480 	cbnz	w0, 0xffffc02061c4 <entry+0x180>
ffffc0206138: f9400341 	ldr	x1, [x26]
ffffc020613c: 900000b9 	adrp	x25, 0xffffc021a000 <entry+0x148>
ffffc0206140: 91000320 	add	x0, x25, #0
ffffc0206144: 91042821 	add	x1, x1, #266
ffffc0206148: f9085001 	str	x1, [x0, #4256]
ffffc020614c: 94000798 	bl	0xffffc0207fac <cell_init>
ffffc0206150: b9000a80 	str	w0, [x20, #8]
ffffc0206154: b9400a80 	ldr	w0, [x20, #8]
ffffc0206158: 35000360 	cbnz	w0, 0xffffc02061c4 <entry+0x180>
ffffc020615c: 97fffe1f 	bl	0xffffc02059d8 <arch_init_early>
ffffc0206160: b9000a80 	str	w0, [x20, #8]
ffffc0206164: b9400a80 	ldr	w0, [x20, #8]
ffffc0206168: 350002e0 	cbnz	w0, 0xffffc02061c4 <entry+0x180>
ffffc020616c: f9400341 	ldr	x1, [x26]
ffffc0206170: d000007b 	adrp	x27, 0xffffc0214000 <entry+0x164>
ffffc0206174: b000009a 	adrp	x26, 0xffffc0217000 <entry+0x174>
ffffc0206178: 9100037b 	add	x27, x27, #0
ffffc020617c: 9100035a 	add	x26, x26, #0
ffffc0206180: f840c020 	ldur	x0, [x1, #12]
ffffc0206184: f841c034 	ldur	x20, [x1, #28]
ffffc0206188: f90037a0 	str	x0, [x29, #104]
ffffc020618c: 8b140014 	add	x20, x0, x20
ffffc0206190: d2820000 	mov	x0, #4096
ffffc0206194: f9003ba0 	str	x0, [x29, #112]
ffffc0206198: d2800020 	mov	x0, #1
ffffc020619c: f9003fa0 	str	x0, [x29, #120]
ffffc02061a0: f94037a1 	ldr	x1, [x29, #104]
ffffc02061a4: eb01029f 	cmp	x20, x1
ffffc02061a8: 54000bc8 	b.hi	0xffffc0206320 <entry+0x2dc>
ffffc02061ac: b0000020 	adrp	x0, 0xffffc020b000 <entry+0x17c>
ffffc02061b0: 9137f400 	add	x0, x0, #3581
ffffc02061b4: 940005fe 	bl	0xffffc02079ac <paging_dump_stats>
ffffc02061b8: b0000020 	adrp	x0, 0xffffc020b000 <entry+0x188>
ffffc02061bc: 91383c00 	add	x0, x0, #3599
ffffc02061c0: 94000259 	bl	0xffffc0206b24 <printk>
ffffc02061c4: 52800037 	mov	w23, #1
ffffc02061c8: 91000260 	add	x0, x19, #0
ffffc02061cc: b9400800 	ldr	w0, [x0, #8]
ffffc02061d0: 350007c0 	cbnz	w0, 0xffffc02062c8 <entry+0x284>
ffffc02061d4: b97002a1 	ldr	w1, [x21, #12288]
ffffc02061d8: b0000020 	adrp	x0, 0xffffc020b000 <entry+0x1a8>
ffffc02061dc: 9138a400 	add	x0, x0, #3625
ffffc02061e0: 94000251 	bl	0xffffc0206b24 <printk>
ffffc02061e4: b97002a0 	ldr	w0, [x21, #12288]
ffffc02061e8: 94000710 	bl	0xffffc0207e28 <cpu_id_valid>
ffffc02061ec: 34000e20 	cbz	w0, 0xffffc02063b0 <entry+0x36c>
ffffc02061f0: 900000a0 	adrp	x0, 0xffffc021a000 <entry+0x1fc>
ffffc02061f4: 91000000 	add	x0, x0, #0
ffffc02061f8: f91806a0 	str	x0, [x21, #12296]
ffffc02061fc: 52800020 	mov	w0, #1
ffffc0206200: b91002a0 	str	w0, [x21, #4096]
ffffc0206204: d0000080 	adrp	x0, 0xffffc0218000 <entry+0x208>
ffffc0206208: 914006b9 	add	x25, x21, #1, lsl #12   // =4096
ffffc020620c: d2b80401 	mov	x1, #3223322624
ffffc0206210: f9402000 	ldr	x0, [x0, #64]
ffffc0206214: f2dfffe1 	movk	x1, #65535, lsl #32
ffffc0206218: f90806a0 	str	x0, [x21, #4104]
ffffc020621c: 91400aa0 	add	x0, x21, #2, lsl #12    // =8192
ffffc0206220: f9080aa0 	str	x0, [x21, #4112]
ffffc0206224: aa1903e0 	mov	x0, x25
ffffc0206228: 940004af 	bl	0xffffc02074e4 <paging_create_hvpt_link>
ffffc020622c: 2a0003f4 	mov	w20, w0
ffffc0206230: 35000b40 	cbnz	w0, 0xffffc0206398 <entry+0x354>
ffffc0206234: f0000080 	adrp	x0, 0xffffc0219000 <entry+0x23c>
ffffc0206238: f9400000 	ldr	x0, [x0]
ffffc020623c: 79407400 	ldrh	w0, [x0, #58]
ffffc0206240: 37000a00 	tbnz	w0, #0, 0xffffc0206380 <entry+0x33c>
ffffc0206244: d0000080 	adrp	x0, 0xffffc0218000 <entry+0x248>
ffffc0206248: d2a00023 	mov	x3, #65536
ffffc020624c: 52800025 	mov	w5, #1
ffffc0206250: d280e824 	mov	x4, #1857
ffffc0206254: f9400401 	ldr	x1, [x0, #8]
ffffc0206258: f2dfe003 	movk	x3, #65280, lsl #32
ffffc020625c: d2880002 	mov	x2, #16384
ffffc0206260: aa1903e0 	mov	x0, x25
ffffc0206264: cb0102a1 	sub	x1, x21, x1
ffffc0206268: 940003c8 	bl	0xffffc0207188 <paging_create>
ffffc020626c: 2a0003f4 	mov	w20, w0
ffffc0206270: 35000940 	cbnz	w0, 0xffffc0206398 <entry+0x354>
ffffc0206274: aa1503e0 	mov	x0, x21
ffffc0206278: 97fffdeb 	bl	0xffffc0205a24 <arch_cpu_init>
ffffc020627c: 2a0003f4 	mov	w20, w0
ffffc0206280: 350008c0 	cbnz	w0, 0xffffc0206398 <entry+0x354>
ffffc0206284: 52800025 	mov	w5, #1
ffffc0206288: d2800004 	mov	x4, #0
ffffc020628c: d2dfe003 	mov	x3, #280375465082880
ffffc0206290: d2a00022 	mov	x2, #65536
ffffc0206294: d2800001 	mov	x1, #0
ffffc0206298: aa1903e0 	mov	x0, x25
ffffc020629c: 940003bb 	bl	0xffffc0207188 <paging_create>
ffffc02062a0: 2a0003f4 	mov	w20, w0
ffffc02062a4: 350007a0 	cbnz	w0, 0xffffc0206398 <entry+0x354>
ffffc02062a8: b0000020 	adrp	x0, 0xffffc020b000 <entry+0x278>
ffffc02062ac: 9138d400 	add	x0, x0, #3637
ffffc02062b0: 9400021d 	bl	0xffffc0206b24 <printk>
ffffc02062b4: d5033bbf 	dmb	ish
ffffc02062b8: 91000261 	add	x1, x19, #0
ffffc02062bc: b9400c20 	ldr	w0, [x1, #12]
ffffc02062c0: 11000400 	add	w0, w0, #1
ffffc02062c4: b9000c20 	str	w0, [x1, #12]
ffffc02062c8: 79400260 	ldrh	w0, [x19]
ffffc02062cc: 91000261 	add	x1, x19, #0
ffffc02062d0: 11000400 	add	w0, w0, #1
ffffc02062d4: 489ffc20 	stlrh	w0, [x1]
ffffc02062d8: aa0103e0 	mov	x0, x1
ffffc02062dc: 91000303 	add	x3, x24, #0
ffffc02062e0: b9400801 	ldr	w1, [x0, #8]
ffffc02062e4: 350000a1 	cbnz	w1, 0xffffc02062f8 <entry+0x2b4>
ffffc02062e8: b9400c01 	ldr	w1, [x0, #12]
ffffc02062ec: b9403462 	ldr	w2, [x3, #52]
ffffc02062f0: 6b01005f 	cmp	w2, w1
ffffc02062f4: 54000628 	b.hi	0xffffc02063b8 <entry+0x374>
ffffc02062f8: 91000260 	add	x0, x19, #0
ffffc02062fc: b9400800 	ldr	w0, [x0, #8]
ffffc0206300: 35001200 	cbnz	w0, 0xffffc0206540 <entry+0x4fc>
ffffc0206304: 340011f7 	cbz	w23, 0xffffc0206540 <entry+0x4fc>
ffffc0206308: 900000b4 	adrp	x20, 0xffffc021a000 <entry+0x314>
ffffc020630c: 52800015 	mov	w21, #0
ffffc0206310: 12800000 	mov	w0, #-1
ffffc0206314: 91000299 	add	x25, x20, #0
ffffc0206318: 1400002a 	b	0xffffc02063c0 <entry+0x37c>
ffffc020631c: 17ffff60 	b	0xffffc020609c <entry+0x58>
ffffc0206320: b94002e2 	ldr	w2, [x23]
ffffc0206324: d0000080 	adrp	x0, 0xffffc0218000 <entry+0x328>
ffffc0206328: f9400400 	ldr	x0, [x0, #8]
ffffc020632c: 34000202 	cbz	w2, 0xffffc020636c <entry+0x328>
ffffc0206330: cb000362 	sub	x2, x27, x0
ffffc0206334: eb01005f 	cmp	x2, x1
ffffc0206338: 540001a1 	b.ne	0xffffc020636c <entry+0x328>
ffffc020633c: f90033a2 	str	x2, [x29, #96]
ffffc0206340: 910183a1 	add	x1, x29, #96
ffffc0206344: 91000320 	add	x0, x25, #0
ffffc0206348: 97fff2cc 	bl	0xffffc0202e78 <arch_map_memory_region>
ffffc020634c: 91000261 	add	x1, x19, #0
ffffc0206350: b9000820 	str	w0, [x1, #8]
ffffc0206354: b9400820 	ldr	w0, [x1, #8]
ffffc0206358: 35fff360 	cbnz	w0, 0xffffc02061c4 <entry+0x180>
ffffc020635c: f94037a0 	ldr	x0, [x29, #104]
ffffc0206360: 91400400 	add	x0, x0, #1, lsl #12     // =4096
ffffc0206364: f90037a0 	str	x0, [x29, #104]
ffffc0206368: 17ffff8e 	b	0xffffc02061a0 <entry+0x15c>
ffffc020636c: cb000340 	sub	x0, x26, x0
ffffc0206370: f90033a0 	str	x0, [x29, #96]
ffffc0206374: 17fffff3 	b	0xffffc0206340 <entry+0x2fc>
ffffc0206378: 52800017 	mov	w23, #0
ffffc020637c: 17ffff93 	b	0xffffc02061c8 <entry+0x184>
ffffc0206380: 91000300 	add	x0, x24, #0
ffffc0206384: f9401c01 	ldr	x1, [x0, #56]
ffffc0206388: aa1903e0 	mov	x0, x25
ffffc020638c: 94000456 	bl	0xffffc02074e4 <paging_create_hvpt_link>
ffffc0206390: 2a0003f4 	mov	w20, w0
ffffc0206394: 34fff580 	cbz	w0, 0xffffc0206244 <entry+0x200>
ffffc0206398: b0000020 	adrp	x0, 0xffffc020b000 <entry+0x368>
ffffc020639c: 9138e400 	add	x0, x0, #3641
ffffc02063a0: 940001e1 	bl	0xffffc0206b24 <printk>
ffffc02063a4: 91000260 	add	x0, x19, #0
ffffc02063a8: b9000814 	str	w20, [x0, #8]
ffffc02063ac: 17ffffc7 	b	0xffffc02062c8 <entry+0x284>
ffffc02063b0: 128002b4 	mov	w20, #-22
ffffc02063b4: 17fffff9 	b	0xffffc0206398 <entry+0x354>
ffffc02063b8: 17ffffca 	b	0xffffc02062e0 <entry+0x29c>
ffffc02063bc: 110006b5 	add	w21, w21, #1
ffffc02063c0: f9485721 	ldr	x1, [x25, #4264]
ffffc02063c4: 12800002 	mov	w2, #-1
ffffc02063c8: 94000620 	bl	0xffffc0207c48 <next_cpu>
ffffc02063cc: f9485721 	ldr	x1, [x25, #4264]
ffffc02063d0: f9400021 	ldr	x1, [x1]
ffffc02063d4: eb20403f 	cmp	x1, w0, uxtw
ffffc02063d8: 54ffff22 	b.hs	0xffffc02063bc <entry+0x378>
ffffc02063dc: 91000318 	add	x24, x24, #0
ffffc02063e0: b9403700 	ldr	w0, [x24, #52]
ffffc02063e4: 6b0002bf 	cmp	w21, w0
ffffc02063e8: 540006e0 	b.eq	0xffffc02064c4 <entry+0x480>
ffffc02063ec: 91000260 	add	x0, x19, #0
ffffc02063f0: 128002a1 	mov	w1, #-22
ffffc02063f4: b9000801 	str	w1, [x0, #8]
ffffc02063f8: 91000260 	add	x0, x19, #0
ffffc02063fc: b9400801 	ldr	w1, [x0, #8]
ffffc0206400: 35000081 	cbnz	w1, 0xffffc0206410 <entry+0x3cc>
ffffc0206404: d5033bbf 	dmb	ish
ffffc0206408: 52800021 	mov	w1, #1
ffffc020640c: b9001001 	str	w1, [x0, #16]
ffffc0206410: 91000260 	add	x0, x19, #0
ffffc0206414: b9400800 	ldr	w0, [x0, #8]
ffffc0206418: 34000980 	cbz	w0, 0xffffc0206548 <entry+0x504>
ffffc020641c: 34000057 	cbz	w23, 0xffffc0206424 <entry+0x3e0>
ffffc0206420: 94000787 	bl	0xffffc020823c <shutdown>
ffffc0206424: 91000273 	add	x19, x19, #0
ffffc0206428: 2a1603e0 	mov	w0, w22
ffffc020642c: b9400a61 	ldr	w1, [x19, #8]
ffffc0206430: 97fffdce 	bl	0xffffc0205b68 <arch_cpu_restore>
ffffc0206434: f9402bfb 	ldr	x27, [sp, #80]
ffffc0206438: b9400a60 	ldr	w0, [x19, #8]
ffffc020643c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0206440: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0206444: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0206448: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc020644c: a8c87bfd 	ldp	x29, x30, [sp], #128
ffffc0206450: d65f03c0 	ret
ffffc0206454: f94002a1 	ldr	x1, [x21]
ffffc0206458: aa1903e0 	mov	x0, x25
ffffc020645c: 940001b2 	bl	0xffffc0206b24 <printk>
ffffc0206460: f94006a0 	ldr	x0, [x21, #8]
ffffc0206464: d63f0000 	blr	x0
ffffc0206468: b9000b40 	str	w0, [x26, #8]
ffffc020646c: b9400b40 	ldr	w0, [x26, #8]
ffffc0206470: 35fffc40 	cbnz	w0, 0xffffc02063f8 <entry+0x3b4>
ffffc0206474: 9100c2b5 	add	x21, x21, #48
ffffc0206478: eb1802bf 	cmp	x21, x24
ffffc020647c: 54fffec3 	b.lo	0xffffc0206454 <entry+0x410>
ffffc0206480: 91000294 	add	x20, x20, #0
ffffc0206484: 91000279 	add	x25, x19, #0
ffffc0206488: 52800018 	mov	w24, #0
ffffc020648c: f9485295 	ldr	x21, [x20, #4256]
ffffc0206490: b94032a0 	ldr	w0, [x21, #48]
ffffc0206494: 91020000 	add	x0, x0, #128
ffffc0206498: 8b0002b5 	add	x21, x21, x0
ffffc020649c: f9485280 	ldr	x0, [x20, #4256]
ffffc02064a0: b9403400 	ldr	w0, [x0, #52]
ffffc02064a4: 6b00031f 	cmp	w24, w0
ffffc02064a8: 540001e3 	b.lo	0xffffc02064e4 <entry+0x4a0>
ffffc02064ac: aa1403e0 	mov	x0, x20
ffffc02064b0: 940006f4 	bl	0xffffc0208080 <config_commit>
ffffc02064b4: b0000020 	adrp	x0, 0xffffc020b000 <entry+0x484>
ffffc02064b8: 91396000 	add	x0, x0, #3672
ffffc02064bc: 9400053c 	bl	0xffffc02079ac <paging_dump_stats>
ffffc02064c0: 17ffffce 	b	0xffffc02063f8 <entry+0x3b4>
ffffc02064c4: 90000055 	adrp	x21, 0xffffc020e000 <entry+0x4a0>
ffffc02064c8: 90000058 	adrp	x24, 0xffffc020e000 <entry+0x4a4>
ffffc02064cc: b0000039 	adrp	x25, 0xffffc020b000 <entry+0x49c>
ffffc02064d0: 910742b5 	add	x21, x21, #464
ffffc02064d4: 9108c318 	add	x24, x24, #560
ffffc02064d8: 91390739 	add	x25, x25, #3649
ffffc02064dc: 9100027a 	add	x26, x19, #0
ffffc02064e0: 17ffffe6 	b	0xffffc0206478 <entry+0x434>
ffffc02064e4: f94006a0 	ldr	x0, [x21, #8]
ffffc02064e8: aa1503e1 	mov	x1, x21
ffffc02064ec: f2402c1f 	tst	x0, #0xfff
ffffc02064f0: 54000081 	b.ne	0xffffc0206500 <entry+0x4bc>
ffffc02064f4: f9400aa0 	ldr	x0, [x21, #16]
ffffc02064f8: f2402c1f 	tst	x0, #0xfff
ffffc02064fc: 54000120 	b.eq	0xffffc0206520 <entry+0x4dc>
ffffc0206500: aa1403e0 	mov	x0, x20
ffffc0206504: 94000bdc 	bl	0xffffc0209474 <mmio_subpage_register>
ffffc0206508: b9000b20 	str	w0, [x25, #8]
ffffc020650c: b9400b20 	ldr	w0, [x25, #8]
ffffc0206510: 35fff740 	cbnz	w0, 0xffffc02063f8 <entry+0x3b4>
ffffc0206514: 910082b5 	add	x21, x21, #32
ffffc0206518: 11000718 	add	w24, w24, #1
ffffc020651c: 17ffffe0 	b	0xffffc020649c <entry+0x458>
ffffc0206520: aa1403e0 	mov	x0, x20
ffffc0206524: 97fff255 	bl	0xffffc0202e78 <arch_map_memory_region>
ffffc0206528: 17fffff8 	b	0xffffc0206508 <entry+0x4c4>
ffffc020652c: b9400801 	ldr	w1, [x0, #8]
ffffc0206530: 35fff701 	cbnz	w1, 0xffffc0206410 <entry+0x3cc>
ffffc0206534: b9401001 	ldr	w1, [x0, #16]
ffffc0206538: 34ffffa1 	cbz	w1, 0xffffc020652c <entry+0x4e8>
ffffc020653c: 17ffffb5 	b	0xffffc0206410 <entry+0x3cc>
ffffc0206540: 91000260 	add	x0, x19, #0
ffffc0206544: 17fffffa 	b	0xffffc020652c <entry+0x4e8>
ffffc0206548: 34000097 	cbz	w23, 0xffffc0206558 <entry+0x514>
ffffc020654c: b0000020 	adrp	x0, 0xffffc020b000 <uint2str+0x4>
ffffc0206550: 9139a400 	add	x0, x0, #3689
ffffc0206554: 94000174 	bl	0xffffc0206b24 <printk>
ffffc0206558: 97fffd4d 	bl	0xffffc0205a8c <arch_cpu_activate_vmm>

0000ffffc020655c <uint2str>:
ffffc020655c: d2b13d03 	mov	x3, #2313682944
ffffc0206560: 52800285 	mov	w5, #20
ffffc0206564: f2c46083 	movk	x3, #8964, lsl #32
ffffc0206568: 52800022 	mov	w2, #1
ffffc020656c: f2f158e3 	movk	x3, #35527, lsl #48
ffffc0206570: d2800146 	mov	x6, #10
ffffc0206574: 9ac30804 	udiv	x4, x0, x3
ffffc0206578: 52000042 	eor	w2, w2, #0x1
ffffc020657c: f100009f 	cmp	x4, #0
ffffc0206580: 9b038080 	msub	x0, x4, x3, x0
ffffc0206584: 1a9f0442 	csinc	w2, w2, wzr, eq
ffffc0206588: 35000062 	cbnz	w2, 0xffffc0206594 <uint2str+0x38>
ffffc020658c: f100047f 	cmp	x3, #1
ffffc0206590: 54000141 	b.ne	0xffffc02065b8 <uint2str+0x5c>
ffffc0206594: 1100c084 	add	w4, w4, #48
ffffc0206598: 91000421 	add	x1, x1, #1
ffffc020659c: 381ff024 	sturb	w4, [x1, #-1]
ffffc02065a0: 52800002 	mov	w2, #0
ffffc02065a4: 9ac60863 	udiv	x3, x3, x6
ffffc02065a8: 710004a5 	subs	w5, w5, #1
ffffc02065ac: 54fffe41 	b.ne	0xffffc0206574 <uint2str+0x18>
ffffc02065b0: aa0103e0 	mov	x0, x1
ffffc02065b4: d65f03c0 	ret
ffffc02065b8: 52800022 	mov	w2, #1
ffffc02065bc: 17fffffa 	b	0xffffc02065a4 <uint2str+0x48>

0000ffffc02065c0 <hex2str>:
ffffc02065c0: b0000026 	adrp	x6, 0xffffc020b000 <hex2str+0x14>
ffffc02065c4: 911b60c6 	add	x6, x6, #1752
ffffc02065c8: 52800205 	mov	w5, #16
ffffc02065cc: 52800023 	mov	w3, #1
ffffc02065d0: d2e20004 	mov	x4, #1152921504606846976
ffffc02065d4: 9ac40807 	udiv	x7, x0, x4
ffffc02065d8: 52000063 	eor	w3, w3, #0x1
ffffc02065dc: f10000ff 	cmp	x7, #0
ffffc02065e0: 9b0480e0 	msub	x0, x7, x4, x0
ffffc02065e4: 1a9f0463 	csinc	w3, w3, wzr, eq
ffffc02065e8: 350000a3 	cbnz	w3, 0xffffc02065fc <hex2str+0x3c>
ffffc02065ec: f100049f 	cmp	x4, #1
ffffc02065f0: 54000060 	b.eq	0xffffc02065fc <hex2str+0x3c>
ffffc02065f4: ea04005f 	tst	x2, x4
ffffc02065f8: 54000140 	b.eq	0xffffc0206620 <hex2str+0x60>
ffffc02065fc: 386768c3 	ldrb	w3, [x6, x7]
ffffc0206600: 91000421 	add	x1, x1, #1
ffffc0206604: 381ff023 	sturb	w3, [x1, #-1]
ffffc0206608: 52800003 	mov	w3, #0
ffffc020660c: d344fc84 	lsr	x4, x4, #4
ffffc0206610: 710004a5 	subs	w5, w5, #1
ffffc0206614: 54fffe01 	b.ne	0xffffc02065d4 <hex2str+0x14>
ffffc0206618: aa0103e0 	mov	x0, x1
ffffc020661c: d65f03c0 	ret
ffffc0206620: 52800023 	mov	w3, #1
ffffc0206624: 17fffffa 	b	0xffffc020660c <hex2str+0x4c>

0000ffffc0206628 <dbg_write_stub>:
ffffc0206628: d65f03c0 	ret

0000ffffc020662c <align>:
ffffc020662c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0206630: aa0103e4 	mov	x4, x1
ffffc0206634: 910003fd 	mov	x29, sp
ffffc0206638: f9000bf3 	str	x19, [sp, #16]
ffffc020663c: aa0203f3 	mov	x19, x2
ffffc0206640: cb010002 	sub	x2, x0, x1
ffffc0206644: eb13005f 	cmp	x2, x19
ffffc0206648: 54000162 	b.hs	0xffffc0206674 <align+0x48>
ffffc020664c: 12001c61 	and	w1, w3, #0xff
ffffc0206650: 52800026 	mov	w6, #1
ffffc0206654: 2a0603e5 	mov	w5, w6
ffffc0206658: cb050007 	sub	x7, x0, x5
ffffc020665c: eb0400ff 	cmp	x7, x4
ffffc0206660: 54000102 	b.hs	0xffffc0206680 <align+0x54>
ffffc0206664: cb020262 	sub	x2, x19, x2
ffffc0206668: aa0403e0 	mov	x0, x4
ffffc020666c: 940009f3 	bl	0xffffc0208e38 <memset>
ffffc0206670: 8b130000 	add	x0, x0, x19
ffffc0206674: f9400bf3 	ldr	x19, [sp, #16]
ffffc0206678: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020667c: d65f03c0 	ret
ffffc0206680: cb050265 	sub	x5, x19, x5
ffffc0206684: 394000e3 	ldrb	w3, [x7]
ffffc0206688: 110004c6 	add	w6, w6, #1
ffffc020668c: 38256883 	strb	w3, [x4, x5]
ffffc0206690: 17fffff1 	b	0xffffc0206654 <align+0x28>

0000ffffc0206694 <console_write>:
ffffc0206694: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0206698: 90000041 	adrp	x1, 0xffffc020e000 <console_write+0x24>
ffffc020669c: 910003fd 	mov	x29, sp
ffffc02066a0: f940ac21 	ldr	x1, [x1, #344]
ffffc02066a4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02066a8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02066ac: aa0003f5 	mov	x21, x0
ffffc02066b0: f9001bf7 	str	x23, [sp, #48]
ffffc02066b4: d63f0020 	blr	x1
ffffc02066b8: d0000080 	adrp	x0, 0xffffc0218000 <console_write+0x6c>
ffffc02066bc: b9400000 	ldr	w0, [x0]
ffffc02066c0: 34000360 	cbz	w0, 0xffffc020672c <console_write+0x98>
ffffc02066c4: d0000073 	adrp	x19, 0xffffc0214000 <console_write+0x68>
ffffc02066c8: 52800020 	mov	w0, #1
ffffc02066cc: b9000260 	str	w0, [x19]
ffffc02066d0: d5033bbf 	dmb	ish
ffffc02066d4: 91000274 	add	x20, x19, #0
ffffc02066d8: f0000096 	adrp	x22, 0xffffc0219000 <console_write+0x90>
ffffc02066dc: 90000057 	adrp	x23, 0xffffc020e000 <console_write+0x68>
ffffc02066e0: 394002a0 	ldrb	w0, [x21]
ffffc02066e4: 34000200 	cbz	w0, 0xffffc0206724 <console_write+0x90>
ffffc02066e8: f9400ec0 	ldr	x0, [x22, #24]
ffffc02066ec: b5000140 	cbnz	x0, 0xffffc0206714 <console_write+0x80>
ffffc02066f0: b9400680 	ldr	w0, [x20, #4]
ffffc02066f4: 384016a1 	ldrb	w1, [x21], #1
ffffc02066f8: 92402800 	and	x0, x0, #0x7ff
ffffc02066fc: 8b000280 	add	x0, x20, x0
ffffc0206700: 39002001 	strb	w1, [x0, #8]
ffffc0206704: b9400680 	ldr	w0, [x20, #4]
ffffc0206708: 11000400 	add	w0, w0, #1
ffffc020670c: b9000680 	str	w0, [x20, #4]
ffffc0206710: 17fffff4 	b	0xffffc02066e0 <console_write+0x4c>
ffffc0206714: 97fff1b7 	bl	0xffffc0202df0 <phys_processor_id>
ffffc0206718: f940cae1 	ldr	x1, [x23, #400]
ffffc020671c: eb01001f 	cmp	x0, x1
ffffc0206720: 54fffe80 	b.eq	0xffffc02066f0 <console_write+0x5c>
ffffc0206724: d5033bbf 	dmb	ish
ffffc0206728: b900027f 	str	wzr, [x19]
ffffc020672c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0206730: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0206734: f9401bf7 	ldr	x23, [sp, #48]
ffffc0206738: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc020673c: d65f03c0 	ret

0000ffffc0206740 <__vprintk>:
ffffc0206740: a9b27bfd 	stp	x29, x30, [sp, #-224]!
ffffc0206744: 910003fd 	mov	x29, sp
ffffc0206748: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020674c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0206750: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0206754: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc0206758: aa0003fa 	mov	x26, x0
ffffc020675c: f9002bfb 	str	x27, [sp, #80]
ffffc0206760: 52800159 	mov	w25, #10
ffffc0206764: b9401834 	ldr	w20, [x1, #24]
ffffc0206768: a9405c33 	ldp	x19, x23, [x1]
ffffc020676c: 910183a1 	add	x1, x29, #96
ffffc0206770: aa0103f5 	mov	x21, x1
ffffc0206774: 9101fc38 	add	x24, x1, #127
ffffc0206778: 39400342 	ldrb	w2, [x26]
ffffc020677c: 91000756 	add	x22, x26, #1
ffffc0206780: 34001562 	cbz	w2, 0xffffc0206a2c <__vprintk+0x2ec>
ffffc0206784: 7100945f 	cmp	w2, #37
ffffc0206788: 540014c1 	b.ne	0xffffc0206a20 <__vprintk+0x2e0>
ffffc020678c: 3900003f 	strb	wzr, [x1]
ffffc0206790: aa1503e0 	mov	x0, x21
ffffc0206794: 97ffffc0 	bl	0xffffc0206694 <console_write>
ffffc0206798: 39400741 	ldrb	w1, [x26, #1]
ffffc020679c: 52800409 	mov	w9, #32
ffffc02067a0: 52800600 	mov	w0, #48
ffffc02067a4: 91000b42 	add	x2, x26, #2
ffffc02067a8: 7100c03f 	cmp	w1, #48
ffffc02067ac: 52800008 	mov	w8, #0
ffffc02067b0: 1a801129 	csel	w9, w9, w0, ne
ffffc02067b4: 5100c020 	sub	w0, w1, #48
ffffc02067b8: 91000456 	add	x22, x2, #1
ffffc02067bc: 12001c00 	and	w0, w0, #0xff
ffffc02067c0: 7100241f 	cmp	w0, #9
ffffc02067c4: 54000429 	b.ls	0xffffc0206848 <__vprintk+0x108>
ffffc02067c8: 7101b03f 	cmp	w1, #108
ffffc02067cc: 54001441 	b.ne	0xffffc0206a54 <__vprintk+0x314>
ffffc02067d0: 39400041 	ldrb	w1, [x2]
ffffc02067d4: 7101b03f 	cmp	w1, #108
ffffc02067d8: 54000061 	b.ne	0xffffc02067e4 <__vprintk+0xa4>
ffffc02067dc: 39400441 	ldrb	w1, [x2, #1]
ffffc02067e0: 91000856 	add	x22, x2, #2
ffffc02067e4: 7101c03f 	cmp	w1, #112
ffffc02067e8: 54000a20 	b.eq	0xffffc020692c <__vprintk+0x1ec>
ffffc02067ec: 54001708 	b.hi	0xffffc0206acc <__vprintk+0x38c>
ffffc02067f0: 71018c3f 	cmp	w1, #99
ffffc02067f4: 540003a0 	b.eq	0xffffc0206868 <__vprintk+0x128>
ffffc02067f8: 7101903f 	cmp	w1, #100
ffffc02067fc: 540013c1 	b.ne	0xffffc0206a74 <__vprintk+0x334>
ffffc0206800: 37f805f4 	tbnz	w20, #31, 0xffffc02068bc <__vprintk+0x17c>
ffffc0206804: 91003e61 	add	x1, x19, #15
ffffc0206808: 2a1403e2 	mov	w2, w20
ffffc020680c: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc0206810: f9400260 	ldr	x0, [x19]
ffffc0206814: 2a0203f4 	mov	w20, w2
ffffc0206818: aa0103f3 	mov	x19, x1
ffffc020681c: b6f80e00 	tbz	x0, #63, 0xffffc02069dc <__vprintk+0x29c>
ffffc0206820: 528005a1 	mov	w1, #45
ffffc0206824: cb0003e0 	neg	x0, x0
ffffc0206828: 390183a1 	strb	w1, [x29, #96]
ffffc020682c: 910006a1 	add	x1, x21, #1
ffffc0206830: 97ffff4b 	bl	0xffffc020655c <uint2str>
ffffc0206834: 2a0903e3 	mov	w3, w9
ffffc0206838: 2a0803e2 	mov	w2, w8
ffffc020683c: aa1503e1 	mov	x1, x21
ffffc0206840: 97ffff7b 	bl	0xffffc020662c <align>
ffffc0206844: 14000046 	b	0xffffc020695c <__vprintk+0x21c>
ffffc0206848: 1b190501 	madd	w1, w8, w25, w1
ffffc020684c: aa1603e2 	mov	x2, x22
ffffc0206850: 5100c028 	sub	w8, w1, #48
ffffc0206854: 385ff2c1 	ldurb	w1, [x22, #-1]
ffffc0206858: 7101fd1f 	cmp	w8, #127
ffffc020685c: 1a9f3108 	csel	w8, w8, wzr, lo
ffffc0206860: 17ffffd5 	b	0xffffc02067b4 <__vprintk+0x74>
ffffc0206864: aa0203f6 	mov	x22, x2
ffffc0206868: 37f80194 	tbnz	w20, #31, 0xffffc0206898 <__vprintk+0x158>
ffffc020686c: 91002e60 	add	x0, x19, #11
ffffc0206870: 2a1403e1 	mov	w1, w20
ffffc0206874: 927df000 	and	x0, x0, #0xfffffffffffffff8
ffffc0206878: b9400262 	ldr	w2, [x19]
ffffc020687c: 2a0103f4 	mov	w20, w1
ffffc0206880: aa0003f3 	mov	x19, x0
ffffc0206884: 910006a0 	add	x0, x21, #1
ffffc0206888: 390183a2 	strb	w2, [x29, #96]
ffffc020688c: aa0003e1 	mov	x1, x0
ffffc0206890: aa1603fa 	mov	x26, x22
ffffc0206894: 17ffffb9 	b	0xffffc0206778 <__vprintk+0x38>
ffffc0206898: 11002281 	add	w1, w20, #8
ffffc020689c: 7100003f 	cmp	w1, #0
ffffc02068a0: 5400008d 	b.le	0xffffc02068b0 <__vprintk+0x170>
ffffc02068a4: 91002e60 	add	x0, x19, #11
ffffc02068a8: 927df000 	and	x0, x0, #0xfffffffffffffff8
ffffc02068ac: 17fffff3 	b	0xffffc0206878 <__vprintk+0x138>
ffffc02068b0: aa1303e0 	mov	x0, x19
ffffc02068b4: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc02068b8: 17fffff0 	b	0xffffc0206878 <__vprintk+0x138>
ffffc02068bc: 11002282 	add	w2, w20, #8
ffffc02068c0: 7100005f 	cmp	w2, #0
ffffc02068c4: 5400008d 	b.le	0xffffc02068d4 <__vprintk+0x194>
ffffc02068c8: 91003e61 	add	x1, x19, #15
ffffc02068cc: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc02068d0: 17ffffd0 	b	0xffffc0206810 <__vprintk+0xd0>
ffffc02068d4: aa1303e1 	mov	x1, x19
ffffc02068d8: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc02068dc: 17ffffcd 	b	0xffffc0206810 <__vprintk+0xd0>
ffffc02068e0: 37f80134 	tbnz	w20, #31, 0xffffc0206904 <__vprintk+0x1c4>
ffffc02068e4: 91002e61 	add	x1, x19, #11
ffffc02068e8: 2a1403e3 	mov	w3, w20
ffffc02068ec: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc02068f0: b9800260 	ldrsw	x0, [x19]
ffffc02068f4: 2a0303f4 	mov	w20, w3
ffffc02068f8: aa0203f6 	mov	x22, x2
ffffc02068fc: aa0103f3 	mov	x19, x1
ffffc0206900: 17ffffc7 	b	0xffffc020681c <__vprintk+0xdc>
ffffc0206904: 11002283 	add	w3, w20, #8
ffffc0206908: 7100007f 	cmp	w3, #0
ffffc020690c: 5400008d 	b.le	0xffffc020691c <__vprintk+0x1dc>
ffffc0206910: 91002e61 	add	x1, x19, #11
ffffc0206914: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc0206918: 17fffff6 	b	0xffffc02068f0 <__vprintk+0x1b0>
ffffc020691c: aa1303e1 	mov	x1, x19
ffffc0206920: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc0206924: 17fffff3 	b	0xffffc02068f0 <__vprintk+0x1b0>
ffffc0206928: aa0203f6 	mov	x22, x2
ffffc020692c: 528f0600 	mov	w0, #30768
ffffc0206930: 7900c3a0 	strh	w0, [x29, #96]
ffffc0206934: 37f80214 	tbnz	w20, #31, 0xffffc0206974 <__vprintk+0x234>
ffffc0206938: 91003e68 	add	x8, x19, #15
ffffc020693c: 2a1403e9 	mov	w9, w20
ffffc0206940: 927df108 	and	x8, x8, #0xfffffffffffffff8
ffffc0206944: f9400260 	ldr	x0, [x19]
ffffc0206948: 92800002 	mov	x2, #-1
ffffc020694c: 91000aa1 	add	x1, x21, #2
ffffc0206950: 97ffff1c 	bl	0xffffc02065c0 <hex2str>
ffffc0206954: 2a0903f4 	mov	w20, w9
ffffc0206958: aa0803f3 	mov	x19, x8
ffffc020695c: eb18001f 	cmp	x0, x24
ffffc0206960: 54fff963 	b.lo	0xffffc020688c <__vprintk+0x14c>
ffffc0206964: 3900001f 	strb	wzr, [x0]
ffffc0206968: aa1503e0 	mov	x0, x21
ffffc020696c: 97ffff4a 	bl	0xffffc0206694 <console_write>
ffffc0206970: 14000055 	b	0xffffc0206ac4 <__vprintk+0x384>
ffffc0206974: 11002289 	add	w9, w20, #8
ffffc0206978: 7100013f 	cmp	w9, #0
ffffc020697c: 5400008d 	b.le	0xffffc020698c <__vprintk+0x24c>
ffffc0206980: 91003e68 	add	x8, x19, #15
ffffc0206984: 927df108 	and	x8, x8, #0xfffffffffffffff8
ffffc0206988: 17ffffef 	b	0xffffc0206944 <__vprintk+0x204>
ffffc020698c: aa1303e8 	mov	x8, x19
ffffc0206990: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc0206994: 17ffffec 	b	0xffffc0206944 <__vprintk+0x204>
ffffc0206998: 1100229b 	add	w27, w20, #8
ffffc020699c: 7100037f 	cmp	w27, #0
ffffc02069a0: 5400008d 	b.le	0xffffc02069b0 <__vprintk+0x270>
ffffc02069a4: 91003e7a 	add	x26, x19, #15
ffffc02069a8: 927df35a 	and	x26, x26, #0xfffffffffffffff8
ffffc02069ac: 14000042 	b	0xffffc0206ab4 <__vprintk+0x374>
ffffc02069b0: aa1303fa 	mov	x26, x19
ffffc02069b4: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc02069b8: 1400003f 	b	0xffffc0206ab4 <__vprintk+0x374>
ffffc02069bc: 91003e62 	add	x2, x19, #15
ffffc02069c0: 2a1403e3 	mov	w3, w20
ffffc02069c4: 927df042 	and	x2, x2, #0xfffffffffffffff8
ffffc02069c8: f9400260 	ldr	x0, [x19]
ffffc02069cc: 2a0303f4 	mov	w20, w3
ffffc02069d0: aa0203f3 	mov	x19, x2
ffffc02069d4: 7101d43f 	cmp	w1, #117
ffffc02069d8: 540001c1 	b.ne	0xffffc0206a10 <__vprintk+0x2d0>
ffffc02069dc: aa1503e1 	mov	x1, x21
ffffc02069e0: 17ffff94 	b	0xffffc0206830 <__vprintk+0xf0>
ffffc02069e4: aa1303e2 	mov	x2, x19
ffffc02069e8: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc02069ec: 17fffff7 	b	0xffffc02069c8 <__vprintk+0x288>
ffffc02069f0: 91002e62 	add	x2, x19, #11
ffffc02069f4: 2a1403e3 	mov	w3, w20
ffffc02069f8: 927df042 	and	x2, x2, #0xfffffffffffffff8
ffffc02069fc: b9400260 	ldr	w0, [x19]
ffffc0206a00: 17fffff3 	b	0xffffc02069cc <__vprintk+0x28c>
ffffc0206a04: aa1303e2 	mov	x2, x19
ffffc0206a08: 8b34c2f3 	add	x19, x23, w20, sxtw
ffffc0206a0c: 17fffffc 	b	0xffffc02069fc <__vprintk+0x2bc>
ffffc0206a10: d2800002 	mov	x2, #0
ffffc0206a14: aa1503e1 	mov	x1, x21
ffffc0206a18: 97fffeea 	bl	0xffffc02065c0 <hex2str>
ffffc0206a1c: 17ffff86 	b	0xffffc0206834 <__vprintk+0xf4>
ffffc0206a20: 91000420 	add	x0, x1, #1
ffffc0206a24: 39000022 	strb	w2, [x1]
ffffc0206a28: 17ffffcd 	b	0xffffc020695c <__vprintk+0x21c>
ffffc0206a2c: 3900003f 	strb	wzr, [x1]
ffffc0206a30: aa1503e0 	mov	x0, x21
ffffc0206a34: 97ffff18 	bl	0xffffc0206694 <console_write>
ffffc0206a38: f9402bfb 	ldr	x27, [sp, #80]
ffffc0206a3c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0206a40: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0206a44: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0206a48: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc0206a4c: a8ce7bfd 	ldp	x29, x30, [sp], #224
ffffc0206a50: d65f03c0 	ret
ffffc0206a54: 7101c03f 	cmp	w1, #112
ffffc0206a58: 54fff680 	b.eq	0xffffc0206928 <__vprintk+0x1e8>
ffffc0206a5c: 54000168 	b.hi	0xffffc0206a88 <__vprintk+0x348>
ffffc0206a60: 71018c3f 	cmp	w1, #99
ffffc0206a64: 54fff000 	b.eq	0xffffc0206864 <__vprintk+0x124>
ffffc0206a68: 7101903f 	cmp	w1, #100
ffffc0206a6c: 54fff3a0 	b.eq	0xffffc02068e0 <__vprintk+0x1a0>
ffffc0206a70: aa0203f6 	mov	x22, x2
ffffc0206a74: 528004a0 	mov	w0, #37
ffffc0206a78: 390183a0 	strb	w0, [x29, #96]
ffffc0206a7c: 390187a1 	strb	w1, [x29, #97]
ffffc0206a80: 91000aa0 	add	x0, x21, #2
ffffc0206a84: 17ffff82 	b	0xffffc020688c <__vprintk+0x14c>
ffffc0206a88: 7101d43f 	cmp	w1, #117
ffffc0206a8c: 540002e0 	b.eq	0xffffc0206ae8 <__vprintk+0x3a8>
ffffc0206a90: 7101e03f 	cmp	w1, #120
ffffc0206a94: 540002a0 	b.eq	0xffffc0206ae8 <__vprintk+0x3a8>
ffffc0206a98: 7101cc3f 	cmp	w1, #115
ffffc0206a9c: 54fffea1 	b.ne	0xffffc0206a70 <__vprintk+0x330>
ffffc0206aa0: aa0203f6 	mov	x22, x2
ffffc0206aa4: 37fff7b4 	tbnz	w20, #31, 0xffffc0206998 <__vprintk+0x258>
ffffc0206aa8: 91003e7a 	add	x26, x19, #15
ffffc0206aac: 2a1403fb 	mov	w27, w20
ffffc0206ab0: 927df35a 	and	x26, x26, #0xfffffffffffffff8
ffffc0206ab4: f9400260 	ldr	x0, [x19]
ffffc0206ab8: 2a1b03f4 	mov	w20, w27
ffffc0206abc: aa1a03f3 	mov	x19, x26
ffffc0206ac0: 97fffef5 	bl	0xffffc0206694 <console_write>
ffffc0206ac4: aa1503e0 	mov	x0, x21
ffffc0206ac8: 17ffff71 	b	0xffffc020688c <__vprintk+0x14c>
ffffc0206acc: 7101d43f 	cmp	w1, #117
ffffc0206ad0: 540001c0 	b.eq	0xffffc0206b08 <__vprintk+0x3c8>
ffffc0206ad4: 7101e03f 	cmp	w1, #120
ffffc0206ad8: 54000180 	b.eq	0xffffc0206b08 <__vprintk+0x3c8>
ffffc0206adc: 7101cc3f 	cmp	w1, #115
ffffc0206ae0: 54fffe20 	b.eq	0xffffc0206aa4 <__vprintk+0x364>
ffffc0206ae4: 17ffffe4 	b	0xffffc0206a74 <__vprintk+0x334>
ffffc0206ae8: aa0203f6 	mov	x22, x2
ffffc0206aec: 36fff834 	tbz	w20, #31, 0xffffc02069f0 <__vprintk+0x2b0>
ffffc0206af0: 11002283 	add	w3, w20, #8
ffffc0206af4: 7100007f 	cmp	w3, #0
ffffc0206af8: 54fff86d 	b.le	0xffffc0206a04 <__vprintk+0x2c4>
ffffc0206afc: 91002e62 	add	x2, x19, #11
ffffc0206b00: 927df042 	and	x2, x2, #0xfffffffffffffff8
ffffc0206b04: 17ffffbe 	b	0xffffc02069fc <__vprintk+0x2bc>
ffffc0206b08: 36fff5b4 	tbz	w20, #31, 0xffffc02069bc <__vprintk+0x27c>
ffffc0206b0c: 11002283 	add	w3, w20, #8
ffffc0206b10: 7100007f 	cmp	w3, #0
ffffc0206b14: 54fff68d 	b.le	0xffffc02069e4 <__vprintk+0x2a4>
ffffc0206b18: 91003e62 	add	x2, x19, #15
ffffc0206b1c: 927df042 	and	x2, x2, #0xfffffffffffffff8
ffffc0206b20: 17ffffaa 	b	0xffffc02069c8 <__vprintk+0x288>

0000ffffc0206b24 <printk>:
ffffc0206b24: a9ae7bfd 	stp	x29, x30, [sp, #-288]!
ffffc0206b28: 910003fd 	mov	x29, sp
ffffc0206b2c: f9000bf3 	str	x19, [sp, #16]
ffffc0206b30: d0000093 	adrp	x19, 0xffffc0218000 <printk+0x54>
ffffc0206b34: 91000273 	add	x19, x19, #0
ffffc0206b38: a90e8ba1 	stp	x1, x2, [x29, #232]
ffffc0206b3c: 910483a1 	add	x1, x29, #288
ffffc0206b40: a90407a1 	stp	x1, x1, [x29, #64]
ffffc0206b44: 910383a1 	add	x1, x29, #224
ffffc0206b48: f9002ba1 	str	x1, [x29, #80]
ffffc0206b4c: 128006e1 	mov	w1, #-56
ffffc0206b50: b9005ba1 	str	w1, [x29, #88]
ffffc0206b54: 12800fe1 	mov	w1, #-128
ffffc0206b58: b9005fa1 	str	w1, [x29, #92]
ffffc0206b5c: 3d801ba0 	str	q0, [x29, #96]
ffffc0206b60: 3d801fa1 	str	q1, [x29, #112]
ffffc0206b64: 3d8023a2 	str	q2, [x29, #128]
ffffc0206b68: 3d8027a3 	str	q3, [x29, #144]
ffffc0206b6c: 3d802ba4 	str	q4, [x29, #160]
ffffc0206b70: 3d802fa5 	str	q5, [x29, #176]
ffffc0206b74: 3d8033a6 	str	q6, [x29, #192]
ffffc0206b78: 3d8037a7 	str	q7, [x29, #208]
ffffc0206b7c: a90f93a3 	stp	x3, x4, [x29, #248]
ffffc0206b80: 91001264 	add	x4, x19, #4
ffffc0206b84: a9109ba5 	stp	x5, x6, [x29, #264]
ffffc0206b88: f9008fa7 	str	x7, [x29, #280]
ffffc0206b8c: f9800091 	prfm	pstl1strm, [x4]
ffffc0206b90: 885ffc81 	ldaxr	w1, [x4]
ffffc0206b94: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc0206b98: 88037c82 	stxr	w3, w2, [x4]
ffffc0206b9c: 35ffffa3 	cbnz	w3, 0xffffc0206b90 <printk+0x6c>
ffffc0206ba0: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc0206ba4: 340000c2 	cbz	w2, 0xffffc0206bbc <printk+0x98>
ffffc0206ba8: d50320bf 	sevl
ffffc0206bac: d503205f 	wfe
ffffc0206bb0: 485ffc83 	ldaxrh	w3, [x4]
ffffc0206bb4: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc0206bb8: 35ffffa2 	cbnz	w2, 0xffffc0206bac <printk+0x88>
ffffc0206bbc: a9440fa2 	ldp	x2, x3, [x29, #64]
ffffc0206bc0: a9020fa2 	stp	x2, x3, [x29, #32]
ffffc0206bc4: a9450fa2 	ldp	x2, x3, [x29, #80]
ffffc0206bc8: 910083a1 	add	x1, x29, #32
ffffc0206bcc: a9030fa2 	stp	x2, x3, [x29, #48]
ffffc0206bd0: 97fffedc 	bl	0xffffc0206740 <__vprintk>
ffffc0206bd4: 79400a60 	ldrh	w0, [x19, #4]
ffffc0206bd8: 91001261 	add	x1, x19, #4
ffffc0206bdc: 11000400 	add	w0, w0, #1
ffffc0206be0: 489ffc20 	stlrh	w0, [x1]
ffffc0206be4: f9400bf3 	ldr	x19, [sp, #16]
ffffc0206be8: a8d27bfd 	ldp	x29, x30, [sp], #288
ffffc0206bec: d65f03c0 	ret

0000ffffc0206bf0 <panic_printk>:
ffffc0206bf0: a9ae7bfd 	stp	x29, x30, [sp, #-288]!
ffffc0206bf4: 910003fd 	mov	x29, sp
ffffc0206bf8: f9000bf3 	str	x19, [sp, #16]
ffffc0206bfc: aa0003f3 	mov	x19, x0
ffffc0206c00: 3d801ba0 	str	q0, [x29, #96]
ffffc0206c04: 3d801fa1 	str	q1, [x29, #112]
ffffc0206c08: 3d8023a2 	str	q2, [x29, #128]
ffffc0206c0c: 3d8027a3 	str	q3, [x29, #144]
ffffc0206c10: 3d802ba4 	str	q4, [x29, #160]
ffffc0206c14: 3d802fa5 	str	q5, [x29, #176]
ffffc0206c18: 3d8033a6 	str	q6, [x29, #192]
ffffc0206c1c: 3d8037a7 	str	q7, [x29, #208]
ffffc0206c20: a90e8ba1 	stp	x1, x2, [x29, #232]
ffffc0206c24: a90f93a3 	stp	x3, x4, [x29, #248]
ffffc0206c28: a9109ba5 	stp	x5, x6, [x29, #264]
ffffc0206c2c: f9008fa7 	str	x7, [x29, #280]
ffffc0206c30: 97fff070 	bl	0xffffc0202df0 <phys_processor_id>
ffffc0206c34: f0000081 	adrp	x1, 0xffffc0219000 <panic_printk+0x90>
ffffc0206c38: d2800022 	mov	x2, #1
ffffc0206c3c: 91006024 	add	x4, x1, #24
ffffc0206c40: c85f7c85 	ldxr	x5, [x4]
ffffc0206c44: ea0200a3 	ands	x3, x5, x2
ffffc0206c48: 54000041 	b.ne	0xffffc0206c50 <panic_printk+0x60>
ffffc0206c4c: aa0200a5 	orr	x5, x5, x2
ffffc0206c50: c8047c85 	stxr	w4, x5, [x4]
ffffc0206c54: d5033bbf 	dmb	ish
ffffc0206c58: 35ffff24 	cbnz	w4, 0xffffc0206c3c <panic_printk+0x4c>
ffffc0206c5c: 90000041 	adrp	x1, 0xffffc020e000 <panic_printk+0x8c>
ffffc0206c60: b4000083 	cbz	x3, 0xffffc0206c70 <panic_printk+0x80>
ffffc0206c64: f940c822 	ldr	x2, [x1, #400]
ffffc0206c68: eb00005f 	cmp	x2, x0
ffffc0206c6c: 54000221 	b.ne	0xffffc0206cb0 <panic_printk+0xc0>
ffffc0206c70: f900c820 	str	x0, [x1, #400]
ffffc0206c74: 910483a0 	add	x0, x29, #288
ffffc0206c78: a90403a0 	stp	x0, x0, [x29, #64]
ffffc0206c7c: 910383a0 	add	x0, x29, #224
ffffc0206c80: f9002ba0 	str	x0, [x29, #80]
ffffc0206c84: 128006e0 	mov	w0, #-56
ffffc0206c88: b9005ba0 	str	w0, [x29, #88]
ffffc0206c8c: 12800fe0 	mov	w0, #-128
ffffc0206c90: b9005fa0 	str	w0, [x29, #92]
ffffc0206c94: a94407a0 	ldp	x0, x1, [x29, #64]
ffffc0206c98: a90207a0 	stp	x0, x1, [x29, #32]
ffffc0206c9c: a94507a0 	ldp	x0, x1, [x29, #80]
ffffc0206ca0: a90307a0 	stp	x0, x1, [x29, #48]
ffffc0206ca4: 910083a1 	add	x1, x29, #32
ffffc0206ca8: aa1303e0 	mov	x0, x19
ffffc0206cac: 97fffea5 	bl	0xffffc0206740 <__vprintk>
ffffc0206cb0: f9400bf3 	ldr	x19, [sp, #16]
ffffc0206cb4: a8d27bfd 	ldp	x29, x30, [sp], #288
ffffc0206cb8: d65f03c0 	ret

0000ffffc0206cbc <arch_paging_flush_page_tlbs>:
ffffc0206cbc: d34cfc00 	lsr	x0, x0, #12
ffffc0206cc0: d5033b9f 	dsb	ish
ffffc0206cc4: d50c8720 	tlbi	vae2, x0
ffffc0206cc8: d5033b9f 	dsb	ish
ffffc0206ccc: d5033fdf 	isb
ffffc0206cd0: d65f03c0 	ret

0000ffffc0206cd4 <find_next_free_page>:
ffffc0206cd4: f9400403 	ldr	x3, [x0, #8]
ffffc0206cd8: eb01007f 	cmp	x3, x1
ffffc0206cdc: 54000189 	b.ls	0xffffc0206d0c <find_next_free_page+0x38>
ffffc0206ce0: f2401422 	ands	x2, x1, #0x3f
ffffc0206ce4: 540000a0 	b.eq	0xffffc0206cf8 <find_next_free_page+0x24>
ffffc0206ce8: 52800804 	mov	w4, #64
ffffc0206cec: 4b020082 	sub	w2, w4, w2
ffffc0206cf0: 92800004 	mov	x4, #-1
ffffc0206cf4: 9ac22482 	lsr	x2, x4, x2
ffffc0206cf8: d346fc21 	lsr	x1, x1, #6
ffffc0206cfc: d346fc65 	lsr	x5, x3, #6
ffffc0206d00: d37df024 	lsl	x4, x1, #3
ffffc0206d04: eb0100bf 	cmp	x5, x1
ffffc0206d08: 54000068 	b.hi	0xffffc0206d14 <find_next_free_page+0x40>
ffffc0206d0c: 92800000 	mov	x0, #-1
ffffc0206d10: 1400000d 	b	0xffffc0206d44 <find_next_free_page+0x70>
ffffc0206d14: f9400c06 	ldr	x6, [x0, #24]
ffffc0206d18: f86468c6 	ldr	x6, [x6, x4]
ffffc0206d1c: 91002084 	add	x4, x4, #8
ffffc0206d20: aa060042 	orr	x2, x2, x6
ffffc0206d24: b100045f 	cmn	x2, #1
ffffc0206d28: 54000100 	b.eq	0xffffc0206d48 <find_next_free_page+0x74>
ffffc0206d2c: aa2203e2 	mvn	x2, x2
ffffc0206d30: dac00042 	rbit	x2, x2
ffffc0206d34: dac01040 	clz	x0, x2
ffffc0206d38: 8b011800 	add	x0, x0, x1, lsl #6
ffffc0206d3c: eb00007f 	cmp	x3, x0
ffffc0206d40: da9f8000 	csinv	x0, x0, xzr, hi
ffffc0206d44: d65f03c0 	ret
ffffc0206d48: 91000421 	add	x1, x1, #1
ffffc0206d4c: d2800002 	mov	x2, #0
ffffc0206d50: 17ffffed 	b	0xffffc0206d04 <find_next_free_page+0x30>

0000ffffc0206d54 <page_alloc_internal>:
ffffc0206d54: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0206d58: 2a0103ee 	mov	w14, w1
ffffc0206d5c: aa0003e8 	mov	x8, x0
ffffc0206d60: aa0e03e9 	mov	x9, x14
ffffc0206d64: 910003fd 	mov	x29, sp
ffffc0206d68: f940000a 	ldr	x10, [x0]
ffffc0206d6c: aa0203eb 	mov	x11, x2
ffffc0206d70: 5280002f 	mov	w15, #1
ffffc0206d74: 8a4a304a 	and	x10, x2, x10, lsr #12
ffffc0206d78: aa0a03e0 	mov	x0, x10
ffffc0206d7c: aa0003e7 	mov	x7, x0
ffffc0206d80: cb0a00e0 	sub	x0, x7, x10
ffffc0206d84: ea0b0000 	ands	x0, x0, x11
ffffc0206d88: 54000060 	b.eq	0xffffc0206d94 <page_alloc_internal+0x40>
ffffc0206d8c: 8b0701c7 	add	x7, x14, x7
ffffc0206d90: cb0000e7 	sub	x7, x7, x0
ffffc0206d94: aa0703e1 	mov	x1, x7
ffffc0206d98: aa0803e0 	mov	x0, x8
ffffc0206d9c: 97ffffce 	bl	0xffffc0206cd4 <find_next_free_page>
ffffc0206da0: 7100013f 	cmp	w9, #0
ffffc0206da4: aa0003e7 	mov	x7, x0
ffffc0206da8: ba411804 	ccmn	x0, #1, #4, ne
ffffc0206dac: 54000081 	b.ne	0xffffc0206dbc <page_alloc_internal+0x68>
ffffc0206db0: d2800000 	mov	x0, #0
ffffc0206db4: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0206db8: d65f03c0 	ret
ffffc0206dbc: cb0a0000 	sub	x0, x0, x10
ffffc0206dc0: ea0b001f 	tst	x0, x11
ffffc0206dc4: 54fffde1 	b.ne	0xffffc0206d80 <page_alloc_internal+0x2c>
ffffc0206dc8: aa0703e0 	mov	x0, x7
ffffc0206dcc: 4b0701ed 	sub	w13, w15, w7
ffffc0206dd0: 0b0001a1 	add	w1, w13, w0
ffffc0206dd4: 2a0703e4 	mov	w4, w7
ffffc0206dd8: 6b01013f 	cmp	w9, w1
ffffc0206ddc: 540002e8 	b.hi	0xffffc0206e38 <page_alloc_internal+0xe4>
ffffc0206de0: 52800002 	mov	w2, #0
ffffc0206de4: d2800025 	mov	x5, #1
ffffc0206de8: 0b020080 	add	w0, w4, w2
ffffc0206dec: f9400d03 	ldr	x3, [x8, #24]
ffffc0206df0: 53067c01 	lsr	w1, w0, #6
ffffc0206df4: 927df063 	and	x3, x3, #0xfffffffffffffff8
ffffc0206df8: 12001400 	and	w0, w0, #0x3f
ffffc0206dfc: 8b214c61 	add	x1, x3, w1, uxtw #3
ffffc0206e00: 9ac020a0 	lsl	x0, x5, x0
ffffc0206e04: c85f7c26 	ldxr	x6, [x1]
ffffc0206e08: aa0000c6 	orr	x6, x6, x0
ffffc0206e0c: c8037c26 	stxr	w3, x6, [x1]
ffffc0206e10: 35ffffa3 	cbnz	w3, 0xffffc0206e04 <page_alloc_internal+0xb0>
ffffc0206e14: 11000442 	add	w2, w2, #1
ffffc0206e18: 6b02013f 	cmp	w9, w2
ffffc0206e1c: 54fffe68 	b.hi	0xffffc0206de8 <page_alloc_internal+0x94>
ffffc0206e20: f9400900 	ldr	x0, [x8, #16]
ffffc0206e24: 8b294009 	add	x9, x0, w9, uxtw
ffffc0206e28: f9400100 	ldr	x0, [x8]
ffffc0206e2c: f9000909 	str	x9, [x8, #16]
ffffc0206e30: 8b073000 	add	x0, x0, x7, lsl #12
ffffc0206e34: 17ffffe0 	b	0xffffc0206db4 <page_alloc_internal+0x60>
ffffc0206e38: 9100040c 	add	x12, x0, #1
ffffc0206e3c: aa0803e0 	mov	x0, x8
ffffc0206e40: aa0c03e1 	mov	x1, x12
ffffc0206e44: 97ffffa4 	bl	0xffffc0206cd4 <find_next_free_page>
ffffc0206e48: b100041f 	cmn	x0, #1
ffffc0206e4c: 54fffb20 	b.eq	0xffffc0206db0 <page_alloc_internal+0x5c>
ffffc0206e50: eb00019f 	cmp	x12, x0
ffffc0206e54: 54fffbe0 	b.eq	0xffffc0206dd0 <page_alloc_internal+0x7c>
ffffc0206e58: 17ffffc9 	b	0xffffc0206d7c <page_alloc_internal+0x28>

0000ffffc0206e5c <flush_pt_entry>:
ffffc0206e5c: 35000081 	cbnz	w1, 0xffffc0206e6c <flush_pt_entry+0x10>
ffffc0206e60: 52800002 	mov	w2, #0
ffffc0206e64: d2800101 	mov	x1, #8
ffffc0206e68: 17fff29b 	b	0xffffc02038d4 <arm_dcaches_flush>
ffffc0206e6c: d65f03c0 	ret

0000ffffc0206e70 <paging_get_phys_invalid>:
ffffc0206e70: 92800000 	mov	x0, #-1
ffffc0206e74: d65f03c0 	ret

0000ffffc0206e78 <page_alloc>:
ffffc0206e78: d2800002 	mov	x2, #0
ffffc0206e7c: 17ffffb6 	b	0xffffc0206d54 <page_alloc_internal>

0000ffffc0206e80 <page_alloc_aligned>:
ffffc0206e80: 51000422 	sub	w2, w1, #1
ffffc0206e84: 17ffffb4 	b	0xffffc0206d54 <page_alloc_internal>

0000ffffc0206e88 <page_free>:
ffffc0206e88: b4000601 	cbz	x1, 0xffffc0206f48 <page_free+0xc0>
ffffc0206e8c: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0206e90: aa0103e3 	mov	x3, x1
ffffc0206e94: 910003fd 	mov	x29, sp
ffffc0206e98: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0206e9c: aa0003f3 	mov	x19, x0
ffffc0206ea0: 2a0203f4 	mov	w20, w2
ffffc0206ea4: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0206ea8: f9001bf7 	str	x23, [sp, #48]
ffffc0206eac: d2820015 	mov	x21, #4096
ffffc0206eb0: 52800816 	mov	w22, #64
ffffc0206eb4: d2800037 	mov	x23, #1
ffffc0206eb8: 51000694 	sub	w20, w20, #1
ffffc0206ebc: 3100069f 	cmn	w20, #1
ffffc0206ec0: 540000c1 	b.ne	0xffffc0206ed8 <page_free+0x50>
ffffc0206ec4: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0206ec8: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0206ecc: f9401bf7 	ldr	x23, [sp, #48]
ffffc0206ed0: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0206ed4: d65f03c0 	ret
ffffc0206ed8: f9401260 	ldr	x0, [x19, #32]
ffffc0206edc: 360000c0 	tbz	w0, #0, 0xffffc0206ef4 <page_free+0x6c>
ffffc0206ee0: aa0303e0 	mov	x0, x3
ffffc0206ee4: d2820002 	mov	x2, #4096
ffffc0206ee8: 52800001 	mov	w1, #0
ffffc0206eec: 940007d3 	bl	0xffffc0208e38 <memset>
ffffc0206ef0: aa0003e3 	mov	x3, x0
ffffc0206ef4: f9400260 	ldr	x0, [x19]
ffffc0206ef8: f9400e61 	ldr	x1, [x19, #24]
ffffc0206efc: cb000060 	sub	x0, x3, x0
ffffc0206f00: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc0206f04: 9ad50c00 	sdiv	x0, x0, x21
ffffc0206f08: 1ad60c02 	sdiv	w2, w0, w22
ffffc0206f0c: 8b22cc22 	add	x2, x1, w2, sxtw #3
ffffc0206f10: 6b0003e1 	negs	w1, w0
ffffc0206f14: 12001421 	and	w1, w1, #0x3f
ffffc0206f18: 12001400 	and	w0, w0, #0x3f
ffffc0206f1c: 5a814400 	csneg	w0, w0, w1, mi
ffffc0206f20: 9ac022e0 	lsl	x0, x23, x0
ffffc0206f24: c85f7c44 	ldxr	x4, [x2]
ffffc0206f28: 8a200084 	bic	x4, x4, x0
ffffc0206f2c: c8017c44 	stxr	w1, x4, [x2]
ffffc0206f30: 35ffffa1 	cbnz	w1, 0xffffc0206f24 <page_free+0x9c>
ffffc0206f34: f9400a60 	ldr	x0, [x19, #16]
ffffc0206f38: 91400463 	add	x3, x3, #1, lsl #12     // =4096
ffffc0206f3c: d1000400 	sub	x0, x0, #1
ffffc0206f40: f9000a60 	str	x0, [x19, #16]
ffffc0206f44: 17ffffdd 	b	0xffffc0206eb8 <page_free+0x30>
ffffc0206f48: d65f03c0 	ret

0000ffffc0206f4c <paging_virt2phys>:
ffffc0206f4c: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0206f50: 910003fd 	mov	x29, sp
ffffc0206f54: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0206f58: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0206f5c: aa0103f5 	mov	x21, x1
ffffc0206f60: f9001bf7 	str	x23, [sp, #48]
ffffc0206f64: aa0203f7 	mov	x23, x2
ffffc0206f68: d0000096 	adrp	x22, 0xffffc0218000 <paging_virt2phys+0x64>
ffffc0206f6c: a9408013 	ldp	x19, x0, [x0, #8]
ffffc0206f70: aa1503e1 	mov	x1, x21
ffffc0206f74: f9400662 	ldr	x2, [x19, #8]
ffffc0206f78: d63f0040 	blr	x2
ffffc0206f7c: aa0003f4 	mov	x20, x0
ffffc0206f80: f9400a62 	ldr	x2, [x19, #16]
ffffc0206f84: aa1703e1 	mov	x1, x23
ffffc0206f88: d63f0040 	blr	x2
ffffc0206f8c: 340001c0 	cbz	w0, 0xffffc0206fc4 <paging_virt2phys+0x78>
ffffc0206f90: f9401262 	ldr	x2, [x19, #32]
ffffc0206f94: aa1503e1 	mov	x1, x21
ffffc0206f98: aa1403e0 	mov	x0, x20
ffffc0206f9c: d63f0040 	blr	x2
ffffc0206fa0: b100041f 	cmn	x0, #1
ffffc0206fa4: 54000121 	b.ne	0xffffc0206fc8 <paging_virt2phys+0x7c>
ffffc0206fa8: f9401e61 	ldr	x1, [x19, #56]
ffffc0206fac: aa1403e0 	mov	x0, x20
ffffc0206fb0: 91014273 	add	x19, x19, #80
ffffc0206fb4: d63f0020 	blr	x1
ffffc0206fb8: f94006c1 	ldr	x1, [x22, #8]
ffffc0206fbc: 8b010000 	add	x0, x0, x1
ffffc0206fc0: 17ffffec 	b	0xffffc0206f70 <paging_virt2phys+0x24>
ffffc0206fc4: 92800000 	mov	x0, #-1
ffffc0206fc8: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0206fcc: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0206fd0: f9401bf7 	ldr	x23, [sp, #48]
ffffc0206fd4: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0206fd8: d65f03c0 	ret

0000ffffc0206fdc <paging_destroy>:
ffffc0206fdc: a9b77bfd 	stp	x29, x30, [sp, #-144]!
ffffc0206fe0: 913ffc42 	add	x2, x2, #4095
ffffc0206fe4: 910003fd 	mov	x29, sp
ffffc0206fe8: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0206fec: 9274cc54 	and	x20, x2, #0xfffffffffffff000
ffffc0206ff0: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0206ff4: aa0103f5 	mov	x21, x1
ffffc0206ff8: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc0206ffc: 9101c3ba 	add	x26, x29, #112
ffffc0207000: a90573fb 	stp	x27, x28, [sp, #80]
ffffc0207004: b000009b 	adrp	x27, 0xffffc0218000 <paging_destroy+0x6c>
ffffc0207008: 9100237b 	add	x27, x27, #8
ffffc020700c: aa0003f9 	mov	x25, x0
ffffc0207010: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0207014: 91002360 	add	x0, x27, #8
ffffc0207018: f90033a0 	str	x0, [x29, #96]
ffffc020701c: b9006fa3 	str	w3, [x29, #108]
ffffc0207020: b5000134 	cbnz	x20, 0xffffc0207044 <paging_destroy+0x68>
ffffc0207024: 52800000 	mov	w0, #0
ffffc0207028: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020702c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0207030: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0207034: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc0207038: a94573fb 	ldp	x27, x28, [sp, #80]
ffffc020703c: a8c97bfd 	ldp	x29, x30, [sp], #144
ffffc0207040: d65f03c0 	ret
ffffc0207044: a940833c 	ldp	x28, x0, [x25, #8]
ffffc0207048: aa1a03f6 	mov	x22, x26
ffffc020704c: 52800013 	mov	w19, #0
ffffc0207050: f9000340 	str	x0, [x26]
ffffc0207054: f9400782 	ldr	x2, [x28, #8]
ffffc0207058: aa1503e1 	mov	x1, x21
ffffc020705c: f94002c0 	ldr	x0, [x22]
ffffc0207060: d63f0040 	blr	x2
ffffc0207064: aa0003f8 	mov	x24, x0
ffffc0207068: f9400b82 	ldr	x2, [x28, #16]
ffffc020706c: d2800021 	mov	x1, #1
ffffc0207070: d63f0040 	blr	x2
ffffc0207074: 34000340 	cbz	w0, 0xffffc02070dc <paging_destroy+0x100>
ffffc0207078: f9401382 	ldr	x2, [x28, #32]
ffffc020707c: aa1503e1 	mov	x1, x21
ffffc0207080: aa1803e0 	mov	x0, x24
ffffc0207084: d63f0040 	blr	x2
ffffc0207088: b100041f 	cmn	x0, #1
ffffc020708c: 54000160 	b.eq	0xffffc02070b8 <paging_destroy+0xdc>
ffffc0207090: b9400380 	ldr	w0, [x28]
ffffc0207094: eb14001f 	cmp	x0, x20
ffffc0207098: 54000229 	b.ls	0xffffc02070dc <paging_destroy+0x100>
ffffc020709c: b9400320 	ldr	w0, [x25]
ffffc02070a0: aa1503e3 	mov	x3, x21
ffffc02070a4: b9406fa4 	ldr	w4, [x29, #108]
ffffc02070a8: aa1803e2 	mov	x2, x24
ffffc02070ac: aa1c03e1 	mov	x1, x28
ffffc02070b0: 9400009c 	bl	0xffffc0207320 <split_hugepage>
ffffc02070b4: 35fffba0 	cbnz	w0, 0xffffc0207028 <paging_destroy+0x4c>
ffffc02070b8: f9401f81 	ldr	x1, [x28, #56]
ffffc02070bc: aa1803e0 	mov	x0, x24
ffffc02070c0: 11000673 	add	w19, w19, #1
ffffc02070c4: 9101439c 	add	x28, x28, #80
ffffc02070c8: d63f0020 	blr	x1
ffffc02070cc: f9400361 	ldr	x1, [x27]
ffffc02070d0: 8b000020 	add	x0, x1, x0
ffffc02070d4: f8008ec0 	str	x0, [x22, #8]!
ffffc02070d8: 17ffffdf 	b	0xffffc0207054 <paging_destroy+0x78>
ffffc02070dc: b9400396 	ldr	w22, [x28]
ffffc02070e0: 52820000 	mov	w0, #4096
ffffc02070e4: 93407e77 	sxtw	x23, w19
ffffc02070e8: 51000673 	sub	w19, w19, #1
ffffc02070ec: 710002df 	cmp	w22, #0
ffffc02070f0: 1a8012d6 	csel	w22, w22, w0, ne
ffffc02070f4: 928000e0 	mov	x0, #-8
ffffc02070f8: 9b007ee0 	mul	x0, x23, x0
ffffc02070fc: 8b33cc13 	add	x19, x0, w19, sxtw #3
ffffc0207100: 8b130353 	add	x19, x26, x19
ffffc0207104: f9402381 	ldr	x1, [x28, #64]
ffffc0207108: aa1803e0 	mov	x0, x24
ffffc020710c: d63f0020 	blr	x1
ffffc0207110: b9406fa1 	ldr	w1, [x29, #108]
ffffc0207114: aa1803e0 	mov	x0, x24
ffffc0207118: 97ffff51 	bl	0xffffc0206e5c <flush_pt_entry>
ffffc020711c: 34000257 	cbz	w23, 0xffffc0207164 <paging_destroy+0x188>
ffffc0207120: f9402781 	ldr	x1, [x28, #72]
ffffc0207124: f8777b58 	ldr	x24, [x26, x23, lsl #3]
ffffc0207128: aa1803e0 	mov	x0, x24
ffffc020712c: d63f0020 	blr	x1
ffffc0207130: 340001a0 	cbz	w0, 0xffffc0207164 <paging_destroy+0x188>
ffffc0207134: f94033a0 	ldr	x0, [x29, #96]
ffffc0207138: d101439c 	sub	x28, x28, #80
ffffc020713c: aa1803e1 	mov	x1, x24
ffffc0207140: 52800022 	mov	w2, #1
ffffc0207144: 97ffff51 	bl	0xffffc0206e88 <page_free>
ffffc0207148: f8777a60 	ldr	x0, [x19, x23, lsl #3]
ffffc020714c: aa1503e1 	mov	x1, x21
ffffc0207150: f9400782 	ldr	x2, [x28, #8]
ffffc0207154: d10006f7 	sub	x23, x23, #1
ffffc0207158: d63f0040 	blr	x2
ffffc020715c: aa0003f8 	mov	x24, x0
ffffc0207160: 17ffffe9 	b	0xffffc0207104 <paging_destroy+0x128>
ffffc0207164: b9400320 	ldr	w0, [x25]
ffffc0207168: 34000060 	cbz	w0, 0xffffc0207174 <paging_destroy+0x198>
ffffc020716c: aa1503e0 	mov	x0, x21
ffffc0207170: 97fffed3 	bl	0xffffc0206cbc <arch_paging_flush_page_tlbs>
ffffc0207174: eb16029f 	cmp	x20, x22
ffffc0207178: 54fff563 	b.lo	0xffffc0207024 <paging_destroy+0x48>
ffffc020717c: 8b1602b5 	add	x21, x21, x22
ffffc0207180: cb160294 	sub	x20, x20, x22
ffffc0207184: 17ffffa7 	b	0xffffc0207020 <paging_destroy+0x44>

0000ffffc0207188 <paging_create>:
ffffc0207188: a9b77bfd 	stp	x29, x30, [sp, #-144]!
ffffc020718c: 913ffc42 	add	x2, x2, #4095
ffffc0207190: 910003fd 	mov	x29, sp
ffffc0207194: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207198: 9274cc73 	and	x19, x3, #0xfffffffffffff000
ffffc020719c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02071a0: 9274cc55 	and	x21, x2, #0xfffffffffffff000
ffffc02071a4: a90363f7 	stp	x23, x24, [sp, #48]
ffffc02071a8: b0000097 	adrp	x23, 0xffffc0218000 <paging_create+0x64>
ffffc02071ac: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc02071b0: 910022f7 	add	x23, x23, #8
ffffc02071b4: 2a0503f9 	mov	w25, w5
ffffc02071b8: 9274cc38 	and	x24, x1, #0xfffffffffffff000
ffffc02071bc: aa0003f6 	mov	x22, x0
ffffc02071c0: a90573fb 	stp	x27, x28, [sp, #80]
ffffc02071c4: 910022e0 	add	x0, x23, #8
ffffc02071c8: a90613a0 	stp	x0, x4, [x29, #96]
ffffc02071cc: b5000075 	cbnz	x21, 0xffffc02071d8 <paging_create+0x50>
ffffc02071d0: 52800000 	mov	w0, #0
ffffc02071d4: 1400004c 	b	0xffffc0207304 <paging_create+0x17c>
ffffc02071d8: a940eeda 	ldp	x26, x27, [x22, #8]
ffffc02071dc: aa13031c 	orr	x28, x24, x19
ffffc02071e0: f9400743 	ldr	x3, [x26, #8]
ffffc02071e4: aa1303e1 	mov	x1, x19
ffffc02071e8: aa1b03e0 	mov	x0, x27
ffffc02071ec: d63f0060 	blr	x3
ffffc02071f0: aa0003f4 	mov	x20, x0
ffffc02071f4: b9400340 	ldr	w0, [x26]
ffffc02071f8: 34000420 	cbz	w0, 0xffffc020727c <paging_create+0xf4>
ffffc02071fc: eb2042bf 	cmp	x21, w0, uxtw
ffffc0207200: 540003e3 	b.lo	0xffffc020727c <paging_create+0xf4>
ffffc0207204: 51000401 	sub	w1, w0, #1
ffffc0207208: ea01039f 	tst	x28, x1
ffffc020720c: 54000381 	b.ne	0xffffc020727c <paging_create+0xf4>
ffffc0207210: 7140041f 	cmp	w0, #1, lsl #12         // =4096
ffffc0207214: 54000129 	b.ls	0xffffc0207238 <paging_create+0xb0>
ffffc0207218: b94002c0 	ldr	w0, [x22]
ffffc020721c: 2a1903e3 	mov	w3, w25
ffffc0207220: b9007ba0 	str	w0, [x29, #120]
ffffc0207224: aa1303e1 	mov	x1, x19
ffffc0207228: a9086fba 	stp	x26, x27, [x29, #128]
ffffc020722c: 9101e3a0 	add	x0, x29, #120
ffffc0207230: b9400342 	ldr	w2, [x26]
ffffc0207234: 97ffff6a 	bl	0xffffc0206fdc <paging_destroy>
ffffc0207238: f9400f43 	ldr	x3, [x26, #24]
ffffc020723c: aa1803e1 	mov	x1, x24
ffffc0207240: f94037a2 	ldr	x2, [x29, #104]
ffffc0207244: aa1403e0 	mov	x0, x20
ffffc0207248: d63f0060 	blr	x3
ffffc020724c: aa1403e0 	mov	x0, x20
ffffc0207250: 2a1903e1 	mov	w1, w25
ffffc0207254: 97ffff02 	bl	0xffffc0206e5c <flush_pt_entry>
ffffc0207258: b94002c0 	ldr	w0, [x22]
ffffc020725c: 34000060 	cbz	w0, 0xffffc0207268 <paging_create+0xe0>
ffffc0207260: aa1303e0 	mov	x0, x19
ffffc0207264: 97fffe96 	bl	0xffffc0206cbc <arch_paging_flush_page_tlbs>
ffffc0207268: b9400340 	ldr	w0, [x26]
ffffc020726c: 8b000318 	add	x24, x24, x0
ffffc0207270: 8b000273 	add	x19, x19, x0
ffffc0207274: cb0002b5 	sub	x21, x21, x0
ffffc0207278: 17ffffd5 	b	0xffffc02071cc <paging_create+0x44>
ffffc020727c: f9400b42 	ldr	x2, [x26, #16]
ffffc0207280: d2800021 	mov	x1, #1
ffffc0207284: aa1403e0 	mov	x0, x20
ffffc0207288: d63f0040 	blr	x2
ffffc020728c: 340001e0 	cbz	w0, 0xffffc02072c8 <paging_create+0x140>
ffffc0207290: b94002c0 	ldr	w0, [x22]
ffffc0207294: 2a1903e4 	mov	w4, w25
ffffc0207298: aa1303e3 	mov	x3, x19
ffffc020729c: aa1403e2 	mov	x2, x20
ffffc02072a0: aa1a03e1 	mov	x1, x26
ffffc02072a4: 9400001f 	bl	0xffffc0207320 <split_hugepage>
ffffc02072a8: 350002e0 	cbnz	w0, 0xffffc0207304 <paging_create+0x17c>
ffffc02072ac: f9401f41 	ldr	x1, [x26, #56]
ffffc02072b0: aa1403e0 	mov	x0, x20
ffffc02072b4: d63f0020 	blr	x1
ffffc02072b8: f94002e2 	ldr	x2, [x23]
ffffc02072bc: 8b02001b 	add	x27, x0, x2
ffffc02072c0: 9101435a 	add	x26, x26, #80
ffffc02072c4: 17ffffc7 	b	0xffffc02071e0 <paging_create+0x58>
ffffc02072c8: f94033a0 	ldr	x0, [x29, #96]
ffffc02072cc: 52800021 	mov	w1, #1
ffffc02072d0: 97fffeea 	bl	0xffffc0206e78 <page_alloc>
ffffc02072d4: aa0003fb 	mov	x27, x0
ffffc02072d8: b4000140 	cbz	x0, 0xffffc0207300 <paging_create+0x178>
ffffc02072dc: f9401b43 	ldr	x3, [x26, #48]
ffffc02072e0: aa1403e0 	mov	x0, x20
ffffc02072e4: f94002e1 	ldr	x1, [x23]
ffffc02072e8: cb010361 	sub	x1, x27, x1
ffffc02072ec: d63f0060 	blr	x3
ffffc02072f0: 2a1903e1 	mov	w1, w25
ffffc02072f4: aa1403e0 	mov	x0, x20
ffffc02072f8: 97fffed9 	bl	0xffffc0206e5c <flush_pt_entry>
ffffc02072fc: 17fffff1 	b	0xffffc02072c0 <paging_create+0x138>
ffffc0207300: 12800160 	mov	w0, #-12
ffffc0207304: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207308: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020730c: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0207310: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc0207314: a94573fb 	ldp	x27, x28, [sp, #80]
ffffc0207318: a8c97bfd 	ldp	x29, x30, [sp], #144
ffffc020731c: d65f03c0 	ret

0000ffffc0207320 <split_hugepage>:
ffffc0207320: a9b97bfd 	stp	x29, x30, [sp, #-112]!
ffffc0207324: 910003fd 	mov	x29, sp
ffffc0207328: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020732c: aa0103f3 	mov	x19, x1
ffffc0207330: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0207334: aa0203f5 	mov	x21, x2
ffffc0207338: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020733c: 2a0003f8 	mov	w24, w0
ffffc0207340: f90023f9 	str	x25, [sp, #64]
ffffc0207344: aa0203e0 	mov	x0, x2
ffffc0207348: aa0303f4 	mov	x20, x3
ffffc020734c: 2a0403f6 	mov	w22, w4
ffffc0207350: f9401022 	ldr	x2, [x1, #32]
ffffc0207354: aa0303e1 	mov	x1, x3
ffffc0207358: d63f0040 	blr	x2
ffffc020735c: b100041f 	cmn	x0, #1
ffffc0207360: 54000500 	b.eq	0xffffc0207400 <split_hugepage+0xe0>
ffffc0207364: b9400262 	ldr	w2, [x19]
ffffc0207368: f9401661 	ldr	x1, [x19, #40]
ffffc020736c: cb0203e2 	neg	x2, x2
ffffc0207370: 8a020017 	and	x23, x0, x2
ffffc0207374: 8a020294 	and	x20, x20, x2
ffffc0207378: aa1503e0 	mov	x0, x21
ffffc020737c: d63f0020 	blr	x1
ffffc0207380: aa0003f9 	mov	x25, x0
ffffc0207384: b0000092 	adrp	x18, 0xffffc0218000 <split_hugepage+0xa8>
ffffc0207388: 91014260 	add	x0, x19, #80
ffffc020738c: f90033a0 	str	x0, [x29, #96]
ffffc0207390: 91002240 	add	x0, x18, #8
ffffc0207394: 52800021 	mov	w1, #1
ffffc0207398: b9005bb8 	str	w24, [x29, #88]
ffffc020739c: 91002000 	add	x0, x0, #8
ffffc02073a0: 97fffeb6 	bl	0xffffc0206e78 <page_alloc>
ffffc02073a4: f90037a0 	str	x0, [x29, #104]
ffffc02073a8: b4000300 	cbz	x0, 0xffffc0207408 <split_hugepage+0xe8>
ffffc02073ac: f9401a62 	ldr	x2, [x19, #48]
ffffc02073b0: f9400641 	ldr	x1, [x18, #8]
ffffc02073b4: cb010001 	sub	x1, x0, x1
ffffc02073b8: aa1503e0 	mov	x0, x21
ffffc02073bc: d63f0040 	blr	x2
ffffc02073c0: 2a1603e1 	mov	w1, w22
ffffc02073c4: aa1503e0 	mov	x0, x21
ffffc02073c8: 97fffea5 	bl	0xffffc0206e5c <flush_pt_entry>
ffffc02073cc: b9400262 	ldr	w2, [x19]
ffffc02073d0: 2a1603e5 	mov	w5, w22
ffffc02073d4: aa1903e4 	mov	x4, x25
ffffc02073d8: aa1403e3 	mov	x3, x20
ffffc02073dc: aa1703e1 	mov	x1, x23
ffffc02073e0: 910163a0 	add	x0, x29, #88
ffffc02073e4: 97ffff69 	bl	0xffffc0207188 <paging_create>
ffffc02073e8: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02073ec: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02073f0: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc02073f4: f94023f9 	ldr	x25, [sp, #64]
ffffc02073f8: a8c77bfd 	ldp	x29, x30, [sp], #112
ffffc02073fc: d65f03c0 	ret
ffffc0207400: 52800000 	mov	w0, #0
ffffc0207404: 17fffff9 	b	0xffffc02073e8 <split_hugepage+0xc8>
ffffc0207408: 12800160 	mov	w0, #-12
ffffc020740c: 17fffff7 	b	0xffffc02073e8 <split_hugepage+0xc8>

0000ffffc0207410 <paging_map_device>:
ffffc0207410: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0207414: aa0103f2 	mov	x18, x1
ffffc0207418: 910003fd 	mov	x29, sp
ffffc020741c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0207420: 913ffc35 	add	x21, x1, #4095
ffffc0207424: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207428: f0000034 	adrp	x20, 0xffffc020e000 <paging_map_device+0x34>
ffffc020742c: d34cfeb5 	lsr	x21, x21, #12
ffffc0207430: 91058294 	add	x20, x20, #352
ffffc0207434: aa0003f6 	mov	x22, x0
ffffc0207438: 2a1503e1 	mov	w1, w21
ffffc020743c: aa1403e0 	mov	x0, x20
ffffc0207440: 97fffe8e 	bl	0xffffc0206e78 <page_alloc>
ffffc0207444: aa0003f3 	mov	x19, x0
ffffc0207448: b4000200 	cbz	x0, 0xffffc0207488 <paging_map_device+0x78>
ffffc020744c: aa0003e3 	mov	x3, x0
ffffc0207450: b0000080 	adrp	x0, 0xffffc0218000 <paging_map_device+0x84>
ffffc0207454: 91002000 	add	x0, x0, #8
ffffc0207458: 52800025 	mov	w5, #1
ffffc020745c: d280e8a4 	mov	x4, #1861
ffffc0207460: aa1203e2 	mov	x2, x18
ffffc0207464: aa1603e1 	mov	x1, x22
ffffc0207468: 9100c000 	add	x0, x0, #48
ffffc020746c: 97ffff47 	bl	0xffffc0207188 <paging_create>
ffffc0207470: 340000c0 	cbz	w0, 0xffffc0207488 <paging_map_device+0x78>
ffffc0207474: aa1303e1 	mov	x1, x19
ffffc0207478: d2800013 	mov	x19, #0
ffffc020747c: 2a1503e2 	mov	w2, w21
ffffc0207480: aa1403e0 	mov	x0, x20
ffffc0207484: 97fffe81 	bl	0xffffc0206e88 <page_free>
ffffc0207488: aa1303e0 	mov	x0, x19
ffffc020748c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207490: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0207494: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0207498: d65f03c0 	ret

0000ffffc020749c <paging_unmap_device>:
ffffc020749c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc02074a0: b0000080 	adrp	x0, 0xffffc0218000 <paging_create_hvpt_link>
ffffc02074a4: 91002000 	add	x0, x0, #8
ffffc02074a8: 52800023 	mov	w3, #1
ffffc02074ac: 910003fd 	mov	x29, sp
ffffc02074b0: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02074b4: 9100c000 	add	x0, x0, #48
ffffc02074b8: aa0103f4 	mov	x20, x1
ffffc02074bc: aa0203f3 	mov	x19, x2
ffffc02074c0: 97fffec7 	bl	0xffffc0206fdc <paging_destroy>
ffffc02074c4: 913ffe62 	add	x2, x19, #4095
ffffc02074c8: aa1403e1 	mov	x1, x20
ffffc02074cc: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02074d0: d34cac42 	ubfx	x2, x2, #12, #32
ffffc02074d4: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc02074d8: f0000020 	adrp	x0, 0xffffc020e000 <paging_create_hvpt_link+0x10>
ffffc02074dc: 91058000 	add	x0, x0, #352
ffffc02074e0: 17fffe6a 	b	0xffffc0206e88 <page_free>

0000ffffc02074e4 <paging_create_hvpt_link>:
ffffc02074e4: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc02074e8: 910003fd 	mov	x29, sp
ffffc02074ec: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02074f0: aa0003f6 	mov	x22, x0
ffffc02074f4: b0000080 	adrp	x0, 0xffffc0218000 <paging_create_hvpt_link+0x54>
ffffc02074f8: 91002000 	add	x0, x0, #8
ffffc02074fc: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207500: 9100c000 	add	x0, x0, #48
ffffc0207504: f9001bf7 	str	x23, [sp, #48]
ffffc0207508: aa0103f4 	mov	x20, x1
ffffc020750c: f9400413 	ldr	x19, [x0, #8]
ffffc0207510: f9400800 	ldr	x0, [x0, #16]
ffffc0207514: f9400662 	ldr	x2, [x19, #8]
ffffc0207518: d63f0040 	blr	x2
ffffc020751c: aa0003f5 	mov	x21, x0
ffffc0207520: f9400662 	ldr	x2, [x19, #8]
ffffc0207524: aa1403e1 	mov	x1, x20
ffffc0207528: f9400ac0 	ldr	x0, [x22, #16]
ffffc020752c: d63f0040 	blr	x2
ffffc0207530: aa0003f7 	mov	x23, x0
ffffc0207534: f9400a62 	ldr	x2, [x19, #16]
ffffc0207538: d2800021 	mov	x1, #1
ffffc020753c: aa1503e0 	mov	x0, x21
ffffc0207540: d63f0040 	blr	x2
ffffc0207544: 350000e0 	cbnz	w0, 0xffffc0207560 <paging_create_hvpt_link+0x7c>
ffffc0207548: 128002a0 	mov	w0, #-22
ffffc020754c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207550: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0207554: f9401bf7 	ldr	x23, [sp, #48]
ffffc0207558: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc020755c: d65f03c0 	ret
ffffc0207560: f9401262 	ldr	x2, [x19, #32]
ffffc0207564: aa1403e1 	mov	x1, x20
ffffc0207568: aa1503e0 	mov	x0, x21
ffffc020756c: d63f0040 	blr	x2
ffffc0207570: b100041f 	cmn	x0, #1
ffffc0207574: 54fffea1 	b.ne	0xffffc0207548 <paging_create_hvpt_link+0x64>
ffffc0207578: a9430676 	ldp	x22, x1, [x19, #48]
ffffc020757c: aa1503e0 	mov	x0, x21
ffffc0207580: d63f0020 	blr	x1
ffffc0207584: aa0003e1 	mov	x1, x0
ffffc0207588: aa1703e0 	mov	x0, x23
ffffc020758c: d63f02c0 	blr	x22
ffffc0207590: aa1403e0 	mov	x0, x20
ffffc0207594: 97fffdca 	bl	0xffffc0206cbc <arch_paging_flush_page_tlbs>
ffffc0207598: 52800000 	mov	w0, #0
ffffc020759c: 17ffffec 	b	0xffffc020754c <paging_create_hvpt_link+0x68>

0000ffffc02075a0 <paging_get_guest_pages>:
ffffc02075a0: 7100405f 	cmp	w2, #16
ffffc02075a4: 54000149 	b.ls	0xffffc02075cc <paging_get_guest_pages+0x2c>
ffffc02075a8: d2800000 	mov	x0, #0
ffffc02075ac: d65f03c0 	ret
ffffc02075b0: d2800000 	mov	x0, #0
ffffc02075b4: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02075b8: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02075bc: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc02075c0: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc02075c4: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc02075c8: d65f03c0 	ret
ffffc02075cc: a9bb7bfd 	stp	x29, x30, [sp, #-80]!
ffffc02075d0: 910003fd 	mov	x29, sp
ffffc02075d4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02075d8: b2641ff4 	mov	x20, #68451041280
ffffc02075dc: 8b224282 	add	x2, x20, w2, uxtw
ffffc02075e0: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc02075e4: d2820019 	mov	x25, #4096
ffffc02075e8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02075ec: a90363f7 	stp	x23, x24, [sp, #48]
ffffc02075f0: f2a00039 	movk	x25, #1, lsl #16
ffffc02075f4: aa0003f7 	mov	x23, x0
ffffc02075f8: aa0103f6 	mov	x22, x1
ffffc02075fc: aa0303f8 	mov	x24, x3
ffffc0207600: d374cc54 	lsl	x20, x2, #12
ffffc0207604: d2dfe015 	mov	x21, #280375465082880
ffffc0207608: f2dfe019 	movk	x25, #65280, lsl #32
ffffc020760c: eb1402bf 	cmp	x21, x20
ffffc0207610: 54000061 	b.ne	0xffffc020761c <paging_get_guest_pages+0x7c>
ffffc0207614: d2dfe000 	mov	x0, #280375465082880
ffffc0207618: 17ffffe7 	b	0xffffc02075b4 <paging_get_guest_pages+0x14>
ffffc020761c: b4000697 	cbz	x23, 0xffffc02076ec <paging_get_guest_pages+0x14c>
ffffc0207620: f94002f3 	ldr	x19, [x23]
ffffc0207624: b4000653 	cbz	x19, 0xffffc02076ec <paging_get_guest_pages+0x14c>
ffffc0207628: f94006e0 	ldr	x0, [x23, #8]
ffffc020762c: d280f821 	mov	x1, #1985
ffffc0207630: 97ffee2c 	bl	0xffffc0202ee0 <arch_paging_gphys2phys>
ffffc0207634: b100041f 	cmn	x0, #1
ffffc0207638: 54000221 	b.ne	0xffffc020767c <paging_get_guest_pages+0xdc>
ffffc020763c: 92800000 	mov	x0, #-1
ffffc0207640: aa1803e1 	mov	x1, x24
ffffc0207644: 97ffee27 	bl	0xffffc0202ee0 <arch_paging_gphys2phys>
ffffc0207648: b100041f 	cmn	x0, #1
ffffc020764c: 54fffb20 	b.eq	0xffffc02075b0 <paging_get_guest_pages+0x10>
ffffc0207650: aa0003e1 	mov	x1, x0
ffffc0207654: 52800025 	mov	w5, #1
ffffc0207658: aa1803e4 	mov	x4, x24
ffffc020765c: aa1503e3 	mov	x3, x21
ffffc0207660: d2820002 	mov	x2, #4096
ffffc0207664: aa1903e0 	mov	x0, x25
ffffc0207668: 97fffec8 	bl	0xffffc0207188 <paging_create>
ffffc020766c: 35fffa20 	cbnz	w0, 0xffffc02075b0 <paging_get_guest_pages+0x10>
ffffc0207670: 914006d6 	add	x22, x22, #1, lsl #12   // =4096
ffffc0207674: 914006b5 	add	x21, x21, #1, lsl #12   // =4096
ffffc0207678: 17ffffe5 	b	0xffffc020760c <paging_get_guest_pages+0x6c>
ffffc020767c: aa0003e1 	mov	x1, x0
ffffc0207680: 52800025 	mov	w5, #1
ffffc0207684: d280f824 	mov	x4, #1985
ffffc0207688: aa1503e3 	mov	x3, x21
ffffc020768c: d2820002 	mov	x2, #4096
ffffc0207690: aa1903e0 	mov	x0, x25
ffffc0207694: 97fffebd 	bl	0xffffc0207188 <paging_create>
ffffc0207698: 35fffd20 	cbnz	w0, 0xffffc020763c <paging_get_guest_pages+0x9c>
ffffc020769c: f9400662 	ldr	x2, [x19, #8]
ffffc02076a0: aa1603e1 	mov	x1, x22
ffffc02076a4: aa1503e0 	mov	x0, x21
ffffc02076a8: d63f0040 	blr	x2
ffffc02076ac: aa0003fa 	mov	x26, x0
ffffc02076b0: f9400a62 	ldr	x2, [x19, #16]
ffffc02076b4: aa1803e1 	mov	x1, x24
ffffc02076b8: d63f0040 	blr	x2
ffffc02076bc: 34fffc00 	cbz	w0, 0xffffc020763c <paging_get_guest_pages+0x9c>
ffffc02076c0: f9401262 	ldr	x2, [x19, #32]
ffffc02076c4: aa1603e1 	mov	x1, x22
ffffc02076c8: aa1a03e0 	mov	x0, x26
ffffc02076cc: d63f0040 	blr	x2
ffffc02076d0: b100041f 	cmn	x0, #1
ffffc02076d4: 54fffb61 	b.ne	0xffffc0207640 <paging_get_guest_pages+0xa0>
ffffc02076d8: f9401e61 	ldr	x1, [x19, #56]
ffffc02076dc: aa1a03e0 	mov	x0, x26
ffffc02076e0: 91014273 	add	x19, x19, #80
ffffc02076e4: d63f0020 	blr	x1
ffffc02076e8: 17ffffd1 	b	0xffffc020762c <paging_get_guest_pages+0x8c>
ffffc02076ec: aa1603e0 	mov	x0, x22
ffffc02076f0: 17ffffd4 	b	0xffffc0207640 <paging_get_guest_pages+0xa0>

0000ffffc02076f4 <paging_map_all_per_cpu>:
ffffc02076f4: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc02076f8: d00000a3 	adrp	x3, 0xffffc021d000 <paging_init+0x20>
ffffc02076fc: 91000063 	add	x3, x3, #0
ffffc0207700: 7100003f 	cmp	w1, #0
ffffc0207704: 8b030003 	add	x3, x0, x3
ffffc0207708: b0000080 	adrp	x0, 0xffffc0218000 <paging_init+0x1c>
ffffc020770c: 52800025 	mov	w5, #1
ffffc0207710: d280e824 	mov	x4, #1857
ffffc0207714: f9400406 	ldr	x6, [x0, #8]
ffffc0207718: 91002000 	add	x0, x0, #8
ffffc020771c: 9a9f1084 	csel	x4, x4, xzr, ne
ffffc0207720: d2840002 	mov	x2, #8192
ffffc0207724: cb060061 	sub	x1, x3, x6
ffffc0207728: 9100c000 	add	x0, x0, #48
ffffc020772c: 17fffe97 	b	0xffffc0207188 <paging_create>

0000ffffc0207730 <paging_init>:
ffffc0207730: a9bb7bfd 	stp	x29, x30, [sp, #-80]!
ffffc0207734: 528000e5 	mov	w5, #7
ffffc0207738: 910003fd 	mov	x29, sp
ffffc020773c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0207740: d0000096 	adrp	x22, 0xffffc0219000 <paging_init+0x58>
ffffc0207744: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207748: b0ffffd4 	adrp	x20, 0xffffc0200000 <paging_map_all_per_cpu+0x38>
ffffc020774c: f94002c4 	ldr	x4, [x22]
ffffc0207750: b0000093 	adrp	x19, 0xffffc0218000 <paging_init+0x64>
ffffc0207754: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0207758: 91000297 	add	x23, x20, #0
ffffc020775c: 91042880 	add	x0, x4, #266
ffffc0207760: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc0207764: 91002272 	add	x18, x19, #8
ffffc0207768: b94032e3 	ldr	w3, [x23, #48]
ffffc020776c: 29488402 	ldp	w2, w1, [x0, #68]
ffffc0207770: 1b050442 	madd	w2, w2, w5, w1
ffffc0207774: b9403c05 	ldr	w5, [x0, #60]
ffffc0207778: b9403401 	ldr	w1, [x0, #52]
ffffc020777c: 0b050021 	add	w1, w1, w5
ffffc0207780: b9404005 	ldr	w5, [x0, #64]
ffffc0207784: 531b6821 	lsl	w1, w1, #5
ffffc0207788: 0b020c22 	add	w2, w1, w2, lsl #3
ffffc020778c: b9403001 	ldr	w1, [x0, #48]
ffffc0207790: b9403800 	ldr	w0, [x0, #56]
ffffc0207794: 0b050021 	add	w1, w1, w5
ffffc0207798: 52800185 	mov	w5, #12
ffffc020779c: 1b050400 	madd	w0, w0, w5, w1
ffffc02077a0: 52823121 	mov	w1, #4489
ffffc02077a4: 91002245 	add	x5, x18, #8
ffffc02077a8: 0b010000 	add	w0, w0, w1
ffffc02077ac: f840c081 	ldur	x1, [x4, #12]
ffffc02077b0: 0b000042 	add	w2, w2, w0
ffffc02077b4: d2b80400 	mov	x0, #3223322624
ffffc02077b8: f2dfffe0 	movk	x0, #65535, lsl #32
ffffc02077bc: cb010000 	sub	x0, x0, x1
ffffc02077c0: f841c081 	ldur	x1, [x4, #28]
ffffc02077c4: d00000a4 	adrp	x4, 0xffffc021d000 <paging_init+0xec>
ffffc02077c8: 91000084 	add	x4, x4, #0
ffffc02077cc: 530c7c42 	lsr	w2, w2, #12
ffffc02077d0: cb040021 	sub	x1, x1, x4
ffffc02077d4: 8b030842 	add	x2, x2, x3, lsl #2
ffffc02077d8: 8b0102e1 	add	x1, x23, x1
ffffc02077dc: d28fffe3 	mov	x3, #32767
ffffc02077e0: f9000660 	str	x0, [x19, #8]
ffffc02077e4: d34cfc21 	lsr	x1, x1, #12
ffffc02077e8: f90004a1 	str	x1, [x5, #8]
ffffc02077ec: 8b030023 	add	x3, x1, x3
ffffc02077f0: 8b433c43 	add	x3, x2, x3, lsr #15
ffffc02077f4: eb03003f 	cmp	x1, x3
ffffc02077f8: 54000128 	b.hi	0xffffc020781c <paging_init+0xec>
ffffc02077fc: 12800177 	mov	w23, #-12
ffffc0207800: 2a1703e0 	mov	w0, w23
ffffc0207804: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207808: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020780c: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0207810: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc0207814: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc0207818: d65f03c0 	ret
ffffc020781c: 8b023082 	add	x2, x4, x2, lsl #12
ffffc0207820: d2800001 	mov	x1, #0
ffffc0207824: a90108a3 	stp	x3, x2, [x5, #16]
ffffc0207828: d2800023 	mov	x3, #1
ffffc020782c: f9000644 	str	x4, [x18, #8]
ffffc0207830: 91002259 	add	x25, x18, #8
ffffc0207834: f9400b20 	ldr	x0, [x25, #16]
ffffc0207838: eb01001f 	cmp	x0, x1
ffffc020783c: 540008c8 	b.hi	0xffffc0207954 <paging_init+0x224>
ffffc0207840: d2800038 	mov	x24, #1
ffffc0207844: 52800081 	mov	w1, #4
ffffc0207848: aa1903e0 	mov	x0, x25
ffffc020784c: f9001338 	str	x24, [x25, #32]
ffffc0207850: 97fffd8a 	bl	0xffffc0206e78 <page_alloc>
ffffc0207854: f0000035 	adrp	x21, 0xffffc020e000 <paging_init+0x140>
ffffc0207858: 910582a1 	add	x1, x21, #352
ffffc020785c: 9100c25a 	add	x26, x18, #48
ffffc0207860: b9003258 	str	w24, [x18, #48]
ffffc0207864: f9000c20 	str	x0, [x1, #24]
ffffc0207868: 900000c0 	adrp	x0, 0xffffc021f000 <paging_init+0x198>
ffffc020786c: 91000000 	add	x0, x0, #0
ffffc0207870: f9002240 	str	x0, [x18, #64]
ffffc0207874: 97ffee86 	bl	0xffffc020328c <arch_paging_init>
ffffc0207878: d0000060 	adrp	x0, 0xffffc0215000 <paging_init+0x180>
ffffc020787c: 91002272 	add	x18, x19, #8
ffffc0207880: b940f001 	ldr	w1, [x0, #240]
ffffc0207884: 5100a020 	sub	w0, w1, #40
ffffc0207888: 51009c21 	sub	w1, w1, #39
ffffc020788c: 7100101f 	cmp	w0, #4
ffffc0207890: aa1903e0 	mov	x0, x25
ffffc0207894: 1ac12301 	lsl	w1, w24, w1
ffffc0207898: 1a812301 	csel	w1, w24, w1, hs
ffffc020789c: 97fffd79 	bl	0xffffc0206e80 <page_alloc_aligned>
ffffc02078a0: f9002e40 	str	x0, [x18, #88]
ffffc02078a4: b4fffac0 	cbz	x0, 0xffffc02077fc <paging_init+0xcc>
ffffc02078a8: f94002c0 	ldr	x0, [x22]
ffffc02078ac: aa1703e3 	mov	x3, x23
ffffc02078b0: f9400661 	ldr	x1, [x19, #8]
ffffc02078b4: 2a1803e5 	mov	w5, w24
ffffc02078b8: d280e824 	mov	x4, #1857
ffffc02078bc: f841c002 	ldur	x2, [x0, #28]
ffffc02078c0: cb0102e1 	sub	x1, x23, x1
ffffc02078c4: aa1a03e0 	mov	x0, x26
ffffc02078c8: 97fffe30 	bl	0xffffc0207188 <paging_create>
ffffc02078cc: 2a0003f7 	mov	w23, w0
ffffc02078d0: 35fff980 	cbnz	w0, 0xffffc0207800 <paging_init+0xd0>
ffffc02078d4: 91000294 	add	x20, x20, #0
ffffc02078d8: d2800018 	mov	x24, #0
ffffc02078dc: b9403280 	ldr	w0, [x20, #48]
ffffc02078e0: eb18001f 	cmp	x0, x24
ffffc02078e4: 54000508 	b.hi	0xffffc0207984 <paging_init+0x254>
ffffc02078e8: f94002c1 	ldr	x1, [x22]
ffffc02078ec: 79407420 	ldrh	w0, [x1, #58]
ffffc02078f0: 3607f880 	tbz	w0, #0, 0xffffc0207800 <paging_init+0xd0>
ffffc02078f4: b9403422 	ldr	w2, [x1, #52]
ffffc02078f8: b2407fe0 	mov	x0, #4294967295
ffffc02078fc: f9401e83 	ldr	x3, [x20, #56]
ffffc0207900: f2dfefe0 	movk	x0, #65407, lsl #32
ffffc0207904: 8b030044 	add	x4, x2, x3
ffffc0207908: eb00009f 	cmp	x4, x0
ffffc020790c: 540000e9 	b.ls	0xffffc0207928 <paging_init+0x1f8>
ffffc0207910: 910582b5 	add	x21, x21, #352
ffffc0207914: b26523e4 	mov	x4, #68585259008
ffffc0207918: f94006a0 	ldr	x0, [x21, #8]
ffffc020791c: 8b040000 	add	x0, x0, x4
ffffc0207920: eb00307f 	cmp	x3, x0, lsl #12
ffffc0207924: 54000403 	b.lo	0xffffc02079a4 <paging_init+0x274>
ffffc0207928: 91002260 	add	x0, x19, #8
ffffc020792c: f842c021 	ldur	x1, [x1, #44]
ffffc0207930: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207934: 52800025 	mov	w5, #1
ffffc0207938: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020793c: d280e8a4 	mov	x4, #1861
ffffc0207940: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0207944: 9100c000 	add	x0, x0, #48
ffffc0207948: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc020794c: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc0207950: 17fffe0e 	b	0xffffc0207188 <paging_create>
ffffc0207954: f9400f22 	ldr	x2, [x25, #24]
ffffc0207958: 53067c20 	lsr	w0, w1, #6
ffffc020795c: 927df042 	and	x2, x2, #0xfffffffffffffff8
ffffc0207960: 8b204c40 	add	x0, x2, w0, uxtw #3
ffffc0207964: 12001422 	and	w2, w1, #0x3f
ffffc0207968: 9ac22062 	lsl	x2, x3, x2
ffffc020796c: c85f7c05 	ldxr	x5, [x0]
ffffc0207970: aa0200a5 	orr	x5, x5, x2
ffffc0207974: c8047c05 	stxr	w4, x5, [x0]
ffffc0207978: 35ffffa4 	cbnz	w4, 0xffffc020796c <paging_init+0x23c>
ffffc020797c: 91000421 	add	x1, x1, #1
ffffc0207980: 17ffffac 	b	0xffffc0207830 <paging_init+0x100>
ffffc0207984: 52800021 	mov	w1, #1
ffffc0207988: 2a1803e0 	mov	w0, w24
ffffc020798c: 97ffff5a 	bl	0xffffc02076f4 <paging_map_all_per_cpu>
ffffc0207990: 35000060 	cbnz	w0, 0xffffc020799c <paging_init+0x26c>
ffffc0207994: 91000718 	add	x24, x24, #1
ffffc0207998: 17ffffd1 	b	0xffffc02078dc <paging_init+0x1ac>
ffffc020799c: 2a0003f7 	mov	w23, w0
ffffc02079a0: 17ffff98 	b	0xffffc0207800 <paging_init+0xd0>
ffffc02079a4: 128002b7 	mov	w23, #-22
ffffc02079a8: 17ffff96 	b	0xffffc0207800 <paging_init+0xd0>

0000ffffc02079ac <paging_dump_stats>:
ffffc02079ac: f0000022 	adrp	x2, 0xffffc020e000 <paging_dump_stats+0x1c>
ffffc02079b0: b0000081 	adrp	x1, 0xffffc0218000 <spin_lock+0x1c>
ffffc02079b4: 91058042 	add	x2, x2, #352
ffffc02079b8: 91002021 	add	x1, x1, #8
ffffc02079bc: a9409045 	ldp	x5, x4, [x2, #8]
ffffc02079c0: f9400823 	ldr	x3, [x1, #16]
ffffc02079c4: f9400c22 	ldr	x2, [x1, #24]
ffffc02079c8: aa0003e1 	mov	x1, x0
ffffc02079cc: 90000020 	adrp	x0, 0xffffc020b000 <spin_lock+0x4>
ffffc02079d0: 913a0000 	add	x0, x0, #3712
ffffc02079d4: 17fffc54 	b	0xffffc0206b24 <printk>

0000ffffc02079d8 <spin_lock>:
ffffc02079d8: f9800011 	prfm	pstl1strm, [x0]
ffffc02079dc: 885ffc01 	ldaxr	w1, [x0]
ffffc02079e0: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc02079e4: 88037c02 	stxr	w3, w2, [x0]
ffffc02079e8: 35ffffa3 	cbnz	w3, 0xffffc02079dc <spin_lock+0x4>
ffffc02079ec: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc02079f0: 340000c2 	cbz	w2, 0xffffc0207a08 <spin_lock+0x30>
ffffc02079f4: d50320bf 	sevl
ffffc02079f8: d503205f 	wfe
ffffc02079fc: 485ffc03 	ldaxrh	w3, [x0]
ffffc0207a00: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc0207a04: 35ffffa2 	cbnz	w2, 0xffffc02079f8 <spin_lock+0x20>
ffffc0207a08: d65f03c0 	ret

0000ffffc0207a0c <spin_unlock>:
ffffc0207a0c: 79400001 	ldrh	w1, [x0]
ffffc0207a10: 11000421 	add	w1, w1, #1
ffffc0207a14: 489ffc01 	stlrh	w1, [x0]
ffffc0207a18: d65f03c0 	ret

0000ffffc0207a1c <cell_owns_cpu>:
ffffc0207a1c: f9485400 	ldr	x0, [x0, #4264]
ffffc0207a20: f9400002 	ldr	x2, [x0]
ffffc0207a24: eb21405f 	cmp	x2, w1, uxtw
ffffc0207a28: 54000123 	b.lo	0xffffc0207a4c <cell_owns_cpu+0x30>
ffffc0207a2c: 53067c22 	lsr	w2, w1, #6
ffffc0207a30: 91002000 	add	x0, x0, #8
ffffc0207a34: 12001421 	and	w1, w1, #0x3f
ffffc0207a38: d37df042 	lsl	x2, x2, #3
ffffc0207a3c: f8626800 	ldr	x0, [x0, x2]
ffffc0207a40: 9ac12400 	lsr	x0, x0, x1
ffffc0207a44: 12000000 	and	w0, w0, #0x1
ffffc0207a48: d65f03c0 	ret
ffffc0207a4c: 52800000 	mov	w0, #0
ffffc0207a50: 17fffffe 	b	0xffffc0207a48 <cell_owns_cpu+0x2c>

0000ffffc0207a54 <unmap_from_root_cell>:
ffffc0207a54: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0207a58: 910003fd 	mov	x29, sp
ffffc0207a5c: f9400001 	ldr	x1, [x0]
ffffc0207a60: a9410c02 	ldp	x2, x3, [x0, #16]
ffffc0207a64: a90107a1 	stp	x1, x1, [x29, #16]
ffffc0207a68: a9020fa2 	stp	x2, x3, [x29, #32]
ffffc0207a6c: f2402c3f 	tst	x1, #0xfff
ffffc0207a70: f0000080 	adrp	x0, 0xffffc021a000 <cell_exit+0x10>
ffffc0207a74: 54000081 	b.ne	0xffffc0207a84 <unmap_from_root_cell+0x30>
ffffc0207a78: f94013a1 	ldr	x1, [x29, #32]
ffffc0207a7c: f2402c3f 	tst	x1, #0xfff
ffffc0207a80: 540000e0 	b.eq	0xffffc0207a9c <unmap_from_root_cell+0x48>
ffffc0207a84: 91000000 	add	x0, x0, #0
ffffc0207a88: 910043a1 	add	x1, x29, #16
ffffc0207a8c: 94000684 	bl	0xffffc020949c <mmio_subpage_unregister>
ffffc0207a90: 52800000 	mov	w0, #0
ffffc0207a94: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0207a98: d65f03c0 	ret
ffffc0207a9c: 910043a1 	add	x1, x29, #16
ffffc0207aa0: 91000000 	add	x0, x0, #0
ffffc0207aa4: 97ffed0a 	bl	0xffffc0202ecc <arch_unmap_memory_region>
ffffc0207aa8: 17fffffb 	b	0xffffc0207a94 <unmap_from_root_cell+0x40>

0000ffffc0207aac <cell_exit>:
ffffc0207aac: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0207ab0: 910003fd 	mov	x29, sp
ffffc0207ab4: f9000bf3 	str	x19, [sp, #16]
ffffc0207ab8: aa0003f3 	mov	x19, x0
ffffc0207abc: 940005fe 	bl	0xffffc02092b4 <mmio_cell_exit>
ffffc0207ac0: d2821600 	mov	x0, #4272
ffffc0207ac4: f9485661 	ldr	x1, [x19, #4264]
ffffc0207ac8: 8b000273 	add	x19, x19, x0
ffffc0207acc: eb13003f 	cmp	x1, x19
ffffc0207ad0: 540000e0 	b.eq	0xffffc0207aec <cell_exit+0x40>
ffffc0207ad4: f9400bf3 	ldr	x19, [sp, #16]
ffffc0207ad8: 52800022 	mov	w2, #1
ffffc0207adc: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0207ae0: b0000080 	adrp	x0, 0xffffc0218000 <remap_to_root_cell+0x2c>
ffffc0207ae4: 91004000 	add	x0, x0, #16
ffffc0207ae8: 17fffce8 	b	0xffffc0206e88 <page_free>
ffffc0207aec: f9400bf3 	ldr	x19, [sp, #16]
ffffc0207af0: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0207af4: d65f03c0 	ret

0000ffffc0207af8 <remap_to_root_cell>:
ffffc0207af8: a9b97bfd 	stp	x29, x30, [sp, #-112]!
ffffc0207afc: 910003fd 	mov	x29, sp
ffffc0207b00: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207b04: f0000094 	adrp	x20, 0xffffc021a000 <remap_to_root_cell+0x58>
ffffc0207b08: 91000294 	add	x20, x20, #0
ffffc0207b0c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0207b10: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0207b14: aa0003f6 	mov	x22, x0
ffffc0207b18: f90023f9 	str	x25, [sp, #64]
ffffc0207b1c: 90000037 	adrp	x23, 0xffffc020b000 <remap_to_root_cell+0x34>
ffffc0207b20: f9485293 	ldr	x19, [x20, #4256]
ffffc0207b24: 2a0103f8 	mov	w24, w1
ffffc0207b28: 913ac2f7 	add	x23, x23, #3760
ffffc0207b2c: 52800015 	mov	w21, #0
ffffc0207b30: 52800019 	mov	w25, #0
ffffc0207b34: b9403260 	ldr	w0, [x19, #48]
ffffc0207b38: 91020000 	add	x0, x0, #128
ffffc0207b3c: 8b000273 	add	x19, x19, x0
ffffc0207b40: f9485280 	ldr	x0, [x20, #4256]
ffffc0207b44: b9403400 	ldr	w0, [x0, #52]
ffffc0207b48: 6b15001f 	cmp	w0, w21
ffffc0207b4c: 54000108 	b.hi	0xffffc0207b6c <remap_to_root_cell+0x74>
ffffc0207b50: 2a1903e0 	mov	w0, w25
ffffc0207b54: f94023f9 	ldr	x25, [sp, #64]
ffffc0207b58: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207b5c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0207b60: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0207b64: a8c77bfd 	ldp	x29, x30, [sp], #112
ffffc0207b68: d65f03c0 	ret
ffffc0207b6c: f9400262 	ldr	x2, [x19]
ffffc0207b70: f94002c0 	ldr	x0, [x22]
ffffc0207b74: eb02001f 	cmp	x0, x2
ffffc0207b78: 54000503 	b.lo	0xffffc0207c18 <remap_to_root_cell+0x120>
ffffc0207b7c: f9400a61 	ldr	x1, [x19, #16]
ffffc0207b80: 8b010043 	add	x3, x2, x1
ffffc0207b84: eb03001f 	cmp	x0, x3
ffffc0207b88: 54000442 	b.hs	0xffffc0207c10 <remap_to_root_cell+0x118>
ffffc0207b8c: f9002ba0 	str	x0, [x29, #80]
ffffc0207b90: f9400262 	ldr	x2, [x19]
ffffc0207b94: 8b020021 	add	x1, x1, x2
ffffc0207b98: cb000020 	sub	x0, x1, x0
ffffc0207b9c: f9400ac1 	ldr	x1, [x22, #16]
ffffc0207ba0: f90033a0 	str	x0, [x29, #96]
ffffc0207ba4: eb01001f 	cmp	x0, x1
ffffc0207ba8: 54000049 	b.ls	0xffffc0207bb0 <remap_to_root_cell+0xb8>
ffffc0207bac: f90033a1 	str	x1, [x29, #96]
ffffc0207bb0: f9400660 	ldr	x0, [x19, #8]
ffffc0207bb4: f9402ba1 	ldr	x1, [x29, #80]
ffffc0207bb8: 8b010000 	add	x0, x0, x1
ffffc0207bbc: f9400261 	ldr	x1, [x19]
ffffc0207bc0: cb010000 	sub	x0, x0, x1
ffffc0207bc4: f9002fa0 	str	x0, [x29, #88]
ffffc0207bc8: f9400e61 	ldr	x1, [x19, #24]
ffffc0207bcc: f2402c1f 	tst	x0, #0xfff
ffffc0207bd0: f90037a1 	str	x1, [x29, #104]
ffffc0207bd4: 910143a1 	add	x1, x29, #80
ffffc0207bd8: 54000081 	b.ne	0xffffc0207be8 <remap_to_root_cell+0xf0>
ffffc0207bdc: f94033a0 	ldr	x0, [x29, #96]
ffffc0207be0: f2402c1f 	tst	x0, #0xfff
ffffc0207be4: 540002c0 	b.eq	0xffffc0207c3c <remap_to_root_cell+0x144>
ffffc0207be8: aa1403e0 	mov	x0, x20
ffffc0207bec: 94000622 	bl	0xffffc0209474 <mmio_subpage_register>
ffffc0207bf0: 2a0003f9 	mov	w25, w0
ffffc0207bf4: 34000080 	cbz	w0, 0xffffc0207c04 <remap_to_root_cell+0x10c>
ffffc0207bf8: 34fffad8 	cbz	w24, 0xffffc0207b50 <remap_to_root_cell+0x58>
ffffc0207bfc: aa1703e0 	mov	x0, x23
ffffc0207c00: 97fffbc9 	bl	0xffffc0206b24 <printk>
ffffc0207c04: 91008273 	add	x19, x19, #32
ffffc0207c08: 110006b5 	add	w21, w21, #1
ffffc0207c0c: 17ffffcd 	b	0xffffc0207b40 <remap_to_root_cell+0x48>
ffffc0207c10: eb02001f 	cmp	x0, x2
ffffc0207c14: 54ffff88 	b.hi	0xffffc0207c04 <remap_to_root_cell+0x10c>
ffffc0207c18: f9400ac1 	ldr	x1, [x22, #16]
ffffc0207c1c: 8b010000 	add	x0, x0, x1
ffffc0207c20: eb00005f 	cmp	x2, x0
ffffc0207c24: 54ffff02 	b.hs	0xffffc0207c04 <remap_to_root_cell+0x10c>
ffffc0207c28: cb020000 	sub	x0, x0, x2
ffffc0207c2c: f9002ba2 	str	x2, [x29, #80]
ffffc0207c30: f90033a0 	str	x0, [x29, #96]
ffffc0207c34: f9400a61 	ldr	x1, [x19, #16]
ffffc0207c38: 17ffffdb 	b	0xffffc0207ba4 <remap_to_root_cell+0xac>
ffffc0207c3c: aa1403e0 	mov	x0, x20
ffffc0207c40: 97ffec8e 	bl	0xffffc0202e78 <arch_map_memory_region>
ffffc0207c44: 17ffffeb 	b	0xffffc0207bf0 <remap_to_root_cell+0xf8>

0000ffffc0207c48 <next_cpu>:
ffffc0207c48: f8408425 	ldr	x5, [x1], #8
ffffc0207c4c: 11000400 	add	w0, w0, #1
ffffc0207c50: eb2040bf 	cmp	x5, w0, uxtw
ffffc0207c54: 54000123 	b.lo	0xffffc0207c78 <next_cpu+0x30>
ffffc0207c58: 6b00005f 	cmp	w2, w0
ffffc0207c5c: 54ffff80 	b.eq	0xffffc0207c4c <next_cpu+0x4>
ffffc0207c60: 53067c03 	lsr	w3, w0, #6
ffffc0207c64: 12001404 	and	w4, w0, #0x3f
ffffc0207c68: d37df063 	lsl	x3, x3, #3
ffffc0207c6c: f8636823 	ldr	x3, [x1, x3]
ffffc0207c70: 9ac42463 	lsr	x3, x3, x4
ffffc0207c74: 3607fec3 	tbz	w3, #0, 0xffffc0207c4c <next_cpu+0x4>
ffffc0207c78: d65f03c0 	ret

0000ffffc0207c7c <cell_suspend>:
ffffc0207c7c: a9bb7bfd 	stp	x29, x30, [sp, #-80]!
ffffc0207c80: 910003fd 	mov	x29, sp
ffffc0207c84: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0207c88: d2840017 	mov	x23, #8192
ffffc0207c8c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0207c90: f2a00037 	movk	x23, #1, lsl #16
ffffc0207c94: d00000b5 	adrp	x21, 0xffffc021d000 <cell_suspend+0x70>
ffffc0207c98: aa0003f6 	mov	x22, x0
ffffc0207c9c: 910002b5 	add	x21, x21, #0
ffffc0207ca0: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207ca4: f90023f9 	str	x25, [sp, #64]
ffffc0207ca8: 12800014 	mov	w20, #-1
ffffc0207cac: f2dfe017 	movk	x23, #65280, lsl #32
ffffc0207cb0: d2828d98 	mov	x24, #5228
ffffc0207cb4: 52800039 	mov	w25, #1
ffffc0207cb8: b95002e2 	ldr	w2, [x23, #4096]
ffffc0207cbc: 2a1403e0 	mov	w0, w20
ffffc0207cc0: f94856c1 	ldr	x1, [x22, #4264]
ffffc0207cc4: 97ffffe1 	bl	0xffffc0207c48 <next_cpu>
ffffc0207cc8: 2a0003f3 	mov	w19, w0
ffffc0207ccc: f94856c0 	ldr	x0, [x22, #4264]
ffffc0207cd0: aa1303f4 	mov	x20, x19
ffffc0207cd4: f9400000 	ldr	x0, [x0]
ffffc0207cd8: eb00027f 	cmp	x19, x0
ffffc0207cdc: 540000e9 	b.ls	0xffffc0207cf8 <cell_suspend+0x7c>
ffffc0207ce0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0207ce4: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0207ce8: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0207cec: f94023f9 	ldr	x25, [sp, #64]
ffffc0207cf0: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc0207cf4: d65f03c0 	ret
ffffc0207cf8: 8b133ab3 	add	x19, x21, x19, lsl #14
ffffc0207cfc: 91400a73 	add	x19, x19, #2, lsl #12   // =8192
ffffc0207d00: 8b180264 	add	x4, x19, x24
ffffc0207d04: aa0403e0 	mov	x0, x4
ffffc0207d08: 97ffff34 	bl	0xffffc02079d8 <spin_lock>
ffffc0207d0c: b9103a79 	str	w25, [x19, #4152]
ffffc0207d10: b9503e62 	ldr	w2, [x19, #4156]
ffffc0207d14: 97ffff3e 	bl	0xffffc0207a0c <spin_unlock>
ffffc0207d18: 35fffd02 	cbnz	w2, 0xffffc0207cb8 <cell_suspend+0x3c>
ffffc0207d1c: b9500260 	ldr	w0, [x19, #4096]
ffffc0207d20: 97ffe2db 	bl	0xffffc020088c <arm_cpu_kick>
ffffc0207d24: b9503e60 	ldr	w0, [x19, #4156]
ffffc0207d28: 35fffc80 	cbnz	w0, 0xffffc0207cb8 <cell_suspend+0x3c>
ffffc0207d2c: 17fffffe 	b	0xffffc0207d24 <cell_suspend+0xa8>

0000ffffc0207d30 <cell_exchange_message>:
ffffc0207d30: f9485003 	ldr	x3, [x0, #4256]
ffffc0207d34: b9402c64 	ldr	w4, [x3, #44]
ffffc0207d38: 37000524 	tbnz	w4, #0, 0xffffc0207ddc <cell_exchange_message+0xac>
ffffc0207d3c: f9402c63 	ldr	x3, [x3, #88]
ffffc0207d40: b900101f 	str	wzr, [x0, #16]
ffffc0207d44: d5033abf 	dmb	ishst
ffffc0207d48: b9000c01 	str	w1, [x0, #12]
ffffc0207d4c: 52000041 	eor	w1, w2, #0x1
ffffc0207d50: 12000021 	and	w1, w1, #0x1
ffffc0207d54: 12000042 	and	w2, w2, #0x1
ffffc0207d58: b9401004 	ldr	w4, [x0, #16]
ffffc0207d5c: b9400805 	ldr	w5, [x0, #8]
ffffc0207d60: 510008a5 	sub	w5, w5, #2
ffffc0207d64: 710004bf 	cmp	w5, #1
ffffc0207d68: 540003a9 	b.ls	0xffffc0207ddc <cell_exchange_message+0xac>
ffffc0207d6c: 71000c9f 	cmp	w4, #3
ffffc0207d70: 1a9f0025 	csel	w5, w1, wzr, eq
ffffc0207d74: 35000345 	cbnz	w5, 0xffffc0207ddc <cell_exchange_message+0xac>
ffffc0207d78: 7100109f 	cmp	w4, #4
ffffc0207d7c: 1a9f0045 	csel	w5, w2, wzr, eq
ffffc0207d80: 350002e5 	cbnz	w5, 0xffffc0207ddc <cell_exchange_message+0xac>
ffffc0207d84: 35000304 	cbnz	w4, 0xffffc0207de4 <cell_exchange_message+0xb4>
ffffc0207d88: f9485004 	ldr	x4, [x0, #4256]
ffffc0207d8c: f9402c84 	ldr	x4, [x4, #88]
ffffc0207d90: b4000244 	cbz	x4, 0xffffc0207dd8 <cell_exchange_message+0xa8>
ffffc0207d94: f1000463 	subs	x3, x3, #1
ffffc0207d98: 54000201 	b.ne	0xffffc0207dd8 <cell_exchange_message+0xa8>
ffffc0207d9c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0207da0: 910003fd 	mov	x29, sp
ffffc0207da4: f9000bf3 	str	x19, [sp, #16]
ffffc0207da8: aa0003f3 	mov	x19, x0
ffffc0207dac: 90000020 	adrp	x0, 0xffffc020b000 <cell_exchange_message+0x8c>
ffffc0207db0: 913ba400 	add	x0, x0, #3817
ffffc0207db4: 97fffb5c 	bl	0xffffc0206b24 <printk>
ffffc0207db8: aa1303e0 	mov	x0, x19
ffffc0207dbc: 97ffffb0 	bl	0xffffc0207c7c <cell_suspend>
ffffc0207dc0: 52800060 	mov	w0, #3
ffffc0207dc4: b9000a60 	str	w0, [x19, #8]
ffffc0207dc8: 52800020 	mov	w0, #1
ffffc0207dcc: f9400bf3 	ldr	x19, [sp, #16]
ffffc0207dd0: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0207dd4: d65f03c0 	ret
ffffc0207dd8: 17ffffe0 	b	0xffffc0207d58 <cell_exchange_message+0x28>
ffffc0207ddc: 52800020 	mov	w0, #1
ffffc0207de0: d65f03c0 	ret
ffffc0207de4: 52800000 	mov	w0, #0
ffffc0207de8: d65f03c0 	ret

0000ffffc0207dec <cell_reconfig_completed>:
ffffc0207dec: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0207df0: 900000a0 	adrp	x0, 0xffffc021b000 <cpu_id_valid+0x18>
ffffc0207df4: 910003fd 	mov	x29, sp
ffffc0207df8: f9000bf3 	str	x19, [sp, #16]
ffffc0207dfc: f9406413 	ldr	x19, [x0, #200]
ffffc0207e00: b5000093 	cbnz	x19, 0xffffc0207e10 <cell_reconfig_completed+0x24>
ffffc0207e04: f9400bf3 	ldr	x19, [sp, #16]
ffffc0207e08: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0207e0c: d65f03c0 	ret
ffffc0207e10: aa1303e0 	mov	x0, x19
ffffc0207e14: 52800022 	mov	w2, #1
ffffc0207e18: 52800041 	mov	w1, #2
ffffc0207e1c: 97ffffc5 	bl	0xffffc0207d30 <cell_exchange_message>
ffffc0207e20: f9486673 	ldr	x19, [x19, #4296]
ffffc0207e24: 17fffff7 	b	0xffffc0207e00 <cell_reconfig_completed+0x14>

0000ffffc0207e28 <cpu_id_valid>:
ffffc0207e28: d0000081 	adrp	x1, 0xffffc0219000 <resume_cpu+0x4>
ffffc0207e2c: f9400021 	ldr	x1, [x1]
ffffc0207e30: 91080022 	add	x2, x1, #512
ffffc0207e34: b853a042 	ldur	w2, [x2, #-198]
ffffc0207e38: 531d7042 	lsl	w2, w2, #3
ffffc0207e3c: eb00005f 	cmp	x2, x0
ffffc0207e40: 54000129 	b.ls	0xffffc0207e64 <cpu_id_valid+0x3c>
ffffc0207e44: 53067c02 	lsr	w2, w0, #6
ffffc0207e48: 91062821 	add	x1, x1, #394
ffffc0207e4c: 12001400 	and	w0, w0, #0x3f
ffffc0207e50: d37df042 	lsl	x2, x2, #3
ffffc0207e54: f8626821 	ldr	x1, [x1, x2]
ffffc0207e58: 9ac02420 	lsr	x0, x1, x0
ffffc0207e5c: 12000000 	and	w0, w0, #0x1
ffffc0207e60: d65f03c0 	ret
ffffc0207e64: 52800000 	mov	w0, #0
ffffc0207e68: 17fffffe 	b	0xffffc0207e60 <cpu_id_valid+0x38>

0000ffffc0207e6c <resume_cpu>:
ffffc0207e6c: d3727c00 	ubfiz	x0, x0, #14, #32
ffffc0207e70: d00000a4 	adrp	x4, 0xffffc021d000 <cell_resume.constprop.11+0x28>
ffffc0207e74: 91000084 	add	x4, x4, #0
ffffc0207e78: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0207e7c: 8b040004 	add	x4, x0, x4
ffffc0207e80: d2868d80 	mov	x0, #13420
ffffc0207e84: 8b000085 	add	x5, x4, x0
ffffc0207e88: 910003fd 	mov	x29, sp
ffffc0207e8c: aa0503e0 	mov	x0, x5
ffffc0207e90: 97fffed2 	bl	0xffffc02079d8 <spin_lock>
ffffc0207e94: b930389f 	str	wzr, [x4, #12344]
ffffc0207e98: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0207e9c: 17fffedc 	b	0xffffc0207a0c <spin_unlock>

0000ffffc0207ea0 <cell_resume.constprop.11>:
ffffc0207ea0: d2840008 	mov	x8, #8192
ffffc0207ea4: f0000086 	adrp	x6, 0xffffc021a000 <cell_resume.constprop.11+0x50>
ffffc0207ea8: f2a00028 	movk	x8, #1, lsl #16
ffffc0207eac: 910000c6 	add	x6, x6, #0
ffffc0207eb0: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0207eb4: 12800007 	mov	w7, #-1
ffffc0207eb8: f2dfe008 	movk	x8, #65280, lsl #32
ffffc0207ebc: 910003fd 	mov	x29, sp
ffffc0207ec0: f94854c1 	ldr	x1, [x6, #4264]
ffffc0207ec4: 2a0703e0 	mov	w0, w7
ffffc0207ec8: b9500102 	ldr	w2, [x8, #4096]
ffffc0207ecc: 97ffff5f 	bl	0xffffc0207c48 <next_cpu>
ffffc0207ed0: f94854c1 	ldr	x1, [x6, #4264]
ffffc0207ed4: 2a0003e7 	mov	w7, w0
ffffc0207ed8: f9400021 	ldr	x1, [x1]
ffffc0207edc: eb20403f 	cmp	x1, w0, uxtw
ffffc0207ee0: 54000062 	b.hs	0xffffc0207eec <cell_resume.constprop.11+0x4c>
ffffc0207ee4: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0207ee8: d65f03c0 	ret
ffffc0207eec: 97ffffe0 	bl	0xffffc0207e6c <resume_cpu>
ffffc0207ef0: 17fffff4 	b	0xffffc0207ec0 <cell_resume.constprop.11+0x20>

0000ffffc0207ef4 <cell_management_prologue.part.2>:
ffffc0207ef4: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0207ef8: 2a0003e3 	mov	w3, w0
ffffc0207efc: 910003fd 	mov	x29, sp
ffffc0207f00: f9000bf3 	str	x19, [sp, #16]
ffffc0207f04: aa0203f3 	mov	x19, x2
ffffc0207f08: f9400260 	ldr	x0, [x19]
ffffc0207f0c: b5000080 	cbnz	x0, 0xffffc0207f1c <cell_management_prologue.part.2+0x28>
ffffc0207f10: 97ffffe4 	bl	0xffffc0207ea0 <cell_resume.constprop.11>
ffffc0207f14: 12800020 	mov	w0, #-2
ffffc0207f18: 14000022 	b	0xffffc0207fa0 <cell_management_prologue.part.2+0xac>
ffffc0207f1c: f9485002 	ldr	x2, [x0, #4256]
ffffc0207f20: b9402842 	ldr	w2, [x2, #40]
ffffc0207f24: eb01005f 	cmp	x2, x1
ffffc0207f28: 54000300 	b.eq	0xffffc0207f88 <cell_management_prologue.part.2+0x94>
ffffc0207f2c: f9486400 	ldr	x0, [x0, #4296]
ffffc0207f30: f9000260 	str	x0, [x19]
ffffc0207f34: 17fffff5 	b	0xffffc0207f08 <cell_management_prologue.part.2+0x14>
ffffc0207f38: 7100087f 	cmp	w3, #2
ffffc0207f3c: 54000061 	b.ne	0xffffc0207f48 <cell_management_prologue.part.2+0x54>
ffffc0207f40: f9486421 	ldr	x1, [x1, #4296]
ffffc0207f44: b5000121 	cbnz	x1, 0xffffc0207f68 <cell_management_prologue.part.2+0x74>
ffffc0207f48: 52800002 	mov	w2, #0
ffffc0207f4c: 52800021 	mov	w1, #1
ffffc0207f50: 97ffff78 	bl	0xffffc0207d30 <cell_exchange_message>
ffffc0207f54: 34000140 	cbz	w0, 0xffffc0207f7c <cell_management_prologue.part.2+0x88>
ffffc0207f58: f9400260 	ldr	x0, [x19]
ffffc0207f5c: 97ffff48 	bl	0xffffc0207c7c <cell_suspend>
ffffc0207f60: 52800000 	mov	w0, #0
ffffc0207f64: 1400000f 	b	0xffffc0207fa0 <cell_management_prologue.part.2+0xac>
ffffc0207f68: eb01001f 	cmp	x0, x1
ffffc0207f6c: 54fffea0 	b.eq	0xffffc0207f40 <cell_management_prologue.part.2+0x4c>
ffffc0207f70: b9400822 	ldr	w2, [x1, #8]
ffffc0207f74: 7100045f 	cmp	w2, #1
ffffc0207f78: 54fffe41 	b.ne	0xffffc0207f40 <cell_management_prologue.part.2+0x4c>
ffffc0207f7c: 97ffffc9 	bl	0xffffc0207ea0 <cell_resume.constprop.11>
ffffc0207f80: 12800000 	mov	w0, #-1
ffffc0207f84: 14000007 	b	0xffffc0207fa0 <cell_management_prologue.part.2+0xac>
ffffc0207f88: f0000081 	adrp	x1, 0xffffc021a000 <cell_init+0x28>
ffffc0207f8c: 91000021 	add	x1, x1, #0
ffffc0207f90: eb01001f 	cmp	x0, x1
ffffc0207f94: 54fffd21 	b.ne	0xffffc0207f38 <cell_management_prologue.part.2+0x44>
ffffc0207f98: 97ffffc2 	bl	0xffffc0207ea0 <cell_resume.constprop.11>
ffffc0207f9c: 128002a0 	mov	w0, #-22
ffffc0207fa0: f9400bf3 	ldr	x19, [sp, #16]
ffffc0207fa4: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0207fa8: d65f03c0 	ret

0000ffffc0207fac <cell_init>:
ffffc0207fac: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0207fb0: 910003fd 	mov	x29, sp
ffffc0207fb4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0207fb8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0207fbc: f9001bf7 	str	x23, [sp, #48]
ffffc0207fc0: f9485001 	ldr	x1, [x0, #4256]
ffffc0207fc4: b9403035 	ldr	w21, [x1, #48]
ffffc0207fc8: 2a1503f6 	mov	w22, w21
ffffc0207fcc: f14006df 	cmp	x22, #1, lsl #12        // =4096
ffffc0207fd0: 54000548 	b.hi	0xffffc0208078 <cell_init+0xcc>
ffffc0207fd4: 91020037 	add	x23, x1, #128
ffffc0207fd8: aa0003f3 	mov	x19, x0
ffffc0207fdc: f10022df 	cmp	x22, #8
ffffc0207fe0: 54000129 	b.ls	0xffffc0208004 <cell_init+0x58>
ffffc0207fe4: 52800021 	mov	w1, #1
ffffc0207fe8: b0000080 	adrp	x0, 0xffffc0218000 <cell_init+0x80>
ffffc0207fec: 91004000 	add	x0, x0, #16
ffffc0207ff0: 97fffba2 	bl	0xffffc0206e78 <page_alloc>
ffffc0207ff4: aa0003f4 	mov	x20, x0
ffffc0207ff8: b50000a0 	cbnz	x0, 0xffffc020800c <cell_init+0x60>
ffffc0207ffc: 12800174 	mov	w20, #-12
ffffc0208000: 14000018 	b	0xffffc0208060 <cell_init+0xb4>
ffffc0208004: d2821601 	mov	x1, #4272
ffffc0208008: 8b010014 	add	x20, x0, x1
ffffc020800c: 92800000 	mov	x0, #-1
ffffc0208010: aa1603e2 	mov	x2, x22
ffffc0208014: 8b354c15 	add	x21, x0, w21, uxtw #3
ffffc0208018: aa1403e0 	mov	x0, x20
ffffc020801c: aa1703e1 	mov	x1, x23
ffffc0208020: f8008415 	str	x21, [x0], #8
ffffc0208024: 94000397 	bl	0xffffc0208e80 <memcpy>
ffffc0208028: f9085674 	str	x20, [x19, #4264]
ffffc020802c: aa1303e0 	mov	x0, x19
ffffc0208030: 940003d2 	bl	0xffffc0208f78 <mmio_cell_init>
ffffc0208034: 2a0003f4 	mov	w20, w0
ffffc0208038: 34000140 	cbz	w0, 0xffffc0208060 <cell_init+0xb4>
ffffc020803c: f9485661 	ldr	x1, [x19, #4264]
ffffc0208040: d2821600 	mov	x0, #4272
ffffc0208044: 8b000273 	add	x19, x19, x0
ffffc0208048: eb13003f 	cmp	x1, x19
ffffc020804c: 540000a0 	b.eq	0xffffc0208060 <cell_init+0xb4>
ffffc0208050: 90000080 	adrp	x0, 0xffffc0218000 <config_commit+0x10>
ffffc0208054: 52800022 	mov	w2, #1
ffffc0208058: 91004000 	add	x0, x0, #16
ffffc020805c: 97fffb8b 	bl	0xffffc0206e88 <page_free>
ffffc0208060: 2a1403e0 	mov	w0, w20
ffffc0208064: f9401bf7 	ldr	x23, [sp, #48]
ffffc0208068: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020806c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0208070: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0208074: d65f03c0 	ret
ffffc0208078: 128002b4 	mov	w20, #-22
ffffc020807c: 17fffff9 	b	0xffffc0208060 <cell_init+0xb4>

0000ffffc0208080 <config_commit>:
ffffc0208080: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0208084: 910003fd 	mov	x29, sp
ffffc0208088: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020808c: d0000094 	adrp	x20, 0xffffc021a000 <cell_destroy_internal+0x8>
ffffc0208090: 91000294 	add	x20, x20, #0
ffffc0208094: aa0003f3 	mov	x19, x0
ffffc0208098: aa1403e0 	mov	x0, x20
ffffc020809c: 97ffe2c7 	bl	0xffffc0200bb8 <arch_flush_cell_vcpu_caches>
ffffc02080a0: b40000b3 	cbz	x19, 0xffffc02080b4 <config_commit+0x34>
ffffc02080a4: eb14027f 	cmp	x19, x20
ffffc02080a8: 54000060 	b.eq	0xffffc02080b4 <config_commit+0x34>
ffffc02080ac: aa1303e0 	mov	x0, x19
ffffc02080b0: 97ffe2c2 	bl	0xffffc0200bb8 <arch_flush_cell_vcpu_caches>
ffffc02080b4: aa1303e0 	mov	x0, x19
ffffc02080b8: 97ffe2e0 	bl	0xffffc0200c38 <arch_config_commit>
ffffc02080bc: aa1303e0 	mov	x0, x19
ffffc02080c0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02080c4: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc02080c8: 14000983 	b	0xffffc020a6d4 <pci_config_commit>

0000ffffc02080cc <cell_destroy_internal>:
ffffc02080cc: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc02080d0: 910003fd 	mov	x29, sp
ffffc02080d4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02080d8: aa0003f4 	mov	x20, x0
ffffc02080dc: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02080e0: d0000096 	adrp	x22, 0xffffc021a000 <cell_destroy_internal+0x5c>
ffffc02080e4: a90363f7 	stp	x23, x24, [sp, #48]
ffffc02080e8: b00000b5 	adrp	x21, 0xffffc021d000 <cell_destroy_internal+0x70>
ffffc02080ec: 910002d7 	add	x23, x22, #0
ffffc02080f0: 910002b5 	add	x21, x21, #0
ffffc02080f4: 12800018 	mov	w24, #-1
ffffc02080f8: f9485681 	ldr	x1, [x20, #4264]
ffffc02080fc: 2a1803e0 	mov	w0, w24
ffffc0208100: 12800002 	mov	w2, #-1
ffffc0208104: 97fffed1 	bl	0xffffc0207c48 <next_cpu>
ffffc0208108: 2a0003f3 	mov	w19, w0
ffffc020810c: f9485681 	ldr	x1, [x20, #4264]
ffffc0208110: aa1303f8 	mov	x24, x19
ffffc0208114: f9400021 	ldr	x1, [x1]
ffffc0208118: eb01027f 	cmp	x19, x1
ffffc020811c: 54000369 	b.ls	0xffffc0208188 <cell_destroy_internal+0xbc>
ffffc0208120: f9485293 	ldr	x19, [x20, #4256]
ffffc0208124: 52800015 	mov	w21, #0
ffffc0208128: d2801416 	mov	x22, #160
ffffc020812c: b9403260 	ldr	w0, [x19, #48]
ffffc0208130: 91020000 	add	x0, x0, #128
ffffc0208134: 8b000273 	add	x19, x19, x0
ffffc0208138: f9485280 	ldr	x0, [x20, #4256]
ffffc020813c: b9403400 	ldr	w0, [x0, #52]
ffffc0208140: 6b15001f 	cmp	w0, w21
ffffc0208144: 540004e8 	b.hi	0xffffc02081e0 <cell_destroy_internal+0x114>
ffffc0208148: d0000033 	adrp	x19, 0xffffc020e000 <cell_destroy_internal+0x94>
ffffc020814c: d0000035 	adrp	x21, 0xffffc020e000 <cell_destroy_internal+0x98>
ffffc0208150: 91080273 	add	x19, x19, #512
ffffc0208154: 910742b5 	add	x21, x21, #464
ffffc0208158: eb15027f 	cmp	x19, x21
ffffc020815c: 54000662 	b.hs	0xffffc0208228 <cell_destroy_internal+0x15c>
ffffc0208160: aa1403e0 	mov	x0, x20
ffffc0208164: 97ffe27a 	bl	0xffffc0200b4c <arch_cell_destroy>
ffffc0208168: aa1403e0 	mov	x0, x20
ffffc020816c: 97ffffc5 	bl	0xffffc0208080 <config_commit>
ffffc0208170: aa1403e0 	mov	x0, x20
ffffc0208174: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0208178: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020817c: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0208180: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0208184: 17fffe4a 	b	0xffffc0207aac <cell_exit>
ffffc0208188: 97ffe1d9 	bl	0xffffc02008ec <arch_park_cpu>
ffffc020818c: 910002c1 	add	x1, x22, #0
ffffc0208190: 53067f00 	lsr	w0, w24, #6
ffffc0208194: 12001702 	and	w2, w24, #0x3f
ffffc0208198: f9485421 	ldr	x1, [x1, #4264]
ffffc020819c: 91002021 	add	x1, x1, #8
ffffc02081a0: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc02081a4: 8b204c20 	add	x0, x1, w0, uxtw #3
ffffc02081a8: d2800021 	mov	x1, #1
ffffc02081ac: 9ac22021 	lsl	x1, x1, x2
ffffc02081b0: c85f7c03 	ldxr	x3, [x0]
ffffc02081b4: aa010063 	orr	x3, x3, x1
ffffc02081b8: c8027c03 	stxr	w2, x3, [x0]
ffffc02081bc: 35ffffa2 	cbnz	w2, 0xffffc02081b0 <cell_destroy_internal+0xe4>
ffffc02081c0: 8b133aa0 	add	x0, x21, x19, lsl #14
ffffc02081c4: d2860201 	mov	x1, #12304
ffffc02081c8: f9180417 	str	x23, [x0, #12296]
ffffc02081cc: b930341f 	str	wzr, [x0, #12340]
ffffc02081d0: 8b010000 	add	x0, x0, x1
ffffc02081d4: a9007c1f 	stp	xzr, xzr, [x0]
ffffc02081d8: a9017c1f 	stp	xzr, xzr, [x0, #16]
ffffc02081dc: 17ffffc7 	b	0xffffc02080f8 <cell_destroy_internal+0x2c>
ffffc02081e0: f9400660 	ldr	x0, [x19, #8]
ffffc02081e4: f2402c1f 	tst	x0, #0xfff
ffffc02081e8: 540000e1 	b.ne	0xffffc0208204 <cell_destroy_internal+0x138>
ffffc02081ec: f9400a60 	ldr	x0, [x19, #16]
ffffc02081f0: f2402c1f 	tst	x0, #0xfff
ffffc02081f4: 54000081 	b.ne	0xffffc0208204 <cell_destroy_internal+0x138>
ffffc02081f8: aa1303e1 	mov	x1, x19
ffffc02081fc: aa1403e0 	mov	x0, x20
ffffc0208200: 97ffeb33 	bl	0xffffc0202ecc <arch_unmap_memory_region>
ffffc0208204: f9400e60 	ldr	x0, [x19, #24]
ffffc0208208: ea16001f 	tst	x0, x22
ffffc020820c: 54000081 	b.ne	0xffffc020821c <cell_destroy_internal+0x150>
ffffc0208210: 52800021 	mov	w1, #1
ffffc0208214: aa1303e0 	mov	x0, x19
ffffc0208218: 97fffe38 	bl	0xffffc0207af8 <remap_to_root_cell>
ffffc020821c: 91008273 	add	x19, x19, #32
ffffc0208220: 110006b5 	add	w21, w21, #1
ffffc0208224: 17ffffc5 	b	0xffffc0208138 <cell_destroy_internal+0x6c>
ffffc0208228: f9401661 	ldr	x1, [x19, #40]
ffffc020822c: aa1403e0 	mov	x0, x20
ffffc0208230: d100c273 	sub	x19, x19, #48
ffffc0208234: d63f0020 	blr	x1
ffffc0208238: 17ffffc8 	b	0xffffc0208158 <cell_destroy_internal+0x8c>

0000ffffc020823c <shutdown>:
ffffc020823c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0208240: 910003fd 	mov	x29, sp
ffffc0208244: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0208248: d0000033 	adrp	x19, 0xffffc020e000 <shutdown+0x24>
ffffc020824c: d0000034 	adrp	x20, 0xffffc020e000 <shutdown+0x28>
ffffc0208250: 91080273 	add	x19, x19, #512
ffffc0208254: 91074294 	add	x20, x20, #464
ffffc0208258: 940006ee 	bl	0xffffc0209e10 <pci_prepare_handover>
ffffc020825c: 97ffe27a 	bl	0xffffc0200c44 <arch_prepare_shutdown>
ffffc0208260: eb14027f 	cmp	x19, x20
ffffc0208264: 54000082 	b.hs	0xffffc0208274 <shutdown+0x38>
ffffc0208268: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020826c: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0208270: d65f03c0 	ret
ffffc0208274: f9400a60 	ldr	x0, [x19, #16]
ffffc0208278: d100c273 	sub	x19, x19, #48
ffffc020827c: d63f0000 	blr	x0
ffffc0208280: 17fffff8 	b	0xffffc0208260 <shutdown+0x24>

0000ffffc0208284 <hypercall>:
ffffc0208284: d2a00024 	mov	x4, #65536
ffffc0208288: f100201f 	cmp	x0, #8
ffffc020828c: f2dfe004 	movk	x4, #65280, lsl #32
ffffc0208290: b9701c83 	ldr	w3, [x4, #12316]
ffffc0208294: 11000463 	add	w3, w3, #1
ffffc0208298: b9301c83 	str	w3, [x4, #12316]
ffffc020829c: 54005308 	b.hi	0xffffc0208cfc <hypercall+0xa78>
ffffc02082a0: 7100201f 	cmp	w0, #8
ffffc02082a4: 540052c8 	b.hi	0xffffc0208cfc <hypercall+0xa78>
ffffc02082a8: a9b97bfd 	stp	x29, x30, [sp, #-112]!
ffffc02082ac: aa0203e3 	mov	x3, x2
ffffc02082b0: 910003fd 	mov	x29, sp
ffffc02082b4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02082b8: aa0103f3 	mov	x19, x1
ffffc02082bc: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02082c0: f0000001 	adrp	x1, 0xffffc020b000 <hypercall+0x48>
ffffc02082c4: a90363f7 	stp	x23, x24, [sp, #48]
ffffc02082c8: 911bb021 	add	x1, x1, #1772
ffffc02082cc: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc02082d0: a90573fb 	stp	x27, x28, [sp, #80]
ffffc02082d4: 78605820 	ldrh	w0, [x1, w0, uxtw #1]
ffffc02082d8: 10000061 	adr	x1, #12
ffffc02082dc: 8b20a820 	add	x0, x1, w0, sxth #2
ffffc02082e0: d61f0000 	br	x0
ffffc02082e4: f9580486 	ldr	x6, [x4, #12296]
ffffc02082e8: d0000080 	adrp	x0, 0xffffc021a000 <hypercall+0xac>
ffffc02082ec: 91000000 	add	x0, x0, #0
ffffc02082f0: b9700095 	ldr	w21, [x4, #12288]
ffffc02082f4: eb0000df 	cmp	x6, x0
ffffc02082f8: 54000961 	b.ne	0xffffc0208424 <hypercall+0x1a0>
ffffc02082fc: b0000094 	adrp	x20, 0xffffc0219000 <hypercall+0xbc>
ffffc0208300: 91000280 	add	x0, x20, #0
ffffc0208304: 91002000 	add	x0, x0, #8
ffffc0208308: 97fffdb4 	bl	0xffffc02079d8 <spin_lock>
ffffc020830c: b9703080 	ldr	w0, [x4, #12336]
ffffc0208310: 340005e0 	cbz	w0, 0xffffc02083cc <hypercall+0x148>
ffffc0208314: d2a00020 	mov	x0, #65536
ffffc0208318: f2dfe000 	movk	x0, #65280, lsl #32
ffffc020831c: b9703002 	ldr	w2, [x0, #12336]
ffffc0208320: 7100045f 	cmp	w2, #1
ffffc0208324: 540007a1 	b.ne	0xffffc0208418 <hypercall+0x194>
ffffc0208328: 91000280 	add	x0, x20, #0
ffffc020832c: b9401001 	ldr	w1, [x0, #16]
ffffc0208330: b9000c02 	str	w2, [x0, #12]
ffffc0208334: 52800002 	mov	w2, #0
ffffc0208338: 11000421 	add	w1, w1, #1
ffffc020833c: b9001001 	str	w1, [x0, #16]
ffffc0208340: 91000293 	add	x19, x20, #0
ffffc0208344: 91002260 	add	x0, x19, #8
ffffc0208348: 97fffdb1 	bl	0xffffc0207a0c <spin_unlock>
ffffc020834c: 37f80302 	tbnz	w2, #31, 0xffffc02083ac <hypercall+0x128>
ffffc0208350: 90ffffc0 	adrp	x0, 0xffffc0200000 <hypercall+0xac>
ffffc0208354: 91000000 	add	x0, x0, #0
ffffc0208358: b9401261 	ldr	w1, [x19, #16]
ffffc020835c: b9403402 	ldr	w2, [x0, #52]
ffffc0208360: 6b01005f 	cmp	w2, w1
ffffc0208364: 540005e8 	b.hi	0xffffc0208420 <hypercall+0x19c>
ffffc0208368: 91002260 	add	x0, x19, #8
ffffc020836c: 97fffd9b 	bl	0xffffc02079d8 <spin_lock>
ffffc0208370: b9400e60 	ldr	w0, [x19, #12]
ffffc0208374: 340000c0 	cbz	w0, 0xffffc020838c <hypercall+0x108>
ffffc0208378: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x100>
ffffc020837c: 913c8c00 	add	x0, x0, #3875
ffffc0208380: 97fff9e9 	bl	0xffffc0206b24 <printk>
ffffc0208384: 97ffffae 	bl	0xffffc020823c <shutdown>
ffffc0208388: b9000e7f 	str	wzr, [x19, #12]
ffffc020838c: 2a1503e1 	mov	w1, w21
ffffc0208390: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x118>
ffffc0208394: 913cf400 	add	x0, x0, #3901
ffffc0208398: 97fff9e3 	bl	0xffffc0206b24 <printk>
ffffc020839c: 91000280 	add	x0, x20, #0
ffffc02083a0: 91002000 	add	x0, x0, #8
ffffc02083a4: 97fffd9a 	bl	0xffffc0207a0c <spin_unlock>
ffffc02083a8: 52800002 	mov	w2, #0
ffffc02083ac: 93407c40 	sxtw	x0, w2
ffffc02083b0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02083b4: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc02083b8: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc02083bc: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc02083c0: a94573fb 	ldp	x27, x28, [sp, #80]
ffffc02083c4: a8c77bfd 	ldp	x29, x30, [sp], #112
ffffc02083c8: d65f03c0 	ret
ffffc02083cc: d0000020 	adrp	x0, 0xffffc020e000 <hypercall+0x160>
ffffc02083d0: 128001e7 	mov	w7, #-16
ffffc02083d4: b00000a8 	adrp	x8, 0xffffc021d000 <hypercall+0x1a4>
ffffc02083d8: 91000108 	add	x8, x8, #0
ffffc02083dc: b9418800 	ldr	w0, [x0, #392]
ffffc02083e0: 7100041f 	cmp	w0, #1
ffffc02083e4: 12800000 	mov	w0, #-1
ffffc02083e8: 1a9f14e7 	csinc	w7, w7, wzr, ne
ffffc02083ec: f94854c1 	ldr	x1, [x6, #4264]
ffffc02083f0: 12800002 	mov	w2, #-1
ffffc02083f4: 97fffe15 	bl	0xffffc0207c48 <next_cpu>
ffffc02083f8: f94854c2 	ldr	x2, [x6, #4264]
ffffc02083fc: 2a0003e1 	mov	w1, w0
ffffc0208400: f9400042 	ldr	x2, [x2]
ffffc0208404: eb02003f 	cmp	x1, x2
ffffc0208408: 54fff868 	b.hi	0xffffc0208314 <hypercall+0x90>
ffffc020840c: 8b013901 	add	x1, x8, x1, lsl #14
ffffc0208410: b9303027 	str	w7, [x1, #12336]
ffffc0208414: 17fffff6 	b	0xffffc02083ec <hypercall+0x168>
ffffc0208418: b930301f 	str	wzr, [x0, #12336]
ffffc020841c: 17ffffc9 	b	0xffffc0208340 <hypercall+0xbc>
ffffc0208420: 17ffffce 	b	0xffffc0208358 <hypercall+0xd4>
ffffc0208424: 12800002 	mov	w2, #-1
ffffc0208428: 17ffffe1 	b	0xffffc02083ac <hypercall+0x128>
ffffc020842c: f9580494 	ldr	x20, [x4, #12296]
ffffc0208430: d0000096 	adrp	x22, 0xffffc021a000 <hypercall+0x1f4>
ffffc0208434: 910002c0 	add	x0, x22, #0
ffffc0208438: eb00029f 	cmp	x20, x0
ffffc020843c: 54003a61 	b.ne	0xffffc0208b88 <hypercall+0x904>
ffffc0208440: aa1403e0 	mov	x0, x20
ffffc0208444: 97fffe0e 	bl	0xffffc0207c7c <cell_suspend>
ffffc0208448: f9486680 	ldr	x0, [x20, #4296]
ffffc020844c: b50006e0 	cbnz	x0, 0xffffc0208528 <hypercall+0x2a4>
ffffc0208450: 92402e74 	and	x20, x19, #0xfff
ffffc0208454: d2820fe0 	mov	x0, #4223
ffffc0208458: 8b000282 	add	x2, x20, x0
ffffc020845c: d280f823 	mov	x3, #1985
ffffc0208460: aa1303e1 	mov	x1, x19
ffffc0208464: d2800000 	mov	x0, #0
ffffc0208468: d34cac42 	ubfx	x2, x2, #12, #32
ffffc020846c: 97fffc4d 	bl	0xffffc02075a0 <paging_get_guest_pages>
ffffc0208470: b4000540 	cbz	x0, 0xffffc0208518 <hypercall+0x294>
ffffc0208474: 8b140015 	add	x21, x0, x20
ffffc0208478: 910002d7 	add	x23, x22, #0
ffffc020847c: 910022b9 	add	x25, x21, #8
ffffc0208480: f94852f8 	ldr	x24, [x23, #4256]
ffffc0208484: aa1903e1 	mov	x1, x25
ffffc0208488: 91002300 	add	x0, x24, #8
ffffc020848c: 94000272 	bl	0xffffc0208e54 <strcmp>
ffffc0208490: 34001c40 	cbz	w0, 0xffffc0208818 <hypercall+0x594>
ffffc0208494: b9402aa0 	ldr	w0, [x21, #40]
ffffc0208498: b9402b01 	ldr	w1, [x24, #40]
ffffc020849c: 6b00003f 	cmp	w1, w0
ffffc02084a0: 54001bc0 	b.eq	0xffffc0208818 <hypercall+0x594>
ffffc02084a4: f94866f7 	ldr	x23, [x23, #4296]
ffffc02084a8: b5fffed7 	cbnz	x23, 0xffffc0208480 <hypercall+0x1fc>
ffffc02084ac: 294866a2 	ldp	w2, w25, [x21, #64]
ffffc02084b0: 528000e1 	mov	w1, #7
ffffc02084b4: b9404aa0 	ldr	w0, [x21, #72]
ffffc02084b8: 1b010339 	madd	w25, w25, w1, w0
ffffc02084bc: b9403ea1 	ldr	w1, [x21, #60]
ffffc02084c0: b94036a0 	ldr	w0, [x21, #52]
ffffc02084c4: 0b010000 	add	w0, w0, w1
ffffc02084c8: 52800181 	mov	w1, #12
ffffc02084cc: 531b6800 	lsl	w0, w0, #5
ffffc02084d0: 0b190c19 	add	w25, w0, w25, lsl #3
ffffc02084d4: b9403aa0 	ldr	w0, [x21, #56]
ffffc02084d8: 1b017c00 	mul	w0, w0, w1
ffffc02084dc: b94032a1 	ldr	w1, [x21, #48]
ffffc02084e0: 11020000 	add	w0, w0, #128
ffffc02084e4: 0b020021 	add	w1, w1, w2
ffffc02084e8: 0b010000 	add	w0, w0, w1
ffffc02084ec: 913ffe82 	add	x2, x20, #4095
ffffc02084f0: 0b000339 	add	w25, w25, w0
ffffc02084f4: 8b190042 	add	x2, x2, x25
ffffc02084f8: d34cfc42 	lsr	x2, x2, #12
ffffc02084fc: f100405f 	cmp	x2, #16
ffffc0208500: 54001908 	b.hi	0xffffc0208820 <hypercall+0x59c>
ffffc0208504: d280f823 	mov	x3, #1985
ffffc0208508: aa1303e1 	mov	x1, x19
ffffc020850c: d2800000 	mov	x0, #0
ffffc0208510: 97fffc24 	bl	0xffffc02075a0 <paging_get_guest_pages>
ffffc0208514: b5000140 	cbnz	x0, 0xffffc020853c <hypercall+0x2b8>
ffffc0208518: 12800173 	mov	w19, #-12
ffffc020851c: 97fffe61 	bl	0xffffc0207ea0 <cell_resume.constprop.11>
ffffc0208520: 93407e60 	sxtw	x0, w19
ffffc0208524: 17ffffa3 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208528: b9400801 	ldr	w1, [x0, #8]
ffffc020852c: 7100043f 	cmp	w1, #1
ffffc0208530: 54001700 	b.eq	0xffffc0208810 <hypercall+0x58c>
ffffc0208534: f9486400 	ldr	x0, [x0, #4296]
ffffc0208538: 17ffffc5 	b	0xffffc020844c <hypercall+0x1c8>
ffffc020853c: d285ffe2 	mov	x2, #12287
ffffc0208540: 8b020337 	add	x23, x25, x2
ffffc0208544: 90000098 	adrp	x24, 0xffffc0218000 <hypercall+0x300>
ffffc0208548: 91004300 	add	x0, x24, #16
ffffc020854c: d34cfef7 	lsr	x23, x23, #12
ffffc0208550: 2a1703e1 	mov	w1, w23
ffffc0208554: 97fffa49 	bl	0xffffc0206e78 <page_alloc>
ffffc0208558: aa0003f4 	mov	x20, x0
ffffc020855c: b4fffde0 	cbz	x0, 0xffffc0208518 <hypercall+0x294>
ffffc0208560: 91400800 	add	x0, x0, #2, lsl #12     // =8192
ffffc0208564: aa1903e2 	mov	x2, x25
ffffc0208568: aa1503e1 	mov	x1, x21
ffffc020856c: b9109a97 	str	w23, [x20, #4248]
ffffc0208570: f9085280 	str	x0, [x20, #4256]
ffffc0208574: 94000243 	bl	0xffffc0208e80 <memcpy>
ffffc0208578: aa1403e0 	mov	x0, x20
ffffc020857c: 97fffe8c 	bl	0xffffc0207fac <cell_init>
ffffc0208580: 2a0003f3 	mov	w19, w0
ffffc0208584: 350012a0 	cbnz	w0, 0xffffc02087d8 <hypercall+0x554>
ffffc0208588: d2a00020 	mov	x0, #65536
ffffc020858c: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0208590: b9700001 	ldr	w1, [x0, #12288]
ffffc0208594: aa1403e0 	mov	x0, x20
ffffc0208598: 97fffd21 	bl	0xffffc0207a1c <cell_owns_cpu>
ffffc020859c: 34000980 	cbz	w0, 0xffffc02086cc <hypercall+0x448>
ffffc02085a0: 128001f3 	mov	w19, #-16
ffffc02085a4: aa1403e0 	mov	x0, x20
ffffc02085a8: 97fffd41 	bl	0xffffc0207aac <cell_exit>
ffffc02085ac: 1400008b 	b	0xffffc02087d8 <hypercall+0x554>
ffffc02085b0: 2a0303e1 	mov	w1, w3
ffffc02085b4: aa0603e0 	mov	x0, x6
ffffc02085b8: 97fffd19 	bl	0xffffc0207a1c <cell_owns_cpu>
ffffc02085bc: 34ffff20 	cbz	w0, 0xffffc02085a0 <hypercall+0x31c>
ffffc02085c0: f9485681 	ldr	x1, [x20, #4264]
ffffc02085c4: 2a0303e0 	mov	w0, w3
ffffc02085c8: 12800002 	mov	w2, #-1
ffffc02085cc: 97fffd9f 	bl	0xffffc0207c48 <next_cpu>
ffffc02085d0: 2a0003e3 	mov	w3, w0
ffffc02085d4: f9485680 	ldr	x0, [x20, #4264]
ffffc02085d8: f9400000 	ldr	x0, [x0]
ffffc02085dc: eb23401f 	cmp	x0, w3, uxtw
ffffc02085e0: 54fffe82 	b.hs	0xffffc02085b0 <hypercall+0x32c>
ffffc02085e4: aa1403e0 	mov	x0, x20
ffffc02085e8: 97ffe127 	bl	0xffffc0200a84 <arch_cell_create>
ffffc02085ec: 2a0003f3 	mov	w19, w0
ffffc02085f0: 35fffda0 	cbnz	w0, 0xffffc02085a4 <hypercall+0x320>
ffffc02085f4: d0000020 	adrp	x0, 0xffffc020e000 <hypercall+0x388>
ffffc02085f8: d000003b 	adrp	x27, 0xffffc020e000 <hypercall+0x38c>
ffffc02085fc: 91074015 	add	x21, x0, #464
ffffc0208600: aa0003f9 	mov	x25, x0
ffffc0208604: 9108c37b 	add	x27, x27, #560
ffffc0208608: eb1b02bf 	cmp	x21, x27
ffffc020860c: 54000663 	b.lo	0xffffc02086d8 <hypercall+0x454>
ffffc0208610: b00000b9 	adrp	x25, 0xffffc021d000 <hypercall+0x3e0>
ffffc0208614: 910002db 	add	x27, x22, #0
ffffc0208618: 91000339 	add	x25, x25, #0
ffffc020861c: 1280001c 	mov	w28, #-1
ffffc0208620: 5280081a 	mov	w26, #64
ffffc0208624: f9485681 	ldr	x1, [x20, #4264]
ffffc0208628: 2a1c03e0 	mov	w0, w28
ffffc020862c: 12800002 	mov	w2, #-1
ffffc0208630: 97fffd86 	bl	0xffffc0207c48 <next_cpu>
ffffc0208634: 2a0003f5 	mov	w21, w0
ffffc0208638: f9485681 	ldr	x1, [x20, #4264]
ffffc020863c: aa1503fc 	mov	x28, x21
ffffc0208640: f9400021 	ldr	x1, [x1]
ffffc0208644: eb0102bf 	cmp	x21, x1
ffffc0208648: 54000709 	b.ls	0xffffc0208728 <hypercall+0x4a4>
ffffc020864c: f9485295 	ldr	x21, [x20, #4256]
ffffc0208650: 5280001a 	mov	w26, #0
ffffc0208654: d280141b 	mov	x27, #160
ffffc0208658: b94032a0 	ldr	w0, [x21, #48]
ffffc020865c: 91020000 	add	x0, x0, #128
ffffc0208660: 8b0002b5 	add	x21, x21, x0
ffffc0208664: f9485280 	ldr	x0, [x20, #4256]
ffffc0208668: b9403400 	ldr	w0, [x0, #52]
ffffc020866c: 6b00035f 	cmp	w26, w0
ffffc0208670: 540008a3 	b.lo	0xffffc0208784 <hypercall+0x500>
ffffc0208674: aa1403e0 	mov	x0, x20
ffffc0208678: 910002d6 	add	x22, x22, #0
ffffc020867c: 97fffe81 	bl	0xffffc0208080 <config_commit>
ffffc0208680: 52800040 	mov	w0, #2
ffffc0208684: b9000a80 	str	w0, [x20, #8]
ffffc0208688: f94866c0 	ldr	x0, [x22, #4296]
ffffc020868c: b5000be0 	cbnz	x0, 0xffffc0208808 <hypercall+0x584>
ffffc0208690: d0000021 	adrp	x1, 0xffffc020e000 <hypercall+0x424>
ffffc0208694: f90866d4 	str	x20, [x22, #4296]
ffffc0208698: b9418820 	ldr	w0, [x1, #392]
ffffc020869c: 11000400 	add	w0, w0, #1
ffffc02086a0: b9018820 	str	w0, [x1, #392]
ffffc02086a4: 97fffdd2 	bl	0xffffc0207dec <cell_reconfig_completed>
ffffc02086a8: f9485281 	ldr	x1, [x20, #4256]
ffffc02086ac: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x434>
ffffc02086b0: 913d4000 	add	x0, x0, #3920
ffffc02086b4: 91002021 	add	x1, x1, #8
ffffc02086b8: 97fff91b 	bl	0xffffc0206b24 <printk>
ffffc02086bc: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x444>
ffffc02086c0: 913d8c00 	add	x0, x0, #3939
ffffc02086c4: 97fffcba 	bl	0xffffc02079ac <paging_dump_stats>
ffffc02086c8: 17ffff95 	b	0xffffc020851c <hypercall+0x298>
ffffc02086cc: 12800003 	mov	w3, #-1
ffffc02086d0: 910002c6 	add	x6, x22, #0
ffffc02086d4: 17ffffbb 	b	0xffffc02085c0 <hypercall+0x33c>
ffffc02086d8: f94012a1 	ldr	x1, [x21, #32]
ffffc02086dc: aa1403e0 	mov	x0, x20
ffffc02086e0: d63f0020 	blr	x1
ffffc02086e4: 2a0003fa 	mov	w26, w0
ffffc02086e8: 340001c0 	cbz	w0, 0xffffc0208720 <hypercall+0x49c>
ffffc02086ec: d100c2b5 	sub	x21, x21, #48
ffffc02086f0: 91074339 	add	x25, x25, #464
ffffc02086f4: eb1902bf 	cmp	x21, x25
ffffc02086f8: 540000a2 	b.hs	0xffffc020870c <hypercall+0x488>
ffffc02086fc: aa1403e0 	mov	x0, x20
ffffc0208700: 2a1a03f3 	mov	w19, w26
ffffc0208704: 97ffe112 	bl	0xffffc0200b4c <arch_cell_destroy>
ffffc0208708: 17ffffa7 	b	0xffffc02085a4 <hypercall+0x320>
ffffc020870c: f94016a1 	ldr	x1, [x21, #40]
ffffc0208710: aa1403e0 	mov	x0, x20
ffffc0208714: d100c2b5 	sub	x21, x21, #48
ffffc0208718: d63f0020 	blr	x1
ffffc020871c: 17fffff6 	b	0xffffc02086f4 <hypercall+0x470>
ffffc0208720: 9100c2b5 	add	x21, x21, #48
ffffc0208724: 17ffffb9 	b	0xffffc0208608 <hypercall+0x384>
ffffc0208728: 97ffe071 	bl	0xffffc02008ec <arch_park_cpu>
ffffc020872c: 1ada0f80 	sdiv	w0, w28, w26
ffffc0208730: f9485761 	ldr	x1, [x27, #4264]
ffffc0208734: 6b1c03e3 	negs	w3, w28
ffffc0208738: 91002021 	add	x1, x1, #8
ffffc020873c: 12001463 	and	w3, w3, #0x3f
ffffc0208740: 927df021 	and	x1, x1, #0xfffffffffffffff8
ffffc0208744: 8b20cc20 	add	x0, x1, w0, sxtw #3
ffffc0208748: 12001781 	and	w1, w28, #0x3f
ffffc020874c: 5a834422 	csneg	w2, w1, w3, mi
ffffc0208750: d2800021 	mov	x1, #1
ffffc0208754: 9ac22021 	lsl	x1, x1, x2
ffffc0208758: c85f7c03 	ldxr	x3, [x0]
ffffc020875c: 8a210063 	bic	x3, x3, x1
ffffc0208760: c8027c03 	stxr	w2, x3, [x0]
ffffc0208764: 35ffffa2 	cbnz	w2, 0xffffc0208758 <hypercall+0x4d4>
ffffc0208768: 8b153b35 	add	x21, x25, x21, lsl #14
ffffc020876c: d2860201 	mov	x1, #12304
ffffc0208770: f91806b4 	str	x20, [x21, #12296]
ffffc0208774: 8b0102b5 	add	x21, x21, x1
ffffc0208778: a9007ebf 	stp	xzr, xzr, [x21]
ffffc020877c: a9017ebf 	stp	xzr, xzr, [x21, #16]
ffffc0208780: 17ffffa9 	b	0xffffc0208624 <hypercall+0x3a0>
ffffc0208784: f9400ea0 	ldr	x0, [x21, #24]
ffffc0208788: ea1b001f 	tst	x0, x27
ffffc020878c: 54000180 	b.eq	0xffffc02087bc <hypercall+0x538>
ffffc0208790: f94006a0 	ldr	x0, [x21, #8]
ffffc0208794: f2402c1f 	tst	x0, #0xfff
ffffc0208798: 540002a0 	b.eq	0xffffc02087ec <hypercall+0x568>
ffffc020879c: aa1503e1 	mov	x1, x21
ffffc02087a0: aa1403e0 	mov	x0, x20
ffffc02087a4: 94000334 	bl	0xffffc0209474 <mmio_subpage_register>
ffffc02087a8: 2a0003f9 	mov	w25, w0
ffffc02087ac: 35000100 	cbnz	w0, 0xffffc02087cc <hypercall+0x548>
ffffc02087b0: 910082b5 	add	x21, x21, #32
ffffc02087b4: 1100075a 	add	w26, w26, #1
ffffc02087b8: 17ffffab 	b	0xffffc0208664 <hypercall+0x3e0>
ffffc02087bc: aa1503e0 	mov	x0, x21
ffffc02087c0: 97fffca5 	bl	0xffffc0207a54 <unmap_from_root_cell>
ffffc02087c4: 2a0003f9 	mov	w25, w0
ffffc02087c8: 34fffe40 	cbz	w0, 0xffffc0208790 <hypercall+0x50c>
ffffc02087cc: 2a1903f3 	mov	w19, w25
ffffc02087d0: aa1403e0 	mov	x0, x20
ffffc02087d4: 97fffe3e 	bl	0xffffc02080cc <cell_destroy_internal>
ffffc02087d8: 2a1703e2 	mov	w2, w23
ffffc02087dc: aa1403e1 	mov	x1, x20
ffffc02087e0: 91004300 	add	x0, x24, #16
ffffc02087e4: 97fff9a9 	bl	0xffffc0206e88 <page_free>
ffffc02087e8: 17ffff4d 	b	0xffffc020851c <hypercall+0x298>
ffffc02087ec: f9400aa0 	ldr	x0, [x21, #16]
ffffc02087f0: f2402c1f 	tst	x0, #0xfff
ffffc02087f4: 54fffd41 	b.ne	0xffffc020879c <hypercall+0x518>
ffffc02087f8: aa1503e1 	mov	x1, x21
ffffc02087fc: aa1403e0 	mov	x0, x20
ffffc0208800: 97ffe99e 	bl	0xffffc0202e78 <arch_map_memory_region>
ffffc0208804: 17ffffe9 	b	0xffffc02087a8 <hypercall+0x524>
ffffc0208808: aa0003f6 	mov	x22, x0
ffffc020880c: 17ffff9f 	b	0xffffc0208688 <hypercall+0x404>
ffffc0208810: 12800013 	mov	w19, #-1
ffffc0208814: 17ffff42 	b	0xffffc020851c <hypercall+0x298>
ffffc0208818: 12800213 	mov	w19, #-17
ffffc020881c: 17ffff40 	b	0xffffc020851c <hypercall+0x298>
ffffc0208820: 128000d3 	mov	w19, #-7
ffffc0208824: 17ffff3e 	b	0xffffc020851c <hypercall+0x298>
ffffc0208828: f9580494 	ldr	x20, [x4, #12296]
ffffc020882c: d0000080 	adrp	x0, 0xffffc021a000 <hypercall+0x5f0>
ffffc0208830: 91000000 	add	x0, x0, #0
ffffc0208834: eb00029f 	cmp	x20, x0
ffffc0208838: 54002381 	b.ne	0xffffc0208ca8 <hypercall+0xa24>
ffffc020883c: aa1403e0 	mov	x0, x20
ffffc0208840: 97fffd0f 	bl	0xffffc0207c7c <cell_suspend>
ffffc0208844: 9101c3a2 	add	x2, x29, #112
ffffc0208848: aa1303e1 	mov	x1, x19
ffffc020884c: 52800000 	mov	w0, #0
ffffc0208850: f81f8c54 	str	x20, [x2, #-8]!
ffffc0208854: 97fffda8 	bl	0xffffc0207ef4 <cell_management_prologue.part.2>
ffffc0208858: 35000ac0 	cbnz	w0, 0xffffc02089b0 <hypercall+0x72c>
ffffc020885c: f94037a0 	ldr	x0, [x29, #104]
ffffc0208860: b950c001 	ldr	w1, [x0, #4288]
ffffc0208864: 340001e1 	cbz	w1, 0xffffc02088a0 <hypercall+0x61c>
ffffc0208868: f9485013 	ldr	x19, [x0, #4256]
ffffc020886c: 52800014 	mov	w20, #0
ffffc0208870: b9403260 	ldr	w0, [x19, #48]
ffffc0208874: 91020000 	add	x0, x0, #128
ffffc0208878: 8b000273 	add	x19, x19, x0
ffffc020887c: f94037a0 	ldr	x0, [x29, #104]
ffffc0208880: f9485000 	ldr	x0, [x0, #4256]
ffffc0208884: b9403400 	ldr	w0, [x0, #52]
ffffc0208888: 6b00029f 	cmp	w20, w0
ffffc020888c: 540007c3 	b.lo	0xffffc0208984 <hypercall+0x700>
ffffc0208890: d2800000 	mov	x0, #0
ffffc0208894: 97fffdfb 	bl	0xffffc0208080 <config_commit>
ffffc0208898: f94037a0 	ldr	x0, [x29, #104]
ffffc020889c: b910c01f 	str	wzr, [x0, #4288]
ffffc02088a0: f94037a3 	ldr	x3, [x29, #104]
ffffc02088a4: 52800001 	mov	w1, #0
ffffc02088a8: d2820002 	mov	x2, #4096
ffffc02088ac: aa0303e0 	mov	x0, x3
ffffc02088b0: 94000162 	bl	0xffffc0208e38 <memset>
ffffc02088b4: aa0003e3 	mov	x3, x0
ffffc02088b8: 52800020 	mov	w0, #1
ffffc02088bc: 79000c60 	strh	w0, [x3, #6]
ffffc02088c0: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x648>
ffffc02088c4: 913ddc00 	add	x0, x0, #3959
ffffc02088c8: b9400001 	ldr	w1, [x0]
ffffc02088cc: b9000061 	str	w1, [x3]
ffffc02088d0: b8402000 	ldur	w0, [x0, #2]
ffffc02088d4: b8002060 	stur	w0, [x3, #2]
ffffc02088d8: f94037a0 	ldr	x0, [x29, #104]
ffffc02088dc: f9485001 	ldr	x1, [x0, #4256]
ffffc02088e0: b9402c21 	ldr	w1, [x1, #44]
ffffc02088e4: 36f00081 	tbz	w1, #30, 0xffffc02088f4 <hypercall+0x670>
ffffc02088e8: b9401461 	ldr	w1, [x3, #20]
ffffc02088ec: 32000021 	orr	w1, w1, #0x1
ffffc02088f0: b9001461 	str	w1, [x3, #20]
ffffc02088f4: f9485001 	ldr	x1, [x0, #4256]
ffffc02088f8: b9402c21 	ldr	w1, [x1, #44]
ffffc02088fc: 36f80081 	tbz	w1, #31, 0xffffc020890c <hypercall+0x688>
ffffc0208900: b9401461 	ldr	w1, [x3, #20]
ffffc0208904: 321f0021 	orr	w1, w1, #0x2
ffffc0208908: b9001461 	str	w1, [x3, #20]
ffffc020890c: f9485001 	ldr	x1, [x0, #4256]
ffffc0208910: b00000b4 	adrp	x20, 0xffffc021d000 <hypercall+0x6e0>
ffffc0208914: 12800013 	mov	w19, #-1
ffffc0208918: 91000294 	add	x20, x20, #0
ffffc020891c: a9461424 	ldp	x4, x5, [x1, #96]
ffffc0208920: a9019464 	stp	x4, x5, [x3, #24]
ffffc0208924: a9471424 	ldp	x4, x5, [x1, #112]
ffffc0208928: a9029464 	stp	x4, x5, [x3, #40]
ffffc020892c: 9400074e 	bl	0xffffc020a664 <pci_cell_reset>
ffffc0208930: f94037a0 	ldr	x0, [x29, #104]
ffffc0208934: 97ffe055 	bl	0xffffc0200a88 <arch_cell_reset>
ffffc0208938: f94037a0 	ldr	x0, [x29, #104]
ffffc020893c: 12800002 	mov	w2, #-1
ffffc0208940: f9485401 	ldr	x1, [x0, #4264]
ffffc0208944: 2a1303e0 	mov	w0, w19
ffffc0208948: 97fffcc0 	bl	0xffffc0207c48 <next_cpu>
ffffc020894c: 2a0003e1 	mov	w1, w0
ffffc0208950: f94037a2 	ldr	x2, [x29, #104]
ffffc0208954: aa0103f3 	mov	x19, x1
ffffc0208958: f9485443 	ldr	x3, [x2, #4264]
ffffc020895c: f9400063 	ldr	x3, [x3]
ffffc0208960: eb03003f 	cmp	x1, x3
ffffc0208964: 540002a9 	b.ls	0xffffc02089b8 <hypercall+0x734>
ffffc0208968: f9485041 	ldr	x1, [x2, #4256]
ffffc020896c: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x6f4>
ffffc0208970: 913df800 	add	x0, x0, #3966
ffffc0208974: 91002021 	add	x1, x1, #8
ffffc0208978: 97fff86b 	bl	0xffffc0206b24 <printk>
ffffc020897c: 52800009 	mov	w9, #0
ffffc0208980: 1400000a 	b	0xffffc02089a8 <hypercall+0x724>
ffffc0208984: f9400e60 	ldr	x0, [x19, #24]
ffffc0208988: 37300080 	tbnz	w0, #6, 0xffffc0208998 <hypercall+0x714>
ffffc020898c: 91008273 	add	x19, x19, #32
ffffc0208990: 11000694 	add	w20, w20, #1
ffffc0208994: 17ffffba 	b	0xffffc020887c <hypercall+0x5f8>
ffffc0208998: aa1303e0 	mov	x0, x19
ffffc020899c: 97fffc2e 	bl	0xffffc0207a54 <unmap_from_root_cell>
ffffc02089a0: 2a0003e9 	mov	w9, w0
ffffc02089a4: 34ffff40 	cbz	w0, 0xffffc020898c <hypercall+0x708>
ffffc02089a8: 97fffd3e 	bl	0xffffc0207ea0 <cell_resume.constprop.11>
ffffc02089ac: 2a0903e0 	mov	w0, w9
ffffc02089b0: 93407c00 	sxtw	x0, w0
ffffc02089b4: 17fffe7f 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc02089b8: 8b013a81 	add	x1, x20, x1, lsl #14
ffffc02089bc: b930343f 	str	wzr, [x1, #12340]
ffffc02089c0: 97ffdfc4 	bl	0xffffc02008d0 <arch_reset_cpu>
ffffc02089c4: 17ffffdd 	b	0xffffc0208938 <hypercall+0x6b4>
ffffc02089c8: f9580494 	ldr	x20, [x4, #12296]
ffffc02089cc: d0000080 	adrp	x0, 0xffffc021a000 <hypercall+0x790>
ffffc02089d0: 91000000 	add	x0, x0, #0
ffffc02089d4: eb00029f 	cmp	x20, x0
ffffc02089d8: 54000d81 	b.ne	0xffffc0208b88 <hypercall+0x904>
ffffc02089dc: aa1403e0 	mov	x0, x20
ffffc02089e0: 97fffca7 	bl	0xffffc0207c7c <cell_suspend>
ffffc02089e4: 9101c3a2 	add	x2, x29, #112
ffffc02089e8: aa1303e1 	mov	x1, x19
ffffc02089ec: 52800020 	mov	w0, #1
ffffc02089f0: f81f8c54 	str	x20, [x2, #-8]!
ffffc02089f4: 97fffd40 	bl	0xffffc0207ef4 <cell_management_prologue.part.2>
ffffc02089f8: 2a0003f3 	mov	w19, w0
ffffc02089fc: 35ffd920 	cbnz	w0, 0xffffc0208520 <hypercall+0x29c>
ffffc0208a00: b00000b5 	adrp	x21, 0xffffc021d000 <hypercall+0x7d0>
ffffc0208a04: 910002b5 	add	x21, x21, #0
ffffc0208a08: 12800014 	mov	w20, #-1
ffffc0208a0c: f94037a0 	ldr	x0, [x29, #104]
ffffc0208a10: 12800002 	mov	w2, #-1
ffffc0208a14: f9485401 	ldr	x1, [x0, #4264]
ffffc0208a18: 2a1403e0 	mov	w0, w20
ffffc0208a1c: 97fffc8b 	bl	0xffffc0207c48 <next_cpu>
ffffc0208a20: 2a0003e2 	mov	w2, w0
ffffc0208a24: f94037a1 	ldr	x1, [x29, #104]
ffffc0208a28: aa0203f4 	mov	x20, x2
ffffc0208a2c: f9485423 	ldr	x3, [x1, #4264]
ffffc0208a30: f9400063 	ldr	x3, [x3]
ffffc0208a34: eb03005f 	cmp	x2, x3
ffffc0208a38: 54000349 	b.ls	0xffffc0208aa0 <hypercall+0x81c>
ffffc0208a3c: b950c020 	ldr	w0, [x1, #4288]
ffffc0208a40: 35ffd6e0 	cbnz	w0, 0xffffc020851c <hypercall+0x298>
ffffc0208a44: 52800040 	mov	w0, #2
ffffc0208a48: f9485034 	ldr	x20, [x1, #4256]
ffffc0208a4c: b9000820 	str	w0, [x1, #8]
ffffc0208a50: 52800020 	mov	w0, #1
ffffc0208a54: b910c020 	str	w0, [x1, #4288]
ffffc0208a58: 52800015 	mov	w21, #0
ffffc0208a5c: b9403280 	ldr	w0, [x20, #48]
ffffc0208a60: 91020000 	add	x0, x0, #128
ffffc0208a64: 8b000294 	add	x20, x20, x0
ffffc0208a68: f94037a0 	ldr	x0, [x29, #104]
ffffc0208a6c: f9485000 	ldr	x0, [x0, #4256]
ffffc0208a70: b9403400 	ldr	w0, [x0, #52]
ffffc0208a74: 6b0002bf 	cmp	w21, w0
ffffc0208a78: 540001c3 	b.lo	0xffffc0208ab0 <hypercall+0x82c>
ffffc0208a7c: d2800000 	mov	x0, #0
ffffc0208a80: 97fffd80 	bl	0xffffc0208080 <config_commit>
ffffc0208a84: f94037a0 	ldr	x0, [x29, #104]
ffffc0208a88: f9485001 	ldr	x1, [x0, #4256]
ffffc0208a8c: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x814>
ffffc0208a90: 913e4400 	add	x0, x0, #3985
ffffc0208a94: 91002021 	add	x1, x1, #8
ffffc0208a98: 97fff823 	bl	0xffffc0206b24 <printk>
ffffc0208a9c: 17fffea0 	b	0xffffc020851c <hypercall+0x298>
ffffc0208aa0: 8b023aa2 	add	x2, x21, x2, lsl #14
ffffc0208aa4: b930345f 	str	wzr, [x2, #12340]
ffffc0208aa8: 97ffdf91 	bl	0xffffc02008ec <arch_park_cpu>
ffffc0208aac: 17ffffd8 	b	0xffffc0208a0c <hypercall+0x788>
ffffc0208ab0: f9400e80 	ldr	x0, [x20, #24]
ffffc0208ab4: 37300080 	tbnz	w0, #6, 0xffffc0208ac4 <hypercall+0x840>
ffffc0208ab8: 91008294 	add	x20, x20, #32
ffffc0208abc: 110006b5 	add	w21, w21, #1
ffffc0208ac0: 17ffffea 	b	0xffffc0208a68 <hypercall+0x7e4>
ffffc0208ac4: 52800001 	mov	w1, #0
ffffc0208ac8: aa1403e0 	mov	x0, x20
ffffc0208acc: 97fffc0b 	bl	0xffffc0207af8 <remap_to_root_cell>
ffffc0208ad0: 34ffff40 	cbz	w0, 0xffffc0208ab8 <hypercall+0x834>
ffffc0208ad4: 2a0003f3 	mov	w19, w0
ffffc0208ad8: 17fffe91 	b	0xffffc020851c <hypercall+0x298>
ffffc0208adc: f9580494 	ldr	x20, [x4, #12296]
ffffc0208ae0: d0000080 	adrp	x0, 0xffffc021a000 <hypercall+0x8a4>
ffffc0208ae4: 91000000 	add	x0, x0, #0
ffffc0208ae8: eb00029f 	cmp	x20, x0
ffffc0208aec: 540004e1 	b.ne	0xffffc0208b88 <hypercall+0x904>
ffffc0208af0: aa1403e0 	mov	x0, x20
ffffc0208af4: 97fffc62 	bl	0xffffc0207c7c <cell_suspend>
ffffc0208af8: 9101c3a2 	add	x2, x29, #112
ffffc0208afc: aa1303e1 	mov	x1, x19
ffffc0208b00: 52800040 	mov	w0, #2
ffffc0208b04: f81f8c54 	str	x20, [x2, #-8]!
ffffc0208b08: 97fffcfb 	bl	0xffffc0207ef4 <cell_management_prologue.part.2>
ffffc0208b0c: 2a0003f3 	mov	w19, w0
ffffc0208b10: 35ffd080 	cbnz	w0, 0xffffc0208520 <hypercall+0x29c>
ffffc0208b14: f94037a0 	ldr	x0, [x29, #104]
ffffc0208b18: f9485001 	ldr	x1, [x0, #4256]
ffffc0208b1c: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x8a4>
ffffc0208b20: 913ea800 	add	x0, x0, #4010
ffffc0208b24: 91002021 	add	x1, x1, #8
ffffc0208b28: 97fff7ff 	bl	0xffffc0206b24 <printk>
ffffc0208b2c: f94037a0 	ldr	x0, [x29, #104]
ffffc0208b30: 97fffd67 	bl	0xffffc02080cc <cell_destroy_internal>
ffffc0208b34: f94037a1 	ldr	x1, [x29, #104]
ffffc0208b38: f9486680 	ldr	x0, [x20, #4296]
ffffc0208b3c: eb01001f 	cmp	x0, x1
ffffc0208b40: 54000201 	b.ne	0xffffc0208b80 <hypercall+0x8fc>
ffffc0208b44: d0000022 	adrp	x2, 0xffffc020e000 <hypercall+0x8d8>
ffffc0208b48: f9486420 	ldr	x0, [x1, #4296]
ffffc0208b4c: f9086680 	str	x0, [x20, #4296]
ffffc0208b50: b9418840 	ldr	w0, [x2, #392]
ffffc0208b54: 51000400 	sub	w0, w0, #1
ffffc0208b58: b9018840 	str	w0, [x2, #392]
ffffc0208b5c: b9509822 	ldr	w2, [x1, #4248]
ffffc0208b60: 90000080 	adrp	x0, 0xffffc0218000 <hypercall+0x91c>
ffffc0208b64: 91004000 	add	x0, x0, #16
ffffc0208b68: 97fff8c8 	bl	0xffffc0206e88 <page_free>
ffffc0208b6c: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x8f4>
ffffc0208b70: 913ef400 	add	x0, x0, #4029
ffffc0208b74: 97fffb8e 	bl	0xffffc02079ac <paging_dump_stats>
ffffc0208b78: 97fffc9d 	bl	0xffffc0207dec <cell_reconfig_completed>
ffffc0208b7c: 17fffe68 	b	0xffffc020851c <hypercall+0x298>
ffffc0208b80: aa0003f4 	mov	x20, x0
ffffc0208b84: 17ffffed 	b	0xffffc0208b38 <hypercall+0x8b4>
ffffc0208b88: 12800013 	mov	w19, #-1
ffffc0208b8c: 17fffe65 	b	0xffffc0208520 <hypercall+0x29c>
ffffc0208b90: f100127f 	cmp	x19, #4
ffffc0208b94: 54000b88 	b.hi	0xffffc0208d04 <hypercall+0xa80>
ffffc0208b98: 7100127f 	cmp	w19, #4
ffffc0208b9c: 54000b48 	b.hi	0xffffc0208d04 <hypercall+0xa80>
ffffc0208ba0: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0x928>
ffffc0208ba4: 911c0000 	add	x0, x0, #1792
ffffc0208ba8: 38734800 	ldrb	w0, [x0, w19, uxtw]
ffffc0208bac: 10000061 	adr	x1, #12
ffffc0208bb0: 8b208820 	add	x0, x1, w0, sxtb #2
ffffc0208bb4: d61f0000 	br	x0
ffffc0208bb8: 90000080 	adrp	x0, 0xffffc0218000 <hypercall+0x974>
ffffc0208bbc: f9400c00 	ldr	x0, [x0, #24]
ffffc0208bc0: 17fffdfc 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208bc4: 90000080 	adrp	x0, 0xffffc0218000 <hypercall+0x980>
ffffc0208bc8: f9401000 	ldr	x0, [x0, #32]
ffffc0208bcc: 17fffdf9 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208bd0: d0000020 	adrp	x0, 0xffffc020e000 <hypercall+0x964>
ffffc0208bd4: f940b400 	ldr	x0, [x0, #360]
ffffc0208bd8: 17fffdf6 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208bdc: d0000020 	adrp	x0, 0xffffc020e000 <hypercall+0x970>
ffffc0208be0: f940b800 	ldr	x0, [x0, #368]
ffffc0208be4: 17fffdf3 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208be8: d0000020 	adrp	x0, 0xffffc020e000 <hypercall+0x97c>
ffffc0208bec: b9418800 	ldr	w0, [x0, #392]
ffffc0208bf0: 17fffdf0 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208bf4: f9580480 	ldr	x0, [x4, #12296]
ffffc0208bf8: d0000081 	adrp	x1, 0xffffc021a000 <hypercall+0x9bc>
ffffc0208bfc: 91000021 	add	x1, x1, #0
ffffc0208c00: eb01001f 	cmp	x0, x1
ffffc0208c04: 54000521 	b.ne	0xffffc0208ca8 <hypercall+0xa24>
ffffc0208c08: f9485001 	ldr	x1, [x0, #4256]
ffffc0208c0c: b9402821 	ldr	w1, [x1, #40]
ffffc0208c10: eb01027f 	cmp	x19, x1
ffffc0208c14: 540000c1 	b.ne	0xffffc0208c2c <hypercall+0x9a8>
ffffc0208c18: b9400800 	ldr	w0, [x0, #8]
ffffc0208c1c: 128002a1 	mov	w1, #-22
ffffc0208c20: 7100101f 	cmp	w0, #4
ffffc0208c24: 1a819000 	csel	w0, w0, w1, ls
ffffc0208c28: 17ffff62 	b	0xffffc02089b0 <hypercall+0x72c>
ffffc0208c2c: f9486400 	ldr	x0, [x0, #4296]
ffffc0208c30: b5fffec0 	cbnz	x0, 0xffffc0208c08 <hypercall+0x984>
ffffc0208c34: 12800020 	mov	w0, #-2
ffffc0208c38: 17ffff5e 	b	0xffffc02089b0 <hypercall+0x72c>
ffffc0208c3c: aa1303e0 	mov	x0, x19
ffffc0208c40: 97fffc7a 	bl	0xffffc0207e28 <cpu_id_valid>
ffffc0208c44: 35000060 	cbnz	w0, 0xffffc0208c50 <hypercall+0x9cc>
ffffc0208c48: 128002a0 	mov	w0, #-22
ffffc0208c4c: 17ffff59 	b	0xffffc02089b0 <hypercall+0x72c>
ffffc0208c50: d2a00020 	mov	x0, #65536
ffffc0208c54: d0000081 	adrp	x1, 0xffffc021a000 <hypercall+0xa18>
ffffc0208c58: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0208c5c: 91000021 	add	x1, x1, #0
ffffc0208c60: f9580400 	ldr	x0, [x0, #12296]
ffffc0208c64: eb01001f 	cmp	x0, x1
ffffc0208c68: 540001a1 	b.ne	0xffffc0208c9c <hypercall+0xa18>
ffffc0208c6c: b4000223 	cbz	x3, 0xffffc0208cb0 <hypercall+0xa2c>
ffffc0208c70: d10fa060 	sub	x0, x3, #1000
ffffc0208c74: f1001c1f 	cmp	x0, #7
ffffc0208c78: 54fffe88 	b.hi	0xffffc0208c48 <hypercall+0x9c4>
ffffc0208c7c: d3727e73 	ubfiz	x19, x19, #14, #32
ffffc0208c80: b00000a1 	adrp	x1, 0xffffc021d000 <hypercall+0xa50>
ffffc0208c84: 91000021 	add	x1, x1, #0
ffffc0208c88: 8b010273 	add	x19, x19, x1
ffffc0208c8c: 8b030a63 	add	x3, x19, x3, lsl #2
ffffc0208c90: b9607060 	ldr	w0, [x3, #8304]
ffffc0208c94: 12007800 	and	w0, w0, #0x7fffffff
ffffc0208c98: 17ffff46 	b	0xffffc02089b0 <hypercall+0x72c>
ffffc0208c9c: 2a1303e1 	mov	w1, w19
ffffc0208ca0: 97fffb5f 	bl	0xffffc0207a1c <cell_owns_cpu>
ffffc0208ca4: 35fffe40 	cbnz	w0, 0xffffc0208c6c <hypercall+0x9e8>
ffffc0208ca8: 12800000 	mov	w0, #-1
ffffc0208cac: 17ffff41 	b	0xffffc02089b0 <hypercall+0x72c>
ffffc0208cb0: d3727e73 	ubfiz	x19, x19, #14, #32
ffffc0208cb4: b00000a1 	adrp	x1, 0xffffc021d000 <hypercall+0xa84>
ffffc0208cb8: 91000021 	add	x1, x1, #0
ffffc0208cbc: 8b010273 	add	x19, x19, x1
ffffc0208cc0: b9703660 	ldr	w0, [x19, #12340]
ffffc0208cc4: 7100001f 	cmp	w0, #0
ffffc0208cc8: 1a9f07e0 	cset	w0, ne
ffffc0208ccc: 531f7800 	lsl	w0, w0, #1
ffffc0208cd0: 17ffff38 	b	0xffffc02089b0 <hypercall+0x72c>
ffffc0208cd4: f9580480 	ldr	x0, [x4, #12296]
ffffc0208cd8: f9485000 	ldr	x0, [x0, #4256]
ffffc0208cdc: b9402c00 	ldr	w0, [x0, #44]
ffffc0208ce0: 36f00160 	tbz	w0, #30, 0xffffc0208d0c <hypercall+0xa88>
ffffc0208ce4: 12001e61 	and	w1, w19, #0xff
ffffc0208ce8: f0000000 	adrp	x0, 0xffffc020b000 <hypercall+0xa70>
ffffc0208cec: 913f5000 	add	x0, x0, #4052
ffffc0208cf0: 97fff78d 	bl	0xffffc0206b24 <printk>
ffffc0208cf4: d2800000 	mov	x0, #0
ffffc0208cf8: 17fffdae 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208cfc: 928004a0 	mov	x0, #-38
ffffc0208d00: d65f03c0 	ret
ffffc0208d04: 928002a0 	mov	x0, #-22
ffffc0208d08: 17fffdaa 	b	0xffffc02083b0 <hypercall+0x12c>
ffffc0208d0c: 92800000 	mov	x0, #-1
ffffc0208d10: 17fffda8 	b	0xffffc02083b0 <hypercall+0x12c>

0000ffffc0208d14 <panic_stop>:
ffffc0208d14: d2840001 	mov	x1, #8192
ffffc0208d18: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0208d1c: f2a00021 	movk	x1, #1, lsl #16
ffffc0208d20: f2dfe001 	movk	x1, #65280, lsl #32
ffffc0208d24: 910003fd 	mov	x29, sp
ffffc0208d28: f9480420 	ldr	x0, [x1, #4104]
ffffc0208d2c: b9500021 	ldr	w1, [x1, #4096]
ffffc0208d30: b4000260 	cbz	x0, 0xffffc0208d7c <panic_stop+0x68>
ffffc0208d34: f9485000 	ldr	x0, [x0, #4256]
ffffc0208d38: f0000002 	adrp	x2, 0xffffc020b000 <panic_stop+0x30>
ffffc0208d3c: 913f5c42 	add	x2, x2, #4055
ffffc0208d40: 91002003 	add	x3, x0, #8
ffffc0208d44: f100001f 	cmp	x0, #0
ffffc0208d48: 9a830042 	csel	x2, x2, x3, eq
ffffc0208d4c: f0000000 	adrp	x0, 0xffffc020b000 <panic_stop+0x44>
ffffc0208d50: 913f7c00 	add	x0, x0, #4063
ffffc0208d54: 97fff7a7 	bl	0xffffc0206bf0 <panic_printk>
ffffc0208d58: 97ffe826 	bl	0xffffc0202df0 <phys_processor_id>
ffffc0208d5c: d0000021 	adrp	x1, 0xffffc020e000 <panic_stop+0x60>
ffffc0208d60: f940c821 	ldr	x1, [x1, #400]
ffffc0208d64: eb01001f 	cmp	x0, x1
ffffc0208d68: 54000081 	b.ne	0xffffc0208d78 <panic_stop+0x64>
ffffc0208d6c: b0000080 	adrp	x0, 0xffffc0219000 <panic_park+0x28>
ffffc0208d70: 91000000 	add	x0, x0, #0
ffffc0208d74: f9000c1f 	str	xzr, [x0, #24]
ffffc0208d78: 97ffdfb1 	bl	0xffffc0200c3c <arch_panic_stop>
ffffc0208d7c: f0000002 	adrp	x2, 0xffffc020b000 <panic_park>
ffffc0208d80: 913f5c42 	add	x2, x2, #4055
ffffc0208d84: 17fffff2 	b	0xffffc0208d4c <panic_stop+0x38>

0000ffffc0208d88 <panic_park>:
ffffc0208d88: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0208d8c: f0000000 	adrp	x0, 0xffffc020b000 <panic_park+0x10>
ffffc0208d90: 913ff400 	add	x0, x0, #4093
ffffc0208d94: 910003fd 	mov	x29, sp
ffffc0208d98: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0208d9c: d2840014 	mov	x20, #8192
ffffc0208da0: f2a00034 	movk	x20, #1, lsl #16
ffffc0208da4: f2dfe014 	movk	x20, #65280, lsl #32
ffffc0208da8: f9480693 	ldr	x19, [x20, #4104]
ffffc0208dac: b9500281 	ldr	w1, [x20, #4096]
ffffc0208db0: f9485262 	ldr	x2, [x19, #4256]
ffffc0208db4: 91002042 	add	x2, x2, #8
ffffc0208db8: 97fff78e 	bl	0xffffc0206bf0 <panic_printk>
ffffc0208dbc: 52800020 	mov	w0, #1
ffffc0208dc0: b00000a6 	adrp	x6, 0xffffc021d000 <panic_park+0x8c>
ffffc0208dc4: 910000c6 	add	x6, x6, #0
ffffc0208dc8: b9103680 	str	w0, [x20, #4148]
ffffc0208dcc: 12800000 	mov	w0, #-1
ffffc0208dd0: f9485661 	ldr	x1, [x19, #4264]
ffffc0208dd4: 12800002 	mov	w2, #-1
ffffc0208dd8: 97fffb9c 	bl	0xffffc0207c48 <next_cpu>
ffffc0208ddc: 2a0003e1 	mov	w1, w0
ffffc0208de0: f9485662 	ldr	x2, [x19, #4264]
ffffc0208de4: f9400042 	ldr	x2, [x2]
ffffc0208de8: eb02003f 	cmp	x1, x2
ffffc0208dec: 54000089 	b.ls	0xffffc0208dfc <panic_park+0x74>
ffffc0208df0: 52800060 	mov	w0, #3
ffffc0208df4: b9000a60 	str	w0, [x19, #8]
ffffc0208df8: 14000004 	b	0xffffc0208e08 <panic_park+0x80>
ffffc0208dfc: 8b0138c1 	add	x1, x6, x1, lsl #14
ffffc0208e00: b9703421 	ldr	w1, [x1, #12340]
ffffc0208e04: 35fffe61 	cbnz	w1, 0xffffc0208dd0 <panic_park+0x48>
ffffc0208e08: 97ffde8b 	bl	0xffffc0200834 <arm_cpu_park>
ffffc0208e0c: 97ffe7f9 	bl	0xffffc0202df0 <phys_processor_id>
ffffc0208e10: d0000021 	adrp	x1, 0xffffc020e000 <panic_park+0xa0>
ffffc0208e14: f940c821 	ldr	x1, [x1, #400]
ffffc0208e18: eb01001f 	cmp	x0, x1
ffffc0208e1c: 54000081 	b.ne	0xffffc0208e2c <panic_park+0xa4>
ffffc0208e20: b0000080 	adrp	x0, 0xffffc0219000 <strcmp+0x10>
ffffc0208e24: 91000000 	add	x0, x0, #0
ffffc0208e28: f9000c1f 	str	xzr, [x0, #24]
ffffc0208e2c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0208e30: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0208e34: d65f03c0 	ret

0000ffffc0208e38 <memset>:
ffffc0208e38: d2800003 	mov	x3, #0
ffffc0208e3c: eb03005f 	cmp	x2, x3
ffffc0208e40: 54000041 	b.ne	0xffffc0208e48 <memset+0x10>
ffffc0208e44: d65f03c0 	ret
ffffc0208e48: 38236801 	strb	w1, [x0, x3]
ffffc0208e4c: 91000463 	add	x3, x3, #1
ffffc0208e50: 17fffffb 	b	0xffffc0208e3c <memset+0x4>

0000ffffc0208e54 <strcmp>:
ffffc0208e54: d2800003 	mov	x3, #0
ffffc0208e58: 38636802 	ldrb	w2, [x0, x3]
ffffc0208e5c: 38636824 	ldrb	w4, [x1, x3]
ffffc0208e60: 6b04005f 	cmp	w2, w4
ffffc0208e64: 54000060 	b.eq	0xffffc0208e70 <strcmp+0x1c>
ffffc0208e68: 4b040040 	sub	w0, w2, w4
ffffc0208e6c: d65f03c0 	ret
ffffc0208e70: 91000463 	add	x3, x3, #1
ffffc0208e74: 35ffff22 	cbnz	w2, 0xffffc0208e58 <strcmp+0x4>
ffffc0208e78: 52800000 	mov	w0, #0
ffffc0208e7c: 17fffffc 	b	0xffffc0208e6c <strcmp+0x18>

0000ffffc0208e80 <memcpy>:
ffffc0208e80: d2800003 	mov	x3, #0
ffffc0208e84: eb03005f 	cmp	x2, x3
ffffc0208e88: 54000041 	b.ne	0xffffc0208e90 <memcpy+0x10>
ffffc0208e8c: d65f03c0 	ret
ffffc0208e90: 38636824 	ldrb	w4, [x1, x3]
ffffc0208e94: 38236804 	strb	w4, [x0, x3]
ffffc0208e98: 91000463 	add	x3, x3, #1
ffffc0208e9c: 17fffffa 	b	0xffffc0208e84 <memcpy+0x4>

0000ffffc0208ea0 <spin_lock>:
ffffc0208ea0: f9800011 	prfm	pstl1strm, [x0]
ffffc0208ea4: 885ffc01 	ldaxr	w1, [x0]
ffffc0208ea8: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc0208eac: 88037c02 	stxr	w3, w2, [x0]
ffffc0208eb0: 35ffffa3 	cbnz	w3, 0xffffc0208ea4 <spin_lock+0x4>
ffffc0208eb4: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc0208eb8: 340000c2 	cbz	w2, 0xffffc0208ed0 <spin_lock+0x30>
ffffc0208ebc: d50320bf 	sevl
ffffc0208ec0: d503205f 	wfe
ffffc0208ec4: 485ffc03 	ldaxrh	w3, [x0]
ffffc0208ec8: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc0208ecc: 35ffffa2 	cbnz	w2, 0xffffc0208ec0 <spin_lock+0x20>
ffffc0208ed0: d65f03c0 	ret

0000ffffc0208ed4 <find_region>:
ffffc0208ed4: 8b224022 	add	x2, x1, w2, uxtw
ffffc0208ed8: f9487007 	ldr	x7, [x0, #4320]
ffffc0208edc: 36000047 	tbz	w7, #0, 0xffffc0208ee4 <find_region+0x10>
ffffc0208ee0: 17fffffe 	b	0xffffc0208ed8 <find_region+0x4>
ffffc0208ee4: b950f805 	ldr	w5, [x0, #4344]
ffffc0208ee8: 5280000b 	mov	w11, #0
ffffc0208eec: 35000065 	cbnz	w5, 0xffffc0208ef8 <find_region+0x24>
ffffc0208ef0: 12800000 	mov	w0, #-1
ffffc0208ef4: 1400001e 	b	0xffffc0208f6c <find_region+0x98>
ffffc0208ef8: 53017ca8 	lsr	w8, w5, #1
ffffc0208efc: f948700d 	ldr	x13, [x0, #4320]
ffffc0208f00: 0b0b0106 	add	w6, w8, w11
ffffc0208f04: f948740a 	ldr	x10, [x0, #4328]
ffffc0208f08: eb0701bf 	cmp	x13, x7
ffffc0208f0c: d37c7cc9 	ubfiz	x9, x6, #4, #32
ffffc0208f10: 8b09014c 	add	x12, x10, x9
ffffc0208f14: f869694a 	ldr	x10, [x10, x9]
ffffc0208f18: 54fffe01 	b.ne	0xffffc0208ed8 <find_region+0x4>
ffffc0208f1c: eb01015f 	cmp	x10, x1
ffffc0208f20: 54000288 	b.hi	0xffffc0208f70 <find_region+0x9c>
ffffc0208f24: f940058b 	ldr	x11, [x12, #8]
ffffc0208f28: 8b0b014b 	add	x11, x10, x11
ffffc0208f2c: eb02017f 	cmp	x11, x2
ffffc0208f30: 540000a2 	b.hs	0xffffc0208f44 <find_region+0x70>
ffffc0208f34: 510004a5 	sub	w5, w5, #1
ffffc0208f38: 110004cb 	add	w11, w6, #1
ffffc0208f3c: 4b0800a5 	sub	w5, w5, w8
ffffc0208f40: 17ffffeb 	b	0xffffc0208eec <find_region+0x18>
ffffc0208f44: b40000c3 	cbz	x3, 0xffffc0208f5c <find_region+0x88>
ffffc0208f48: f900006a 	str	x10, [x3]
ffffc0208f4c: f9487805 	ldr	x5, [x0, #4336]
ffffc0208f50: 8b0900a5 	add	x5, x5, x9
ffffc0208f54: a94024a8 	ldp	x8, x9, [x5]
ffffc0208f58: a9002488 	stp	x8, x9, [x4]
ffffc0208f5c: f9487005 	ldr	x5, [x0, #4320]
ffffc0208f60: eb0700bf 	cmp	x5, x7
ffffc0208f64: 54fffba1 	b.ne	0xffffc0208ed8 <find_region+0x4>
ffffc0208f68: 2a0603e0 	mov	w0, w6
ffffc0208f6c: d65f03c0 	ret
ffffc0208f70: 2a0803e5 	mov	w5, w8
ffffc0208f74: 17ffffde 	b	0xffffc0208eec <find_region+0x18>

0000ffffc0208f78 <mmio_cell_init>:
ffffc0208f78: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0208f7c: 910003fd 	mov	x29, sp
ffffc0208f80: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0208f84: d0000034 	adrp	x20, 0xffffc020e000 <mmio_cell_init+0x24>
ffffc0208f88: f90013f5 	str	x21, [sp, #32]
ffffc0208f8c: aa0003f3 	mov	x19, x0
ffffc0208f90: d0000035 	adrp	x21, 0xffffc020e000 <mmio_cell_init+0x30>
ffffc0208f94: 91074294 	add	x20, x20, #464
ffffc0208f98: 9108c2b5 	add	x21, x21, #560
ffffc0208f9c: eb15029f 	cmp	x20, x21
ffffc0208fa0: 54000343 	b.lo	0xffffc0209008 <mmio_cell_init+0x90>
ffffc0208fa4: f9485262 	ldr	x2, [x19, #4256]
ffffc0208fa8: 52800001 	mov	w1, #0
ffffc0208fac: b9403040 	ldr	w0, [x2, #48]
ffffc0208fb0: 91020000 	add	x0, x0, #128
ffffc0208fb4: 8b000040 	add	x0, x2, x0
ffffc0208fb8: b9403443 	ldr	w3, [x2, #52]
ffffc0208fbc: 6b01007f 	cmp	w3, w1
ffffc0208fc0: 54000348 	b.hi	0xffffc0209028 <mmio_cell_init+0xb0>
ffffc0208fc4: b950fe61 	ldr	w1, [x19, #4348]
ffffc0208fc8: 90000080 	adrp	x0, 0xffffc0218000 <mmio_cell_init+0x90>
ffffc0208fcc: 91004000 	add	x0, x0, #16
ffffc0208fd0: d37be821 	lsl	x1, x1, #5
ffffc0208fd4: 913ffc21 	add	x1, x1, #4095
ffffc0208fd8: d34cac21 	ubfx	x1, x1, #12, #32
ffffc0208fdc: 97fff7a7 	bl	0xffffc0206e78 <page_alloc>
ffffc0208fe0: b40003c0 	cbz	x0, 0xffffc0209058 <mmio_cell_init+0xe0>
ffffc0208fe4: b950fe61 	ldr	w1, [x19, #4348]
ffffc0208fe8: f9087660 	str	x0, [x19, #4328]
ffffc0208fec: 8b011000 	add	x0, x0, x1, lsl #4
ffffc0208ff0: f9087a60 	str	x0, [x19, #4336]
ffffc0208ff4: 52800000 	mov	w0, #0
ffffc0208ff8: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0208ffc: f94013f5 	ldr	x21, [sp, #32]
ffffc0209000: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0209004: d65f03c0 	ret
ffffc0209008: f9400e81 	ldr	x1, [x20, #24]
ffffc020900c: aa1303e0 	mov	x0, x19
ffffc0209010: 9100c294 	add	x20, x20, #48
ffffc0209014: d63f0020 	blr	x1
ffffc0209018: b950fe61 	ldr	w1, [x19, #4348]
ffffc020901c: 0b000020 	add	w0, w1, w0
ffffc0209020: b910fe60 	str	w0, [x19, #4348]
ffffc0209024: 17ffffde 	b	0xffffc0208f9c <mmio_cell_init+0x24>
ffffc0209028: f9400403 	ldr	x3, [x0, #8]
ffffc020902c: f2402c7f 	tst	x3, #0xfff
ffffc0209030: 54000081 	b.ne	0xffffc0209040 <mmio_cell_init+0xc8>
ffffc0209034: f9400803 	ldr	x3, [x0, #16]
ffffc0209038: f2402c7f 	tst	x3, #0xfff
ffffc020903c: 54000080 	b.eq	0xffffc020904c <mmio_cell_init+0xd4>
ffffc0209040: b950fe63 	ldr	w3, [x19, #4348]
ffffc0209044: 11000463 	add	w3, w3, #1
ffffc0209048: b910fe63 	str	w3, [x19, #4348]
ffffc020904c: 91008000 	add	x0, x0, #32
ffffc0209050: 11000421 	add	w1, w1, #1
ffffc0209054: 17ffffd9 	b	0xffffc0208fb8 <mmio_cell_init+0x40>
ffffc0209058: 12800160 	mov	w0, #-12
ffffc020905c: 17ffffe7 	b	0xffffc0208ff8 <mmio_cell_init+0x80>

0000ffffc0209060 <mmio_region_register>:
ffffc0209060: aa0003e5 	mov	x5, x0
ffffc0209064: aa0103ea 	mov	x10, x1
ffffc0209068: aa0203ee 	mov	x14, x2
ffffc020906c: d2821b01 	mov	x1, #4312
ffffc0209070: aa0303eb 	mov	x11, x3
ffffc0209074: 8b010009 	add	x9, x0, x1
ffffc0209078: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020907c: aa0903e0 	mov	x0, x9
ffffc0209080: 910003fd 	mov	x29, sp
ffffc0209084: 97ffff87 	bl	0xffffc0208ea0 <spin_lock>
ffffc0209088: b950f8a1 	ldr	w1, [x5, #4344]
ffffc020908c: b950fca0 	ldr	w0, [x5, #4348]
ffffc0209090: 6b00003f 	cmp	w1, w0
ffffc0209094: 54000103 	b.lo	0xffffc02090b4 <mmio_region_register+0x54>
ffffc0209098: 7961b0a0 	ldrh	w0, [x5, #4312]
ffffc020909c: 11000400 	add	w0, w0, #1
ffffc02090a0: 489ffd20 	stlrh	w0, [x9]
ffffc02090a4: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc02090a8: f0000000 	adrp	x0, 0xffffc020c000 <mmio_region_register+0x54>
ffffc02090ac: 91006800 	add	x0, x0, #26
ffffc02090b0: 17fff69d 	b	0xffffc0206b24 <printk>
ffffc02090b4: d2800006 	mov	x6, #0
ffffc02090b8: 2a0603e2 	mov	w2, w6
ffffc02090bc: d37cecc0 	lsl	x0, x6, #4
ffffc02090c0: 6b06003f 	cmp	w1, w6
ffffc02090c4: 540000c9 	b.ls	0xffffc02090dc <mmio_region_register+0x7c>
ffffc02090c8: f94874a3 	ldr	x3, [x5, #4328]
ffffc02090cc: 910004c6 	add	x6, x6, #1
ffffc02090d0: f8606863 	ldr	x3, [x3, x0]
ffffc02090d4: eb0a007f 	cmp	x3, x10
ffffc02090d8: 54ffff09 	b.ls	0xffffc02090b8 <mmio_region_register+0x58>
ffffc02090dc: f94870a6 	ldr	x6, [x5, #4320]
ffffc02090e0: 910004c6 	add	x6, x6, #1
ffffc02090e4: f90870a6 	str	x6, [x5, #4320]
ffffc02090e8: d5033bbf 	dmb	ish
ffffc02090ec: b950f8a7 	ldr	w7, [x5, #4344]
ffffc02090f0: d37c7ce8 	ubfiz	x8, x7, #4, #32
ffffc02090f4: 6b07005f 	cmp	w2, w7
ffffc02090f8: f94874a6 	ldr	x6, [x5, #4328]
ffffc02090fc: 540002c3 	b.lo	0xffffc0209154 <mmio_region_register+0xf4>
ffffc0209100: f82068ca 	str	x10, [x6, x0]
ffffc0209104: f94874a1 	ldr	x1, [x5, #4328]
ffffc0209108: 8b000021 	add	x1, x1, x0
ffffc020910c: f900042e 	str	x14, [x1, #8]
ffffc0209110: f94878a1 	ldr	x1, [x5, #4336]
ffffc0209114: f820682b 	str	x11, [x1, x0]
ffffc0209118: f94878a1 	ldr	x1, [x5, #4336]
ffffc020911c: 8b000020 	add	x0, x1, x0
ffffc0209120: f9000404 	str	x4, [x0, #8]
ffffc0209124: b950f8a0 	ldr	w0, [x5, #4344]
ffffc0209128: 11000400 	add	w0, w0, #1
ffffc020912c: b910f8a0 	str	w0, [x5, #4344]
ffffc0209130: d5033bbf 	dmb	ish
ffffc0209134: f94870a0 	ldr	x0, [x5, #4320]
ffffc0209138: 91000400 	add	x0, x0, #1
ffffc020913c: f90870a0 	str	x0, [x5, #4320]
ffffc0209140: 7961b0a0 	ldrh	w0, [x5, #4312]
ffffc0209144: 11000400 	add	w0, w0, #1
ffffc0209148: 489ffd20 	stlrh	w0, [x9]
ffffc020914c: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0209150: d65f03c0 	ret
ffffc0209154: 510004e7 	sub	w7, w7, #1
ffffc0209158: 8b0800c3 	add	x3, x6, x8
ffffc020915c: d37c7ce1 	ubfiz	x1, x7, #4, #32
ffffc0209160: 8b0100c6 	add	x6, x6, x1
ffffc0209164: a94034cc 	ldp	x12, x13, [x6]
ffffc0209168: a900346c 	stp	x12, x13, [x3]
ffffc020916c: f94878a6 	ldr	x6, [x5, #4336]
ffffc0209170: 8b0800c3 	add	x3, x6, x8
ffffc0209174: 8b0100c6 	add	x6, x6, x1
ffffc0209178: d1004108 	sub	x8, x8, #16
ffffc020917c: a94034cc 	ldp	x12, x13, [x6]
ffffc0209180: a900346c 	stp	x12, x13, [x3]
ffffc0209184: 17ffffdc 	b	0xffffc02090f4 <mmio_region_register+0x94>

0000ffffc0209188 <mmio_region_unregister>:
ffffc0209188: aa0003ee 	mov	x14, x0
ffffc020918c: aa0103e5 	mov	x5, x1
ffffc0209190: d2821b01 	mov	x1, #4312
ffffc0209194: 8b01000f 	add	x15, x0, x1
ffffc0209198: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020919c: aa0f03e0 	mov	x0, x15
ffffc02091a0: 910003fd 	mov	x29, sp
ffffc02091a4: 97ffff3f 	bl	0xffffc0208ea0 <spin_lock>
ffffc02091a8: aa0503e1 	mov	x1, x5
ffffc02091ac: d2800004 	mov	x4, #0
ffffc02091b0: d2800003 	mov	x3, #0
ffffc02091b4: 52800022 	mov	w2, #1
ffffc02091b8: aa0e03e0 	mov	x0, x14
ffffc02091bc: 97ffff46 	bl	0xffffc0208ed4 <find_region>
ffffc02091c0: 37f80240 	tbnz	w0, #31, 0xffffc0209208 <mmio_region_unregister+0x80>
ffffc02091c4: f94871c1 	ldr	x1, [x14, #4320]
ffffc02091c8: 91000421 	add	x1, x1, #1
ffffc02091cc: f90871c1 	str	x1, [x14, #4320]
ffffc02091d0: d5033bbf 	dmb	ish
ffffc02091d4: 93407c01 	sxtw	x1, w0
ffffc02091d8: 91000421 	add	x1, x1, #1
ffffc02091dc: d37cec21 	lsl	x1, x1, #4
ffffc02091e0: b950f9c3 	ldr	w3, [x14, #4344]
ffffc02091e4: 2a0003e2 	mov	w2, w0
ffffc02091e8: 6b00007f 	cmp	w3, w0
ffffc02091ec: 54000188 	b.hi	0xffffc020921c <mmio_region_unregister+0x94>
ffffc02091f0: 51000463 	sub	w3, w3, #1
ffffc02091f4: b910f9c3 	str	w3, [x14, #4344]
ffffc02091f8: d5033bbf 	dmb	ish
ffffc02091fc: f94871c0 	ldr	x0, [x14, #4320]
ffffc0209200: 91000400 	add	x0, x0, #1
ffffc0209204: f90871c0 	str	x0, [x14, #4320]
ffffc0209208: 7961b1c0 	ldrh	w0, [x14, #4312]
ffffc020920c: 11000400 	add	w0, w0, #1
ffffc0209210: 489ffde0 	stlrh	w0, [x15]
ffffc0209214: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0209218: d65f03c0 	ret
ffffc020921c: f94875c3 	ldr	x3, [x14, #4328]
ffffc0209220: d37c7c42 	ubfiz	x2, x2, #4, #32
ffffc0209224: 11000400 	add	w0, w0, #1
ffffc0209228: 8b020064 	add	x4, x3, x2
ffffc020922c: 8b010063 	add	x3, x3, x1
ffffc0209230: a9401c66 	ldp	x6, x7, [x3]
ffffc0209234: a9001c86 	stp	x6, x7, [x4]
ffffc0209238: f94879c3 	ldr	x3, [x14, #4336]
ffffc020923c: 8b020062 	add	x2, x3, x2
ffffc0209240: 8b010063 	add	x3, x3, x1
ffffc0209244: 91004021 	add	x1, x1, #16
ffffc0209248: a9401464 	ldp	x4, x5, [x3]
ffffc020924c: a9001444 	stp	x4, x5, [x2]
ffffc0209250: 17ffffe4 	b	0xffffc02091e0 <mmio_region_unregister+0x58>

0000ffffc0209254 <mmio_handle_access>:
ffffc0209254: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0209258: aa0003ee 	mov	x14, x0
ffffc020925c: 910003fd 	mov	x29, sp
ffffc0209260: b9400802 	ldr	w2, [x0, #8]
ffffc0209264: 910083a4 	add	x4, x29, #32
ffffc0209268: f9400001 	ldr	x1, [x0]
ffffc020926c: d2840000 	mov	x0, #8192
ffffc0209270: f2a00020 	movk	x0, #1, lsl #16
ffffc0209274: 910063a3 	add	x3, x29, #24
ffffc0209278: f2dfe000 	movk	x0, #65280, lsl #32
ffffc020927c: f9480400 	ldr	x0, [x0, #4104]
ffffc0209280: 97ffff15 	bl	0xffffc0208ed4 <find_region>
ffffc0209284: 37f80140 	tbnz	w0, #31, 0xffffc02092ac <mmio_handle_access+0x58>
ffffc0209288: a9418ba1 	ldp	x1, x2, [x29, #24]
ffffc020928c: f94001c0 	ldr	x0, [x14]
ffffc0209290: cb010000 	sub	x0, x0, x1
ffffc0209294: f90001c0 	str	x0, [x14]
ffffc0209298: f94017a0 	ldr	x0, [x29, #40]
ffffc020929c: aa0e03e1 	mov	x1, x14
ffffc02092a0: d63f0040 	blr	x2
ffffc02092a4: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc02092a8: d65f03c0 	ret
ffffc02092ac: 52800000 	mov	w0, #0
ffffc02092b0: 17fffffd 	b	0xffffc02092a4 <mmio_handle_access+0x50>

0000ffffc02092b4 <mmio_cell_exit>:
ffffc02092b4: b950fc02 	ldr	w2, [x0, #4348]
ffffc02092b8: f9487401 	ldr	x1, [x0, #4328]
ffffc02092bc: f0000060 	adrp	x0, 0xffffc0218000 <mmio_perform_access+0x24>
ffffc02092c0: 91004000 	add	x0, x0, #16
ffffc02092c4: d37be842 	lsl	x2, x2, #5
ffffc02092c8: 913ffc42 	add	x2, x2, #4095
ffffc02092cc: d34cac42 	ubfx	x2, x2, #12, #32
ffffc02092d0: 17fff6ee 	b	0xffffc0206e88 <page_free>

0000ffffc02092d4 <mmio_perform_access>:
ffffc02092d4: 29411023 	ldp	w3, w4, [x1, #8]
ffffc02092d8: f9400022 	ldr	x2, [x1]
ffffc02092dc: 51000463 	sub	w3, w3, #1
ffffc02092e0: 71001c7f 	cmp	w3, #7
ffffc02092e4: 34000284 	cbz	w4, 0xffffc0209334 <mmio_perform_access+0x60>
ffffc02092e8: 54000128 	b.hi	0xffffc020930c <mmio_perform_access+0x38>
ffffc02092ec: d0000004 	adrp	x4, 0xffffc020b000 <mmio_perform_access+0x20>
ffffc02092f0: 911c2084 	add	x4, x4, #1800
ffffc02092f4: 38634883 	ldrb	w3, [x4, w3, uxtw]
ffffc02092f8: 10000064 	adr	x4, #12
ffffc02092fc: 8b238883 	add	x3, x4, w3, sxtb #2
ffffc0209300: d61f0060 	br	x3
ffffc0209304: 39404021 	ldrb	w1, [x1, #16]
ffffc0209308: 38226801 	strb	w1, [x0, x2]
ffffc020930c: d65f03c0 	ret
ffffc0209310: 79402021 	ldrh	w1, [x1, #16]
ffffc0209314: 78226801 	strh	w1, [x0, x2]
ffffc0209318: 17fffffd 	b	0xffffc020930c <mmio_perform_access+0x38>
ffffc020931c: f9400821 	ldr	x1, [x1, #16]
ffffc0209320: b8226801 	str	w1, [x0, x2]
ffffc0209324: 17fffffa 	b	0xffffc020930c <mmio_perform_access+0x38>
ffffc0209328: f9400821 	ldr	x1, [x1, #16]
ffffc020932c: f8226801 	str	x1, [x0, x2]
ffffc0209330: 17fffff7 	b	0xffffc020930c <mmio_perform_access+0x38>
ffffc0209334: 54fffec8 	b.hi	0xffffc020930c <mmio_perform_access+0x38>
ffffc0209338: d0000004 	adrp	x4, 0xffffc020b000 <mmio_perform_access+0x6c>
ffffc020933c: 911c4084 	add	x4, x4, #1808
ffffc0209340: 38634883 	ldrb	w3, [x4, w3, uxtw]
ffffc0209344: 10000064 	adr	x4, #12
ffffc0209348: 8b238883 	add	x3, x4, w3, sxtb #2
ffffc020934c: d61f0060 	br	x3
ffffc0209350: 38626800 	ldrb	w0, [x0, x2]
ffffc0209354: 92401c00 	and	x0, x0, #0xff
ffffc0209358: f9000820 	str	x0, [x1, #16]
ffffc020935c: 17ffffec 	b	0xffffc020930c <mmio_perform_access+0x38>
ffffc0209360: 78626800 	ldrh	w0, [x0, x2]
ffffc0209364: 92403c00 	and	x0, x0, #0xffff
ffffc0209368: 17fffffc 	b	0xffffc0209358 <mmio_perform_access+0x84>
ffffc020936c: b8626800 	ldr	w0, [x0, x2]
ffffc0209370: 2a0003e0 	mov	w0, w0
ffffc0209374: 17fffff9 	b	0xffffc0209358 <mmio_perform_access+0x84>
ffffc0209378: f8626800 	ldr	x0, [x0, x2]
ffffc020937c: 17fffff7 	b	0xffffc0209358 <mmio_perform_access+0x84>

0000ffffc0209380 <mmio_handle_subpage>:
ffffc0209380: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0209384: 910003fd 	mov	x29, sp
ffffc0209388: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020938c: aa0003f4 	mov	x20, x0
ffffc0209390: aa0103f3 	mov	x19, x1
ffffc0209394: b9400c20 	ldr	w0, [x1, #12]
ffffc0209398: f9400024 	ldr	x4, [x1]
ffffc020939c: 7100001f 	cmp	w0, #0
ffffc02093a0: f9400e82 	ldr	x2, [x20, #24]
ffffc02093a4: 9a9f07e3 	cset	x3, ne
ffffc02093a8: b9400820 	ldr	w0, [x1, #8]
ffffc02093ac: f9400281 	ldr	x1, [x20]
ffffc02093b0: 91000463 	add	x3, x3, #1
ffffc02093b4: ea03005f 	tst	x2, x3
ffffc02093b8: 8b010081 	add	x1, x4, x1
ffffc02093bc: 540003e0 	b.eq	0xffffc0209438 <mmio_handle_subpage+0xb8>
ffffc02093c0: 53103c03 	lsl	w3, w0, #16
ffffc02093c4: ea02007f 	tst	x3, x2
ffffc02093c8: 54000380 	b.eq	0xffffc0209438 <mmio_handle_subpage+0xb8>
ffffc02093cc: 51000400 	sub	w0, w0, #1
ffffc02093d0: 9274cc21 	and	x1, x1, #0xfffffffffffff000
ffffc02093d4: ea04001f 	tst	x0, x4
ffffc02093d8: 54000040 	b.eq	0xffffc02093e0 <mmio_handle_subpage+0x60>
ffffc02093dc: 364002e2 	tbz	w2, #8, 0xffffc0209438 <mmio_handle_subpage+0xb8>
ffffc02093e0: d2820002 	mov	x2, #4096
ffffc02093e4: aa0203e0 	mov	x0, x2
ffffc02093e8: 52800025 	mov	w5, #1
ffffc02093ec: d280e8a4 	mov	x4, #1861
ffffc02093f0: f2a00020 	movk	x0, #1, lsl #16
ffffc02093f4: d2dfe003 	mov	x3, #280375465082880
ffffc02093f8: f2dfe000 	movk	x0, #65280, lsl #32
ffffc02093fc: 97fff763 	bl	0xffffc0207188 <paging_create>
ffffc0209400: 350001c0 	cbnz	w0, 0xffffc0209438 <mmio_handle_subpage+0xb8>
ffffc0209404: f9400260 	ldr	x0, [x19]
ffffc0209408: d2dfe002 	mov	x2, #280375465082880
ffffc020940c: aa1303e1 	mov	x1, x19
ffffc0209410: 9274cc00 	and	x0, x0, #0xfffffffffffff000
ffffc0209414: cb000042 	sub	x2, x2, x0
ffffc0209418: f9400280 	ldr	x0, [x20]
ffffc020941c: 92402c00 	and	x0, x0, #0xfff
ffffc0209420: 8b000040 	add	x0, x2, x0
ffffc0209424: 97ffffac 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0209428: 52800020 	mov	w0, #1
ffffc020942c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0209430: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0209434: d65f03c0 	ret
ffffc0209438: 29410a63 	ldp	w3, w2, [x19, #8]
ffffc020943c: d0000000 	adrp	x0, 0xffffc020b000 <mmio_handle_subpage+0xc4>
ffffc0209440: f9400284 	ldr	x4, [x20]
ffffc0209444: 912f8c00 	add	x0, x0, #3043
ffffc0209448: 7100005f 	cmp	w2, #0
ffffc020944c: f9400262 	ldr	x2, [x19]
ffffc0209450: d0000001 	adrp	x1, 0xffffc020b000 <mmio_handle_subpage+0xd8>
ffffc0209454: 912fa421 	add	x1, x1, #3049
ffffc0209458: 9a800021 	csel	x1, x1, x0, eq
ffffc020945c: 8b020082 	add	x2, x4, x2
ffffc0209460: f0000000 	adrp	x0, 0xffffc020c000 <mmio_handle_subpage+0xec>
ffffc0209464: 91013800 	add	x0, x0, #78
ffffc0209468: 97fff5e2 	bl	0xffffc0206bf0 <panic_printk>
ffffc020946c: 12800000 	mov	w0, #-1
ffffc0209470: 17ffffef 	b	0xffffc020942c <mmio_handle_subpage+0xac>

0000ffffc0209474 <mmio_subpage_register>:
ffffc0209474: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0209478: aa0103e4 	mov	x4, x1
ffffc020947c: 90000003 	adrp	x3, 0xffffc0209000 <mmio_subpage_register+0x8>
ffffc0209480: 910e0063 	add	x3, x3, #896
ffffc0209484: 910003fd 	mov	x29, sp
ffffc0209488: a9408821 	ldp	x1, x2, [x1, #8]
ffffc020948c: 97fffef5 	bl	0xffffc0209060 <mmio_region_register>
ffffc0209490: 52800000 	mov	w0, #0
ffffc0209494: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0209498: d65f03c0 	ret

0000ffffc020949c <mmio_subpage_unregister>:
ffffc020949c: f9400421 	ldr	x1, [x1, #8]
ffffc02094a0: 17ffff3a 	b	0xffffc0209188 <mmio_region_unregister>

0000ffffc02094a4 <pci_msix_access_handler>:
ffffc02094a4: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc02094a8: 910003fd 	mov	x29, sp
ffffc02094ac: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02094b0: aa0003f3 	mov	x19, x0
ffffc02094b4: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc02094b8: f9400022 	ldr	x2, [x1]
ffffc02094bc: f9400003 	ldr	x3, [x0]
ffffc02094c0: 53020c55 	ubfx	w21, w2, #2, #2
ffffc02094c4: f240045f 	tst	x2, #0x3
ffffc02094c8: 540004c1 	b.ne	0xffffc0209560 <pci_msix_access_handler+0xbc>
ffffc02094cc: aa0103f4 	mov	x20, x1
ffffc02094d0: 79404863 	ldrh	w3, [x3, #36]
ffffc02094d4: d344fc41 	lsr	x1, x2, #4
ffffc02094d8: 6b03003f 	cmp	w1, w3
ffffc02094dc: b9400e84 	ldr	w4, [x20, #12]
ffffc02094e0: 34000244 	cbz	w4, 0xffffc0209528 <pci_msix_access_handler+0x84>
ffffc02094e4: 540003e2 	b.hs	0xffffc0209560 <pci_msix_access_handler+0xbc>
ffffc02094e8: f9402802 	ldr	x2, [x0, #80]
ffffc02094ec: d37c7c36 	ubfiz	x22, x1, #4, #32
ffffc02094f0: 2a1503e3 	mov	w3, w21
ffffc02094f4: f9400a84 	ldr	x4, [x20, #16]
ffffc02094f8: 8b160042 	add	x2, x2, x22
ffffc02094fc: b8237844 	str	w4, [x2, x3, lsl #2]
ffffc0209500: 97ffe796 	bl	0xffffc0203358 <arch_pci_update_msix_vector>
ffffc0209504: 37f802e0 	tbnz	w0, #31, 0xffffc0209560 <pci_msix_access_handler+0xbc>
ffffc0209508: 71000ebf 	cmp	w21, #3
ffffc020950c: 540000a1 	b.ne	0xffffc0209520 <pci_msix_access_handler+0x7c>
ffffc0209510: f9402660 	ldr	x0, [x19, #72]
ffffc0209514: f9400a81 	ldr	x1, [x20, #16]
ffffc0209518: 8b160000 	add	x0, x0, x22
ffffc020951c: b9000c01 	str	w1, [x0, #12]
ffffc0209520: 52800020 	mov	w0, #1
ffffc0209524: 14000018 	b	0xffffc0209584 <pci_msix_access_handler+0xe0>
ffffc0209528: 54000062 	b.hs	0xffffc0209534 <pci_msix_access_handler+0x90>
ffffc020952c: 71000ebf 	cmp	w21, #3
ffffc0209530: 540000c1 	b.ne	0xffffc0209548 <pci_msix_access_handler+0xa4>
ffffc0209534: f9402660 	ldr	x0, [x19, #72]
ffffc0209538: b8626800 	ldr	w0, [x0, x2]
ffffc020953c: 2a0003e0 	mov	w0, w0
ffffc0209540: f9000a80 	str	x0, [x20, #16]
ffffc0209544: 17fffff7 	b	0xffffc0209520 <pci_msix_access_handler+0x7c>
ffffc0209548: f9402800 	ldr	x0, [x0, #80]
ffffc020954c: d37c7c21 	ubfiz	x1, x1, #4, #32
ffffc0209550: 2a1503f5 	mov	w21, w21
ffffc0209554: 8b010001 	add	x1, x0, x1
ffffc0209558: b8757820 	ldr	w0, [x1, x21, lsl #2]
ffffc020955c: 17fffff9 	b	0xffffc0209540 <pci_msix_access_handler+0x9c>
ffffc0209560: f9400260 	ldr	x0, [x19]
ffffc0209564: 79400801 	ldrh	w1, [x0, #4]
ffffc0209568: f0000000 	adrp	x0, 0xffffc020c000 <pci_msix_access_handler+0xd0>
ffffc020956c: 9101f800 	add	x0, x0, #126
ffffc0209570: 12000823 	and	w3, w1, #0x7
ffffc0209574: d3431c22 	ubfx	x2, x1, #3, #5
ffffc0209578: 53087c21 	lsr	w1, w1, #8
ffffc020957c: 97fff59d 	bl	0xffffc0206bf0 <panic_printk>
ffffc0209580: 12800000 	mov	w0, #-1
ffffc0209584: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0209588: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020958c: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0209590: d65f03c0 	ret

0000ffffc0209594 <pci_update_msix.isra.0>:
ffffc0209594: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0209598: 910003fd 	mov	x29, sp
ffffc020959c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc02095a0: aa0003f4 	mov	x20, x0
ffffc02095a4: 52800013 	mov	w19, #0
ffffc02095a8: f9400280 	ldr	x0, [x20]
ffffc02095ac: 79404800 	ldrh	w0, [x0, #36]
ffffc02095b0: 6b00027f 	cmp	w19, w0
ffffc02095b4: 540000a3 	b.lo	0xffffc02095c8 <pci_update_msix.isra.0+0x34>
ffffc02095b8: 52800000 	mov	w0, #0
ffffc02095bc: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc02095c0: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc02095c4: d65f03c0 	ret
ffffc02095c8: 2a1303e1 	mov	w1, w19
ffffc02095cc: aa1403e0 	mov	x0, x20
ffffc02095d0: 97ffe762 	bl	0xffffc0203358 <arch_pci_update_msix_vector>
ffffc02095d4: 37ffff40 	tbnz	w0, #31, 0xffffc02095bc <pci_update_msix.isra.0+0x28>
ffffc02095d8: 11000673 	add	w19, w19, #1
ffffc02095dc: 17fffff3 	b	0xffffc02095a8 <pci_update_msix.isra.0+0x14>

0000ffffc02095e0 <jailhouse_cell_pci_caps>:
ffffc02095e0: b9403802 	ldr	w2, [x0, #56]
ffffc02095e4: 52800181 	mov	w1, #12
ffffc02095e8: 52800703 	mov	w3, #56
ffffc02095ec: 9ba17c42 	umull	x2, w2, w1
ffffc02095f0: b9404401 	ldr	w1, [x0, #68]
ffffc02095f4: 9ba30821 	umaddl	x1, w1, w3, x2
ffffc02095f8: b9403003 	ldr	w3, [x0, #48]
ffffc02095fc: b9404002 	ldr	w2, [x0, #64]
ffffc0209600: 8b030042 	add	x2, x2, x3
ffffc0209604: b9403403 	ldr	w3, [x0, #52]
ffffc0209608: 8b020022 	add	x2, x1, x2
ffffc020960c: b9403c01 	ldr	w1, [x0, #60]
ffffc0209610: 8b030021 	add	x1, x1, x3
ffffc0209614: 8b011441 	add	x1, x2, x1, lsl #5
ffffc0209618: 91020021 	add	x1, x1, #128
ffffc020961c: 8b010000 	add	x0, x0, x1
ffffc0209620: d65f03c0 	ret

0000ffffc0209624 <pci_find_capability>:
ffffc0209624: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0209628: aa0003e5 	mov	x5, x0
ffffc020962c: 12003c24 	and	w4, w1, #0xffff
ffffc0209630: 910003fd 	mov	x29, sp
ffffc0209634: f9400400 	ldr	x0, [x0, #8]
ffffc0209638: f9485000 	ldr	x0, [x0, #4256]
ffffc020963c: 97ffffe9 	bl	0xffffc02095e0 <jailhouse_cell_pci_caps>
ffffc0209640: f94000a1 	ldr	x1, [x5]
ffffc0209644: 79403c22 	ldrh	w2, [x1, #30]
ffffc0209648: 8b020c00 	add	x0, x0, x2, lsl #3
ffffc020964c: 79404022 	ldrh	w2, [x1, #32]
ffffc0209650: 8b020c02 	add	x2, x0, x2, lsl #3
ffffc0209654: eb02001f 	cmp	x0, x2
ffffc0209658: 54000081 	b.ne	0xffffc0209668 <pci_find_capability+0x44>
ffffc020965c: d2800000 	mov	x0, #0
ffffc0209660: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0209664: d65f03c0 	ret
ffffc0209668: 79400401 	ldrh	w1, [x0, #2]
ffffc020966c: 6b04003f 	cmp	w1, w4
ffffc0209670: 540000a8 	b.hi	0xffffc0209684 <pci_find_capability+0x60>
ffffc0209674: 79400803 	ldrh	w3, [x0, #4]
ffffc0209678: 0b010063 	add	w3, w3, w1
ffffc020967c: 6b04007f 	cmp	w3, w4
ffffc0209680: 54ffff0c 	b.gt	0xffffc0209660 <pci_find_capability+0x3c>
ffffc0209684: 91002000 	add	x0, x0, #8
ffffc0209688: 17fffff3 	b	0xffffc0209654 <pci_find_capability+0x30>

0000ffffc020968c <jailhouse_cell_pci_devices>:
ffffc020968c: b9403001 	ldr	w1, [x0, #48]
ffffc0209690: 52800183 	mov	w3, #12
ffffc0209694: b9404002 	ldr	w2, [x0, #64]
ffffc0209698: 8b010042 	add	x2, x2, x1
ffffc020969c: b9403801 	ldr	w1, [x0, #56]
ffffc02096a0: 9ba30821 	umaddl	x1, w1, w3, x2
ffffc02096a4: b9403403 	ldr	w3, [x0, #52]
ffffc02096a8: 91020022 	add	x2, x1, #128
ffffc02096ac: b9403c01 	ldr	w1, [x0, #60]
ffffc02096b0: 8b030021 	add	x1, x1, x3
ffffc02096b4: 8b011441 	add	x1, x2, x1, lsl #5
ffffc02096b8: 8b010000 	add	x0, x0, x1
ffffc02096bc: d65f03c0 	ret

0000ffffc02096c0 <pci_mmio_count_regions>:
ffffc02096c0: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc02096c4: 910003fd 	mov	x29, sp
ffffc02096c8: f9485004 	ldr	x4, [x0, #4256]
ffffc02096cc: aa0403e0 	mov	x0, x4
ffffc02096d0: 97ffffef 	bl	0xffffc020968c <jailhouse_cell_pci_devices>
ffffc02096d4: b9404482 	ldr	w2, [x4, #68]
ffffc02096d8: 90000081 	adrp	x1, 0xffffc0219000 <pci_mmio_count_regions+0x58>
ffffc02096dc: 52800703 	mov	w3, #56
ffffc02096e0: f9400021 	ldr	x1, [x1]
ffffc02096e4: 9ba30042 	umaddl	x2, w2, w3, x0
ffffc02096e8: f844c021 	ldur	x1, [x1, #76]
ffffc02096ec: f100003f 	cmp	x1, #0
ffffc02096f0: 1a9f07e1 	cset	w1, ne
ffffc02096f4: eb02001f 	cmp	x0, x2
ffffc02096f8: 54000081 	b.ne	0xffffc0209708 <pci_mmio_count_regions+0x48>
ffffc02096fc: 2a0103e0 	mov	w0, w1
ffffc0209700: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0209704: d65f03c0 	ret
ffffc0209708: 39400003 	ldrb	w3, [x0]
ffffc020970c: 71000c7f 	cmp	w3, #3
ffffc0209710: 54000081 	b.ne	0xffffc0209720 <pci_mmio_count_regions+0x60>
ffffc0209714: 11000821 	add	w1, w1, #2
ffffc0209718: 9100e000 	add	x0, x0, #56
ffffc020971c: 17fffff6 	b	0xffffc02096f4 <pci_mmio_count_regions+0x34>
ffffc0209720: f9401403 	ldr	x3, [x0, #40]
ffffc0209724: f100007f 	cmp	x3, #0
ffffc0209728: 1a810421 	cinc	w1, w1, ne
ffffc020972c: 17fffffb 	b	0xffffc0209718 <pci_mmio_count_regions+0x58>

0000ffffc0209730 <pci_read_config>:
ffffc0209730: f0000083 	adrp	x3, 0xffffc021c000 <pci_read_config+0x4c>
ffffc0209734: 91000064 	add	x4, x3, #0
ffffc0209738: 12003c00 	and	w0, w0, #0xffff
ffffc020973c: 12003c21 	and	w1, w1, #0xffff
ffffc0209740: f9400063 	ldr	x3, [x3]
ffffc0209744: b4000083 	cbz	x3, 0xffffc0209754 <pci_read_config+0x24>
ffffc0209748: 39402084 	ldrb	w4, [x4, #8]
ffffc020974c: 6b40209f 	cmp	w4, w0, lsr #8
ffffc0209750: 54000042 	b.hs	0xffffc0209758 <pci_read_config+0x28>
ffffc0209754: 17ffe6de 	b	0xffffc02032cc <arch_pci_read_config>
ffffc0209758: d3743c00 	ubfiz	x0, x0, #12, #16
ffffc020975c: 7100045f 	cmp	w2, #1
ffffc0209760: 8b212001 	add	x1, x0, w1, uxth
ffffc0209764: 54000081 	b.ne	0xffffc0209774 <pci_read_config+0x44>
ffffc0209768: 38616860 	ldrb	w0, [x3, x1]
ffffc020976c: 12001c00 	and	w0, w0, #0xff
ffffc0209770: d65f03c0 	ret
ffffc0209774: 7100085f 	cmp	w2, #2
ffffc0209778: 54000081 	b.ne	0xffffc0209788 <pci_read_config+0x58>
ffffc020977c: 78616860 	ldrh	w0, [x3, x1]
ffffc0209780: 12003c00 	and	w0, w0, #0xffff
ffffc0209784: 17fffffb 	b	0xffffc0209770 <pci_read_config+0x40>
ffffc0209788: b8616860 	ldr	w0, [x3, x1]
ffffc020978c: 17fffff9 	b	0xffffc0209770 <pci_read_config+0x40>

0000ffffc0209790 <pci_write_config>:
ffffc0209790: f0000084 	adrp	x4, 0xffffc021c000 <pci_write_config+0x4c>
ffffc0209794: 91000085 	add	x5, x4, #0
ffffc0209798: 12003c00 	and	w0, w0, #0xffff
ffffc020979c: 12003c21 	and	w1, w1, #0xffff
ffffc02097a0: f9400084 	ldr	x4, [x4]
ffffc02097a4: b4000084 	cbz	x4, 0xffffc02097b4 <pci_write_config+0x24>
ffffc02097a8: 394020a5 	ldrb	w5, [x5, #8]
ffffc02097ac: 6b4020bf 	cmp	w5, w0, lsr #8
ffffc02097b0: 54000042 	b.hs	0xffffc02097b8 <pci_write_config+0x28>
ffffc02097b4: 17ffe6c8 	b	0xffffc02032d4 <arch_pci_write_config>
ffffc02097b8: d3743c00 	ubfiz	x0, x0, #12, #16
ffffc02097bc: 7100047f 	cmp	w3, #1
ffffc02097c0: 8b212001 	add	x1, x0, w1, uxth
ffffc02097c4: 54000081 	b.ne	0xffffc02097d4 <pci_write_config+0x44>
ffffc02097c8: 12001c42 	and	w2, w2, #0xff
ffffc02097cc: 38216882 	strb	w2, [x4, x1]
ffffc02097d0: d65f03c0 	ret
ffffc02097d4: 7100087f 	cmp	w3, #2
ffffc02097d8: 54000081 	b.ne	0xffffc02097e8 <pci_write_config+0x58>
ffffc02097dc: 12003c42 	and	w2, w2, #0xffff
ffffc02097e0: 78216882 	strh	w2, [x4, x1]
ffffc02097e4: 17fffffb 	b	0xffffc02097d0 <pci_write_config+0x40>
ffffc02097e8: b8216882 	str	w2, [x4, x1]
ffffc02097ec: 17fffff9 	b	0xffffc02097d0 <pci_write_config+0x40>

0000ffffc02097f0 <pci_suppress_msix>:
ffffc02097f0: b9403804 	ldr	w4, [x0, #56]
ffffc02097f4: 34000042 	cbz	w2, 0xffffc02097fc <pci_suppress_msix+0xc>
ffffc02097f8: 32020084 	orr	w4, w4, #0x40000000
ffffc02097fc: f9400000 	ldr	x0, [x0]
ffffc0209800: 52800083 	mov	w3, #4
ffffc0209804: 79400421 	ldrh	w1, [x1, #2]
ffffc0209808: 2a0403e2 	mov	w2, w4
ffffc020980c: 79400800 	ldrh	w0, [x0, #4]
ffffc0209810: 17ffffe0 	b	0xffffc0209790 <pci_write_config>

0000ffffc0209814 <pci_shutdown>:
ffffc0209814: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0209818: 910003fd 	mov	x29, sp
ffffc020981c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0209820: b0000094 	adrp	x20, 0xffffc021a000 <pci_shutdown+0x50>
ffffc0209824: 91000294 	add	x20, x20, #0
ffffc0209828: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020982c: a90363f7 	stp	x23, x24, [sp, #48]
ffffc0209830: f9486a93 	ldr	x19, [x20, #4304]
ffffc0209834: b40001b3 	cbz	x19, 0xffffc0209868 <pci_shutdown+0x54>
ffffc0209838: d297d077 	mov	x23, #48771
ffffc020983c: f2a5f417 	movk	x23, #12192, lsl #16
ffffc0209840: f2c17d17 	movk	x23, #3048, lsl #32
ffffc0209844: f2f05f57 	movk	x23, #33530, lsl #48
ffffc0209848: f9486a80 	ldr	x0, [x20, #4304]
ffffc020984c: f9485281 	ldr	x1, [x20, #4256]
ffffc0209850: cb000260 	sub	x0, x19, x0
ffffc0209854: 9343fc00 	asr	x0, x0, #3
ffffc0209858: b9404421 	ldr	w1, [x1, #68]
ffffc020985c: 9b177c00 	mul	x0, x0, x23
ffffc0209860: eb01001f 	cmp	x0, x1
ffffc0209864: 540000cb 	b.lt	0xffffc020987c <pci_shutdown+0x68>
ffffc0209868: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020986c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0209870: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc0209874: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0209878: d65f03c0 	ret
ffffc020987c: f9400660 	ldr	x0, [x19, #8]
ffffc0209880: b4000260 	cbz	x0, 0xffffc02098cc <pci_shutdown+0xb8>
ffffc0209884: f9485000 	ldr	x0, [x0, #4256]
ffffc0209888: 52800018 	mov	w24, #0
ffffc020988c: 97ffff55 	bl	0xffffc02095e0 <jailhouse_cell_pci_caps>
ffffc0209890: f9400261 	ldr	x1, [x19]
ffffc0209894: 79403c35 	ldrh	w21, [x1, #30]
ffffc0209898: 8b150c15 	add	x21, x0, x21, lsl #3
ffffc020989c: f9400260 	ldr	x0, [x19]
ffffc02098a0: 79404001 	ldrh	w1, [x0, #32]
ffffc02098a4: 6b18003f 	cmp	w1, w24
ffffc02098a8: 54000168 	b.hi	0xffffc02098d4 <pci_shutdown+0xc0>
ffffc02098ac: f9400661 	ldr	x1, [x19, #8]
ffffc02098b0: eb14003f 	cmp	x1, x20
ffffc02098b4: 540000c0 	b.eq	0xffffc02098cc <pci_shutdown+0xb8>
ffffc02098b8: 79400800 	ldrh	w0, [x0, #4]
ffffc02098bc: 52800043 	mov	w3, #2
ffffc02098c0: 52808002 	mov	w2, #1024
ffffc02098c4: 52800081 	mov	w1, #4
ffffc02098c8: 97ffffb2 	bl	0xffffc0209790 <pci_write_config>
ffffc02098cc: 91056273 	add	x19, x19, #344
ffffc02098d0: 17ffffde 	b	0xffffc0209848 <pci_shutdown+0x34>
ffffc02098d4: 794002a0 	ldrh	w0, [x21]
ffffc02098d8: 7100141f 	cmp	w0, #5
ffffc02098dc: 540002a1 	b.ne	0xffffc0209930 <pci_shutdown+0x11c>
ffffc02098e0: d2800036 	mov	x22, #1
ffffc02098e4: 14000009 	b	0xffffc0209908 <pci_shutdown+0xf4>
ffffc02098e8: 8b160a60 	add	x0, x19, x22, lsl #2
ffffc02098ec: 794006a1 	ldrh	w1, [x21, #2]
ffffc02098f0: 52800083 	mov	w3, #4
ffffc02098f4: b9402802 	ldr	w2, [x0, #40]
ffffc02098f8: 0b160821 	add	w1, w1, w22, lsl #2
ffffc02098fc: 79400880 	ldrh	w0, [x4, #4]
ffffc0209900: 910006d6 	add	x22, x22, #1
ffffc0209904: 97ffffa3 	bl	0xffffc0209790 <pci_write_config>
ffffc0209908: f9400264 	ldr	x4, [x19]
ffffc020990c: 39408c80 	ldrb	w0, [x4, #35]
ffffc0209910: 7100001f 	cmp	w0, #0
ffffc0209914: 1a9f07e0 	cset	w0, ne
ffffc0209918: 11000c00 	add	w0, w0, #3
ffffc020991c: 6b16001f 	cmp	w0, w22
ffffc0209920: 54fffe48 	b.hi	0xffffc02098e8 <pci_shutdown+0xd4>
ffffc0209924: 910022b5 	add	x21, x21, #8
ffffc0209928: 11000718 	add	w24, w24, #1
ffffc020992c: 17ffffdc 	b	0xffffc020989c <pci_shutdown+0x88>
ffffc0209930: 7100441f 	cmp	w0, #17
ffffc0209934: 54ffff81 	b.ne	0xffffc0209924 <pci_shutdown+0x110>
ffffc0209938: d2800000 	mov	x0, #0
ffffc020993c: 1400000d 	b	0xffffc0209970 <pci_shutdown+0x15c>
ffffc0209940: d37cec05 	lsl	x5, x0, #4
ffffc0209944: 52800001 	mov	w1, #0
ffffc0209948: a9448e62 	ldp	x2, x3, [x19, #72]
ffffc020994c: 2a0103e4 	mov	w4, w1
ffffc0209950: 11000421 	add	w1, w1, #1
ffffc0209954: 8b050063 	add	x3, x3, x5
ffffc0209958: 8b050042 	add	x2, x2, x5
ffffc020995c: 71000c3f 	cmp	w1, #3
ffffc0209960: b8647863 	ldr	w3, [x3, x4, lsl #2]
ffffc0209964: b8247843 	str	w3, [x2, x4, lsl #2]
ffffc0209968: 54ffff01 	b.ne	0xffffc0209948 <pci_shutdown+0x134>
ffffc020996c: 91000400 	add	x0, x0, #1
ffffc0209970: f9400261 	ldr	x1, [x19]
ffffc0209974: 79404821 	ldrh	w1, [x1, #36]
ffffc0209978: 6b00003f 	cmp	w1, w0
ffffc020997c: 54fffe28 	b.hi	0xffffc0209940 <pci_shutdown+0x12c>
ffffc0209980: 52800002 	mov	w2, #0
ffffc0209984: aa1503e1 	mov	x1, x21
ffffc0209988: aa1303e0 	mov	x0, x19
ffffc020998c: 97ffff99 	bl	0xffffc02097f0 <pci_suppress_msix>
ffffc0209990: 17ffffe5 	b	0xffffc0209924 <pci_shutdown+0x110>

0000ffffc0209994 <pci_get_assigned_device>:
ffffc0209994: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0209998: aa0003e4 	mov	x4, x0
ffffc020999c: 12003c26 	and	w6, w1, #0xffff
ffffc02099a0: 910003fd 	mov	x29, sp
ffffc02099a4: f9485005 	ldr	x5, [x0, #4256]
ffffc02099a8: aa0503e0 	mov	x0, x5
ffffc02099ac: 97ffff38 	bl	0xffffc020968c <jailhouse_cell_pci_devices>
ffffc02099b0: b94044a5 	ldr	w5, [x5, #68]
ffffc02099b4: d2800002 	mov	x2, #0
ffffc02099b8: d2800701 	mov	x1, #56
ffffc02099bc: 6b0200bf 	cmp	w5, w2
ffffc02099c0: 54000068 	b.hi	0xffffc02099cc <pci_get_assigned_device+0x38>
ffffc02099c4: d2800000 	mov	x0, #0
ffffc02099c8: 1400000c 	b	0xffffc02099f8 <pci_get_assigned_device+0x64>
ffffc02099cc: 91000443 	add	x3, x2, #1
ffffc02099d0: 9b010067 	madd	x7, x3, x1, x0
ffffc02099d4: 785cc0e7 	ldurh	w7, [x7, #-52]
ffffc02099d8: 6b0600ff 	cmp	w7, w6
ffffc02099dc: 54000121 	b.ne	0xffffc0209a00 <pci_get_assigned_device+0x6c>
ffffc02099e0: f9486880 	ldr	x0, [x4, #4304]
ffffc02099e4: d2802b01 	mov	x1, #344
ffffc02099e8: 9b010040 	madd	x0, x2, x1, x0
ffffc02099ec: f9400401 	ldr	x1, [x0, #8]
ffffc02099f0: f100003f 	cmp	x1, #0
ffffc02099f4: 9a9f1000 	csel	x0, x0, xzr, ne
ffffc02099f8: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc02099fc: d65f03c0 	ret
ffffc0209a00: aa0303e2 	mov	x2, x3
ffffc0209a04: 17ffffee 	b	0xffffc02099bc <pci_get_assigned_device+0x28>

0000ffffc0209a08 <pci_cfg_read_moderate>:
ffffc0209a08: aa0303e2 	mov	x2, x3
ffffc0209a0c: b50000a0 	cbnz	x0, 0xffffc0209a20 <pci_cfg_read_moderate+0x18>
ffffc0209a10: 12800000 	mov	w0, #-1
ffffc0209a14: b9000060 	str	w0, [x3]
ffffc0209a18: 52800040 	mov	w0, #2
ffffc0209a1c: d65f03c0 	ret
ffffc0209a20: f9400009 	ldr	x9, [x0]
ffffc0209a24: aa0003e7 	mov	x7, x0
ffffc0209a28: 12003c26 	and	w6, w1, #0xffff
ffffc0209a2c: 39400123 	ldrb	w3, [x9]
ffffc0209a30: 7100087f 	cmp	w3, #2
ffffc0209a34: 540002c0 	b.eq	0xffffc0209a8c <pci_cfg_read_moderate+0x84>
ffffc0209a38: 510040c1 	sub	w1, w6, #16
ffffc0209a3c: 12003c25 	and	w5, w1, #0xffff
ffffc0209a40: 71005cbf 	cmp	w5, #23
ffffc0209a44: 54000108 	b.hi	0xffffc0209a64 <pci_cfg_read_moderate+0x5c>
ffffc0209a48: 13027c21 	asr	w1, w1, #2
ffffc0209a4c: 531d04c6 	ubfiz	w6, w6, #3, #2
ffffc0209a50: 91001021 	add	x1, x1, #4
ffffc0209a54: b8617800 	ldr	w0, [x0, x1, lsl #2]
ffffc0209a58: 1ac62406 	lsr	w6, w0, w6
ffffc0209a5c: b9000046 	str	w6, [x2]
ffffc0209a60: 17ffffee 	b	0xffffc0209a18 <pci_cfg_read_moderate+0x10>
ffffc0209a64: 5100c0c1 	sub	w1, w6, #48
ffffc0209a68: 12003c21 	and	w1, w1, #0xffff
ffffc0209a6c: 71000c3f 	cmp	w1, #3
ffffc0209a70: 54000068 	b.hi	0xffffc0209a7c <pci_cfg_read_moderate+0x74>
ffffc0209a74: b900005f 	str	wzr, [x2]
ffffc0209a78: 17ffffe8 	b	0xffffc0209a18 <pci_cfg_read_moderate+0x10>
ffffc0209a7c: 71000c7f 	cmp	w3, #3
ffffc0209a80: 54000061 	b.ne	0xffffc0209a8c <pci_cfg_read_moderate+0x84>
ffffc0209a84: 2a0603e1 	mov	w1, w6
ffffc0209a88: 14000454 	b	0xffffc020abd8 <ivshmem_pci_cfg_read>
ffffc0209a8c: 7100fcdf 	cmp	w6, #63
ffffc0209a90: 540003c9 	b.ls	0xffffc0209b08 <pci_cfg_read_moderate+0x100>
ffffc0209a94: aa0203e8 	mov	x8, x2
ffffc0209a98: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc0209a9c: 2a0603e1 	mov	w1, w6
ffffc0209aa0: aa0703e0 	mov	x0, x7
ffffc0209aa4: 910003fd 	mov	x29, sp
ffffc0209aa8: 97fffedf 	bl	0xffffc0209624 <pci_find_capability>
ffffc0209aac: b4000320 	cbz	x0, 0xffffc0209b10 <pci_cfg_read_moderate+0x108>
ffffc0209ab0: 79400001 	ldrh	w1, [x0]
ffffc0209ab4: 7100143f 	cmp	w1, #5
ffffc0209ab8: 540002c1 	b.ne	0xffffc0209b10 <pci_cfg_read_moderate+0x108>
ffffc0209abc: 79400400 	ldrh	w0, [x0, #2]
ffffc0209ac0: 4b0000c6 	sub	w6, w6, w0
ffffc0209ac4: 71000cdf 	cmp	w6, #3
ffffc0209ac8: 54000249 	b.ls	0xffffc0209b10 <pci_cfg_read_moderate+0x108>
ffffc0209acc: 710024df 	cmp	w6, #9
ffffc0209ad0: 540000a9 	b.ls	0xffffc0209ae4 <pci_cfg_read_moderate+0xdc>
ffffc0209ad4: 39408d20 	ldrb	w0, [x9, #35]
ffffc0209ad8: 340001c0 	cbz	w0, 0xffffc0209b10 <pci_cfg_read_moderate+0x108>
ffffc0209adc: 710034df 	cmp	w6, #13
ffffc0209ae0: 54000188 	b.hi	0xffffc0209b10 <pci_cfg_read_moderate+0x108>
ffffc0209ae4: 927e74c4 	and	x4, x6, #0xfffffffc
ffffc0209ae8: 531d04c6 	ubfiz	w6, w6, #3, #2
ffffc0209aec: 8b0400e4 	add	x4, x7, x4
ffffc0209af0: b9402880 	ldr	w0, [x4, #40]
ffffc0209af4: 1ac62406 	lsr	w6, w0, w6
ffffc0209af8: 52800040 	mov	w0, #2
ffffc0209afc: b9000106 	str	w6, [x8]
ffffc0209b00: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc0209b04: d65f03c0 	ret
ffffc0209b08: 52800020 	mov	w0, #1
ffffc0209b0c: d65f03c0 	ret
ffffc0209b10: 52800020 	mov	w0, #1
ffffc0209b14: 17fffffb 	b	0xffffc0209b00 <pci_cfg_read_moderate+0xf8>

0000ffffc0209b18 <pci_cfg_write_moderate>:
ffffc0209b18: b50000e0 	cbnz	x0, 0xffffc0209b34 <pci_cfg_write_moderate+0x1c>
ffffc0209b1c: 52800000 	mov	w0, #0
ffffc0209b20: d65f03c0 	ret
ffffc0209b24: 52800000 	mov	w0, #0
ffffc0209b28: f9400bf3 	ldr	x19, [sp, #16]
ffffc0209b2c: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc0209b30: d65f03c0 	ret
ffffc0209b34: 128000e9 	mov	w9, #-8
ffffc0209b38: 12003c28 	and	w8, w1, #0xffff
ffffc0209b3c: f940000a 	ldr	x10, [x0]
ffffc0209b40: aa0003e7 	mov	x7, x0
ffffc0209b44: 1b097c42 	mul	w2, w2, w9
ffffc0209b48: 531d0506 	ubfiz	w6, w8, #3, #2
ffffc0209b4c: 92800009 	mov	x9, #-1
ffffc0209b50: 11010042 	add	w2, w2, #64
ffffc0209b54: 9ac22522 	lsr	x2, x9, x2
ffffc0209b58: 9ac62049 	lsl	x9, x2, x6
ffffc0209b5c: 1ac62066 	lsl	w6, w3, w6
ffffc0209b60: 39400143 	ldrb	w3, [x10]
ffffc0209b64: 7100087f 	cmp	w3, #2
ffffc0209b68: 54000a00 	b.eq	0xffffc0209ca8 <pci_cfg_write_moderate+0x190>
ffffc0209b6c: 51004101 	sub	w1, w8, #16
ffffc0209b70: 12003c22 	and	w2, w1, #0xffff
ffffc0209b74: 71005c5f 	cmp	w2, #23
ffffc0209b78: 54000bc8 	b.hi	0xffffc0209cf0 <pci_cfg_write_moderate+0x1d8>
ffffc0209b7c: 52800080 	mov	w0, #4
ffffc0209b80: 1ac00c21 	sdiv	w1, w1, w0
ffffc0209b84: d37ef421 	lsl	x1, x1, #2
ffffc0209b88: 8b0100e7 	add	x7, x7, x1
ffffc0209b8c: 8b010141 	add	x1, x10, x1
ffffc0209b90: b94010e0 	ldr	w0, [x7, #16]
ffffc0209b94: 4a060006 	eor	w6, w0, w6
ffffc0209b98: 0a0900c2 	and	w2, w6, w9
ffffc0209b9c: b8406026 	ldur	w6, [x1, #6]
ffffc0209ba0: 0a060042 	and	w2, w2, w6
ffffc0209ba4: 4a000042 	eor	w2, w2, w0
ffffc0209ba8: 52800040 	mov	w0, #2
ffffc0209bac: b90010e2 	str	w2, [x7, #16]
ffffc0209bb0: d65f03c0 	ret
ffffc0209bb4: d3423d01 	ubfx	x1, x8, #2, #14
ffffc0209bb8: d0000000 	adrp	x0, 0xffffc020b000 <pci_cfg_write_moderate+0xa8>
ffffc0209bbc: 911c6000 	add	x0, x0, #1816
ffffc0209bc0: 91080000 	add	x0, x0, #512
ffffc0209bc4: d37df021 	lsl	x1, x1, #3
ffffc0209bc8: 1400003e 	b	0xffffc0209cc0 <pci_cfg_write_moderate+0x1a8>
ffffc0209bcc: 71000c7f 	cmp	w3, #3
ffffc0209bd0: 54000a01 	b.ne	0xffffc0209d10 <pci_cfg_write_moderate+0x1f8>
ffffc0209bd4: 2a0603e3 	mov	w3, w6
ffffc0209bd8: 53027d01 	lsr	w1, w8, #2
ffffc0209bdc: aa0703e0 	mov	x0, x7
ffffc0209be0: 140003aa 	b	0xffffc020aa88 <ivshmem_pci_cfg_write>
ffffc0209be4: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc0209be8: 2a0803e1 	mov	w1, w8
ffffc0209bec: aa0703e0 	mov	x0, x7
ffffc0209bf0: 910003fd 	mov	x29, sp
ffffc0209bf4: f9000bf3 	str	x19, [sp, #16]
ffffc0209bf8: 97fffe8b 	bl	0xffffc0209624 <pci_find_capability>
ffffc0209bfc: b4fff940 	cbz	x0, 0xffffc0209b24 <pci_cfg_write_moderate+0xc>
ffffc0209c00: 79400c01 	ldrh	w1, [x0, #6]
ffffc0209c04: 3607f901 	tbz	w1, #0, 0xffffc0209b24 <pci_cfg_write_moderate+0xc>
ffffc0209c08: 79400413 	ldrh	w19, [x0, #2]
ffffc0209c0c: 79400001 	ldrh	w1, [x0]
ffffc0209c10: 4b130113 	sub	w19, w8, w19
ffffc0209c14: 7100143f 	cmp	w1, #5
ffffc0209c18: 540002a1 	b.ne	0xffffc0209c6c <pci_cfg_write_moderate+0x154>
ffffc0209c1c: 7100267f 	cmp	w19, #9
ffffc0209c20: 540000a9 	b.ls	0xffffc0209c34 <pci_cfg_write_moderate+0x11c>
ffffc0209c24: 39408d41 	ldrb	w1, [x10, #35]
ffffc0209c28: 340001e1 	cbz	w1, 0xffffc0209c64 <pci_cfg_write_moderate+0x14c>
ffffc0209c2c: 7100367f 	cmp	w19, #13
ffffc0209c30: 540001a8 	b.hi	0xffffc0209c64 <pci_cfg_write_moderate+0x14c>
ffffc0209c34: 927e7661 	and	x1, x19, #0xfffffffc
ffffc0209c38: 8b0100e1 	add	x1, x7, x1
ffffc0209c3c: b9402822 	ldr	w2, [x1, #40]
ffffc0209c40: 0a290042 	bic	w2, w2, w9
ffffc0209c44: 2a060046 	orr	w6, w2, w6
ffffc0209c48: b9002826 	str	w6, [x1, #40]
ffffc0209c4c: aa0003e1 	mov	x1, x0
ffffc0209c50: aa0703e0 	mov	x0, x7
ffffc0209c54: 97ffe5a5 	bl	0xffffc02032e8 <arch_pci_update_msi>
ffffc0209c58: 37fff660 	tbnz	w0, #31, 0xffffc0209b24 <pci_cfg_write_moderate+0xc>
ffffc0209c5c: 71000e7f 	cmp	w19, #3
ffffc0209c60: 54000208 	b.hi	0xffffc0209ca0 <pci_cfg_write_moderate+0x188>
ffffc0209c64: 52800020 	mov	w0, #1
ffffc0209c68: 17ffffb0 	b	0xffffc0209b28 <pci_cfg_write_moderate+0x10>
ffffc0209c6c: 7100443f 	cmp	w1, #17
ffffc0209c70: 54ffffa1 	b.ne	0xffffc0209c64 <pci_cfg_write_moderate+0x14c>
ffffc0209c74: 71000e7f 	cmp	w19, #3
ffffc0209c78: 54ffff68 	b.hi	0xffffc0209c64 <pci_cfg_write_moderate+0x14c>
ffffc0209c7c: b94038e2 	ldr	w2, [x7, #56]
ffffc0209c80: aa0703e0 	mov	x0, x7
ffffc0209c84: 0a290042 	bic	w2, w2, w9
ffffc0209c88: 2a060046 	orr	w6, w2, w6
ffffc0209c8c: b90038e6 	str	w6, [x7, #56]
ffffc0209c90: 97fffe41 	bl	0xffffc0209594 <pci_update_msix.isra.0>
ffffc0209c94: 2a2003e0 	mvn	w0, w0
ffffc0209c98: 531f7c00 	lsr	w0, w0, #31
ffffc0209c9c: 17ffffa3 	b	0xffffc0209b28 <pci_cfg_write_moderate+0x10>
ffffc0209ca0: 52800040 	mov	w0, #2
ffffc0209ca4: 17ffffa1 	b	0xffffc0209b28 <pci_cfg_write_moderate+0x10>
ffffc0209ca8: 7100fd1f 	cmp	w8, #63
ffffc0209cac: 54fff9c8 	b.hi	0xffffc0209be4 <pci_cfg_write_moderate+0xcc>
ffffc0209cb0: d3423d00 	ubfx	x0, x8, #2, #14
ffffc0209cb4: d0000001 	adrp	x1, 0xffffc020b000 <pci_cfg_write_moderate+0x1a4>
ffffc0209cb8: 911c6021 	add	x1, x1, #1816
ffffc0209cbc: d37df000 	lsl	x0, x0, #3
ffffc0209cc0: 8b010002 	add	x2, x0, x1
ffffc0209cc4: b8616800 	ldr	w0, [x0, x1]
ffffc0209cc8: b9400442 	ldr	w2, [x2, #4]
ffffc0209ccc: 0a090042 	and	w2, w2, w9
ffffc0209cd0: 6b09005f 	cmp	w2, w9
ffffc0209cd4: 54fff241 	b.ne	0xffffc0209b1c <pci_cfg_write_moderate+0x4>
ffffc0209cd8: 7100041f 	cmp	w0, #1
ffffc0209cdc: 54fff780 	b.eq	0xffffc0209bcc <pci_cfg_write_moderate+0xb4>
ffffc0209ce0: 7100081f 	cmp	w0, #2
ffffc0209ce4: 1a9f17e0 	cset	w0, eq
ffffc0209ce8: 531f7800 	lsl	w0, w0, #1
ffffc0209cec: d65f03c0 	ret
ffffc0209cf0: 7100fd1f 	cmp	w8, #63
ffffc0209cf4: 54fff609 	b.ls	0xffffc0209bb4 <pci_cfg_write_moderate+0x9c>
ffffc0209cf8: 71000c7f 	cmp	w3, #3
ffffc0209cfc: 54fff741 	b.ne	0xffffc0209be4 <pci_cfg_write_moderate+0xcc>
ffffc0209d00: 2a0603e3 	mov	w3, w6
ffffc0209d04: 2a0903e2 	mov	w2, w9
ffffc0209d08: 53027d01 	lsr	w1, w8, #2
ffffc0209d0c: 17ffffb5 	b	0xffffc0209be0 <pci_cfg_write_moderate+0xc8>
ffffc0209d10: 52800020 	mov	w0, #1
ffffc0209d14: d65f03c0 	ret

0000ffffc0209d18 <pci_mmconfig_access_handler>:
ffffc0209d18: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0209d1c: 910003fd 	mov	x29, sp
ffffc0209d20: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0209d24: aa0103f4 	mov	x20, x1
ffffc0209d28: f90013f5 	str	x21, [sp, #32]
ffffc0209d2c: f9400033 	ldr	x19, [x1]
ffffc0209d30: b9400828 	ldr	w8, [x1, #8]
ffffc0209d34: 12002e75 	and	w21, w19, #0xfff
ffffc0209d38: 7100111f 	cmp	w8, #4
ffffc0209d3c: 530c6e73 	ubfx	w19, w19, #12, #16
ffffc0209d40: 540001c8 	b.hi	0xffffc0209d78 <pci_mmconfig_access_handler+0x60>
ffffc0209d44: d2840000 	mov	x0, #8192
ffffc0209d48: 2a1303e1 	mov	w1, w19
ffffc0209d4c: f2a00020 	movk	x0, #1, lsl #16
ffffc0209d50: f2dfe000 	movk	x0, #65280, lsl #32
ffffc0209d54: f9480400 	ldr	x0, [x0, #4104]
ffffc0209d58: 97ffff0f 	bl	0xffffc0209994 <pci_get_assigned_device>
ffffc0209d5c: b9400e82 	ldr	w2, [x20, #12]
ffffc0209d60: 12003ea1 	and	w1, w21, #0xffff
ffffc0209d64: 340001e2 	cbz	w2, 0xffffc0209da0 <pci_mmconfig_access_handler+0x88>
ffffc0209d68: b9401283 	ldr	w3, [x20, #16]
ffffc0209d6c: 2a0803e2 	mov	w2, w8
ffffc0209d70: 97ffff6a 	bl	0xffffc0209b18 <pci_cfg_write_moderate>
ffffc0209d74: 35000380 	cbnz	w0, 0xffffc0209de4 <pci_mmconfig_access_handler+0xcc>
ffffc0209d78: b9400a85 	ldr	w5, [x20, #8]
ffffc0209d7c: 2a1503e4 	mov	w4, w21
ffffc0209d80: 12000a63 	and	w3, w19, #0x7
ffffc0209d84: d3431e62 	ubfx	x2, x19, #3, #5
ffffc0209d88: 53087e61 	lsr	w1, w19, #8
ffffc0209d8c: f0000000 	adrp	x0, 0xffffc020c000 <pci_mmconfig_access_handler+0x80>
ffffc0209d90: 9102f800 	add	x0, x0, #190
ffffc0209d94: 97fff397 	bl	0xffffc0206bf0 <panic_printk>
ffffc0209d98: 12800000 	mov	w0, #-1
ffffc0209d9c: 1400000b 	b	0xffffc0209dc8 <pci_mmconfig_access_handler+0xb0>
ffffc0209da0: 9100f3a3 	add	x3, x29, #60
ffffc0209da4: 2a0803e2 	mov	w2, w8
ffffc0209da8: 97ffff18 	bl	0xffffc0209a08 <pci_cfg_read_moderate>
ffffc0209dac: 7100041f 	cmp	w0, #1
ffffc0209db0: 54000141 	b.ne	0xffffc0209dd8 <pci_mmconfig_access_handler+0xc0>
ffffc0209db4: f0000080 	adrp	x0, 0xffffc021c000 <pci_enabled_msi_vectors+0x10>
ffffc0209db8: aa1403e1 	mov	x1, x20
ffffc0209dbc: f9400000 	ldr	x0, [x0]
ffffc0209dc0: 97fffd45 	bl	0xffffc02092d4 <mmio_perform_access>
ffffc0209dc4: 52800020 	mov	w0, #1
ffffc0209dc8: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0209dcc: f94013f5 	ldr	x21, [sp, #32]
ffffc0209dd0: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0209dd4: d65f03c0 	ret
ffffc0209dd8: b9403fa0 	ldr	w0, [x29, #60]
ffffc0209ddc: f9000a80 	str	x0, [x20, #16]
ffffc0209de0: 17fffff9 	b	0xffffc0209dc4 <pci_mmconfig_access_handler+0xac>
ffffc0209de4: 7100041f 	cmp	w0, #1
ffffc0209de8: 54fffee1 	b.ne	0xffffc0209dc4 <pci_mmconfig_access_handler+0xac>
ffffc0209dec: 17fffff2 	b	0xffffc0209db4 <pci_mmconfig_access_handler+0x9c>

0000ffffc0209df0 <pci_enabled_msi_vectors>:
ffffc0209df0: 3940a800 	ldrb	w0, [x0, #42]
ffffc0209df4: 360000a0 	tbz	w0, #0, 0xffffc0209e08 <pci_enabled_msi_vectors+0x18>
ffffc0209df8: d3441801 	ubfx	x1, x0, #4, #3
ffffc0209dfc: 52800020 	mov	w0, #1
ffffc0209e00: 1ac12000 	lsl	w0, w0, w1
ffffc0209e04: d65f03c0 	ret
ffffc0209e08: 52800000 	mov	w0, #0
ffffc0209e0c: 17fffffe 	b	0xffffc0209e04 <pci_enabled_msi_vectors+0x14>

0000ffffc0209e10 <pci_prepare_handover>:
ffffc0209e10: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc0209e14: 910003fd 	mov	x29, sp
ffffc0209e18: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0209e1c: b0000094 	adrp	x20, 0xffffc021a000 <pci_prepare_handover+0x50>
ffffc0209e20: 91000294 	add	x20, x20, #0
ffffc0209e24: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc0209e28: f9001bf7 	str	x23, [sp, #48]
ffffc0209e2c: f9486a93 	ldr	x19, [x20, #4304]
ffffc0209e30: b40001b3 	cbz	x19, 0xffffc0209e64 <pci_prepare_handover+0x54>
ffffc0209e34: d297d077 	mov	x23, #48771
ffffc0209e38: f2a5f417 	movk	x23, #12192, lsl #16
ffffc0209e3c: f2c17d17 	movk	x23, #3048, lsl #32
ffffc0209e40: f2f05f57 	movk	x23, #33530, lsl #48
ffffc0209e44: f9486a80 	ldr	x0, [x20, #4304]
ffffc0209e48: f9485281 	ldr	x1, [x20, #4256]
ffffc0209e4c: cb000260 	sub	x0, x19, x0
ffffc0209e50: 9343fc00 	asr	x0, x0, #3
ffffc0209e54: b9404421 	ldr	w1, [x1, #68]
ffffc0209e58: 9b177c00 	mul	x0, x0, x23
ffffc0209e5c: eb01001f 	cmp	x0, x1
ffffc0209e60: 540000cb 	b.lt	0xffffc0209e78 <pci_prepare_handover+0x68>
ffffc0209e64: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0209e68: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc0209e6c: f9401bf7 	ldr	x23, [sp, #48]
ffffc0209e70: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc0209e74: d65f03c0 	ret
ffffc0209e78: f9400660 	ldr	x0, [x19, #8]
ffffc0209e7c: b5000060 	cbnz	x0, 0xffffc0209e88 <pci_prepare_handover+0x78>
ffffc0209e80: 91056273 	add	x19, x19, #344
ffffc0209e84: 17fffff0 	b	0xffffc0209e44 <pci_prepare_handover+0x34>
ffffc0209e88: f9485000 	ldr	x0, [x0, #4256]
ffffc0209e8c: 52800016 	mov	w22, #0
ffffc0209e90: 97fffdd4 	bl	0xffffc02095e0 <jailhouse_cell_pci_caps>
ffffc0209e94: f9400261 	ldr	x1, [x19]
ffffc0209e98: 79403c35 	ldrh	w21, [x1, #30]
ffffc0209e9c: 8b150c15 	add	x21, x0, x21, lsl #3
ffffc0209ea0: f9400260 	ldr	x0, [x19]
ffffc0209ea4: 79404000 	ldrh	w0, [x0, #32]
ffffc0209ea8: 6b16001f 	cmp	w0, w22
ffffc0209eac: 54fffea9 	b.ls	0xffffc0209e80 <pci_prepare_handover+0x70>
ffffc0209eb0: 794002a0 	ldrh	w0, [x21]
ffffc0209eb4: 7100141f 	cmp	w0, #5
ffffc0209eb8: 540000e1 	b.ne	0xffffc0209ed4 <pci_prepare_handover+0xc4>
ffffc0209ebc: aa1503e1 	mov	x1, x21
ffffc0209ec0: aa1303e0 	mov	x0, x19
ffffc0209ec4: 97ffe508 	bl	0xffffc02032e4 <arch_pci_suppress_msi>
ffffc0209ec8: 910022b5 	add	x21, x21, #8
ffffc0209ecc: 110006d6 	add	w22, w22, #1
ffffc0209ed0: 17fffff4 	b	0xffffc0209ea0 <pci_prepare_handover+0x90>
ffffc0209ed4: 7100441f 	cmp	w0, #17
ffffc0209ed8: 54ffff81 	b.ne	0xffffc0209ec8 <pci_prepare_handover+0xb8>
ffffc0209edc: 52800022 	mov	w2, #1
ffffc0209ee0: aa1503e1 	mov	x1, x21
ffffc0209ee4: aa1303e0 	mov	x0, x19
ffffc0209ee8: 97fffe42 	bl	0xffffc02097f0 <pci_suppress_msix>
ffffc0209eec: 17fffff7 	b	0xffffc0209ec8 <pci_prepare_handover+0xb8>

0000ffffc0209ef0 <pci_reset_device>:
ffffc0209ef0: a902fc1f 	stp	xzr, xzr, [x0, #40]
ffffc0209ef4: d2800001 	mov	x1, #0
ffffc0209ef8: f9400004 	ldr	x4, [x0]
ffffc0209efc: 79404882 	ldrh	w2, [x4, #36]
ffffc0209f00: 6b01005f 	cmp	w2, w1
ffffc0209f04: 540000a8 	b.hi	0xffffc0209f18 <pci_reset_device+0x28>
ffffc0209f08: 39400081 	ldrb	w1, [x4]
ffffc0209f0c: 71000c3f 	cmp	w1, #3
ffffc0209f10: 540001e1 	b.ne	0xffffc0209f4c <pci_reset_device+0x5c>
ffffc0209f14: 140003b3 	b	0xffffc020ade0 <ivshmem_reset>
ffffc0209f18: d37cec23 	lsl	x3, x1, #4
ffffc0209f1c: f9402802 	ldr	x2, [x0, #80]
ffffc0209f20: 91000421 	add	x1, x1, #1
ffffc0209f24: f823685f 	str	xzr, [x2, x3]
ffffc0209f28: f9402802 	ldr	x2, [x0, #80]
ffffc0209f2c: 8b030042 	add	x2, x2, x3
ffffc0209f30: b900085f 	str	wzr, [x2, #8]
ffffc0209f34: f9402802 	ldr	x2, [x0, #80]
ffffc0209f38: 8b030042 	add	x2, x2, x3
ffffc0209f3c: 39403043 	ldrb	w3, [x2, #12]
ffffc0209f40: 32000063 	orr	w3, w3, #0x1
ffffc0209f44: 39003043 	strb	w3, [x2, #12]
ffffc0209f48: 17ffffec 	b	0xffffc0209ef8 <pci_reset_device+0x8>
ffffc0209f4c: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc0209f50: 52800043 	mov	w3, #2
ffffc0209f54: 52800081 	mov	w1, #4
ffffc0209f58: 2a0303e2 	mov	w2, w3
ffffc0209f5c: 910003fd 	mov	x29, sp
ffffc0209f60: a90153f3 	stp	x19, x20, [sp, #16]
ffffc0209f64: aa0003f3 	mov	x19, x0
ffffc0209f68: f90013f5 	str	x21, [sp, #32]
ffffc0209f6c: 52800015 	mov	w21, #0
ffffc0209f70: 79400880 	ldrh	w0, [x4, #4]
ffffc0209f74: 97fffe07 	bl	0xffffc0209790 <pci_write_config>
ffffc0209f78: f9400660 	ldr	x0, [x19, #8]
ffffc0209f7c: f9485000 	ldr	x0, [x0, #4256]
ffffc0209f80: 97fffd98 	bl	0xffffc02095e0 <jailhouse_cell_pci_caps>
ffffc0209f84: f9400261 	ldr	x1, [x19]
ffffc0209f88: 79403c34 	ldrh	w20, [x1, #30]
ffffc0209f8c: 8b140c14 	add	x20, x0, x20, lsl #3
ffffc0209f90: f9400260 	ldr	x0, [x19]
ffffc0209f94: 79404001 	ldrh	w1, [x0, #32]
ffffc0209f98: 6b15003f 	cmp	w1, w21
ffffc0209f9c: 54000128 	b.hi	0xffffc0209fc0 <pci_reset_device+0xd0>
ffffc0209fa0: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc0209fa4: 52800043 	mov	w3, #2
ffffc0209fa8: f94013f5 	ldr	x21, [sp, #32]
ffffc0209fac: 52808002 	mov	w2, #1024
ffffc0209fb0: 79400800 	ldrh	w0, [x0, #4]
ffffc0209fb4: 52800081 	mov	w1, #4
ffffc0209fb8: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc0209fbc: 17fffdf5 	b	0xffffc0209790 <pci_write_config>
ffffc0209fc0: 79400281 	ldrh	w1, [x20]
ffffc0209fc4: 7100143f 	cmp	w1, #5
ffffc0209fc8: 7a511824 	ccmp	w1, #17, #4, ne
ffffc0209fcc: 540000e1 	b.ne	0xffffc0209fe8 <pci_reset_device+0xf8>
ffffc0209fd0: 79400800 	ldrh	w0, [x0, #4]
ffffc0209fd4: 52800043 	mov	w3, #2
ffffc0209fd8: 79400681 	ldrh	w1, [x20, #2]
ffffc0209fdc: 52800002 	mov	w2, #0
ffffc0209fe0: 11000821 	add	w1, w1, #2
ffffc0209fe4: 97fffdeb 	bl	0xffffc0209790 <pci_write_config>
ffffc0209fe8: 79400280 	ldrh	w0, [x20]
ffffc0209fec: 7100441f 	cmp	w0, #17
ffffc0209ff0: 540001e1 	b.ne	0xffffc020a02c <pci_reset_device+0x13c>
ffffc0209ff4: d2800000 	mov	x0, #0
ffffc0209ff8: 14000008 	b	0xffffc020a018 <pci_reset_device+0x128>
ffffc0209ffc: a9448a61 	ldp	x1, x2, [x19, #72]
ffffc020a000: d37cec03 	lsl	x3, x0, #4
ffffc020a004: 91000400 	add	x0, x0, #1
ffffc020a008: 8b030042 	add	x2, x2, x3
ffffc020a00c: 8b030021 	add	x1, x1, x3
ffffc020a010: b9400c42 	ldr	w2, [x2, #12]
ffffc020a014: b9000c22 	str	w2, [x1, #12]
ffffc020a018: f9400261 	ldr	x1, [x19]
ffffc020a01c: 2a0003f5 	mov	w21, w0
ffffc020a020: 79404821 	ldrh	w1, [x1, #36]
ffffc020a024: 6b00003f 	cmp	w1, w0
ffffc020a028: 54fffea8 	b.hi	0xffffc0209ffc <pci_reset_device+0x10c>
ffffc020a02c: 91002294 	add	x20, x20, #8
ffffc020a030: 110006b5 	add	w21, w21, #1
ffffc020a034: 17ffffd7 	b	0xffffc0209f90 <pci_reset_device+0xa0>

0000ffffc020a038 <pci_remove_physical_device>:
ffffc020a038: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc020a03c: 910003fd 	mov	x29, sp
ffffc020a040: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020a044: aa0003f3 	mov	x19, x0
ffffc020a048: f9400414 	ldr	x20, [x0, #8]
ffffc020a04c: f9400000 	ldr	x0, [x0]
ffffc020a050: f9485284 	ldr	x4, [x20, #4256]
ffffc020a054: 79400801 	ldrh	w1, [x0, #4]
ffffc020a058: d0000000 	adrp	x0, 0xffffc020c000 <pci_remove_physical_device+0x28>
ffffc020a05c: 91002084 	add	x4, x4, #8
ffffc020a060: 91042400 	add	x0, x0, #265
ffffc020a064: 12000823 	and	w3, w1, #0x7
ffffc020a068: d3431c22 	ubfx	x2, x1, #3, #5
ffffc020a06c: 53087c21 	lsr	w1, w1, #8
ffffc020a070: 97fff2ad 	bl	0xffffc0206b24 <printk>
ffffc020a074: aa1303e0 	mov	x0, x19
ffffc020a078: 97ffff9e 	bl	0xffffc0209ef0 <pci_reset_device>
ffffc020a07c: aa1303e0 	mov	x0, x19
ffffc020a080: 97ffe498 	bl	0xffffc02032e0 <arch_pci_remove_physical_device>
ffffc020a084: f9402661 	ldr	x1, [x19, #72]
ffffc020a088: f900067f 	str	xzr, [x19, #8]
ffffc020a08c: b40002e1 	cbz	x1, 0xffffc020a0e8 <pci_remove_physical_device+0xb0>
ffffc020a090: f9400260 	ldr	x0, [x19]
ffffc020a094: 79404c02 	ldrh	w2, [x0, #38]
ffffc020a098: f9401400 	ldr	x0, [x0, #40]
ffffc020a09c: 97fff500 	bl	0xffffc020749c <paging_unmap_device>
ffffc020a0a0: f9402a61 	ldr	x1, [x19, #80]
ffffc020a0a4: 91016260 	add	x0, x19, #88
ffffc020a0a8: eb00003f 	cmp	x1, x0
ffffc020a0ac: 54000120 	b.eq	0xffffc020a0d0 <pci_remove_physical_device+0x98>
ffffc020a0b0: f9400260 	ldr	x0, [x19]
ffffc020a0b4: 79404802 	ldrh	w2, [x0, #36]
ffffc020a0b8: d0000060 	adrp	x0, 0xffffc0218000 <pci_remove_physical_device+0xb8>
ffffc020a0bc: 91004000 	add	x0, x0, #16
ffffc020a0c0: d37cec42 	lsl	x2, x2, #4
ffffc020a0c4: 913ffc42 	add	x2, x2, #4095
ffffc020a0c8: d34cac42 	ubfx	x2, x2, #12, #32
ffffc020a0cc: 97fff36f 	bl	0xffffc0206e88 <page_free>
ffffc020a0d0: f9400260 	ldr	x0, [x19]
ffffc020a0d4: f9401401 	ldr	x1, [x0, #40]
ffffc020a0d8: aa1403e0 	mov	x0, x20
ffffc020a0dc: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a0e0: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020a0e4: 17fffc29 	b	0xffffc0209188 <mmio_region_unregister>
ffffc020a0e8: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a0ec: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020a0f0: d65f03c0 	ret

0000ffffc020a0f4 <pci_add_physical_device>:
ffffc020a0f4: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc020a0f8: 910003fd 	mov	x29, sp
ffffc020a0fc: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020a100: aa0003f4 	mov	x20, x0
ffffc020a104: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020a108: aa0103f3 	mov	x19, x1
ffffc020a10c: d2800016 	mov	x22, #0
ffffc020a110: f9400020 	ldr	x0, [x1]
ffffc020a114: f9485284 	ldr	x4, [x20, #4256]
ffffc020a118: 79400801 	ldrh	w1, [x0, #4]
ffffc020a11c: 91002084 	add	x4, x4, #8
ffffc020a120: 79404c15 	ldrh	w21, [x0, #38]
ffffc020a124: d0000000 	adrp	x0, 0xffffc020c000 <pci_add_physical_device+0x38>
ffffc020a128: 12000823 	and	w3, w1, #0x7
ffffc020a12c: 9104e800 	add	x0, x0, #314
ffffc020a130: d3431c22 	ubfx	x2, x1, #3, #5
ffffc020a134: 53087c21 	lsr	w1, w1, #8
ffffc020a138: 97fff27b 	bl	0xffffc0206b24 <printk>
ffffc020a13c: f9400260 	ldr	x0, [x19]
ffffc020a140: 110042c1 	add	w1, w22, #16
ffffc020a144: 52800082 	mov	w2, #4
ffffc020a148: 79400800 	ldrh	w0, [x0, #4]
ffffc020a14c: 97fffd79 	bl	0xffffc0209730 <pci_read_config>
ffffc020a150: 8b160261 	add	x1, x19, x22
ffffc020a154: 910012d6 	add	x22, x22, #4
ffffc020a158: f10062df 	cmp	x22, #24
ffffc020a15c: b9001020 	str	w0, [x1, #16]
ffffc020a160: 54fffee1 	b.ne	0xffffc020a13c <pci_add_physical_device+0x48>
ffffc020a164: aa1303e1 	mov	x1, x19
ffffc020a168: aa1403e0 	mov	x0, x20
ffffc020a16c: 97ffe45b 	bl	0xffffc02032d8 <arch_pci_add_physical_device>
ffffc020a170: 2a0003f6 	mov	w22, w0
ffffc020a174: 350005a0 	cbnz	w0, 0xffffc020a228 <pci_add_physical_device+0x134>
ffffc020a178: f9400260 	ldr	x0, [x19]
ffffc020a17c: f9401400 	ldr	x0, [x0, #40]
ffffc020a180: b4000460 	cbz	x0, 0xffffc020a20c <pci_add_physical_device+0x118>
ffffc020a184: 92403eb5 	and	x21, x21, #0xffff
ffffc020a188: aa1503e1 	mov	x1, x21
ffffc020a18c: 97fff4a1 	bl	0xffffc0207410 <paging_map_device>
ffffc020a190: f9002660 	str	x0, [x19, #72]
ffffc020a194: b4000240 	cbz	x0, 0xffffc020a1dc <pci_add_physical_device+0xe8>
ffffc020a198: f9400260 	ldr	x0, [x19]
ffffc020a19c: 79404801 	ldrh	w1, [x0, #36]
ffffc020a1a0: 7100403f 	cmp	w1, #16
ffffc020a1a4: 54000249 	b.ls	0xffffc020a1ec <pci_add_physical_device+0xf8>
ffffc020a1a8: d37c3c21 	ubfiz	x1, x1, #4, #16
ffffc020a1ac: d0000060 	adrp	x0, 0xffffc0218000 <pci_add_physical_device+0xf0>
ffffc020a1b0: 913ffc21 	add	x1, x1, #4095
ffffc020a1b4: 91004000 	add	x0, x0, #16
ffffc020a1b8: d34cac21 	ubfx	x1, x1, #12, #32
ffffc020a1bc: 97fff32f 	bl	0xffffc0206e78 <page_alloc>
ffffc020a1c0: f9002a60 	str	x0, [x19, #80]
ffffc020a1c4: b5000140 	cbnz	x0, 0xffffc020a1ec <pci_add_physical_device+0xf8>
ffffc020a1c8: f9400260 	ldr	x0, [x19]
ffffc020a1cc: aa1503e2 	mov	x2, x21
ffffc020a1d0: f9402661 	ldr	x1, [x19, #72]
ffffc020a1d4: f9401400 	ldr	x0, [x0, #40]
ffffc020a1d8: 97fff4b1 	bl	0xffffc020749c <paging_unmap_device>
ffffc020a1dc: aa1303e0 	mov	x0, x19
ffffc020a1e0: 12800176 	mov	w22, #-12
ffffc020a1e4: 97ffe43f 	bl	0xffffc02032e0 <arch_pci_remove_physical_device>
ffffc020a1e8: 14000010 	b	0xffffc020a228 <pci_add_physical_device+0x134>
ffffc020a1ec: f9400260 	ldr	x0, [x19]
ffffc020a1f0: f0ffffe3 	adrp	x3, 0xffffc0209000 <pci_add_physical_device+0xf8>
ffffc020a1f4: aa1303e4 	mov	x4, x19
ffffc020a1f8: 91129063 	add	x3, x3, #1188
ffffc020a1fc: aa1503e2 	mov	x2, x21
ffffc020a200: f9401401 	ldr	x1, [x0, #40]
ffffc020a204: aa1403e0 	mov	x0, x20
ffffc020a208: 97fffb96 	bl	0xffffc0209060 <mmio_region_register>
ffffc020a20c: f9000674 	str	x20, [x19, #8]
ffffc020a210: 90000080 	adrp	x0, 0xffffc021a000 <pci_cell_exit+0x14>
ffffc020a214: 91000000 	add	x0, x0, #0
ffffc020a218: eb00029f 	cmp	x20, x0
ffffc020a21c: 54000060 	b.eq	0xffffc020a228 <pci_add_physical_device+0x134>
ffffc020a220: aa1303e0 	mov	x0, x19
ffffc020a224: 97ffff33 	bl	0xffffc0209ef0 <pci_reset_device>
ffffc020a228: 2a1603e0 	mov	w0, w22
ffffc020a22c: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a230: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020a234: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc020a238: d65f03c0 	ret

0000ffffc020a23c <pci_cell_exit>:
ffffc020a23c: a9bb7bfd 	stp	x29, x30, [sp, #-80]!
ffffc020a240: 910003fd 	mov	x29, sp
ffffc020a244: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020a248: 90000094 	adrp	x20, 0xffffc021a000 <pci_cell_exit+0x4c>
ffffc020a24c: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020a250: 91000294 	add	x20, x20, #0
ffffc020a254: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020a258: eb14001f 	cmp	x0, x20
ffffc020a25c: f90023f9 	str	x25, [sp, #64]
ffffc020a260: 540008c0 	b.eq	0xffffc020a378 <pci_cell_exit+0x13c>
ffffc020a264: aa0003f5 	mov	x21, x0
ffffc020a268: f9485000 	ldr	x0, [x0, #4256]
ffffc020a26c: d297d078 	mov	x24, #48771
ffffc020a270: d0000017 	adrp	x23, 0xffffc020c000 <pci_cell_exit+0x3c>
ffffc020a274: f2a5f418 	movk	x24, #12192, lsl #16
ffffc020a278: 91059ef7 	add	x23, x23, #359
ffffc020a27c: f2c17d18 	movk	x24, #3048, lsl #32
ffffc020a280: f9486ab3 	ldr	x19, [x21, #4304]
ffffc020a284: b9404416 	ldr	w22, [x0, #68]
ffffc020a288: f2f05f58 	movk	x24, #33530, lsl #48
ffffc020a28c: aa1803f9 	mov	x25, x24
ffffc020a290: f9486aa1 	ldr	x1, [x21, #4304]
ffffc020a294: f94852a2 	ldr	x2, [x21, #4256]
ffffc020a298: cb010260 	sub	x0, x19, x1
ffffc020a29c: 9343fc00 	asr	x0, x0, #3
ffffc020a2a0: b9404442 	ldr	w2, [x2, #68]
ffffc020a2a4: 9b187c00 	mul	x0, x0, x24
ffffc020a2a8: eb02001f 	cmp	x0, x2
ffffc020a2ac: 540001ab 	b.lt	0xffffc020a2e0 <pci_cell_exit+0xa4>
ffffc020a2b0: 52802b00 	mov	w0, #344
ffffc020a2b4: d281ffe2 	mov	x2, #4095
ffffc020a2b8: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a2bc: 9ba00ac2 	umaddl	x2, w22, w0, x2
ffffc020a2c0: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020a2c4: d0000060 	adrp	x0, 0xffffc0218000 <pci_cell_exit+0xc0>
ffffc020a2c8: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc020a2cc: d34cac42 	ubfx	x2, x2, #12, #32
ffffc020a2d0: f94023f9 	ldr	x25, [sp, #64]
ffffc020a2d4: 91004000 	add	x0, x0, #16
ffffc020a2d8: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc020a2dc: 17fff2eb 	b	0xffffc0206e88 <page_free>
ffffc020a2e0: f9400660 	ldr	x0, [x19, #8]
ffffc020a2e4: b40000e0 	cbz	x0, 0xffffc020a300 <pci_cell_exit+0xc4>
ffffc020a2e8: f9400260 	ldr	x0, [x19]
ffffc020a2ec: 39400000 	ldrb	w0, [x0]
ffffc020a2f0: 71000c1f 	cmp	w0, #3
ffffc020a2f4: aa1303e0 	mov	x0, x19
ffffc020a2f8: 54000081 	b.ne	0xffffc020a308 <pci_cell_exit+0xcc>
ffffc020a2fc: 940002f0 	bl	0xffffc020aebc <ivshmem_exit>
ffffc020a300: 91056273 	add	x19, x19, #344
ffffc020a304: 17ffffe3 	b	0xffffc020a290 <pci_cell_exit+0x54>
ffffc020a308: 97ffff4c 	bl	0xffffc020a038 <pci_remove_physical_device>
ffffc020a30c: f9485280 	ldr	x0, [x20, #4256]
ffffc020a310: f9486a82 	ldr	x2, [x20, #4304]
ffffc020a314: b9404404 	ldr	w4, [x0, #68]
ffffc020a318: aa0203e1 	mov	x1, x2
ffffc020a31c: cb020020 	sub	x0, x1, x2
ffffc020a320: 9343fc00 	asr	x0, x0, #3
ffffc020a324: 9b197c00 	mul	x0, x0, x25
ffffc020a328: eb04001f 	cmp	x0, x4
ffffc020a32c: 54fffeaa 	b.ge	0xffffc020a300 <pci_cell_exit+0xc4>
ffffc020a330: f9400023 	ldr	x3, [x1]
ffffc020a334: f9400260 	ldr	x0, [x19]
ffffc020a338: 79400466 	ldrh	w6, [x3, #2]
ffffc020a33c: 79400405 	ldrh	w5, [x0, #2]
ffffc020a340: 6b0500df 	cmp	w6, w5
ffffc020a344: 54000161 	b.ne	0xffffc020a370 <pci_cell_exit+0x134>
ffffc020a348: 79400863 	ldrh	w3, [x3, #4]
ffffc020a34c: 79400800 	ldrh	w0, [x0, #4]
ffffc020a350: 6b00007f 	cmp	w3, w0
ffffc020a354: 540000e1 	b.ne	0xffffc020a370 <pci_cell_exit+0x134>
ffffc020a358: aa1403e0 	mov	x0, x20
ffffc020a35c: 97ffff66 	bl	0xffffc020a0f4 <pci_add_physical_device>
ffffc020a360: 36fffd00 	tbz	w0, #31, 0xffffc020a300 <pci_cell_exit+0xc4>
ffffc020a364: aa1703e0 	mov	x0, x23
ffffc020a368: 97fff1ef 	bl	0xffffc0206b24 <printk>
ffffc020a36c: 17ffffe5 	b	0xffffc020a300 <pci_cell_exit+0xc4>
ffffc020a370: 91056021 	add	x1, x1, #344
ffffc020a374: 17ffffea 	b	0xffffc020a31c <pci_cell_exit+0xe0>
ffffc020a378: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a37c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020a380: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc020a384: f94023f9 	ldr	x25, [sp, #64]
ffffc020a388: a8c57bfd 	ldp	x29, x30, [sp], #80
ffffc020a38c: d65f03c0 	ret

0000ffffc020a390 <pci_cell_init>:
ffffc020a390: a9b97bfd 	stp	x29, x30, [sp, #-112]!
ffffc020a394: 910003fd 	mov	x29, sp
ffffc020a398: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020a39c: aa0003f4 	mov	x20, x0
ffffc020a3a0: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020a3a4: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020a3a8: a9046bf9 	stp	x25, x26, [sp, #64]
ffffc020a3ac: a90573fb 	stp	x27, x28, [sp, #80]
ffffc020a3b0: f9485000 	ldr	x0, [x0, #4256]
ffffc020a3b4: b9404413 	ldr	w19, [x0, #68]
ffffc020a3b8: 97fffcb5 	bl	0xffffc020968c <jailhouse_cell_pci_devices>
ffffc020a3bc: aa0003fa 	mov	x26, x0
ffffc020a3c0: d0000080 	adrp	x0, 0xffffc021c000 <pci_cell_init+0x78>
ffffc020a3c4: 91000000 	add	x0, x0, #0
ffffc020a3c8: f9400801 	ldr	x1, [x0, #16]
ffffc020a3cc: b40000e1 	cbz	x1, 0xffffc020a3e8 <pci_cell_init+0x58>
ffffc020a3d0: f9400c02 	ldr	x2, [x0, #24]
ffffc020a3d4: f0ffffe3 	adrp	x3, 0xffffc0209000 <pci_cell_init+0x40>
ffffc020a3d8: d2800004 	mov	x4, #0
ffffc020a3dc: 91346063 	add	x3, x3, #3352
ffffc020a3e0: aa1403e0 	mov	x0, x20
ffffc020a3e4: 97fffb1f 	bl	0xffffc0209060 <mmio_region_register>
ffffc020a3e8: f9485280 	ldr	x0, [x20, #4256]
ffffc020a3ec: b9404400 	ldr	w0, [x0, #68]
ffffc020a3f0: 35000140 	cbnz	w0, 0xffffc020a418 <pci_cell_init+0x88>
ffffc020a3f4: 52800015 	mov	w21, #0
ffffc020a3f8: 2a1503e0 	mov	w0, w21
ffffc020a3fc: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a400: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020a404: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc020a408: a9446bf9 	ldp	x25, x26, [sp, #64]
ffffc020a40c: a94573fb 	ldp	x27, x28, [sp, #80]
ffffc020a410: a8c77bfd 	ldp	x29, x30, [sp], #112
ffffc020a414: d65f03c0 	ret
ffffc020a418: 52802b18 	mov	w24, #344
ffffc020a41c: d281ffe1 	mov	x1, #4095
ffffc020a420: d0000060 	adrp	x0, 0xffffc0218000 <pci_cell_init+0xc8>
ffffc020a424: 91004000 	add	x0, x0, #16
ffffc020a428: 9bb80661 	umaddl	x1, w19, w24, x1
ffffc020a42c: d34cac21 	ubfx	x1, x1, #12, #32
ffffc020a430: 97fff292 	bl	0xffffc0206e78 <page_alloc>
ffffc020a434: f9086a80 	str	x0, [x20, #4304]
ffffc020a438: b4000da0 	cbz	x0, 0xffffc020a5ec <pci_cell_init+0x25c>
ffffc020a43c: 90000099 	adrp	x25, 0xffffc021a000 <pci_cell_init+0xec>
ffffc020a440: 91000339 	add	x25, x25, #0
ffffc020a444: 52800016 	mov	w22, #0
ffffc020a448: 5280071b 	mov	w27, #56
ffffc020a44c: f9485280 	ldr	x0, [x20, #4256]
ffffc020a450: b9404400 	ldr	w0, [x0, #68]
ffffc020a454: 6b16001f 	cmp	w0, w22
ffffc020a458: 540000e8 	b.hi	0xffffc020a474 <pci_cell_init+0xe4>
ffffc020a45c: 90000080 	adrp	x0, 0xffffc021a000 <pci_cell_init+0x10c>
ffffc020a460: 91000000 	add	x0, x0, #0
ffffc020a464: eb00029f 	cmp	x20, x0
ffffc020a468: 54fffc61 	b.ne	0xffffc020a3f4 <pci_cell_init+0x64>
ffffc020a46c: 97fffe69 	bl	0xffffc0209e10 <pci_prepare_handover>
ffffc020a470: 17ffffe1 	b	0xffffc020a3f4 <pci_cell_init+0x64>
ffffc020a474: f9486a9c 	ldr	x28, [x20, #4304]
ffffc020a478: 9bb87ed7 	umull	x23, w22, w24
ffffc020a47c: 9bbb7ec1 	umull	x1, w22, w27
ffffc020a480: 8b170393 	add	x19, x28, x23
ffffc020a484: 8b010340 	add	x0, x26, x1
ffffc020a488: 91016262 	add	x2, x19, #88
ffffc020a48c: f8376b80 	str	x0, [x28, x23]
ffffc020a490: f9002a62 	str	x2, [x19, #80]
ffffc020a494: 38616b41 	ldrb	w1, [x26, x1]
ffffc020a498: 71000c3f 	cmp	w1, #3
ffffc020a49c: 54000101 	b.ne	0xffffc020a4bc <pci_cell_init+0x12c>
ffffc020a4a0: aa1303e1 	mov	x1, x19
ffffc020a4a4: aa1403e0 	mov	x0, x20
ffffc020a4a8: 940001da 	bl	0xffffc020ac10 <ivshmem_init>
ffffc020a4ac: 2a0003f5 	mov	w21, w0
ffffc020a4b0: 35000980 	cbnz	w0, 0xffffc020a5e0 <pci_cell_init+0x250>
ffffc020a4b4: 110006d6 	add	w22, w22, #1
ffffc020a4b8: 17ffffe5 	b	0xffffc020a44c <pci_cell_init+0xbc>
ffffc020a4bc: 79400801 	ldrh	w1, [x0, #4]
ffffc020a4c0: aa1903e0 	mov	x0, x25
ffffc020a4c4: 97fffd34 	bl	0xffffc0209994 <pci_get_assigned_device>
ffffc020a4c8: b4000040 	cbz	x0, 0xffffc020a4d0 <pci_cell_init+0x140>
ffffc020a4cc: 97fffedb 	bl	0xffffc020a038 <pci_remove_physical_device>
ffffc020a4d0: aa1303e1 	mov	x1, x19
ffffc020a4d4: aa1403e0 	mov	x0, x20
ffffc020a4d8: 97ffff07 	bl	0xffffc020a0f4 <pci_add_physical_device>
ffffc020a4dc: 2a0003f5 	mov	w21, w0
ffffc020a4e0: 35000800 	cbnz	w0, 0xffffc020a5e0 <pci_cell_init+0x250>
ffffc020a4e4: f9400660 	ldr	x0, [x19, #8]
ffffc020a4e8: f9485000 	ldr	x0, [x0, #4256]
ffffc020a4ec: 97fffc3d 	bl	0xffffc02095e0 <jailhouse_cell_pci_caps>
ffffc020a4f0: f8776b81 	ldr	x1, [x28, x23]
ffffc020a4f4: 5280001c 	mov	w28, #0
ffffc020a4f8: 79403c35 	ldrh	w21, [x1, #30]
ffffc020a4fc: 8b150c15 	add	x21, x0, x21, lsl #3
ffffc020a500: f9400260 	ldr	x0, [x19]
ffffc020a504: 79404001 	ldrh	w1, [x0, #32]
ffffc020a508: 6b1c003f 	cmp	w1, w28
ffffc020a50c: 54fffd49 	b.ls	0xffffc020a4b4 <pci_cell_init+0x124>
ffffc020a510: 794002a1 	ldrh	w1, [x21]
ffffc020a514: 7100143f 	cmp	w1, #5
ffffc020a518: 54000301 	b.ne	0xffffc020a578 <pci_cell_init+0x1e8>
ffffc020a51c: 79400803 	ldrh	w3, [x0, #4]
ffffc020a520: d2800017 	mov	x23, #0
ffffc020a524: f9400260 	ldr	x0, [x19]
ffffc020a528: 39408c00 	ldrb	w0, [x0, #35]
ffffc020a52c: 7100001f 	cmp	w0, #0
ffffc020a530: 1a9f07e0 	cset	w0, ne
ffffc020a534: 11000c00 	add	w0, w0, #3
ffffc020a538: 6b17001f 	cmp	w0, w23
ffffc020a53c: 54000088 	b.hi	0xffffc020a54c <pci_cell_init+0x1bc>
ffffc020a540: 910022b5 	add	x21, x21, #8
ffffc020a544: 1100079c 	add	w28, w28, #1
ffffc020a548: 17ffffee 	b	0xffffc020a500 <pci_cell_init+0x170>
ffffc020a54c: 794006a1 	ldrh	w1, [x21, #2]
ffffc020a550: 2a0303e0 	mov	w0, w3
ffffc020a554: b9006fa3 	str	w3, [x29, #108]
ffffc020a558: 52800082 	mov	w2, #4
ffffc020a55c: 0b170821 	add	w1, w1, w23, lsl #2
ffffc020a560: 97fffc74 	bl	0xffffc0209730 <pci_read_config>
ffffc020a564: 8b170a61 	add	x1, x19, x23, lsl #2
ffffc020a568: b9406fa3 	ldr	w3, [x29, #108]
ffffc020a56c: 910006f7 	add	x23, x23, #1
ffffc020a570: b9002820 	str	w0, [x1, #40]
ffffc020a574: 17ffffec 	b	0xffffc020a524 <pci_cell_init+0x194>
ffffc020a578: 7100443f 	cmp	w1, #17
ffffc020a57c: 54fffe21 	b.ne	0xffffc020a540 <pci_cell_init+0x1b0>
ffffc020a580: 794006a1 	ldrh	w1, [x21, #2]
ffffc020a584: 52800082 	mov	w2, #4
ffffc020a588: 79400800 	ldrh	w0, [x0, #4]
ffffc020a58c: 97fffc69 	bl	0xffffc0209730 <pci_read_config>
ffffc020a590: b9003a60 	str	w0, [x19, #56]
ffffc020a594: d2800000 	mov	x0, #0
ffffc020a598: f9400261 	ldr	x1, [x19]
ffffc020a59c: 79404821 	ldrh	w1, [x1, #36]
ffffc020a5a0: 6b00003f 	cmp	w1, w0
ffffc020a5a4: 54fffce9 	b.ls	0xffffc020a540 <pci_cell_init+0x1b0>
ffffc020a5a8: d37cec04 	lsl	x4, x0, #4
ffffc020a5ac: 52800001 	mov	w1, #0
ffffc020a5b0: f9402662 	ldr	x2, [x19, #72]
ffffc020a5b4: 2a0103e3 	mov	w3, w1
ffffc020a5b8: 11000421 	add	w1, w1, #1
ffffc020a5bc: 8b040042 	add	x2, x2, x4
ffffc020a5c0: 7100103f 	cmp	w1, #4
ffffc020a5c4: b8637845 	ldr	w5, [x2, x3, lsl #2]
ffffc020a5c8: f9402a62 	ldr	x2, [x19, #80]
ffffc020a5cc: 8b040042 	add	x2, x2, x4
ffffc020a5d0: b8237845 	str	w5, [x2, x3, lsl #2]
ffffc020a5d4: 54fffee1 	b.ne	0xffffc020a5b0 <pci_cell_init+0x220>
ffffc020a5d8: 91000400 	add	x0, x0, #1
ffffc020a5dc: 17ffffef 	b	0xffffc020a598 <pci_cell_init+0x208>
ffffc020a5e0: aa1403e0 	mov	x0, x20
ffffc020a5e4: 97ffff16 	bl	0xffffc020a23c <pci_cell_exit>
ffffc020a5e8: 17ffff84 	b	0xffffc020a3f8 <pci_cell_init+0x68>
ffffc020a5ec: 12800175 	mov	w21, #-12
ffffc020a5f0: 17ffff82 	b	0xffffc020a3f8 <pci_cell_init+0x68>

0000ffffc020a5f4 <pci_init>:
ffffc020a5f4: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc020a5f8: f0000060 	adrp	x0, 0xffffc0219000 <pci_init+0x40>
ffffc020a5fc: 910003fd 	mov	x29, sp
ffffc020a600: f9400002 	ldr	x2, [x0]
ffffc020a604: f9000bf3 	str	x19, [sp, #16]
ffffc020a608: d0000093 	adrp	x19, 0xffffc021c000 <pci_init+0x5c>
ffffc020a60c: 91000263 	add	x3, x19, #0
ffffc020a610: 39415041 	ldrb	w1, [x2, #84]
ffffc020a614: f844c040 	ldur	x0, [x2, #76]
ffffc020a618: 39002061 	strb	w1, [x3, #8]
ffffc020a61c: 11000421 	add	w1, w1, #1
ffffc020a620: d36c2021 	ubfiz	x1, x1, #20, #9
ffffc020a624: a9010460 	stp	x0, x1, [x3, #16]
ffffc020a628: b50000c0 	cbnz	x0, 0xffffc020a640 <pci_init+0x4c>
ffffc020a62c: f9400bf3 	ldr	x19, [sp, #16]
ffffc020a630: 90000080 	adrp	x0, 0xffffc021a000 <pci_cell_reset+0xc>
ffffc020a634: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020a638: 91000000 	add	x0, x0, #0
ffffc020a63c: 17ffff55 	b	0xffffc020a390 <pci_cell_init>
ffffc020a640: 39415442 	ldrb	w2, [x2, #85]
ffffc020a644: 35ffff42 	cbnz	w2, 0xffffc020a62c <pci_init+0x38>
ffffc020a648: 97fff372 	bl	0xffffc0207410 <paging_map_device>
ffffc020a64c: f9000260 	str	x0, [x19]
ffffc020a650: b5fffee0 	cbnz	x0, 0xffffc020a62c <pci_init+0x38>
ffffc020a654: 12800160 	mov	w0, #-12
ffffc020a658: f9400bf3 	ldr	x19, [sp, #16]
ffffc020a65c: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020a660: d65f03c0 	ret

0000ffffc020a664 <pci_cell_reset>:
ffffc020a664: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc020a668: 910003fd 	mov	x29, sp
ffffc020a66c: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020a670: aa0003f4 	mov	x20, x0
ffffc020a674: f90013f5 	str	x21, [sp, #32]
ffffc020a678: d297d075 	mov	x21, #48771
ffffc020a67c: f2a5f415 	movk	x21, #12192, lsl #16
ffffc020a680: f9486813 	ldr	x19, [x0, #4304]
ffffc020a684: f2c17d15 	movk	x21, #3048, lsl #32
ffffc020a688: f2f05f55 	movk	x21, #33530, lsl #48
ffffc020a68c: f9486a80 	ldr	x0, [x20, #4304]
ffffc020a690: f9485281 	ldr	x1, [x20, #4256]
ffffc020a694: cb000260 	sub	x0, x19, x0
ffffc020a698: 9343fc00 	asr	x0, x0, #3
ffffc020a69c: b9404421 	ldr	w1, [x1, #68]
ffffc020a6a0: 9b157c00 	mul	x0, x0, x21
ffffc020a6a4: eb01001f 	cmp	x0, x1
ffffc020a6a8: 540000ab 	b.lt	0xffffc020a6bc <pci_cell_reset+0x58>
ffffc020a6ac: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a6b0: f94013f5 	ldr	x21, [sp, #32]
ffffc020a6b4: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc020a6b8: d65f03c0 	ret
ffffc020a6bc: f9400660 	ldr	x0, [x19, #8]
ffffc020a6c0: b4000060 	cbz	x0, 0xffffc020a6cc <pci_cell_reset+0x68>
ffffc020a6c4: aa1303e0 	mov	x0, x19
ffffc020a6c8: 97fffe0a 	bl	0xffffc0209ef0 <pci_reset_device>
ffffc020a6cc: 91056273 	add	x19, x19, #344
ffffc020a6d0: 17ffffef 	b	0xffffc020a68c <pci_cell_reset+0x28>

0000ffffc020a6d4 <pci_config_commit>:
ffffc020a6d4: b4000ae0 	cbz	x0, 0xffffc020a830 <pci_config_commit+0x15c>
ffffc020a6d8: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc020a6dc: 910003fd 	mov	x29, sp
ffffc020a6e0: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020a6e4: 90000095 	adrp	x21, 0xffffc021a000 <pci_config_commit+0x50>
ffffc020a6e8: 910002b5 	add	x21, x21, #0
ffffc020a6ec: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020a6f0: d297d077 	mov	x23, #48771
ffffc020a6f4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020a6f8: f2a5f417 	movk	x23, #12192, lsl #16
ffffc020a6fc: f9486ab3 	ldr	x19, [x21, #4304]
ffffc020a700: f2c17d17 	movk	x23, #3048, lsl #32
ffffc020a704: f2f05f57 	movk	x23, #33530, lsl #48
ffffc020a708: f9486aa0 	ldr	x0, [x21, #4304]
ffffc020a70c: f94852a1 	ldr	x1, [x21, #4256]
ffffc020a710: cb000260 	sub	x0, x19, x0
ffffc020a714: 9343fc00 	asr	x0, x0, #3
ffffc020a718: b9404421 	ldr	w1, [x1, #68]
ffffc020a71c: 9b177c00 	mul	x0, x0, x23
ffffc020a720: eb01001f 	cmp	x0, x1
ffffc020a724: 540000cb 	b.lt	0xffffc020a73c <pci_config_commit+0x68>
ffffc020a728: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020a72c: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020a730: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc020a734: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc020a738: d65f03c0 	ret
ffffc020a73c: f9400660 	ldr	x0, [x19, #8]
ffffc020a740: b5000060 	cbnz	x0, 0xffffc020a74c <pci_config_commit+0x78>
ffffc020a744: 91056273 	add	x19, x19, #344
ffffc020a748: 17fffff0 	b	0xffffc020a708 <pci_config_commit+0x34>
ffffc020a74c: f9485000 	ldr	x0, [x0, #4256]
ffffc020a750: 52800016 	mov	w22, #0
ffffc020a754: 97fffba3 	bl	0xffffc02095e0 <jailhouse_cell_pci_caps>
ffffc020a758: f9400261 	ldr	x1, [x19]
ffffc020a75c: 79403c34 	ldrh	w20, [x1, #30]
ffffc020a760: 8b140c14 	add	x20, x0, x20, lsl #3
ffffc020a764: f9400260 	ldr	x0, [x19]
ffffc020a768: 79404001 	ldrh	w1, [x0, #32]
ffffc020a76c: 6b16003f 	cmp	w1, w22
ffffc020a770: 54000128 	b.hi	0xffffc020a794 <pci_config_commit+0xc0>
ffffc020a774: 39400000 	ldrb	w0, [x0]
ffffc020a778: 71000c1f 	cmp	w0, #3
ffffc020a77c: 54fffe41 	b.ne	0xffffc020a744 <pci_config_commit+0x70>
ffffc020a780: aa1303e0 	mov	x0, x19
ffffc020a784: 97ffe16a 	bl	0xffffc0202d2c <arch_ivshmem_update_msix>
ffffc020a788: 34fffde0 	cbz	w0, 0xffffc020a744 <pci_config_commit+0x70>
ffffc020a78c: d2800014 	mov	x20, #0
ffffc020a790: 14000009 	b	0xffffc020a7b4 <pci_config_commit+0xe0>
ffffc020a794: 79400280 	ldrh	w0, [x20]
ffffc020a798: 7100141f 	cmp	w0, #5
ffffc020a79c: 54000281 	b.ne	0xffffc020a7ec <pci_config_commit+0x118>
ffffc020a7a0: aa1403e1 	mov	x1, x20
ffffc020a7a4: aa1303e0 	mov	x0, x19
ffffc020a7a8: 97ffe2d0 	bl	0xffffc02032e8 <arch_pci_update_msi>
ffffc020a7ac: 2a0003f8 	mov	w24, w0
ffffc020a7b0: 34000238 	cbz	w24, 0xffffc020a7f4 <pci_config_commit+0x120>
ffffc020a7b4: f9400260 	ldr	x0, [x19]
ffffc020a7b8: 79400801 	ldrh	w1, [x0, #4]
ffffc020a7bc: d0000000 	adrp	x0, 0xffffc020c000 <pci_config_commit+0xf0>
ffffc020a7c0: 91067400 	add	x0, x0, #413
ffffc020a7c4: 12000823 	and	w3, w1, #0x7
ffffc020a7c8: d3431c22 	ubfx	x2, x1, #3, #5
ffffc020a7cc: 53087c21 	lsr	w1, w1, #8
ffffc020a7d0: 97fff108 	bl	0xffffc0206bf0 <panic_printk>
ffffc020a7d4: b4000274 	cbz	x20, 0xffffc020a820 <pci_config_commit+0x14c>
ffffc020a7d8: 79400281 	ldrh	w1, [x20]
ffffc020a7dc: d0000000 	adrp	x0, 0xffffc020c000 <pci_config_commit+0x110>
ffffc020a7e0: 91075400 	add	x0, x0, #469
ffffc020a7e4: 97fff103 	bl	0xffffc0206bf0 <panic_printk>
ffffc020a7e8: 97fff94b 	bl	0xffffc0208d14 <panic_stop>
ffffc020a7ec: 7100441f 	cmp	w0, #17
ffffc020a7f0: 54000080 	b.eq	0xffffc020a800 <pci_config_commit+0x12c>
ffffc020a7f4: 91002294 	add	x20, x20, #8
ffffc020a7f8: 110006d6 	add	w22, w22, #1
ffffc020a7fc: 17ffffda 	b	0xffffc020a764 <pci_config_commit+0x90>
ffffc020a800: aa1303e0 	mov	x0, x19
ffffc020a804: 97fffb64 	bl	0xffffc0209594 <pci_update_msix.isra.0>
ffffc020a808: 52800002 	mov	w2, #0
ffffc020a80c: 2a0003f8 	mov	w24, w0
ffffc020a810: aa1403e1 	mov	x1, x20
ffffc020a814: aa1303e0 	mov	x0, x19
ffffc020a818: 97fffbf6 	bl	0xffffc02097f0 <pci_suppress_msix>
ffffc020a81c: 17ffffe5 	b	0xffffc020a7b0 <pci_config_commit+0xdc>
ffffc020a820: d0000000 	adrp	x0, 0xffffc020c000 <pci_config_commit+0x154>
ffffc020a824: 91013000 	add	x0, x0, #76
ffffc020a828: 97fff0f2 	bl	0xffffc0206bf0 <panic_printk>
ffffc020a82c: 17ffffef 	b	0xffffc020a7e8 <pci_config_commit+0x114>
ffffc020a830: d65f03c0 	ret

0000ffffc020a834 <spin_lock>:
ffffc020a834: f9800011 	prfm	pstl1strm, [x0]
ffffc020a838: 885ffc01 	ldaxr	w1, [x0]
ffffc020a83c: 11404022 	add	w2, w1, #16, lsl #12    // =65536
ffffc020a840: 88037c02 	stxr	w3, w2, [x0]
ffffc020a844: 35ffffa3 	cbnz	w3, 0xffffc020a838 <spin_lock+0x4>
ffffc020a848: 4ac14022 	eor	w2, w1, w1, ror #16
ffffc020a84c: 340000c2 	cbz	w2, 0xffffc020a864 <spin_lock+0x30>
ffffc020a850: d50320bf 	sevl
ffffc020a854: d503205f 	wfe
ffffc020a858: 485ffc03 	ldaxrh	w3, [x0]
ffffc020a85c: 4a414062 	eor	w2, w3, w1, lsr #16
ffffc020a860: 35ffffa2 	cbnz	w2, 0xffffc020a854 <spin_lock+0x20>
ffffc020a864: d65f03c0 	ret

0000ffffc020a868 <ivshmem_msix_mmio>:
ffffc020a868: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020a86c: 910003fd 	mov	x29, sp
ffffc020a870: f9400022 	ldr	x2, [x1]
ffffc020a874: f9403c03 	ldr	x3, [x0, #120]
ffffc020a878: f240045f 	tst	x2, #0x3
ffffc020a87c: 540002e1 	b.ne	0xffffc020a8d8 <ivshmem_msix_mmio+0x70>
ffffc020a880: f1003c5f 	cmp	x2, #15
ffffc020a884: b9400c24 	ldr	w4, [x1, #12]
ffffc020a888: 540000a9 	b.ls	0xffffc020a89c <ivshmem_msix_mmio+0x34>
ffffc020a88c: 35000264 	cbnz	w4, 0xffffc020a8d8 <ivshmem_msix_mmio+0x70>
ffffc020a890: f900083f 	str	xzr, [x1, #16]
ffffc020a894: 52800020 	mov	w0, #1
ffffc020a898: 1400000b 	b	0xffffc020a8c4 <ivshmem_msix_mmio+0x5c>
ffffc020a89c: 927ef442 	and	x2, x2, #0xfffffffffffffffc
ffffc020a8a0: f9402863 	ldr	x3, [x3, #80]
ffffc020a8a4: 34000144 	cbz	w4, 0xffffc020a8cc <ivshmem_msix_mmio+0x64>
ffffc020a8a8: f9400821 	ldr	x1, [x1, #16]
ffffc020a8ac: b8226861 	str	w1, [x3, x2]
ffffc020a8b0: f9403c00 	ldr	x0, [x0, #120]
ffffc020a8b4: 97ffe11e 	bl	0xffffc0202d2c <arch_ivshmem_update_msix>
ffffc020a8b8: 7100001f 	cmp	w0, #0
ffffc020a8bc: 12800001 	mov	w1, #-1
ffffc020a8c0: 5a811420 	cneg	w0, w1, eq
ffffc020a8c4: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc020a8c8: d65f03c0 	ret
ffffc020a8cc: b8626860 	ldr	w0, [x3, x2]
ffffc020a8d0: f9000820 	str	x0, [x1, #16]
ffffc020a8d4: 17fffff0 	b	0xffffc020a894 <ivshmem_msix_mmio+0x2c>
ffffc020a8d8: f9400060 	ldr	x0, [x3]
ffffc020a8dc: 79400801 	ldrh	w1, [x0, #4]
ffffc020a8e0: d0000000 	adrp	x0, 0xffffc020c000 <ivshmem_msix_mmio+0x80>
ffffc020a8e4: 9101f800 	add	x0, x0, #126
ffffc020a8e8: 12000823 	and	w3, w1, #0x7
ffffc020a8ec: d3431c22 	ubfx	x2, x1, #3, #5
ffffc020a8f0: 53087c21 	lsr	w1, w1, #8
ffffc020a8f4: 97fff0bf 	bl	0xffffc0206bf0 <panic_printk>
ffffc020a8f8: 12800000 	mov	w0, #-1
ffffc020a8fc: 17fffff2 	b	0xffffc020a8c4 <ivshmem_msix_mmio+0x5c>

0000ffffc020a900 <ivshmem_remote_interrupt>:
ffffc020a900: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc020a904: 910003fd 	mov	x29, sp
ffffc020a908: f9000bf3 	str	x19, [sp, #16]
ffffc020a90c: aa0003f3 	mov	x19, x0
ffffc020a910: 91024000 	add	x0, x0, #144
ffffc020a914: 97ffffc8 	bl	0xffffc020a834 <spin_lock>
ffffc020a918: f9404660 	ldr	x0, [x19, #136]
ffffc020a91c: b4000040 	cbz	x0, 0xffffc020a924 <ivshmem_remote_interrupt+0x24>
ffffc020a920: 97ffe0fe 	bl	0xffffc0202d18 <arch_ivshmem_trigger_interrupt>
ffffc020a924: 79412260 	ldrh	w0, [x19, #144]
ffffc020a928: 91024261 	add	x1, x19, #144
ffffc020a92c: 11000400 	add	w0, w0, #1
ffffc020a930: 489ffc20 	stlrh	w0, [x1]
ffffc020a934: f9400bf3 	ldr	x19, [sp, #16]
ffffc020a938: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020a93c: d65f03c0 	ret

0000ffffc020a940 <ivshmem_register_mmio>:
ffffc020a940: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020a944: aa0003e4 	mov	x4, x0
ffffc020a948: aa0103e5 	mov	x5, x1
ffffc020a94c: 910003fd 	mov	x29, sp
ffffc020a950: f9400022 	ldr	x2, [x1]
ffffc020a954: b9400c23 	ldr	w3, [x1, #12]
ffffc020a958: b5000182 	cbnz	x2, 0xffffc020a988 <ivshmem_register_mmio+0x48>
ffffc020a95c: 34000103 	cbz	w3, 0xffffc020a97c <ivshmem_register_mmio+0x3c>
ffffc020a960: f9400821 	ldr	x1, [x1, #16]
ffffc020a964: 12000021 	and	w1, w1, #0x1
ffffc020a968: b9009801 	str	w1, [x0, #152]
ffffc020a96c: 97ffe110 	bl	0xffffc0202dac <arch_ivshmem_update_intx>
ffffc020a970: 52800020 	mov	w0, #1
ffffc020a974: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc020a978: d65f03c0 	ret
ffffc020a97c: b9409800 	ldr	w0, [x0, #152]
ffffc020a980: f90008a0 	str	x0, [x5, #16]
ffffc020a984: 17fffffb 	b	0xffffc020a970 <ivshmem_register_mmio+0x30>
ffffc020a988: f100205f 	cmp	x2, #8
ffffc020a98c: 54000081 	b.ne	0xffffc020a99c <ivshmem_register_mmio+0x5c>
ffffc020a990: 350005a3 	cbnz	w3, 0xffffc020aa44 <ivshmem_register_mmio+0x104>
ffffc020a994: b9405c00 	ldr	w0, [x0, #92]
ffffc020a998: 17fffffa 	b	0xffffc020a980 <ivshmem_register_mmio+0x40>
ffffc020a99c: f100305f 	cmp	x2, #12
ffffc020a9a0: 540000c1 	b.ne	0xffffc020a9b8 <ivshmem_register_mmio+0x78>
ffffc020a9a4: 34000063 	cbz	w3, 0xffffc020a9b0 <ivshmem_register_mmio+0x70>
ffffc020a9a8: 97ffffd6 	bl	0xffffc020a900 <ivshmem_remote_interrupt>
ffffc020a9ac: 17fffff1 	b	0xffffc020a970 <ivshmem_register_mmio+0x30>
ffffc020a9b0: f900083f 	str	xzr, [x1, #16]
ffffc020a9b4: 17ffffef 	b	0xffffc020a970 <ivshmem_register_mmio+0x30>
ffffc020a9b8: f100405f 	cmp	x2, #16
ffffc020a9bc: 540000e1 	b.ne	0xffffc020a9d8 <ivshmem_register_mmio+0x98>
ffffc020a9c0: 34000083 	cbz	w3, 0xffffc020a9d0 <ivshmem_register_mmio+0x90>
ffffc020a9c4: f9400821 	ldr	x1, [x1, #16]
ffffc020a9c8: b9006001 	str	w1, [x0, #96]
ffffc020a9cc: 17fffff7 	b	0xffffc020a9a8 <ivshmem_register_mmio+0x68>
ffffc020a9d0: b9406000 	ldr	w0, [x0, #96]
ffffc020a9d4: 17ffffeb 	b	0xffffc020a980 <ivshmem_register_mmio+0x40>
ffffc020a9d8: f100505f 	cmp	x2, #20
ffffc020a9dc: 540001e1 	b.ne	0xffffc020aa18 <ivshmem_register_mmio+0xd8>
ffffc020a9e0: 35000323 	cbnz	w3, 0xffffc020aa44 <ivshmem_register_mmio+0x104>
ffffc020a9e4: 91024000 	add	x0, x0, #144
ffffc020a9e8: 97ffff93 	bl	0xffffc020a834 <spin_lock>
ffffc020a9ec: f9404480 	ldr	x0, [x4, #136]
ffffc020a9f0: b4000100 	cbz	x0, 0xffffc020aa10 <ivshmem_register_mmio+0xd0>
ffffc020a9f4: b9406000 	ldr	w0, [x0, #96]
ffffc020a9f8: f90008a0 	str	x0, [x5, #16]
ffffc020a9fc: 91024081 	add	x1, x4, #144
ffffc020aa00: 79412080 	ldrh	w0, [x4, #144]
ffffc020aa04: 11000400 	add	w0, w0, #1
ffffc020aa08: 489ffc20 	stlrh	w0, [x1]
ffffc020aa0c: 17ffffd9 	b	0xffffc020a970 <ivshmem_register_mmio+0x30>
ffffc020aa10: d2800000 	mov	x0, #0
ffffc020aa14: 17fffff9 	b	0xffffc020a9f8 <ivshmem_register_mmio+0xb8>
ffffc020aa18: b0000000 	adrp	x0, 0xffffc020b000 <ivshmem_register_mmio+0xdc>
ffffc020aa1c: b0000001 	adrp	x1, 0xffffc020b000 <ivshmem_register_mmio+0xe0>
ffffc020aa20: 7100007f 	cmp	w3, #0
ffffc020aa24: 912f8c00 	add	x0, x0, #3043
ffffc020aa28: 912fa421 	add	x1, x1, #3049
ffffc020aa2c: 9a800021 	csel	x1, x1, x0, eq
ffffc020aa30: d0000000 	adrp	x0, 0xffffc020c000 <ivshmem_register_mmio+0xf8>
ffffc020aa34: 91078c00 	add	x0, x0, #483
ffffc020aa38: 97fff06e 	bl	0xffffc0206bf0 <panic_printk>
ffffc020aa3c: 12800000 	mov	w0, #-1
ffffc020aa40: 17ffffcd 	b	0xffffc020a974 <ivshmem_register_mmio+0x34>
ffffc020aa44: b0000001 	adrp	x1, 0xffffc020b000 <ivshmem_register_mmio+0x108>
ffffc020aa48: 912f8c21 	add	x1, x1, #3043
ffffc020aa4c: 17fffff9 	b	0xffffc020aa30 <ivshmem_register_mmio+0xf0>

0000ffffc020aa50 <ivshmem_is_msix_masked>:
ffffc020aa50: 39414c01 	ldrb	w1, [x0, #83]
ffffc020aa54: 121a0421 	and	w1, w1, #0xc0
ffffc020aa58: 7102003f 	cmp	w1, #128
ffffc020aa5c: 54000121 	b.ne	0xffffc020aa80 <ivshmem_is_msix_masked+0x30>
ffffc020aa60: f9403c01 	ldr	x1, [x0, #120]
ffffc020aa64: f9402821 	ldr	x1, [x1, #80]
ffffc020aa68: 39403021 	ldrb	w1, [x1, #12]
ffffc020aa6c: 370000a1 	tbnz	w1, #0, 0xffffc020aa80 <ivshmem_is_msix_masked+0x30>
ffffc020aa70: b9400400 	ldr	w0, [x0, #4]
ffffc020aa74: d27e0000 	eor	x0, x0, #0x4
ffffc020aa78: d3420800 	ubfx	x0, x0, #2, #1
ffffc020aa7c: d65f03c0 	ret
ffffc020aa80: 52800020 	mov	w0, #1
ffffc020aa84: 17fffffe 	b	0xffffc020aa7c <ivshmem_is_msix_masked+0x2c>

0000ffffc020aa88 <ivshmem_pci_cfg_write>:
ffffc020aa88: 7100583f 	cmp	w1, #22
ffffc020aa8c: 54000069 	b.ls	0xffffc020aa98 <ivshmem_pci_cfg_write+0x10>
ffffc020aa90: 52800000 	mov	w0, #0
ffffc020aa94: d65f03c0 	ret
ffffc020aa98: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc020aa9c: 7100043f 	cmp	w1, #1
ffffc020aaa0: 910003fd 	mov	x29, sp
ffffc020aaa4: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020aaa8: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020aaac: f9402013 	ldr	x19, [x0, #64]
ffffc020aab0: b8615a74 	ldr	w20, [x19, w1, uxtw #2]
ffffc020aab4: 0a220294 	bic	w20, w20, w2
ffffc020aab8: 2a030294 	orr	w20, w20, w3
ffffc020aabc: 540000a0 	b.eq	0xffffc020aad0 <ivshmem_pci_cfg_write+0x48>
ffffc020aac0: 7100503f 	cmp	w1, #20
ffffc020aac4: 540006e0 	b.eq	0xffffc020aba0 <ivshmem_pci_cfg_write+0x118>
ffffc020aac8: 52800040 	mov	w0, #2
ffffc020aacc: 14000031 	b	0xffffc020ab90 <ivshmem_pci_cfg_write+0x108>
ffffc020aad0: 79400a60 	ldrh	w0, [x19, #4]
ffffc020aad4: 12003e96 	and	w22, w20, #0xffff
ffffc020aad8: f9403e75 	ldr	x21, [x19, #120]
ffffc020aadc: 4a0002c1 	eor	w1, w22, w0
ffffc020aae0: 37100481 	tbnz	w1, #2, 0xffffc020ab70 <ivshmem_pci_cfg_write+0xe8>
ffffc020aae4: 79400a60 	ldrh	w0, [x19, #4]
ffffc020aae8: 4a0002c1 	eor	w1, w22, w0
ffffc020aaec: 360ffee1 	tbz	w1, #1, 0xffffc020aac8 <ivshmem_pci_cfg_write+0x40>
ffffc020aaf0: 360800e0 	tbz	w0, #1, 0xffffc020ab0c <ivshmem_pci_cfg_write+0x84>
ffffc020aaf4: f9403661 	ldr	x1, [x19, #104]
ffffc020aaf8: f94006a0 	ldr	x0, [x21, #8]
ffffc020aafc: 97fff9a3 	bl	0xffffc0209188 <mmio_region_unregister>
ffffc020ab00: f9403a61 	ldr	x1, [x19, #112]
ffffc020ab04: f94006a0 	ldr	x0, [x21, #8]
ffffc020ab08: 97fff9a0 	bl	0xffffc0209188 <mmio_region_unregister>
ffffc020ab0c: 36080276 	tbz	w22, #1, 0xffffc020ab58 <ivshmem_pci_cfg_write+0xd0>
ffffc020ab10: f9400aa1 	ldr	x1, [x21, #16]
ffffc020ab14: aa1303e4 	mov	x4, x19
ffffc020ab18: d2802002 	mov	x2, #256
ffffc020ab1c: 90000003 	adrp	x3, 0xffffc020a000 <ivshmem_pci_cfg_write+0x94>
ffffc020ab20: 927cec21 	and	x1, x1, #0xfffffffffffffff0
ffffc020ab24: f9003661 	str	x1, [x19, #104]
ffffc020ab28: 91250063 	add	x3, x3, #2368
ffffc020ab2c: f94006a0 	ldr	x0, [x21, #8]
ffffc020ab30: 97fff94c 	bl	0xffffc0209060 <mmio_region_register>
ffffc020ab34: f94012a1 	ldr	x1, [x21, #32]
ffffc020ab38: 90000003 	adrp	x3, 0xffffc020a000 <ivshmem_pci_cfg_write+0xb0>
ffffc020ab3c: aa1303e4 	mov	x4, x19
ffffc020ab40: 9121a063 	add	x3, x3, #2152
ffffc020ab44: 927cec21 	and	x1, x1, #0xfffffffffffffff0
ffffc020ab48: f9003a61 	str	x1, [x19, #112]
ffffc020ab4c: d2800402 	mov	x2, #32
ffffc020ab50: f94006a0 	ldr	x0, [x21, #8]
ffffc020ab54: 97fff943 	bl	0xffffc0209060 <mmio_region_register>
ffffc020ab58: 79400a60 	ldrh	w0, [x19, #4]
ffffc020ab5c: 121f0294 	and	w20, w20, #0x2
ffffc020ab60: 121e7800 	and	w0, w0, #0xfffffffd
ffffc020ab64: 2a140014 	orr	w20, w0, w20
ffffc020ab68: 79000a74 	strh	w20, [x19, #4]
ffffc020ab6c: 17ffffd7 	b	0xffffc020aac8 <ivshmem_pci_cfg_write+0x40>
ffffc020ab70: 121d7800 	and	w0, w0, #0xfffffffb
ffffc020ab74: 121e0281 	and	w1, w20, #0x4
ffffc020ab78: 2a010000 	orr	w0, w0, w1
ffffc020ab7c: 79000a60 	strh	w0, [x19, #4]
ffffc020ab80: aa1503e0 	mov	x0, x21
ffffc020ab84: 97ffe06a 	bl	0xffffc0202d2c <arch_ivshmem_update_msix>
ffffc020ab88: 34fffae0 	cbz	w0, 0xffffc020aae4 <ivshmem_pci_cfg_write+0x5c>
ffffc020ab8c: 52800000 	mov	w0, #0
ffffc020ab90: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020ab94: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020ab98: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc020ab9c: d65f03c0 	ret
ffffc020aba0: b9405260 	ldr	w0, [x19, #80]
ffffc020aba4: 531f7e82 	lsr	w2, w20, #31
ffffc020aba8: b9003fb4 	str	w20, [x29, #60]
ffffc020abac: d35e7a94 	ubfx	x20, x20, #30, #1
ffffc020abb0: 2a0003e1 	mov	w1, w0
ffffc020abb4: 33010040 	bfi	w0, w2, #31, #1
ffffc020abb8: 33020280 	bfi	w0, w20, #30, #1
ffffc020abbc: 6b00003f 	cmp	w1, w0
ffffc020abc0: 54fff840 	b.eq	0xffffc020aac8 <ivshmem_pci_cfg_write+0x40>
ffffc020abc4: b9005260 	str	w0, [x19, #80]
ffffc020abc8: f9403e60 	ldr	x0, [x19, #120]
ffffc020abcc: 97ffe058 	bl	0xffffc0202d2c <arch_ivshmem_update_msix>
ffffc020abd0: 35fffde0 	cbnz	w0, 0xffffc020ab8c <ivshmem_pci_cfg_write+0x104>
ffffc020abd4: 17ffffbd 	b	0xffffc020aac8 <ivshmem_pci_cfg_write+0x40>

0000ffffc020abd8 <ivshmem_pci_cfg_read>:
ffffc020abd8: 12003c21 	and	w1, w1, #0xffff
ffffc020abdc: 71016c3f 	cmp	w1, #91
ffffc020abe0: 54000128 	b.hi	0xffffc020ac04 <ivshmem_pci_cfg_read+0x2c>
ffffc020abe4: d3423c23 	ubfx	x3, x1, #2, #14
ffffc020abe8: f9402000 	ldr	x0, [x0, #64]
ffffc020abec: 531d0421 	ubfiz	w1, w1, #3, #2
ffffc020abf0: b8637800 	ldr	w0, [x0, x3, lsl #2]
ffffc020abf4: 1ac12401 	lsr	w1, w0, w1
ffffc020abf8: b9000041 	str	w1, [x2]
ffffc020abfc: 52800040 	mov	w0, #2
ffffc020ac00: d65f03c0 	ret
ffffc020ac04: 12800000 	mov	w0, #-1
ffffc020ac08: b9000040 	str	w0, [x2]
ffffc020ac0c: 17fffffc 	b	0xffffc020abfc <ivshmem_pci_cfg_read+0x24>

0000ffffc020ac10 <ivshmem_init>:
ffffc020ac10: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc020ac14: 910003fd 	mov	x29, sp
ffffc020ac18: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020ac1c: aa0003f4 	mov	x20, x0
ffffc020ac20: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020ac24: aa0103f6 	mov	x22, x1
ffffc020ac28: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020ac2c: f9400035 	ldr	x21, [x1]
ffffc020ac30: f9485004 	ldr	x4, [x0, #4256]
ffffc020ac34: d0000000 	adrp	x0, 0xffffc020c000 <ivshmem_init+0x2c>
ffffc020ac38: 91085400 	add	x0, x0, #533
ffffc020ac3c: 79400aa1 	ldrh	w1, [x21, #4]
ffffc020ac40: 91002084 	add	x4, x4, #8
ffffc020ac44: 12000823 	and	w3, w1, #0x7
ffffc020ac48: d3431c22 	ubfx	x2, x1, #3, #5
ffffc020ac4c: 53087c21 	lsr	w1, w1, #8
ffffc020ac50: 97ffefb5 	bl	0xffffc0206b24 <printk>
ffffc020ac54: f9485281 	ldr	x1, [x20, #4256]
ffffc020ac58: b94032a3 	ldr	w3, [x21, #48]
ffffc020ac5c: b9403420 	ldr	w0, [x1, #52]
ffffc020ac60: 6b00007f 	cmp	w3, w0
ffffc020ac64: 54000a62 	b.hs	0xffffc020adb0 <ivshmem_init+0x1a0>
ffffc020ac68: b9403020 	ldr	w0, [x1, #48]
ffffc020ac6c: 52800402 	mov	w2, #32
ffffc020ac70: d0000097 	adrp	x23, 0xffffc021c000 <ivshmem_init+0xa8>
ffffc020ac74: 91020000 	add	x0, x0, #128
ffffc020ac78: f94012f3 	ldr	x19, [x23, #32]
ffffc020ac7c: 9ba20063 	umaddl	x3, w3, w2, x0
ffffc020ac80: 8b030038 	add	x24, x1, x3
ffffc020ac84: b5000133 	cbnz	x19, 0xffffc020aca8 <ivshmem_init+0x98>
ffffc020ac88: 52800021 	mov	w1, #1
ffffc020ac8c: d0000060 	adrp	x0, 0xffffc0218000 <ivshmem_init+0xb4>
ffffc020ac90: 91004000 	add	x0, x0, #16
ffffc020ac94: 97fff079 	bl	0xffffc0206e78 <page_alloc>
ffffc020ac98: aa0003f3 	mov	x19, x0
ffffc020ac9c: b50007c0 	cbnz	x0, 0xffffc020ad94 <ivshmem_init+0x184>
ffffc020aca0: 12800160 	mov	w0, #-12
ffffc020aca4: 14000037 	b	0xffffc020ad80 <ivshmem_init+0x170>
ffffc020aca8: 79428262 	ldrh	w2, [x19, #320]
ffffc020acac: 79400aa0 	ldrh	w0, [x21, #4]
ffffc020acb0: 6b00005f 	cmp	w2, w0
ffffc020acb4: 54000820 	b.eq	0xffffc020adb8 <ivshmem_init+0x1a8>
ffffc020acb8: f940a673 	ldr	x19, [x19, #328]
ffffc020acbc: 17fffff2 	b	0xffffc020ac84 <ivshmem_init+0x74>
ffffc020acc0: 9a9f0000 	csel	x0, x0, xzr, eq
ffffc020acc4: 52800406 	mov	w6, #32
ffffc020acc8: 8b000260 	add	x0, x19, x0
ffffc020accc: f9403c00 	ldr	x0, [x0, #120]
ffffc020acd0: a9400804 	ldp	x4, x2, [x0]
ffffc020acd4: f9485042 	ldr	x2, [x2, #4256]
ffffc020acd8: b9403080 	ldr	w0, [x4, #48]
ffffc020acdc: b9403045 	ldr	w5, [x2, #48]
ffffc020ace0: 9ba61400 	umaddl	x0, w0, w6, x5
ffffc020ace4: 91020000 	add	x0, x0, #128
ffffc020ace8: 8b000045 	add	x5, x2, x0
ffffc020acec: f8606846 	ldr	x6, [x2, x0]
ffffc020acf0: f8636820 	ldr	x0, [x1, x3]
ffffc020acf4: eb0000df 	cmp	x6, x0
ffffc020acf8: 540005c1 	b.ne	0xffffc020adb0 <ivshmem_init+0x1a0>
ffffc020acfc: f94008a3 	ldr	x3, [x5, #16]
ffffc020ad00: f9400b00 	ldr	x0, [x24, #16]
ffffc020ad04: eb00007f 	cmp	x3, x0
ffffc020ad08: 54000541 	b.ne	0xffffc020adb0 <ivshmem_init+0x1a0>
ffffc020ad0c: 79406883 	ldrh	w3, [x4, #52]
ffffc020ad10: 79406aa0 	ldrh	w0, [x21, #52]
ffffc020ad14: 6b00007f 	cmp	w3, w0
ffffc020ad18: 540004c1 	b.ne	0xffffc020adb0 <ivshmem_init+0x1a0>
ffffc020ad1c: d0000000 	adrp	x0, 0xffffc020c000 <ivshmem_init+0x114>
ffffc020ad20: 91002042 	add	x2, x2, #8
ffffc020ad24: 91002021 	add	x1, x1, #8
ffffc020ad28: 91092800 	add	x0, x0, #586
ffffc020ad2c: 97ffef7e 	bl	0xffffc0206b24 <printk>
ffffc020ad30: 2a1703e0 	mov	w0, w23
ffffc020ad34: d2801405 	mov	x5, #160
ffffc020ad38: 520002e1 	eor	w1, w23, #0x1
ffffc020ad3c: 52801404 	mov	w4, #160
ffffc020ad40: aa0103e2 	mov	x2, x1
ffffc020ad44: 9b054c00 	madd	x0, x0, x5, x19
ffffc020ad48: 9b054c21 	madd	x1, x1, x5, x19
ffffc020ad4c: 9ba44ee3 	umaddl	x3, w23, w4, x19
ffffc020ad50: b9005c17 	str	w23, [x0, #92]
ffffc020ad54: a907e016 	stp	x22, x24, [x0, #120]
ffffc020ad58: f90022c3 	str	x3, [x22, #64]
ffffc020ad5c: f9403c25 	ldr	x5, [x1, #120]
ffffc020ad60: b4000085 	cbz	x5, 0xffffc020ad70 <ivshmem_init+0x160>
ffffc020ad64: 9ba44c53 	umaddl	x19, w2, w4, x19
ffffc020ad68: f9004413 	str	x19, [x0, #136]
ffffc020ad6c: f9004423 	str	x3, [x1, #136]
ffffc020ad70: f90006d4 	str	x20, [x22, #8]
ffffc020ad74: aa1603e0 	mov	x0, x22
ffffc020ad78: 97fffc5e 	bl	0xffffc0209ef0 <pci_reset_device>
ffffc020ad7c: 52800000 	mov	w0, #0
ffffc020ad80: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020ad84: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020ad88: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc020ad8c: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc020ad90: d65f03c0 	ret
ffffc020ad94: 79400aa0 	ldrh	w0, [x21, #4]
ffffc020ad98: 79028260 	strh	w0, [x19, #320]
ffffc020ad9c: f94012e0 	ldr	x0, [x23, #32]
ffffc020ada0: f900a660 	str	x0, [x19, #328]
ffffc020ada4: f90012f3 	str	x19, [x23, #32]
ffffc020ada8: 52800017 	mov	w23, #0
ffffc020adac: 17ffffe1 	b	0xffffc020ad30 <ivshmem_init+0x120>
ffffc020adb0: 128002a0 	mov	w0, #-22
ffffc020adb4: 17fffff3 	b	0xffffc020ad80 <ivshmem_init+0x170>
ffffc020adb8: f9403e64 	ldr	x4, [x19, #120]
ffffc020adbc: d2801400 	mov	x0, #160
ffffc020adc0: f100009f 	cmp	x4, #0
ffffc020adc4: 9a9f1002 	csel	x2, x0, xzr, ne
ffffc020adc8: 1a9f07f7 	cset	w23, ne
ffffc020adcc: 8b020262 	add	x2, x19, x2
ffffc020add0: f9403c42 	ldr	x2, [x2, #120]
ffffc020add4: b4fff762 	cbz	x2, 0xffffc020acc0 <ivshmem_init+0xb0>
ffffc020add8: 128001e0 	mov	w0, #-16
ffffc020addc: 17ffffe9 	b	0xffffc020ad80 <ivshmem_init+0x170>

0000ffffc020ade0 <ivshmem_reset>:
ffffc020ade0: a9bd7bfd 	stp	x29, x30, [sp, #-48]!
ffffc020ade4: 910003fd 	mov	x29, sp
ffffc020ade8: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020adec: aa0003f4 	mov	x20, x0
ffffc020adf0: f90013f5 	str	x21, [sp, #32]
ffffc020adf4: f9402013 	ldr	x19, [x0, #64]
ffffc020adf8: b9400660 	ldr	w0, [x19, #4]
ffffc020adfc: 360800e0 	tbz	w0, #1, 0xffffc020ae18 <ivshmem_reset+0x38>
ffffc020ae00: f9403661 	ldr	x1, [x19, #104]
ffffc020ae04: f9400680 	ldr	x0, [x20, #8]
ffffc020ae08: 97fff8e0 	bl	0xffffc0209188 <mmio_region_unregister>
ffffc020ae0c: f9403a61 	ldr	x1, [x19, #112]
ffffc020ae10: f9400680 	ldr	x0, [x20, #8]
ffffc020ae14: 97fff8dd 	bl	0xffffc0209188 <mmio_region_unregister>
ffffc020ae18: 91005280 	add	x0, x20, #20
ffffc020ae1c: 52800095 	mov	w21, #4
ffffc020ae20: d2800b82 	mov	x2, #92
ffffc020ae24: b0000001 	adrp	x1, 0xffffc020b000 <ivshmem_reset+0x48>
ffffc020ae28: 912c6021 	add	x1, x1, #2840
ffffc020ae2c: a9007c1f 	stp	xzr, xzr, [x0]
ffffc020ae30: aa1303e0 	mov	x0, x19
ffffc020ae34: b9001295 	str	w21, [x20, #16]
ffffc020ae38: b900269f 	str	wzr, [x20, #36]
ffffc020ae3c: b9003a9f 	str	wzr, [x20, #56]
ffffc020ae40: 97fff810 	bl	0xffffc0208e80 <memcpy>
ffffc020ae44: f9400280 	ldr	x0, [x20]
ffffc020ae48: 79406800 	ldrh	w0, [x0, #52]
ffffc020ae4c: 53185c00 	lsl	w0, w0, #8
ffffc020ae50: 32081c00 	orr	w0, w0, #0xff000000
ffffc020ae54: b9000a60 	str	w0, [x19, #8]
ffffc020ae58: f9400280 	ldr	x0, [x20]
ffffc020ae5c: 79404801 	ldrh	w1, [x0, #36]
ffffc020ae60: 350002a1 	cbnz	w1, 0xffffc020aeb4 <ivshmem_reset+0xd4>
ffffc020ae64: 79400800 	ldrh	w0, [x0, #4]
ffffc020ae68: b900367f 	str	wzr, [x19, #52]
ffffc020ae6c: d3431000 	ubfx	x0, x0, #3, #2
ffffc020ae70: 11000400 	add	w0, w0, #1
ffffc020ae74: 53185c00 	lsl	w0, w0, #8
ffffc020ae78: b9003e60 	str	w0, [x19, #60]
ffffc020ae7c: f9404260 	ldr	x0, [x19, #128]
ffffc020ae80: f9400401 	ldr	x1, [x0, #8]
ffffc020ae84: b9004261 	str	w1, [x19, #64]
ffffc020ae88: b9400c01 	ldr	w1, [x0, #12]
ffffc020ae8c: b9004661 	str	w1, [x19, #68]
ffffc020ae90: f9400801 	ldr	x1, [x0, #16]
ffffc020ae94: b9004a61 	str	w1, [x19, #72]
ffffc020ae98: b9401400 	ldr	w0, [x0, #20]
ffffc020ae9c: b9004e60 	str	w0, [x19, #76]
ffffc020aea0: b900627f 	str	wzr, [x19, #96]
ffffc020aea4: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020aea8: f94013f5 	ldr	x21, [sp, #32]
ffffc020aeac: a8c37bfd 	ldp	x29, x30, [sp], #48
ffffc020aeb0: d65f03c0 	ret
ffffc020aeb4: b9002295 	str	w21, [x20, #32]
ffffc020aeb8: 17fffff1 	b	0xffffc020ae7c <ivshmem_reset+0x9c>

0000ffffc020aebc <ivshmem_exit>:
ffffc020aebc: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc020aec0: 910003fd 	mov	x29, sp
ffffc020aec4: f9000bf3 	str	x19, [sp, #16]
ffffc020aec8: f9402013 	ldr	x19, [x0, #64]
ffffc020aecc: f9404664 	ldr	x4, [x19, #136]
ffffc020aed0: b5000244 	cbnz	x4, 0xffffc020af18 <ivshmem_exit+0x5c>
ffffc020aed4: d0000080 	adrp	x0, 0xffffc021c000 <ivshmem_exit+0x60>
ffffc020aed8: 52801404 	mov	w4, #160
ffffc020aedc: f9401001 	ldr	x1, [x0, #32]
ffffc020aee0: 91008000 	add	x0, x0, #32
ffffc020aee4: b40002e1 	cbz	x1, 0xffffc020af40 <ivshmem_exit+0x84>
ffffc020aee8: b9405e62 	ldr	w2, [x19, #92]
ffffc020aeec: f940a423 	ldr	x3, [x1, #328]
ffffc020aef0: 9ba40442 	umaddl	x2, w2, w4, x1
ffffc020aef4: eb02027f 	cmp	x19, x2
ffffc020aef8: 540002a1 	b.ne	0xffffc020af4c <ivshmem_exit+0x90>
ffffc020aefc: f9000003 	str	x3, [x0]
ffffc020af00: 52800022 	mov	w2, #1
ffffc020af04: d0000060 	adrp	x0, 0xffffc0218000 <ivshmem_exit+0x80>
ffffc020af08: 91004000 	add	x0, x0, #16
ffffc020af0c: f9400bf3 	ldr	x19, [sp, #16]
ffffc020af10: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020af14: 17ffefdd 	b	0xffffc0206e88 <page_free>
ffffc020af18: 91024080 	add	x0, x4, #144
ffffc020af1c: 97fffe46 	bl	0xffffc020a834 <spin_lock>
ffffc020af20: 79412080 	ldrh	w0, [x4, #144]
ffffc020af24: 91024081 	add	x1, x4, #144
ffffc020af28: f900449f 	str	xzr, [x4, #136]
ffffc020af2c: 11000400 	add	w0, w0, #1
ffffc020af30: 489ffc20 	stlrh	w0, [x1]
ffffc020af34: aa1303e0 	mov	x0, x19
ffffc020af38: 97fffe72 	bl	0xffffc020a900 <ivshmem_remote_interrupt>
ffffc020af3c: f9003e7f 	str	xzr, [x19, #120]
ffffc020af40: f9400bf3 	ldr	x19, [sp, #16]
ffffc020af44: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020af48: d65f03c0 	ret
ffffc020af4c: 91052020 	add	x0, x1, #328
ffffc020af50: aa0303e1 	mov	x1, x3
ffffc020af54: 17ffffe4 	b	0xffffc020aee4 <ivshmem_exit+0x28>

0000ffffc020af58 <uart_write>:
ffffc020af58: a9bc7bfd 	stp	x29, x30, [sp, #-64]!
ffffc020af5c: 910003fd 	mov	x29, sp
ffffc020af60: a9025bf5 	stp	x21, x22, [sp, #32]
ffffc020af64: d0000095 	adrp	x21, 0xffffc021c000 <uart_write+0x54>
ffffc020af68: a90363f7 	stp	x23, x24, [sp, #48]
ffffc020af6c: aa1503f7 	mov	x23, x21
ffffc020af70: a90153f3 	stp	x19, x20, [sp, #16]
ffffc020af74: f0000078 	adrp	x24, 0xffffc0219000 <uart_write+0x58>
ffffc020af78: 52800013 	mov	w19, #0
ffffc020af7c: 90000036 	adrp	x22, 0xffffc020e000 <uart_write+0x34>
ffffc020af80: 71002a7f 	cmp	w19, #10
ffffc020af84: 540002a0 	b.eq	0xffffc020afd8 <uart_write+0x80>
ffffc020af88: 39400013 	ldrb	w19, [x0]
ffffc020af8c: 91000414 	add	x20, x0, #1
ffffc020af90: 350000d3 	cbnz	w19, 0xffffc020afa8 <uart_write+0x50>
ffffc020af94: a94153f3 	ldp	x19, x20, [sp, #16]
ffffc020af98: a9425bf5 	ldp	x21, x22, [sp, #32]
ffffc020af9c: a94363f7 	ldp	x23, x24, [sp, #48]
ffffc020afa0: a8c47bfd 	ldp	x29, x30, [sp], #64
ffffc020afa4: d65f03c0 	ret
ffffc020afa8: f94016a0 	ldr	x0, [x21, #40]
ffffc020afac: f9401401 	ldr	x1, [x0, #40]
ffffc020afb0: d63f0020 	blr	x1
ffffc020afb4: 35ffffa0 	cbnz	w0, 0xffffc020afa8 <uart_write+0x50>
ffffc020afb8: f9400f00 	ldr	x0, [x24, #24]
ffffc020afbc: b5000140 	cbnz	x0, 0xffffc020afe4 <uart_write+0x8c>
ffffc020afc0: f94016e0 	ldr	x0, [x23, #40]
ffffc020afc4: 2a1303e1 	mov	w1, w19
ffffc020afc8: f9401802 	ldr	x2, [x0, #48]
ffffc020afcc: d63f0040 	blr	x2
ffffc020afd0: aa1403e0 	mov	x0, x20
ffffc020afd4: 17ffffeb 	b	0xffffc020af80 <uart_write+0x28>
ffffc020afd8: aa0003f4 	mov	x20, x0
ffffc020afdc: 528001b3 	mov	w19, #13
ffffc020afe0: 17fffff2 	b	0xffffc020afa8 <uart_write+0x50>
ffffc020afe4: 97ffdf83 	bl	0xffffc0202df0 <phys_processor_id>
ffffc020afe8: f940cac1 	ldr	x1, [x22, #400]
ffffc020afec: eb01001f 	cmp	x0, x1
ffffc020aff0: 54fffe80 	b.eq	0xffffc020afc0 <uart_write+0x68>
ffffc020aff4: 17ffffe8 	b	0xffffc020af94 <uart_write+0x3c>

0000ffffc020aff8 <reg_out_mmio8>:
ffffc020aff8: 2a0103e1 	mov	w1, w1
ffffc020affc: f9400000 	ldr	x0, [x0]
ffffc020b000: 12001c42 	and	w2, w2, #0xff
ffffc020b004: 38216802 	strb	w2, [x0, x1]
ffffc020b008: d65f03c0 	ret

0000ffffc020b00c <reg_in_mmio8>:
ffffc020b00c: 2a0103e1 	mov	w1, w1
ffffc020b010: f9400000 	ldr	x0, [x0]
ffffc020b014: 38616800 	ldrb	w0, [x0, x1]
ffffc020b018: 12001c00 	and	w0, w0, #0xff
ffffc020b01c: d65f03c0 	ret

0000ffffc020b020 <reg_out_mmio32>:
ffffc020b020: 531e7421 	lsl	w1, w1, #2
ffffc020b024: f9400000 	ldr	x0, [x0]
ffffc020b028: b8216802 	str	w2, [x0, x1]
ffffc020b02c: d65f03c0 	ret

0000ffffc020b030 <reg_in_mmio32>:
ffffc020b030: 531e7421 	lsl	w1, w1, #2
ffffc020b034: f9400000 	ldr	x0, [x0]
ffffc020b038: b8616800 	ldr	w0, [x0, x1]
ffffc020b03c: d65f03c0 	ret

0000ffffc020b040 <uart_init>:
ffffc020b040: d0000061 	adrp	x1, 0xffffc0219000 <uart_init+0x38>
ffffc020b044: f9400021 	ldr	x1, [x1]
ffffc020b048: 79407421 	ldrh	w1, [x1, #58]
ffffc020b04c: 12000421 	and	w1, w1, #0x3
ffffc020b050: 7100043f 	cmp	w1, #1
ffffc020b054: 540000e1 	b.ne	0xffffc020b070 <uart_init+0x30>
ffffc020b058: f0ffffe1 	adrp	x1, 0xffffc020a000 <uart_init+0x14>
ffffc020b05c: 913fe021 	add	x1, x1, #4088
ffffc020b060: f9000801 	str	x1, [x0, #16]
ffffc020b064: 90000001 	adrp	x1, 0xffffc020b000 <uart_init+0x24>
ffffc020b068: 91003021 	add	x1, x1, #12
ffffc020b06c: f9000c01 	str	x1, [x0, #24]
ffffc020b070: f9400401 	ldr	x1, [x0, #8]
ffffc020b074: b9401021 	ldr	w1, [x1, #16]
ffffc020b078: 34000341 	cbz	w1, 0xffffc020b0e0 <uart_init+0xa0>
ffffc020b07c: a9be7bfd 	stp	x29, x30, [sp, #-32]!
ffffc020b080: 52801002 	mov	w2, #128
ffffc020b084: 52800061 	mov	w1, #3
ffffc020b088: 910003fd 	mov	x29, sp
ffffc020b08c: f9000bf3 	str	x19, [sp, #16]
ffffc020b090: aa0003f3 	mov	x19, x0
ffffc020b094: f9400803 	ldr	x3, [x0, #16]
ffffc020b098: d63f0060 	blr	x3
ffffc020b09c: a9408e60 	ldp	x0, x3, [x19, #8]
ffffc020b0a0: 52800001 	mov	w1, #0
ffffc020b0a4: 39404002 	ldrb	w2, [x0, #16]
ffffc020b0a8: aa1303e0 	mov	x0, x19
ffffc020b0ac: d63f0060 	blr	x3
ffffc020b0b0: a9408e60 	ldp	x0, x3, [x19, #8]
ffffc020b0b4: 52800021 	mov	w1, #1
ffffc020b0b8: 39404402 	ldrb	w2, [x0, #17]
ffffc020b0bc: aa1303e0 	mov	x0, x19
ffffc020b0c0: d63f0060 	blr	x3
ffffc020b0c4: aa1303e0 	mov	x0, x19
ffffc020b0c8: f9400a63 	ldr	x3, [x19, #16]
ffffc020b0cc: f9400bf3 	ldr	x19, [sp, #16]
ffffc020b0d0: 52800062 	mov	w2, #3
ffffc020b0d4: a8c27bfd 	ldp	x29, x30, [sp], #32
ffffc020b0d8: 2a0203e1 	mov	w1, w2
ffffc020b0dc: d61f0060 	br	x3
ffffc020b0e0: d65f03c0 	ret

0000ffffc020b0e4 <uart_is_busy>:
ffffc020b0e4: a9bf7bfd 	stp	x29, x30, [sp, #-16]!
ffffc020b0e8: 528000a1 	mov	w1, #5
ffffc020b0ec: 910003fd 	mov	x29, sp
ffffc020b0f0: f9400c02 	ldr	x2, [x0, #24]
ffffc020b0f4: d63f0040 	blr	x2
ffffc020b0f8: d27b0000 	eor	x0, x0, #0x20
ffffc020b0fc: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc020b100: 53051400 	ubfx	w0, w0, #5, #1
ffffc020b104: d65f03c0 	ret

0000ffffc020b108 <uart_write_char>:
ffffc020b108: f9400803 	ldr	x3, [x0, #16]
ffffc020b10c: 12001c22 	and	w2, w1, #0xff
ffffc020b110: 52800001 	mov	w1, #0
ffffc020b114: d61f0060 	br	x3

Disassembly of section .trampoline:

0000ffffc0213000 <enable_mmu_el2>:
ffffc0213000: 58000541 	ldr	x1, 0xffffc02130a8 <$d>
ffffc0213004: d51ca201 	msr	MAIR_EL2, x1
ffffc0213008: 58000541 	ldr	x1, 0xffffc02130b0 <$d+0x8>
ffffc021300c: d51c2041 	msr	TCR_EL2, x1
ffffc0213010: d51c2000 	msr	TTBR0_EL2, x0
ffffc0213014: d5033fdf 	isb
ffffc0213018: d50c871f 	tlbi	alle2
ffffc021301c: d503379f 	dsb	nsh
ffffc0213020: 580004c1 	ldr	x1, 0xffffc02130b8 <$d+0x10>
ffffc0213024: d51c1001 	msr	SCTLR_EL2, x1
ffffc0213028: d5033fdf 	isb
ffffc021302c: d50c871f 	tlbi	alle2
ffffc0213030: d503379f 	dsb	nsh
ffffc0213034: d65f03c0 	ret

0000ffffc0213038 <shutdown_el2>:
ffffc0213038: d53c1001 	mrs	x1, SCTLR_EL2
ffffc021303c: 58000422 	ldr	x2, 0xffffc02130c0 <$d+0x18>
ffffc0213040: 8a220021 	bic	x1, x1, x2
ffffc0213044: d51c1001 	msr	SCTLR_EL2, x1
ffffc0213048: d5033fdf 	isb
ffffc021304c: d51ca21f 	msr	MAIR_EL2, xzr
ffffc0213050: d51c201f 	msr	TTBR0_EL2, xzr
ffffc0213054: d51c205f 	msr	TCR_EL2, xzr
ffffc0213058: d5033fdf 	isb
ffffc021305c: 913c0000 	add	x0, x0, #3840

0000ffffc0213060 <vmreturn>:
ffffc0213060: 9100001f 	mov	sp, x0
ffffc0213064: a8c103e1 	ldp	x1, x0, [sp], #16
ffffc0213068: a8c10be1 	ldp	x1, x2, [sp], #16
ffffc021306c: a8c113e3 	ldp	x3, x4, [sp], #16
ffffc0213070: a8c11be5 	ldp	x5, x6, [sp], #16
ffffc0213074: a8c123e7 	ldp	x7, x8, [sp], #16
ffffc0213078: a8c12be9 	ldp	x9, x10, [sp], #16
ffffc021307c: a8c133eb 	ldp	x11, x12, [sp], #16
ffffc0213080: a8c13bed 	ldp	x13, x14, [sp], #16
ffffc0213084: a8c143ef 	ldp	x15, x16, [sp], #16
ffffc0213088: a8c14bf1 	ldp	x17, x18, [sp], #16
ffffc021308c: a8c153f3 	ldp	x19, x20, [sp], #16
ffffc0213090: a8c15bf5 	ldp	x21, x22, [sp], #16
ffffc0213094: a8c163f7 	ldp	x23, x24, [sp], #16
ffffc0213098: a8c16bf9 	ldp	x25, x26, [sp], #16
ffffc021309c: a8c173fb 	ldp	x27, x28, [sp], #16
ffffc02130a0: a8c17bfd 	ldp	x29, x30, [sp], #16
ffffc02130a4: d69f03e0 	eret

0000ffffc02130a8 <$d>:
ffffc02130a8: ff 04 44 00      	.word	0x004404ff
ffffc02130ac: 00 00 00 00      	.word	0x00000000
ffffc02130b0: 10 35 85 80      	.word	0x80853510
ffffc02130b4: 00 00 00 00      	.word	0x00000000
ffffc02130b8: 35 18 c5 30      	.word	0x30c51835
ffffc02130bc: 00 00 00 00      	.word	0x00000000
ffffc02130c0: 05 10 00 00      	.word	0x00001005
ffffc02130c4: 00 00 00 00      	.word	0x00000000
