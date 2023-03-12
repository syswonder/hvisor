
target/aarch64-unknown-linux-gnu/release/armv8-baremetal-demo-rust：     文件格式 elf64-littleaarch64


Disassembly of section .text.boot:

0000000040000000 <_start>:
    40000000:	5800009e 	ldr	x30, 40000010 <_start+0x10>
    40000004:	910003df 	mov	sp, x30
    40000008:	94000004 	bl	40000018 <not_main>
    4000000c:	14000000 	b	4000000c <_start+0xc>
    40000010:	400042b0 	.inst	0x400042b0 ; undefined
    40000014:	00000000 	udf	#0

Disassembly of section .text.not_main:

0000000040000018 <not_main>:
    40000018:	52a12008 	mov	w8, #0x9000000             	// #150994944
    4000001c:	52800829 	mov	w9, #0x41                  	// #65
    40000020:	52800e4a 	mov	w10, #0x72                  	// #114
    40000024:	52800c6b 	mov	w11, #0x63                  	// #99
    40000028:	52800d0c 	mov	w12, #0x68                  	// #104
    4000002c:	39000109 	strb	w9, [x8]
    40000030:	39000109 	strb	w9, [x8]
    40000034:	528006c9 	mov	w9, #0x36                  	// #54
    40000038:	3900010a 	strb	w10, [x8]
    4000003c:	3900010b 	strb	w11, [x8]
    40000040:	5280068b 	mov	w11, #0x34                  	// #52
    40000044:	3900010c 	strb	w12, [x8]
    40000048:	5280040c 	mov	w12, #0x20                  	// #32
    4000004c:	39000109 	strb	w9, [x8]
    40000050:	52800849 	mov	w9, #0x42                  	// #66
    40000054:	3900010b 	strb	w11, [x8]
    40000058:	52800c2b 	mov	w11, #0x61                  	// #97
    4000005c:	3900010c 	strb	w12, [x8]
    40000060:	39000109 	strb	w9, [x8]
    40000064:	52800ca9 	mov	w9, #0x65                  	// #101
    40000068:	3900010b 	strb	w11, [x8]
    4000006c:	3900010a 	strb	w10, [x8]
    40000070:	528009aa 	mov	w10, #0x4d                  	// #77
    40000074:	39000109 	strb	w9, [x8]
    40000078:	3900010c 	strb	w12, [x8]
    4000007c:	52800e8c 	mov	w12, #0x74                  	// #116
    40000080:	3900010a 	strb	w10, [x8]
    40000084:	39000109 	strb	w9, [x8]
    40000088:	52800d89 	mov	w9, #0x6c                  	// #108
    4000008c:	3900010c 	strb	w12, [x8]
    40000090:	3900010b 	strb	w11, [x8]
    40000094:	39000109 	strb	w9, [x8]
    40000098:	d65f03c0 	ret

Disassembly of section .dynamic:

