
gic-demo-linked.o：     文件格式 elf64-littleaarch64


Disassembly of section .boot:

0000000000000000 <__reset_entry>:
__reset_entry():
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:47
.endm

	.section ".boot", "ax"
	.globl __reset_entry
__reset_entry:
	ldr	x0, =vectors
   0:	58007c40 	ldr	x0, f88 <vectors+0x788>
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:48
	msr	vbar_el1, x0
   4:	d518c000 	msr	vbar_el1, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:50

	ldr	x0, =stack_top
   8:	58007c40 	ldr	x0, f90 <vectors+0x790>
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:51
	mov	sp, x0
   c:	9100001f 	mov	sp, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:53

	mov	x0, #(3 << 20)
  10:	d2a00600 	mov	x0, #0x300000              	// #3145728
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:54
	msr	cpacr_el1, x0
  14:	d5181040 	msr	cpacr_el1, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:56

	msr	daif, xzr
  18:	d51b423f 	msr	daif, xzr
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:58

	isb
  1c:	d5033fdf 	isb
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:60

	b	c_entry
  20:	14000fc2 	b	3f28 <c_entry>

0000000000000024 <handle_irq>:
handle_irq():
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:63

handle_irq:
	sub sp, sp, #(16 * 16)
  24:	d10403ff 	sub	sp, sp, #0x100
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:64
	stp x0, x1, [sp, #(0 * 16)]
  28:	a90007e0 	stp	x0, x1, [sp]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:65
	stp x2, x3, [sp, #(1 * 16)]
  2c:	a9010fe2 	stp	x2, x3, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:66
	stp x4, x5, [sp, #(2 * 16)]
  30:	a90217e4 	stp	x4, x5, [sp, #32]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:67
	stp x6, x7, [sp, #(3 * 16)]
  34:	a9031fe6 	stp	x6, x7, [sp, #48]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:68
	stp x8, x9, [sp, #(4 * 16)]
  38:	a90427e8 	stp	x8, x9, [sp, #64]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:69
	stp x10, x11, [sp, #(5 * 16)]
  3c:	a9052fea 	stp	x10, x11, [sp, #80]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:70
	stp x12, x13, [sp, #(6 * 16)]
  40:	a90637ec 	stp	x12, x13, [sp, #96]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:71
	stp x14, x15, [sp, #(7 * 16)]
  44:	a9073fee 	stp	x14, x15, [sp, #112]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:72
	stp x16, x17, [sp, #(8 * 16)]
  48:	a90847f0 	stp	x16, x17, [sp, #128]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:73
	stp x18, x19, [sp, #(9 * 16)]
  4c:	a9094ff2 	stp	x18, x19, [sp, #144]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:74
	stp x20, x21, [sp, #(10 * 16)]
  50:	a90a57f4 	stp	x20, x21, [sp, #160]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:75
	stp x22, x23, [sp, #(11 * 16)]
  54:	a90b5ff6 	stp	x22, x23, [sp, #176]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:76
	stp x24, x25, [sp, #(12 * 16)]
  58:	a90c67f8 	stp	x24, x25, [sp, #192]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:77
	stp x26, x27, [sp, #(13 * 16)]
  5c:	a90d6ffa 	stp	x26, x27, [sp, #208]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:78
	stp x28, x29, [sp, #(14 * 16)]
  60:	a90e77fc 	stp	x28, x29, [sp, #224]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:79
	str x30, [sp, #(15 * 16)]
  64:	f9007bfe 	str	x30, [sp, #240]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:81

	bl	vector_irq
  68:	94000f2c 	bl	3d18 <vector_irq>
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:83

	ldp x0, x1, [sp, #(0 * 16)]
  6c:	a94007e0 	ldp	x0, x1, [sp]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:84
	ldp x2, x3, [sp, #(1 * 16)]
  70:	a9410fe2 	ldp	x2, x3, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:85
	ldp x4, x5, [sp, #(2 * 16)]
  74:	a94217e4 	ldp	x4, x5, [sp, #32]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:86
	ldp x6, x7, [sp, #(3 * 16)]
  78:	a9431fe6 	ldp	x6, x7, [sp, #48]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:87
	ldp x8, x9, [sp, #(4 * 16)]
  7c:	a94427e8 	ldp	x8, x9, [sp, #64]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:88
	ldp x10, x11, [sp, #(5 * 16)]
  80:	a9452fea 	ldp	x10, x11, [sp, #80]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:89
	ldp x12, x13, [sp, #(6 * 16)]
  84:	a94637ec 	ldp	x12, x13, [sp, #96]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:90
	ldp x14, x15, [sp, #(7 * 16)]
  88:	a9473fee 	ldp	x14, x15, [sp, #112]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:91
	ldp x16, x17, [sp, #(8 * 16)]
  8c:	a94847f0 	ldp	x16, x17, [sp, #128]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:92
	ldp x18, x19, [sp, #(9 * 16)]
  90:	a9494ff2 	ldp	x18, x19, [sp, #144]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:93
	ldp x20, x21, [sp, #(10 * 16)]
  94:	a94a57f4 	ldp	x20, x21, [sp, #160]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:94
	ldp x22, x23, [sp, #(11 * 16)]
  98:	a94b5ff6 	ldp	x22, x23, [sp, #176]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:95
	ldp x24, x25, [sp, #(12 * 16)]
  9c:	a94c67f8 	ldp	x24, x25, [sp, #192]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:96
	ldp x26, x27, [sp, #(13 * 16)]
  a0:	a94d6ffa 	ldp	x26, x27, [sp, #208]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:97
	ldp x28, x29, [sp, #(14 * 16)]
  a4:	a94e77fc 	ldp	x28, x29, [sp, #224]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:98
	ldr x30, [sp, #(15 * 16)]
  a8:	f9407bfe 	ldr	x30, [sp, #240]
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:99
	add sp, sp, #(16 * 16)
  ac:	910403ff 	add	sp, sp, #0x100
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:101

	eret
  b0:	d69f03e0 	eret
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:104

.weak vector_irq
	b	.
  b4:	14000000 	b	b4 <handle_irq+0x90>
  b8:	d503201f 	nop
  bc:	d503201f 	nop
  c0:	d503201f 	nop
  c4:	d503201f 	nop
  c8:	d503201f 	nop
  cc:	d503201f 	nop
  d0:	d503201f 	nop
  d4:	d503201f 	nop
  d8:	d503201f 	nop
  dc:	d503201f 	nop
  e0:	d503201f 	nop
  e4:	d503201f 	nop
  e8:	d503201f 	nop
  ec:	d503201f 	nop
  f0:	d503201f 	nop
  f4:	d503201f 	nop
  f8:	d503201f 	nop
  fc:	d503201f 	nop
 100:	d503201f 	nop
 104:	d503201f 	nop
 108:	d503201f 	nop
 10c:	d503201f 	nop
 110:	d503201f 	nop
 114:	d503201f 	nop
 118:	d503201f 	nop
 11c:	d503201f 	nop
 120:	d503201f 	nop
 124:	d503201f 	nop
 128:	d503201f 	nop
 12c:	d503201f 	nop
 130:	d503201f 	nop
 134:	d503201f 	nop
 138:	d503201f 	nop
 13c:	d503201f 	nop
 140:	d503201f 	nop
 144:	d503201f 	nop
 148:	d503201f 	nop
 14c:	d503201f 	nop
 150:	d503201f 	nop
 154:	d503201f 	nop
 158:	d503201f 	nop
 15c:	d503201f 	nop
 160:	d503201f 	nop
 164:	d503201f 	nop
 168:	d503201f 	nop
 16c:	d503201f 	nop
 170:	d503201f 	nop
 174:	d503201f 	nop
 178:	d503201f 	nop
 17c:	d503201f 	nop
 180:	d503201f 	nop
 184:	d503201f 	nop
 188:	d503201f 	nop
 18c:	d503201f 	nop
 190:	d503201f 	nop
 194:	d503201f 	nop
 198:	d503201f 	nop
 19c:	d503201f 	nop
 1a0:	d503201f 	nop
 1a4:	d503201f 	nop
 1a8:	d503201f 	nop
 1ac:	d503201f 	nop
 1b0:	d503201f 	nop
 1b4:	d503201f 	nop
 1b8:	d503201f 	nop
 1bc:	d503201f 	nop
 1c0:	d503201f 	nop
 1c4:	d503201f 	nop
 1c8:	d503201f 	nop
 1cc:	d503201f 	nop
 1d0:	d503201f 	nop
 1d4:	d503201f 	nop
 1d8:	d503201f 	nop
 1dc:	d503201f 	nop
 1e0:	d503201f 	nop
 1e4:	d503201f 	nop
 1e8:	d503201f 	nop
 1ec:	d503201f 	nop
 1f0:	d503201f 	nop
 1f4:	d503201f 	nop
 1f8:	d503201f 	nop
 1fc:	d503201f 	nop
 200:	d503201f 	nop
 204:	d503201f 	nop
 208:	d503201f 	nop
 20c:	d503201f 	nop
 210:	d503201f 	nop
 214:	d503201f 	nop
 218:	d503201f 	nop
 21c:	d503201f 	nop
 220:	d503201f 	nop
 224:	d503201f 	nop
 228:	d503201f 	nop
 22c:	d503201f 	nop
 230:	d503201f 	nop
 234:	d503201f 	nop
 238:	d503201f 	nop
 23c:	d503201f 	nop
 240:	d503201f 	nop
 244:	d503201f 	nop
 248:	d503201f 	nop
 24c:	d503201f 	nop
 250:	d503201f 	nop
 254:	d503201f 	nop
 258:	d503201f 	nop
 25c:	d503201f 	nop
 260:	d503201f 	nop
 264:	d503201f 	nop
 268:	d503201f 	nop
 26c:	d503201f 	nop
 270:	d503201f 	nop
 274:	d503201f 	nop
 278:	d503201f 	nop
 27c:	d503201f 	nop
 280:	d503201f 	nop
 284:	d503201f 	nop
 288:	d503201f 	nop
 28c:	d503201f 	nop
 290:	d503201f 	nop
 294:	d503201f 	nop
 298:	d503201f 	nop
 29c:	d503201f 	nop
 2a0:	d503201f 	nop
 2a4:	d503201f 	nop
 2a8:	d503201f 	nop
 2ac:	d503201f 	nop
 2b0:	d503201f 	nop
 2b4:	d503201f 	nop
 2b8:	d503201f 	nop
 2bc:	d503201f 	nop
 2c0:	d503201f 	nop
 2c4:	d503201f 	nop
 2c8:	d503201f 	nop
 2cc:	d503201f 	nop
 2d0:	d503201f 	nop
 2d4:	d503201f 	nop
 2d8:	d503201f 	nop
 2dc:	d503201f 	nop
 2e0:	d503201f 	nop
 2e4:	d503201f 	nop
 2e8:	d503201f 	nop
 2ec:	d503201f 	nop
 2f0:	d503201f 	nop
 2f4:	d503201f 	nop
 2f8:	d503201f 	nop
 2fc:	d503201f 	nop
 300:	d503201f 	nop
 304:	d503201f 	nop
 308:	d503201f 	nop
 30c:	d503201f 	nop
 310:	d503201f 	nop
 314:	d503201f 	nop
 318:	d503201f 	nop
 31c:	d503201f 	nop
 320:	d503201f 	nop
 324:	d503201f 	nop
 328:	d503201f 	nop
 32c:	d503201f 	nop
 330:	d503201f 	nop
 334:	d503201f 	nop
 338:	d503201f 	nop
 33c:	d503201f 	nop
 340:	d503201f 	nop
 344:	d503201f 	nop
 348:	d503201f 	nop
 34c:	d503201f 	nop
 350:	d503201f 	nop
 354:	d503201f 	nop
 358:	d503201f 	nop
 35c:	d503201f 	nop
 360:	d503201f 	nop
 364:	d503201f 	nop
 368:	d503201f 	nop
 36c:	d503201f 	nop
 370:	d503201f 	nop
 374:	d503201f 	nop
 378:	d503201f 	nop
 37c:	d503201f 	nop
 380:	d503201f 	nop
 384:	d503201f 	nop
 388:	d503201f 	nop
 38c:	d503201f 	nop
 390:	d503201f 	nop
 394:	d503201f 	nop
 398:	d503201f 	nop
 39c:	d503201f 	nop
 3a0:	d503201f 	nop
 3a4:	d503201f 	nop
 3a8:	d503201f 	nop
 3ac:	d503201f 	nop
 3b0:	d503201f 	nop
 3b4:	d503201f 	nop
 3b8:	d503201f 	nop
 3bc:	d503201f 	nop
 3c0:	d503201f 	nop
 3c4:	d503201f 	nop
 3c8:	d503201f 	nop
 3cc:	d503201f 	nop
 3d0:	d503201f 	nop
 3d4:	d503201f 	nop
 3d8:	d503201f 	nop
 3dc:	d503201f 	nop
 3e0:	d503201f 	nop
 3e4:	d503201f 	nop
 3e8:	d503201f 	nop
 3ec:	d503201f 	nop
 3f0:	d503201f 	nop
 3f4:	d503201f 	nop
 3f8:	d503201f 	nop
 3fc:	d503201f 	nop
 400:	d503201f 	nop
 404:	d503201f 	nop
 408:	d503201f 	nop
 40c:	d503201f 	nop
 410:	d503201f 	nop
 414:	d503201f 	nop
 418:	d503201f 	nop
 41c:	d503201f 	nop
 420:	d503201f 	nop
 424:	d503201f 	nop
 428:	d503201f 	nop
 42c:	d503201f 	nop
 430:	d503201f 	nop
 434:	d503201f 	nop
 438:	d503201f 	nop
 43c:	d503201f 	nop
 440:	d503201f 	nop
 444:	d503201f 	nop
 448:	d503201f 	nop
 44c:	d503201f 	nop
 450:	d503201f 	nop
 454:	d503201f 	nop
 458:	d503201f 	nop
 45c:	d503201f 	nop
 460:	d503201f 	nop
 464:	d503201f 	nop
 468:	d503201f 	nop
 46c:	d503201f 	nop
 470:	d503201f 	nop
 474:	d503201f 	nop
 478:	d503201f 	nop
 47c:	d503201f 	nop
 480:	d503201f 	nop
 484:	d503201f 	nop
 488:	d503201f 	nop
 48c:	d503201f 	nop
 490:	d503201f 	nop
 494:	d503201f 	nop
 498:	d503201f 	nop
 49c:	d503201f 	nop
 4a0:	d503201f 	nop
 4a4:	d503201f 	nop
 4a8:	d503201f 	nop
 4ac:	d503201f 	nop
 4b0:	d503201f 	nop
 4b4:	d503201f 	nop
 4b8:	d503201f 	nop
 4bc:	d503201f 	nop
 4c0:	d503201f 	nop
 4c4:	d503201f 	nop
 4c8:	d503201f 	nop
 4cc:	d503201f 	nop
 4d0:	d503201f 	nop
 4d4:	d503201f 	nop
 4d8:	d503201f 	nop
 4dc:	d503201f 	nop
 4e0:	d503201f 	nop
 4e4:	d503201f 	nop
 4e8:	d503201f 	nop
 4ec:	d503201f 	nop
 4f0:	d503201f 	nop
 4f4:	d503201f 	nop
 4f8:	d503201f 	nop
 4fc:	d503201f 	nop
 500:	d503201f 	nop
 504:	d503201f 	nop
 508:	d503201f 	nop
 50c:	d503201f 	nop
 510:	d503201f 	nop
 514:	d503201f 	nop
 518:	d503201f 	nop
 51c:	d503201f 	nop
 520:	d503201f 	nop
 524:	d503201f 	nop
 528:	d503201f 	nop
 52c:	d503201f 	nop
 530:	d503201f 	nop
 534:	d503201f 	nop
 538:	d503201f 	nop
 53c:	d503201f 	nop
 540:	d503201f 	nop
 544:	d503201f 	nop
 548:	d503201f 	nop
 54c:	d503201f 	nop
 550:	d503201f 	nop
 554:	d503201f 	nop
 558:	d503201f 	nop
 55c:	d503201f 	nop
 560:	d503201f 	nop
 564:	d503201f 	nop
 568:	d503201f 	nop
 56c:	d503201f 	nop
 570:	d503201f 	nop
 574:	d503201f 	nop
 578:	d503201f 	nop
 57c:	d503201f 	nop
 580:	d503201f 	nop
 584:	d503201f 	nop
 588:	d503201f 	nop
 58c:	d503201f 	nop
 590:	d503201f 	nop
 594:	d503201f 	nop
 598:	d503201f 	nop
 59c:	d503201f 	nop
 5a0:	d503201f 	nop
 5a4:	d503201f 	nop
 5a8:	d503201f 	nop
 5ac:	d503201f 	nop
 5b0:	d503201f 	nop
 5b4:	d503201f 	nop
 5b8:	d503201f 	nop
 5bc:	d503201f 	nop
 5c0:	d503201f 	nop
 5c4:	d503201f 	nop
 5c8:	d503201f 	nop
 5cc:	d503201f 	nop
 5d0:	d503201f 	nop
 5d4:	d503201f 	nop
 5d8:	d503201f 	nop
 5dc:	d503201f 	nop
 5e0:	d503201f 	nop
 5e4:	d503201f 	nop
 5e8:	d503201f 	nop
 5ec:	d503201f 	nop
 5f0:	d503201f 	nop
 5f4:	d503201f 	nop
 5f8:	d503201f 	nop
 5fc:	d503201f 	nop
 600:	d503201f 	nop
 604:	d503201f 	nop
 608:	d503201f 	nop
 60c:	d503201f 	nop
 610:	d503201f 	nop
 614:	d503201f 	nop
 618:	d503201f 	nop
 61c:	d503201f 	nop
 620:	d503201f 	nop
 624:	d503201f 	nop
 628:	d503201f 	nop
 62c:	d503201f 	nop
 630:	d503201f 	nop
 634:	d503201f 	nop
 638:	d503201f 	nop
 63c:	d503201f 	nop
 640:	d503201f 	nop
 644:	d503201f 	nop
 648:	d503201f 	nop
 64c:	d503201f 	nop
 650:	d503201f 	nop
 654:	d503201f 	nop
 658:	d503201f 	nop
 65c:	d503201f 	nop
 660:	d503201f 	nop
 664:	d503201f 	nop
 668:	d503201f 	nop
 66c:	d503201f 	nop
 670:	d503201f 	nop
 674:	d503201f 	nop
 678:	d503201f 	nop
 67c:	d503201f 	nop
 680:	d503201f 	nop
 684:	d503201f 	nop
 688:	d503201f 	nop
 68c:	d503201f 	nop
 690:	d503201f 	nop
 694:	d503201f 	nop
 698:	d503201f 	nop
 69c:	d503201f 	nop
 6a0:	d503201f 	nop
 6a4:	d503201f 	nop
 6a8:	d503201f 	nop
 6ac:	d503201f 	nop
 6b0:	d503201f 	nop
 6b4:	d503201f 	nop
 6b8:	d503201f 	nop
 6bc:	d503201f 	nop
 6c0:	d503201f 	nop
 6c4:	d503201f 	nop
 6c8:	d503201f 	nop
 6cc:	d503201f 	nop
 6d0:	d503201f 	nop
 6d4:	d503201f 	nop
 6d8:	d503201f 	nop
 6dc:	d503201f 	nop
 6e0:	d503201f 	nop
 6e4:	d503201f 	nop
 6e8:	d503201f 	nop
 6ec:	d503201f 	nop
 6f0:	d503201f 	nop
 6f4:	d503201f 	nop
 6f8:	d503201f 	nop
 6fc:	d503201f 	nop
 700:	d503201f 	nop
 704:	d503201f 	nop
 708:	d503201f 	nop
 70c:	d503201f 	nop
 710:	d503201f 	nop
 714:	d503201f 	nop
 718:	d503201f 	nop
 71c:	d503201f 	nop
 720:	d503201f 	nop
 724:	d503201f 	nop
 728:	d503201f 	nop
 72c:	d503201f 	nop
 730:	d503201f 	nop
 734:	d503201f 	nop
 738:	d503201f 	nop
 73c:	d503201f 	nop
 740:	d503201f 	nop
 744:	d503201f 	nop
 748:	d503201f 	nop
 74c:	d503201f 	nop
 750:	d503201f 	nop
 754:	d503201f 	nop
 758:	d503201f 	nop
 75c:	d503201f 	nop
 760:	d503201f 	nop
 764:	d503201f 	nop
 768:	d503201f 	nop
 76c:	d503201f 	nop
 770:	d503201f 	nop
 774:	d503201f 	nop
 778:	d503201f 	nop
 77c:	d503201f 	nop
 780:	d503201f 	nop
 784:	d503201f 	nop
 788:	d503201f 	nop
 78c:	d503201f 	nop
 790:	d503201f 	nop
 794:	d503201f 	nop
 798:	d503201f 	nop
 79c:	d503201f 	nop
 7a0:	d503201f 	nop
 7a4:	d503201f 	nop
 7a8:	d503201f 	nop
 7ac:	d503201f 	nop
 7b0:	d503201f 	nop
 7b4:	d503201f 	nop
 7b8:	d503201f 	nop
 7bc:	d503201f 	nop
 7c0:	d503201f 	nop
 7c4:	d503201f 	nop
 7c8:	d503201f 	nop
 7cc:	d503201f 	nop
 7d0:	d503201f 	nop
 7d4:	d503201f 	nop
 7d8:	d503201f 	nop
 7dc:	d503201f 	nop
 7e0:	d503201f 	nop
 7e4:	d503201f 	nop
 7e8:	d503201f 	nop
 7ec:	d503201f 	nop
 7f0:	d503201f 	nop
 7f4:	d503201f 	nop
 7f8:	d503201f 	nop
 7fc:	d503201f 	nop

0000000000000800 <vectors>:
vectors():
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:109

	.globl vectors
	.align 11
vectors:
	ventry	.
 800:	14000000 	b	800 <vectors>
 804:	d503201f 	nop
 808:	d503201f 	nop
 80c:	d503201f 	nop
 810:	d503201f 	nop
 814:	d503201f 	nop
 818:	d503201f 	nop
 81c:	d503201f 	nop
 820:	d503201f 	nop
 824:	d503201f 	nop
 828:	d503201f 	nop
 82c:	d503201f 	nop
 830:	d503201f 	nop
 834:	d503201f 	nop
 838:	d503201f 	nop
 83c:	d503201f 	nop
 840:	d503201f 	nop
 844:	d503201f 	nop
 848:	d503201f 	nop
 84c:	d503201f 	nop
 850:	d503201f 	nop
 854:	d503201f 	nop
 858:	d503201f 	nop
 85c:	d503201f 	nop
 860:	d503201f 	nop
 864:	d503201f 	nop
 868:	d503201f 	nop
 86c:	d503201f 	nop
 870:	d503201f 	nop
 874:	d503201f 	nop
 878:	d503201f 	nop
 87c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:110
	ventry	.
 880:	14000000 	b	880 <vectors+0x80>
 884:	d503201f 	nop
 888:	d503201f 	nop
 88c:	d503201f 	nop
 890:	d503201f 	nop
 894:	d503201f 	nop
 898:	d503201f 	nop
 89c:	d503201f 	nop
 8a0:	d503201f 	nop
 8a4:	d503201f 	nop
 8a8:	d503201f 	nop
 8ac:	d503201f 	nop
 8b0:	d503201f 	nop
 8b4:	d503201f 	nop
 8b8:	d503201f 	nop
 8bc:	d503201f 	nop
 8c0:	d503201f 	nop
 8c4:	d503201f 	nop
 8c8:	d503201f 	nop
 8cc:	d503201f 	nop
 8d0:	d503201f 	nop
 8d4:	d503201f 	nop
 8d8:	d503201f 	nop
 8dc:	d503201f 	nop
 8e0:	d503201f 	nop
 8e4:	d503201f 	nop
 8e8:	d503201f 	nop
 8ec:	d503201f 	nop
 8f0:	d503201f 	nop
 8f4:	d503201f 	nop
 8f8:	d503201f 	nop
 8fc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:111
	ventry	.
 900:	14000000 	b	900 <vectors+0x100>
 904:	d503201f 	nop
 908:	d503201f 	nop
 90c:	d503201f 	nop
 910:	d503201f 	nop
 914:	d503201f 	nop
 918:	d503201f 	nop
 91c:	d503201f 	nop
 920:	d503201f 	nop
 924:	d503201f 	nop
 928:	d503201f 	nop
 92c:	d503201f 	nop
 930:	d503201f 	nop
 934:	d503201f 	nop
 938:	d503201f 	nop
 93c:	d503201f 	nop
 940:	d503201f 	nop
 944:	d503201f 	nop
 948:	d503201f 	nop
 94c:	d503201f 	nop
 950:	d503201f 	nop
 954:	d503201f 	nop
 958:	d503201f 	nop
 95c:	d503201f 	nop
 960:	d503201f 	nop
 964:	d503201f 	nop
 968:	d503201f 	nop
 96c:	d503201f 	nop
 970:	d503201f 	nop
 974:	d503201f 	nop
 978:	d503201f 	nop
 97c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:112
	ventry	.
 980:	14000000 	b	980 <vectors+0x180>
 984:	d503201f 	nop
 988:	d503201f 	nop
 98c:	d503201f 	nop
 990:	d503201f 	nop
 994:	d503201f 	nop
 998:	d503201f 	nop
 99c:	d503201f 	nop
 9a0:	d503201f 	nop
 9a4:	d503201f 	nop
 9a8:	d503201f 	nop
 9ac:	d503201f 	nop
 9b0:	d503201f 	nop
 9b4:	d503201f 	nop
 9b8:	d503201f 	nop
 9bc:	d503201f 	nop
 9c0:	d503201f 	nop
 9c4:	d503201f 	nop
 9c8:	d503201f 	nop
 9cc:	d503201f 	nop
 9d0:	d503201f 	nop
 9d4:	d503201f 	nop
 9d8:	d503201f 	nop
 9dc:	d503201f 	nop
 9e0:	d503201f 	nop
 9e4:	d503201f 	nop
 9e8:	d503201f 	nop
 9ec:	d503201f 	nop
 9f0:	d503201f 	nop
 9f4:	d503201f 	nop
 9f8:	d503201f 	nop
 9fc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:114

	ventry	.
 a00:	14000000 	b	a00 <vectors+0x200>
 a04:	d503201f 	nop
 a08:	d503201f 	nop
 a0c:	d503201f 	nop
 a10:	d503201f 	nop
 a14:	d503201f 	nop
 a18:	d503201f 	nop
 a1c:	d503201f 	nop
 a20:	d503201f 	nop
 a24:	d503201f 	nop
 a28:	d503201f 	nop
 a2c:	d503201f 	nop
 a30:	d503201f 	nop
 a34:	d503201f 	nop
 a38:	d503201f 	nop
 a3c:	d503201f 	nop
 a40:	d503201f 	nop
 a44:	d503201f 	nop
 a48:	d503201f 	nop
 a4c:	d503201f 	nop
 a50:	d503201f 	nop
 a54:	d503201f 	nop
 a58:	d503201f 	nop
 a5c:	d503201f 	nop
 a60:	d503201f 	nop
 a64:	d503201f 	nop
 a68:	d503201f 	nop
 a6c:	d503201f 	nop
 a70:	d503201f 	nop
 a74:	d503201f 	nop
 a78:	d503201f 	nop
 a7c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:115
	ventry	handle_irq
 a80:	17fffd69 	b	24 <handle_irq>
 a84:	d503201f 	nop
 a88:	d503201f 	nop
 a8c:	d503201f 	nop
 a90:	d503201f 	nop
 a94:	d503201f 	nop
 a98:	d503201f 	nop
 a9c:	d503201f 	nop
 aa0:	d503201f 	nop
 aa4:	d503201f 	nop
 aa8:	d503201f 	nop
 aac:	d503201f 	nop
 ab0:	d503201f 	nop
 ab4:	d503201f 	nop
 ab8:	d503201f 	nop
 abc:	d503201f 	nop
 ac0:	d503201f 	nop
 ac4:	d503201f 	nop
 ac8:	d503201f 	nop
 acc:	d503201f 	nop
 ad0:	d503201f 	nop
 ad4:	d503201f 	nop
 ad8:	d503201f 	nop
 adc:	d503201f 	nop
 ae0:	d503201f 	nop
 ae4:	d503201f 	nop
 ae8:	d503201f 	nop
 aec:	d503201f 	nop
 af0:	d503201f 	nop
 af4:	d503201f 	nop
 af8:	d503201f 	nop
 afc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:116
	ventry	.
 b00:	14000000 	b	b00 <vectors+0x300>
 b04:	d503201f 	nop
 b08:	d503201f 	nop
 b0c:	d503201f 	nop
 b10:	d503201f 	nop
 b14:	d503201f 	nop
 b18:	d503201f 	nop
 b1c:	d503201f 	nop
 b20:	d503201f 	nop
 b24:	d503201f 	nop
 b28:	d503201f 	nop
 b2c:	d503201f 	nop
 b30:	d503201f 	nop
 b34:	d503201f 	nop
 b38:	d503201f 	nop
 b3c:	d503201f 	nop
 b40:	d503201f 	nop
 b44:	d503201f 	nop
 b48:	d503201f 	nop
 b4c:	d503201f 	nop
 b50:	d503201f 	nop
 b54:	d503201f 	nop
 b58:	d503201f 	nop
 b5c:	d503201f 	nop
 b60:	d503201f 	nop
 b64:	d503201f 	nop
 b68:	d503201f 	nop
 b6c:	d503201f 	nop
 b70:	d503201f 	nop
 b74:	d503201f 	nop
 b78:	d503201f 	nop
 b7c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:117
	ventry	.
 b80:	14000000 	b	b80 <vectors+0x380>
 b84:	d503201f 	nop
 b88:	d503201f 	nop
 b8c:	d503201f 	nop
 b90:	d503201f 	nop
 b94:	d503201f 	nop
 b98:	d503201f 	nop
 b9c:	d503201f 	nop
 ba0:	d503201f 	nop
 ba4:	d503201f 	nop
 ba8:	d503201f 	nop
 bac:	d503201f 	nop
 bb0:	d503201f 	nop
 bb4:	d503201f 	nop
 bb8:	d503201f 	nop
 bbc:	d503201f 	nop
 bc0:	d503201f 	nop
 bc4:	d503201f 	nop
 bc8:	d503201f 	nop
 bcc:	d503201f 	nop
 bd0:	d503201f 	nop
 bd4:	d503201f 	nop
 bd8:	d503201f 	nop
 bdc:	d503201f 	nop
 be0:	d503201f 	nop
 be4:	d503201f 	nop
 be8:	d503201f 	nop
 bec:	d503201f 	nop
 bf0:	d503201f 	nop
 bf4:	d503201f 	nop
 bf8:	d503201f 	nop
 bfc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:119

	ventry	.
 c00:	14000000 	b	c00 <vectors+0x400>
 c04:	d503201f 	nop
 c08:	d503201f 	nop
 c0c:	d503201f 	nop
 c10:	d503201f 	nop
 c14:	d503201f 	nop
 c18:	d503201f 	nop
 c1c:	d503201f 	nop
 c20:	d503201f 	nop
 c24:	d503201f 	nop
 c28:	d503201f 	nop
 c2c:	d503201f 	nop
 c30:	d503201f 	nop
 c34:	d503201f 	nop
 c38:	d503201f 	nop
 c3c:	d503201f 	nop
 c40:	d503201f 	nop
 c44:	d503201f 	nop
 c48:	d503201f 	nop
 c4c:	d503201f 	nop
 c50:	d503201f 	nop
 c54:	d503201f 	nop
 c58:	d503201f 	nop
 c5c:	d503201f 	nop
 c60:	d503201f 	nop
 c64:	d503201f 	nop
 c68:	d503201f 	nop
 c6c:	d503201f 	nop
 c70:	d503201f 	nop
 c74:	d503201f 	nop
 c78:	d503201f 	nop
 c7c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:120
	ventry	handle_irq
 c80:	17fffce9 	b	24 <handle_irq>
 c84:	d503201f 	nop
 c88:	d503201f 	nop
 c8c:	d503201f 	nop
 c90:	d503201f 	nop
 c94:	d503201f 	nop
 c98:	d503201f 	nop
 c9c:	d503201f 	nop
 ca0:	d503201f 	nop
 ca4:	d503201f 	nop
 ca8:	d503201f 	nop
 cac:	d503201f 	nop
 cb0:	d503201f 	nop
 cb4:	d503201f 	nop
 cb8:	d503201f 	nop
 cbc:	d503201f 	nop
 cc0:	d503201f 	nop
 cc4:	d503201f 	nop
 cc8:	d503201f 	nop
 ccc:	d503201f 	nop
 cd0:	d503201f 	nop
 cd4:	d503201f 	nop
 cd8:	d503201f 	nop
 cdc:	d503201f 	nop
 ce0:	d503201f 	nop
 ce4:	d503201f 	nop
 ce8:	d503201f 	nop
 cec:	d503201f 	nop
 cf0:	d503201f 	nop
 cf4:	d503201f 	nop
 cf8:	d503201f 	nop
 cfc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:121
	ventry	.
 d00:	14000000 	b	d00 <vectors+0x500>
 d04:	d503201f 	nop
 d08:	d503201f 	nop
 d0c:	d503201f 	nop
 d10:	d503201f 	nop
 d14:	d503201f 	nop
 d18:	d503201f 	nop
 d1c:	d503201f 	nop
 d20:	d503201f 	nop
 d24:	d503201f 	nop
 d28:	d503201f 	nop
 d2c:	d503201f 	nop
 d30:	d503201f 	nop
 d34:	d503201f 	nop
 d38:	d503201f 	nop
 d3c:	d503201f 	nop
 d40:	d503201f 	nop
 d44:	d503201f 	nop
 d48:	d503201f 	nop
 d4c:	d503201f 	nop
 d50:	d503201f 	nop
 d54:	d503201f 	nop
 d58:	d503201f 	nop
 d5c:	d503201f 	nop
 d60:	d503201f 	nop
 d64:	d503201f 	nop
 d68:	d503201f 	nop
 d6c:	d503201f 	nop
 d70:	d503201f 	nop
 d74:	d503201f 	nop
 d78:	d503201f 	nop
 d7c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:122
	ventry	.
 d80:	14000000 	b	d80 <vectors+0x580>
 d84:	d503201f 	nop
 d88:	d503201f 	nop
 d8c:	d503201f 	nop
 d90:	d503201f 	nop
 d94:	d503201f 	nop
 d98:	d503201f 	nop
 d9c:	d503201f 	nop
 da0:	d503201f 	nop
 da4:	d503201f 	nop
 da8:	d503201f 	nop
 dac:	d503201f 	nop
 db0:	d503201f 	nop
 db4:	d503201f 	nop
 db8:	d503201f 	nop
 dbc:	d503201f 	nop
 dc0:	d503201f 	nop
 dc4:	d503201f 	nop
 dc8:	d503201f 	nop
 dcc:	d503201f 	nop
 dd0:	d503201f 	nop
 dd4:	d503201f 	nop
 dd8:	d503201f 	nop
 ddc:	d503201f 	nop
 de0:	d503201f 	nop
 de4:	d503201f 	nop
 de8:	d503201f 	nop
 dec:	d503201f 	nop
 df0:	d503201f 	nop
 df4:	d503201f 	nop
 df8:	d503201f 	nop
 dfc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:124

	ventry	.
 e00:	14000000 	b	e00 <vectors+0x600>
 e04:	d503201f 	nop
 e08:	d503201f 	nop
 e0c:	d503201f 	nop
 e10:	d503201f 	nop
 e14:	d503201f 	nop
 e18:	d503201f 	nop
 e1c:	d503201f 	nop
 e20:	d503201f 	nop
 e24:	d503201f 	nop
 e28:	d503201f 	nop
 e2c:	d503201f 	nop
 e30:	d503201f 	nop
 e34:	d503201f 	nop
 e38:	d503201f 	nop
 e3c:	d503201f 	nop
 e40:	d503201f 	nop
 e44:	d503201f 	nop
 e48:	d503201f 	nop
 e4c:	d503201f 	nop
 e50:	d503201f 	nop
 e54:	d503201f 	nop
 e58:	d503201f 	nop
 e5c:	d503201f 	nop
 e60:	d503201f 	nop
 e64:	d503201f 	nop
 e68:	d503201f 	nop
 e6c:	d503201f 	nop
 e70:	d503201f 	nop
 e74:	d503201f 	nop
 e78:	d503201f 	nop
 e7c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:125
	ventry	.
 e80:	14000000 	b	e80 <vectors+0x680>
 e84:	d503201f 	nop
 e88:	d503201f 	nop
 e8c:	d503201f 	nop
 e90:	d503201f 	nop
 e94:	d503201f 	nop
 e98:	d503201f 	nop
 e9c:	d503201f 	nop
 ea0:	d503201f 	nop
 ea4:	d503201f 	nop
 ea8:	d503201f 	nop
 eac:	d503201f 	nop
 eb0:	d503201f 	nop
 eb4:	d503201f 	nop
 eb8:	d503201f 	nop
 ebc:	d503201f 	nop
 ec0:	d503201f 	nop
 ec4:	d503201f 	nop
 ec8:	d503201f 	nop
 ecc:	d503201f 	nop
 ed0:	d503201f 	nop
 ed4:	d503201f 	nop
 ed8:	d503201f 	nop
 edc:	d503201f 	nop
 ee0:	d503201f 	nop
 ee4:	d503201f 	nop
 ee8:	d503201f 	nop
 eec:	d503201f 	nop
 ef0:	d503201f 	nop
 ef4:	d503201f 	nop
 ef8:	d503201f 	nop
 efc:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:126
	ventry	.
 f00:	14000000 	b	f00 <vectors+0x700>
 f04:	d503201f 	nop
 f08:	d503201f 	nop
 f0c:	d503201f 	nop
 f10:	d503201f 	nop
 f14:	d503201f 	nop
 f18:	d503201f 	nop
 f1c:	d503201f 	nop
 f20:	d503201f 	nop
 f24:	d503201f 	nop
 f28:	d503201f 	nop
 f2c:	d503201f 	nop
 f30:	d503201f 	nop
 f34:	d503201f 	nop
 f38:	d503201f 	nop
 f3c:	d503201f 	nop
 f40:	d503201f 	nop
 f44:	d503201f 	nop
 f48:	d503201f 	nop
 f4c:	d503201f 	nop
 f50:	d503201f 	nop
 f54:	d503201f 	nop
 f58:	d503201f 	nop
 f5c:	d503201f 	nop
 f60:	d503201f 	nop
 f64:	d503201f 	nop
 f68:	d503201f 	nop
 f6c:	d503201f 	nop
 f70:	d503201f 	nop
 f74:	d503201f 	nop
 f78:	d503201f 	nop
 f7c:	d503201f 	nop
/home/clk/workspace/jailhouse/inmates/lib/arm64/header.S:127
	ventry	.
 f80:	14000000 	b	f80 <vectors+0x780>
 f84:	00000000 	.inst	0x00000000 ; undefined
 f88:	00000800 	.word	0x00000800
 f8c:	00000000 	.word	0x00000000
 f90:	00006000 	.word	0x00006000
 f94:	00000000 	.word	0x00000000

Disassembly of section .text.handle_IRQ:

0000000000003020 <handle_IRQ>:
handle_IRQ():
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:36
static void handle_IRQ(unsigned int irqn)
{
	static u64 min_delta = ~0ULL, max_delta = 0;
	u64 delta;

	if (irqn != TIMER_IRQ)
    3020:	71006c1f 	cmp	w0, #0x1b
    3024:	54000601 	b.ne	30e4 <handle_IRQ+0xc4>  // b.any
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:32
{
    3028:	a9bd53f3 	stp	x19, x20, [sp, #-48]!
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:39
		return;

	delta = timer_get_ticks() - expected_ticks;
    302c:	f0fffff4 	adrp	x20, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:32
{
    3030:	a9015bf5 	stp	x21, x22, [sp, #16]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:40
	if (delta < min_delta)
    3034:	b0000015 	adrp	x21, 4000 <uart_8250_init+0x38>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:32
{
    3038:	f90013fe 	str	x30, [sp, #32]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:39
	delta = timer_get_ticks() - expected_ticks;
    303c:	940003a8 	bl	3edc <timer_get_ticks>
    3040:	f9400281 	ldr	x1, [x20]
    3044:	cb010000 	sub	x0, x0, x1
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:40
	if (delta < min_delta)
    3048:	f94372a1 	ldr	x1, [x21, #1760]
    304c:	eb01001f 	cmp	x0, x1
    3050:	54000042 	b.cs	3058 <handle_IRQ+0x38>  // b.hs, b.nlast
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:41
		min_delta = delta;
    3054:	f90372a0 	str	x0, [x21, #1760]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:42
	if (delta > max_delta)
    3058:	91000293 	add	x19, x20, #0x0
    305c:	f9400661 	ldr	x1, [x19, #8]
    3060:	eb01001f 	cmp	x0, x1
    3064:	54000049 	b.ls	306c <handle_IRQ+0x4c>  // b.plast
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:43
		max_delta = delta;
    3068:	f9000660 	str	x0, [x19, #8]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:46

	printk("Timer fired, jitter: %6ld ns, min: %6ld ns, max: %6ld ns\n",
	       (long)timer_ticks_to_ns(delta),
    306c:	9400039e 	bl	3ee4 <timer_ticks_to_ns>
    3070:	aa0003f6 	mov	x22, x0
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:47
	       (long)timer_ticks_to_ns(min_delta),
    3074:	f94372a0 	ldr	x0, [x21, #1760]
    3078:	9400039b 	bl	3ee4 <timer_ticks_to_ns>
    307c:	aa0003f5 	mov	x21, x0
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:48
	       (long)timer_ticks_to_ns(max_delta));
    3080:	f9400660 	ldr	x0, [x19, #8]
    3084:	94000398 	bl	3ee4 <timer_ticks_to_ns>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:45
	printk("Timer fired, jitter: %6ld ns, min: %6ld ns, max: %6ld ns\n",
    3088:	aa0003e3 	mov	x3, x0
    308c:	aa1603e1 	mov	x1, x22
    3090:	aa1503e2 	mov	x2, x21
    3094:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3098:	91170000 	add	x0, x0, #0x5c0
    309c:	9400016c 	bl	364c <printk>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:50

	if (led_reg)
    30a0:	f9400a61 	ldr	x1, [x19, #16]
    30a4:	b40000e1 	cbz	x1, 30c0 <handle_IRQ+0xa0>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:51
		mmio_write32(led_reg, mmio_read32(led_reg) ^ (1 << led_pin));
    30a8:	b9401a63 	ldr	w3, [x19, #24]
    30ac:	52800020 	mov	w0, #0x1                   	// #1
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	*(volatile u16 *)address = value;
}

static inline u32 mmio_read32(void *address)
{
	return *(volatile u32 *)address;
    30b0:	b9400022 	ldr	w2, [x1]
handle_IRQ():
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:51
    30b4:	1ac32000 	lsl	w0, w0, w3
    30b8:	4a020000 	eor	w0, w0, w2
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
}

static inline void mmio_write32(void *address, u32 value)
{
	*(volatile u32 *)address = value;
    30bc:	b9000020 	str	w0, [x1]
handle_IRQ():
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:53

	expected_ticks = timer_get_ticks() + ticks_per_beat;
    30c0:	94000387 	bl	3edc <timer_get_ticks>
    30c4:	aa0003e1 	mov	x1, x0
    30c8:	f9401260 	ldr	x0, [x19, #32]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:55
	timer_start(ticks_per_beat);
}
    30cc:	a9415bf5 	ldp	x21, x22, [sp, #16]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:53
	expected_ticks = timer_get_ticks() + ticks_per_beat;
    30d0:	8b000021 	add	x1, x1, x0
    30d4:	f9000281 	str	x1, [x20]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:55
}
    30d8:	f94013fe 	ldr	x30, [sp, #32]
    30dc:	a8c353f3 	ldp	x19, x20, [sp], #48
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:54
	timer_start(ticks_per_beat);
    30e0:	1400038e 	b	3f18 <timer_start>
    30e4:	d65f03c0 	ret

Disassembly of section .text.inmate_main:

00000000000030e8 <inmate_main>:
inmate_main():
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:58

void inmate_main(void)
{
    30e8:	a9be53f3 	stp	x19, x20, [sp, #-32]!
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:59
	printk("Initializing the GIC...\n");
    30ec:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    30f0:	9117e800 	add	x0, x0, #0x5fa
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:58
{
    30f4:	f9000bfe 	str	x30, [sp, #16]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:59
	printk("Initializing the GIC...\n");
    30f8:	94000155 	bl	364c <printk>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:60
	gic_setup(handle_IRQ);
    30fc:	90000000 	adrp	x0, 3000 <page_directory>
    3100:	91008000 	add	x0, x0, #0x20
    3104:	9400031b 	bl	3d70 <gic_setup>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:64
	gic_enable_irq(TIMER_IRQ);

	printk("Initializing the timer...\n");
	ticks_per_beat = timer_get_frequency() / BEATS_PER_SEC;
    3108:	f0fffff4 	adrp	x20, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:61
	gic_enable_irq(TIMER_IRQ);
    310c:	52800360 	mov	w0, #0x1b                  	// #27
    3110:	94000329 	bl	3db4 <gic_enable_irq>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:63
	printk("Initializing the timer...\n");
    3114:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3118:	91184c00 	add	x0, x0, #0x613
    311c:	9400014c 	bl	364c <printk>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:64
	ticks_per_beat = timer_get_frequency() / BEATS_PER_SEC;
    3120:	91000293 	add	x19, x20, #0x0
    3124:	9400036c 	bl	3ed4 <timer_get_frequency>
    3128:	d2800141 	mov	x1, #0xa                   	// #10
    312c:	9ac10800 	udiv	x0, x0, x1
    3130:	f9001260 	str	x0, [x19, #32]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:65
	expected_ticks = timer_get_ticks() + ticks_per_beat;
    3134:	9400036a 	bl	3edc <timer_get_ticks>
    3138:	aa0003e1 	mov	x1, x0
    313c:	f9401260 	ldr	x0, [x19, #32]
    3140:	8b000021 	add	x1, x1, x0
    3144:	f9000281 	str	x1, [x20]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:66
	timer_start(ticks_per_beat);
    3148:	94000374 	bl	3f18 <timer_start>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:68

	led_reg = (void *)(unsigned long)cmdline_parse_int("led-reg", 0);
    314c:	d2800001 	mov	x1, #0x0                   	// #0
    3150:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3154:	9118b800 	add	x0, x0, #0x62e
    3158:	94000046 	bl	3270 <cmdline_parse_int>
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:69
	led_pin = cmdline_parse_int("led-pin", 0);
    315c:	d2800001 	mov	x1, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:68
	led_reg = (void *)(unsigned long)cmdline_parse_int("led-reg", 0);
    3160:	f9000a60 	str	x0, [x19, #16]
/home/clk/workspace/jailhouse/inmates/demos/arm64/../arm/gic-demo.c:69
	led_pin = cmdline_parse_int("led-pin", 0);
    3164:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3168:	9118d800 	add	x0, x0, #0x636
    316c:	94000041 	bl	3270 <cmdline_parse_int>
    3170:	b9001a60 	str	w0, [x19, #24]
halt():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:95
}

static inline void __attribute__((noreturn)) halt(void)
{
	while (1)
		asm volatile("wfi" : : : "memory");
    3174:	d503207f 	wfi
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:94
	while (1)
    3178:	17ffffff 	b	3174 <inmate_main+0x8c>

Disassembly of section .text.get_param:

000000000000317c <get_param>:
get_param():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:46
#define CMDLINE_BUFFER_SIZE 256
CMDLINE_BUFFER(CMDLINE_BUFFER_SIZE) __attribute__((weak));

static bool get_param(const char *param, char *value_buffer,
		      unsigned long buffer_size)
{
    317c:	a9bd53f3 	stp	x19, x20, [sp, #-48]!
    3180:	aa0103f4 	mov	x20, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:48
	unsigned long param_len = strlen(param);
	const char *p = cmdline;
    3184:	d0fffff3 	adrp	x19, 1000 <cmdline>
    3188:	91000273 	add	x19, x19, #0x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:46
{
    318c:	a9015bf5 	stp	x21, x22, [sp, #16]
    3190:	aa0203f5 	mov	x21, x2
    3194:	a9027bf7 	stp	x23, x30, [sp, #32]
    3198:	aa0003f7 	mov	x23, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:47
	unsigned long param_len = strlen(param);
    319c:	940002b3 	bl	3c68 <strlen>
    31a0:	aa0003f6 	mov	x22, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:52

	while (1) {
		/* read over leading blanks */
		while (*p == ' ')
    31a4:	39400260 	ldrb	w0, [x19]
    31a8:	7100801f 	cmp	w0, #0x20
    31ac:	54000200 	b.eq	31ec <get_param+0x70>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:55
			p++;

		if (strncmp(p, param, param_len) == 0) {
    31b0:	aa1603e2 	mov	x2, x22
    31b4:	aa1703e1 	mov	x1, x23
    31b8:	aa1303e0 	mov	x0, x19
    31bc:	940002b2 	bl	3c84 <strncmp>
    31c0:	35000380 	cbnz	w0, 3230 <get_param+0xb4>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:58
			p += param_len;

			*value_buffer = 0;
    31c4:	3900029f 	strb	wzr, [x20]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:56
			p += param_len;
    31c8:	8b160260 	add	x0, x19, x22
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:60
			/* extract parameter value */
			if (*p == '=') {
    31cc:	38766a61 	ldrb	w1, [x19, x22]
    31d0:	7100f43f 	cmp	w1, #0x3d
    31d4:	54000181 	b.ne	3204 <get_param+0x88>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:61
				p++;
    31d8:	91000400 	add	x0, x0, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:62
				while (buffer_size > 1) {
    31dc:	f10006bf 	cmp	x21, #0x1
    31e0:	540000a8 	b.hi	31f4 <get_param+0x78>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:68
					if (*p == ' ' || *p == 0)
						break;
					*value_buffer++ = *p++;
					buffer_size--;
				}
				if (buffer_size > 0)
    31e4:	b4000115 	cbz	x21, 3204 <get_param+0x88>
    31e8:	14000006 	b	3200 <get_param+0x84>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:53
			p++;
    31ec:	91000673 	add	x19, x19, #0x1
    31f0:	17ffffed 	b	31a4 <get_param+0x28>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:63
					if (*p == ' ' || *p == 0)
    31f4:	39400001 	ldrb	w1, [x0]
    31f8:	121a7822 	and	w2, w1, #0xffffffdf
    31fc:	350000e2 	cbnz	w2, 3218 <get_param+0x9c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:69
					*value_buffer = 0;
    3200:	3900029f 	strb	wzr, [x20]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:71
			}
			return true;
    3204:	52800020 	mov	w0, #0x1                   	// #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:81
			if (*p == 0)
				return false;
			p++;
		}
	}
}
    3208:	a9415bf5 	ldp	x21, x22, [sp, #16]
    320c:	a9427bf7 	ldp	x23, x30, [sp, #32]
    3210:	a8c353f3 	ldp	x19, x20, [sp], #48
    3214:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:65
					*value_buffer++ = *p++;
    3218:	91000400 	add	x0, x0, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:66
					buffer_size--;
    321c:	d10006b5 	sub	x21, x21, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:65
					*value_buffer++ = *p++;
    3220:	38001681 	strb	w1, [x20], #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:66
					buffer_size--;
    3224:	17ffffee 	b	31dc <get_param+0x60>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:76
			if (*p == 0)
    3228:	340000c0 	cbz	w0, 3240 <get_param+0xc4>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:78
			p++;
    322c:	91000673 	add	x19, x19, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:75
		while (*p != ' ') {
    3230:	39400260 	ldrb	w0, [x19]
    3234:	7100801f 	cmp	w0, #0x20
    3238:	54ffff81 	b.ne	3228 <get_param+0xac>  // b.any
    323c:	17ffffda 	b	31a4 <get_param+0x28>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:77
				return false;
    3240:	52800000 	mov	w0, #0x0                   	// #0
    3244:	17fffff1 	b	3208 <get_param+0x8c>

Disassembly of section .text.cmdline_parse_str:

0000000000003248 <cmdline_parse_str>:
cmdline_parse_str():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:86

const char *cmdline_parse_str(const char *param, char *value_buffer,
			      unsigned long buffer_size,
			      const char *default_value)
{
    3248:	a9be53f3 	stp	x19, x20, [sp, #-32]!
    324c:	aa0103f4 	mov	x20, x1
    3250:	aa0303f3 	mov	x19, x3
    3254:	f9000bfe 	str	x30, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:87
	if (get_param(param, value_buffer, buffer_size))
    3258:	97ffffc9 	bl	317c <get_param>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:90
		return value_buffer;
	else
		return default_value;
    325c:	7100001f 	cmp	w0, #0x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:91
}
    3260:	9a931280 	csel	x0, x20, x19, ne  // ne = any
    3264:	f9400bfe 	ldr	x30, [sp, #16]
    3268:	a8c253f3 	ldp	x19, x20, [sp], #32
    326c:	d65f03c0 	ret

Disassembly of section .text.cmdline_parse_int:

0000000000003270 <cmdline_parse_int>:
cmdline_parse_int():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:94

long long cmdline_parse_int(const char *param, long long default_value)
{
    3270:	a9bd7bf3 	stp	x19, x30, [sp, #-48]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:100
	char value_buffer[32];
	char *p = value_buffer;
	bool negative = false;
	long long value = 0;

	if (!get_param(param, value_buffer, sizeof(value_buffer)))
    3274:	d2800402 	mov	x2, #0x20                  	// #32
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:94
{
    3278:	aa0103f3 	mov	x19, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:100
	if (!get_param(param, value_buffer, sizeof(value_buffer)))
    327c:	910043e1 	add	x1, sp, #0x10
    3280:	97ffffbf 	bl	317c <get_param>
    3284:	35000080 	cbnz	w0, 3294 <cmdline_parse_int+0x24>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:101
		return default_value;
    3288:	aa1303e0 	mov	x0, x19
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:130
			p++;
		} while (*p != 0);
	}

	return negative ? -value : value;
}
    328c:	a8c37bf3 	ldp	x19, x30, [sp], #48
    3290:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:103
	if (strncmp(p, "0x", 2) == 0) {
    3294:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    3298:	910043e0 	add	x0, sp, #0x10
    329c:	9118f821 	add	x1, x1, #0x63e
    32a0:	d2800042 	mov	x2, #0x2                   	// #2
    32a4:	94000278 	bl	3c84 <strncmp>
    32a8:	350003c0 	cbnz	w0, 3320 <cmdline_parse_int+0xb0>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:104
		p += 2;
    32ac:	91004be1 	add	x1, sp, #0x12
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:98
	long long value = 0;
    32b0:	d2800000 	mov	x0, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:106
			if (*p >= '0' && *p <= '9')
    32b4:	39400022 	ldrb	w2, [x1]
    32b8:	5100c043 	sub	w3, w2, #0x30
    32bc:	12001c63 	and	w3, w3, #0xff
    32c0:	7100247f 	cmp	w3, #0x9
    32c4:	540000e8 	b.hi	32e0 <cmdline_parse_int+0x70>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:107
				value = (value << 4) + *p - '0';
    32c8:	92401c42 	and	x2, x2, #0xff
    32cc:	8b001040 	add	x0, x2, x0, lsl #4
    32d0:	d100c000 	sub	x0, x0, #0x30
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:115
		} while (*p != 0);
    32d4:	38401c22 	ldrb	w2, [x1, #1]!
    32d8:	35fffee2 	cbnz	w2, 32b4 <cmdline_parse_int+0x44>
    32dc:	17ffffec 	b	328c <cmdline_parse_int+0x1c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:108
			else if (*p >= 'A' && *p <= 'F')
    32e0:	51010443 	sub	w3, w2, #0x41
    32e4:	12001c63 	and	w3, w3, #0xff
    32e8:	7100147f 	cmp	w3, #0x5
    32ec:	540000a8 	b.hi	3300 <cmdline_parse_int+0x90>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:109
				value = (value << 4) + *p - 'A' + 10;
    32f0:	92401c42 	and	x2, x2, #0xff
    32f4:	8b001040 	add	x0, x2, x0, lsl #4
    32f8:	d100dc00 	sub	x0, x0, #0x37
    32fc:	17fffff6 	b	32d4 <cmdline_parse_int+0x64>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:110
			else if (*p >= 'a' && *p <= 'f')
    3300:	51018443 	sub	w3, w2, #0x61
    3304:	12001c63 	and	w3, w3, #0xff
    3308:	7100147f 	cmp	w3, #0x5
    330c:	54fffbe8 	b.hi	3288 <cmdline_parse_int+0x18>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:111
				value = (value << 4) + *p - 'a' + 10;
    3310:	92401c42 	and	x2, x2, #0xff
    3314:	8b001040 	add	x0, x2, x0, lsl #4
    3318:	d1015c00 	sub	x0, x0, #0x57
    331c:	17ffffee 	b	32d4 <cmdline_parse_int+0x64>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:117
		if (*p == '-' || *p == '+')
    3320:	394043e0 	ldrb	w0, [sp, #16]
    3324:	7100b41f 	cmp	w0, #0x2d
    3328:	1a9f17e3 	cset	w3, eq  // eq = none
    332c:	7100ac1f 	cmp	w0, #0x2b
    3330:	7a401860 	ccmp	w3, #0x0, #0x0, ne  // ne = any
    3334:	54000240 	b.eq	337c <cmdline_parse_int+0x10c>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:118
			negative = (*p++ == '-');
    3338:	910047e2 	add	x2, sp, #0x11
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:96
	char *p = value_buffer;
    333c:	d2800000 	mov	x0, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:122
				value = (value * 10) + *p - '0';
    3340:	d2800145 	mov	x5, #0xa                   	// #10
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:121
			if (*p >= '0' && *p <= '9')
    3344:	39400041 	ldrb	w1, [x2]
    3348:	5100c024 	sub	w4, w1, #0x30
    334c:	12001c84 	and	w4, w4, #0xff
    3350:	7100249f 	cmp	w4, #0x9
    3354:	54fff9a8 	b.hi	3288 <cmdline_parse_int+0x18>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:122
				value = (value * 10) + *p - '0';
    3358:	92401c21 	and	x1, x1, #0xff
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:126
		} while (*p != 0);
    335c:	38401c44 	ldrb	w4, [x2, #1]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:122
				value = (value * 10) + *p - '0';
    3360:	9b050401 	madd	x1, x0, x5, x1
    3364:	d100c020 	sub	x0, x1, #0x30
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:126
		} while (*p != 0);
    3368:	35fffee4 	cbnz	w4, 3344 <cmdline_parse_int+0xd4>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:129
	return negative ? -value : value;
    336c:	34fff903 	cbz	w3, 328c <cmdline_parse_int+0x1c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:129 (discriminator 1)
    3370:	d2800600 	mov	x0, #0x30                  	// #48
    3374:	cb010000 	sub	x0, x0, x1
    3378:	17ffffc5 	b	328c <cmdline_parse_int+0x1c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:96
	char *p = value_buffer;
    337c:	910043e2 	add	x2, sp, #0x10
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:97
	bool negative = false;
    3380:	52800003 	mov	w3, #0x0                   	// #0
    3384:	17ffffee 	b	333c <cmdline_parse_int+0xcc>

Disassembly of section .text.cmdline_parse_bool:

0000000000003388 <cmdline_parse_bool>:
cmdline_parse_bool():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:133

bool cmdline_parse_bool(const char *param, bool default_value)
{
    3388:	a9be7bf3 	stp	x19, x30, [sp, #-32]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:137
	char value_buffer[8];

	/* return the default value if the parameter is not provided */
	if (!get_param(param, value_buffer, sizeof(value_buffer)))
    338c:	d2800102 	mov	x2, #0x8                   	// #8
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:133
{
    3390:	2a0103f3 	mov	w19, w1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:137
	if (!get_param(param, value_buffer, sizeof(value_buffer)))
    3394:	910063e1 	add	x1, sp, #0x18
    3398:	97ffff79 	bl	317c <get_param>
    339c:	35000080 	cbnz	w0, 33ac <cmdline_parse_bool+0x24>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:138
		return default_value;
    33a0:	2a1303e0 	mov	w0, w19
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:149
	if (!strncasecmp(value_buffer, "false", 5) ||
	    !strncmp(value_buffer, "0", 1))
		return false;

	return default_value;
}
    33a4:	a8c27bf3 	ldp	x19, x30, [sp], #32
    33a8:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:140
	if (!strncasecmp(value_buffer, "true", 4) ||
    33ac:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    33b0:	910063e0 	add	x0, sp, #0x18
    33b4:	91190421 	add	x1, x1, #0x641
    33b8:	d2800082 	mov	x2, #0x4                   	// #4
    33bc:	94000241 	bl	3cc0 <strncasecmp>
    33c0:	35000060 	cbnz	w0, 33cc <cmdline_parse_bool+0x44>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:142
		return true;
    33c4:	52800020 	mov	w0, #0x1                   	// #1
    33c8:	17fffff7 	b	33a4 <cmdline_parse_bool+0x1c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:141 (discriminator 1)
	    !strncmp(value_buffer, "1", 1) || strlen(value_buffer) == 0)
    33cc:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    33d0:	910063e0 	add	x0, sp, #0x18
    33d4:	911ab821 	add	x1, x1, #0x6ae
    33d8:	d2800022 	mov	x2, #0x1                   	// #1
    33dc:	9400022a 	bl	3c84 <strncmp>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:140 (discriminator 1)
	if (!strncasecmp(value_buffer, "true", 4) ||
    33e0:	34ffff20 	cbz	w0, 33c4 <cmdline_parse_bool+0x3c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:141
	    !strncmp(value_buffer, "1", 1) || strlen(value_buffer) == 0)
    33e4:	910063e0 	add	x0, sp, #0x18
    33e8:	94000220 	bl	3c68 <strlen>
    33ec:	b4fffec0 	cbz	x0, 33c4 <cmdline_parse_bool+0x3c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:144
	if (!strncasecmp(value_buffer, "false", 5) ||
    33f0:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    33f4:	910063e0 	add	x0, sp, #0x18
    33f8:	91191821 	add	x1, x1, #0x646
    33fc:	d28000a2 	mov	x2, #0x5                   	// #5
    3400:	94000230 	bl	3cc0 <strncasecmp>
    3404:	340000e0 	cbz	w0, 3420 <cmdline_parse_bool+0x98>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:145 (discriminator 1)
	    !strncmp(value_buffer, "0", 1))
    3408:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    340c:	910063e0 	add	x0, sp, #0x18
    3410:	911ae821 	add	x1, x1, #0x6ba
    3414:	d2800022 	mov	x2, #0x1                   	// #1
    3418:	9400021b 	bl	3c84 <strncmp>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:144 (discriminator 1)
	if (!strncasecmp(value_buffer, "false", 5) ||
    341c:	35fffc20 	cbnz	w0, 33a0 <cmdline_parse_bool+0x18>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../cmdline.c:146
		return false;
    3420:	52800000 	mov	w0, #0x0                   	// #0
    3424:	17ffffe0 	b	33a4 <cmdline_parse_bool+0x1c>

Disassembly of section .text.div_u64_u64:

0000000000003428 <div_u64_u64>:
div_u64_u64():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:22
{
	unsigned long long result = 0;
	unsigned long long tmp_res, tmp_div;

	while (dividend >= divisor) {
		tmp_div = divisor << 1;
    3428:	d37ff825 	lsl	x5, x1, #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:17
{
    342c:	aa0003e2 	mov	x2, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:18
	unsigned long long result = 0;
    3430:	d2800000 	mov	x0, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:21
	while (dividend >= divisor) {
    3434:	eb01005f 	cmp	x2, x1
    3438:	54000142 	b.cs	3460 <div_u64_u64+0x38>  // b.hs, b.nlast
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:34
		}
		dividend -= divisor * tmp_res;
		result += tmp_res;
	}
	return result;
}
    343c:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:25
			tmp_res <<= 1;
    3440:	d37ff863 	lsl	x3, x3, #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:26
			if (tmp_div & (1ULL << 63))
    3444:	b7f80084 	tbnz	x4, #63, 3454 <div_u64_u64+0x2c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:28
			tmp_div <<= 1;
    3448:	d37ff884 	lsl	x4, x4, #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:24
		while (dividend >= tmp_div) {
    344c:	eb04005f 	cmp	x2, x4
    3450:	54ffff82 	b.cs	3440 <div_u64_u64+0x18>  // b.hs, b.nlast
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:30
		dividend -= divisor * tmp_res;
    3454:	9b018862 	msub	x2, x3, x1, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:31
		result += tmp_res;
    3458:	8b030000 	add	x0, x0, x3
    345c:	17fffff6 	b	3434 <div_u64_u64+0xc>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:22
		tmp_div = divisor << 1;
    3460:	aa0503e4 	mov	x4, x5
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:23
		tmp_res = 1;
    3464:	d2800023 	mov	x3, #0x1                   	// #1
    3468:	17fffff9 	b	344c <div_u64_u64+0x24>

Disassembly of section .text.uint2str:

000000000000346c <uint2str>:
uint2str():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:48

#endif /* BITS_PER_LONG >= 64 */

static char *uint2str(unsigned long long value, char *buf)
{
	unsigned long long digit, divisor = 10000000000000000000ULL;
    346c:	d2b13d07 	mov	x7, #0x89e80000            	// #2313682944
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:47
{
    3470:	aa0003e9 	mov	x9, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:48
	unsigned long long digit, divisor = 10000000000000000000ULL;
    3474:	f2c46087 	movk	x7, #0x2304, lsl #32
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:47
{
    3478:	aa0103e6 	mov	x6, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:49
	int first_digit = 1;
    347c:	52800028 	mov	w8, #0x1                   	// #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:48
	unsigned long long digit, divisor = 10000000000000000000ULL;
    3480:	f2f158e7 	movk	x7, #0x8ac7, lsl #48
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:47
{
    3484:	f81f0ffe 	str	x30, [sp, #-16]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:52

	while (divisor > 0) {
		digit = div_u64_u64(value, divisor);
    3488:	aa0703e1 	mov	x1, x7
    348c:	aa0903e0 	mov	x0, x9
    3490:	97ffffe6 	bl	3428 <div_u64_u64>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:54
		value -= digit * divisor;
		if (!first_digit || digit > 0 || divisor == 1) {
    3494:	f100001f 	cmp	x0, #0x0
    3498:	52000108 	eor	w8, w8, #0x1
    349c:	1a9f07e1 	cset	w1, ne  // ne = any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:53
		value -= digit * divisor;
    34a0:	9b07a409 	msub	x9, x0, x7, x9
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:54
		if (!first_digit || digit > 0 || divisor == 1) {
    34a4:	2a080028 	orr	w8, w1, w8
    34a8:	35000068 	cbnz	w8, 34b4 <uint2str+0x48>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:54 (discriminator 1)
    34ac:	f10004ff 	cmp	x7, #0x1
    34b0:	54000181 	b.ne	34e0 <uint2str+0x74>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:55
			*buf++ = '0' + digit;
    34b4:	1100c001 	add	w1, w0, #0x30
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:56
			first_digit = 0;
    34b8:	52800008 	mov	w8, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:55
			*buf++ = '0' + digit;
    34bc:	380014c1 	strb	w1, [x6], #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:58
		}
		divisor = div_u64_u64(divisor, 10);
    34c0:	aa0703e0 	mov	x0, x7
    34c4:	d2800141 	mov	x1, #0xa                   	// #10
    34c8:	97ffffd8 	bl	3428 <div_u64_u64>
    34cc:	aa0003e7 	mov	x7, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:51
	while (divisor > 0) {
    34d0:	b5fffdc0 	cbnz	x0, 3488 <uint2str+0x1c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:62
	}

	return buf;
}
    34d4:	aa0603e0 	mov	x0, x6
    34d8:	f84107fe 	ldr	x30, [sp], #16
    34dc:	d65f03c0 	ret
    34e0:	52800028 	mov	w8, #0x1                   	// #1
    34e4:	17fffff7 	b	34c0 <uint2str+0x54>

Disassembly of section .text.hex2str:

00000000000034e8 <hex2str>:
hex2str():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:85
	while (divisor > 0) {
		digit = div_u64_u64(value, divisor);
		value -= digit * divisor;
		if (!first_digit || digit > 0 || divisor == 1 ||
		    divisor & leading_zero_mask) {
			*buf++ = hexdigit[digit];
    34e8:	b000000a 	adrp	x10, 4000 <uart_8250_init+0x38>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:75
{
    34ec:	aa0003e7 	mov	x7, x0
    34f0:	aa0203e9 	mov	x9, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:85
			*buf++ = hexdigit[digit];
    34f4:	9115a94a 	add	x10, x10, #0x56a
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:75
{
    34f8:	aa0103e6 	mov	x6, x1
    34fc:	52800208 	mov	w8, #0x10                  	// #16
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:78
	int first_digit = 1;
    3500:	5280002b 	mov	w11, #0x1                   	// #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:77
	unsigned long long digit, divisor = 0x1000000000000000ULL;
    3504:	d2e20001 	mov	x1, #0x1000000000000000    	// #1152921504606846976
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:75
{
    3508:	f81f0ffe 	str	x30, [sp, #-16]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:81
		digit = div_u64_u64(value, divisor);
    350c:	aa0703e0 	mov	x0, x7
    3510:	97ffffc6 	bl	3428 <div_u64_u64>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:82
		value -= digit * divisor;
    3514:	9b019c07 	msub	x7, x0, x1, x7
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:83
		if (!first_digit || digit > 0 || divisor == 1 ||
    3518:	340000cb 	cbz	w11, 3530 <hex2str+0x48>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:83 (discriminator 1)
    351c:	f100043f 	cmp	x1, #0x1
    3520:	54000080 	b.eq	3530 <hex2str+0x48>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:84 (discriminator 2)
		    divisor & leading_zero_mask) {
    3524:	8a010122 	and	x2, x9, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:83 (discriminator 2)
		if (!first_digit || digit > 0 || divisor == 1 ||
    3528:	aa000042 	orr	x2, x2, x0
    352c:	b4000082 	cbz	x2, 353c <hex2str+0x54>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:85
			*buf++ = hexdigit[digit];
    3530:	38606940 	ldrb	w0, [x10, x0]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:86
			first_digit = 0;
    3534:	5280000b 	mov	w11, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:85
			*buf++ = hexdigit[digit];
    3538:	380014c0 	strb	w0, [x6], #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:88
		}
		divisor >>= 4;
    353c:	d344fc21 	lsr	x1, x1, #4
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:80
	while (divisor > 0) {
    3540:	71000508 	subs	w8, w8, #0x1
    3544:	54fffe41 	b.ne	350c <hex2str+0x24>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:92
	}

	return buf;
}
    3548:	aa0603e0 	mov	x0, x6
    354c:	f84107fe 	ldr	x30, [sp], #16
    3550:	d65f03c0 	ret

Disassembly of section .text.align:

0000000000003554 <align>:
align():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:95

static char *align(char *p1, char *p0, unsigned long width, char fill)
{
    3554:	a9be53f3 	stp	x19, x20, [sp, #-32]!
    3558:	aa0203f4 	mov	x20, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:98
	unsigned int n;

	if (p1 - p0 >= width)
    355c:	cb010002 	sub	x2, x0, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:95
{
    3560:	f9000bfe 	str	x30, [sp, #16]
    3564:	aa0103f3 	mov	x19, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:98
	if (p1 - p0 >= width)
    3568:	eb14005f 	cmp	x2, x20
    356c:	54000162 	b.cs	3598 <align+0x44>  // b.hs, b.nlast
    3570:	12001c61 	and	w1, w3, #0xff
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:101
		return p1;

	for (n = 1; p1 - n >= p0; n++)
    3574:	52800024 	mov	w4, #0x1                   	// #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:101 (discriminator 1)
    3578:	cb244006 	sub	x6, x0, w4, uxtw
    357c:	2a0403e5 	mov	w5, w4
    3580:	eb1300df 	cmp	x6, x19
    3584:	54000102 	b.cs	35a4 <align+0x50>  // b.hs, b.nlast
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:103
		*(p0 + width - n) = *(p1 - n);
	memset(p0, fill, width - (p1 - p0));
    3588:	cb020282 	sub	x2, x20, x2
    358c:	aa1303e0 	mov	x0, x19
    3590:	940001a1 	bl	3c14 <memset>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:104
	return p0 + width;
    3594:	8b140260 	add	x0, x19, x20
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:105
}
    3598:	f9400bfe 	ldr	x30, [sp, #16]
    359c:	a8c253f3 	ldp	x19, x20, [sp], #32
    35a0:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:102 (discriminator 3)
		*(p0 + width - n) = *(p1 - n);
    35a4:	cb050285 	sub	x5, x20, x5
    35a8:	394000c3 	ldrb	w3, [x6]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:101 (discriminator 3)
	for (n = 1; p1 - n >= p0; n++)
    35ac:	11000484 	add	w4, w4, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:102 (discriminator 3)
		*(p0 + width - n) = *(p1 - n);
    35b0:	38256a63 	strb	w3, [x19, x5]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:101 (discriminator 3)
	for (n = 1; p1 - n >= p0; n++)
    35b4:	17fffff1 	b	3578 <align+0x24>

Disassembly of section .text.console_write:

00000000000035b8 <console_write>:
console_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:51

static struct uart_chip *chip;
static bool virtual_console;

static void console_write(const char *msg)
{
    35b8:	a9be53f3 	stp	x19, x20, [sp, #-32]!
    35bc:	aa0003f4 	mov	x20, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:54
	char c = 0;

	if (!chip && !virtual_console)
    35c0:	f0ffffe0 	adrp	x0, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:51
{
    35c4:	a9017bf5 	stp	x21, x30, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:54
	if (!chip && !virtual_console)
    35c8:	9100a015 	add	x21, x0, #0x28
    35cc:	f9401400 	ldr	x0, [x0, #40]
    35d0:	b5000060 	cbnz	x0, 35dc <console_write+0x24>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:54 (discriminator 1)
    35d4:	b9400aa0 	ldr	w0, [x21, #8]
    35d8:	34000340 	cbz	w0, 3640 <console_write+0x88>
    35dc:	52800013 	mov	w19, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:58
		return;

	while (1) {
		if (c == '\n')
    35e0:	71002a7f 	cmp	w19, #0xa
    35e4:	54000280 	b.eq	3634 <console_write+0x7c>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:61
			c = '\r';
		else
			c = *msg++;
    35e8:	39400293 	ldrb	w19, [x20]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:62
		if (!c)
    35ec:	340002b3 	cbz	w19, 3640 <console_write+0x88>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:61
			c = *msg++;
    35f0:	91000694 	add	x20, x20, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:65
			break;

		if (chip) {
    35f4:	f94002a0 	ldr	x0, [x21]
    35f8:	b4000120 	cbz	x0, 361c <console_write+0x64>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:66
			while (chip->is_busy(chip))
    35fc:	f94002a0 	ldr	x0, [x21]
    3600:	f9401c01 	ldr	x1, [x0, #56]
    3604:	d63f0020 	blr	x1
    3608:	350001a0 	cbnz	w0, 363c <console_write+0x84>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:68
				cpu_relax();
			chip->write(chip, c);
    360c:	f94002a0 	ldr	x0, [x21]
    3610:	2a1303e1 	mov	w1, w19
    3614:	f9402002 	ldr	x2, [x0, #64]
    3618:	d63f0040 	blr	x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:71
		}

		if (virtual_console)
    361c:	b9400aa0 	ldr	w0, [x21, #8]
    3620:	34fffe00 	cbz	w0, 35e0 <console_write+0x28>
jailhouse_call_arg1():
/home/clk/workspace/jailhouse/inmates/../include/arch/arm64/asm/jailhouse_hypercall.h:77
}

static inline __u64 jailhouse_call_arg1(__u64 num, __u64 arg1)
{
	register __u64 num_result asm(JAILHOUSE_CALL_NUM_RESULT) = num;
	register __u64 __arg1 asm(JAILHOUSE_CALL_ARG1) = arg1;
    3624:	92401e61 	and	x1, x19, #0xff
/home/clk/workspace/jailhouse/inmates/../include/arch/arm64/asm/jailhouse_hypercall.h:76
	register __u64 num_result asm(JAILHOUSE_CALL_NUM_RESULT) = num;
    3628:	d2800100 	mov	x0, #0x8                   	// #8
/home/clk/workspace/jailhouse/inmates/../include/arch/arm64/asm/jailhouse_hypercall.h:79

	asm volatile(
    362c:	d4094902 	hvc	#0x4a48
/home/clk/workspace/jailhouse/inmates/../include/arch/arm64/asm/jailhouse_hypercall.h:84
		JAILHOUSE_CALL_INS
		: "=r" (num_result)
		: "r" (num_result), "r" (__arg1)
		: "memory");
	return num_result;
    3630:	17ffffec 	b	35e0 <console_write+0x28>
console_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:59
			c = '\r';
    3634:	528001b3 	mov	w19, #0xd                   	// #13
    3638:	17ffffef 	b	35f4 <console_write+0x3c>
cpu_relax():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/asm/processor.h:47
#define dsb(domain)	asm volatile("dsb " #domain ::: "memory")

static inline void cpu_relax(void)
{
	asm volatile("" : : : "memory");
}
    363c:	17fffff0 	b	35fc <console_write+0x44>
console_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:74
			jailhouse_call_arg1(JAILHOUSE_HC_DEBUG_CONSOLE_PUTC, c);
	}
}
    3640:	a9417bf5 	ldp	x21, x30, [sp, #16]
    3644:	a8c253f3 	ldp	x19, x20, [sp], #32
    3648:	d65f03c0 	ret

Disassembly of section .text.printk:

000000000000364c <printk>:
printk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:125
}

#include "../../../hypervisor/printk-core.c"

void printk(const char *fmt, ...)
{
    364c:	a9a553f3 	stp	x19, x20, [sp, #-432]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:129
	static bool inited = false;
	va_list ap;

	if (!inited) {
    3650:	f0fffff3 	adrp	x19, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:125
{
    3654:	a9015bf5 	stp	x21, x22, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:129
	if (!inited) {
    3658:	9100a275 	add	x21, x19, #0x28
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:125
{
    365c:	a90263f7 	stp	x23, x24, [sp, #32]
    3660:	a9036bf9 	stp	x25, x26, [sp, #48]
    3664:	aa0003f9 	mov	x25, x0
    3668:	f90023fe 	str	x30, [sp, #64]
    366c:	3d803fe0 	str	q0, [sp, #240]
    3670:	3d8043e1 	str	q1, [sp, #256]
    3674:	3d8047e2 	str	q2, [sp, #272]
    3678:	3d804be3 	str	q3, [sp, #288]
    367c:	3d804fe4 	str	q4, [sp, #304]
    3680:	3d8053e5 	str	q5, [sp, #320]
    3684:	3d8057e6 	str	q6, [sp, #336]
    3688:	3d805be7 	str	q7, [sp, #352]
    368c:	a9178be1 	stp	x1, x2, [sp, #376]
    3690:	a91893e3 	stp	x3, x4, [sp, #392]
    3694:	a9199be5 	stp	x5, x6, [sp, #408]
    3698:	f900d7e7 	str	x7, [sp, #424]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:129
	if (!inited) {
    369c:	b9400ea0 	ldr	w0, [x21, #12]
    36a0:	35000520 	cbnz	w0, 3744 <printk+0xf8>
console_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:84
	if (JAILHOUSE_COMM_HAS_DBG_PUTC_PERMITTED(comm_region->flags))
    36a4:	d2b00000 	mov	x0, #0x80000000            	// #2147483648
    36a8:	b9401401 	ldr	w1, [x0, #20]
    36ac:	360000e1 	tbz	w1, #0, 36c8 <printk+0x7c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:86
			JAILHOUSE_COMM_HAS_DBG_PUTC_ACTIVE(comm_region->flags));
    36b0:	b9401401 	ldr	w1, [x0, #20]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:85
		virtual_console = cmdline_parse_bool("con-virtual",
    36b4:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    36b8:	91193000 	add	x0, x0, #0x64c
    36bc:	d3410421 	ubfx	x1, x1, #1, #1
    36c0:	97ffff32 	bl	3388 <cmdline_parse_bool>
    36c4:	b9000aa0 	str	w0, [x21, #8]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:88
	type = cmdline_parse_str("con-type", buf, sizeof(buf), "");
    36c8:	9101c3e1 	add	x1, sp, #0x70
    36cc:	b0000003 	adrp	x3, 4000 <uart_8250_init+0x38>
    36d0:	d2800402 	mov	x2, #0x20                  	// #32
    36d4:	91184863 	add	x3, x3, #0x612
    36d8:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:89
	for (c = uart_array; *c; c++)
    36dc:	b0000014 	adrp	x20, 4000 <uart_8250_init+0x38>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:88
	type = cmdline_parse_str("con-type", buf, sizeof(buf), "");
    36e0:	91196000 	add	x0, x0, #0x658
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:91
		    (!*type && console->type == (*c)->type)) {
    36e4:	d2800317 	mov	x23, #0x18                  	// #24
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:88
	type = cmdline_parse_str("con-type", buf, sizeof(buf), "");
    36e8:	97fffed8 	bl	3248 <cmdline_parse_str>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:89
	for (c = uart_array; *c; c++)
    36ec:	911bc294 	add	x20, x20, #0x6f0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:88
	type = cmdline_parse_str("con-type", buf, sizeof(buf), "");
    36f0:	aa0003f6 	mov	x22, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:91
		    (!*type && console->type == (*c)->type)) {
    36f4:	f2b00017 	movk	x23, #0x8000, lsl #16
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:89
	for (c = uart_array; *c; c++)
    36f8:	f9400280 	ldr	x0, [x20]
    36fc:	b40001c0 	cbz	x0, 3734 <printk+0xe8>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:90
		if (!strcmp(type, (*c)->name) ||
    3700:	f9400001 	ldr	x1, [x0]
    3704:	aa1603e0 	mov	x0, x22
    3708:	9400016c 	bl	3cb8 <strcmp>
    370c:	34000100 	cbz	w0, 372c <printk+0xe0>
    3710:	394002c0 	ldrb	w0, [x22]
    3714:	35000820 	cbnz	w0, 3818 <printk+0x1cc>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:91
		    (!*type && console->type == (*c)->type)) {
    3718:	f9400280 	ldr	x0, [x20]
    371c:	79401ae1 	ldrh	w1, [x23, #12]
    3720:	79401000 	ldrh	w0, [x0, #8]
    3724:	6b00003f 	cmp	w1, w0
    3728:	54000781 	b.ne	3818 <printk+0x1cc>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:92
			chip = *c;
    372c:	f9400280 	ldr	x0, [x20]
    3730:	f9001660 	str	x0, [x19, #40]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:96
	if (!chip)
    3734:	f9401660 	ldr	x0, [x19, #40]
    3738:	b5000740 	cbnz	x0, 3820 <printk+0x1d4>
printk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:131
		console_init();
		inited = true;
    373c:	52800020 	mov	w0, #0x1                   	// #1
    3740:	b9000ea0 	str	w0, [x21, #12]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:134
	}

	va_start(ap, fmt);
    3744:	9106c3e0 	add	x0, sp, #0x1b0
    3748:	a90503e0 	stp	x0, x0, [sp, #80]
    374c:	9105c3e0 	add	x0, sp, #0x170
    3750:	f90033e0 	str	x0, [sp, #96]
    3754:	12800fe0 	mov	w0, #0xffffff80            	// #-128
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
					length = SZ_LONGLONG;
					c = *fmt++;
				}
			}

			switch (c) {
    3758:	b0000015 	adrp	x21, 4000 <uart_8250_init+0x38>
    375c:	b0000016 	adrp	x22, 4000 <uart_8250_init+0x38>
    3760:	9106c3f3 	add	x19, sp, #0x1b0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:194
				break;
			}
		} else {
			*p++ = c;
		}
		if (p >= &buf[sizeof(buf) - 1]) {
    3764:	9103bff7 	add	x23, sp, #0xef
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    3768:	911562b5 	add	x21, x21, #0x558
    376c:	911512d6 	add	x22, x22, #0x544
printk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:134
    3770:	b9006fe0 	str	w0, [sp, #108]
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:116
	p = buf;
    3774:	9101c3e0 	add	x0, sp, #0x70
printk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:134
    3778:	128006f4 	mov	w20, #0xffffffc8            	// #-56
    377c:	b9006bf4 	str	w20, [sp, #104]
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:119
		c = *fmt++;
    3780:	aa1903f8 	mov	x24, x25
    3784:	38401701 	ldrb	w1, [x24], #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:120
		if (c == 0) {
    3788:	34001b21 	cbz	w1, 3aec <printk+0x4a0>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:122
		} else if (c == '%') {
    378c:	7100943f 	cmp	w1, #0x25
    3790:	54001aa1 	b.ne	3ae4 <printk+0x498>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:123
			*p = 0;
    3794:	3900001f 	strb	wzr, [x0]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:124
			console_write(buf);
    3798:	9101c3e0 	add	x0, sp, #0x70
    379c:	97ffff87 	bl	35b8 <console_write>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:127
			c = *fmt++;
    37a0:	39400721 	ldrb	w1, [x25, #1]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:131
			fill = (c == '0') ? '0' : ' ';
    37a4:	5280060d 	mov	w13, #0x30                  	// #48
    37a8:	52800400 	mov	w0, #0x20                  	// #32
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:127
			c = *fmt++;
    37ac:	91000b22 	add	x2, x25, #0x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:131
			fill = (c == '0') ? '0' : ' ';
    37b0:	7100c03f 	cmp	w1, #0x30
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:129
			width = 0;
    37b4:	5280000c 	mov	w12, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:131
			fill = (c == '0') ? '0' : ' ';
    37b8:	1a8001ad 	csel	w13, w13, w0, eq  // eq = none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:133
				width = width * 10 + c - '0';
    37bc:	52800143 	mov	w3, #0xa                   	// #10
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:132
			while (c >= '0' && c <= '9') {
    37c0:	5100c020 	sub	w0, w1, #0x30
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:134
				c = *fmt++;
    37c4:	91000458 	add	x24, x2, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:132
			while (c >= '0' && c <= '9') {
    37c8:	12001c00 	and	w0, w0, #0xff
    37cc:	7100241f 	cmp	w0, #0x9
    37d0:	54000649 	b.ls	3898 <printk+0x24c>  // b.plast
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:140
			if (c == 'l') {
    37d4:	7101b03f 	cmp	w1, #0x6c
    37d8:	54001de1 	b.ne	3b94 <printk+0x548>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:142
				c = *fmt++;
    37dc:	39400041 	ldrb	w1, [x2]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:143
				if (c == 'l') {
    37e0:	7101b03f 	cmp	w1, #0x6c
    37e4:	54001961 	b.ne	3b10 <printk+0x4c4>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:145
					c = *fmt++;
    37e8:	39400441 	ldrb	w1, [x2, #1]
    37ec:	91000858 	add	x24, x2, #0x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    37f0:	7101903f 	cmp	w1, #0x64
    37f4:	540019e0 	b.eq	3b30 <printk+0x4e4>  // b.none
    37f8:	540005e8 	b.hi	38b4 <printk+0x268>  // b.pmore
    37fc:	71018c3f 	cmp	w1, #0x63
    3800:	54000740 	b.eq	38e8 <printk+0x29c>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:187
				*p++ = '%';
    3804:	528004a0 	mov	w0, #0x25                  	// #37
    3808:	3901c3e0 	strb	w0, [sp, #112]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:188
				*p++ = c;
    380c:	9101cbe0 	add	x0, sp, #0x72
    3810:	3901c7e1 	strb	w1, [sp, #113]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:189
				break;
    3814:	1400003e 	b	390c <printk+0x2c0>
console_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:89
	for (c = uart_array; *c; c++)
    3818:	91002294 	add	x20, x20, #0x8
    381c:	17ffffb7 	b	36f8 <printk+0xac>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:100
		cmdline_parse_int("con-base", console->address);
    3820:	d2800314 	mov	x20, #0x18                  	// #24
    3824:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3828:	f2b00014 	movk	x20, #0x8000, lsl #16
    382c:	91198400 	add	x0, x0, #0x661
    3830:	f9400281 	ldr	x1, [x20]
    3834:	97fffe8f 	bl	3270 <cmdline_parse_int>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:99
	chip->base = (void *)(unsigned long)
    3838:	f9401661 	ldr	x1, [x19, #40]
    383c:	f9000820 	str	x0, [x1, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:101
	chip->divider = cmdline_parse_int("con-divider", console->divider);
    3840:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3844:	9119a800 	add	x0, x0, #0x66a
    3848:	b9401281 	ldr	w1, [x20, #16]
    384c:	97fffe89 	bl	3270 <cmdline_parse_int>
    3850:	aa0003e1 	mov	x1, x0
    3854:	f9401660 	ldr	x0, [x19, #40]
    3858:	b9001801 	str	w1, [x0, #24]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:105
	arch_console_init(chip);
    385c:	94000165 	bl	3df0 <arch_console_init>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:107
	chip->init(chip);
    3860:	f9401660 	ldr	x0, [x19, #40]
    3864:	f9401801 	ldr	x1, [x0, #48]
    3868:	d63f0020 	blr	x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:109
	if (chip->divider == 0) {
    386c:	f9401660 	ldr	x0, [x19, #40]
    3870:	b9401800 	ldr	w0, [x0, #24]
    3874:	35fff640 	cbnz	w0, 373c <printk+0xf0>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:89
	for (c = uart_array; *c; c++)
    3878:	52800c93 	mov	w19, #0x64                  	// #100
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:116
				if (chip->is_busy(chip))
    387c:	f94002a0 	ldr	x0, [x21]
    3880:	f9401c01 	ldr	x1, [x0, #56]
    3884:	d63f0020 	blr	x1
    3888:	35ffff80 	cbnz	w0, 3878 <printk+0x22c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:115
			for (n = 0; n < UART_IDLE_LOOPS; n++)
    388c:	71000673 	subs	w19, w19, #0x1
    3890:	54ffff61 	b.ne	387c <printk+0x230>  // b.any
    3894:	17ffffaa 	b	373c <printk+0xf0>
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:133
				width = width * 10 + c - '0';
    3898:	1b03058c 	madd	w12, w12, w3, w1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:136
					width = 0;
    389c:	aa1803e2 	mov	x2, x24
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:134
				c = *fmt++;
    38a0:	385ff301 	ldurb	w1, [x24, #-1]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:133
				width = width * 10 + c - '0';
    38a4:	5100c18c 	sub	w12, w12, #0x30
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:136
					width = 0;
    38a8:	7101fd9f 	cmp	w12, #0x7f
    38ac:	1a9f318c 	csel	w12, w12, wzr, cc  // cc = lo, ul, last
    38b0:	17ffffc4 	b	37c0 <printk+0x174>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    38b4:	5101c020 	sub	w0, w1, #0x70
    38b8:	12001c02 	and	w2, w0, #0xff
    38bc:	7100205f 	cmp	w2, #0x8
    38c0:	54fffa28 	b.hi	3804 <printk+0x1b8>  // b.pmore
    38c4:	7100201f 	cmp	w0, #0x8
    38c8:	54fff9e8 	b.hi	3804 <printk+0x1b8>  // b.pmore
    38cc:	b0000002 	adrp	x2, 4000 <uart_8250_init+0x38>
    38d0:	9114c042 	add	x2, x2, #0x530
    38d4:	78605840 	ldrh	w0, [x2, w0, uxtw #1]
    38d8:	10000062 	adr	x2, 38e4 <printk+0x298>
    38dc:	8b20a840 	add	x0, x2, w0, sxth #2
    38e0:	d61f0000 	br	x0
    38e4:	aa0203f8 	mov	x24, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:151
				*p++ = (unsigned char)va_arg(ap, int);
    38e8:	37f80174 	tbnz	w20, #31, 3914 <printk+0x2c8>
    38ec:	91002e60 	add	x0, x19, #0xb
    38f0:	2a1403e1 	mov	w1, w20
    38f4:	927df000 	and	x0, x0, #0xfffffffffffffff8
    38f8:	b9400262 	ldr	w2, [x19]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:152
				break;
    38fc:	2a0103f4 	mov	w20, w1
    3900:	aa0003f3 	mov	x19, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:151
				*p++ = (unsigned char)va_arg(ap, int);
    3904:	9101c7e0 	add	x0, sp, #0x71
    3908:	3901c3e2 	strb	w2, [sp, #112]
    390c:	aa1803f9 	mov	x25, x24
    3910:	17ffff9c 	b	3780 <printk+0x134>
    3914:	11002281 	add	w1, w20, #0x8
    3918:	7100003f 	cmp	w1, #0x0
    391c:	5400008d 	b.le	392c <printk+0x2e0>
    3920:	91002e60 	add	x0, x19, #0xb
    3924:	927df000 	and	x0, x0, #0xfffffffffffffff8
    3928:	17fffff4 	b	38f8 <printk+0x2ac>
    392c:	9106c3e2 	add	x2, sp, #0x1b0
    3930:	aa1303e0 	mov	x0, x19
    3934:	8b34c053 	add	x19, x2, w20, sxtw
    3938:	17fffff0 	b	38f8 <printk+0x2ac>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:157
					v = va_arg(ap, long);
    393c:	11002282 	add	w2, w20, #0x8
    3940:	7100005f 	cmp	w2, #0x0
    3944:	5400008d 	b.le	3954 <printk+0x308>
    3948:	91003e61 	add	x1, x19, #0xf
    394c:	927df021 	and	x1, x1, #0xfffffffffffffff8
    3950:	1400007c 	b	3b40 <printk+0x4f4>
    3954:	9106c3e0 	add	x0, sp, #0x1b0
    3958:	aa1303e1 	mov	x1, x19
    395c:	8b34c013 	add	x19, x0, w20, sxtw
    3960:	14000078 	b	3b40 <printk+0x4f4>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:159
					v = va_arg(ap, int);
    3964:	37f80134 	tbnz	w20, #31, 3988 <printk+0x33c>
    3968:	91002e61 	add	x1, x19, #0xb
    396c:	2a1403e3 	mov	w3, w20
    3970:	927df021 	and	x1, x1, #0xfffffffffffffff8
    3974:	b9800260 	ldrsw	x0, [x19]
    3978:	2a0303f4 	mov	w20, w3
    397c:	aa0203f8 	mov	x24, x2
    3980:	aa0103f3 	mov	x19, x1
    3984:	14000072 	b	3b4c <printk+0x500>
    3988:	11002283 	add	w3, w20, #0x8
    398c:	7100007f 	cmp	w3, #0x0
    3990:	5400008d 	b.le	39a0 <printk+0x354>
    3994:	91002e61 	add	x1, x19, #0xb
    3998:	927df021 	and	x1, x1, #0xfffffffffffffff8
    399c:	17fffff6 	b	3974 <printk+0x328>
    39a0:	9106c3e0 	add	x0, sp, #0x1b0
    39a4:	aa1303e1 	mov	x1, x19
    39a8:	8b34c013 	add	x19, x0, w20, sxtw
    39ac:	17fffff2 	b	3974 <printk+0x328>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    39b0:	aa0203f8 	mov	x24, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:164
				*p++ = '0';
    39b4:	528f0600 	mov	w0, #0x7830                	// #30768
    39b8:	7900e3e0 	strh	w0, [sp, #112]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:166
				v = va_arg(ap, unsigned long);
    39bc:	37f80214 	tbnz	w20, #31, 39fc <printk+0x3b0>
    39c0:	91003e6c 	add	x12, x19, #0xf
    39c4:	2a1403ed 	mov	w13, w20
    39c8:	927df18c 	and	x12, x12, #0xfffffffffffffff8
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:167
				p = hex2str(v, p, (unsigned long)-1);
    39cc:	f9400260 	ldr	x0, [x19]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:168
				break;
    39d0:	2a0d03f4 	mov	w20, w13
    39d4:	aa0c03f3 	mov	x19, x12
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:167
				p = hex2str(v, p, (unsigned long)-1);
    39d8:	9101cbe1 	add	x1, sp, #0x72
    39dc:	92800002 	mov	x2, #0xffffffffffffffff    	// #-1
    39e0:	97fffec2 	bl	34e8 <hex2str>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:194
		if (p >= &buf[sizeof(buf) - 1]) {
    39e4:	eb17001f 	cmp	x0, x23
    39e8:	54fff923 	b.cc	390c <printk+0x2c0>  // b.lo, b.ul, b.last
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:195
			*p = 0;
    39ec:	3900001f 	strb	wzr, [x0]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:196
			console_write(buf);
    39f0:	9101c3e0 	add	x0, sp, #0x70
    39f4:	97fffef1 	bl	35b8 <console_write>
    39f8:	14000014 	b	3a48 <printk+0x3fc>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:166
				v = va_arg(ap, unsigned long);
    39fc:	1100228d 	add	w13, w20, #0x8
    3a00:	710001bf 	cmp	w13, #0x0
    3a04:	5400008d 	b.le	3a14 <printk+0x3c8>
    3a08:	91003e6c 	add	x12, x19, #0xf
    3a0c:	927df18c 	and	x12, x12, #0xfffffffffffffff8
    3a10:	17ffffef 	b	39cc <printk+0x380>
    3a14:	9106c3e0 	add	x0, sp, #0x1b0
    3a18:	aa1303ec 	mov	x12, x19
    3a1c:	8b34c013 	add	x19, x0, w20, sxtw
    3a20:	17ffffeb 	b	39cc <printk+0x380>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    3a24:	aa0203f8 	mov	x24, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:170
				console_write(va_arg(ap, const char *));
    3a28:	37f80154 	tbnz	w20, #31, 3a50 <printk+0x404>
    3a2c:	91003e79 	add	x25, x19, #0xf
    3a30:	2a1403fa 	mov	w26, w20
    3a34:	927df339 	and	x25, x25, #0xfffffffffffffff8
    3a38:	f9400260 	ldr	x0, [x19]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:171
				break;
    3a3c:	2a1a03f4 	mov	w20, w26
    3a40:	aa1903f3 	mov	x19, x25
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:170
				console_write(va_arg(ap, const char *));
    3a44:	97fffedd 	bl	35b8 <console_write>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:197
			p = buf;
    3a48:	9101c3e0 	add	x0, sp, #0x70
    3a4c:	17ffffb0 	b	390c <printk+0x2c0>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:170
				console_write(va_arg(ap, const char *));
    3a50:	1100229a 	add	w26, w20, #0x8
    3a54:	7100035f 	cmp	w26, #0x0
    3a58:	5400008d 	b.le	3a68 <printk+0x41c>
    3a5c:	91003e79 	add	x25, x19, #0xf
    3a60:	927df339 	and	x25, x25, #0xfffffffffffffff8
    3a64:	17fffff5 	b	3a38 <printk+0x3ec>
    3a68:	9106c3e0 	add	x0, sp, #0x1b0
    3a6c:	aa1303f9 	mov	x25, x19
    3a70:	8b34c013 	add	x19, x0, w20, sxtw
    3a74:	17fffff1 	b	3a38 <printk+0x3ec>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:177
					v = va_arg(ap, unsigned long);
    3a78:	91003e62 	add	x2, x19, #0xf
    3a7c:	2a1403e3 	mov	w3, w20
    3a80:	927df042 	and	x2, x2, #0xfffffffffffffff8
    3a84:	f9400260 	ldr	x0, [x19]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:179
					v = va_arg(ap, unsigned int);
    3a88:	2a0303f4 	mov	w20, w3
    3a8c:	aa0203f3 	mov	x19, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:180
				if (c == 'u')
    3a90:	7101d43f 	cmp	w1, #0x75
    3a94:	54000201 	b.ne	3ad4 <printk+0x488>  // b.any
int2str():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:180
    3a98:	9101c3e1 	add	x1, sp, #0x70
    3a9c:	14000031 	b	3b60 <printk+0x514>
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:177
					v = va_arg(ap, unsigned long);
    3aa0:	9106c3e0 	add	x0, sp, #0x1b0
    3aa4:	aa1303e2 	mov	x2, x19
    3aa8:	8b34c013 	add	x19, x0, w20, sxtw
    3aac:	17fffff6 	b	3a84 <printk+0x438>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:179
					v = va_arg(ap, unsigned int);
    3ab0:	91002e62 	add	x2, x19, #0xb
    3ab4:	2a1403e3 	mov	w3, w20
    3ab8:	927df042 	and	x2, x2, #0xfffffffffffffff8
    3abc:	b9400260 	ldr	w0, [x19]
    3ac0:	17fffff2 	b	3a88 <printk+0x43c>
    3ac4:	9106c3e0 	add	x0, sp, #0x1b0
    3ac8:	aa1303e2 	mov	x2, x19
    3acc:	8b34c013 	add	x19, x0, w20, sxtw
    3ad0:	17fffffb 	b	3abc <printk+0x470>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:183
					p = hex2str(v, p, 0);
    3ad4:	9101c3e1 	add	x1, sp, #0x70
    3ad8:	d2800002 	mov	x2, #0x0                   	// #0
    3adc:	97fffe83 	bl	34e8 <hex2str>
    3ae0:	14000021 	b	3b64 <printk+0x518>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:192
			*p++ = c;
    3ae4:	38001401 	strb	w1, [x0], #1
    3ae8:	17ffffbf 	b	39e4 <printk+0x398>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:201
		}
	}

	*p = 0;
    3aec:	3900001f 	strb	wzr, [x0]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:202
	console_write(buf);
    3af0:	9101c3e0 	add	x0, sp, #0x70
    3af4:	97fffeb1 	bl	35b8 <console_write>
printk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../printk.c:139

	__vprintk(fmt, ap);

	va_end(ap);
}
    3af8:	a9415bf5 	ldp	x21, x22, [sp, #16]
    3afc:	a94263f7 	ldp	x23, x24, [sp, #32]
    3b00:	a9436bf9 	ldp	x25, x26, [sp, #48]
    3b04:	f94023fe 	ldr	x30, [sp, #64]
    3b08:	a8db53f3 	ldp	x19, x20, [sp], #432
    3b0c:	d65f03c0 	ret
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    3b10:	7101e03f 	cmp	w1, #0x78
    3b14:	54ffe788 	b.hi	3804 <printk+0x1b8>  // b.pmore
    3b18:	7101bc3f 	cmp	w1, #0x6f
    3b1c:	540002e8 	b.hi	3b78 <printk+0x52c>  // b.pmore
    3b20:	71018c3f 	cmp	w1, #0x63
    3b24:	54ffee20 	b.eq	38e8 <printk+0x29c>  // b.none
    3b28:	7101903f 	cmp	w1, #0x64
    3b2c:	54ffe6c1 	b.ne	3804 <printk+0x1b8>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:157
					v = va_arg(ap, long);
    3b30:	37fff074 	tbnz	w20, #31, 393c <printk+0x2f0>
    3b34:	91003e61 	add	x1, x19, #0xf
    3b38:	2a1403e2 	mov	w2, w20
    3b3c:	927df021 	and	x1, x1, #0xfffffffffffffff8
    3b40:	2a0203f4 	mov	w20, w2
    3b44:	f9400260 	ldr	x0, [x19]
    3b48:	aa0103f3 	mov	x19, x1
int2str():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:66
	if (value < 0) {
    3b4c:	b6fffa60 	tbz	x0, #63, 3a98 <printk+0x44c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:67
		*buf++ = '-';
    3b50:	528005a1 	mov	w1, #0x2d                  	// #45
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:68
		value = -value;
    3b54:	cb0003e0 	neg	x0, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:67
		*buf++ = '-';
    3b58:	3901c3e1 	strb	w1, [sp, #112]
    3b5c:	9101c7e1 	add	x1, sp, #0x71
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:70
	return uint2str(value, buf);
    3b60:	97fffe43 	bl	346c <uint2str>
__vprintk():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:184
				p = align(p, p0, width, fill);
    3b64:	2a0d03e3 	mov	w3, w13
    3b68:	2a0c03e2 	mov	w2, w12
    3b6c:	9101c3e1 	add	x1, sp, #0x70
    3b70:	97fffe79 	bl	3554 <align>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:185
				break;
    3b74:	17ffff9c 	b	39e4 <printk+0x398>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    3b78:	5101c020 	sub	w0, w1, #0x70
    3b7c:	7100201f 	cmp	w0, #0x8
    3b80:	54ffe428 	b.hi	3804 <printk+0x1b8>  // b.pmore
    3b84:	78605ac0 	ldrh	w0, [x22, w0, uxtw #1]
    3b88:	10000062 	adr	x2, 3b94 <printk+0x548>
    3b8c:	8b20a840 	add	x0, x2, w0, sxth #2
    3b90:	d61f0000 	br	x0
    3b94:	7101e03f 	cmp	w1, #0x78
    3b98:	540000e8 	b.hi	3bb4 <printk+0x568>  // b.pmore
    3b9c:	7101bc3f 	cmp	w1, #0x6f
    3ba0:	540000e8 	b.hi	3bbc <printk+0x570>  // b.pmore
    3ba4:	71018c3f 	cmp	w1, #0x63
    3ba8:	54ffe9e0 	b.eq	38e4 <printk+0x298>  // b.none
    3bac:	7101903f 	cmp	w1, #0x64
    3bb0:	54ffeda0 	b.eq	3964 <printk+0x318>  // b.none
    3bb4:	aa0203f8 	mov	x24, x2
    3bb8:	17ffff13 	b	3804 <printk+0x1b8>
    3bbc:	5101c020 	sub	w0, w1, #0x70
    3bc0:	7100201f 	cmp	w0, #0x8
    3bc4:	54ffff88 	b.hi	3bb4 <printk+0x568>  // b.pmore
    3bc8:	78605aa0 	ldrh	w0, [x21, w0, uxtw #1]
    3bcc:	10000063 	adr	x3, 3bd8 <printk+0x58c>
    3bd0:	8b20a860 	add	x0, x3, w0, sxth #2
    3bd4:	d61f0000 	br	x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:177
					v = va_arg(ap, unsigned long);
    3bd8:	36fff514 	tbz	w20, #31, 3a78 <printk+0x42c>
    3bdc:	11002283 	add	w3, w20, #0x8
    3be0:	7100007f 	cmp	w3, #0x0
    3be4:	54fff5ed 	b.le	3aa0 <printk+0x454>
    3be8:	91003e62 	add	x2, x19, #0xf
    3bec:	927df042 	and	x2, x2, #0xfffffffffffffff8
    3bf0:	17ffffa5 	b	3a84 <printk+0x438>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:149
			switch (c) {
    3bf4:	aa0203f8 	mov	x24, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../../../hypervisor/printk-core.c:179
					v = va_arg(ap, unsigned int);
    3bf8:	36fff5d4 	tbz	w20, #31, 3ab0 <printk+0x464>
    3bfc:	11002283 	add	w3, w20, #0x8
    3c00:	7100007f 	cmp	w3, #0x0
    3c04:	54fff60d 	b.le	3ac4 <printk+0x478>
    3c08:	91002e62 	add	x2, x19, #0xb
    3c0c:	927df042 	and	x2, x2, #0xfffffffffffffff8
    3c10:	17ffffab 	b	3abc <printk+0x470>

Disassembly of section .text.memset:

0000000000003c14 <memset>:
memset():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:60

void *memset(void *s, int c, unsigned long n)
{
	u8 *p = s;

	while (n-- > 0)
    3c14:	d2800003 	mov	x3, #0x0                   	// #0
    3c18:	eb03005f 	cmp	x2, x3
    3c1c:	54000041 	b.ne	3c24 <memset+0x10>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:63
		*p++ = c;
	return s;
}
    3c20:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:61
		*p++ = c;
    3c24:	38236801 	strb	w1, [x0, x3]
    3c28:	91000463 	add	x3, x3, #0x1
    3c2c:	17fffffb 	b	3c18 <memset+0x4>

Disassembly of section .text.memcmp:

0000000000003c30 <memcmp>:
memcmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:70
int memcmp(const void *s1, const void *s2, unsigned long n)
{
	const unsigned char *_s1 = s1, *_s2 = s2;

	while (n-- > 0)
		if (*_s1++ != *_s2++)
    3c30:	d1000421 	sub	x1, x1, #0x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:69
	while (n-- > 0)
    3c34:	d2800003 	mov	x3, #0x0                   	// #0
    3c38:	eb03005f 	cmp	x2, x3
    3c3c:	54000061 	b.ne	3c48 <memcmp+0x18>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:72
			return _s1[-1] < _s2[-1] ? -1 : 1;
	return 0;
    3c40:	52800000 	mov	w0, #0x0                   	// #0
    3c44:	14000008 	b	3c64 <memcmp+0x34>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:70
		if (*_s1++ != *_s2++)
    3c48:	38636805 	ldrb	w5, [x0, x3]
    3c4c:	91000463 	add	x3, x3, #0x1
    3c50:	38636824 	ldrb	w4, [x1, x3]
    3c54:	6b0400bf 	cmp	w5, w4
    3c58:	54ffff00 	b.eq	3c38 <memcmp+0x8>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:71
			return _s1[-1] < _s2[-1] ? -1 : 1;
    3c5c:	52800020 	mov	w0, #0x1                   	// #1
    3c60:	5a9f2000 	csinv	w0, w0, wzr, cs  // cs = hs, nlast
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:73
}
    3c64:	d65f03c0 	ret

Disassembly of section .text.strlen:

0000000000003c68 <strlen>:
strlen():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:76

unsigned long strlen(const char *s1)
{
    3c68:	aa0003e1 	mov	x1, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:77
	unsigned long len = 0;
    3c6c:	d2800000 	mov	x0, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:79

	while (*s1++)
    3c70:	38606822 	ldrb	w2, [x1, x0]
    3c74:	35000042 	cbnz	w2, 3c7c <strlen+0x14>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:83
		len++;

	return len;
}
    3c78:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:80
		len++;
    3c7c:	91000400 	add	x0, x0, #0x1
    3c80:	17fffffc 	b	3c70 <strlen+0x8>

Disassembly of section .text.strncmp:

0000000000003c84 <strncmp>:
strncmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:86

int strncmp(const char *s1, const char *s2, unsigned long n)
{
    3c84:	aa0003e5 	mov	x5, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:89
	int diff;

	while (n-- > 0) {
    3c88:	d2800004 	mov	x4, #0x0                   	// #0
    3c8c:	eb04005f 	cmp	x2, x4
    3c90:	54000061 	b.ne	3c9c <strncmp+0x18>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:98
		if (*s1 == 0)
			break;
		s1++;
		s2++;
	}
	return 0;
    3c94:	52800000 	mov	w0, #0x0                   	// #0
    3c98:	14000007 	b	3cb4 <strncmp+0x30>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:90
		diff = *s1 - *s2;
    3c9c:	386468a6 	ldrb	w6, [x5, x4]
    3ca0:	38646823 	ldrb	w3, [x1, x4]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:91
		if (diff)
    3ca4:	6b0300c0 	subs	w0, w6, w3
    3ca8:	54000061 	b.ne	3cb4 <strncmp+0x30>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:93
		if (*s1 == 0)
    3cac:	91000484 	add	x4, x4, #0x1
    3cb0:	35fffee6 	cbnz	w6, 3c8c <strncmp+0x8>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:99
}
    3cb4:	d65f03c0 	ret

Disassembly of section .text.strcmp:

0000000000003cb8 <strcmp>:
strcmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:103

int strcmp(const char *s1, const char *s2)
{
	return strncmp(s1, s2, -1);
    3cb8:	92800002 	mov	x2, #0xffffffffffffffff    	// #-1
    3cbc:	17fffff2 	b	3c84 <strncmp>

Disassembly of section .text.strncasecmp:

0000000000003cc0 <strncasecmp>:
strncasecmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:107
}

int strncasecmp(const char *s1, const char *s2, unsigned long n)
{
    3cc0:	aa0003e7 	mov	x7, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:110
	int diff;

	while (n-- > 0) {
    3cc4:	d2800004 	mov	x4, #0x0                   	// #0
    3cc8:	eb02009f 	cmp	x4, x2
    3ccc:	54000061 	b.ne	3cd8 <strncasecmp+0x18>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:120
			break;
		s1++;
		s2++;
	}

	return 0;
    3cd0:	52800000 	mov	w0, #0x0                   	// #0
    3cd4:	14000010 	b	3d14 <strncasecmp+0x54>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:111
		diff = tolower(*s1) - tolower(*s2);
    3cd8:	386468e6 	ldrb	w6, [x7, x4]
tolower():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:43
   return (c >= 'A' && c <= 'Z') ? c + ('a' - 'A') : c;
    3cdc:	510104c0 	sub	w0, w6, #0x41
strncasecmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:111
		diff = tolower(*s1) - tolower(*s2);
    3ce0:	2a0603e3 	mov	w3, w6
tolower():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:43
   return (c >= 'A' && c <= 'Z') ? c + ('a' - 'A') : c;
    3ce4:	7100641f 	cmp	w0, #0x19
    3ce8:	54000048 	b.hi	3cf0 <strncasecmp+0x30>  // b.pmore
    3cec:	110080c3 	add	w3, w6, #0x20
strncasecmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:111
		diff = tolower(*s1) - tolower(*s2);
    3cf0:	38646825 	ldrb	w5, [x1, x4]
tolower():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:43
   return (c >= 'A' && c <= 'Z') ? c + ('a' - 'A') : c;
    3cf4:	510104a0 	sub	w0, w5, #0x41
    3cf8:	7100641f 	cmp	w0, #0x19
    3cfc:	54000048 	b.hi	3d04 <strncasecmp+0x44>  // b.pmore
    3d00:	110080a5 	add	w5, w5, #0x20
strncasecmp():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:112
		if (diff)
    3d04:	6b050060 	subs	w0, w3, w5
    3d08:	54000061 	b.ne	3d14 <strncasecmp+0x54>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:114
		if (*s1 == 0)
    3d0c:	91000484 	add	x4, x4, #0x1
    3d10:	35fffdc6 	cbnz	w6, 3cc8 <strncasecmp+0x8>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../string.c:121
}
    3d14:	d65f03c0 	ret

Disassembly of section .text.vector_irq:

0000000000003d18 <vector_irq>:
vector_irq():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:50
static irq_handler_t irq_handler = (irq_handler_t)NULL;
static const struct gic *gic = &gic_v2;

/* Replaces the weak reference in header.S */
void vector_irq(void)
{
    3d18:	a9be53f3 	stp	x19, x20, [sp, #-32]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:54
	u32 irqn;

	while (1) {
		irqn = gic->read_ack();
    3d1c:	b0000014 	adrp	x20, 4000 <uart_8250_init+0x38>
    3d20:	911ba294 	add	x20, x20, #0x6e8
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:50
{
    3d24:	a9017bf5 	stp	x21, x30, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:58
		if (irqn == 0x3ff)
			break;

		if (irq_handler)
    3d28:	f0fffff5 	adrp	x21, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:54
		irqn = gic->read_ack();
    3d2c:	f9400280 	ldr	x0, [x20]
    3d30:	f9400c00 	ldr	x0, [x0, #24]
    3d34:	d63f0000 	blr	x0
    3d38:	2a0003f3 	mov	w19, w0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:55
		if (irqn == 0x3ff)
    3d3c:	710ffc1f 	cmp	w0, #0x3ff
    3d40:	54000120 	b.eq	3d64 <vector_irq+0x4c>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:58
		if (irq_handler)
    3d44:	f9401ea1 	ldr	x1, [x21, #56]
    3d48:	b4000041 	cbz	x1, 3d50 <vector_irq+0x38>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:59
			irq_handler(irqn);
    3d4c:	d63f0020 	blr	x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:61

		gic->write_eoi(irqn);
    3d50:	f9400280 	ldr	x0, [x20]
    3d54:	f9400801 	ldr	x1, [x0, #16]
    3d58:	2a1303e0 	mov	w0, w19
    3d5c:	d63f0020 	blr	x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:54
		irqn = gic->read_ack();
    3d60:	17fffff3 	b	3d2c <vector_irq+0x14>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:63
	}
}
    3d64:	a9417bf5 	ldp	x21, x30, [sp, #16]
    3d68:	a8c253f3 	ldp	x19, x20, [sp], #32
    3d6c:	d65f03c0 	ret

Disassembly of section .text.gic_setup:

0000000000003d70 <gic_setup>:
gic_setup():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:66

void gic_setup(irq_handler_t handler)
{
    3d70:	a9bf7bf3 	stp	x19, x30, [sp, #-16]!
    3d74:	aa0003f3 	mov	x19, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:67
	if (comm_region->gic_version == 3)
    3d78:	d2b00000 	mov	x0, #0x80000000            	// #2147483648
    3d7c:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    3d80:	3940e000 	ldrb	w0, [x0, #56]
    3d84:	71000c1f 	cmp	w0, #0x3
    3d88:	54000081 	b.ne	3d98 <gic_setup+0x28>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:68
		gic = &gic_v3;
    3d8c:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3d90:	91168000 	add	x0, x0, #0x5a0
    3d94:	f9037420 	str	x0, [x1, #1768]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:70

	gic->init();
    3d98:	f9437420 	ldr	x0, [x1, #1768]
    3d9c:	f9400000 	ldr	x0, [x0]
    3da0:	d63f0000 	blr	x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:72

	irq_handler = handler;
    3da4:	f0ffffe0 	adrp	x0, 2000 <expected_ticks>
    3da8:	f9001c13 	str	x19, [x0, #56]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:75

	gic_setup_irq_stack();
}
    3dac:	a8c17bf3 	ldp	x19, x30, [sp], #16
    3db0:	d65f03c0 	ret

Disassembly of section .text.gic_enable_irq:

0000000000003db4 <gic_enable_irq>:
gic_enable_irq():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic.c:79

void gic_enable_irq(unsigned int irq)
{
	gic->enable(irq);
    3db4:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    3db8:	f9437421 	ldr	x1, [x1, #1768]
    3dbc:	f9400421 	ldr	x1, [x1, #8]
    3dc0:	aa0103f0 	mov	x16, x1
    3dc4:	d61f0200 	br	x16

Disassembly of section .text.reg_out_mmio8:

0000000000003dc8 <reg_out_mmio8>:
reg_out_mmio8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:44
#include <inmate.h>
#include <uart.h>

static void reg_out_mmio8(struct uart_chip *chip, unsigned int reg, u32 value)
{
	mmio_write8(chip->base + reg, value);
    3dc8:	f9400800 	ldr	x0, [x0, #16]
    3dcc:	2a0103e1 	mov	w1, w1
    3dd0:	12001c42 	and	w2, w2, #0xff
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
	*(volatile u8 *)address = value;
    3dd4:	38216802 	strb	w2, [x0, x1]
reg_out_mmio8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:45
}
    3dd8:	d65f03c0 	ret

Disassembly of section .text.reg_in_mmio8:

0000000000003ddc <reg_in_mmio8>:
reg_in_mmio8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:49

static u32 reg_in_mmio8(struct uart_chip *chip, unsigned int reg)
{
	return mmio_read8(chip->base + reg);
    3ddc:	f9400800 	ldr	x0, [x0, #16]
    3de0:	2a0103e1 	mov	w1, w1
mmio_read8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:59
	return *(volatile u8 *)address;
    3de4:	38616800 	ldrb	w0, [x0, x1]
reg_in_mmio8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:50
}
    3de8:	12001c00 	and	w0, w0, #0xff
    3dec:	d65f03c0 	ret

Disassembly of section .text.arch_console_init:

0000000000003df0 <arch_console_init>:
arch_console_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:53

void arch_console_init(struct uart_chip *chip)
{
    3df0:	a9bd53f3 	stp	x19, x20, [sp, #-48]!
    3df4:	aa0003f4 	mov	x20, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:60
	unsigned int gate_nr;
	bool gate_inverted;
	void *clock_reg;
	u32 clock_gates;

	gate_nr = cmdline_parse_int("con-gate-nr", console->gate_nr);
    3df8:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:53
{
    3dfc:	a9015bf5 	stp	x21, x22, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:60
	gate_nr = cmdline_parse_int("con-gate-nr", console->gate_nr);
    3e00:	d2800315 	mov	x21, #0x18                  	// #24
    3e04:	f2b00015 	movk	x21, #0x8000, lsl #16
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:53
{
    3e08:	a9027bf7 	stp	x23, x30, [sp, #32]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:60
	gate_nr = cmdline_parse_int("con-gate-nr", console->gate_nr);
    3e0c:	9119d800 	add	x0, x0, #0x676
    3e10:	b94016a1 	ldr	w1, [x21, #20]
    3e14:	97fffd17 	bl	3270 <cmdline_parse_int>
    3e18:	aa0003f7 	mov	x23, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:61
	gate_inverted = cmdline_parse_bool("con-gate-inverted",
    3e1c:	79401ea1 	ldrh	w1, [x21, #14]
    3e20:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3e24:	911a0800 	add	x0, x0, #0x682
    3e28:	d34c3021 	ubfx	x1, x1, #12, #1
    3e2c:	97fffd57 	bl	3388 <cmdline_parse_bool>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:64
					CON_HAS_INVERTED_GATE(console->flags));
	clock_reg = (void *)(unsigned long)
		cmdline_parse_int("con-clock-reg", console->clock_reg);
    3e30:	f9400ea1 	ldr	x1, [x21, #24]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:61
	gate_inverted = cmdline_parse_bool("con-gate-inverted",
    3e34:	2a0003f6 	mov	w22, w0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:64
		cmdline_parse_int("con-clock-reg", console->clock_reg);
    3e38:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3e3c:	911a5000 	add	x0, x0, #0x694
    3e40:	97fffd0c 	bl	3270 <cmdline_parse_int>
    3e44:	aa0003f3 	mov	x19, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:66

	if (cmdline_parse_bool("con-regdist-1",
    3e48:	79401ea1 	ldrh	w1, [x21, #14]
    3e4c:	b0000000 	adrp	x0, 4000 <uart_8250_init+0x38>
    3e50:	911a8800 	add	x0, x0, #0x6a2
    3e54:	d27f0021 	eor	x1, x1, #0x2
    3e58:	53010421 	ubfx	w1, w1, #1, #1
    3e5c:	97fffd4b 	bl	3388 <cmdline_parse_bool>
    3e60:	340000e0 	cbz	w0, 3e7c <arch_console_init+0x8c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:68
			       CON_USES_REGDIST_1(console->flags))) {
		chip->reg_out = reg_out_mmio8;
    3e64:	90000000 	adrp	x0, 3000 <page_directory>
    3e68:	91372000 	add	x0, x0, #0xdc8
    3e6c:	f9001280 	str	x0, [x20, #32]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:69
		chip->reg_in = reg_in_mmio8;
    3e70:	90000000 	adrp	x0, 3000 <page_directory>
    3e74:	91377000 	add	x0, x0, #0xddc
    3e78:	f9001680 	str	x0, [x20, #40]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:72
	}

	if (chip->base)
    3e7c:	f9400a80 	ldr	x0, [x20, #16]
    3e80:	b4000080 	cbz	x0, 3e90 <arch_console_init+0xa0>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:73
		map_range(chip->base, PAGE_SIZE, MAP_UNCACHED);
    3e84:	52800022 	mov	w2, #0x1                   	// #1
    3e88:	d2820001 	mov	x1, #0x1000                	// #4096
    3e8c:	940000db 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:75

	if (clock_reg) {
    3e90:	b40001b3 	cbz	x19, 3ec4 <arch_console_init+0xd4>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:76
		map_range(clock_reg, PAGE_SIZE, MAP_UNCACHED);
    3e94:	aa1303e0 	mov	x0, x19
    3e98:	52800022 	mov	w2, #0x1                   	// #1
    3e9c:	d2820001 	mov	x1, #0x1000                	// #4096
    3ea0:	940000d6 	bl	41f8 <map_range>
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    3ea4:	b9400260 	ldr	w0, [x19]
arch_console_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:79
		clock_gates = mmio_read32(clock_reg);
		if (gate_inverted)
			clock_gates &= ~(1 << gate_nr);
    3ea8:	52800022 	mov	w2, #0x1                   	// #1
    3eac:	1ad72041 	lsl	w1, w2, w23
    3eb0:	710002df 	cmp	w22, #0x0
    3eb4:	0a210002 	bic	w2, w0, w1
    3eb8:	2a000021 	orr	w1, w1, w0
    3ebc:	1a820021 	csel	w1, w1, w2, eq  // eq = none
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    3ec0:	b9000261 	str	w1, [x19]
arch_console_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/printk.c:84
		else
			clock_gates |= (1 << gate_nr);
		mmio_write32(clock_reg, clock_gates);
	}
}
    3ec4:	a9415bf5 	ldp	x21, x22, [sp, #16]
    3ec8:	a9427bf7 	ldp	x23, x30, [sp, #32]
    3ecc:	a8c353f3 	ldp	x19, x20, [sp], #48
    3ed0:	d65f03c0 	ret

Disassembly of section .text.timer_get_frequency:

0000000000003ed4 <timer_get_frequency>:
timer_get_frequency():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:48

unsigned long timer_get_frequency(void)
{
	unsigned long freq;

	arm_read_sysreg(CNTFRQ_EL0, freq);
    3ed4:	d53be000 	mrs	x0, cntfrq_el0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:50
	return freq;
}
    3ed8:	d65f03c0 	ret

Disassembly of section .text.timer_get_ticks:

0000000000003edc <timer_get_ticks>:
timer_get_ticks():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:56

u64 timer_get_ticks(void)
{
	u64 pct64;

	arm_read_sysreg(CNTPCT_EL0, pct64);
    3edc:	d53be020 	mrs	x0, cntpct_el0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:58
	return pct64;
}
    3ee0:	d65f03c0 	ret

Disassembly of section .text.timer_ticks_to_ns:

0000000000003ee4 <timer_ticks_to_ns>:
timer_ticks_to_ns():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:73
	return cnt;
}

u64 timer_ticks_to_ns(u64 ticks)
{
	return emul_division(ticks * 1000,
    3ee4:	d2807d01 	mov	x1, #0x3e8                 	// #1000
    3ee8:	9b017c01 	mul	x1, x0, x1
timer_get_frequency():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:48
	arm_read_sysreg(CNTFRQ_EL0, freq);
    3eec:	d53be002 	mrs	x2, cntfrq_el0
timer_ticks_to_ns():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:74
			     timer_get_frequency() / 1000 / 1000);
    3ef0:	d2884800 	mov	x0, #0x4240                	// #16960
    3ef4:	f2a001e0 	movk	x0, #0xf, lsl #16
    3ef8:	9ac00842 	udiv	x2, x2, x0
emul_division():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:62
	unsigned long cnt = 0;
    3efc:	d2800000 	mov	x0, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:64
	while (val > div) {
    3f00:	eb01005f 	cmp	x2, x1
    3f04:	54000043 	b.cc	3f0c <timer_ticks_to_ns+0x28>  // b.lo, b.ul, b.last
timer_ticks_to_ns():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:75
}
    3f08:	d65f03c0 	ret
emul_division():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:65
		val -= div;
    3f0c:	cb020021 	sub	x1, x1, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:66
		cnt++;
    3f10:	91000400 	add	x0, x0, #0x1
    3f14:	17fffffb 	b	3f00 <timer_ticks_to_ns+0x1c>

Disassembly of section .text.timer_start:

0000000000003f18 <timer_start>:
timer_start():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:79

void timer_start(u64 timeout)
{
	arm_write_sysreg(CNTV_TVAL_EL0, timeout);
    3f18:	d51be300 	msr	cntv_tval_el0, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:80
	arm_write_sysreg(CNTV_CTL_EL0, 1);
    3f1c:	d2800020 	mov	x0, #0x1                   	// #1
    3f20:	d51be320 	msr	cntv_ctl_el0, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/timer.c:81
}
    3f24:	d65f03c0 	ret

Disassembly of section .text.c_entry:

0000000000003f28 <c_entry>:
c_entry():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:43
#include <inmate.h>

void __attribute__((noreturn)) c_entry(void);

void __attribute__((noreturn)) c_entry(void)
{
    3f28:	f81f0ffe 	str	x30, [sp, #-16]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:44
	arch_init_early();
    3f2c:	940000fe 	bl	4324 <arch_init_early>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:47

	/* check if the ABI version of the communication region matches */
	if (comm_region->revision != COMM_REGION_ABI_REVISION ||
    3f30:	d2b00000 	mov	x0, #0x80000000            	// #2147483648
    3f34:	79400c01 	ldrh	w1, [x0, #6]
    3f38:	7100043f 	cmp	w1, #0x1
    3f3c:	540000e0 	b.eq	3f58 <c_entry+0x30>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:50
	    memcmp(comm_region->signature, COMM_REGION_MAGIC,
		   sizeof(comm_region->signature))) {
		comm_region->cell_state = JAILHOUSE_CELL_FAILED_COMM_REV;
    3f40:	d2b00000 	mov	x0, #0x80000000            	// #2147483648
    3f44:	52800081 	mov	w1, #0x4                   	// #4
    3f48:	b9000801 	str	w1, [x0, #8]
arch_disable_irqs():
/home/clk/workspace/jailhouse/inmates/lib/arm64/include/arch/inmate.h:43

void __attribute__((used)) vector_irq(void);

static inline void arch_disable_irqs(void)
{
	asm volatile("msr daifset, #3"); /* disable IRQs and FIQs */
    3f4c:	d50343df 	msr	daifset, #0x3
halt():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:95
		asm volatile("wfi" : : : "memory");
    3f50:	d503207f 	wfi
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:94
	while (1)
    3f54:	17ffffff 	b	3f50 <c_entry+0x28>
c_entry():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:48 (discriminator 1)
	    memcmp(comm_region->signature, COMM_REGION_MAGIC,
    3f58:	b0000001 	adrp	x1, 4000 <uart_8250_init+0x38>
    3f5c:	d28000c2 	mov	x2, #0x6                   	// #6
    3f60:	911ac021 	add	x1, x1, #0x6b0
    3f64:	97ffff33 	bl	3c30 <memcmp>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:47 (discriminator 1)
	if (comm_region->revision != COMM_REGION_ABI_REVISION ||
    3f68:	35fffec0 	cbnz	w0, 3f40 <c_entry+0x18>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../setup.c:52
	} else {
		inmate_main();
    3f6c:	97fffc5f 	bl	30e8 <inmate_main>
    3f70:	17fffff7 	b	3f4c <c_entry+0x24>

Disassembly of section .text.reg_out_mmio32:

0000000000003f74 <reg_out_mmio32>:
reg_out_mmio32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:55
#define UART_LSR		0x5
#define  UART_LSR_THRE		0x20

static void reg_out_mmio32(struct uart_chip *chip, unsigned int reg, u32 value)
{
	        mmio_write32(chip->base + reg * 4, value);
    3f74:	f9400800 	ldr	x0, [x0, #16]
    3f78:	531e7421 	lsl	w1, w1, #2
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    3f7c:	b8216802 	str	w2, [x0, x1]
reg_out_mmio32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:56
}
    3f80:	d65f03c0 	ret

Disassembly of section .text.reg_in_mmio32:

0000000000003f84 <reg_in_mmio32>:
reg_in_mmio32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:60

static u32 reg_in_mmio32(struct uart_chip *chip, unsigned int reg)
{
	        return mmio_read32(chip->base + reg * 4);
    3f84:	f9400800 	ldr	x0, [x0, #16]
    3f88:	531e7421 	lsl	w1, w1, #2
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    3f8c:	b8616800 	ldr	w0, [x0, x1]
reg_in_mmio32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:61
}
    3f90:	d65f03c0 	ret

Disassembly of section .text.uart_8250_is_busy:

0000000000003f94 <uart_8250_is_busy>:
uart_8250_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:74
		chip->reg_out(chip, UART_LCR, UART_LCR_8N1);
	}
}

static bool uart_8250_is_busy(struct uart_chip *chip)
{
    3f94:	f81f0ffe 	str	x30, [sp, #-16]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:75
	return !(chip->reg_in(chip, UART_LSR) & UART_LSR_THRE);
    3f98:	528000a1 	mov	w1, #0x5                   	// #5
    3f9c:	f9401402 	ldr	x2, [x0, #40]
    3fa0:	d63f0040 	blr	x2
    3fa4:	d27b0000 	eor	x0, x0, #0x20
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:76
}
    3fa8:	f84107fe 	ldr	x30, [sp], #16
    3fac:	53051400 	ubfx	w0, w0, #5, #1
    3fb0:	d65f03c0 	ret

Disassembly of section .text.uart_8250_write:

0000000000003fb4 <uart_8250_write>:
uart_8250_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:80

static void uart_8250_write(struct uart_chip *chip, char c)
{
	chip->reg_out(chip, UART_TX, c);
    3fb4:	f9401003 	ldr	x3, [x0, #32]
    3fb8:	12001c22 	and	w2, w1, #0xff
    3fbc:	52800001 	mov	w1, #0x0                   	// #0
    3fc0:	aa0303f0 	mov	x16, x3
    3fc4:	d61f0200 	br	x16

Disassembly of section .text.uart_8250_init:

0000000000003fc8 <uart_8250_init>:
uart_8250_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:65
	if (chip->divider) {
    3fc8:	b9401801 	ldr	w1, [x0, #24]
    3fcc:	34000301 	cbz	w1, 402c <uart_8250_init+0x64>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:64
{
    3fd0:	a9bf7bf3 	stp	x19, x30, [sp, #-16]!
    3fd4:	aa0003f3 	mov	x19, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:66
		chip->reg_out(chip, UART_LCR, UART_LCR_DLAB);
    3fd8:	52801002 	mov	w2, #0x80                  	// #128
    3fdc:	f9401003 	ldr	x3, [x0, #32]
    3fe0:	52800061 	mov	w1, #0x3                   	// #3
    3fe4:	d63f0060 	blr	x3
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:67
		chip->reg_out(chip, UART_DLL, chip->divider);
    3fe8:	b9401a62 	ldr	w2, [x19, #24]
    3fec:	aa1303e0 	mov	x0, x19
    3ff0:	f9401263 	ldr	x3, [x19, #32]
    3ff4:	52800001 	mov	w1, #0x0                   	// #0
    3ff8:	d63f0060 	blr	x3
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:68
		chip->reg_out(chip, UART_DLM, 0);
    3ffc:	f9401263 	ldr	x3, [x19, #32]
    4000:	aa1303e0 	mov	x0, x19
    4004:	52800002 	mov	w2, #0x0                   	// #0
    4008:	52800021 	mov	w1, #0x1                   	// #1
    400c:	d63f0060 	blr	x3
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:69
		chip->reg_out(chip, UART_LCR, UART_LCR_8N1);
    4010:	aa1303e0 	mov	x0, x19
    4014:	52800062 	mov	w2, #0x3                   	// #3
    4018:	f9401263 	ldr	x3, [x19, #32]
    401c:	2a0203e1 	mov	w1, w2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:71
}
    4020:	a8c17bf3 	ldp	x19, x30, [sp], #16
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../uart-8250.c:69
		chip->reg_out(chip, UART_LCR, UART_LCR_8N1);
    4024:	aa0303f0 	mov	x16, x3
    4028:	d61f0200 	br	x16
    402c:	d65f03c0 	ret

Disassembly of section .text.gic_v2_enable:

0000000000004030 <gic_v2_enable>:
gic_v2_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:57
static void *gicc_v2_base;
static void *gicd_v2_base;

static void gic_v2_enable(unsigned int irqn)
{
	mmio_write32(gicd_v2_base + GICD_ISENABLER + ((irqn >> 3) & ~0x3),
    4030:	d0ffffe2 	adrp	x2, 2000 <expected_ticks>
    4034:	53037c01 	lsr	w1, w0, #3
    4038:	927e6821 	and	x1, x1, #0x1ffffffc
    403c:	f9402043 	ldr	x3, [x2, #64]
    4040:	91040021 	add	x1, x1, #0x100
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:58
		     1 << (irqn & 0x1f));
    4044:	52800022 	mov	w2, #0x1                   	// #1
    4048:	1ac02040 	lsl	w0, w2, w0
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    404c:	b8216860 	str	w0, [x3, x1]
gic_v2_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:59
}
    4050:	d65f03c0 	ret

Disassembly of section .text.gic_v2_write_eoi:

0000000000004054 <gic_v2_write_eoi>:
gic_v2_write_eoi():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:78
	return 0;
}

static void gic_v2_write_eoi(u32 irqn)
{
	mmio_write32(gicc_v2_base + GICC_EOIR, irqn);
    4054:	d0ffffe1 	adrp	x1, 2000 <expected_ticks>
    4058:	f9402421 	ldr	x1, [x1, #72]
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
    405c:	b9001020 	str	w0, [x1, #16]
gic_v2_write_eoi():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:79
}
    4060:	d65f03c0 	ret

Disassembly of section .text.gic_v2_read_ack:

0000000000004064 <gic_v2_read_ack>:
gic_v2_read_ack():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:83

static u32 gic_v2_read_ack(void)
{
	return mmio_read32(gicc_v2_base + GICC_IAR) & 0x3ff;
    4064:	d0ffffe0 	adrp	x0, 2000 <expected_ticks>
    4068:	f9402400 	ldr	x0, [x0, #72]
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    406c:	b9400c00 	ldr	w0, [x0, #12]
gic_v2_read_ack():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:84
}
    4070:	12002400 	and	w0, w0, #0x3ff
    4074:	d65f03c0 	ret

Disassembly of section .text.gic_v2_init:

0000000000004078 <gic_v2_init>:
gic_v2_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:62
{
    4078:	a9be53f3 	stp	x19, x20, [sp, #-32]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:63
	gicc_v2_base = (void*)(unsigned long)comm_region->gicc_base;
    407c:	d2b00001 	mov	x1, #0x80000000            	// #2147483648
    4080:	d0fffff3 	adrp	x19, 2000 <expected_ticks>
    4084:	91010274 	add	x20, x19, #0x40
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:62
{
    4088:	f9000bfe 	str	x30, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:66
	map_range(gicc_v2_base, PAGE_SIZE, MAP_UNCACHED);
    408c:	52800022 	mov	w2, #0x1                   	// #1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:63
	gicc_v2_base = (void*)(unsigned long)comm_region->gicc_base;
    4090:	f9402420 	ldr	x0, [x1, #72]
    4094:	f9000680 	str	x0, [x20, #8]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:64
	gicd_v2_base = (void*)(unsigned long)comm_region->gicd_base;
    4098:	f9402021 	ldr	x1, [x1, #64]
    409c:	f9002261 	str	x1, [x19, #64]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:66
	map_range(gicc_v2_base, PAGE_SIZE, MAP_UNCACHED);
    40a0:	d2820001 	mov	x1, #0x1000                	// #4096
    40a4:	94000055 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:67
	map_range(gicd_v2_base, PAGE_SIZE, MAP_UNCACHED);
    40a8:	f9402260 	ldr	x0, [x19, #64]
    40ac:	52800022 	mov	w2, #0x1                   	// #1
    40b0:	d2820001 	mov	x1, #0x1000                	// #4096
    40b4:	94000051 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:69
	mmio_write32(gicc_v2_base + GICC_CTLR, GICC_CTLR_GRPEN1);
    40b8:	f9400681 	ldr	x1, [x20, #8]
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    40bc:	52800020 	mov	w0, #0x1                   	// #1
    40c0:	52801e02 	mov	w2, #0xf0                  	// #240
    40c4:	b9000020 	str	w0, [x1]
    40c8:	b9000422 	str	w2, [x1, #4]
gic_v2_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:71
	mmio_write32(gicd_v2_base + GICD_CTLR, GICD_CTLR_ENABLE);
    40cc:	f9402261 	ldr	x1, [x19, #64]
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
    40d0:	b9000020 	str	w0, [x1]
gic_v2_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v2.c:74
}
    40d4:	52800000 	mov	w0, #0x0                   	// #0
    40d8:	f9400bfe 	ldr	x30, [sp, #16]
    40dc:	a8c253f3 	ldp	x19, x20, [sp], #32
    40e0:	d65f03c0 	ret

Disassembly of section .text.gic_v3_write_eoi:

00000000000040e4 <gic_v3_write_eoi>:
gic_v3_write_eoi():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:113
	return 0;
}

static void gic_v3_write_eoi(u32 irqn)
{
	arm_write_sysreg(ICC_EOIR1_EL1, irqn);
    40e4:	2a0003e0 	mov	w0, w0
    40e8:	d518cc20 	msr	s3_0_c12_c12_1, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:114
}
    40ec:	d65f03c0 	ret

Disassembly of section .text.gic_v3_read_ack:

00000000000040f0 <gic_v3_read_ack>:
gic_v3_read_ack():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:120

static u32 gic_v3_read_ack(void)
{
	u32 val;

	arm_read_sysreg(ICC_IAR1_EL1, val);
    40f0:	d538cc00 	mrs	x0, s3_0_c12_c12_0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:122
	return val & 0xffffff;
}
    40f4:	12005c00 	and	w0, w0, #0xffffff
    40f8:	d65f03c0 	ret

Disassembly of section .text.gic_v3_init:

00000000000040fc <gic_v3_init>:
gic_v3_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:65
{
    40fc:	a9be53f3 	stp	x19, x20, [sp, #-32]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:72
	redist_addr = (void*)(unsigned long)comm_region->gicr_base;
    4100:	d2b00000 	mov	x0, #0x80000000            	// #2147483648
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:73
	gicd_v3_base = (void*)(unsigned long)comm_region->gicd_base;
    4104:	d0fffff3 	adrp	x19, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:65
{
    4108:	f9000bfe 	str	x30, [sp, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:75
	map_range(gicd_v3_base, PAGE_SIZE, MAP_UNCACHED);
    410c:	52800022 	mov	w2, #0x1                   	// #1
    4110:	d2820001 	mov	x1, #0x1000                	// #4096
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:72
	redist_addr = (void*)(unsigned long)comm_region->gicr_base;
    4114:	f9402814 	ldr	x20, [x0, #80]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:73
	gicd_v3_base = (void*)(unsigned long)comm_region->gicd_base;
    4118:	f9402000 	ldr	x0, [x0, #64]
    411c:	f9002a60 	str	x0, [x19, #80]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:75
	map_range(gicd_v3_base, PAGE_SIZE, MAP_UNCACHED);
    4120:	94000036 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:76
	map_range(redist_addr, PAGE_SIZE, MAP_UNCACHED);
    4124:	aa1403e0 	mov	x0, x20
    4128:	52800022 	mov	w2, #0x1                   	// #1
    412c:	d2820001 	mov	x1, #0x1000                	// #4096
    4130:	94000032 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:78
	arm_read_sysreg(MPIDR, mpidr);
    4134:	d53800a0 	mrs	x0, mpidr_el1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:79
	aff = (MPIDR_AFFINITY_LEVEL(mpidr, 3) << 24 |
    4138:	d360fc01 	lsr	x1, x0, #32
    413c:	12005c00 	and	w0, w0, #0xffffff
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:91
		if ((typer >> 32) == aff) {
    4140:	2a016000 	orr	w0, w0, w1, lsl #24
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    4144:	91403281 	add	x1, x20, #0xc, lsl #12
    4148:	b97fe821 	ldr	w1, [x1, #16360]
gic_v3_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:87
		if (GICR_PIDR2_ARCH(pidr) != 3)
    414c:	d3441c21 	ubfx	x1, x1, #4, #4
    4150:	71000c3f 	cmp	w1, #0x3
    4154:	540000c1 	b.ne	416c <gic_v3_init+0x70>  // b.any
mmio_read64():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:89
	return *(volatile u64 *)address;
    4158:	f9400681 	ldr	x1, [x20, #8]
gic_v3_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:91
		if ((typer >> 32) == aff) {
    415c:	eb41801f 	cmp	x0, x1, lsr #32
    4160:	540000e0 	b.eq	417c <gic_v3_init+0x80>  // b.none
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:96
		redist_addr += 0x20000;
    4164:	91408294 	add	x20, x20, #0x20, lsl #12
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:97
	} while (!(typer & GICR_TYPER_Last));
    4168:	3627fee1 	tbz	w1, #4, 4144 <gic_v3_init+0x48>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:100
		return -1;
    416c:	12800000 	mov	w0, #0xffffffff            	// #-1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:109
}
    4170:	f9400bfe 	ldr	x30, [sp, #16]
    4174:	a8c253f3 	ldp	x19, x20, [sp], #32
    4178:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:102
	gicr_v3_base = gicr;
    417c:	91014273 	add	x19, x19, #0x50
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:104
	arm_write_sysreg(ICC_CTLR_EL1, 0);
    4180:	d2800000 	mov	x0, #0x0                   	// #0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:102
	gicr_v3_base = gicr;
    4184:	f9000674 	str	x20, [x19, #8]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:104
	arm_write_sysreg(ICC_CTLR_EL1, 0);
    4188:	d518cc80 	msr	s3_0_c12_c12_4, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:105
	arm_write_sysreg(ICC_PMR_EL1, 0xf0);
    418c:	d2801e00 	mov	x0, #0xf0                  	// #240
    4190:	d5184600 	msr	s3_0_c4_c6_0, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:106
	arm_write_sysreg(ICC_IGRPEN1_EL1, ICC_IGRPEN1_EN);
    4194:	d2800020 	mov	x0, #0x1                   	// #1
    4198:	d518cce0 	msr	s3_0_c12_c12_7, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:108
	return 0;
    419c:	52800000 	mov	w0, #0x0                   	// #0
    41a0:	17fffff4 	b	4170 <gic_v3_init+0x74>

Disassembly of section .text.gic_v3_enable:

00000000000041a4 <gic_v3_enable>:
gic_v3_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:56
	if (is_sgi_ppi(irqn))
    41a4:	71007c1f 	cmp	w0, #0x1f
    41a8:	54000108 	b.hi	41c8 <gic_v3_enable+0x24>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:57
		mmio_write32(gicr_v3_base + GICR_SGI_BASE + GICR_ISENABLER,
    41ac:	d0ffffe1 	adrp	x1, 2000 <expected_ticks>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:58
			     1 << irqn);
    41b0:	52800022 	mov	w2, #0x1                   	// #1
    41b4:	1ac02040 	lsl	w0, w2, w0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:57
		mmio_write32(gicr_v3_base + GICR_SGI_BASE + GICR_ISENABLER,
    41b8:	f9402c21 	ldr	x1, [x1, #88]
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    41bc:	91404021 	add	x1, x1, #0x10, lsl #12
    41c0:	b9010020 	str	w0, [x1, #256]
gic_v3_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:62
}
    41c4:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:59
	else if (is_spi(irqn))
    41c8:	51008001 	sub	w1, w0, #0x20
    41cc:	710f6c3f 	cmp	w1, #0x3db
    41d0:	54ffffa8 	b.hi	41c4 <gic_v3_enable+0x20>  // b.pmore
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:60
		mmio_write32(gicd_v3_base + GICD_ISENABLER + (irqn / 32) * 4,
    41d4:	53057c01 	lsr	w1, w0, #5
    41d8:	d0ffffe2 	adrp	x2, 2000 <expected_ticks>
    41dc:	531e7421 	lsl	w1, w1, #2
    41e0:	f9402843 	ldr	x3, [x2, #80]
    41e4:	11040021 	add	w1, w1, #0x100
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:61
			     1 << (irqn % 32));
    41e8:	52800022 	mov	w2, #0x1                   	// #1
    41ec:	1ac02040 	lsl	w0, w2, w0
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
    41f0:	b8216860 	str	w0, [x3, x1]
gic_v3_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/gic-v3.c:62
}
    41f4:	17fffff4 	b	41c4 <gic_v3_enable+0x20>

Disassembly of section .text.map_range:

00000000000041f8 <map_range>:
map_range():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:45
#include <asm/sysregs.h>

static u64 __attribute__((aligned(4096))) page_directory[4];

void map_range(void *start, unsigned long size, enum map_type map_type)
{
    41f8:	a9bb53f3 	stp	x19, x20, [sp, #-80]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:52
	unsigned pgd_index;
	u64 *pmd;

	vaddr = (unsigned long)start;

	size += (vaddr & ~HUGE_PAGE_MASK) + HUGE_PAGE_SIZE - 1;
    41fc:	92405013 	and	x19, x0, #0x1fffff
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:45
{
    4200:	a90263f7 	stp	x23, x24, [sp, #32]
    4204:	2a0203f8 	mov	w24, w2
    4208:	b24053e2 	mov	x2, #0x1fffff              	// #2097151
    420c:	8b020021 	add	x1, x1, x2
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:57
	size &= HUGE_PAGE_MASK;

	while (size) {
		pgd_index = PGD_INDEX(vaddr);
		if (!(page_directory[pgd_index] & LONG_DESC_TABLE)) {
    4210:	f0fffff7 	adrp	x23, 3000 <page_directory>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:52
	size += (vaddr & ~HUGE_PAGE_MASK) + HUGE_PAGE_SIZE - 1;
    4214:	8b010273 	add	x19, x19, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:57
		if (!(page_directory[pgd_index] & LONG_DESC_TABLE)) {
    4218:	910002f7 	add	x23, x23, #0x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:53
	size &= HUGE_PAGE_MASK;
    421c:	926baa73 	and	x19, x19, #0xffffffffffe00000
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:45
{
    4220:	a9015bf5 	stp	x21, x22, [sp, #16]
    4224:	8b130015 	add	x21, x0, x19
    4228:	a9036bf9 	stp	x25, x26, [sp, #48]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:71
			pmd = (u64*)(unsigned long)
				(page_directory[pgd_index] & ~LONG_DESC_TABLE);
		}

		pmd_entry = vaddr & HUGE_PAGE_MASK;
		pmd_entry |= LATTR_AF | LATTR_INNER_SHAREABLE | \
    422c:	d280e03a 	mov	x26, #0x701                 	// #1793
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:45
{
    4230:	f90023fe 	str	x30, [sp, #64]
    4234:	cb1302b6 	sub	x22, x21, x19
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:55
	while (size) {
    4238:	b5000113 	cbnz	x19, 4258 <map_range+0x60>
synchronization_barrier():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/asm/processor.h:56
	dmb(ish);
}

static inline void synchronization_barrier(void)
{
	dsb(ish);
    423c:	d5033b9f 	dsb	ish
map_range():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:90
	 * As long es we only add entries and do not modify entries, a
	 * synchronization barrier is enough to propagate changes. Otherwise we
	 * need to flush the TLB.
	 */
	synchronization_barrier();
}
    4240:	a9415bf5 	ldp	x21, x22, [sp, #16]
    4244:	a94263f7 	ldp	x23, x24, [sp, #32]
    4248:	a9436bf9 	ldp	x25, x26, [sp, #48]
    424c:	f94023fe 	ldr	x30, [sp, #64]
    4250:	a8c553f3 	ldp	x19, x20, [sp], #80
    4254:	d65f03c0 	ret
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:57
		if (!(page_directory[pgd_index] & LONG_DESC_TABLE)) {
    4258:	d35ef6d9 	ubfx	x25, x22, #30, #32
    425c:	f8797af4 	ldr	x20, [x23, x25, lsl #3]
    4260:	f240069f 	tst	x20, #0x3
    4264:	540002a1 	b.ne	42b8 <map_range+0xc0>  // b.any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:58
			pmd = alloc(PAGE_SIZE, PAGE_SIZE);
    4268:	d2820001 	mov	x1, #0x1000                	// #4096
    426c:	aa0103e0 	mov	x0, x1
    4270:	940000a5 	bl	4504 <alloc>
    4274:	aa0003f4 	mov	x20, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:59
			memset(pmd, 0, PAGE_SIZE);
    4278:	d2820002 	mov	x2, #0x1000                	// #4096
    427c:	52800001 	mov	w1, #0x0                   	// #0
    4280:	97fffe65 	bl	3c14 <memset>
synchronization_barrier():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/asm/processor.h:56
    4284:	d5033b9f 	dsb	ish
map_range():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:64
				(unsigned long)pmd | LONG_DESC_TABLE;
    4288:	b2400680 	orr	x0, x20, #0x3
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:63
			page_directory[pgd_index] =
    428c:	f8397ae0 	str	x0, [x23, x25, lsl #3]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:70
		pmd_entry = vaddr & HUGE_PAGE_MASK;
    4290:	926baac0 	and	x0, x22, #0xffffffffffe00000
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:78
		pmd[PMD_INDEX(vaddr)] = pmd_entry;
    4294:	d35576d6 	ubfx	x22, x22, #21, #9
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:71
		pmd_entry |= LATTR_AF | LATTR_INNER_SHAREABLE | \
    4298:	aa1a0001 	orr	x1, x0, x26
    429c:	7100031f 	cmp	w24, #0x0
    42a0:	d280e0a2 	mov	x2, #0x705                 	// #1797
    42a4:	aa020000 	orr	x0, x0, x2
    42a8:	9a811000 	csel	x0, x0, x1, ne  // ne = any
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:80
		size -= HUGE_PAGE_SIZE;
    42ac:	d1480273 	sub	x19, x19, #0x200, lsl #12
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:78
		pmd[PMD_INDEX(vaddr)] = pmd_entry;
    42b0:	f8367a80 	str	x0, [x20, x22, lsl #3]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:81
		vaddr += HUGE_PAGE_SIZE;
    42b4:	17ffffe0 	b	4234 <map_range+0x3c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:66
			pmd = (u64*)(unsigned long)
    42b8:	927ef694 	and	x20, x20, #0xfffffffffffffffc
    42bc:	17fffff5 	b	4290 <map_range+0x98>

Disassembly of section .text.arch_mmu_enable:

00000000000042c0 <arch_mmu_enable>:
arch_mmu_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:93

void arch_mmu_enable(void)
{
    42c0:	f81f0ffe 	str	x30, [sp, #-16]!
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:96
	unsigned long mair, sctlr;

	map_range((void*)CONFIG_INMATE_BASE, 0x10000, MAP_CACHED);
    42c4:	52800002 	mov	w2, #0x0                   	// #0
    42c8:	d2a00021 	mov	x1, #0x10000               	// #65536
    42cc:	d2800000 	mov	x0, #0x0                   	// #0
    42d0:	97ffffca 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:97
	map_range((void*)COMM_REGION_BASE, PAGE_SIZE, MAP_CACHED);
    42d4:	52800002 	mov	w2, #0x0                   	// #0
    42d8:	d2820001 	mov	x1, #0x1000                	// #4096
    42dc:	d2b00000 	mov	x0, #0x80000000            	// #2147483648
    42e0:	97ffffc6 	bl	41f8 <map_range>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:108
	 * Attributes 0: inner/outer: normal memory, outer write-back
	 *		 non-transient
	 * Attributes 1: device memory
	 */
	mair = MAIR_ATTR(1, MAIR_ATTR_DEVICE) | MAIR_ATTR(0, MAIR_ATTR_WBRWA);
	arm_write_sysreg(MAIR, mair);
    42e4:	d2801fe0 	mov	x0, #0xff                  	// #255
    42e8:	d518a200 	msr	mair_el1, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:110

	arm_write_sysreg(TRANSL_CONT_REG, TRANSL_CONT_REG_SETTINGS);
    42ec:	d286a320 	mov	x0, #0x3519                	// #13593
    42f0:	f2c000a0 	movk	x0, #0x5, lsl #32
    42f4:	d5182040 	msr	tcr_el1, x0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:112

	arm_write_sysreg(TTBR0, page_directory);
    42f8:	f0ffffe0 	adrp	x0, 3000 <page_directory>
    42fc:	91000000 	add	x0, x0, #0x0
    4300:	d5182000 	msr	ttbr0_el1, x0
instruction_barrier():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/asm/processor.h:61
}

static inline void instruction_barrier(void)
{
	asm volatile("isb");
    4304:	d5033fdf 	isb
arch_mmu_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:116
	/* This barrier ensures that TTBR0 is set before enabling the MMU. */
	instruction_barrier();

	arm_read_sysreg(SCTLR, sctlr);
    4308:	d5381000 	mrs	x0, sctlr_el1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:117
	sctlr |= SCTLR_MMU_CACHES;
    430c:	d28200a1 	mov	x1, #0x1005                	// #4101
    4310:	aa010000 	orr	x0, x0, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:118
	arm_write_sysreg(SCTLR, sctlr);
    4314:	d5181000 	msr	sctlr_el1, x0
instruction_barrier():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/asm/processor.h:61
    4318:	d5033fdf 	isb
arch_mmu_enable():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/mem.c:122
	/* This barrier ensures that the MMU is actually on */
	instruction_barrier();
	/* MMU is enabled from now on */
}
    431c:	f84107fe 	ldr	x30, [sp], #16
    4320:	d65f03c0 	ret

Disassembly of section .text.arch_init_early:

0000000000004324 <arch_init_early>:
arch_init_early():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/setup.c:43

#include <inmate.h>

void arch_init_early(void)
{
	arch_mmu_enable();
    4324:	17ffffe7 	b	42c0 <arch_mmu_enable>

Disassembly of section .text.uart_hscif_is_busy:

0000000000004328 <uart_hscif_is_busy>:
uart_hscif_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:72
	}
}

static bool uart_hscif_is_busy(struct uart_chip *chip)
{
	return !(mmio_read16(chip->base + HSCIF_HSFSR) & HSCIF_HSFSR_TDFE);
    4328:	f9400800 	ldr	x0, [x0, #16]
mmio_read16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:69
	return *(volatile u16 *)address;
    432c:	79402000 	ldrh	w0, [x0, #16]
uart_hscif_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:72
    4330:	d27b0000 	eor	x0, x0, #0x20
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:73
}
    4334:	53051400 	ubfx	w0, w0, #5, #1
    4338:	d65f03c0 	ret

Disassembly of section .text.uart_hscif_write:

000000000000433c <uart_hscif_write>:
uart_hscif_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:77

static void uart_hscif_write(struct uart_chip *chip, char c)
{
	mmio_write8(chip->base + HSCIF_HSFTDR, c);
    433c:	f9400802 	ldr	x2, [x0, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:76
{
    4340:	12001c21 	and	w1, w1, #0xff
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
	*(volatile u8 *)address = value;
    4344:	39003041 	strb	w1, [x2, #12]
uart_hscif_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:78
	mmio_write16(chip->base + HSCIF_HSFSR,
    4348:	529ff3e2 	mov	w2, #0xff9f                	// #65439
    434c:	f9400801 	ldr	x1, [x0, #16]
mmio_read16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:69
	return *(volatile u16 *)address;
    4350:	79402020 	ldrh	w0, [x1, #16]
uart_hscif_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:78
    4354:	0a020000 	and	w0, w0, w2
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    4358:	79002020 	strh	w0, [x1, #16]
uart_hscif_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:81
		     mmio_read16(chip->base + HSCIF_HSFSR) &
				 ~(HSCIF_HSFSR_TDFE | HSCIF_HSFSR_TEND));
}
    435c:	d65f03c0 	ret

Disassembly of section .text.uart_hscif_init:

0000000000004360 <uart_hscif_init>:
uart_hscif_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:60
	if (chip->divider) {
    4360:	b9401801 	ldr	w1, [x0, #24]
    4364:	34000201 	cbz	w1, 43a4 <uart_hscif_init+0x44>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:61
		hsscr = mmio_read16(chip->base + HSCIF_HSSCR);
    4368:	f9400803 	ldr	x3, [x0, #16]
mmio_read16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:69
	return *(volatile u16 *)address;
    436c:	79401061 	ldrh	w1, [x3, #8]
    4370:	12003c21 	and	w1, w1, #0xffff
uart_hscif_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:62
		mmio_write16(chip->base + HSCIF_HSSCR,
    4374:	121a7422 	and	w2, w1, #0xffffffcf
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:66
		mmio_write16(chip->base + HSCIF_HSSCR, hsscr | HSCIF_HSSCR_TE);
    4378:	321b0021 	orr	w1, w1, #0x20
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:62
		mmio_write16(chip->base + HSCIF_HSSCR,
    437c:	12003c42 	and	w2, w2, #0xffff
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    4380:	79001062 	strh	w2, [x3, #8]
uart_hscif_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:64
		mmio_write8(chip->base + HSCIF_HSBRR, chip->divider);
    4384:	f9400802 	ldr	x2, [x0, #16]
    4388:	39406003 	ldrb	w3, [x0, #24]
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
	*(volatile u8 *)address = value;
    438c:	39001043 	strb	w3, [x2, #4]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    4390:	52800803 	mov	w3, #0x40                  	// #64
uart_hscif_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:65
		mmio_write16(chip->base + HSCIF_HSTTRGR, HSCIF_FIFO_SIZE / 2);
    4394:	f9400802 	ldr	x2, [x0, #16]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
    4398:	7900b043 	strh	w3, [x2, #88]
uart_hscif_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:66
		mmio_write16(chip->base + HSCIF_HSSCR, hsscr | HSCIF_HSSCR_TE);
    439c:	f9400800 	ldr	x0, [x0, #16]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
    43a0:	79001001 	strh	w1, [x0, #8]
uart_hscif_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-hscif.c:68
}
    43a4:	d65f03c0 	ret

Disassembly of section .text.uart_imx_init:

00000000000043a8 <uart_imx_init>:
uart_imx_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:47
#define UTS_TXEMPTY		(1 << 6)

static void uart_imx_init(struct uart_chip *chip)
{
	/* Initialization currently done by Linux. */
}
    43a8:	d65f03c0 	ret

Disassembly of section .text.uart_imx_is_busy:

00000000000043ac <uart_imx_is_busy>:
uart_imx_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:51

static bool uart_imx_is_busy(struct uart_chip *chip)
{
	return !(mmio_read32(chip->base + UTS) & UTS_TXEMPTY);
    43ac:	f9400800 	ldr	x0, [x0, #16]
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    43b0:	b940b400 	ldr	w0, [x0, #180]
uart_imx_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:51
    43b4:	d27a0000 	eor	x0, x0, #0x40
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:52
}
    43b8:	53061800 	ubfx	w0, w0, #6, #1
    43bc:	d65f03c0 	ret

Disassembly of section .text.uart_imx_write:

00000000000043c0 <uart_imx_write>:
uart_imx_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:56

static void uart_imx_write(struct uart_chip *chip, char c)
{
	mmio_write32(chip->base + UTXD, c);
    43c0:	f9400800 	ldr	x0, [x0, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:55
{
    43c4:	12001c21 	and	w1, w1, #0xff
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    43c8:	b9004001 	str	w1, [x0, #64]
uart_imx_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-imx.c:57
}
    43cc:	d65f03c0 	ret

Disassembly of section .text.uart_mvebu_init:

00000000000043d0 <uart_mvebu_init>:
uart_mvebu_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-mvebu.c:48
#define UART_STAT		0xc
#define  UART_STAT_TX_FULL	(1 << 11)

static void uart_mvebu_init(struct uart_chip *chip)
{
}
    43d0:	d65f03c0 	ret

Disassembly of section .text.uart_mvebu_is_busy:

00000000000043d4 <uart_mvebu_is_busy>:
uart_mvebu_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-mvebu.c:52

static bool uart_mvebu_is_busy(struct uart_chip *chip)
{
	return !!(mmio_read32(chip->base + UART_STAT) & UART_STAT_TX_FULL);
    43d4:	f9400800 	ldr	x0, [x0, #16]
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    43d8:	b9400c00 	ldr	w0, [x0, #12]
uart_mvebu_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-mvebu.c:53
}
    43dc:	d34b2c00 	ubfx	x0, x0, #11, #1
    43e0:	d65f03c0 	ret

Disassembly of section .text.uart_mvebu_write:

00000000000043e4 <uart_mvebu_write>:
uart_mvebu_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-mvebu.c:57

static void uart_mvebu_write(struct uart_chip *chip, char c)
{
	mmio_write32(chip->base + UART_TSH, c);
    43e4:	f9400800 	ldr	x0, [x0, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-mvebu.c:56
{
    43e8:	12001c21 	and	w1, w1, #0xff
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    43ec:	b9000401 	str	w1, [x0, #4]
uart_mvebu_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-mvebu.c:58
}
    43f0:	d65f03c0 	ret

Disassembly of section .text.uart_pl011_is_busy:

00000000000043f4 <uart_pl011_is_busy>:
uart_pl011_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:75
}

static bool uart_pl011_is_busy(struct uart_chip *chip)
{
	/* FIFO full or busy */
	return (mmio_read32(chip->base + UARTFR) &
    43f4:	f9400800 	ldr	x0, [x0, #16]
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    43f8:	b9401801 	ldr	w1, [x0, #24]
uart_pl011_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:76
		(UARTFR_TXFF | UARTFR_BUSY)) != 0;
    43fc:	52800500 	mov	w0, #0x28                  	// #40
    4400:	6a00003f 	tst	w1, w0
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:77
}
    4404:	1a9f07e0 	cset	w0, ne  // ne = any
    4408:	d65f03c0 	ret

Disassembly of section .text.uart_pl011_write:

000000000000440c <uart_pl011_write>:
uart_pl011_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:81

static void uart_pl011_write(struct uart_chip *chip, char c)
{
	mmio_write32(chip->base + UARTDR, c);
    440c:	f9400800 	ldr	x0, [x0, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:80
{
    4410:	12001c21 	and	w1, w1, #0xff
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    4414:	b9000001 	str	w1, [x0]
uart_pl011_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:82
}
    4418:	d65f03c0 	ret

Disassembly of section .text.uart_pl011_init:

000000000000441c <uart_pl011_init>:
uart_pl011_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:61
	if (chip->divider) {
    441c:	b9401801 	ldr	w1, [x0, #24]
    4420:	340001c1 	cbz	w1, 4458 <uart_pl011_init+0x3c>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:62
		mmio_write16(chip->base + UARTCR, 0);
    4424:	f9400801 	ldr	x1, [x0, #16]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    4428:	7900603f 	strh	wzr, [x1, #48]
uart_pl011_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:63
		while (mmio_read8(chip->base + UARTFR) & UARTFR_BUSY)
    442c:	f9400801 	ldr	x1, [x0, #16]
mmio_read8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:59
	return *(volatile u8 *)address;
    4430:	39406022 	ldrb	w2, [x1, #24]
uart_pl011_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:63
    4434:	37180142 	tbnz	w2, #3, 445c <uart_pl011_init+0x40>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:65
		mmio_write16(chip->base + UARTIBRD, chip->divider);
    4438:	79403002 	ldrh	w2, [x0, #24]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    443c:	79004822 	strh	w2, [x1, #36]
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
	*(volatile u8 *)address = value;
    4440:	52800c02 	mov	w2, #0x60                  	// #96
uart_pl011_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:66
		mmio_write8(chip->base + UARTLCR_H, UARTLCR_H_WLEN);
    4444:	f9400801 	ldr	x1, [x0, #16]
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
    4448:	3900b022 	strb	w2, [x1, #44]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    444c:	52862021 	mov	w1, #0x3101                	// #12545
uart_pl011_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:67
		mmio_write16(chip->base + UARTCR, UARTCR_EN | UARTCR_TXE |
    4450:	f9400800 	ldr	x0, [x0, #16]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
    4454:	79006001 	strh	w1, [x0, #48]
uart_pl011_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-pl011.c:70
}
    4458:	d65f03c0 	ret
cpu_relax():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/asm/processor.h:47
}
    445c:	17fffff4 	b	442c <uart_pl011_init+0x10>

Disassembly of section .text.uart_scifa_is_busy:

0000000000004460 <uart_scifa_is_busy>:
uart_scifa_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:73
	}
}

static bool uart_scifa_is_busy(struct uart_chip *chip)
{
	return !(mmio_read16(chip->base + SCIFA_SCASSR) & SCIFA_SCASSR_TDFE);
    4460:	f9400800 	ldr	x0, [x0, #16]
mmio_read16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:69
	return *(volatile u16 *)address;
    4464:	79402800 	ldrh	w0, [x0, #20]
uart_scifa_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:73
    4468:	d27b0000 	eor	x0, x0, #0x20
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:74
}
    446c:	53051400 	ubfx	w0, w0, #5, #1
    4470:	d65f03c0 	ret

Disassembly of section .text.uart_scifa_write:

0000000000004474 <uart_scifa_write>:
uart_scifa_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:78

static void uart_scifa_write(struct uart_chip *chip, char c)
{
	mmio_write8(chip->base + SCIFA_SCAFTDR, c);
    4474:	f9400802 	ldr	x2, [x0, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:77
{
    4478:	12001c21 	and	w1, w1, #0xff
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
	*(volatile u8 *)address = value;
    447c:	39008041 	strb	w1, [x2, #32]
uart_scifa_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:79
	mmio_write16(chip->base + SCIFA_SCASSR,
    4480:	529ff3e2 	mov	w2, #0xff9f                	// #65439
    4484:	f9400801 	ldr	x1, [x0, #16]
mmio_read16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:69
	return *(volatile u16 *)address;
    4488:	79402820 	ldrh	w0, [x1, #20]
uart_scifa_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:79
    448c:	0a020000 	and	w0, w0, w2
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    4490:	79002820 	strh	w0, [x1, #20]
uart_scifa_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:82
		     mmio_read16(chip->base + SCIFA_SCASSR) &
				 ~(SCIFA_SCASSR_TDFE | SCIFA_SCASSR_TEND));
}
    4494:	d65f03c0 	ret

Disassembly of section .text.uart_scifa_init:

0000000000004498 <uart_scifa_init>:
uart_scifa_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:61
	if (chip->divider) {
    4498:	b9401801 	ldr	w1, [x0, #24]
    449c:	340001e1 	cbz	w1, 44d8 <uart_scifa_init+0x40>
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:62
		scascr = mmio_read16(chip->base + SCIFA_SCASCR);
    44a0:	f9400803 	ldr	x3, [x0, #16]
mmio_read16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:69
	return *(volatile u16 *)address;
    44a4:	79401061 	ldrh	w1, [x3, #8]
    44a8:	12003c21 	and	w1, w1, #0xffff
uart_scifa_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:63
		mmio_write16(chip->base + SCIFA_SCASCR,
    44ac:	121a7422 	and	w2, w1, #0xffffffcf
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:67
		mmio_write16(chip->base + SCIFA_SCASCR, scascr | SCIFA_SCASCR_TE);
    44b0:	321b0021 	orr	w1, w1, #0x20
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:63
		mmio_write16(chip->base + SCIFA_SCASCR,
    44b4:	12003c42 	and	w2, w2, #0xffff
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    44b8:	79001062 	strh	w2, [x3, #8]
uart_scifa_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:65
		mmio_write8(chip->base + SCIFA_SCABRR, chip->divider);
    44bc:	f9400802 	ldr	x2, [x0, #16]
    44c0:	39406003 	ldrb	w3, [x0, #24]
mmio_write8():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:64
	*(volatile u8 *)address = value;
    44c4:	39001043 	strb	w3, [x2, #4]
uart_scifa_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:66
		mmio_write16(chip->base + SCIFA_SCAFCR, SCIFA_TTRG_32BYTES);
    44c8:	f9400802 	ldr	x2, [x0, #16]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
	*(volatile u16 *)address = value;
    44cc:	7900305f 	strh	wzr, [x2, #24]
uart_scifa_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:67
		mmio_write16(chip->base + SCIFA_SCASCR, scascr | SCIFA_SCASCR_TE);
    44d0:	f9400800 	ldr	x0, [x0, #16]
mmio_write16():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:74
    44d4:	79001001 	strh	w1, [x0, #8]
uart_scifa_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-scifa.c:69
}
    44d8:	d65f03c0 	ret

Disassembly of section .text.uart_xuartps_init:

00000000000044dc <uart_xuartps_init>:
uart_xuartps_init():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:48
#define  UART_SR_TXEMPTY	0x8
#define UART_FIFO		0x30

static void uart_xuartps_init(struct uart_chip *chip)
{
}
    44dc:	d65f03c0 	ret

Disassembly of section .text.uart_xuartps_is_busy:

00000000000044e0 <uart_xuartps_is_busy>:
uart_xuartps_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:52

static bool uart_xuartps_is_busy(struct uart_chip *chip)
{
	return !(mmio_read32(chip->base + UART_SR) & UART_SR_TXEMPTY);
    44e0:	f9400800 	ldr	x0, [x0, #16]
mmio_read32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:79
	return *(volatile u32 *)address;
    44e4:	b9402c00 	ldr	w0, [x0, #44]
uart_xuartps_is_busy():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:52
    44e8:	d27d0000 	eor	x0, x0, #0x8
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:53
}
    44ec:	53030c00 	ubfx	w0, w0, #3, #1
    44f0:	d65f03c0 	ret

Disassembly of section .text.uart_xuartps_write:

00000000000044f4 <uart_xuartps_write>:
uart_xuartps_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:57

static void uart_xuartps_write(struct uart_chip *chip, char c)
{
	mmio_write32(chip->base + UART_FIFO, c);
    44f4:	f9400800 	ldr	x0, [x0, #16]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:56
{
    44f8:	12001c21 	and	w1, w1, #0xff
mmio_write32():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/include/inmate.h:84
	*(volatile u32 *)address = value;
    44fc:	b9003001 	str	w1, [x0, #48]
uart_xuartps_write():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/uart-xuartps.c:58
}
    4500:	d65f03c0 	ret

Disassembly of section .text.alloc:

0000000000004504 <alloc>:
alloc():
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../alloc.c:45

static unsigned long heap_pos = (unsigned long)stack_top;

void *alloc(unsigned long size, unsigned long align)
{
	unsigned long base = (heap_pos + align - 1) & ~(align - 1);
    4504:	90000003 	adrp	x3, 4000 <uart_8250_init+0x38>
    4508:	f9449462 	ldr	x2, [x3, #2344]
    450c:	d1000442 	sub	x2, x2, #0x1
    4510:	8b010042 	add	x2, x2, x1
    4514:	cb0103e1 	neg	x1, x1
    4518:	8a010041 	and	x1, x2, x1
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../alloc.c:47

	heap_pos = base + size;
    451c:	8b000020 	add	x0, x1, x0
    4520:	f9049460 	str	x0, [x3, #2344]
/home/clk/workspace/jailhouse/inmates/lib/arm64/../arm-common/../alloc.c:49
	return (void *)base;
}
    4524:	aa0103e0 	mov	x0, x1
    4528:	d65f03c0 	ret
