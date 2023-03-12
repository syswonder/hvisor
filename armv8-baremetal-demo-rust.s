
target/aarch64-unknown-linux-gnu/debug/armv8-baremetal-demo-rust：     文件格式 elf64-littleaarch64


Disassembly of section .text.boot:

0000000040000000 <_start>:
    40000000:	5800009e 	ldr	x30, 40000010 <_start+0x10>
    40000004:	910003df 	mov	sp, x30
    40000008:	9400009a 	bl	40000270 <not_main>
    4000000c:	00000000 	udf	#0
    40000010:	40004528 	.inst	0x40004528 ; undefined
    40000014:	00000000 	udf	#0

Disassembly of section .text._ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E:

0000000040000018 <_ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E>:
    40000018:	d10303ff 	sub	sp, sp, #0xc0
    4000001c:	f90003e0 	str	x0, [sp]
    40000020:	f90007e1 	str	x1, [sp, #8]
    40000024:	f90027e0 	str	x0, [sp, #72]
    40000028:	f9002be1 	str	x1, [sp, #80]
    4000002c:	f90033e0 	str	x0, [sp, #96]
    40000030:	f9001be0 	str	x0, [sp, #48]
    40000034:	f9401be8 	ldr	x8, [sp, #48]
    40000038:	f90037e8 	str	x8, [sp, #104]
    4000003c:	f9003be8 	str	x8, [sp, #112]
    40000040:	2a1f03e8 	mov	w8, wzr
    40000044:	37000128 	tbnz	w8, #0, 40000068 <_ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E+0x50>
    40000048:	14000001 	b	4000004c <_ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E+0x34>
    4000004c:	f94003e8 	ldr	x8, [sp]
    40000050:	f94007e9 	ldr	x9, [sp, #8]
    40000054:	f90057e9 	str	x9, [sp, #168]
    40000058:	f9005be9 	str	x9, [sp, #176]
    4000005c:	8b090108 	add	x8, x8, x9
    40000060:	f90013e8 	str	x8, [sp, #32]
    40000064:	14000011 	b	400000a8 <_ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E+0x90>
    40000068:	f94003e8 	ldr	x8, [sp]
    4000006c:	f94007e9 	ldr	x9, [sp, #8]
    40000070:	f9003fe9 	str	x9, [sp, #120]
    40000074:	f90043e8 	str	x8, [sp, #128]
    40000078:	f90047e9 	str	x9, [sp, #136]
    4000007c:	8b090108 	add	x8, x8, x9
    40000080:	f9004be8 	str	x8, [sp, #144]
    40000084:	f9404be8 	ldr	x8, [sp, #144]
    40000088:	f9004fe8 	str	x8, [sp, #152]
    4000008c:	f90053e8 	str	x8, [sp, #160]
    40000090:	f90023e8 	str	x8, [sp, #64]
    40000094:	f94023e8 	ldr	x8, [sp, #64]
    40000098:	f9001fe8 	str	x8, [sp, #56]
    4000009c:	f9401fe8 	ldr	x8, [sp, #56]
    400000a0:	f90013e8 	str	x8, [sp, #32]
    400000a4:	14000001 	b	400000a8 <_ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E+0x90>
    400000a8:	f94003e8 	ldr	x8, [sp]
    400000ac:	f9005fe8 	str	x8, [sp, #184]
    400000b0:	f90017e8 	str	x8, [sp, #40]
    400000b4:	f94013e8 	ldr	x8, [sp, #32]
    400000b8:	f94017e9 	ldr	x9, [sp, #40]
    400000bc:	f9000fe9 	str	x9, [sp, #24]
    400000c0:	f9000be8 	str	x8, [sp, #16]
    400000c4:	f9400be0 	ldr	x0, [sp, #16]
    400000c8:	f9400fe1 	ldr	x1, [sp, #24]
    400000cc:	910303ff 	add	sp, sp, #0xc0
    400000d0:	d65f03c0 	ret

Disassembly of section .text._ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E:

00000000400000d4 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E>:
    400000d4:	d10483ff 	sub	sp, sp, #0x120
    400000d8:	f9008bfd 	str	x29, [sp, #272]
    400000dc:	f90007e0 	str	x0, [sp, #8]
    400000e0:	aa0003e8 	mov	x8, x0
    400000e4:	f90027e8 	str	x8, [sp, #72]
    400000e8:	f9400408 	ldr	x8, [x0, #8]
    400000ec:	f9002fe8 	str	x8, [sp, #88]
    400000f0:	f90033e8 	str	x8, [sp, #96]
    400000f4:	f90013e8 	str	x8, [sp, #32]
    400000f8:	f94013e8 	ldr	x8, [sp, #32]
    400000fc:	f90037e8 	str	x8, [sp, #104]
    40000100:	f9003be8 	str	x8, [sp, #112]
    40000104:	52800028 	mov	w8, #0x1                   	// #1
    40000108:	37000148 	tbnz	w8, #0, 40000130 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x5c>
    4000010c:	14000001 	b	40000110 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x3c>
    40000110:	f94007e9 	ldr	x9, [sp, #8]
    40000114:	f9400528 	ldr	x8, [x9, #8]
    40000118:	f9004be8 	str	x8, [sp, #144]
    4000011c:	f9400129 	ldr	x9, [x9]
    40000120:	eb090108 	subs	x8, x8, x9
    40000124:	1a9f17e8 	cset	w8, eq  // eq = none
    40000128:	370001e8 	tbnz	w8, #0, 40000164 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x90>
    4000012c:	14000009 	b	40000150 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x7c>
    40000130:	f94007e8 	ldr	x8, [sp, #8]
    40000134:	f9400108 	ldr	x8, [x8]
    40000138:	f9003fe8 	str	x8, [sp, #120]
    4000013c:	f90017e8 	str	x8, [sp, #40]
    40000140:	f94017e8 	ldr	x8, [sp, #40]
    40000144:	f90043e8 	str	x8, [sp, #128]
    40000148:	f90047e8 	str	x8, [sp, #136]
    4000014c:	17fffff1 	b	40000110 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x3c>
    40000150:	f94007e8 	ldr	x8, [sp, #8]
    40000154:	f9004fe8 	str	x8, [sp, #152]
    40000158:	2a1f03e8 	mov	w8, wzr
    4000015c:	37000108 	tbnz	w8, #0, 4000017c <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0xa8>
    40000160:	1400001b 	b	400001cc <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0xf8>
    40000164:	f9000bff 	str	xzr, [sp, #16]
    40000168:	14000001 	b	4000016c <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x98>
    4000016c:	f9400be0 	ldr	x0, [sp, #16]
    40000170:	f9408bfd 	ldr	x29, [sp, #272]
    40000174:	910483ff 	add	sp, sp, #0x120
    40000178:	d65f03c0 	ret
    4000017c:	f94007e8 	ldr	x8, [sp, #8]
    40000180:	f9400109 	ldr	x9, [x8]
    40000184:	f9006fe9 	str	x9, [sp, #216]
    40000188:	f90073e9 	str	x9, [sp, #224]
    4000018c:	9280000a 	mov	x10, #0xffffffffffffffff    	// #-1
    40000190:	f90077ea 	str	x10, [sp, #232]
    40000194:	f1000529 	subs	x9, x9, #0x1
    40000198:	f9007be9 	str	x9, [sp, #240]
    4000019c:	f9407be9 	ldr	x9, [sp, #240]
    400001a0:	f9007fe9 	str	x9, [sp, #248]
    400001a4:	f90083e9 	str	x9, [sp, #256]
    400001a8:	f90023e9 	str	x9, [sp, #64]
    400001ac:	f94023e9 	ldr	x9, [sp, #64]
    400001b0:	f9001fe9 	str	x9, [sp, #56]
    400001b4:	f9401fe9 	ldr	x9, [sp, #56]
    400001b8:	f9000109 	str	x9, [x8]
    400001bc:	f9400508 	ldr	x8, [x8, #8]
    400001c0:	f90087e8 	str	x8, [sp, #264]
    400001c4:	f9000fe8 	str	x8, [sp, #24]
    400001c8:	14000013 	b	40000214 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x140>
    400001cc:	f94007ea 	ldr	x10, [sp, #8]
    400001d0:	f9400548 	ldr	x8, [x10, #8]
    400001d4:	f90053e8 	str	x8, [sp, #160]
    400001d8:	f90057e8 	str	x8, [sp, #168]
    400001dc:	f9400549 	ldr	x9, [x10, #8]
    400001e0:	f9005be9 	str	x9, [sp, #176]
    400001e4:	f9005fe9 	str	x9, [sp, #184]
    400001e8:	5280002b 	mov	w11, #0x1                   	// #1
    400001ec:	f90063eb 	str	x11, [sp, #192]
    400001f0:	91000529 	add	x9, x9, #0x1
    400001f4:	f90067e9 	str	x9, [sp, #200]
    400001f8:	f94067e9 	ldr	x9, [sp, #200]
    400001fc:	f9006be9 	str	x9, [sp, #208]
    40000200:	f9001be9 	str	x9, [sp, #48]
    40000204:	f9401be9 	ldr	x9, [sp, #48]
    40000208:	f9000549 	str	x9, [x10, #8]
    4000020c:	f9000fe8 	str	x8, [sp, #24]
    40000210:	14000001 	b	40000214 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x140>
    40000214:	f9400fe8 	ldr	x8, [sp, #24]
    40000218:	f9000be8 	str	x8, [sp, #16]
    4000021c:	17ffffd4 	b	4000016c <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E+0x98>

Disassembly of section .text._ZN4core3ptr14write_volatile17h13aa5119cc79ba29E:

0000000040000220 <_ZN4core3ptr14write_volatile17h13aa5119cc79ba29E>:
    40000220:	d10043ff 	sub	sp, sp, #0x10
    40000224:	aa0003e8 	mov	x8, x0
    40000228:	f90003e8 	str	x8, [sp]
    4000022c:	39003fe1 	strb	w1, [sp, #15]
    40000230:	39000001 	strb	w14000023c, [x0]
    40000234:	910043ff 	add	sp, sp, #0x10
    40000238:	d65f03c0 	ret

Disassembly of section .text._ZN4core5array98_$LT$impl$u20$core..iter..traits..collect..IntoIterator$u20$for$u20$$RF$$u5b$T$u3b$$u20$N$u5d$$GT$9into_iter17hea5b6789b0385f99E:

000000004000023c <_ZN4core5array98_$LT$impl$u20$core..iter..traits..collect..IntoIterator$u20$for$u20$$RF$$u5b$T$u3b$$u20$N$u5d$$GT$9into_iter17hea5b6789b0385f99E>:
    4000023c:	d100c3ff 	sub	sp, sp, #0x30
    40000240:	f90013fe 	str	x30, [sp, #32]
    40000244:	aa0003e8 	mov	x8, x0
    40000248:	f90007e8 	str	x8, [sp, #8]
    4000024c:	aa0003e8 	mov	x8, x0
    40000250:	f9000be8 	str	x8, [sp, #16]
    40000254:	52800368 	mov	w8, #0x1b                  	// #27
    40000258:	2a0803e1 	mov	w1, w8
    4000025c:	f9000fe1 	str	x1, [sp, #24]
    40000260:	97ffff6e 	bl	40000018 <_ZN4core5slice4iter13Iter$LT$T$GT$3new17h4f998d9a04b0b8e7E>
    40000264:	f94013fe 	ldr	x30, [sp, #32]
    40000268:	9100c3ff 	add	sp, sp, #0x30
    4000026c:	d65f03c0 	ret

Disassembly of section .text.not_main:

0000000040000270 <not_main>:
    40000270:	d10103ff 	sub	sp, sp, #0x40 // #0x40 == 0x0100_0000 == 8Byte, 栈指针sp寄存器上移8字节
    40000274:	f9001bfe 	str	x30, [sp, #48] // 将x30中的字数据写入sp+48的位置，x30存储的是栈顶
    40000278:	90000000 	adrp	x0, 40000000 <_start>
    4000027c:	9113d000 	add	x0, x0, #0x4f4
    40000280:	aa0003e8 	mov	x8, x0
    40000284:	f90013e8 	str	x8, [sp, #32]
    40000288:	97ffffed 	bl	4000023c <_ZN4core5array98_$LT$impl$u20$core..iter..traits..collect..IntoIterator$u20$for$u20$$RF$$u5b$T$u3b$$u20$N$u5d$$GT$9into_iter17hea5b6789b0385f99E>
    4000028c:	f90007e0 	str	x0, [sp, #8]
    40000290:	f9000be1 	str	x1, [sp, #16]
    40000294:	14000001 	b	40000298 <not_main+0x28>
    40000298:	910023e0 	add	x0, sp, #0x8
    4000029c:	97ffff8e 	bl	400000d4 <_ZN91_$LT$core..slice..iter..Iter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h64bad2637c595524E>
    400002a0:	f9000fe0 	str	x0, [sp, #24]
    400002a4:	f9400fe8 	ldr	x8, [sp, #24]
    400002a8:	f1000108 	subs	x8, x8, #0x0
    400002ac:	1a9f17e8 	cset	w8, eq  // eq = none
    400002b0:	12000108 	and	w8, w8, #0x1
    400002b4:	72000108 	ands	w8, w8, #0x1
    400002b8:	9a9f17e8 	cset	x8, eq  // eq = none
    400002bc:	f1000108 	subs	x8, x8, #0x0
    400002c0:	1a9f07e8 	cset	w8, ne  // ne = any
    400002c4:	370000a8 	tbnz	w8, #0, 400002d8 <not_main+0x68>
    400002c8:	14000001 	b	400002cc <not_main+0x5c>
    400002cc:	f9401bfe 	ldr	x30, [sp, #48]
    400002d0:	910103ff 	add	sp, sp, #0x40
    400002d4:	d65f03c0 	ret
    400002d8:	f9400fe8 	ldr	x8, [sp, #24]
    400002dc:	aa0803e9 	mov	x9, x8
    400002e0:	f90017e9 	str	x9, [sp, #40]
    400002e4:	39400101 	ldrb	w1, [x8]
    400002e8:	52a12008 	mov	w8, #0x9000000             	// #150994944
    400002ec:	2a0803e0 	mov	w0, w8
    400002f0:	97ffffcc 	bl	40000220 <_ZN4core3ptr14write_volatile17h13aa5119cc79ba29E>
    400002f4:	17ffffe9 	b	40000298 <not_main+0x28>

Disassembly of section .dynamic:

00000000400002f8 <.dynamic>:
    400002f8:	00000004 	udf	#4
    400002fc:	00000000 	udf	#0
    40000300:	400004c0 	.inst	0x400004c0 ; undefined
    40000304:	00000000 	udf	#0
    40000308:	6ffffef5 	.inst	0x6ffffef5 ; undefined
    4000030c:	00000000 	udf	#0
    40000310:	400004d8 	.inst	0x400004d8 ; undefined
    40000314:	00000000 	udf	#0
    40000318:	00000005 	udf	#5
    4000031c:	00000000 	udf	#0
    40000320:	400004b8 	.inst	0x400004b8 ; undefined
    40000324:	00000000 	udf	#0
    40000328:	00000006 	udf	#6
    4000032c:	00000000 	udf	#0
    40000330:	40000488 	.inst	0x40000488 ; undefined
    40000334:	00000000 	udf	#0
    40000338:	0000000a 	udf	#10
    4000033c:	00000000 	udf	#0
    40000340:	00000001 	udf	#1
    40000344:	00000000 	udf	#0
    40000348:	0000000b 	udf	#11
    4000034c:	00000000 	udf	#0
    40000350:	00000018 	udf	#24
    40000354:	00000000 	udf	#0
    40000358:	00000015 	udf	#21
	...
    40000368:	00000007 	udf	#7
    4000036c:	00000000 	udf	#0
    40000370:	40000510 	.inst	0x40000510 ; undefined
    40000374:	00000000 	udf	#0
    40000378:	00000008 	udf	#8
    4000037c:	00000000 	udf	#0
    40000380:	00000018 	udf	#24
    40000384:	00000000 	udf	#0
    40000388:	00000009 	udf	#9
    4000038c:	00000000 	udf	#0
    40000390:	00000018 	udf	#24
    40000394:	00000000 	udf	#0
    40000398:	00000016 	udf	#22
	...
    400003a8:	00000018 	udf	#24
	...
    400003b8:	6ffffffb 	.inst	0x6ffffffb ; undefined
    400003bc:	00000000 	udf	#0
    400003c0:	08000001 	stxrb	w0, w1, [x0]
    400003c4:	00000000 	udf	#0
    400003c8:	6ffffff9 	.inst	0x6ffffff9 ; undefined
    400003cc:	00000000 	udf	#0
    400003d0:	00000001 	udf	#1
	...

Disassembly of section .got:

0000000040000428 <.got>:
    40000428:	400002f8 	.inst	0x400002f8 ; undefined
    4000042c:	00000000 	udf	#0

Disassembly of section .got.plt:

0000000040000430 <.got.plt>:
	...

Disassembly of section .interp:

0000000040000448 <.interp>:
    40000448:	62696c2f 	.inst	0x62696c2f ; undefined
    4000044c:	2d646c2f 	ldp	s15, s27, [x1, #-224]
    40000450:	756e696c 	.inst	0x756e696c ; undefined
    40000454:	61612d78 	.inst	0x61612d78 ; undefined
    40000458:	36686372 	tbz	w18, #13, 400010c4 <.got.plt+0xc94>
    4000045c:	6f732e34 	.inst	0x6f732e34 ; undefined
    40000460:	地址 0x0000000040000460 越界。


Disassembly of section .note.gnu.build-id:

0000000040000464 <.note.gnu.build-id>:
    40000464:	00000004 	udf	#4
    40000468:	00000014 	udf	#20
    4000046c:	00000003 	udf	#3
    40000470:	00554e47 	.inst	0x00554e47 ; undefined
    40000474:	89c80f83 	.inst	0x89c80f83 ; undefined
    40000478:	82566662 	.inst	0x82566662 ; undefined
    4000047c:	142a3b38 	b	40a8f15c <stack_top+0xa8ac34>
    40000480:	5547f253 	.inst	0x5547f253 ; undefined
    40000484:	bf008e14 	.inst	0xbf008e14 ; undefined

Disassembly of section .dynsym:

0000000040000488 <.dynsym>:
	...
    400004a4:	00010003 	.inst	0x00010003 ; undefined
    400004a8:	40000000 	.inst	0x40000000 ; undefined
	...

Disassembly of section .dynstr:

00000000400004b8 <.dynstr>:
	...

Disassembly of section .hash:

00000000400004c0 <.hash>:
    400004c0:	00000001 	udf	#1
    400004c4:	00000002 	udf	#2
	...

Disassembly of section .gnu.hash:

00000000400004d8 <.gnu.hash>:
    400004d8:	00000001 	udf	#1
    400004dc:	00000001 	udf	#1
    400004e0:	00000001 	udf	#1
	...

Disassembly of section .rodata..L__unnamed_1:

00000000400004f4 <.rodata..L__unnamed_1>:
    400004f4:	63724141 	.inst	0x63724141 ; undefined
    400004f8:	20343668 	.inst	0x20343668 ; undefined
    400004fc:	65726142 	fnmls	z2.h, p0/m, z10.h, z18.h
    40000500:	74654d20 	.inst	0x74654d20 ; undefined
    40000504:	42206c61 	.inst	0x42206c61 ; undefined
    40000508:	6f432079 	umlal2	v25.4s, v3.8h, v3.h[0]
    4000050c:	地址 0x000000004000050c 越界。


Disassembly of section .rela.dyn:

0000000040000510 <stack_top-0x4018>:
    40000510:	40000010 	.inst	0x40000010 ; undefined
    40000514:	00000000 	udf	#0
    40000518:	00000403 	udf	#1027
    4000051c:	00000000 	udf	#0
    40000520:	40004528 	.inst	0x40004528 ; undefined
    40000524:	00000000 	udf	#0

Disassembly of section .debug_abbrev:

0000000000000000 <.debug_abbrev>:
   0:	25011101 	cmpge	p1.b, p4/z, z8.b, #1
   4:	0305130e 	.inst	0x0305130e ; undefined
   8:	1b17100e 	madd	w14, w0, w23, w4
   c:	1942b40e 	cpyfmtrn	[x14]!, [x2]!, x0!
  10:	17550111 	b	fffffffffd540454 <stack_top+0xffffffffbd53bf2c>
  14:	39020000 	strb	w0, [x0, #128]
  18:	000e0301 	.inst	0x000e0301 ; undefined
  1c:	012e0300 	.inst	0x012e0300 ; undefined
  20:	0e030e6e 	dup	v14.8b, w19
  24:	053b0b3a 	ext	z26.b, z26.b, z25.b, #218
  28:	0b201349 	add	w9, w26, w0, uxtb #4
  2c:	2f040000 	.inst	0x2f040000 ; undefined
  30:	03134900 	.inst	0x03134900 ; undefined
  34:	0500000e 	orr	z14.s, z14.s, #0x1
  38:	0e030034 	tbl	v20.8b, {v1.16b}, v3.8b
  3c:	3a0f0188 	adcs	w8, w12, w15
  40:	49053b0b 	.inst	0x49053b0b ; undefined
  44:	06000013 	.inst	0x06000013 ; undefined
  48:	0e030113 	tbl	v19.8b, {v8.16b}, v3.8b
  4c:	01880b0b 	.inst	0x01880b0b ; undefined
  50:	0700000f 	.inst	0x0700000f ; undefined
  54:	0e03000d 	tbl	v13.8b, {v0.16b}, v3.8b
  58:	01881349 	.inst	0x01881349 ; undefined
  5c:	000b380f 	.inst	0x000b380f ; undefined
  60:	012e0800 	.inst	0x012e0800 ; undefined
  64:	06120111 	.inst	0x06120111 ; undefined
  68:	0e6e1840 	.inst	0x0e6e1840 ; undefined
  6c:	0b3a0e03 	add	w3, w16, w26, uxtb #3
  70:	13490b3b 	.inst	0x13490b3b ; undefined
  74:	05090000 	.inst	0x05090000 ; undefined
  78:	03180200 	.inst	0x03180200 ; undefined
  7c:	3b0b3a0e 	.inst	0x3b0b3a0e ; undefined
  80:	0013490b 	.inst	0x0013490b ; undefined
  84:	011d0a00 	.inst	0x011d0a00 ; undefined
  88:	01111331 	.inst	0x01111331 ; undefined
  8c:	0b580612 	add	w18, w16, w24, lsr #1
  90:	0b570b59 	add	w25, w26, w23, lsr #2
  94:	340b0000 	cbz	w0, 16094 <_start-0x3ffe9f6c>
  98:	31180200 	adds	w0, w16, #0x600
  9c:	0c000013 	st4	{v19.8b-v22.8b}, [x0]
  a0:	0111010b 	.inst	0x0111010b ; undefined
  a4:	00000612 	udf	#1554
  a8:	0200340d 	.inst	0x0200340d ; undefined
  ac:	880e0318 	stxr	w14, w24, [x24]
  b0:	0b3a0f01 	add	w1, w24, w26, uxtb #3
  b4:	13490b3b 	.inst	0x13490b3b ; undefined
  b8:	1d0e0000 	.inst	0x1d0e0000 ; undefined
  bc:	11133101 	add	w1, w8, #0x4cc
  c0:	58061201 	ldr	x1, c300 <_start-0x3fff3d00>
  c4:	5705590b 	.inst	0x5705590b ; undefined
  c8:	0f00000b 	.inst	0x0f00000b ; undefined
  cc:	0e6e012e 	saddl	v14.4s, v9.4h, v14.4h
  d0:	0b3a0e03 	add	w3, w16, w26, uxtb #3
  d4:	13490b3b 	.inst	0x13490b3b ; undefined
  d8:	00000b20 	udf	#2848
  dc:	03003410 	.inst	0x03003410 ; undefined
  e0:	0f01880e 	.inst	0x0f01880e ; undefined
  e4:	0b3b0b3a 	add	w26, w25, w27, uxtb #2
  e8:	00001349 	udf	#4937
  ec:	00010b11 	.inst	0x00010b11 ; undefined
  f0:	011d1200 	.inst	0x011d1200 ; undefined
  f4:	17551331 	b	fffffffffd544db8 <stack_top+0xffffffffbd540890>
  f8:	0b590b58 	add	w24, w26, w25, lsr #2
  fc:	00000b57 	udf	#2903
 100:	31001d13 	adds	w19, w8, #0x7
 104:	12011113 	and	w19, w8, #0x8000000f
 108:	590b5806 	.inst	0x590b5806 ; undefined
 10c:	000b5705 	.inst	0x000b5705 ; undefined
 110:	00341400 	.inst	0x00341400 ; NYI
 114:	0b3a0e03 	add	w3, w16, w26, uxtb #3
 118:	13490b3b 	.inst	0x13490b3b ; undefined
 11c:	33150000 	bfi	w0, w0, #11, #1
 120:	00131501 	.inst	0x00131501 ; undefined
 124:	000d1600 	.inst	0x000d1600 ; undefined
 128:	01881349 	.inst	0x01881349 ; undefined
 12c:	340b380f 	cbz	w15, 1682c <_start-0x3ffe97d4>
 130:	17000019 	b	fffffffffc000194 <stack_top+0xffffffffbbffbc6c>
 134:	0b160119 	add	w25, w8, w22
 138:	19180000 	stlurb	w0, [x0, #-128]
 13c:	19000001 	stlurb	w1, [x0]
 140:	0e030024 	tbl	v4.8b, {v1.16b}, v3.8b
 144:	0b0b0b3e 	add	w30, w25, w11, lsl #2
 148:	0f1a0000 	.inst	0x0f1a0000 ; undefined
 14c:	03134900 	.inst	0x03134900 ; undefined
 150:	0006330e 	.inst	0x0006330e ; undefined
 154:	000f1b00 	.inst	0x000f1b00 ; undefined
 158:	06331349 	.inst	0x06331349 ; undefined
 15c:	01000000 	.inst	0x01000000 ; undefined
 160:	0e250111 	saddl	v17.8h, v8.8b, v5.8b
 164:	0e030513 	dup	v19.8b, v8.b[1]
 168:	0e1b1710 	.inst	0x0e1b1710 ; undefined
 16c:	111942b4 	add	w20, w21, #0x650
 170:	00061201 	.inst	0x00061201 ; undefined
 174:	01390200 	.inst	0x01390200 ; undefined
 178:	00000e03 	udf	#3587
 17c:	11012e03 	add	w3, w16, #0x4b
 180:	40061201 	.inst	0x40061201 ; undefined
 184:	030e6e18 	.inst	0x030e6e18 ; undefined
 188:	3b0b3a0e 	.inst	0x3b0b3a0e ; undefined
 18c:	04000005 	add	z5.b, p0/m, z5.b, z0.b
 190:	18020005 	ldr	w5, 4190 <_start-0x3fffbe70>
 194:	0b3a0e03 	add	w3, w16, w26, uxtb #3
 198:	1349053b 	.inst	0x1349053b ; undefined
 19c:	2f050000 	.inst	0x2f050000 ; undefined
 1a0:	03134900 	.inst	0x03134900 ; undefined
 1a4:	0600000e 	.inst	0x0600000e ; undefined
 1a8:	0e030024 	tbl	v4.8b, {v1.16b}, v3.8b
 1ac:	0b0b0b3e 	add	w30, w25, w11, lsl #2
 1b0:	0f070000 	.inst	0x0f070000 ; undefined
 1b4:	03134900 	.inst	0x03134900 ; undefined
 1b8:	0006330e 	.inst	0x0006330e ; undefined
 1bc:	11010000 	add	w0, w0, #0x40
 1c0:	130e2501 	sbfiz	w1, w8, #18, #10
 1c4:	100e0305 	adr	x5, 1c224 <_start-0x3ffe3ddc>
 1c8:	b40e1b17 	cbz	x23, 1c528 <_start-0x3ffe3ad8>
 1cc:	01111942 	.inst	0x01111942 ; undefined
 1d0:	00000612 	udf	#1554
 1d4:	03013902 	.inst	0x03013902 ; undefined
 1d8:	0300000e 	.inst	0x0300000e ; undefined
 1dc:	0e6e012e 	saddl	v14.4s, v9.4h, v14.4h
 1e0:	0b3a0e03 	add	w3, w16, w26, uxtb #3
 1e4:	1349053b 	.inst	0x1349053b ; undefined
 1e8:	00000b20 	udf	#2848
 1ec:	49002f04 	.inst	0x49002f04 ; undefined
 1f0:	000e0313 	.inst	0x000e0313 ; undefined
 1f4:	00340500 	.inst	0x00340500 ; NYI
 1f8:	01880e03 	.inst	0x01880e03 ; undefined
 1fc:	3b0b3a0f 	.inst	0x3b0b3a0f ; undefined
 200:	00134905 	.inst	0x00134905 ; undefined
 204:	01130600 	.inst	0x01130600 ; undefined
 208:	0b0b0e03 	add	w3, w16, w11, lsl #3
 20c:	000f0188 	.inst	0x000f0188 ; undefined
 210:	000d0700 	.inst	0x000d0700 ; undefined
 214:	13490e03 	.inst	0x13490e03 ; undefined
 218:	380f0188 	sturb	w8, [x12, #240]
 21c:	0800000b 	stxrb	w0, w11, [x0]
 220:	0111012e 	.inst	0x0111012e ; undefined
 224:	18400612 	ldr	w18, 802e4 <_start-0x3ff7fd1c>
 228:	0e030e6e 	dup	v14.8b, w19
 22c:	053b0b3a 	ext	z26.b, z26.b, z25.b, #218
 230:	00001349 	udf	#4937
 234:	02000509 	.inst	0x02000509 ; undefined
 238:	3a0e0318 	adcs	w24, w24, w14
 23c:	49053b0b 	.inst	0x49053b0b ; undefined
 240:	0a000013 	and	w19, w0, w0
 244:	1331011d 	.inst	0x1331011d ; undefined
 248:	06120111 	.inst	0x06120111 ; undefined
 24c:	05590b58 	mov	z24.h, p9/z, #90
 250:	00000b57 	udf	#2903
 254:	0200340b 	.inst	0x0200340b ; undefined
 258:	00133118 	.inst	0x00133118 ; undefined
 25c:	00240c00 	.inst	0x00240c00 ; NYI
 260:	0b3e0e03 	add	w3, w16, w30, uxtb #3
 264:	00000b0b 	udf	#2827
 268:	49000f0d 	.inst	0x49000f0d ; undefined
 26c:	330e0313 	bfi	w19, w24, #18, #1
 270:	0e000006 	tbl	v6.8b, {v0.16b}, v0.8b
 274:	1349000f 	.inst	0x1349000f ; undefined
 278:	00000633 	udf	#1587
 27c:	4901010f 	.inst	0x4901010f ; undefined
 280:	10000013 	adr	x19, 280 <_start-0x3ffffd80>
 284:	13490021 	.inst	0x13490021 ; undefined
 288:	0b370d22 	add	w2, w9, w23, uxtb #3
 28c:	24110000 	cmphs	p0.b, p0/z, z0.b, z17.b
 290:	0b0e0300 	add	w0, w24, w14
 294:	000b3e0b 	.inst	0x000b3e0b ; undefined
 298:	11010000 	add	w0, w0, #0x40
 29c:	130e2501 	sbfiz	w1, w8, #18, #10
 2a0:	100e0305 	adr	x5, 1c300 <_start-0x3ffe3d00>
 2a4:	b40e1b17 	cbz	x23, 1c604 <_start-0x3ffe39fc>
 2a8:	01111942 	.inst	0x01111942 ; undefined
 2ac:	00000612 	udf	#1554
 2b0:	03013902 	.inst	0x03013902 ; undefined
 2b4:	0300000e 	.inst	0x0300000e ; undefined
 2b8:	0111012e 	.inst	0x0111012e ; undefined
 2bc:	18400612 	ldr	w18, 8037c <_start-0x3ff7fc84>
 2c0:	0b3a0e03 	add	w3, w16, w26, uxtb #3
 2c4:	193f0b3b 	.inst	0x193f0b3b ; undefined
 2c8:	0b040000 	add	w0, w0, w4
 2cc:	00175501 	.inst	0x00175501 ; undefined
 2d0:	00340500 	.inst	0x00340500 ; NYI
 2d4:	0e031802 	uzp1	v2.8b, v0.8b, v3.8b
 2d8:	3a0f0188 	adcs	w8, w12, w15
 2dc:	490b3b0b 	.inst	0x490b3b0b ; undefined
 2e0:	06000013 	.inst	0x06000013 ; undefined
 2e4:	0111010b 	.inst	0x0111010b ; undefined
 2e8:	00000612 	udf	#1554
 2ec:	03011307 	.inst	0x03011307 ; undefined
 2f0:	880b0b0e 	stxr	w11, w14, [x24]
 2f4:	00000f01 	udf	#3841
 2f8:	49002f08 	.inst	0x49002f08 ; undefined
 2fc:	000e0313 	.inst	0x000e0313 ; undefined
 300:	000d0900 	.inst	0x000d0900 ; undefined
 304:	13490e03 	.inst	0x13490e03 ; undefined
 308:	380f0188 	sturb	w8, [x12, #240]
 30c:	0a00000b 	and	w11, w0, w0
 310:	0e030024 	tbl	v4.8b, {v1.16b}, v3.8b
 314:	0b0b0b3e 	add	w30, w25, w11, lsl #2
 318:	0f0b0000 	.inst	0x0f0b0000 ; undefined
 31c:	03134900 	.inst	0x03134900 ; undefined
 320:	0006330e 	.inst	0x0006330e ; undefined
 324:	01010c00 	.inst	0x01010c00 ; undefined
 328:	00001349 	udf	#4937
 32c:	4900210d 	.inst	0x4900210d ; undefined
 330:	370d2213 	tbnz	w19, #1, ffffffffffffa770 <stack_top+0xffffffffbfff6248>
 334:	0e00000b 	tbl	v11.8b, {v0.16b}, v0.8b
 338:	0e030024 	tbl	v4.8b, {v1.16b}, v3.8b
 33c:	0b3e0b0b 	add	w11, w24, w30, uxtb #2
 340:	地址 0x0000000000000340 越界。


Disassembly of section .debug_info:

0000000000000000 <.debug_info>:
       0:	00000ce9 	udf	#3305
       4:	00000004 	udf	#4
       8:	01080000 	.inst	0x01080000 ; undefined
       c:	00000000 	udf	#0
      10:	0041001c 	.inst	0x0041001c ; undefined
      14:	00000000 	udf	#0
      18:	00600000 	.inst	0x00600000 ; undefined
	...
      24:	00300000 	.inst	0x00300000 ; NYI
      28:	84020000 	ld1sb	{z0.s}, p0/z, [x0, z2.s, uxtw]
      2c:	02000000 	.inst	0x02000000 ; undefined
      30:	00000089 	udf	#137
      34:	00008f02 	udf	#36610
      38:	009a0300 	.inst	0x009a0300 ; undefined
      3c:	00e30000 	.inst	0x00e30000 ; undefined
      40:	dc020000 	.inst	0xdc020000 ; undefined
      44:	000c6101 	.inst	0x000c6101 ; undefined
      48:	5a040100 	sbc	w0, w8, w4
      4c:	9800000c 	ldrsw	x12, 4c <_start-0x3fffffb4>
      50:	05000000 	orr	z0.s, z0.s, #0x1
      54:	000000f8 	udf	#248
      58:	01dc0201 	.inst	0x01dc0201 ; undefined
      5c:	00000c6e 	udf	#3182
      60:	38020000 	sturb	w0, [x0, #32]
      64:	06000006 	.inst	0x06000006 ; undefined
      68:	0000081c 	udf	#2076
      6c:	5a040810 	.inst	0x5a040810 ; undefined
      70:	9800000c 	ldrsw	x12, 70 <_start-0x3fffff90>
      74:	07000000 	.inst	0x07000000 ; undefined
      78:	00000102 	udf	#258
      7c:	000009c1 	udf	#2497
      80:	3d070808 	str	b8, [x0, #450]
      84:	61000006 	.inst	0x61000006 ; undefined
      88:	0800000c 	stxrb	w0, w12, [x0]
      8c:	06410700 	.inst	0x06410700 ; undefined
      90:	0b8d0000 	add	w0, w0, w13, asr #0
      94:	00010000 	.inst	0x00010000 ; undefined
      98:	00001808 	udf	#6152
      9c:	00000040 	udf	#64
      a0:	0000bc00 	udf	#48128
      a4:	796f0100 	ldrh	w0, [x8, #6016]
      a8:	b400000a 	cbz	x10, a8 <_start-0x3fffff58>
      ac:	0100000a 	.inst	0x0100000a ; undefined
      b0:	00006754 	udf	#26452
      b4:	91030900 	add	x0, x8, #0xc2
      b8:	008900c8 	.inst	0x008900c8 ; undefined
      bc:	54010000 	b.eq	20bc <_start-0x3fffdf44>  // b.none
      c0:	00000c6e 	udf	#3182
      c4:	0000390a 	udf	#14602
      c8:	00002c00 	udf	#11264
      cc:	00000040 	udf	#64
      d0:	00000400 	udf	#1024
      d4:	19550100 	ldapurb	w0, [x8, #-176]
      d8:	c891030b 	stllr	x11, [x24]
      dc:	00005300 	udf	#21248
      e0:	300c0000 	adr	x0, 180e1 <_start-0x3ffe7f1f>
      e4:	00400000 	.inst	0x00400000 ; undefined
      e8:	94000000 	bl	e8 <_start-0x3fffff18>
      ec:	0d000000 	st1	{v0.b}[0], [x0]
      f0:	00e09103 	.inst	0x00e09103 ; undefined
      f4:	00000102 	udf	#258
      f8:	61550101 	.inst	0x61550101 ; undefined
      fc:	0a00000c 	and	w12, w0, w0
     100:	000005fe 	udf	#1534
     104:	40000030 	.inst	0x40000030 ; undefined
     108:	00000000 	udf	#0
     10c:	00000014 	udf	#20
     110:	0b195801 	add	w1, w0, w25, lsl #22
     114:	00e09103 	.inst	0x00e09103 ; undefined
     118:	00000617 	udf	#1559
     11c:	00064f0a 	.inst	0x00064f0a ; undefined
     120:	00003c00 	udf	#15360
     124:	00000040 	udf	#64
     128:	00000800 	udf	#2048
     12c:	12330300 	and	w0, w24, #0x2000
     130:	e891030b 	.inst	0xe891030b ; undefined
     134:	00065f00 	.inst	0x00065f00 ; undefined
     138:	06240a00 	.inst	0x06240a00 ; undefined
     13c:	003c0000 	.inst	0x003c0000 ; NYI
     140:	00004000 	udf	#16384
     144:	00080000 	.inst	0x00080000 ; undefined
     148:	25030000 	cmpge	p0.b, p0/z, z0.b, #3
     14c:	91030b11 	add	x17, x24, #0xc2
     150:	063d00e8 	.inst	0x063d00e8 ; undefined
     154:	00000000 	udf	#0
     158:	068a0a00 	.inst	0x068a0a00 ; undefined
     15c:	00580000 	.inst	0x00580000 ; undefined
     160:	00004000 	udf	#16384
     164:	000c0000 	.inst	0x000c0000 ; undefined
     168:	5b010000 	.inst	0x5b010000 ; undefined
     16c:	91030b50 	add	x16, x26, #0xc2
     170:	06a400e0 	.inst	0x06a400e0 ; undefined
     174:	030b0000 	.inst	0x030b0000 ; undefined
     178:	b101a891 	adds	x17, x4, #0x6a
     17c:	0e000006 	tbl	v6.8b, {v0.16b}, v0.8b
     180:	000006bf 	udf	#1727
     184:	4000005c 	.inst	0x4000005c ; undefined
     188:	00000000 	udf	#0
     18c:	00000008 	udf	#8
     190:	17039803 	b	fffffffffc0e619c <stack_top+0xffffffffbc0e1c74>
     194:	e091030b 	ld1w	{za2h.s[w12, 3]}, p0/z, [x24, x17, lsl #2]
     198:	0006d900 	.inst	0x0006d900 ; undefined
     19c:	91030b00 	add	x0, x24, #0xc2
     1a0:	06e601b0 	.inst	0x06e601b0 ; undefined
     1a4:	00000000 	udf	#0
     1a8:	0007230a 	.inst	0x0007230a ; undefined
     1ac:	00007400 	udf	#29696
     1b0:	00000040 	udf	#64
     1b4:	00003000 	udf	#12288
     1b8:	245b0100 	cmphs	p0.h, p0/z, z8.h, z27.h
     1bc:	e091030b 	ld1w	{za2h.s[w12, 3]}, p0/z, [x24, x17, lsl #2]
     1c0:	00073d00 	.inst	0x00073d00 ; undefined
     1c4:	91030b00 	add	x0, x24, #0xc2
     1c8:	074a00f8 	.inst	0x074a00f8 ; undefined
     1cc:	f40e0000 	.inst	0xf40e0000 ; undefined
     1d0:	74000006 	.inst	0x74000006 ; undefined
     1d4:	00400000 	.inst	0x00400000 ; undefined
     1d8:	04000000 	add	z0.b, p0/m, z0.b, z0.b
     1dc:	03000000 	.inst	0x03000000 ; undefined
     1e0:	0b0e044e 	add	w14, w2, w14, lsl #1
     1e4:	00e09103 	.inst	0x00e09103 ; undefined
     1e8:	00000716 	udf	#1814
     1ec:	07580e00 	.inst	0x07580e00 ; undefined
     1f0:	00780000 	.inst	0x00780000 ; undefined
     1f4:	00004000 	udf	#16384
     1f8:	00140000 	.inst	0x00140000 ; undefined
     1fc:	4e030000 	tbl	v0.16b, {v0.16b}, v3.16b
     200:	030b1b04 	.inst	0x030b1b04 ; undefined
     204:	7200f891 	.inst	0x7200f891 ; undefined
     208:	0b000007 	add	w7, w0, w0
     20c:	01809103 	.inst	0x01809103 ; undefined
     210:	0000077f 	udf	#1919
     214:	00078d0e 	.inst	0x00078d0e ; undefined
     218:	00007c00 	udf	#31744
     21c:	00000040 	udf	#64
     220:	00001000 	udf	#4096
     224:	043c0300 	add	z0.b, z24.b, z28.b
     228:	91030b0e 	add	x14, x24, #0xc2
     22c:	07a70180 	.inst	0x07a70180 ; undefined
     230:	030b0000 	.inst	0x030b0000 ; undefined
     234:	b4018891 	cbz	x17, 3344 <_start-0x3fffccbc>
     238:	00000007 	udf	#7
     23c:	07c20e00 	.inst	0x07c20e00 ; undefined
     240:	008c0000 	.inst	0x008c0000 ; undefined
     244:	00004000 	udf	#16384
     248:	00180000 	.inst	0x00180000 ; undefined
     24c:	4e030000 	tbl	v0.16b, {v0.16b}, v3.16b
     250:	030b2f04 	.inst	0x030b2f04 ; undefined
     254:	e400e091 	st1b	{z17.b}, p0, [x4]
     258:	0b000007 	add	w7, w0, w0
     25c:	01989103 	.inst	0x01989103 ; undefined
     260:	000007f0 	udf	#2032
     264:	0009590a 	.inst	0x0009590a ; undefined
     268:	00009000 	udf	#36864
     26c:	00000040 	udf	#64
     270:	00001400 	udf	#5120
     274:	09620300 	.inst	0x09620300 ; undefined
     278:	df91030b 	.inst	0xdf91030b ; undefined
     27c:	00097200 	.inst	0x00097200 ; undefined
     280:	91030b00 	add	x0, x24, #0xc2
     284:	097d01a0 	.inst	0x097d01a0 ; undefined
     288:	00000000 	udf	#0
     28c:	00ac0c00 	.inst	0x00ac0c00 ; undefined
     290:	00004000 	udf	#16384
     294:	00180000 	.inst	0x00180000 ; undefined
     298:	020d0000 	.inst	0x020d0000 ; undefined
     29c:	063d2091 	.inst	0x063d2091 ; undefined
     2a0:	01010000 	.inst	0x01010000 ; undefined
     2a4:	000c615a 	.inst	0x000c615a ; undefined
     2a8:	09dc0a00 	.inst	0x09dc0a00 ; undefined
     2ac:	00b00000 	.inst	0x00b00000 ; undefined
     2b0:	00004000 	udf	#16384
     2b4:	00040000 	.inst	0x00040000 ; undefined
     2b8:	5d010000 	.inst	0x5d010000 ; undefined
     2bc:	91030b19 	add	x25, x24, #0xc2
     2c0:	09f501b8 	.inst	0x09f501b8 ; undefined
     2c4:	00000000 	udf	#0
     2c8:	0c5a0400 	.inst	0x0c5a0400 ; undefined
     2cc:	00980000 	.inst	0x00980000 ; undefined
     2d0:	0f000000 	.inst	0x0f000000 ; undefined
     2d4:	000007aa 	udf	#1962
     2d8:	000007f1 	udf	#2033
     2dc:	0c614b06 	.inst	0x0c614b06 ; undefined
     2e0:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     2e4:	00000c5a 	udf	#3162
     2e8:	00000098 	udf	#152
     2ec:	0000f810 	udf	#63504
     2f0:	4b060100 	sub	w0, w8, w6
     2f4:	00000cd8 	udf	#3288
     2f8:	00082510 	.inst	0x00082510 ; undefined
     2fc:	4b060100 	sub	w0, w8, w6
     300:	00000c95 	udf	#3221
     304:	082c1011 	.inst	0x082c1011 ; undefined
     308:	06010000 	.inst	0x06010000 ; undefined
     30c:	000cbe50 	.inst	0x000cbe50 ; undefined
     310:	00000000 	udf	#0
     314:	000a6e02 	.inst	0x000a6e02 ; undefined
     318:	00d40800 	.inst	0x00d40800 ; undefined
     31c:	00004000 	udf	#16384
     320:	014c0000 	.inst	0x014c0000 ; undefined
     324:	6f010000 	.inst	0x6f010000 ; undefined
     328:	00000abc 	udf	#2748
     32c:	00000b36 	udf	#2870
     330:	0bfb7c06 	.inst	0x0bfb7c06 ; undefined
     334:	03090000 	.inst	0x03090000 ; undefined
     338:	f800c891 	sttr	x17, [x4, #12]
     33c:	06000000 	.inst	0x06000000 ; undefined
     340:	000cd87c 	.inst	0x000cd87c ; undefined
     344:	0a020a00 	and	w0, w16, w2, lsl #2
     348:	00f00000 	.inst	0x00f00000 ; undefined
     34c:	00004000 	udf	#16384
     350:	00040000 	.inst	0x00040000 ; undefined
     354:	84060000 	ld1sb	{z0.s}, p0/z, [x0, z6.s, uxtw]
     358:	91030b26 	add	x6, x25, #0xc2
     35c:	0a1c00d8 	and	w24, w6, w28
     360:	0a000000 	and	w0, w0, w0
     364:	00000aac 	udf	#2732
     368:	400000f4 	.inst	0x400000f4 ; undefined
     36c:	00000000 	udf	#0
     370:	00000014 	udf	#20
     374:	0b2f8406 	add	w6, w0, w15, sxtb #1
     378:	00e09103 	.inst	0x00e09103 ; undefined
     37c:	00000ac5 	udf	#2757
     380:	000afd0a 	.inst	0x000afd0a ; undefined
     384:	00010000 	.inst	0x00010000 ; undefined
     388:	00000040 	udf	#64
     38c:	00000800 	udf	#2048
     390:	12320700 	and	w0, w24, #0xc000
     394:	e891030b 	.inst	0xe891030b ; undefined
     398:	000b0d00 	.inst	0x000b0d00 ; undefined
     39c:	0ad20a00 	and	w0, w16, w18, ror #2
     3a0:	01000000 	.inst	0x01000000 ; undefined
     3a4:	00004000 	udf	#16384
     3a8:	00080000 	.inst	0x00080000 ; undefined
     3ac:	24070000 	cmphs	p0.b, p0/z, z0.b, z7.b
     3b0:	91030b11 	add	x17, x24, #0xc2
     3b4:	0aeb00e8 	bic	w8, w7, w11, ror #0
     3b8:	00000000 	udf	#0
     3bc:	07fd0a00 	.inst	0x07fd0a00 ; undefined
     3c0:	013c0000 	.inst	0x013c0000 ; undefined
     3c4:	00004000 	udf	#16384
     3c8:	00100000 	.inst	0x00100000 ; undefined
     3cc:	86060000 	.inst	0x86060000 ; undefined
     3d0:	91030b2a 	add	x10, x25, #0xc2
     3d4:	081600f8 	stxrb	w22, w24, [x7]
     3d8:	6c0a0000 	stnp	d0, d0, [x0, #160]
     3dc:	48000006 	stxrh	w0, w6, [x0]
     3e0:	00400001 	.inst	0x00400001 ; undefined
     3e4:	04000000 	add	z0.b, p0/m, z0.b, z0.b
     3e8:	03000000 	.inst	0x03000000 ; undefined
     3ec:	030b1233 	.inst	0x030b1233 ; undefined
     3f0:	7c018091 	stur	h17, [x4, #24]
     3f4:	0a000006 	and	w6, w0, w0
     3f8:	00000823 	udf	#2083
     3fc:	40000148 	.inst	0x40000148 ; undefined
     400:	00000000 	udf	#0
     404:	00000004 	udf	#4
     408:	0b112503 	add	w3, w8, w17, lsl #9
     40c:	01809103 	.inst	0x01809103 ; undefined
     410:	0000083c 	udf	#2108
     414:	12000000 	and	w0, w0, #0x1
     418:	000002d3 	udf	#723
     41c:	00000000 	udf	#0
     420:	0b352b06 	add	w6, w24, w21, uxth #2
     424:	01989103 	.inst	0x01989103 ; undefined
     428:	000002ec 	udf	#748
     42c:	0008780a 	.inst	0x0008780a ; undefined
     430:	00018800 	.inst	0x00018800 ; undefined
     434:	00000040 	udf	#64
     438:	00003000 	udf	#12288
     43c:	27390600 	.inst	0x27390600 ; undefined
     440:	d891030b 	prfm	plil2strm, fffffffffff224a0 <stack_top+0xffffffffbff1df78>
     444:	00089201 	.inst	0x00089201 ; undefined
     448:	08490e00 	ldxrb	w0, [x16]
     44c:	01880000 	.inst	0x01880000 ; undefined
     450:	00004000 	udf	#16384
     454:	00080000 	.inst	0x00080000 ; undefined
     458:	9c030000 	ldr	q0, 6458 <_start-0x3fff9ba8>
     45c:	030b0e04 	.inst	0x030b0e04 ; undefined
     460:	6b01d891 	.inst	0x6b01d891 ; undefined
     464:	00000008 	udf	#8
     468:	0008ad0e 	.inst	0x0008ad0e ; undefined
     46c:	00019000 	.inst	0x00019000 ; undefined
     470:	00000040 	udf	#64
     474:	00001400 	udf	#5120
     478:	049c0300 	.inst	0x049c0300 ; undefined
     47c:	91030b1b 	add	x27, x24, #0xc2
     480:	08c701e0 	ldlarb	w0, [x15]
     484:	d50e0000 	sys	#6, C0, C0, #0, x0
     488:	9000000b 	adrp	x11, 0 <_start-0x40000000>
     48c:	00400001 	.inst	0x00400001 ; undefined
     490:	04000000 	add	z0.b, p0/m, z0.b, z0.b
     494:	03000000 	.inst	0x03000000 ; undefined
     498:	132f048a 	.inst	0x132f048a ; undefined
     49c:	00000ba9 	udf	#2985
     4a0:	40000190 	.inst	0x40000190 ; undefined
     4a4:	00000000 	udf	#0
     4a8:	00000004 	udf	#4
     4ac:	1b054a08 	madd	w8, w16, w5, w18
     4b0:	08e20e00 	.inst	0x08e20e00 ; undefined
     4b4:	01940000 	.inst	0x01940000 ; undefined
     4b8:	00004000 	udf	#16384
     4bc:	00100000 	.inst	0x00100000 ; undefined
     4c0:	8a030000 	and	x0, x0, x3
     4c4:	030b0e04 	.inst	0x030b0e04 ; undefined
     4c8:	fc01e091 	stur	d17, [x4, #30]
     4cc:	0b000008 	add	w8, w0, w0
     4d0:	01e89103 	.inst	0x01e89103 ; undefined
     4d4:	00000909 	udf	#2313
     4d8:	170e0000 	b	fffffffffc3804d8 <stack_top+0xffffffffbc37bfb0>
     4dc:	a4000009 	ld1rqb	{z9.b}, p0/z, [x0, x0]
     4e0:	00400001 	.inst	0x00400001 ; undefined
     4e4:	14000000 	b	4e4 <_start-0x3ffffb1c>
     4e8:	03000000 	.inst	0x03000000 ; undefined
     4ec:	0b2f049c 	add	w28, w4, w15, uxtb #1
     4f0:	01d89103 	.inst	0x01d89103 ; undefined
     4f4:	00000939 	udf	#2361
     4f8:	f891030b 	prfum	plil2strm, [x24, #-240]
     4fc:	00094501 	.inst	0x00094501 ; undefined
     500:	098a0a00 	.inst	0x098a0a00 ; undefined
     504:	01a80000 	.inst	0x01a80000 ; undefined
     508:	00004000 	udf	#16384
     50c:	00100000 	.inst	0x00100000 ; undefined
     510:	62030000 	.inst	0x62030000 ; undefined
     514:	91030b09 	add	x9, x24, #0xc2
     518:	09a300d7 	.inst	0x09a300d7 ; undefined
     51c:	030b0000 	.inst	0x030b0000 ; undefined
     520:	ae028091 	.inst	0xae028091 ; undefined
     524:	00000009 	udf	#9
     528:	2a0a0000 	orr	w0, w0, w10
     52c:	d800000a 	prfm	plil2keep, 52c <_start-0x3ffffad4>
     530:	00400001 	.inst	0x00400001 ; undefined
     534:	04000000 	add	z0.b, p0/m, z0.b, z0.b
     538:	06000000 	.inst	0x06000000 ; undefined
     53c:	030b2850 	.inst	0x030b2850 ; undefined
     540:	4401a091 	.inst	0x4401a091 ; undefined
     544:	0000000a 	udf	#10
     548:	0001dc0c 	.inst	0x0001dc0c ; undefined
     54c:	00000040 	udf	#64
     550:	00003400 	udf	#13312
     554:	91030b00 	add	x0, x24, #0xc2
     558:	030501a8 	.inst	0x030501a8 ; undefined
     55c:	520a0000 	eor	w0, w0, #0x400000
     560:	e400000a 	.inst	0xe400000a ; undefined
     564:	00400001 	.inst	0x00400001 ; undefined
     568:	08000000 	stxrb	w0, w0, [x0]
     56c:	06000000 	.inst	0x06000000 ; undefined
     570:	030b4953 	.inst	0x030b4953 ; undefined
     574:	6c01b091 	stnp	d17, d12, [x4, #24]
     578:	0000000a 	udf	#10
     57c:	000b1b0a 	.inst	0x000b1b0a ; undefined
     580:	0001ec00 	.inst	0x0001ec00 ; undefined
     584:	00000040 	udf	#64
     588:	00001400 	udf	#5120
     58c:	52530600 	.inst	0x52530600 ; undefined
     590:	b891030b 	ldursw	x11, [x24, #-240]
     594:	000b3501 	.inst	0x000b3501 ; undefined
     598:	0b500e00 	add	w0, w16, w16, lsr #3
     59c:	01f00000 	.inst	0x01f00000 ; undefined
     5a0:	00004000 	udf	#16384
     5a4:	00100000 	.inst	0x00100000 ; undefined
     5a8:	fc070000 	stur	d0, [x0, #112]
     5ac:	030b1703 	.inst	0x030b1703 ; undefined
     5b0:	6a01b891 	.inst	0x6a01b891 ; undefined
     5b4:	0b00000b 	add	w11, w0, w0
     5b8:	01c09103 	.inst	0x01c09103 ; undefined
     5bc:	00000b77 	udf	#2935
     5c0:	7a0a0000 	sbcs	w0, w0, w10
     5c4:	0000000a 	udf	#10
     5c8:	00400002 	.inst	0x00400002 ; undefined
     5cc:	04000000 	add	z0.b, p0/m, z0.b, z0.b
     5d0:	06000000 	.inst	0x06000000 ; undefined
     5d4:	030b2953 	.inst	0x030b2953 ; undefined
     5d8:	9301d091 	.inst	0x9301d091 ; undefined
     5dc:	0000000a 	udf	#10
     5e0:	5a040000 	sbc	w0, w0, w4
     5e4:	9800000c 	ldrsw	x12, 5e4 <_start-0x3ffffa1c>
     5e8:	00000000 	udf	#0
     5ec:	02000000 	.inst	0x02000000 ; undefined
     5f0:	00000102 	udf	#258
     5f4:	00011902 	.inst	0x00011902 ; undefined
     5f8:	008f0200 	.inst	0x008f0200 ; undefined
     5fc:	230f0000 	.inst	0x230f0000 ; undefined
     600:	79000001 	strh	w1, [x0]
     604:	03000001 	.inst	0x03000001 ; undefined
     608:	000c9c22 	.inst	0x000c9c22 ; undefined
     60c:	5a040100 	sbc	w0, w8, w4
     610:	9800000c 	ldrsw	x12, 610 <_start-0x3ffff9f0>
     614:	10000000 	adr	x0, 614 <_start-0x3ffff9ec>
     618:	000000f8 	udf	#248
     61c:	61220301 	.inst	0x61220301 ; undefined
     620:	0000000c 	udf	#12
     624:	00018a0f 	.inst	0x00018a0f ; undefined
     628:	0001dd00 	.inst	0x0001dd00 ; undefined
     62c:	95cb0300 	bl	72c122c <_start-0x38d3edd4>
     630:	0100000c 	.inst	0x0100000c ; undefined
     634:	000c5a04 	.inst	0x000c5a04 ; undefined
     638:	00009800 	udf	#38912
     63c:	00f81000 	.inst	0x00f81000 ; undefined
     640:	03010000 	.inst	0x03010000 ; undefined
     644:	000c61cb 	.inst	0x000c61cb ; undefined
     648:	e6020000 	.inst	0xe6020000 ; undefined
     64c:	0f000001 	.inst	0x0f000001 ; undefined
     650:	000001ee 	udf	#494
     654:	00000252 	udf	#594
     658:	0c9c2403 	st1	{v3.4h-v6.4h}, [x0], x28
     65c:	10010000 	adr	x0, 265c <_start-0x3fffd9a4>
     660:	00000102 	udf	#258
     664:	61240301 	.inst	0x61240301 ; undefined
     668:	0000000c 	udf	#12
     66c:	0001ee0f 	.inst	0x0001ee0f ; undefined
     670:	00025200 	.inst	0x00025200 ; undefined
     674:	9c240300 	ldr	q0, 486d4 <_start-0x3ffb792c>
     678:	0100000c 	.inst	0x0100000c ; undefined
     67c:	00010210 	.inst	0x00010210 ; undefined
     680:	24030100 	cmphs	p0.b, p0/z, z8.b, z3.b
     684:	00000c61 	udf	#3169
     688:	5f030000 	.inst	0x5f030000 ; undefined
     68c:	e3000002 	.inst	0xe3000002 ; undefined
     690:	03000003 	.inst	0x03000003 ; undefined
     694:	0c610393 	.inst	0x0c610393 ; undefined
     698:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     69c:	00000c5a 	udf	#3162
     6a0:	00000098 	udf	#152
     6a4:	0000f805 	udf	#63493
     6a8:	93030100 	.inst	0x93030100 ; undefined
     6ac:	000c6103 	.inst	0x000c6103 ; undefined
     6b0:	02b10500 	.inst	0x02b10500 ; undefined
     6b4:	03010000 	.inst	0x03010000 ; undefined
     6b8:	0c950393 	st4	{v19.8b-v22.8b}, [x28], x21
     6bc:	03000000 	.inst	0x03000000 ; undefined
     6c0:	000002b7 	udf	#695
     6c4:	000004c0 	udf	#1216
     6c8:	6101cc03 	.inst	0x6101cc03 ; undefined
     6cc:	0100000c 	.inst	0x0100000c ; undefined
     6d0:	000c5a04 	.inst	0x000c5a04 ; undefined
     6d4:	00009800 	udf	#38912
     6d8:	00f80500 	.inst	0x00f80500 ; undefined
     6dc:	03010000 	.inst	0x03010000 ; undefined
     6e0:	0c6101cc 	.inst	0x0c6101cc ; undefined
     6e4:	b1050000 	adds	x0, x0, #0x140
     6e8:	01000002 	.inst	0x01000002 ; undefined
     6ec:	a301cc03 	.inst	0xa301cc03 ; undefined
     6f0:	0000000c 	udf	#12
     6f4:	0003140f 	.inst	0x0003140f ; undefined
     6f8:	00036700 	.inst	0x00036700 ; undefined
     6fc:	613a0300 	.inst	0x613a0300 ; undefined
     700:	0100000c 	.inst	0x0100000c ; undefined
     704:	000c5a04 	.inst	0x000c5a04 ; undefined
     708:	00009800 	udf	#38912
     70c:	0c5a0400 	.inst	0x0c5a0400 ; undefined
     710:	03120000 	.inst	0x03120000 ; undefined
     714:	f8100000 	stur	x0, [x0, #-256]
     718:	01000000 	.inst	0x01000000 ; undefined
     71c:	0c613a03 	.inst	0x0c613a03 ; undefined
     720:	03000000 	.inst	0x03000000 ; undefined
     724:	00000374 	udf	#884
     728:	000003d5 	udf	#981
     72c:	61044d03 	.inst	0x61044d03 ; undefined
     730:	0100000c 	.inst	0x0100000c ; undefined
     734:	000c5a04 	.inst	0x000c5a04 ; undefined
     738:	00009800 	udf	#38912
     73c:	00f80500 	.inst	0x00f80500 ; undefined
     740:	03010000 	.inst	0x03010000 ; undefined
     744:	0c61044d 	.inst	0x0c61044d ; undefined
     748:	b1050000 	adds	x0, x0, #0x140
     74c:	01000002 	.inst	0x01000002 ; undefined
     750:	95044d03 	bl	4113b5c <_start-0x3beec4a4>
     754:	0000000c 	udf	#12
     758:	0003eb03 	.inst	0x0003eb03 ; undefined
     75c:	00044700 	.inst	0x00044700 ; undefined
     760:	04380300 	add	z0.b, z24.b, z24.b
     764:	00000c61 	udf	#3169
     768:	0c5a0401 	.inst	0x0c5a0401 ; undefined
     76c:	00980000 	.inst	0x00980000 ; undefined
     770:	b1050000 	adds	x0, x0, #0x140
     774:	01000002 	.inst	0x01000002 ; undefined
     778:	95043803 	bl	410e784 <_start-0x3bef187c>
     77c:	0500000c 	orr	z12.s, z12.s, #0x1
     780:	000000f8 	udf	#248
     784:	04380301 	add	z1.b, z24.b, z24.b
     788:	00000c61 	udf	#3169
     78c:	04580300 	orr	z0.h, p0/m, z0.h, z24.h
     790:	04b70000 	add	z0.s, z0.s, z23.s
     794:	1c030000 	ldr	s0, 6794 <_start-0x3fff986c>
     798:	000c6102 	.inst	0x000c6102 ; undefined
     79c:	5a040100 	sbc	w0, w8, w4
     7a0:	9800000c 	ldrsw	x12, 7a0 <_start-0x3ffff860>
     7a4:	05000000 	orr	z0.s, z0.s, #0x1
     7a8:	000000f8 	udf	#248
     7ac:	021c0301 	.inst	0x021c0301 ; undefined
     7b0:	00000c61 	udf	#3169
     7b4:	0002b105 	.inst	0x0002b105 ; undefined
     7b8:	1c030100 	ldr	s0, 67d8 <_start-0x3fff9828>
     7bc:	000ca302 	.inst	0x000ca302 ; undefined
     7c0:	cb0f0000 	sub	x0, x0, x15
     7c4:	2b000004 	adds	w4, w0, w0
     7c8:	03000005 	.inst	0x03000005 ; undefined
     7cc:	000c615e 	.inst	0x000c615e ; undefined
     7d0:	5a040100 	sbc	w0, w8, w4
     7d4:	9800000c 	ldrsw	x12, 7d4 <_start-0x3ffff82c>
     7d8:	04000000 	add	z0.b, p0/m, z0.b, z0.b
     7dc:	00000c5a 	udf	#3162
     7e0:	00000312 	udf	#786
     7e4:	00054410 	.inst	0x00054410 ; undefined
     7e8:	5e030100 	sha1c	q0, s8, v3.4s
     7ec:	00000c61 	udf	#3169
     7f0:	0000f810 	udf	#63504
     7f4:	5e030100 	sha1c	q0, s8, v3.4s
     7f8:	00000c61 	udf	#3169
     7fc:	01230f00 	.inst	0x01230f00 ; undefined
     800:	01790000 	.inst	0x01790000 ; undefined
     804:	22030000 	.inst	0x22030000 ; undefined
     808:	00000c9c 	udf	#3228
     80c:	0c5a0401 	.inst	0x0c5a0401 ; undefined
     810:	00980000 	.inst	0x00980000 ; undefined
     814:	f8100000 	stur	x0, [x0, #-256]
     818:	01000000 	.inst	0x01000000 ; undefined
     81c:	0c612203 	.inst	0x0c612203 ; undefined
     820:	0f000000 	.inst	0x0f000000 ; undefined
     824:	0000018a 	udf	#394
     828:	000001dd 	udf	#477
     82c:	0c95cb03 	.inst	0x0c95cb03 ; undefined
     830:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     834:	00000c5a 	udf	#3162
     838:	00000098 	udf	#152
     83c:	0000f810 	udf	#63504
     840:	cb030100 	sub	x0, x8, x3
     844:	00000c61 	udf	#3169
     848:	03140f00 	.inst	0x03140f00 ; undefined
     84c:	03670000 	.inst	0x03670000 ; undefined
     850:	3a030000 	adcs	w0, w0, w3
     854:	00000c61 	udf	#3169
     858:	0c5a0401 	.inst	0x0c5a0401 ; undefined
     85c:	00980000 	.inst	0x00980000 ; undefined
     860:	5a040000 	sbc	w0, w0, w4
     864:	1200000c 	and	w12, w0, #0x1
     868:	10000003 	adr	x3, 868 <_start-0x3ffff798>
     86c:	000000f8 	udf	#248
     870:	613a0301 	.inst	0x613a0301 ; undefined
     874:	0000000c 	udf	#12
     878:	00083003 	.inst	0x00083003 ; undefined
     87c:	00089100 	.inst	0x00089100 ; undefined
     880:	049b0300 	bic	z0.s, p0/m, z0.s, z24.s
     884:	00000c61 	udf	#3169
     888:	0c5a0401 	.inst	0x0c5a0401 ; undefined
     88c:	00980000 	.inst	0x00980000 ; undefined
     890:	f8050000 	stur	x0, [x0, #80]
     894:	01000000 	.inst	0x01000000 ; undefined
     898:	61049b03 	.inst	0x61049b03 ; undefined
     89c:	0500000c 	orr	z12.s, z12.s, #0x1
     8a0:	000002b1 	udf	#689
     8a4:	049b0301 	bic	z1.s, p0/m, z1.s, z24.s
     8a8:	00000c95 	udf	#3221
     8ac:	09620300 	.inst	0x09620300 ; undefined
     8b0:	09be0000 	.inst	0x09be0000 ; undefined
     8b4:	86030000 	.inst	0x86030000 ; undefined
     8b8:	000c6104 	.inst	0x000c6104 ; undefined
     8bc:	5a040100 	sbc	w0, w8, w4
     8c0:	9800000c 	ldrsw	x12, 8c0 <_start-0x3ffff740>
     8c4:	05000000 	orr	z0.s, z0.s, #0x1
     8c8:	000000f8 	udf	#248
     8cc:	04860301 	.inst	0x04860301 ; undefined
     8d0:	00000c61 	udf	#3169
     8d4:	0002b105 	.inst	0x0002b105 ; undefined
     8d8:	86030100 	.inst	0x86030100 ; undefined
     8dc:	000c9504 	.inst	0x000c9504 ; undefined
     8e0:	58030000 	ldr	x0, 68e0 <_start-0x3fff9720>
     8e4:	b7000004 	tbnz	x4, #32, 8e4 <_start-0x3ffff71c>
     8e8:	03000004 	.inst	0x03000004 ; undefined
     8ec:	0c61021c 	.inst	0x0c61021c ; undefined
     8f0:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     8f4:	00000c5a 	udf	#3162
     8f8:	00000098 	udf	#152
     8fc:	0000f805 	udf	#63493
     900:	1c030100 	ldr	s0, 6920 <_start-0x3fff96e0>
     904:	000c6102 	.inst	0x000c6102 ; undefined
     908:	02b10500 	.inst	0x02b10500 ; undefined
     90c:	03010000 	.inst	0x03010000 ; undefined
     910:	0ca3021c 	.inst	0x0ca3021c ; undefined
     914:	0f000000 	.inst	0x0f000000 ; undefined
     918:	000004cb 	udf	#1227
     91c:	0000052b 	udf	#1323
     920:	0c615e03 	.inst	0x0c615e03 ; undefined
     924:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     928:	00000c5a 	udf	#3162
     92c:	00000098 	udf	#152
     930:	000c5a04 	.inst	0x000c5a04 ; undefined
     934:	00031200 	.inst	0x00031200 ; undefined
     938:	05441000 	.inst	0x05441000 ; undefined
     93c:	03010000 	.inst	0x03010000 ; undefined
     940:	000c615e 	.inst	0x000c615e ; undefined
     944:	00f81000 	.inst	0x00f81000 ; undefined
     948:	03010000 	.inst	0x03010000 ; undefined
     94c:	000c615e 	.inst	0x000c615e ; undefined
     950:	00000000 	udf	#0
     954:	00054902 	.inst	0x00054902 ; undefined
     958:	05520f00 	mov	z0.h, p2/z, #120
     95c:	058c0000 	.inst	0x058c0000 ; undefined
     960:	6f040000 	.inst	0x6f040000 ; undefined
     964:	00000c61 	udf	#3169
     968:	0c5a0401 	.inst	0x0c5a0401 ; undefined
     96c:	00980000 	.inst	0x00980000 ; undefined
     970:	49140000 	.inst	0x49140000 ; undefined
     974:	04000005 	add	z5.b, p0/m, z5.b, z0.b
     978:	000caa71 	.inst	0x000caa71 ; undefined
     97c:	059f1000 	mov	z0.s, p15/z, #-128
     980:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     984:	000cb170 	.inst	0x000cb170 ; undefined
     988:	520f0000 	eor	w0, w0, #0x20000
     98c:	8c000005 	.inst	0x8c000005 ; undefined
     990:	04000005 	add	z5.b, p0/m, z5.b, z0.b
     994:	000c616f 	.inst	0x000c616f ; undefined
     998:	5a040100 	sbc	w0, w8, w4
     99c:	9800000c 	ldrsw	x12, 99c <_start-0x3ffff664>
     9a0:	14000000 	b	9a0 <_start-0x3ffff660>
     9a4:	00000549 	udf	#1353
     9a8:	0caa7104 	.inst	0x0caa7104 ; undefined
     9ac:	9f100000 	.inst	0x9f100000 ; undefined
     9b0:	01000005 	.inst	0x01000005 ; undefined
     9b4:	0cb17004 	.inst	0x0cb17004 ; undefined
     9b8:	00000000 	udf	#0
     9bc:	0005b602 	.inst	0x0005b602 ; undefined
     9c0:	05c70600 	.inst	0x05c70600 ; undefined
     9c4:	08080000 	stxrb	w8, w0, [x0]
     9c8:	000c5a04 	.inst	0x000c5a04 ; undefined
     9cc:	00009800 	udf	#38912
     9d0:	05bf0700 	zip2	z0.q, z24.q, z31.q
     9d4:	0c610000 	.inst	0x0c610000 ; undefined
     9d8:	00080000 	.inst	0x00080000 ; undefined
     9dc:	0005d30f 	.inst	0x0005d30f ; undefined
     9e0:	00061e00 	.inst	0x00061e00 ; undefined
     9e4:	c1c50500 	.inst	0xc1c50500 ; undefined
     9e8:	01000009 	.inst	0x01000009 ; undefined
     9ec:	000c5a04 	.inst	0x000c5a04 ; undefined
     9f0:	00009800 	udf	#38912
     9f4:	01021000 	.inst	0x01021000 ; undefined
     9f8:	05010000 	orr	z0.s, z0.s, #0x1
     9fc:	000cbec5 	.inst	0x000cbec5 ; undefined
     a00:	5e030000 	sha1c	q0, s0, v3.4s
     a04:	e3000006 	.inst	0xe3000006 ; undefined
     a08:	05000000 	orr	z0.s, z0.s, #0x1
     a0c:	0cbe0145 	.inst	0x0cbe0145 ; undefined
     a10:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     a14:	00000c5a 	udf	#3162
     a18:	00000098 	udf	#152
     a1c:	0000f805 	udf	#63493
     a20:	45050100 	.inst	0x45050100 ; undefined
     a24:	0009c101 	.inst	0x0009c101 ; undefined
     a28:	5e030000 	sha1c	q0, s0, v3.4s
     a2c:	e3000006 	.inst	0xe3000006 ; undefined
     a30:	05000000 	orr	z0.s, z0.s, #0x1
     a34:	0cbe0145 	.inst	0x0cbe0145 ; undefined
     a38:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     a3c:	00000c5a 	udf	#3162
     a40:	00000098 	udf	#152
     a44:	0000f805 	udf	#63493
     a48:	45050100 	.inst	0x45050100 ; undefined
     a4c:	0009c101 	.inst	0x0009c101 ; undefined
     a50:	5e030000 	sha1c	q0, s0, v3.4s
     a54:	e3000006 	.inst	0xe3000006 ; undefined
     a58:	05000000 	orr	z0.s, z0.s, #0x1
     a5c:	0cbe0145 	.inst	0x0cbe0145 ; undefined
     a60:	04010000 	sub	z0.b, p0/m, z0.b, z0.b
     a64:	00000c5a 	udf	#3162
     a68:	00000098 	udf	#152
     a6c:	0000f805 	udf	#63493
     a70:	45050100 	.inst	0x45050100 ; undefined
     a74:	0009c101 	.inst	0x0009c101 ; undefined
     a78:	d30f0000 	.inst	0xd30f0000 ; undefined
     a7c:	1e000005 	.inst	0x1e000005 ; undefined
     a80:	05000006 	orr	z6.s, z6.s, #0x1
     a84:	0009c1c5 	.inst	0x0009c1c5 ; undefined
     a88:	5a040100 	sbc	w0, w8, w4
     a8c:	9800000c 	ldrsw	x12, a8c <_start-0x3ffff574>
     a90:	10000000 	adr	x0, a90 <_start-0x3ffff570>
     a94:	00000102 	udf	#258
     a98:	bec50501 	.inst	0xbec50501 ; undefined
     a9c:	0000000c 	udf	#12
     aa0:	a1020000 	.inst	0xa1020000 ; undefined
     aa4:	02000006 	.inst	0x02000006 ; undefined
     aa8:	0000008f 	udf	#143
     aac:	0006a90f 	.inst	0x0006a90f ; undefined
     ab0:	00017900 	.inst	0x00017900 ; undefined
     ab4:	9c210700 	ldr	q0, 42b94 <_start-0x3ffbd46c>
     ab8:	0100000c 	.inst	0x0100000c ; undefined
     abc:	000c5a04 	.inst	0x000c5a04 ; undefined
     ac0:	00009800 	udf	#38912
     ac4:	00f81000 	.inst	0x00f81000 ; undefined
     ac8:	07010000 	.inst	0x07010000 ; undefined
     acc:	000cbe21 	.inst	0x000cbe21 ; undefined
     ad0:	fb0f0000 	.inst	0xfb0f0000 ; undefined
     ad4:	dd000006 	.inst	0xdd000006 ; undefined
     ad8:	07000001 	.inst	0x07000001 ; undefined
     adc:	000c95d1 	.inst	0x000c95d1 ; undefined
     ae0:	5a040100 	sbc	w0, w8, w4
     ae4:	9800000c 	ldrsw	x12, ae4 <_start-0x3ffff51c>
     ae8:	10000000 	adr	x0, ae8 <_start-0x3ffff518>
     aec:	000000f8 	udf	#248
     af0:	bed10701 	.inst	0xbed10701 ; undefined
     af4:	0000000c 	udf	#12
     af8:	0001e602 	.inst	0x0001e602 ; undefined
     afc:	074a0f00 	.inst	0x074a0f00 ; undefined
     b00:	02520000 	.inst	0x02520000 ; undefined
     b04:	23070000 	.inst	0x23070000 ; undefined
     b08:	00000c9c 	udf	#3228
     b0c:	01021001 	.inst	0x01021001 ; undefined
     b10:	07010000 	.inst	0x07010000 ; undefined
     b14:	000cbe23 	.inst	0x000cbe23 ; undefined
     b18:	03000000 	.inst	0x03000000 ; undefined
     b1c:	000009cf 	udf	#2511
     b20:	000003e3 	udf	#995
     b24:	be03f707 	.inst	0xbe03f707 ; undefined
     b28:	0100000c 	.inst	0x0100000c ; undefined
     b2c:	000c5a04 	.inst	0x000c5a04 ; undefined
     b30:	00009800 	udf	#38912
     b34:	00f80500 	.inst	0x00f80500 ; undefined
     b38:	07010000 	.inst	0x07010000 ; undefined
     b3c:	0cbe03f7 	.inst	0x0cbe03f7 ; undefined
     b40:	b1050000 	adds	x0, x0, #0x140
     b44:	01000002 	.inst	0x01000002 ; undefined
     b48:	9503f707 	bl	40fe764 <_start-0x3bf0189c>
     b4c:	0000000c 	udf	#12
     b50:	000a1d03 	.inst	0x000a1d03 ; undefined
     b54:	0004c000 	.inst	0x0004c000 ; undefined
     b58:	01d80700 	.inst	0x01d80700 ; undefined
     b5c:	00000cbe 	udf	#3262
     b60:	0c5a0401 	.inst	0x0c5a0401 ; undefined
     b64:	00980000 	.inst	0x00980000 ; undefined
     b68:	f8050000 	stur	x0, [x0, #80]
     b6c:	01000000 	.inst	0x01000000 ; undefined
     b70:	be01d807 	.inst	0xbe01d807 ; undefined
     b74:	0500000c 	orr	z12.s, z12.s, #0x1
     b78:	000002b1 	udf	#689
     b7c:	01d80701 	.inst	0x01d80701 ; undefined
     b80:	00000ca3 	udf	#3235
     b84:	00000000 	udf	#0
     b88:	00064202 	.inst	0x00064202 ; undefined
     b8c:	064d0600 	.inst	0x064d0600 ; undefined
     b90:	01000000 	.inst	0x01000000 ; undefined
     b94:	000ccb04 	.inst	0x000ccb04 ; undefined
     b98:	00009800 	udf	#38912
     b9c:	02000000 	.inst	0x02000000 ; undefined
     ba0:	000008a7 	udf	#2215
     ba4:	0008ab02 	.inst	0x0008ab02 ; undefined
     ba8:	08b40300 	.inst	0x08b40300 ; undefined
     bac:	08fc0000 	.inst	0x08fc0000 ; undefined
     bb0:	9a080000 	adc	x0, x0, x8
     bb4:	000ca304 	.inst	0x000ca304 ; undefined
     bb8:	f8050100 	stur	x0, [x8, #80]
     bbc:	01000000 	.inst	0x01000000 ; undefined
     bc0:	a3049a08 	.inst	0xa3049a08 ; undefined
     bc4:	0500000c 	orr	z12.s, z12.s, #0x1
     bc8:	00000909 	udf	#2313
     bcc:	049a0801 	and	z1.s, p2/m, z1.s, z0.s
     bd0:	00000ca3 	udf	#3235
     bd4:	090d0300 	.inst	0x090d0300 ; undefined
     bd8:	09550000 	.inst	0x09550000 ; undefined
     bdc:	49080000 	.inst	0x49080000 ; undefined
     be0:	000ca305 	.inst	0x000ca305 ; undefined
     be4:	f8050100 	stur	x0, [x8, #80]
     be8:	01000000 	.inst	0x01000000 ; undefined
     bec:	a3054908 	.inst	0xa3054908 ; undefined
     bf0:	0000000c 	udf	#12
     bf4:	3f020000 	.inst	0x3f020000 ; undefined
     bf8:	0600000b 	.inst	0x0600000b ; undefined
     bfc:	00000b58 	udf	#2904
     c00:	07150808 	.inst	0x07150808 ; undefined
     c04:	1600000c 	b	fffffffff8000c34 <stack_top+0xffffffffb7ffc70c>
     c08:	00000ce5 	udf	#3301
     c0c:	00170008 	.inst	0x00170008 ; undefined
     c10:	000b4a07 	.inst	0x000b4a07 ; undefined
     c14:	000c2a00 	.inst	0x000c2a00 ; undefined
     c18:	00000800 	udf	#2048
     c1c:	0b4f0718 	add	w24, w24, w15, lsr #1
     c20:	0c3b0000 	.inst	0x0c3b0000 ; undefined
     c24:	00080000 	.inst	0x00080000 ; undefined
     c28:	4a060000 	eor	w0, w0, w6
     c2c:	0800000b 	stxrb	w0, w11, [x0]
     c30:	0ccb0408 	ld4	{v8.4h-v11.4h}, [x0], x11
     c34:	00980000 	.inst	0x00980000 ; undefined
     c38:	06000000 	.inst	0x06000000 ; undefined
     c3c:	00000b4f 	udf	#2895
     c40:	cb040808 	sub	x8, x0, x4, lsl #2
     c44:	9800000c 	ldrsw	x12, c44 <_start-0x3ffff3bc>
     c48:	07000000 	.inst	0x07000000 ; undefined
     c4c:	00000b54 	udf	#2900
     c50:	00000ccb 	udf	#3275
     c54:	00000008 	udf	#8
     c58:	f5190000 	.inst	0xf5190000 ; undefined
     c5c:	07000000 	.inst	0x07000000 ; undefined
     c60:	0c5a1a01 	.inst	0x0c5a1a01 ; undefined
     c64:	00ee0000 	.inst	0x00ee0000 ; undefined
     c68:	00000000 	udf	#0
     c6c:	13060000 	sbfiz	w0, w0, #26, #1
     c70:	10000001 	adr	x1, c70 <_start-0x3ffff390>
     c74:	00fd0708 	.inst	0x00fd0708 ; undefined
     c78:	0c8c0000 	st4	{v0.8b-v3.8b}, [x0], x12
     c7c:	00080000 	.inst	0x00080000 ; undefined
     c80:	00010607 	.inst	0x00010607 ; undefined
     c84:	000c9500 	.inst	0x000c9500 ; undefined
     c88:	00080800 	.inst	0x00080800 ; undefined
     c8c:	000c5a1b 	.inst	0x000c5a1b ; undefined
     c90:	00000000 	udf	#0
     c94:	010d1900 	.inst	0x010d1900 ; undefined
     c98:	08070000 	stxrb	w7, w0, [x0]
     c9c:	00018519 	.inst	0x00018519 ; undefined
     ca0:	19010200 	stlurb	w0, [x16, #16]
     ca4:	0000030c 	udf	#780
     ca8:	b3190805 	.inst	0xb3190805 ; undefined
     cac:	07000005 	.inst	0x07000005 ; undefined
     cb0:	0caa1a00 	.inst	0x0caa1a00 ; undefined
     cb4:	05ac0000 	zip1	z0.q, z0.q, z12.q
     cb8:	00000000 	udf	#0
     cbc:	5a1a0000 	sbc	w0, w0, w26
     cc0:	3000000c 	adr	x12, cc1 <_start-0x3ffff33f>
     cc4:	00000006 	udf	#6
     cc8:	1a000000 	adc	w0, w0, w0
     ccc:	00000c5a 	udf	#3162
     cd0:	00000649 	udf	#1609
     cd4:	00000000 	udf	#0
     cd8:	0000671a 	udf	#26394
     cdc:	00080400 	.inst	0x00080400 ; undefined
     ce0:	00000000 	udf	#0
     ce4:	0b461900 	add	w0, w8, w6, lsr #6
     ce8:	08070000 	stxrb	w7, w0, [x0]
     cec:	00008900 	udf	#35072
     cf0:	5f000400 	.inst	0x5f000400 ; undefined
     cf4:	08000001 	stxrb	w0, w1, [x0]
     cf8:	00000001 	udf	#1
     cfc:	64001c00 	.inst	0x64001c00 ; undefined
     d00:	7d00000b 	str	h11, [x0]
     d04:	60000003 	.inst	0x60000003 ; undefined
     d08:	20000000 	.inst	0x20000000 ; undefined
     d0c:	00400002 	.inst	0x00400002 ; undefined
     d10:	1c000000 	ldr	s0, d10 <_start-0x3ffff2f0>
     d14:	02000000 	.inst	0x02000000 ; undefined
     d18:	00000084 	udf	#132
     d1c:	00010202 	.inst	0x00010202 ; undefined
     d20:	02200300 	.inst	0x02200300 ; undefined
     d24:	00004000 	udf	#16384
     d28:	001c0000 	.inst	0x001c0000 ; undefined
     d2c:	6f010000 	.inst	0x6f010000 ; undefined
     d30:	00000b83 	udf	#2947
     d34:	00000bb4 	udf	#2996
     d38:	04063401 	.inst	0x04063401 ; undefined
     d3c:	c7009102 	.inst	0xc7009102 ; undefined
     d40:	0100000b 	.inst	0x0100000b ; undefined
     d44:	007f0634 	.inst	0x007f0634 ; undefined
     d48:	02040000 	.inst	0x02040000 ; undefined
     d4c:	0bcb0f91 	.inst	0x0bcb0f91 ; undefined
     d50:	34010000 	cbz	w0, 2d50 <_start-0x3fffd2b0>
     d54:	00007806 	udf	#30726
     d58:	00780500 	.inst	0x00780500 ; undefined
     d5c:	00980000 	.inst	0x00980000 ; undefined
     d60:	00000000 	udf	#0
     d64:	00f50600 	.inst	0x00f50600 ; undefined
     d68:	01070000 	.inst	0x01070000 ; undefined
     d6c:	00007807 	udf	#30727
     d70:	00063000 	.inst	0x00063000 ; undefined
     d74:	00000000 	udf	#0
     d78:	01a90000 	.inst	0x01a90000 ; undefined
     d7c:	00040000 	.inst	0x00040000 ; undefined
     d80:	000001be 	udf	#446
     d84:	00000108 	udf	#264
     d88:	001c0000 	.inst	0x001c0000 ; undefined
     d8c:	00000bcf 	udf	#3023
     d90:	00000405 	udf	#1029
     d94:	00000060 	udf	#96
     d98:	4000023c 	.inst	0x4000023c ; undefined
     d9c:	00000000 	udf	#0
     da0:	00000034 	udf	#52
     da4:	00008402 	udf	#33794
     da8:	00890200 	.inst	0x00890200 ; undefined
     dac:	8f020000 	.inst	0x8f020000 ; undefined
     db0:	03000000 	.inst	0x03000000 ; undefined
     db4:	00000bee 	udf	#3054
     db8:	00000c35 	udf	#3125
     dbc:	6702e502 	.inst	0x6702e502 ; undefined
     dc0:	01000000 	.inst	0x01000000 ; undefined
     dc4:	00013c04 	.inst	0x00013c04 ; undefined
     dc8:	00009800 	udf	#38912
     dcc:	00f80500 	.inst	0x00f80500 ; undefined
     dd0:	02010000 	.inst	0x02010000 ; undefined
     dd4:	015d02e5 	.inst	0x015d02e5 ; undefined
     dd8:	00000000 	udf	#0
     ddc:	00063802 	.inst	0x00063802 ; undefined
     de0:	081c0600 	stxrb	w28, w0, [x16]
     de4:	08100000 	stxrb	w16, w0, [x0]
     de8:	00013c04 	.inst	0x00013c04 ; undefined
     dec:	00009800 	udf	#38912
     df0:	01020700 	.inst	0x01020700 ; undefined
     df4:	00a50000 	.inst	0x00a50000 ; undefined
     df8:	08080000 	stxrb	w8, w0, [x0]
     dfc:	00063d07 	.inst	0x00063d07 ; undefined
     e00:	00014300 	.inst	0x00014300 ; undefined
     e04:	07000800 	.inst	0x07000800 ; undefined
     e08:	00000641 	udf	#1601
     e0c:	000000c8 	udf	#200
     e10:	00000001 	udf	#1
     e14:	01020200 	.inst	0x01020200 ; undefined
     e18:	b6020000 	tbz	x0, #32, 4e18 <_start-0x3fffb1e8>
     e1c:	06000005 	.inst	0x06000005 ; undefined
     e20:	000005c7 	udf	#1479
     e24:	3c040808 	.inst	0x3c040808 ; undefined
     e28:	98000001 	ldrsw	x1, e28 <_start-0x3ffff1d8>
     e2c:	07000000 	.inst	0x07000000 ; undefined
     e30:	000005bf 	udf	#1471
     e34:	00000143 	udf	#323
     e38:	00000008 	udf	#8
     e3c:	06420200 	.inst	0x06420200 ; undefined
     e40:	4d060000 	.inst	0x4d060000 ; undefined
     e44:	00000006 	udf	#6
     e48:	01500401 	.inst	0x01500401 ; undefined
     e4c:	00980000 	.inst	0x00980000 ; undefined
     e50:	00000000 	udf	#0
     e54:	000c3e02 	.inst	0x000c3e02 ; undefined
     e58:	0c440200 	.inst	0x0c440200 ; undefined
     e5c:	3c080000 	stur	b0, [x0, #128]
     e60:	00400002 	.inst	0x00400002 ; undefined
     e64:	34000000 	cbz	w0, e64 <_start-0x3ffff19c>
     e68:	01000000 	.inst	0x01000000 ; undefined
     e6c:	000c4e6f 	.inst	0x000c4e6f ; undefined
     e70:	000cdf00 	.inst	0x000cdf00 ; undefined
     e74:	01430100 	.inst	0x01430100 ; undefined
     e78:	00000067 	udf	#103
     e7c:	08910209 	stllrb	w9, [x16]
     e80:	000000f8 	udf	#248
     e84:	8b014301 	add	x1, x24, x1, lsl #16
     e88:	0a000001 	and	w1, w0, w0
     e8c:	00000039 	udf	#57
     e90:	40000260 	.inst	0x40000260 ; undefined
     e94:	00000000 	udf	#0
     e98:	00000004 	udf	#4
     e9c:	0e014401 	.inst	0x0e014401 ; undefined
     ea0:	1091020b 	adr	x11, fffffffffff22ee0 <stack_top+0xffffffffbff1e9b8>
     ea4:	00000053 	udf	#83
     ea8:	013c0400 	.inst	0x013c0400 ; undefined
     eac:	00980000 	.inst	0x00980000 ; undefined
     eb0:	00000000 	udf	#0
     eb4:	f50c0000 	.inst	0xf50c0000 ; undefined
     eb8:	07000000 	.inst	0x07000000 ; undefined
     ebc:	013c0d01 	.inst	0x013c0d01 ; undefined
     ec0:	00ee0000 	.inst	0x00ee0000 ; undefined
     ec4:	00000000 	udf	#0
     ec8:	3c0d0000 	stur	b0, [x0, #208]
     ecc:	49000001 	.inst	0x49000001 ; undefined
     ed0:	00000006 	udf	#6
     ed4:	06000000 	.inst	0x06000000 ; undefined
     ed8:	00000113 	udf	#275
     edc:	fd070810 	str	d16, [x0, #3600]
     ee0:	7b000000 	.inst	0x7b000000 ; undefined
     ee4:	08000001 	stxrb	w0, w1, [x0]
     ee8:	01060700 	.inst	0x01060700 ; undefined
     eec:	01840000 	.inst	0x01840000 ; undefined
     ef0:	08080000 	stxrb	w8, w0, [x0]
     ef4:	013c0e00 	.inst	0x013c0e00 ; undefined
     ef8:	00000000 	udf	#0
     efc:	0d0c0000 	.inst	0x0d0c0000 ; undefined
     f00:	07000001 	.inst	0x07000001 ; undefined
     f04:	01980d08 	.inst	0x01980d08 ; undefined
     f08:	0d050000 	.inst	0x0d050000 ; undefined
     f0c:	00000000 	udf	#0
     f10:	3c0f0000 	stur	b0, [x0, #240]
     f14:	10000001 	adr	x1, f14 <_start-0x3ffff0ec>
     f18:	000001a5 	udf	#421
     f1c:	11001b00 	add	w0, w24, #0x6
     f20:	00000cf1 	udf	#3313
     f24:	4f000708 	movi	v8.4s, #0x18
     f28:	04000001 	add	z1.b, p0/m, z1.b, z0.b
     f2c:	00029a00 	.inst	0x00029a00 ; undefined
     f30:	00010800 	.inst	0x00010800 ; undefined
     f34:	1c000000 	ldr	s0, f34 <_start-0x3ffff0cc>
     f38:	000d0f00 	.inst	0x000d0f00 ; undefined
     f3c:	0004ec00 	.inst	0x0004ec00 ; undefined
     f40:	00006000 	udf	#24576
     f44:	00027000 	.inst	0x00027000 ; undefined
     f48:	00000040 	udf	#64
     f4c:	00008800 	udf	#34816
     f50:	0d2e0200 	.inst	0x0d2e0200 ; undefined
     f54:	70030000 	adr	x0, 6f57 <_start-0x3fff90a9>
     f58:	00400002 	.inst	0x00400002 ; undefined
     f5c:	88000000 	stxr	w0, w0, [x0]
     f60:	01000000 	.inst	0x01000000 ; undefined
     f64:	000d486f 	.inst	0x000d486f ; undefined
     f68:	040c0100 	sabd	z0.b, p0/m, z0.b, z8.b
     f6c:	00000060 	udf	#96
     f70:	20910205 	.inst	0x20910205 ; undefined
     f74:	00000d51 	udf	#3409
     f78:	310e0101 	adds	w1, w8, #0x380
     f7c:	04000001 	add	z1.b, p0/m, z1.b, z0.b
     f80:	00000090 	udf	#144
     f84:	08910205 	stllrb	w5, [x16]
     f88:	00000638 	udf	#1592
     f8c:	9c0f0101 	ldr	q1, 1efac <_start-0x3ffe1054>
     f90:	06000000 	.inst	0x06000000 ; undefined
     f94:	400002e4 	.inst	0x400002e4 ; undefined
     f98:	00000000 	udf	#0
     f9c:	00000014 	udf	#20
     fa0:	28910205 	stp	w5, w0, [x16], #136
     fa4:	00000d59 	udf	#3417
     fa8:	240f0101 	cmphs	p1.b, p0/z, z8.b, z15.b
     fac:	00000001 	udf	#1
     fb0:	00000000 	udf	#0
     fb4:	00008402 	udf	#33794
     fb8:	00890200 	.inst	0x00890200 ; undefined
     fbc:	38020000 	sturb	w0, [x0, #32]
     fc0:	07000006 	.inst	0x07000006 ; undefined
     fc4:	0000081c 	udf	#2076
     fc8:	10080810 	adr	x16, 110c8 <_start-0x3ffeef38>
     fcc:	98000001 	ldrsw	x1, fcc <_start-0x3ffff034>
     fd0:	09000000 	.inst	0x09000000 ; undefined
     fd4:	00000102 	udf	#258
     fd8:	000000da 	udf	#218
     fdc:	3d090808 	str	b8, [x0, #578]
     fe0:	17000006 	b	fffffffffc000ff8 <stack_top+0xffffffffbbffcad0>
     fe4:	08000001 	stxrb	w0, w1, [x0]
     fe8:	06410900 	.inst	0x06410900 ; undefined
     fec:	00fd0000 	.inst	0x00fd0000 ; undefined
     ff0:	00010000 	.inst	0x00010000 ; undefined
     ff4:	02000000 	.inst	0x02000000 ; undefined
     ff8:	00000102 	udf	#258
     ffc:	0005b602 	.inst	0x0005b602 ; undefined
    1000:	05c70700 	.inst	0x05c70700 ; undefined
    1004:	08080000 	stxrb	w8, w0, [x0]
    1008:	00011008 	.inst	0x00011008 ; undefined
    100c:	00009800 	udf	#38912
    1010:	05bf0900 	uzp1	z0.q, z8.q, z31.q
    1014:	01170000 	.inst	0x01170000 ; undefined
    1018:	00080000 	.inst	0x00080000 ; undefined
    101c:	02000000 	.inst	0x02000000 ; undefined
    1020:	00000642 	udf	#1602
    1024:	00064d07 	.inst	0x00064d07 ; undefined
    1028:	08010000 	stxrb	w1, w0, [x0]
    102c:	00000124 	udf	#292
    1030:	00000098 	udf	#152
    1034:	0a000000 	and	w0, w0, w0
    1038:	000000f5 	udf	#245
    103c:	100b0107 	adr	x7, 1705c <_start-0x3ffe8fa4>
    1040:	ee000001 	.inst	0xee000001 ; undefined
    1044:	00000000 	udf	#0
    1048:	0b000000 	add	w0, w0, w0
    104c:	00000110 	udf	#272
    1050:	00000649 	udf	#1609
    1054:	00000000 	udf	#0
    1058:	00013e0b 	.inst	0x00013e0b ; undefined
    105c:	000d0500 	.inst	0x000d0500 ; undefined
    1060:	00000000 	udf	#0
    1064:	01100c00 	.inst	0x01100c00 ; undefined
    1068:	4b0d0000 	sub	w0, w0, w13
    106c:	00000001 	udf	#1
    1070:	f10e001b 	subs	x27, x0, #0x380
    1074:	0800000c 	stxrb	w0, w12, [x0]
    1078:	地址 0x0000000000001078 越界。


Disassembly of section .debug_aranges:

0000000000000000 <.debug_aranges>:
   0:	0000003c 	udf	#60
   4:	00000002 	udf	#2
   8:	00080000 	.inst	0x00080000 ; undefined
   c:	ffffffff 	.inst	0xffffffff ; undefined
  10:	40000018 	.inst	0x40000018 ; undefined
  14:	00000000 	udf	#0
  18:	000000bc 	udf	#188
  1c:	00000000 	udf	#0
  20:	400000d4 	.inst	0x400000d4 ; undefined
  24:	00000000 	udf	#0
  28:	0000014c 	udf	#332
	...
  40:	0000002c 	udf	#44
  44:	0ced0002 	.inst	0x0ced0002 ; undefined
  48:	00080000 	.inst	0x00080000 ; undefined
  4c:	ffffffff 	.inst	0xffffffff ; undefined
  50:	40000220 	.inst	0x40000220 ; undefined
  54:	00000000 	udf	#0
  58:	0000001c 	udf	#28
	...
  70:	0000002c 	udf	#44
  74:	0d7a0002 	.inst	0x0d7a0002 ; undefined
  78:	00080000 	.inst	0x00080000 ; undefined
  7c:	ffffffff 	.inst	0xffffffff ; undefined
  80:	4000023c 	.inst	0x4000023c ; undefined
  84:	00000000 	udf	#0
  88:	00000034 	udf	#52
	...
  a0:	0000002c 	udf	#44
  a4:	0f270002 	.inst	0x0f270002 ; undefined
  a8:	00080000 	.inst	0x00080000 ; undefined
  ac:	ffffffff 	.inst	0xffffffff ; undefined
  b0:	40000270 	.inst	0x40000270 ; undefined
  b4:	00000000 	udf	#0
  b8:	00000088 	udf	#136
	...

Disassembly of section .debug_ranges:

0000000000000000 <.debug_ranges>:
   0:	4000015c 	.inst	0x4000015c ; undefined
   4:	00000000 	udf	#0
   8:	40000164 	.inst	0x40000164 ; undefined
   c:	00000000 	udf	#0
  10:	40000180 	.inst	0x40000180 ; undefined
  14:	00000000 	udf	#0
  18:	40000214 	.inst	0x40000214 ; undefined
	...
  30:	40000018 	.inst	0x40000018 ; undefined
  34:	00000000 	udf	#0
  38:	400000d4 	.inst	0x400000d4 ; undefined
  3c:	00000000 	udf	#0
  40:	400000d4 	.inst	0x400000d4 ; undefined
  44:	00000000 	udf	#0
  48:	40000220 	.inst	0x40000220 ; undefined
	...
  60:	00000018 	udf	#24
  64:	00000000 	udf	#0
  68:	0000005c 	udf	#92
  6c:	00000000 	udf	#0
  70:	00000068 	udf	#104
  74:	00000000 	udf	#0
  78:	00000088 	udf	#136
	...
  90:	00000024 	udf	#36
  94:	00000000 	udf	#0
  98:	0000005c 	udf	#92
  9c:	00000000 	udf	#0
  a0:	00000068 	udf	#104
  a4:	00000000 	udf	#0
  a8:	00000088 	udf	#136
	...

Disassembly of section .debug_str:

0000000000000000 <.debug_str>:
   0:	6e616c63 	umin	v3.8h, v3.8h, v1.8h
   4:	4c4c2067 	.inst	0x4c4c2067 ; undefined
   8:	28204d56 	stnp	w22, w19, [x10, #-256]
   c:	74737572 	.inst	0x74737572 ; undefined
  10:	65762063 	fmls	z3.h, p0/m, z3.h, z22.h
  14:	6f697372 	fcmla	v18.8h, v27.8h, v9.h[1], #270
  18:	2e31206e 	usubl	v14.8h, v3.8b, v17.8b
  1c:	302e3936 	adr	x22, 5c741 <_start-0x3ffa38bf>
  20:	67696e2d 	.inst	0x67696e2d ; undefined
  24:	796c7468 	ldrh	w8, [x3, #5690]
  28:	37302820 	tbnz	w0, #6, 52c <_start-0x3ffffad4>
  2c:	33393963 	.inst	0x33393963 ; undefined
  30:	20616265 	.inst	0x20616265 ; undefined
  34:	33323032 	.inst	0x33323032 ; undefined
  38:	2d32302d 	stp	s13, s12, [x1, #-112]
  3c:	29293332 	stp	w18, w12, [x25, #-184]
  40:	63727300 	.inst	0x63727300 ; undefined
  44:	69616d2f 	ldpsw	x15, x27, [x9, #-248]
  48:	73722e6e 	.inst	0x73722e6e ; undefined
  4c:	312f402f 	adds	w15, w1, #0xbd0
  50:	74696e78 	.inst	0x74696e78 ; undefined
  54:	3137316f 	adds	w15, w11, #0xdcc
  58:	73647365 	.inst	0x73647365 ; undefined
  5c:	00687163 	.inst	0x00687163 ; undefined
  60:	6f6f722f 	fcmla	v15.8h, v17.8h, v15.h[1], #270
  64:	72612f74 	.inst	0x72612f74 ; undefined
  68:	72612f6d 	.inst	0x72612f6d ; undefined
  6c:	2d38766d 	stp	s13, s29, [x19, #-64]
  70:	65726162 	fnmls	z2.h, p0/m, z11.h, z18.h
  74:	6174656d 	.inst	0x6174656d ; undefined
  78:	65642d6c 	fmls	z12.h, p3/m, z11.h, z4.h
  7c:	722d6f6d 	ands	w13, w27, #0xfff87fff
  80:	00747375 	.inst	0x00747375 ; undefined
  84:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
  88:	696c7300 	ldpsw	x0, x28, [x24, #-160]
  8c:	7b006563 	.inst	0x7b006563 ; undefined
  90:	6c706d69 	ldnp	d9, d27, [x11, #-256]
  94:	007d3023 	.inst	0x007d3023 ; undefined
  98:	5a5f0054 	.inst	0x5a5f0054 ; undefined
  9c:	6f63344e 	ursra	v14.2d, v2.2d, #29
  a0:	73356572 	.inst	0x73356572 ; undefined
  a4:	6563696c 	fnmls	z12.h, p2/m, z11.h, z3.h
  a8:	245f3932 	cmpne	p2.h, p6/z, z9.h, z31.d
  ac:	6924544c 	stgp	x12, x21, [x2, #-896]
  b0:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
  b4:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
  b8:	62357524 	.inst	0x62357524 ; undefined
  bc:	75245424 	.inst	0x75245424 ; undefined
  c0:	24246435 	cmpls	p5.b, p1/z, z1.b, #17
  c4:	36245447 	tbz	w7, #4, ffffffffffff8b4c <stack_top+0xffffffffbfff4624>
  c8:	705f7361 	adr	x1, bef37 <_start-0x3ff410c9>
  cc:	37317274 	tbnz	w20, #6, 2f18 <_start-0x3fffd0e8>
  d0:	33373668 	.inst	0x33373668 ; undefined
  d4:	66323765 	.inst	0x66323765 ; undefined
  d8:	36616634 	tbz	w20, #12, 2d9c <_start-0x3fffd264>
  dc:	63613663 	.inst	0x63613663 ; undefined
  e0:	61004532 	.inst	0x61004532 ; undefined
  e4:	74705f73 	.inst	0x74705f73 ; undefined
  e8:	38753c72 	.inst	0x38753c72 ; undefined
  ec:	632a003e 	.inst	0x632a003e ; undefined
  f0:	74736e6f 	.inst	0x74736e6f ; undefined
  f4:	00387520 	.inst	0x00387520 ; NYI
  f8:	666c6573 	.inst	0x666c6573 ; undefined
  fc:	74616400 	.inst	0x74616400 ; undefined
 100:	74705f61 	.inst	0x74705f61 ; undefined
 104:	656c0072 	fmla	z18.h, p0/m, z3.h, z12.h
 108:	6874676e 	.inst	0x6874676e ; undefined
 10c:	69737500 	ldpsw	x0, x29, [x8, #-104]
 110:	2600657a 	.inst	0x2600657a ; undefined
 114:	5d38755b 	.inst	0x5d38755b ; undefined
 118:	6e6f6300 	rsubhn2	v0.8h, v24.4s, v15.4s
 11c:	705f7473 	adr	x19, befab <_start-0x3ff41055>
 120:	5f007274 	.inst	0x5f007274 ; undefined
 124:	63344e5a 	.inst	0x63344e5a ; undefined
 128:	3365726f 	.inst	0x3365726f ; undefined
 12c:	39727470 	ldrb	w16, [x3, #3229]
 130:	736e6f63 	.inst	0x736e6f63 ; undefined
 134:	74705f74 	.inst	0x74705f74 ; undefined
 138:	5f333372 	.inst	0x5f333372 ; undefined
 13c:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 140:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 144:	30327524 	adr	x4, 64fe9 <_start-0x3ff9b017>
 148:	50422424 	adr	x4, 845ce <_start-0x3ff7ba32>
 14c:	6e6f6324 	rsubhn2	v4.8h, v25.4s, v15.4s
 150:	75247473 	.inst	0x75247473 ; undefined
 154:	54243032 	bc.cs	48758 <_start-0x3ffb78a8>  // bc.hs, bc.nlast
 158:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 15c:	5f736937 	.inst	0x5f736937 ; undefined
 160:	6c6c756e 	ldnp	d14, d29, [x11, #-320]
 164:	34683731 	cbz	w17, d0848 <_start-0x3ff2f7b8>
 168:	62346163 	.inst	0x62346163 ; undefined
 16c:	65366264 	.inst	0x65366264 ; undefined
 170:	35336365 	cbnz	w5, 66ddc <_start-0x3ff99224>
 174:	45386139 	.inst	0x45386139 ; undefined
 178:	5f736900 	.inst	0x5f736900 ; undefined
 17c:	6c6c756e 	ldnp	d14, d29, [x11, #-320]
 180:	3e38753c 	.inst	0x3e38753c ; undefined
 184:	6f6f6200 	umlsl2	v0.4s, v16.8h, v15.h[2]
 188:	5a5f006c 	.inst	0x5a5f006c ; undefined
 18c:	6f63344e 	ursra	v14.2d, v2.2d, #29
 190:	70336572 	adr	x18, 66e3f <_start-0x3ff991c1>
 194:	63397274 	.inst	0x63397274 ; undefined
 198:	74736e6f 	.inst	0x74736e6f ; undefined
 19c:	7274705f 	.inst	0x7274705f ; undefined
 1a0:	245f3333 	cmpne	p3.h, p4/z, z25.h, z31.d
 1a4:	6924544c 	stgp	x12, x21, [x2, #-896]
 1a8:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
 1ac:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 1b0:	24504224 	cmpge	p4.h, p0/z, z17.h, z16.d
 1b4:	736e6f63 	.inst	0x736e6f63 ; undefined
 1b8:	32752474 	.inst	0x32752474 ; undefined
 1bc:	24542430 	cmpne	p0.h, p1/z, z1.h, z20.d
 1c0:	34245447 	cbz	w7, 48c48 <_start-0x3ffb73b8>
 1c4:	72646461 	.inst	0x72646461 ; undefined
 1c8:	38683731 	.inst	0x38683731 ; undefined
 1cc:	61646333 	.inst	0x61646333 ; undefined
 1d0:	32376533 	orr	w19, w9, #0xfffffe07
 1d4:	37366561 	tbnz	w1, #6, ffffffffffffce80 <stack_top+0xffffffffbfff8958>
 1d8:	45636664 	addhnt	z4.b, z19.h, z3.h
 1dc:	64646100 	.inst	0x64646100 ; undefined
 1e0:	38753c72 	.inst	0x38753c72 ; undefined
 1e4:	7369003e 	.inst	0x7369003e ; undefined
 1e8:	6c756e5f 	ldnp	d31, d27, [x18, #-176]
 1ec:	5a5f006c 	.inst	0x5a5f006c ; undefined
 1f0:	6f63344e 	ursra	v14.2d, v2.2d, #29
 1f4:	70336572 	adr	x18, 66ea3 <_start-0x3ff9915d>
 1f8:	63397274 	.inst	0x63397274 ; undefined
 1fc:	74736e6f 	.inst	0x74736e6f ; undefined
 200:	7274705f 	.inst	0x7274705f ; undefined
 204:	245f3333 	cmpne	p3.h, p4/z, z25.h, z31.d
 208:	6924544c 	stgp	x12, x21, [x2, #-896]
 20c:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
 210:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 214:	24504224 	cmpge	p4.h, p0/z, z17.h, z16.d
 218:	736e6f63 	.inst	0x736e6f63 ; undefined
 21c:	32752474 	.inst	0x32752474 ; undefined
 220:	24542430 	cmpne	p0.h, p1/z, z1.h, z20.d
 224:	37245447 	tbnz	w7, #4, ffffffffffff8cac <stack_top+0xffffffffbfff4784>
 228:	6e5f7369 	.inst	0x6e5f7369 ; undefined
 22c:	316c6c75 	adds	w21, w3, #0xb1b, lsl #12
 230:	6e757232 	uabdl2	v18.4s, v17.8h, v21.8h
 234:	656d6974 	fnmls	z20.h, p2/m, z11.h, z13.h
 238:	706d695f 	adr	xzr, daf63 <_start-0x3ff2509d>
 23c:	6837316c 	.inst	0x6837316c ; undefined
 240:	63663763 	.inst	0x63663763 ; undefined
 244:	31623039 	adds	w25, w1, #0x88c, lsl #12
 248:	36373833 	tbz	w19, #6, ffffffffffffe94c <stack_top+0xffffffffbfffa424>
 24c:	63353866 	.inst	0x63353866 ; undefined
 250:	75720045 	.inst	0x75720045 ; undefined
 254:	6d69746e 	ldp	d14, d29, [x3, #-368]
 258:	6d695f65 	ldp	d5, d23, [x27, #-368]
 25c:	5f006c70 	.inst	0x5f006c70 ; undefined
 260:	63344e5a 	.inst	0x63344e5a ; undefined
 264:	3365726f 	.inst	0x3365726f ; undefined
 268:	39727470 	ldrb	w16, [x3, #3229]
 26c:	736e6f63 	.inst	0x736e6f63 ; undefined
 270:	74705f74 	.inst	0x74705f74 ; undefined
 274:	5f333372 	.inst	0x5f333372 ; undefined
 278:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 27c:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 280:	30327524 	adr	x4, 65125 <_start-0x3ff9aedb>
 284:	50422424 	adr	x4, 8470a <_start-0x3ff7b8f6>
 288:	6e6f6324 	rsubhn2	v4.8h, v25.4s, v15.4s
 28c:	75247473 	.inst	0x75247473 ; undefined
 290:	54243032 	bc.cs	48894 <_start-0x3ffb776c>  // bc.hs, bc.nlast
 294:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 298:	64646133 	.inst	0x64646133 ; undefined
 29c:	38683731 	.inst	0x38683731 ; undefined
 2a0:	65633732 	fmls	z18.h, p5/m, z25.h, z3.h
 2a4:	31366461 	adds	w1, w3, #0xd99
 2a8:	36613230 	tbz	w16, #12, 28ec <_start-0x3fffd714>
 2ac:	45623737 	uqshrnt	z23.s, z25.d, #30
 2b0:	756f6300 	.inst	0x756f6300 ; undefined
 2b4:	5f00746e 	.inst	0x5f00746e ; undefined
 2b8:	63344e5a 	.inst	0x63344e5a ; undefined
 2bc:	3365726f 	.inst	0x3365726f ; undefined
 2c0:	39727470 	ldrb	w16, [x3, #3229]
 2c4:	736e6f63 	.inst	0x736e6f63 ; undefined
 2c8:	74705f74 	.inst	0x74705f74 ; undefined
 2cc:	5f333372 	.inst	0x5f333372 ; undefined
 2d0:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 2d4:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 2d8:	30327524 	adr	x4, 6517d <_start-0x3ff9ae83>
 2dc:	50422424 	adr	x4, 84762 <_start-0x3ff7b89e>
 2e0:	6e6f6324 	rsubhn2	v4.8h, v25.4s, v15.4s
 2e4:	75247473 	.inst	0x75247473 ; undefined
 2e8:	54243032 	bc.cs	488ec <_start-0x3ffb7714>  // bc.hs, bc.nlast
 2ec:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 2f0:	66666f36 	.inst	0x66666f36 ; undefined
 2f4:	31746573 	adds	w19, w11, #0xd19, lsl #12
 2f8:	33386837 	.inst	0x33386837 ; undefined
 2fc:	36646239 	tbz	w25, #12, ffffffffffff8f40 <stack_top+0xffffffffbfff4a18>
 300:	39393539 	strb	w25, [x9, #3661]
 304:	31383362 	adds	w2, w27, #0xe0c
 308:	00453435 	.inst	0x00453435 ; undefined
 30c:	7a697369 	.inst	0x7a697369 ; undefined
 310:	00550065 	.inst	0x00550065 ; undefined
 314:	344e5a5f 	cbz	wzr, 9ce5c <_start-0x3ff631a4>
 318:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 31c:	72747033 	.inst	0x72747033 ; undefined
 320:	6e6f6339 	rsubhn2	v25.8h, v25.4s, v15.4s
 324:	705f7473 	adr	x19, bf1b3 <_start-0x3ff40e4d>
 328:	33337274 	.inst	0x33337274 ; undefined
 32c:	544c245f 	bc.nv	987b4 <_start-0x3ff6784c>
 330:	706d6924 	adr	x4, db057 <_start-0x3ff24fa9>
 334:	3275246c 	.inst	0x3275246c ; undefined
 338:	42242430 	.inst	0x42242430 ; undefined
 33c:	6f632450 	urshr	v16.2d, v2.2d, #29
 340:	2474736e 	cmplo	p14.h, p4/z, z27.h, #81
 344:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 348:	54472454 	bc.mi	8e7d0 <_start-0x3ff71830>  // bc.first
 34c:	61633424 	.inst	0x61633424 ; undefined
 350:	37317473 	tbnz	w19, #6, 31dc <_start-0x3fffce24>
 354:	61323368 	.inst	0x61323368 ; undefined
 358:	33633066 	.inst	0x33633066 ; undefined
 35c:	31616639 	adds	w25, w17, #0x859, lsl #12
 360:	66646361 	.inst	0x66646361 ; undefined
 364:	63004538 	.inst	0x63004538 ; undefined
 368:	3c747361 	.inst	0x3c747361 ; undefined
 36c:	202c3875 	.inst	0x202c3875 ; undefined
 370:	003e3875 	.inst	0x003e3875 ; NYI
 374:	344e5a5f 	cbz	wzr, 9cebc <_start-0x3ff63144>
 378:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 37c:	72747033 	.inst	0x72747033 ; undefined
 380:	6e6f6339 	rsubhn2	v25.8h, v25.4s, v15.4s
 384:	705f7473 	adr	x19, bf213 <_start-0x3ff40ded>
 388:	33337274 	.inst	0x33337274 ; undefined
 38c:	544c245f 	bc.nv	98814 <_start-0x3ff677ec>
 390:	706d6924 	adr	x4, db0b7 <_start-0x3ff24f49>
 394:	3275246c 	.inst	0x3275246c ; undefined
 398:	42242430 	.inst	0x42242430 ; undefined
 39c:	6f632450 	urshr	v16.2d, v2.2d, #29
 3a0:	2474736e 	cmplo	p14.h, p4/z, z27.h, #81
 3a4:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 3a8:	54472454 	bc.mi	8e830 <_start-0x3ff717d0>  // bc.first
 3ac:	77373124 	.inst	0x77373124 ; undefined
 3b0:	70706172 	adr	x18, e0fdf <_start-0x3ff1f021>
 3b4:	5f676e69 	.inst	0x5f676e69 ; undefined
 3b8:	65747962 	fnmls	z2.h, p6/m, z11.h, z20.h
 3bc:	6464615f 	.inst	0x6464615f ; undefined
 3c0:	37683731 	tbnz	w17, #13, aa4 <_start-0x3ffff55c>
 3c4:	31653762 	adds	w2, w27, #0x94d, lsl #12
 3c8:	33313564 	.inst	0x33313564 ; undefined
 3cc:	36633061 	tbz	w1, #12, 69d8 <_start-0x3fff9628>
 3d0:	45646563 	addhnt	z3.b, z11.h, z4.h
 3d4:	61727700 	.inst	0x61727700 ; undefined
 3d8:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
 3dc:	79625f67 	ldrh	w7, [x27, #4398]
 3e0:	615f6574 	.inst	0x615f6574 ; undefined
 3e4:	753c6464 	.inst	0x753c6464 ; undefined
 3e8:	5f003e38 	.inst	0x5f003e38 ; undefined
 3ec:	63344e5a 	.inst	0x63344e5a ; undefined
 3f0:	3365726f 	.inst	0x3365726f ; undefined
 3f4:	39727470 	ldrb	w16, [x3, #3229]
 3f8:	736e6f63 	.inst	0x736e6f63 ; undefined
 3fc:	74705f74 	.inst	0x74705f74 ; undefined
 400:	5f333372 	.inst	0x5f333372 ; undefined
 404:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 408:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 40c:	30327524 	adr	x4, 652b1 <_start-0x3ff9ad4f>
 410:	50422424 	adr	x4, 84896 <_start-0x3ff7b76a>
 414:	6e6f6324 	rsubhn2	v4.8h, v25.4s, v15.4s
 418:	75247473 	.inst	0x75247473 ; undefined
 41c:	54243032 	bc.cs	48a20 <_start-0x3ffb75e0>  // bc.hs, bc.nlast
 420:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 424:	72773231 	.inst	0x72773231 ; undefined
 428:	69707061 	ldpsw	x1, x28, [x3, #-128]
 42c:	615f676e 	.inst	0x615f676e ; undefined
 430:	37316464 	tbnz	w4, #6, 30bc <_start-0x3fffcf44>
 434:	38333468 	.inst	0x38333468 ; undefined
 438:	63336630 	.inst	0x63336630 ; undefined
 43c:	38376330 	ldumaxb	w23, w16, [x25]
 440:	63353833 	.inst	0x63353833 ; undefined
 444:	77004533 	.inst	0x77004533 ; undefined
 448:	70706172 	adr	x18, e1077 <_start-0x3ff1ef89>
 44c:	5f676e69 	.inst	0x5f676e69 ; undefined
 450:	3c646461 	.inst	0x3c646461 ; undefined
 454:	003e3875 	.inst	0x003e3875 ; NYI
 458:	344e5a5f 	cbz	wzr, 9cfa0 <_start-0x3ff63060>
 45c:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 460:	72747033 	.inst	0x72747033 ; undefined
 464:	6e6f6339 	rsubhn2	v25.8h, v25.4s, v15.4s
 468:	705f7473 	adr	x19, bf2f7 <_start-0x3ff40d09>
 46c:	33337274 	.inst	0x33337274 ; undefined
 470:	544c245f 	bc.nv	988f8 <_start-0x3ff67708>
 474:	706d6924 	adr	x4, db19b <_start-0x3ff24e65>
 478:	3275246c 	.inst	0x3275246c ; undefined
 47c:	42242430 	.inst	0x42242430 ; undefined
 480:	6f632450 	urshr	v16.2d, v2.2d, #29
 484:	2474736e 	cmplo	p14.h, p4/z, z27.h, #81
 488:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 48c:	54472454 	bc.mi	8e914 <_start-0x3ff716ec>  // bc.first
 490:	77353124 	.inst	0x77353124 ; undefined
 494:	70706172 	adr	x18, e10c3 <_start-0x3ff1ef3d>
 498:	5f676e69 	.inst	0x5f676e69 ; undefined
 49c:	7366666f 	.inst	0x7366666f ; undefined
 4a0:	37317465 	tbnz	w5, #6, 332c <_start-0x3fffccd4>
 4a4:	32363968 	orr	w8, w11, #0x1fffc00
 4a8:	62316432 	.inst	0x62316432 ; undefined
 4ac:	37613862 	tbnz	w2, #12, 2bb8 <_start-0x3fffd448>
 4b0:	65653530 	fmls	z16.h, p5/m, z9.h, z5.h
 4b4:	77004538 	.inst	0x77004538 ; undefined
 4b8:	70706172 	adr	x18, e10e7 <_start-0x3ff1ef19>
 4bc:	5f676e69 	.inst	0x5f676e69 ; undefined
 4c0:	7366666f 	.inst	0x7366666f ; undefined
 4c4:	753c7465 	.inst	0x753c7465 ; undefined
 4c8:	5f003e38 	.inst	0x5f003e38 ; undefined
 4cc:	63344e5a 	.inst	0x63344e5a ; undefined
 4d0:	3365726f 	.inst	0x3365726f ; undefined
 4d4:	39727470 	ldrb	w16, [x3, #3229]
 4d8:	736e6f63 	.inst	0x736e6f63 ; undefined
 4dc:	74705f74 	.inst	0x74705f74 ; undefined
 4e0:	5f333372 	.inst	0x5f333372 ; undefined
 4e4:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 4e8:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 4ec:	30327524 	adr	x4, 65391 <_start-0x3ff9ac6f>
 4f0:	50422424 	adr	x4, 84976 <_start-0x3ff7b68a>
 4f4:	6e6f6324 	rsubhn2	v4.8h, v25.4s, v15.4s
 4f8:	75247473 	.inst	0x75247473 ; undefined
 4fc:	54243032 	bc.cs	48b00 <_start-0x3ffb7500>  // bc.hs, bc.nlast
 500:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 504:	69773631 	ldpsw	x17, x13, [x17, #-72]
 508:	6d5f6874 	ldp	d20, d26, [x3, #496]
 50c:	64617465 	.inst	0x64617465 ; undefined
 510:	5f617461 	sqshl	d1, d3, #33
 514:	3731666f 	tbnz	w15, #6, 31e0 <_start-0x3fffce20>
 518:	63666168 	.inst	0x63666168 ; undefined
 51c:	63333463 	.inst	0x63333463 ; undefined
 520:	63393436 	.inst	0x63393436 ; undefined
 524:	33633866 	.inst	0x33633866 ; undefined
 528:	77004562 	.inst	0x77004562 ; undefined
 52c:	5f687469 	sqshl	d9, d3, #40
 530:	6174656d 	.inst	0x6174656d ; undefined
 534:	61746164 	.inst	0x61746164 ; undefined
 538:	3c666f5f 	.inst	0x3c666f5f ; undefined
 53c:	202c3875 	.inst	0x202c3875 ; undefined
 540:	003e3875 	.inst	0x003e3875 ; NYI
 544:	6174656d 	.inst	0x6174656d ; undefined
 548:	74656d00 	.inst	0x74656d00 ; undefined
 54c:	74616461 	.inst	0x74616461 ; undefined
 550:	5a5f0061 	.inst	0x5a5f0061 ; undefined
 554:	6f63344e 	ursra	v14.2d, v2.2d, #29
 558:	70336572 	adr	x18, 67207 <_start-0x3ff98df9>
 55c:	6d387274 	stp	d20, d28, [x19, #-128]
 560:	64617465 	.inst	0x64617465 ; undefined
 564:	31617461 	adds	w1, w3, #0x85d, lsl #12
 568:	6f726634 	sqshlu	v20.2d, v17.2d, #50
 56c:	61725f6d 	.inst	0x61725f6d ; undefined
 570:	61705f77 	.inst	0x61705f77 ; undefined
 574:	31737472 	adds	w18, w3, #0xcdd, lsl #12
 578:	62626837 	.inst	0x62626837 ; undefined
 57c:	37393163 	tbnz	w3, #7, 2ba8 <_start-0x3fffd458>
 580:	30313534 	adr	x20, 62c25 <_start-0x3ff9d3db>
 584:	38343238 	ldsetb	w20, w24, [x17]
 588:	00453737 	.inst	0x00453737 ; undefined
 58c:	6d6f7266 	ldp	d6, d28, [x19, #-272]
 590:	7761725f 	.inst	0x7761725f ; undefined
 594:	7261705f 	.inst	0x7261705f ; undefined
 598:	753c7374 	.inst	0x753c7374 ; undefined
 59c:	64003e38 	.inst	0x64003e38 ; undefined
 5a0:	5f617461 	sqshl	d1, d3, #33
 5a4:	72646461 	.inst	0x72646461 ; undefined
 5a8:	00737365 	.inst	0x00737365 ; undefined
 5ac:	6e6f632a 	rsubhn2	v10.8h, v25.4s, v15.4s
 5b0:	28207473 	stnp	w19, w29, [x3, #-256]
 5b4:	6f6e0029 	mla	v9.8h, v1.8h, v14.h[2]
 5b8:	756e5f6e 	.inst	0x756e5f6e ; undefined
 5bc:	70006c6c 	adr	x12, 134b <_start-0x3fffecb5>
 5c0:	746e696f 	.inst	0x746e696f ; undefined
 5c4:	4e007265 	tbx	v5.16b, {v19.16b-v22.16b}, v0.16b
 5c8:	754e6e6f 	.inst	0x754e6e6f ; undefined
 5cc:	753c6c6c 	.inst	0x753c6c6c ; undefined
 5d0:	5f003e38 	.inst	0x5f003e38 ; undefined
 5d4:	63344e5a 	.inst	0x63344e5a ; undefined
 5d8:	3365726f 	.inst	0x3365726f ; undefined
 5dc:	38727470 	.inst	0x38727470 ; undefined
 5e0:	5f6e6f6e 	.inst	0x5f6e6f6e ; undefined
 5e4:	6c6c756e 	ldnp	d14, d29, [x11, #-320]
 5e8:	6f4e3631 	ursra	v17.2d, v17.2d, #50
 5ec:	6c754e6e 	ldnp	d14, d19, [x19, #-176]
 5f0:	544c246c 	b.gt	98a7c <_start-0x3ff67584>
 5f4:	47245424 	.inst	0x47245424 ; undefined
 5f8:	33312454 	.inst	0x33312454 ; undefined
 5fc:	5f77656e 	.inst	0x5f77656e ; undefined
 600:	68636e75 	.inst	0x68636e75 ; undefined
 604:	656b6365 	fnmls	z5.h, p0/m, z27.h, z11.h
 608:	68373164 	.inst	0x68373164 ; undefined
 60c:	34643337 	cbz	w23, c8c70 <_start-0x3ff37390>
 610:	39353632 	strb	w18, [x17, #3405]
 614:	66643134 	.inst	0x66643134 ; undefined
 618:	61313336 	.inst	0x61313336 ; undefined
 61c:	656e0045 	fmla	z5.h, p0/m, z2.h, z14.h
 620:	6e755f77 	uqrshl	v23.8h, v27.8h, v21.8h
 624:	63656863 	.inst	0x63656863 ; undefined
 628:	3c64656b 	.inst	0x3c64656b ; undefined
 62c:	003e3875 	.inst	0x003e3875 ; NYI
 630:	74756d2a 	.inst	0x74756d2a ; undefined
 634:	00387520 	.inst	0x00387520 ; NYI
 638:	72657469 	.inst	0x72657469 ; undefined
 63c:	646e6500 	.inst	0x646e6500 ; undefined
 640:	616d5f00 	.inst	0x616d5f00 ; undefined
 644:	72656b72 	.inst	0x72656b72 ; undefined
 648:	38752600 	.inst	0x38752600 ; undefined
 64c:	61685000 	.inst	0x61685000 ; undefined
 650:	6d6f746e 	ldp	d14, d29, [x3, #-272]
 654:	61746144 	.inst	0x61746144 ; undefined
 658:	3875263c 	.inst	0x3875263c ; undefined
 65c:	5a5f003e 	.inst	0x5a5f003e ; undefined
 660:	6f63344e 	ursra	v14.2d, v2.2d, #29
 664:	70336572 	adr	x18, 67313 <_start-0x3ff98ced>
 668:	6e387274 	uabdl2	v20.8h, v19.16b, v24.16b
 66c:	6e5f6e6f 	.inst	0x6e5f6e6f ; undefined
 670:	316c6c75 	adds	w21, w3, #0xb1b, lsl #12
 674:	6e6f4e36 	uqshl	v22.8h, v17.8h, v15.8h
 678:	6c6c754e 	ldnp	d14, d29, [x10, #-320]
 67c:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 680:	54472454 	bc.mi	8eb08 <_start-0x3ff714f8>  // bc.first
 684:	73613624 	.inst	0x73613624 ; undefined
 688:	7274705f 	.inst	0x7274705f ; undefined
 68c:	61683731 	.inst	0x61683731 ; undefined
 690:	33363564 	.inst	0x33363564 ; undefined
 694:	39653865 	ldrb	w5, [x3, #2382]
 698:	66626135 	.inst	0x66626135 ; undefined
 69c:	45386133 	.inst	0x45386133 ; undefined
 6a0:	74756d00 	.inst	0x74756d00 ; undefined
 6a4:	7274705f 	.inst	0x7274705f ; undefined
 6a8:	4e5a5f00 	.inst	0x4e5a5f00 ; undefined
 6ac:	726f6334 	.inst	0x726f6334 ; undefined
 6b0:	74703365 	.inst	0x74703365 ; undefined
 6b4:	756d3772 	.inst	0x756d3772 ; undefined
 6b8:	74705f74 	.inst	0x74705f74 ; undefined
 6bc:	5f313372 	.inst	0x5f313372 ; undefined
 6c0:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 6c4:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 6c8:	30327524 	adr	x4, 6556d <_start-0x3ff9aa93>
 6cc:	50422424 	adr	x4, 84b52 <_start-0x3ff7b4ae>
 6d0:	74756d24 	.inst	0x74756d24 ; undefined
 6d4:	30327524 	adr	x4, 65579 <_start-0x3ff9aa87>
 6d8:	47245424 	.inst	0x47245424 ; undefined
 6dc:	69372454 	stgp	x20, x9, [x2, #-288]
 6e0:	756e5f73 	.inst	0x756e5f73 ; undefined
 6e4:	37316c6c 	tbnz	w12, #6, 3470 <_start-0x3fffcb90>
 6e8:	34646468 	cbz	w8, c9374 <_start-0x3ff36c8c>
 6ec:	62616237 	.inst	0x62616237 ; undefined
 6f0:	62303634 	.inst	0x62303634 ; undefined
 6f4:	32383862 	orr	w2, w3, #0x7fff00
 6f8:	5f004566 	.inst	0x5f004566 ; undefined
 6fc:	63344e5a 	.inst	0x63344e5a ; undefined
 700:	3365726f 	.inst	0x3365726f ; undefined
 704:	37727470 	tbnz	w16, #14, 5590 <_start-0x3fffaa70>
 708:	5f74756d 	sqshl	d13, d11, #52
 70c:	33727470 	.inst	0x33727470 ; undefined
 710:	4c245f31 	.inst	0x4c245f31 ; undefined
 714:	6d692454 	ldp	d20, d9, [x2, #-368]
 718:	75246c70 	.inst	0x75246c70 ; undefined
 71c:	24243032 	cmpls	p2.b, p4/z, z1.b, #16
 720:	6d245042 	stp	d2, d20, [x2, #-448]
 724:	75247475 	.inst	0x75247475 ; undefined
 728:	54243032 	bc.cs	48d2c <_start-0x3ffb72d4>  // bc.hs, bc.nlast
 72c:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 730:	64646134 	.inst	0x64646134 ; undefined
 734:	68373172 	.inst	0x68373172 ; undefined
 738:	35393932 	cbnz	w18, 72e5c <_start-0x3ff8d1a4>
 73c:	65613464 	fmls	z4.h, p5/m, z3.h, z1.h
 740:	38633063 	ldsetlb	w3, w3, [x3]
 744:	38613061 	ldsetlb	w1, w1, [x3]
 748:	5a5f0045 	.inst	0x5a5f0045 ; undefined
 74c:	6f63344e 	ursra	v14.2d, v2.2d, #29
 750:	70336572 	adr	x18, 673ff <_start-0x3ff98c01>
 754:	6d377274 	stp	d20, d28, [x19, #-144]
 758:	705f7475 	adr	x21, bf5e7 <_start-0x3ff40a19>
 75c:	31337274 	adds	w20, w19, #0xcdc
 760:	544c245f 	bc.nv	98be8 <_start-0x3ff67418>
 764:	706d6924 	adr	x4, db48b <_start-0x3ff24b75>
 768:	3275246c 	.inst	0x3275246c ; undefined
 76c:	42242430 	.inst	0x42242430 ; undefined
 770:	756d2450 	.inst	0x756d2450 ; undefined
 774:	32752474 	.inst	0x32752474 ; undefined
 778:	24542430 	cmpne	p0.h, p1/z, z1.h, z20.d
 77c:	37245447 	tbnz	w7, #4, ffffffffffff9204 <stack_top+0xffffffffbfff4cdc>
 780:	6e5f7369 	.inst	0x6e5f7369 ; undefined
 784:	316c6c75 	adds	w21, w3, #0xb1b, lsl #12
 788:	6e757232 	uabdl2	v18.4s, v17.8h, v21.8h
 78c:	656d6974 	fnmls	z20.h, p2/m, z11.h, z13.h
 790:	706d695f 	adr	xzr, db4bb <_start-0x3ff24b45>
 794:	6837316c 	.inst	0x6837316c ; undefined
 798:	35346236 	cbnz	w22, 693dc <_start-0x3ff96c24>
 79c:	62333233 	.inst	0x62333233 ; undefined
 7a0:	66336465 	.inst	0x66336465 ; undefined
 7a4:	30636436 	adr	x22, c7429 <_start-0x3ff38bd7>
 7a8:	5a5f0045 	.inst	0x5a5f0045 ; undefined
 7ac:	6f63344e 	ursra	v14.2d, v2.2d, #29
 7b0:	73356572 	.inst	0x73356572 ; undefined
 7b4:	6563696c 	fnmls	z12.h, p2/m, z11.h, z3.h
 7b8:	65746934 	fnmls	z20.h, p2/m, z9.h, z20.h
 7bc:	49333172 	.inst	0x49333172 ; undefined
 7c0:	24726574 	cmpls	p4.h, p1/z, z11.h, #73
 7c4:	5424544c 	b.gt	4924c <_start-0x3ffb6db4>
 7c8:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 7cc:	6f703431 	ursra	v17.2d, v1.2d, #16
 7d0:	695f7473 	ldpsw	x19, x29, [x3, #248]
 7d4:	735f636e 	.inst	0x735f636e ; undefined
 7d8:	74726174 	.inst	0x74726174 ; undefined
 7dc:	64683731 	.inst	0x64683731 ; undefined
 7e0:	36383338 	tbz	w24, #7, e44 <_start-0x3ffff1bc>
 7e4:	37663864 	tbnz	w4, #12, ffffffffffffcef0 <stack_top+0xffffffffbfff89c8>
 7e8:	33303963 	.inst	0x33303963 ; undefined
 7ec:	45623663 	uqshrnt	z3.s, z19.d, #30
 7f0:	736f7000 	.inst	0x736f7000 ; undefined
 7f4:	6e695f74 	uqrshl	v20.8h, v27.8h, v9.8h
 7f8:	74735f63 	.inst	0x74735f63 ; undefined
 7fc:	3c747261 	.inst	0x3c747261 ; undefined
 800:	003e3875 	.inst	0x003e3875 ; NYI
 804:	74756d26 	.inst	0x74756d26 ; undefined
 808:	726f6320 	.inst	0x726f6320 ; undefined
 80c:	733a3a65 	.inst	0x733a3a65 ; undefined
 810:	6563696c 	fnmls	z12.h, p2/m, z11.h, z3.h
 814:	74693a3a 	.inst	0x74693a3a ; undefined
 818:	3a3a7265 	.inst	0x3a3a7265 ; undefined
 81c:	72657449 	.inst	0x72657449 ; undefined
 820:	3e38753c 	.inst	0x3e38753c ; undefined
 824:	66666f00 	.inst	0x66666f00 ; undefined
 828:	00746573 	.inst	0x00746573 ; undefined
 82c:	00646c6f 	.inst	0x00646c6f ; undefined
 830:	344e5a5f 	cbz	wzr, 9d378 <_start-0x3ff62c88>
 834:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 838:	72747033 	.inst	0x72747033 ; undefined
 83c:	6e6f6339 	rsubhn2	v25.8h, v25.4s, v15.4s
 840:	705f7473 	adr	x19, bf6cf <_start-0x3ff40931>
 844:	33337274 	.inst	0x33337274 ; undefined
 848:	544c245f 	bc.nv	98cd0 <_start-0x3ff67330>
 84c:	706d6924 	adr	x4, db573 <_start-0x3ff24a8d>
 850:	3275246c 	.inst	0x3275246c ; undefined
 854:	42242430 	.inst	0x42242430 ; undefined
 858:	6f632450 	urshr	v16.2d, v2.2d, #29
 85c:	2474736e 	cmplo	p14.h, p4/z, z27.h, #81
 860:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 864:	54472454 	bc.mi	8ecec <_start-0x3ff71314>  // bc.first
 868:	77373124 	.inst	0x77373124 ; undefined
 86c:	70706172 	adr	x18, e149b <_start-0x3ff1eb65>
 870:	5f676e69 	.inst	0x5f676e69 ; undefined
 874:	65747962 	fnmls	z2.h, p6/m, z11.h, z20.h
 878:	6275735f 	.inst	0x6275735f ; undefined
 87c:	65683731 	fmls	z17.h, p5/m, z25.h, z8.h
 880:	30356636 	adr	x22, 6b545 <_start-0x3ff94abb>
 884:	30323538 	adr	x24, 64f29 <_start-0x3ff9b0d7>
 888:	33393930 	.inst	0x33393930 ; undefined
 88c:	45383736 	uqshrnt	z22.h, z25.s, #8
 890:	61727700 	.inst	0x61727700 ; undefined
 894:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
 898:	79625f67 	ldrh	w7, [x27, #4398]
 89c:	735f6574 	.inst	0x735f6574 ; undefined
 8a0:	753c6275 	.inst	0x753c6275 ; undefined
 8a4:	6e003e38 	.inst	0x6e003e38 ; undefined
 8a8:	7b006d75 	.inst	0x7b006d75 ; undefined
 8ac:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 8b0:	007d3523 	.inst	0x007d3523 ; undefined
 8b4:	344e5a5f 	cbz	wzr, 9d3fc <_start-0x3ff62c04>
 8b8:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 8bc:	6d756e33 	ldp	d19, d27, [x17, #-176]
 8c0:	245f3332 	cmpne	p2.h, p4/z, z25.h, z31.d
 8c4:	6924544c 	stgp	x12, x21, [x2, #-896]
 8c8:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
 8cc:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 8d0:	7a697369 	.inst	0x7a697369 ; undefined
 8d4:	54472465 	b.pl	8ed60 <_start-0x3ff712a0>  // b.nfrst
 8d8:	77323124 	.inst	0x77323124 ; undefined
 8dc:	70706172 	adr	x18, e150b <_start-0x3ff1eaf5>
 8e0:	5f676e69 	.inst	0x5f676e69 ; undefined
 8e4:	31627573 	adds	w19, w11, #0x89d, lsl #12
 8e8:	63386837 	.inst	0x63386837 ; undefined
 8ec:	32623761 	.inst	0x32623761 ; undefined
 8f0:	37623438 	tbnz	w24, #12, 4f74 <_start-0x3fffb08c>
 8f4:	39623364 	ldrb	w4, [x27, #2188]
 8f8:	00456232 	.inst	0x00456232 ; undefined
 8fc:	70617277 	adr	x23, c374b <_start-0x3ff3c8b5>
 900:	676e6970 	.inst	0x676e6970 ; undefined
 904:	6275735f 	.inst	0x6275735f ; undefined
 908:	73687200 	.inst	0x73687200 ; undefined
 90c:	4e5a5f00 	.inst	0x4e5a5f00 ; undefined
 910:	726f6334 	.inst	0x726f6334 ; undefined
 914:	756e3365 	.inst	0x756e3365 ; undefined
 918:	5f33326d 	.inst	0x5f33326d ; undefined
 91c:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 920:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 924:	30327524 	adr	x4, 657c9 <_start-0x3ff9a837>
 928:	69736924 	ldpsw	x4, x26, [x9, #-104]
 92c:	4724657a 	.inst	0x4724657a ; undefined
 930:	32312454 	orr	w20, w2, #0x1ff8000
 934:	70617277 	adr	x23, c3783 <_start-0x3ff3c87d>
 938:	676e6970 	.inst	0x676e6970 ; undefined
 93c:	67656e5f 	.inst	0x67656e5f ; undefined
 940:	33683731 	.inst	0x33683731 ; undefined
 944:	35663965 	cbnz	w5, cd070 <_start-0x3ff32f90>
 948:	36363162 	tbz	w2, #6, ffffffffffffcf74 <stack_top+0xffffffffbfff8a4c>
 94c:	62343334 	.inst	0x62343334 ; undefined
 950:	45636231 	addhnb	z17.b, z17.h, z3.h
 954:	61727700 	.inst	0x61727700 ; undefined
 958:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
 95c:	656e5f67 	fnmla	z7.h, p7/m, z27.h, z14.h
 960:	5a5f0067 	.inst	0x5a5f0067 ; undefined
 964:	6f63344e 	ursra	v14.2d, v2.2d, #29
 968:	70336572 	adr	x18, 67617 <_start-0x3ff989e9>
 96c:	63397274 	.inst	0x63397274 ; undefined
 970:	74736e6f 	.inst	0x74736e6f ; undefined
 974:	7274705f 	.inst	0x7274705f ; undefined
 978:	245f3333 	cmpne	p3.h, p4/z, z25.h, z31.d
 97c:	6924544c 	stgp	x12, x21, [x2, #-896]
 980:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
 984:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 988:	24504224 	cmpge	p4.h, p0/z, z17.h, z16.d
 98c:	736e6f63 	.inst	0x736e6f63 ; undefined
 990:	32752474 	.inst	0x32752474 ; undefined
 994:	24542430 	cmpne	p0.h, p1/z, z1.h, z20.d
 998:	31245447 	adds	w7, w2, #0x915
 99c:	61727732 	.inst	0x61727732 ; undefined
 9a0:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
 9a4:	75735f67 	.inst	0x75735f67 ; undefined
 9a8:	68373162 	.inst	0x68373162 ; undefined
 9ac:	36663762 	tbz	w2, #12, ffffffffffffd098 <stack_top+0xffffffffbfff8b70>
 9b0:	34356263 	cbz	w3, 6b5fc <_start-0x3ff94a04>
 9b4:	61306437 	.inst	0x61306437 ; undefined
 9b8:	64343333 	.inst	0x64343333 ; undefined
 9bc:	72770045 	.inst	0x72770045 ; undefined
 9c0:	69707061 	ldpsw	x1, x28, [x3, #-128]
 9c4:	735f676e 	.inst	0x735f676e ; undefined
 9c8:	753c6275 	.inst	0x753c6275 ; undefined
 9cc:	5f003e38 	.inst	0x5f003e38 ; undefined
 9d0:	63344e5a 	.inst	0x63344e5a ; undefined
 9d4:	3365726f 	.inst	0x3365726f ; undefined
 9d8:	37727470 	tbnz	w16, #14, 5864 <_start-0x3fffa79c>
 9dc:	5f74756d 	sqshl	d13, d11, #52
 9e0:	33727470 	.inst	0x33727470 ; undefined
 9e4:	4c245f31 	.inst	0x4c245f31 ; undefined
 9e8:	6d692454 	ldp	d20, d9, [x2, #-368]
 9ec:	75246c70 	.inst	0x75246c70 ; undefined
 9f0:	24243032 	cmpls	p2.b, p4/z, z1.b, #16
 9f4:	6d245042 	stp	d2, d20, [x2, #-448]
 9f8:	75247475 	.inst	0x75247475 ; undefined
 9fc:	54243032 	bc.cs	49000 <_start-0x3ffb7000>  // bc.hs, bc.nlast
 a00:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 a04:	64646133 	.inst	0x64646133 ; undefined
 a08:	36683731 	tbz	w17, #13, 10ec <_start-0x3fffef14>
 a0c:	39396534 	strb	w20, [x9, #3673]
 a10:	36656665 	tbz	w5, #12, ffffffffffffb6dc <stack_top+0xffffffffbfff71b4>
 a14:	66633631 	.inst	0x66633631 ; undefined
 a18:	45616136 	addhnb	z22.b, z9.h, z1.h
 a1c:	4e5a5f00 	.inst	0x4e5a5f00 ; undefined
 a20:	726f6334 	.inst	0x726f6334 ; undefined
 a24:	74703365 	.inst	0x74703365 ; undefined
 a28:	756d3772 	.inst	0x756d3772 ; undefined
 a2c:	74705f74 	.inst	0x74705f74 ; undefined
 a30:	5f313372 	.inst	0x5f313372 ; undefined
 a34:	24544c24 	cmpge	p4.h, p3/z, z1.h, z20.d
 a38:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 a3c:	30327524 	adr	x4, 658e1 <_start-0x3ff9a71f>
 a40:	50422424 	adr	x4, 84ec6 <_start-0x3ff7b13a>
 a44:	74756d24 	.inst	0x74756d24 ; undefined
 a48:	30327524 	adr	x4, 658ed <_start-0x3ff9a713>
 a4c:	47245424 	.inst	0x47245424 ; undefined
 a50:	6f362454 	urshr	v20.4s, v2.4s, #10
 a54:	65736666 	fnmls	z6.h, p1/m, z19.h, z19.h
 a58:	68373174 	.inst	0x68373174 ; undefined
 a5c:	33653465 	.inst	0x33653465 ; undefined
 a60:	37666636 	tbnz	w22, #12, ffffffffffffd724 <stack_top+0xffffffffbfff91fc>
 a64:	63633637 	.inst	0x63633637 ; undefined
 a68:	32343931 	orr	w17, w9, #0x7fff000
 a6c:	697b0045 	ldpsw	x5, x0, [x2, #-40]
 a70:	236c706d 	.inst	0x236c706d ; undefined
 a74:	7d313831 	str	h17, [x1, #6300]
 a78:	4e5a5f00 	.inst	0x4e5a5f00 ; undefined
 a7c:	726f6334 	.inst	0x726f6334 ; undefined
 a80:	6c733565 	ldnp	d5, d13, [x11, #-208]
 a84:	34656369 	cbz	w9, cb6f0 <_start-0x3ff34910>
 a88:	72657469 	.inst	0x72657469 ; undefined
 a8c:	74493331 	.inst	0x74493331 ; undefined
 a90:	4c247265 	.inst	0x4c247265 ; undefined
 a94:	24542454 	cmpne	p4.h, p1/z, z2.h, z20.d
 a98:	33245447 	.inst	0x33245447 ; undefined
 a9c:	3177656e 	adds	w14, w11, #0xdd9, lsl #12
 aa0:	66346837 	.inst	0x66346837 ; undefined
 aa4:	64383939 	.inst	0x64383939 ; undefined
 aa8:	34306139 	cbz	w25, 616cc <_start-0x3ff9e934>
 aac:	38623062 	ldsetlb	w2, w2, [x3]
 ab0:	00453765 	.inst	0x00453765 ; undefined
 ab4:	3c77656e 	.inst	0x3c77656e ; undefined
 ab8:	003e3875 	.inst	0x003e3875 ; NYI
 abc:	394e5a5f 	ldrb	wzr, [x18, #918]
 ac0:	4c245f31 	.inst	0x4c245f31 ; undefined
 ac4:	6f632454 	urshr	v20.2d, v2.2d, #29
 ac8:	2e2e6572 	umax	v18.8b, v11.8b, v14.8b
 acc:	63696c73 	.inst	0x63696c73 ; undefined
 ad0:	692e2e65 	stgp	x5, x11, [x19, #-576]
 ad4:	2e726574 	umax	v20.4h, v11.4h, v18.4h
 ad8:	6574492e 	fnmla	z14.h, p2/m, z9.h, z20.h
 adc:	544c2472 	bc.cs	98f68 <_start-0x3ff67098>  // bc.hs, bc.nlast
 ae0:	47245424 	.inst	0x47245424 ; undefined
 ae4:	75242454 	.inst	0x75242454 ; undefined
 ae8:	61243032 	.inst	0x61243032 ; undefined
 aec:	32752473 	.inst	0x32752473 ; undefined
 af0:	6f632430 	urshr	v16.2d, v1.2d, #29
 af4:	2e2e6572 	umax	v18.8b, v11.8b, v14.8b
 af8:	72657469 	.inst	0x72657469 ; undefined
 afc:	72742e2e 	.inst	0x72742e2e ; undefined
 b00:	73746961 	.inst	0x73746961 ; undefined
 b04:	74692e2e 	.inst	0x74692e2e ; undefined
 b08:	74617265 	.inst	0x74617265 ; undefined
 b0c:	2e2e726f 	uabdl	v15.8h, v19.8b, v14.8b
 b10:	72657449 	.inst	0x72657449 ; undefined
 b14:	726f7461 	.inst	0x726f7461 ; undefined
 b18:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 b1c:	78656e34 	.inst	0x78656e34 ; undefined
 b20:	68373174 	.inst	0x68373174 ; undefined
 b24:	61623436 	.inst	0x61623436 ; undefined
 b28:	33363264 	.inst	0x33363264 ; undefined
 b2c:	39356337 	strb	w23, [x25, #3416]
 b30:	34323535 	cbz	w21, 651d4 <_start-0x3ff9ae2c>
 b34:	656e0045 	fmla	z5.h, p0/m, z2.h, z14.h
 b38:	753c7478 	.inst	0x753c7478 ; undefined
 b3c:	6f003e38 	.inst	0x6f003e38 ; undefined
 b40:	6f697470 	uqshl	v16.2d, v3.2d, #41
 b44:	3675006e 	tbz	w14, #14, ffffffffffffab50 <stack_top+0xffffffffbfff6628>
 b48:	6f4e0034 	mla	v20.8h, v1.8h, v14.h[0]
 b4c:	5300656e 	ubfx	w14, w11, #0, #26
 b50:	00656d6f 	.inst	0x00656d6f ; undefined
 b54:	00305f5f 	.inst	0x00305f5f ; NYI
 b58:	6974704f 	ldpsw	x15, x28, [x2, #-96]
 b5c:	263c6e6f 	.inst	0x263c6e6f ; undefined
 b60:	003e3875 	.inst	0x003e3875 ; NYI
 b64:	2f637273 	fcmla	v19.4h, v19.4h, v3.h[1], #270
 b68:	6e69616d 	rsubhn2	v13.8h, v11.4s, v9.4s
 b6c:	2f73722e 	fcmla	v14.4h, v17.4h, v19.h[1], #270
 b70:	74332f40 	.inst	0x74332f40 ; undefined
 b74:	30793078 	adr	x24, f3181 <_start-0x3ff0ce7f>
 b78:	64693775 	.inst	0x64693775 ; undefined
 b7c:	67696c35 	.inst	0x67696c35 ; undefined
 b80:	5f00776b 	.inst	0x5f00776b ; undefined
 b84:	63344e5a 	.inst	0x63344e5a ; undefined
 b88:	3365726f 	.inst	0x3365726f ; undefined
 b8c:	31727470 	adds	w16, w3, #0xc9d, lsl #12
 b90:	69727734 	ldpsw	x20, x29, [x25, #-112]
 b94:	765f6574 	.inst	0x765f6574 ; undefined
 b98:	74616c6f 	.inst	0x74616c6f ; undefined
 b9c:	31656c69 	adds	w9, w3, #0x95b, lsl #12
 ba0:	33316837 	.inst	0x33316837 ; undefined
 ba4:	31356161 	adds	w1, w11, #0xd58
 ba8:	63633931 	.inst	0x63633931 ; undefined
 bac:	61623937 	.inst	0x61623937 ; undefined
 bb0:	00453932 	.inst	0x00453932 ; undefined
 bb4:	74697277 	.inst	0x74697277 ; undefined
 bb8:	6f765f65 	.inst	0x6f765f65 ; undefined
 bbc:	6974616c 	ldpsw	x12, x24, [x11, #-96]
 bc0:	753c656c 	.inst	0x753c656c ; undefined
 bc4:	64003e38 	.inst	0x64003e38 ; undefined
 bc8:	73007473 	.inst	0x73007473 ; undefined
 bcc:	73006372 	.inst	0x73006372 ; undefined
 bd0:	6d2f6372 	stp	d18, d24, [x27, #-272]
 bd4:	2e6e6961 	.inst	0x2e6e6961 ; undefined
 bd8:	402f7372 	.inst	0x402f7372 ; undefined
 bdc:	3070342f 	adr	x15, e1261 <_start-0x3ff1ed9f>
 be0:	666b786d 	.inst	0x666b786d ; undefined
 be4:	6a6a736e 	bics	w14, w27, w10, lsr #28
 be8:	7a373677 	.inst	0x7a373677 ; undefined
 bec:	5a5f0068 	.inst	0x5a5f0068 ; undefined
 bf0:	6f63344e 	ursra	v14.2d, v2.2d, #29
 bf4:	73356572 	.inst	0x73356572 ; undefined
 bf8:	6563696c 	fnmls	z12.h, p2/m, z11.h, z3.h
 bfc:	245f3932 	cmpne	p2.h, p6/z, z9.h, z31.d
 c00:	6924544c 	stgp	x12, x21, [x2, #-896]
 c04:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
 c08:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 c0c:	62357524 	.inst	0x62357524 ; undefined
 c10:	75245424 	.inst	0x75245424 ; undefined
 c14:	24246435 	cmpls	p5.b, p1/z, z1.b, #17
 c18:	34245447 	cbz	w7, 496a0 <_start-0x3ffb6960>
 c1c:	72657469 	.inst	0x72657469 ; undefined
 c20:	65683731 	fmls	z17.h, p5/m, z25.h, z8.h
 c24:	66323031 	.inst	0x66323031 ; undefined
 c28:	30333733 	adr	x19, 6730d <_start-0x3ff98cf3>
 c2c:	37326664 	tbnz	w4, #6, 58f8 <_start-0x3fffa708>
 c30:	45373337 	uqshrnb	z23.h, z25.s, #9
 c34:	65746900 	fnmls	z0.h, p2/m, z8.h, z20.h
 c38:	38753c72 	.inst	0x38753c72 ; undefined
 c3c:	7261003e 	.inst	0x7261003e ; undefined
 c40:	00796172 	.inst	0x00796172 ; undefined
 c44:	706d697b 	adr	x27, db973 <_start-0x3ff2468d>
 c48:	3331236c 	.inst	0x3331236c ; undefined
 c4c:	5a5f007d 	.inst	0x5a5f007d ; undefined
 c50:	6f63344e 	ursra	v14.2d, v2.2d, #29
 c54:	61356572 	.inst	0x61356572 ; undefined
 c58:	79617272 	ldrh	w18, [x19, #4280]
 c5c:	245f3839 	cmpne	p9.h, p6/z, z1.h, z31.d
 c60:	6924544c 	stgp	x12, x21, [x2, #-896]
 c64:	246c706d 	cmplo	p13.h, p4/z, z3.h, #49
 c68:	24303275 	cmpls	p5.b, p4/z, z19.b, #64
 c6c:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 c70:	74692e2e 	.inst	0x74692e2e ; undefined
 c74:	2e2e7265 	uabdl	v5.8h, v19.8b, v14.8b
 c78:	69617274 	ldpsw	x20, x28, [x19, #-248]
 c7c:	2e2e7374 	uabdl	v20.8h, v27.8b, v14.8b
 c80:	6c6c6f63 	ldnp	d3, d27, [x27, #-320]
 c84:	2e746365 	rsubhn	v5.4h, v27.4s, v20.4s
 c88:	746e492e 	.inst	0x746e492e ; undefined
 c8c:	6574496f 	fnmla	z15.h, p2/m, z11.h, z20.h
 c90:	6f746172 	umlsl2	v18.4s, v11.8h, v4.h[3]
 c94:	32752472 	.inst	0x32752472 ; undefined
 c98:	6f662430 	urshr	v16.2d, v1.2d, #26
 c9c:	32752472 	.inst	0x32752472 ; undefined
 ca0:	52242430 	eor	w16, w1, #0xf000003f
 ca4:	75242446 	.inst	0x75242446 ; undefined
 ca8:	54246235 	bc.pl	498ec <_start-0x3ffb6714>  // bc.nfrst
 cac:	62337524 	.inst	0x62337524 ; undefined
 cb0:	32752424 	.inst	0x32752424 ; undefined
 cb4:	244e2430 	cmpne	p0.h, p1/z, z1.h, z14.d
 cb8:	24643575 	cmpls	p5.h, p5/z, z11.h, #16
 cbc:	24544724 	cmpge	p4.h, p1/z, z25.h, z20.d
 cc0:	746e6939 	.inst	0x746e6939 ; undefined
 cc4:	74695f6f 	.inst	0x74695f6f ; undefined
 cc8:	37317265 	tbnz	w5, #6, 3b14 <_start-0x3fffc4ec>
 ccc:	35616568 	cbnz	w8, c3978 <_start-0x3ff3c688>
 cd0:	38373662 	.inst	0x38373662 ; undefined
 cd4:	33306239 	.inst	0x33306239 ; undefined
 cd8:	39663538 	ldrb	w24, [x9, #2445]
 cdc:	69004539 	stgp	x25, x17, [x9]
 ce0:	5f6f746e 	sqshl	d14, d3, #47
 ce4:	72657469 	.inst	0x72657469 ; undefined
 ce8:	2c38753c 	stnp	s28, s29, [x9, #-64]
 cec:	3e373220 	.inst	0x3e373220 ; undefined
 cf0:	415f5f00 	.inst	0x415f5f00 ; undefined
 cf4:	59415252 	ldapurh	w18, [x18, #21]
 cf8:	5a49535f 	.inst	0x5a49535f ; undefined
 cfc:	59545f45 	.inst	0x59545f45 ; undefined
 d00:	5f5f4550 	.inst	0x5f5f4550 ; undefined
 d04:	755b2600 	.inst	0x755b2600 ; undefined
 d08:	32203b38 	orr	w24, w25, #0x7fff
 d0c:	73005d37 	.inst	0x73005d37 ; undefined
 d10:	6d2f6372 	stp	d18, d24, [x27, #-272]
 d14:	2e6e6961 	.inst	0x2e6e6961 ; undefined
 d18:	402f7372 	.inst	0x402f7372 ; undefined
 d1c:	7475342f 	.inst	0x7475342f ; undefined
 d20:	63683636 	.inst	0x63683636 ; undefined
 d24:	696c7774 	ldpsw	x20, x29, [x27, #-160]
 d28:	64737771 	.inst	0x64737771 ; undefined
 d2c:	72610033 	.inst	0x72610033 ; undefined
 d30:	5f38766d 	sqshl	s13, s19, #24
 d34:	65726162 	fnmls	z2.h, p0/m, z11.h, z18.h
 d38:	6174656d 	.inst	0x6174656d ; undefined
 d3c:	65645f6c 	fnmla	z12.h, p7/m, z27.h, z4.h
 d40:	725f6f6d 	.inst	0x725f6f6d ; undefined
 d44:	00747375 	.inst	0x00747375 ; undefined
 d48:	5f746f6e 	.inst	0x5f746f6e ; undefined
 d4c:	6e69616d 	rsubhn2	v13.8h, v11.4s, v9.4s
 d50:	74756f00 	.inst	0x74756f00 ; undefined
 d54:	7274735f 	.inst	0x7274735f ; undefined
 d58:	74796200 	.inst	0x74796200 ; undefined
 d5c:	地址 0x0000000000000d5c 越界。


Disassembly of section .debug_pubnames:

0000000000000000 <.debug_pubnames>:
   0:	00000234 	udf	#564
   4:	00000002 	udf	#2
   8:	0ced0000 	.inst	0x0ced0000 ; undefined
   c:	0bd50000 	.inst	0x0bd50000 ; undefined
  10:	72770000 	.inst	0x72770000 ; undefined
  14:	69707061 	ldpsw	x1, x28, [x3, #-128]
  18:	6e5f676e 	.inst	0x6e5f676e ; undefined
  1c:	88006765 	stxr	w0, w5, [x27]
  20:	6d00000b 	stp	d11, d0, [x0]
  24:	656b7261 	fnmls	z1.h, p4/m, z19.h, z11.h
  28:	002a0072 	.inst	0x002a0072 ; NYI
  2c:	6f630000 	mla	v0.8h, v0.8h, v3.h[2]
  30:	23006572 	.inst	0x23006572 ; undefined
  34:	77000007 	.inst	0x77000007 ; undefined
  38:	70706172 	adr	x18, e0c67 <_start-0x3ff1f399>
  3c:	5f676e69 	.inst	0x5f676e69 ; undefined
  40:	65747962 	fnmls	z2.h, p6/m, z11.h, z20.h
  44:	6464615f 	.inst	0x6464615f ; undefined
  48:	3e38753c 	.inst	0x3e38753c ; undefined
  4c:	00084900 	.inst	0x00084900 ; undefined
  50:	73616300 	.inst	0x73616300 ; undefined
  54:	38753c74 	.inst	0x38753c74 ; undefined
  58:	3875202c 	ldeorlb	w21, w12, [x1]
  5c:	0aa2003e 	bic	w30, w1, w2, asr #0
  60:	756d0000 	.inst	0x756d0000 ; undefined
  64:	74705f74 	.inst	0x74705f74 ; undefined
  68:	08780072 	.inst	0x08780072 ; undefined
  6c:	72770000 	.inst	0x72770000 ; undefined
  70:	69707061 	ldpsw	x1, x28, [x3, #-128]
  74:	625f676e 	.inst	0x625f676e ; undefined
  78:	5f657479 	sqshl	d25, d3, #37
  7c:	3c627573 	.inst	0x3c627573 ; undefined
  80:	003e3875 	.inst	0x003e3875 ; NYI
  84:	00000b1b 	udf	#2843
  88:	3c646461 	.inst	0x3c646461 ; undefined
  8c:	003e3875 	.inst	0x003e3875 ; NYI
  90:	00000917 	udf	#2327
  94:	68746977 	.inst	0x68746977 ; undefined
  98:	74656d5f 	.inst	0x74656d5f ; undefined
  9c:	74616461 	.inst	0x74616461 ; undefined
  a0:	666f5f61 	.inst	0x666f5f61 ; undefined
  a4:	2c38753c 	stnp	s28, s29, [x9, #-64]
  a8:	3e387520 	.inst	0x3e387520 ; undefined
  ac:	000ba900 	.inst	0x000ba900 ; undefined
  b0:	61727700 	.inst	0x61727700 ; undefined
  b4:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
  b8:	75735f67 	.inst	0x75735f67 ; undefined
  bc:	00980062 	.inst	0x00980062 ; undefined
  c0:	656e0000 	fmla	z0.h, p0/m, z0.h, z14.h
  c4:	38753c77 	.inst	0x38753c77 ; undefined
  c8:	05ef003e 	.inst	0x05ef003e ; undefined
  cc:	74700000 	.inst	0x74700000 ; undefined
  d0:	0af80072 	bic	w18, w3, w24, ror #0
  d4:	73690000 	.inst	0x73690000 ; undefined
  d8:	6c756e5f 	ldnp	d31, d27, [x18, #-176]
  dc:	0bf6006c 	.inst	0x0bf6006c ; undefined
  e0:	706f0000 	adr	x0, de0e3 <_start-0x3ff21f1d>
  e4:	6e6f6974 	.inst	0x6e6f6974 ; undefined
  e8:	00095400 	.inst	0x00095400 ; undefined
  ec:	74656d00 	.inst	0x74656d00 ; undefined
  f0:	74616461 	.inst	0x74616461 ; undefined
  f4:	08230061 	.inst	0x08230061 ; undefined
  f8:	64610000 	fmla	z0.h, z0.h, z1.h[4]
  fc:	753c7264 	.inst	0x753c7264 ; undefined
 100:	a4003e38 	ld1rqb	{z24.b}, p7/z, [x17]
 104:	7b00000b 	.inst	0x7b00000b ; undefined
 108:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 10c:	007d3523 	.inst	0x007d3523 ; undefined
 110:	000009bc 	udf	#2492
 114:	5f6e6f6e 	.inst	0x5f6e6f6e ; undefined
 118:	6c6c756e 	ldnp	d14, d29, [x11, #-320]
 11c:	00031900 	.inst	0x00031900 ; undefined
 120:	78656e00 	.inst	0x78656e00 ; undefined
 124:	38753c74 	.inst	0x38753c74 ; undefined
 128:	0314003e 	.inst	0x0314003e ; undefined
 12c:	697b0000 	.inst	0x697b0000 ; undefined
 130:	236c706d 	.inst	0x236c706d ; undefined
 134:	7d313831 	str	h17, [x1, #6300]
 138:	00098a00 	.inst	0x00098a00 ; undefined
 13c:	6f726600 	sqshlu	v0.2d, v16.2d, #50
 140:	61725f6d 	.inst	0x61725f6d ; undefined
 144:	61705f77 	.inst	0x61705f77 ; undefined
 148:	3c737472 	.inst	0x3c737472 ; undefined
 14c:	003e3875 	.inst	0x003e3875 ; NYI
 150:	00000b50 	udf	#2896
 154:	7366666f 	.inst	0x7366666f ; undefined
 158:	753c7465 	.inst	0x753c7465 ; undefined
 15c:	2f003e38 	.inst	0x2f003e38 ; undefined
 160:	73000000 	.inst	0x73000000 ; undefined
 164:	6563696c 	fnmls	z12.h, p2/m, z11.h, z3.h
 168:	000a5200 	.inst	0x000a5200 ; undefined
 16c:	5f736100 	.inst	0x5f736100 ; undefined
 170:	3c727470 	.inst	0x3c727470 ; undefined
 174:	003e3875 	.inst	0x003e3875 ; NYI
 178:	000002d3 	udf	#723
 17c:	74736f70 	.inst	0x74736f70 ; undefined
 180:	636e695f 	.inst	0x636e695f ; undefined
 184:	6174735f 	.inst	0x6174735f ; undefined
 188:	753c7472 	.inst	0x753c7472 ; undefined
 18c:	62003e38 	.inst	0x62003e38 ; undefined
 190:	69000000 	stgp	x0, x0, [x0]
 194:	00726574 	.inst	0x00726574 ; undefined
 198:	0000066c 	udf	#1644
 19c:	746e7572 	.inst	0x746e7572 ; undefined
 1a0:	5f656d69 	.inst	0x5f656d69 ; undefined
 1a4:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 1a8:	0008e200 	.inst	0x0008e200 ; undefined
 1ac:	61727700 	.inst	0x61727700 ; undefined
 1b0:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
 1b4:	666f5f67 	.inst	0x666f5f67 ; undefined
 1b8:	74657366 	.inst	0x74657366 ; undefined
 1bc:	3e38753c 	.inst	0x3e38753c ; undefined
 1c0:	00075800 	.inst	0x00075800 ; undefined
 1c4:	61727700 	.inst	0x61727700 ; undefined
 1c8:	6e697070 	uabdl2	v16.4s, v3.8h, v9.8h
 1cc:	64615f67 	.inst	0x64615f67 ; undefined
 1d0:	38753c64 	.inst	0x38753c64 ; undefined
 1d4:	08ad003e 	.inst	0x08ad003e ; undefined
 1d8:	72770000 	.inst	0x72770000 ; undefined
 1dc:	69707061 	ldpsw	x1, x28, [x3, #-128]
 1e0:	735f676e 	.inst	0x735f676e ; undefined
 1e4:	753c6275 	.inst	0x753c6275 ; undefined
 1e8:	7a003e38 	.inst	0x7a003e38 ; undefined
 1ec:	6e00000a 	ext	v10.16b, v0.16b, v0.16b, #0
 1f0:	755f7765 	.inst	0x755f7765 ; undefined
 1f4:	6568636e 	fnmls	z14.h, p0/m, z27.h, z8.h
 1f8:	64656b63 	.inst	0x64656b63 ; undefined
 1fc:	3e38753c 	.inst	0x3e38753c ; undefined
 200:	0005f400 	.inst	0x0005f400 ; undefined
 204:	6e6f6300 	rsubhn2	v0.8h, v24.4s, v15.4s
 208:	705f7473 	adr	x19, bf097 <_start-0x3ff40f69>
 20c:	a7007274 	.inst	0xa7007274 ; undefined
 210:	7b00000a 	.inst	0x7b00000a ; undefined
 214:	6c706d69 	ldnp	d9, d27, [x11, #-256]
 218:	007d3023 	.inst	0x007d3023 ; undefined
 21c:	000007fd 	udf	#2045
 220:	6e5f7369 	.inst	0x6e5f7369 ; undefined
 224:	3c6c6c75 	.inst	0x3c6c6c75 ; undefined
 228:	003e3875 	.inst	0x003e3875 ; NYI
 22c:	00000b9f 	udf	#2975
 230:	006d756e 	.inst	0x006d756e ; undefined
 234:	00000000 	udf	#0
 238:	00000036 	udf	#54
 23c:	0ced0002 	.inst	0x0ced0002 ; undefined
 240:	008d0000 	.inst	0x008d0000 ; undefined
 244:	00340000 	.inst	0x00340000 ; NYI
 248:	72770000 	.inst	0x72770000 ; undefined
 24c:	5f657469 	sqshl	d9, d3, #37
 250:	616c6f76 	.inst	0x616c6f76 ; undefined
 254:	656c6974 	fnmls	z20.h, p2/m, z11.h, z12.h
 258:	3e38753c 	.inst	0x3e38753c ; undefined
 25c:	00002f00 	udf	#12032
 260:	72747000 	.inst	0x72747000 ; undefined
 264:	00002a00 	udf	#10752
 268:	726f6300 	.inst	0x726f6300 ; undefined
 26c:	00000065 	udf	#101
 270:	00920000 	.inst	0x00920000 ; undefined
 274:	00020000 	.inst	0x00020000 ; undefined
 278:	00000d7a 	udf	#3450
 27c:	000001ad 	udf	#429
 280:	0000002f 	udf	#47
 284:	63696c73 	.inst	0x63696c73 ; undefined
 288:	00df0065 	.inst	0x00df0065 ; undefined
 28c:	697b0000 	.inst	0x697b0000 ; undefined
 290:	236c706d 	.inst	0x236c706d ; undefined
 294:	007d3331 	.inst	0x007d3331 ; undefined
 298:	000000c3 	udf	#195
 29c:	6b72616d 	.inst	0x6b72616d ; undefined
 2a0:	62007265 	.inst	0x62007265 ; undefined
 2a4:	69000000 	stgp	x0, x0, [x0]
 2a8:	00726574 	.inst	0x00726574 ; undefined
 2ac:	000000a0 	udf	#160
 2b0:	5f6e6f6e 	.inst	0x5f6e6f6e ; undefined
 2b4:	6c6c756e 	ldnp	d14, d29, [x11, #-320]
 2b8:	00009b00 	udf	#39680
 2bc:	72747000 	.inst	0x72747000 ; undefined
 2c0:	00002a00 	udf	#10752
 2c4:	726f6300 	.inst	0x726f6300 ; undefined
 2c8:	00e40065 	.inst	0x00e40065 ; undefined
 2cc:	6e690000 	uaddl2	v0.4s, v0.8h, v9.8h
 2d0:	695f6f74 	ldpsw	x20, x27, [x27, #248]
 2d4:	3c726574 	.inst	0x3c726574 ; undefined
 2d8:	202c3875 	.inst	0x202c3875 ; undefined
 2dc:	003e3732 	.inst	0x003e3732 ; NYI
 2e0:	00000039 	udf	#57
 2e4:	72657469 	.inst	0x72657469 ; undefined
 2e8:	3e38753c 	.inst	0x3e38753c ; undefined
 2ec:	00003400 	udf	#13312
 2f0:	6d697b00 	ldp	d0, d30, [x24, #-368]
 2f4:	30236c70 	adr	x16, 47081 <_start-0x3ffb8f7f>
 2f8:	00da007d 	.inst	0x00da007d ; undefined
 2fc:	72610000 	.inst	0x72610000 ; undefined
 300:	00796172 	.inst	0x00796172 ; undefined
 304:	00000000 	udf	#0
 308:	00000075 	udf	#117
 30c:	0f270002 	.inst	0x0f270002 ; undefined
 310:	01530000 	.inst	0x01530000 ; undefined
 314:	00920000 	.inst	0x00920000 ; undefined
 318:	6c730000 	ldnp	d0, d0, [x0, #-208]
 31c:	00656369 	.inst	0x00656369 ; undefined
 320:	000000f8 	udf	#248
 324:	6b72616d 	.inst	0x6b72616d ; undefined
 328:	97007265 	bl	fffffffffc01ccbc <stack_top+0xffffffffbc018794>
 32c:	69000000 	stgp	x0, x0, [x0]
 330:	00726574 	.inst	0x00726574 ; undefined
 334:	0000002f 	udf	#47
 338:	5f746f6e 	.inst	0x5f746f6e ; undefined
 33c:	6e69616d 	rsubhn2	v13.8h, v11.4s, v9.4s
 340:	0000d000 	udf	#53248
 344:	72747000 	.inst	0x72747000 ; undefined
 348:	0000d500 	udf	#54528
 34c:	6e6f6e00 	umin	v0.8h, v16.8h, v15.8h
 350:	6c756e5f 	ldnp	d31, d27, [x18, #-176]
 354:	008d006c 	.inst	0x008d006c ; undefined
 358:	6f630000 	mla	v0.8h, v0.8h, v3.h[2]
 35c:	2a006572 	orr	w18, w11, w0, lsl #25
 360:	61000000 	.inst	0x61000000 ; undefined
 364:	38766d72 	.inst	0x38766d72 ; undefined
 368:	7261625f 	.inst	0x7261625f ; undefined
 36c:	74656d65 	.inst	0x74656d65 ; undefined
 370:	645f6c61 	fcmla	z1.h, p3/m, z3.h, z31.h, #270
 374:	5f6f6d65 	.inst	0x5f6f6d65 ; undefined
 378:	74737572 	.inst	0x74737572 ; undefined
 37c:	00000000 	udf	#0
	...

Disassembly of section .debug_pubtypes:

0000000000000000 <.debug_pubtypes>:
   0:	000000e2 	udf	#226
   4:	00000002 	udf	#2
   8:	0ced0000 	.inst	0x0ced0000 ; undefined
   c:	0cd80000 	ld4	{v0.8b-v3.8b}, [x0], x24
  10:	6d260000 	stp	d0, d0, [x0, #-416]
  14:	63207475 	.inst	0x63207475 ; undefined
  18:	3a65726f 	.inst	0x3a65726f ; undefined
  1c:	696c733a 	ldpsw	x26, x28, [x25, #-160]
  20:	3a3a6563 	.inst	0x3a3a6563 ; undefined
  24:	72657469 	.inst	0x72657469 ; undefined
  28:	74493a3a 	.inst	0x74493a3a ; undefined
  2c:	753c7265 	.inst	0x753c7265 ; undefined
  30:	b1003e38 	adds	x24, x17, #0xf
  34:	2a00000c 	orr	w12, w0, w0
  38:	736e6f63 	.inst	0x736e6f63 ; undefined
  3c:	29282074 	stp	w20, w8, [x3, #-192]
  40:	000ca300 	.inst	0x000ca300 ; undefined
  44:	69736900 	ldpsw	x0, x26, [x8, #-104]
  48:	e500657a 	stnt1w	{z26.s}, p1, [x11, x0, lsl #2]
  4c:	7500000c 	.inst	0x7500000c ; undefined
  50:	fb003436 	.inst	0xfb003436 ; undefined
  54:	4f00000b 	.inst	0x4f00000b ; undefined
  58:	6f697470 	uqshl	v16.2d, v3.2d, #41
  5c:	75263c6e 	.inst	0x75263c6e ; undefined
  60:	6e003e38 	.inst	0x6e003e38 ; undefined
  64:	2600000c 	.inst	0x2600000c ; undefined
  68:	5d38755b 	.inst	0x5d38755b ; undefined
  6c:	000c9c00 	.inst	0x000c9c00 ; undefined
  70:	6f6f6200 	umlsl2	v0.4s, v16.8h, v15.h[2]
  74:	09c1006c 	.inst	0x09c1006c ; undefined
  78:	6f4e0000 	mla	v0.8h, v0.8h, v14.h[0]
  7c:	6c754e6e 	ldnp	d14, d19, [x19, #-176]
  80:	38753c6c 	.inst	0x38753c6c ; undefined
  84:	0cbe003e 	.inst	0x0cbe003e ; undefined
  88:	6d2a0000 	stp	d0, d0, [x0, #-352]
  8c:	75207475 	.inst	0x75207475 ; undefined
  90:	0c950038 	st4	{v24.8b-v27.8b}, [x1], x21
  94:	73750000 	.inst	0x73750000 ; undefined
  98:	00657a69 	.inst	0x00657a69 ; undefined
  9c:	00000caa 	udf	#3242
  a0:	cb002928 	sub	x8, x9, x0, lsl #10
  a4:	2600000c 	.inst	0x2600000c ; undefined
  a8:	5a003875 	.inst	0x5a003875 ; undefined
  ac:	7500000c 	.inst	0x7500000c ; undefined
  b0:	00670038 	.inst	0x00670038 ; undefined
  b4:	74490000 	.inst	0x74490000 ; undefined
  b8:	753c7265 	.inst	0x753c7265 ; undefined
  bc:	61003e38 	.inst	0x61003e38 ; undefined
  c0:	2a00000c 	orr	w12, w0, w0
  c4:	736e6f63 	.inst	0x736e6f63 ; undefined
  c8:	38752074 	ldeorlb	w21, w20, [x3]
  cc:	000b8d00 	.inst	0x000b8d00 ; undefined
  d0:	61685000 	.inst	0x61685000 ; undefined
  d4:	6d6f746e 	ldp	d14, d29, [x3, #-272]
  d8:	61746144 	.inst	0x61746144 ; undefined
  dc:	3875263c 	.inst	0x3875263c ; undefined
  e0:	0000003e 	udf	#62
  e4:	00210000 	.inst	0x00210000 ; NYI
  e8:	00020000 	.inst	0x00020000 ; undefined
  ec:	00000ced 	udf	#3309
  f0:	0000008d 	udf	#141
  f4:	00000078 	udf	#120
  f8:	7f003875 	.inst	0x7f003875 ; undefined
  fc:	2a000000 	orr	w0, w0, w0
 100:	2074756d 	.inst	0x2074756d ; undefined
 104:	00003875 	udf	#14453
 108:	7f000000 	.inst	0x7f000000 ; undefined
 10c:	02000000 	.inst	0x02000000 ; undefined
 110:	000d7a00 	.inst	0x000d7a00 ; undefined
 114:	0001ad00 	.inst	0x0001ad00 ; undefined
 118:	00018400 	.inst	0x00018400 ; undefined
 11c:	69737500 	ldpsw	x0, x29, [x8, #-104]
 120:	c800657a 	stxr	w0, x26, [x11]
 124:	50000000 	adr	x0, 126 <_start-0x3ffffeda>
 128:	746e6168 	.inst	0x746e6168 ; undefined
 12c:	61446d6f 	.inst	0x61446d6f ; undefined
 130:	263c6174 	.inst	0x263c6174 ; undefined
 134:	003e3875 	.inst	0x003e3875 ; NYI
 138:	0000018b 	udf	#395
 13c:	38755b26 	ldrb	w6, [x25, w21, uxtw #0]
 140:	3732203b 	tbnz	w27, #6, 4544 <_start-0x3fffbabc>
 144:	0150005d 	.inst	0x0150005d ; undefined
 148:	75260000 	.inst	0x75260000 ; undefined
 14c:	00670038 	.inst	0x00670038 ; undefined
 150:	74490000 	.inst	0x74490000 ; undefined
 154:	753c7265 	.inst	0x753c7265 ; undefined
 158:	5d003e38 	.inst	0x5d003e38 ; undefined
 15c:	26000001 	.inst	0x26000001 ; undefined
 160:	5d38755b 	.inst	0x5d38755b ; undefined
 164:	00013c00 	.inst	0x00013c00 ; undefined
 168:	00387500 	.inst	0x00387500 ; NYI
 16c:	000000a5 	udf	#165
 170:	4e6e6f4e 	smin	v14.8h, v26.8h, v14.8h
 174:	3c6c6c75 	.inst	0x3c6c6c75 ; undefined
 178:	003e3875 	.inst	0x003e3875 ; NYI
 17c:	00000143 	udf	#323
 180:	6e6f632a 	rsubhn2	v10.8h, v25.4s, v15.4s
 184:	75207473 	.inst	0x75207473 ; undefined
 188:	00000038 	udf	#56
 18c:	006b0000 	.inst	0x006b0000 ; undefined
 190:	00020000 	.inst	0x00020000 ; undefined
 194:	00000f27 	udf	#3879
 198:	00000153 	udf	#339
 19c:	00000131 	udf	#305
 1a0:	38755b26 	ldrb	w6, [x25, w21, uxtw #0]
 1a4:	3732203b 	tbnz	w27, #6, 45a8 <_start-0x3fffba58>
 1a8:	00fd005d 	.inst	0x00fd005d ; undefined
 1ac:	68500000 	.inst	0x68500000 ; undefined
 1b0:	6f746e61 	.inst	0x6f746e61 ; undefined
 1b4:	7461446d 	.inst	0x7461446d ; undefined
 1b8:	75263c61 	.inst	0x75263c61 ; undefined
 1bc:	24003e38 	cmpne	p8.b, p7/z, z17.b, z0.d
 1c0:	26000001 	.inst	0x26000001 ; undefined
 1c4:	9c003875 	ldr	q21, 8d0 <_start-0x3ffff730>
 1c8:	49000000 	.inst	0x49000000 ; undefined
 1cc:	3c726574 	.inst	0x3c726574 ; undefined
 1d0:	003e3875 	.inst	0x003e3875 ; NYI
 1d4:	00000110 	udf	#272
 1d8:	da003875 	.inst	0xda003875 ; undefined
 1dc:	4e000000 	tbl	v0.16b, {v0.16b}, v0.16b
 1e0:	754e6e6f 	.inst	0x754e6e6f ; undefined
 1e4:	753c6c6c 	.inst	0x753c6c6c ; undefined
 1e8:	17003e38 	b	fffffffffc00fac8 <stack_top+0xffffffffbc00b5a0>
 1ec:	2a000001 	orr	w1, w0, w0
 1f0:	736e6f63 	.inst	0x736e6f63 ; undefined
 1f4:	38752074 	ldeorlb	w21, w20, [x3]
 1f8:	00000000 	udf	#0
	...

Disassembly of section .debug_frame:

0000000000000000 <.debug_frame>:
   0:	00000014 	udf	#20
   4:	ffffffff 	.inst	0xffffffff ; undefined
   8:	00080004 	.inst	0x00080004 ; undefined
   c:	0c1e7c01 	.inst	0x0c1e7c01 ; undefined
  10:	0000001f 	udf	#31
  14:	00000000 	udf	#0
  18:	0000001c 	udf	#28
  1c:	00000000 	udf	#0
  20:	40000018 	.inst	0x40000018 ; undefined
  24:	00000000 	udf	#0
  28:	000000bc 	udf	#188
  2c:	00000000 	udf	#0
  30:	01c00e44 	.inst	0x01c00e44 ; undefined
  34:	00000000 	udf	#0
  38:	0000001c 	udf	#28
  3c:	00000000 	udf	#0
  40:	400000d4 	.inst	0x400000d4 ; undefined
  44:	00000000 	udf	#0
  48:	0000014c 	udf	#332
  4c:	00000000 	udf	#0
  50:	02a00e44 	.inst	0x02a00e44 ; undefined
  54:	00049d44 	.inst	0x00049d44 ; undefined
  58:	00000014 	udf	#20
  5c:	ffffffff 	.inst	0xffffffff ; undefined
  60:	00080004 	.inst	0x00080004 ; undefined
  64:	0c1e7c01 	.inst	0x0c1e7c01 ; undefined
  68:	0000001f 	udf	#31
  6c:	00000000 	udf	#0
  70:	0000001c 	udf	#28
  74:	00000058 	udf	#88
  78:	40000220 	.inst	0x40000220 ; undefined
  7c:	00000000 	udf	#0
  80:	0000001c 	udf	#28
  84:	00000000 	udf	#0
  88:	00100e44 	.inst	0x00100e44 ; undefined
  8c:	00000000 	udf	#0
  90:	00000014 	udf	#20
  94:	ffffffff 	.inst	0xffffffff ; undefined
  98:	00080004 	.inst	0x00080004 ; undefined
  9c:	0c1e7c01 	.inst	0x0c1e7c01 ; undefined
  a0:	0000001f 	udf	#31
  a4:	00000000 	udf	#0
  a8:	0000001c 	udf	#28
  ac:	00000090 	udf	#144
  b0:	4000023c 	.inst	0x4000023c ; undefined
  b4:	00000000 	udf	#0
  b8:	00000034 	udf	#52
  bc:	00000000 	udf	#0
  c0:	44300e44 	mls	z4.h, z18.h, z0.h[2]
  c4:	0000049e 	udf	#1182
  c8:	00000014 	udf	#20
  cc:	ffffffff 	.inst	0xffffffff ; undefined
  d0:	00080004 	.inst	0x00080004 ; undefined
  d4:	0c1e7c01 	.inst	0x0c1e7c01 ; undefined
  d8:	0000001f 	udf	#31
  dc:	00000000 	udf	#0
  e0:	0000001c 	udf	#28
  e4:	000000c8 	udf	#200
  e8:	40000270 	.inst	0x40000270 ; undefined
  ec:	00000000 	udf	#0
  f0:	00000088 	udf	#136
  f4:	00000000 	udf	#0
  f8:	44400e44 	sqdmlslbt	z4.h, z18.b, z0.b
  fc:	0000049e 	udf	#1182

Disassembly of section .debug_line:

0000000000000000 <.debug_line>:
   0:	00000379 	udf	#889
   4:	01a00004 	.inst	0x01a00004 ; undefined
   8:	01010000 	.inst	0x01010000 ; undefined
   c:	0d0efb01 	.inst	0x0d0efb01 ; undefined
  10:	01010100 	.inst	0x01010100 ; undefined
  14:	00000001 	udf	#1
  18:	01000001 	.inst	0x01000001 ; undefined
  1c:	7375722f 	.inst	0x7375722f ; undefined
  20:	302f6374 	adr	x20, 5ec8d <_start-0x3ffa1373>
  24:	39396337 	strb	w23, [x25, #3672]
  28:	61626533 	.inst	0x61626533 ; undefined
  2c:	36376238 	tbz	w24, #6, ffffffffffffec70 <stack_top+0xffffffffbfffa748>
  30:	34656165 	cbz	w5, cac5c <_start-0x3ff353a4>
  34:	39653739 	ldrb	w25, [x25, #2381]
  38:	33333438 	.inst	0x33333438 ; undefined
  3c:	37306561 	tbnz	w1, #6, ce8 <_start-0x3ffff318>
  40:	30306235 	adr	x21, 60c85 <_start-0x3ff9f37b>
  44:	62313066 	.inst	0x62313066 ; undefined
  48:	2f303165 	.inst	0x2f303165 ; undefined
  4c:	7262696c 	.inst	0x7262696c ; undefined
  50:	2f797261 	fcmla	v1.4h, v19.4h, v25.h[1], #270
  54:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
  58:	6372732f 	.inst	0x6372732f ; undefined
  5c:	696c732f 	ldpsw	x15, x28, [x25, #-160]
  60:	2f006563 	mvni	v3.2s, #0xb, lsl #24
  64:	74737572 	.inst	0x74737572 ; undefined
  68:	37302f63 	tbnz	w3, #6, 654 <_start-0x3ffff9ac>
  6c:	33393963 	.inst	0x33393963 ; undefined
  70:	38616265 	ldumaxlb	w1, w5, [x19]
  74:	65363762 	.inst	0x65363762 ; undefined
  78:	39346561 	strb	w1, [x11, #3353]
  7c:	38396537 	.inst	0x38396537 ; undefined
  80:	61333334 	.inst	0x61333334 ; undefined
  84:	35373065 	cbnz	w5, 6e690 <_start-0x3ff91970>
  88:	66303062 	.inst	0x66303062 ; undefined
  8c:	65623130 	fmls	z16.h, p4/m, z9.h, z2.h
  90:	6c2f3031 	stnp	d17, d12, [x1, #-272]
  94:	61726269 	.inst	0x61726269 ; undefined
  98:	632f7972 	.inst	0x632f7972 ; undefined
  9c:	2f65726f 	fcmla	v15.4h, v19.4h, v5.h[1], #270
  a0:	2f637273 	fcmla	v19.4h, v19.4h, v3.h[1], #270
  a4:	00727470 	.inst	0x00727470 ; undefined
  a8:	7375722f 	.inst	0x7375722f ; undefined
  ac:	302f6374 	adr	x20, 5ed19 <_start-0x3ffa12e7>
  b0:	39396337 	strb	w23, [x25, #3672]
  b4:	61626533 	.inst	0x61626533 ; undefined
  b8:	36376238 	tbz	w24, #6, ffffffffffffecfc <stack_top+0xffffffffbfffa7d4>
  bc:	34656165 	cbz	w5, cace8 <_start-0x3ff35318>
  c0:	39653739 	ldrb	w25, [x25, #2381]
  c4:	33333438 	.inst	0x33333438 ; undefined
  c8:	37306561 	tbnz	w1, #6, d74 <_start-0x3ffff28c>
  cc:	30306235 	adr	x21, 60d11 <_start-0x3ff9f2ef>
  d0:	62313066 	.inst	0x62313066 ; undefined
  d4:	2f303165 	.inst	0x2f303165 ; undefined
  d8:	7262696c 	.inst	0x7262696c ; undefined
  dc:	2f797261 	fcmla	v1.4h, v19.4h, v25.h[1], #270
  e0:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
  e4:	6372732f 	.inst	0x6372732f ; undefined
  e8:	696c732f 	ldpsw	x15, x28, [x25, #-160]
  ec:	692f6563 	stgp	x3, x25, [x11, #-544]
  f0:	00726574 	.inst	0x00726574 ; undefined
  f4:	7375722f 	.inst	0x7375722f ; undefined
  f8:	302f6374 	adr	x20, 5ed65 <_start-0x3ffa129b>
  fc:	39396337 	strb	w23, [x25, #3672]
 100:	61626533 	.inst	0x61626533 ; undefined
 104:	36376238 	tbz	w24, #6, ffffffffffffed48 <stack_top+0xffffffffbfffa820>
 108:	34656165 	cbz	w5, cad34 <_start-0x3ff352cc>
 10c:	39653739 	ldrb	w25, [x25, #2381]
 110:	33333438 	.inst	0x33333438 ; undefined
 114:	37306561 	tbnz	w1, #6, dc0 <_start-0x3ffff240>
 118:	30306235 	adr	x21, 60d5d <_start-0x3ff9f2a3>
 11c:	62313066 	.inst	0x62313066 ; undefined
 120:	2f303165 	.inst	0x2f303165 ; undefined
 124:	7262696c 	.inst	0x7262696c ; undefined
 128:	2f797261 	fcmla	v1.4h, v19.4h, v25.h[1], #270
 12c:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 130:	6372732f 	.inst	0x6372732f ; undefined
 134:	6d756e2f 	ldp	d15, d27, [x17, #-176]
 138:	74690000 	.inst	0x74690000 ; undefined
 13c:	722e7265 	ands	w5, w19, #0xfffc7fff
 140:	00010073 	.inst	0x00010073 ; undefined
 144:	646f6d00 	.inst	0x646f6d00 ; undefined
 148:	0073722e 	.inst	0x0073722e ; undefined
 14c:	63000001 	.inst	0x63000001 ; undefined
 150:	74736e6f 	.inst	0x74736e6f ; undefined
 154:	7274705f 	.inst	0x7274705f ; undefined
 158:	0073722e 	.inst	0x0073722e ; undefined
 15c:	6d000002 	stp	d2, d0, [x0]
 160:	64617465 	.inst	0x64617465 ; undefined
 164:	2e617461 	uabd	v1.4h, v3.4h, v1.4h
 168:	02007372 	.inst	0x02007372 ; undefined
 16c:	6f6e0000 	mla	v0.8h, v0.8h, v14.h[2]
 170:	756e5f6e 	.inst	0x756e5f6e ; undefined
 174:	722e6c6c 	ands	w12, w3, #0xfffc3fff
 178:	00020073 	.inst	0x00020073 ; undefined
 17c:	63616d00 	.inst	0x63616d00 ; undefined
 180:	2e736f72 	umin	v18.4h, v27.4h, v19.4h
 184:	03007372 	.inst	0x03007372 ; undefined
 188:	756d0000 	.inst	0x756d0000 ; undefined
 18c:	74705f74 	.inst	0x74705f74 ; undefined
 190:	73722e72 	.inst	0x73722e72 ; undefined
 194:	00000200 	udf	#512
 198:	5f746e69 	.inst	0x5f746e69 ; undefined
 19c:	7263616d 	.inst	0x7263616d ; undefined
 1a0:	722e736f 	ands	w15, w27, #0xfffc7fff
 1a4:	00040073 	.inst	0x00040073 ; undefined
 1a8:	09000000 	.inst	0x09000000 ; undefined
 1ac:	00001802 	udf	#6146
 1b0:	00000040 	udf	#64
 1b4:	00d30300 	.inst	0x00d30300 ; undefined
 1b8:	05020401 	orr	z1.d, z1.d, #0x1ffffffff
 1bc:	89030a09 	.inst	0x89030a09 ; undefined
 1c0:	043c0803 	.inst	0x043c0803 ; undefined
 1c4:	03240503 	.inst	0x03240503 ; undefined
 1c8:	054a7cd6 	.inst	0x054a7cd6 ; undefined
 1cc:	064a0612 	.inst	0x064a0612 ; undefined
 1d0:	82019c03 	.inst	0x82019c03 ; undefined
 1d4:	14050104 	b	1405e4 <_start-0x3febfa1c>
 1d8:	827f8c03 	.inst	0x827f8c03 ; undefined
 1dc:	7fa50306 	.inst	0x7fa50306 ; undefined
 1e0:	03540582 	.inst	0x03540582 ; undefined
 1e4:	048200db 	.inst	0x048200db ; undefined
 1e8:	061e0503 	.inst	0x061e0503 ; undefined
 1ec:	4a06bd03 	.inst	0x4a06bd03 ; undefined
 1f0:	b9031205 	str	w5, [x16, #784]
 1f4:	01044a7c 	.inst	0x01044a7c ; undefined
 1f8:	8a031105 	and	x5, x8, x3, lsl #4
 1fc:	0306827d 	.inst	0x0306827d ; undefined
 200:	054a7fa5 	.inst	0x054a7fa5 ; undefined
 204:	00db0336 	.inst	0x00db0336 ; undefined
 208:	05030482 	orr	z2.d, z2.d, #0xffffffff0000001f
 20c:	60030609 	.inst	0x60030609 ; undefined
 210:	031e054a 	.inst	0x031e054a ; undefined
 214:	054a0881 	.inst	0x054a0881 ; undefined
 218:	7be50312 	.inst	0x7be50312 ; undefined
 21c:	031d054a 	.inst	0x031d054a ; undefined
 220:	04f27cc1 	.inst	0x04f27cc1 ; undefined
 224:	03240504 	.inst	0x03240504 ; undefined
 228:	0e054a14 	.inst	0x0e054a14 ; undefined
 22c:	01044a06 	.inst	0x01044a06 ; undefined
 230:	03061105 	.inst	0x03061105 ; undefined
 234:	0306f265 	.inst	0x0306f265 ; undefined
 238:	054a7fa5 	.inst	0x054a7fa5 ; undefined
 23c:	dd030630 	.inst	0xdd030630 ; undefined
 240:	05044a00 	.inst	0x05044a00 ; undefined
 244:	ec030d05 	.inst	0xec030d05 ; undefined
 248:	01044a00 	.inst	0x01044a00 ; undefined
 24c:	94034005 	bl	d0260 <_start-0x3ff2fda0>
 250:	0d054a7f 	.inst	0x0d054a7f ; undefined
 254:	06054a06 	.inst	0x06054a06 ; undefined
 258:	1002bc06 	adr	x6, 59d8 <_start-0x3fffa628>
 25c:	04010100 	sub	z0.b, p0/m, z0.b, z8.b
 260:	02090006 	.inst	0x02090006 ; undefined
 264:	400000d4 	.inst	0x400000d4 ; undefined
 268:	00000000 	udf	#0
 26c:	0100fb03 	.inst	0x0100fb03 ; undefined
 270:	080a1d05 	stxrb	w10, w5, [x8]
 274:	05050444 	.inst	0x05050444 ; undefined
 278:	01c20309 	.inst	0x01c20309 ; undefined
 27c:	05070482 	.inst	0x05070482 ; undefined
 280:	7dec0324 	.inst	0x7dec0324 ; undefined
 284:	0612054a 	.inst	0x0612054a ; undefined
 288:	a303064a 	.inst	0xa303064a ; undefined
 28c:	06048201 	.inst	0x06048201 ; undefined
 290:	b0031805 	adrp	x5, 6301000 <_start-0x39cff000>
 294:	0306827f 	.inst	0x0306827f ; undefined
 298:	06827efb 	.inst	0x06827efb ; undefined
 29c:	4a018803 	.inst	0x4a018803 ; undefined
 2a0:	7ef80306 	.inst	0x7ef80306 ; undefined
 2a4:	2105ac08 	.inst	0x2105ac08 ; undefined
 2a8:	01860306 	.inst	0x01860306 ; undefined
 2ac:	0503044a 	orr	z10.d, z10.d, #0xffffffff00000007
 2b0:	7fad0324 	.inst	0x7fad0324 ; undefined
 2b4:	06120582 	.inst	0x06120582 ; undefined
 2b8:	9c03064a 	ldr	q10, 6380 <_start-0x3fff9c80>
 2bc:	06048201 	.inst	0x06048201 ; undefined
 2c0:	b6031505 	tbz	x5, #32, 6560 <_start-0x3fff9aa0>
 2c4:	03064a7f 	.inst	0x03064a7f ; undefined
 2c8:	054a7efb 	.inst	0x054a7efb ; undefined
 2cc:	8b03061e 	add	x30, x16, x3, lsl #1
 2d0:	14054a01 	b	152ad4 <_start-0x3fead52c>
 2d4:	05824103 	and	z3.d, z3.d, #0xff00000000000001
 2d8:	823d0319 	.inst	0x823d0319 ; undefined
 2dc:	05491505 	.inst	0x05491505 ; undefined
 2e0:	0306500e 	.inst	0x0306500e ; undefined
 2e4:	05f27ef2 	.inst	0x05f27ef2 ; undefined
 2e8:	3903061d 	strb	w29, [x16, #193]
 2ec:	0503044a 	orr	z10.d, z10.d, #0xffffffff00000007
 2f0:	08048409 	stlxrb	w4, w9, [x0]
 2f4:	e0030d05 	ld1b	{za0h.b[w12, 5]}, p3/z, [x8, x3]
 2f8:	03048208 	.inst	0x03048208 ; undefined
 2fc:	86031205 	.inst	0x86031205 ; undefined
 300:	1d054a7b 	.inst	0x1d054a7b ; undefined
 304:	f27cc103 	ands	x3, x8, #0x1ffffffffffff0
 308:	24050404 	cmphs	p4.b, p1/z, z0.b, z5.b
 30c:	054a1403 	.inst	0x054a1403 ; undefined
 310:	044a060e 	smin	z14.h, p1/m, z14.h, z16.h
 314:	06110506 	.inst	0x06110506 ; undefined
 318:	05ba4303 	.inst	0x05ba4303 ; undefined
 31c:	4a150315 	eor	w21, w24, w21
 320:	06b81105 	.inst	0x06b81105 ; undefined
 324:	4a7fb403 	.inst	0x4a7fb403 ; undefined
 328:	03061f05 	.inst	0x03061f05 ; undefined
 32c:	044a00d0 	smin	z16.h, p0/m, z16.h, z6.h
 330:	03090505 	.inst	0x03090505 ; undefined
 334:	048201f6 	.inst	0x048201f6 ; undefined
 338:	03400506 	.inst	0x03400506 ; undefined
 33c:	044a7e8d 	mls	z13.h, p7/m, z20.h, z10.h
 340:	03090505 	.inst	0x03090505 ; undefined
 344:	048201f3 	.inst	0x048201f3 ; undefined
 348:	031e0507 	.inst	0x031e0507 ; undefined
 34c:	058205b6 	and	z22.d, z22.d, #0x3fffffffffff
 350:	7be30312 	.inst	0x7be30312 ; undefined
 354:	0505044a 	.inst	0x0505044a ; undefined
 358:	7dea030d 	.inst	0x7dea030d ; undefined
 35c:	050604f2 	.inst	0x050604f2 ; undefined
 360:	7f8a0315 	.inst	0x7f8a0315 ; undefined
 364:	1105834a 	add	w10, w26, #0x160
 368:	054a7803 	.inst	0x054a7803 ; undefined
 36c:	4a3f031e 	eon	w30, w24, wzr
 370:	4a061905 	eor	w5, w8, w6, lsl #6
 374:	47061505 	.inst	0x47061505 ; undefined
 378:	01000402 	.inst	0x01000402 ; undefined
 37c:	00008401 	udf	#33793
 380:	63000400 	.inst	0x63000400 ; undefined
 384:	01000000 	.inst	0x01000000 ; undefined
 388:	0efb0101 	.inst	0x0efb0101 ; undefined
 38c:	0101000d 	.inst	0x0101000d ; undefined
 390:	00000101 	udf	#257
 394:	00000100 	udf	#256
 398:	75722f01 	.inst	0x75722f01 ; undefined
 39c:	2f637473 	.inst	0x2f637473 ; undefined
 3a0:	39633730 	ldrb	w16, [x25, #2253]
 3a4:	62653339 	.inst	0x62653339 ; undefined
 3a8:	37623861 	tbnz	w1, #12, 4ab4 <_start-0x3fffb54c>
 3ac:	65616536 	fnmls	z22.h, p1/m, z9.h, z1.h
 3b0:	65373934 	.inst	0x65373934 ; undefined
 3b4:	33343839 	.inst	0x33343839 ; undefined
 3b8:	30656133 	adr	x19, cafdd <_start-0x3ff35023>
 3bc:	30623537 	adr	x23, c4a61 <_start-0x3ff3b59f>
 3c0:	31306630 	adds	w16, w17, #0xc19
 3c4:	30316562 	adr	x2, 63071 <_start-0x3ff9cf8f>
 3c8:	62696c2f 	.inst	0x62696c2f ; undefined
 3cc:	79726172 	ldrh	w18, [x11, #6448]
 3d0:	726f632f 	.inst	0x726f632f ; undefined
 3d4:	72732f65 	.inst	0x72732f65 ; undefined
 3d8:	74702f63 	.inst	0x74702f63 ; undefined
 3dc:	6d000072 	stp	d18, d0, [x3]
 3e0:	722e646f 	ands	w15, w3, #0xfffc0fff
 3e4:	00010073 	.inst	0x00010073 ; undefined
 3e8:	09000000 	.inst	0x09000000 ; undefined
 3ec:	00022002 	.inst	0x00022002 ; undefined
 3f0:	00000040 	udf	#64
 3f4:	0cb30300 	.inst	0x0cb30300 ; undefined
 3f8:	0a090501 	and	w1, w8, w9, lsl #1
 3fc:	4c0205f9 	.inst	0x4c0205f9 ; undefined
 400:	01000802 	.inst	0x01000802 ; undefined
 404:	0000e301 	udf	#58113
 408:	b6000400 	tbz	x0, #32, 488 <_start-0x3ffffb78>
 40c:	01000000 	.inst	0x01000000 ; undefined
 410:	0efb0101 	.inst	0x0efb0101 ; undefined
 414:	0101000d 	.inst	0x0101000d ; undefined
 418:	00000101 	udf	#257
 41c:	00000100 	udf	#256
 420:	75722f01 	.inst	0x75722f01 ; undefined
 424:	2f637473 	.inst	0x2f637473 ; undefined
 428:	39633730 	ldrb	w16, [x25, #2253]
 42c:	62653339 	.inst	0x62653339 ; undefined
 430:	37623861 	tbnz	w1, #12, 4b3c <_start-0x3fffb4c4>
 434:	65616536 	fnmls	z22.h, p1/m, z9.h, z1.h
 438:	65373934 	.inst	0x65373934 ; undefined
 43c:	33343839 	.inst	0x33343839 ; undefined
 440:	30656133 	adr	x19, cb065 <_start-0x3ff34f9b>
 444:	30623537 	adr	x23, c4ae9 <_start-0x3ff3b517>
 448:	31306630 	adds	w16, w17, #0xc19
 44c:	30316562 	adr	x2, 630f9 <_start-0x3ff9cf07>
 450:	62696c2f 	.inst	0x62696c2f ; undefined
 454:	79726172 	ldrh	w18, [x11, #6448]
 458:	726f632f 	.inst	0x726f632f ; undefined
 45c:	72732f65 	.inst	0x72732f65 ; undefined
 460:	72612f63 	.inst	0x72612f63 ; undefined
 464:	00796172 	.inst	0x00796172 ; undefined
 468:	7375722f 	.inst	0x7375722f ; undefined
 46c:	302f6374 	adr	x20, 5f0d9 <_start-0x3ffa0f27>
 470:	39396337 	strb	w23, [x25, #3672]
 474:	61626533 	.inst	0x61626533 ; undefined
 478:	36376238 	tbz	w24, #6, fffffffffffff0bc <stack_top+0xffffffffbfffab94>
 47c:	34656165 	cbz	w5, cb0a8 <_start-0x3ff34f58>
 480:	39653739 	ldrb	w25, [x25, #2381]
 484:	33333438 	.inst	0x33333438 ; undefined
 488:	37306561 	tbnz	w1, #6, 1134 <_start-0x3fffeecc>
 48c:	30306235 	adr	x21, 610d1 <_start-0x3ff9ef2f>
 490:	62313066 	.inst	0x62313066 ; undefined
 494:	2f303165 	.inst	0x2f303165 ; undefined
 498:	7262696c 	.inst	0x7262696c ; undefined
 49c:	2f797261 	fcmla	v1.4h, v19.4h, v25.h[1], #270
 4a0:	65726f63 	fnmls	z3.h, p3/m, z27.h, z18.h
 4a4:	6372732f 	.inst	0x6372732f ; undefined
 4a8:	696c732f 	ldpsw	x15, x28, [x25, #-160]
 4ac:	00006563 	udf	#25955
 4b0:	2e646f6d 	umin	v13.4h, v27.4h, v4.4h
 4b4:	01007372 	.inst	0x01007372 ; undefined
 4b8:	6f6d0000 	mla	v0.8h, v0.8h, v13.h[2]
 4bc:	73722e64 	.inst	0x73722e64 ; undefined
 4c0:	00000200 	udf	#512
 4c4:	02090000 	.inst	0x02090000 ; undefined
 4c8:	4000023c 	.inst	0x4000023c ; undefined
 4cc:	00000000 	udf	#0
 4d0:	0102c203 	.inst	0x0102c203 ; undefined
 4d4:	f30a0905 	.inst	0xf30a0905 ; undefined
 4d8:	a2030204 	.inst	0xa2030204 ; undefined
 4dc:	043c0803 	.inst	0x043c0803 ; undefined
 4e0:	03060501 	.inst	0x03060501 ; undefined
 4e4:	024a7cdf 	.inst	0x024a7cdf ; undefined
 4e8:	0101000c 	.inst	0x0101000c ; undefined
 4ec:	0000005f 	udf	#95
 4f0:	00230004 	.inst	0x00230004 ; NYI
 4f4:	01010000 	.inst	0x01010000 ; undefined
 4f8:	0d0efb01 	.inst	0x0d0efb01 ; undefined
 4fc:	01010100 	.inst	0x01010100 ; undefined
 500:	00000001 	udf	#1
 504:	01000001 	.inst	0x01000001 ; undefined
 508:	00637273 	.inst	0x00637273 ; undefined
 50c:	69616d00 	ldpsw	x0, x27, [x8, #-248]
 510:	73722e6e 	.inst	0x73722e6e ; undefined
 514:	00000100 	udf	#256
 518:	02090000 	.inst	0x02090000 ; undefined
 51c:	40000270 	.inst	0x40000270 ; undefined
 520:	00000000 	udf	#0
 524:	05010b03 	orr	z3.s, z3.s, #0x80ffffff
 528:	05f40a13 	.inst	0x05f40a13 ; undefined
 52c:	05058311 	.inst	0x05058311 ; undefined
 530:	7103ba06 	subs	w6, w16, #0xee
 534:	0311054a 	.inst	0x0311054a ; undefined
 538:	02054a0f 	.inst	0x02054a0f ; undefined
 53c:	17300206 	b	fffffffffcc00d54 <stack_top+0xffffffffbcbfc82c>
 540:	05b50905 	uzp1	z5.q, z8.q, z21.q
 544:	0d05bc28 	.inst	0x0d05bc28 ; undefined
 548:	0802ba06 	stlxrb	w2, w6, [x16]
 54c:	地址 0x000000000000054c 越界。