00000000400000a0 <.dynamic>:
    400000a0:	00000004 	udf	#4
    400000a4:	00000000 	udf	#0
    400000a8:	40000268 	.inst	0x40000268 ; undefined
    400000ac:	00000000 	udf	#0
    400000b0:	6ffffef5 	.inst	0x6ffffef5 ; undefined
    400000b4:	00000000 	udf	#0
    400000b8:	40000278 	.inst	0x40000278 ; undefined
    400000bc:	00000000 	udf	#0
    400000c0:	00000005 	udf	#5
    400000c4:	00000000 	udf	#0
    400000c8:	40000260 	.inst	0x40000260 ; undefined
    400000cc:	00000000 	udf	#0
    400000d0:	00000006 	udf	#6
    400000d4:	00000000 	udf	#0
    400000d8:	40000230 	.inst	0x40000230 ; undefined
    400000dc:	00000000 	udf	#0
    400000e0:	0000000a 	udf	#10
    400000e4:	00000000 	udf	#0
    400000e8:	00000001 	udf	#1
    400000ec:	00000000 	udf	#0
    400000f0:	0000000b 	udf	#11
    400000f4:	00000000 	udf	#0
    400000f8:	00000018 	udf	#24
    400000fc:	00000000 	udf	#0
    40000100:	00000015 	udf	#21
	...
    40000110:	00000007 	udf	#7
    40000114:	00000000 	udf	#0
    40000118:	40000298 	.inst	0x40000298 ; undefined
    4000011c:	00000000 	udf	#0
    40000120:	00000008 	udf	#8
    40000124:	00000000 	udf	#0
    40000128:	00000018 	udf	#24
    4000012c:	00000000 	udf	#0
    40000130:	00000009 	udf	#9
    40000134:	00000000 	udf	#0
    40000138:	00000018 	udf	#24
    4000013c:	00000000 	udf	#0
    40000140:	00000016 	udf	#22
	...
    40000150:	00000018 	udf	#24
	...
    40000160:	6ffffffb 	.inst	0x6ffffffb ; undefined
    40000164:	00000000 	udf	#0
    40000168:	08000001 	stxrb	w0, w1, [x0]
    4000016c:	00000000 	udf	#0
    40000170:	6ffffff9 	.inst	0x6ffffff9 ; undefined
    40000174:	00000000 	udf	#0
    40000178:	00000001 	udf	#1
	...

Disassembly of section .got:

00000000400001d0 <.got>:
    400001d0:	400000a0 	.inst	0x400000a0 ; undefined
    400001d4:	00000000 	udf	#0

Disassembly of section .got.plt:

00000000400001d8 <.got.plt>:
	...

Disassembly of section .interp:

00000000400001f0 <.interp>:
    400001f0:	62696c2f 	.inst	0x62696c2f ; undefined
    400001f4:	2d646c2f 	ldp	s15, s27, [x1, #-224]
    400001f8:	756e696c 	.inst	0x756e696c ; undefined
    400001fc:	61612d78 	.inst	0x61612d78 ; undefined
    40000200:	36686372 	tbz	w18, #13, 40000e6c <.got.plt+0xc94>
    40000204:	6f732e34 	.inst	0x6f732e34 ; undefined
    40000208:	地址 0x0000000040000208 越界。


Disassembly of section .note.gnu.build-id:

000000004000020c <.note.gnu.build-id>:
    4000020c:	00000004 	udf	#4
    40000210:	00000014 	udf	#20
    40000214:	00000003 	udf	#3
    40000218:	00554e47 	.inst	0x00554e47 ; undefined
    4000021c:	69564cac 	ldpsw	x12, x19, [x5, #176]
    40000220:	bc1bcaf1 	.inst	0xbc1bcaf1 ; undefined
    40000224:	80dce9da 	.inst	0x80dce9da ; undefined
    40000228:	d8a7c9ac 	prfm	plil3keep, 3ff4fb5c <_start-0xb04a4>
    4000022c:	5469648d 	b.le	400d2ebc <stack_top+0xcec0c>

Disassembly of section .dynsym:

0000000040000230 <.dynsym>:
	...
    4000024c:	00010003 	.inst	0x00010003 ; undefined
    40000250:	40000000 	.inst	0x40000000 ; undefined
	...

Disassembly of section .dynstr:

0000000040000260 <.dynstr>:
	...

Disassembly of section .hash:

0000000040000268 <.hash>:
    40000268:	00000000 	udf	#0
    4000026c:	00000002 	udf	#2
	...

Disassembly of section .gnu.hash:

0000000040000278 <.gnu.hash>:
    40000278:	00000001 	udf	#1
    4000027c:	00000001 	udf	#1
    40000280:	00000001 	udf	#1
	...

Disassembly of section .rela.dyn:

0000000040000298 <stack_top-0x4018>:
    40000298:	40000010 	.inst	0x40000010 ; undefined
    4000029c:	00000000 	udf	#0
    400002a0:	00000403 	udf	#1027
    400002a4:	00000000 	udf	#0
    400002a8:	400042b0 	.inst	0x400042b0 ; undefined
    400002ac:	00000000 	udf	#0
