	PTE_TABLE_FLAGS =0x3 //11
	PAGE_DEFAULT_FLAG=0x741 //11101000001
	PAGE_DEVICE_FLAG=0x745 //11101000101

    MAIR_FLAG=0x004404ff //10001000000010011111111
    SCTLR_FLAG=0x30c51835 //110000110001010001100000110101
    TCR_FLAG=0x80853510 //10000000100001010011010100010000

    .section .bss
    .align 12 // 对齐到页边界
boot_pt_l0:
    .zero 0x1000

    .align 12
boot_pt_l1:
    .zero 0x2000

    .section .text
    .global set_boot_pt
boot_pt_init:
    // TODO: re-enable hvisor pt when system boot success 
    adrp x1, boot_pt_l0
    adrp x2, boot_pt_l1
    add x2, x2, PTE_TABLE_FLAGS
    str x2, [x1]                // set boot_pt_l0, index=0 -> boot_pt_l1

    ldr x3, =0x1000
    add x2, x2, x3
    str x2, [x1, #8]

    adrp x6, boot_pt_l1
    mov x2, #0 
    ldr x5, =PAGE_DEVICE_FLAG   // fill the l1 pte with type block(1G)

loop:
    cmp x2, #1024               // 40 bit array
    bge end_loop

    lsl x4, x2, #30
    orr x4, x4, x5  
    str x4, [x6] 

    add x2, x2, #1 
    add x6, x6, #0x8            // pte lenth
    
    b loop

end_loop:

    ret

enable_boot_pt:
    adrp x1, boot_pt_l0
    msr ttbr0_el2, x1

    isb
	tlbi alle2
    dsb	nsh

    // enable mmu...
    ldr	x1, =MAIR_FLAG     
    msr	mair_el2, x1       // memory attributes for pagetable
    ldr	x1, =TCR_FLAG
	msr	tcr_el2, x1        // translate control, virt range = [0, 2^48)

	/* Enable MMU, allow cacheability for instructions and data */
	ldr	x1, =SCTLR_FLAG
	msr	sctlr_el2, x1      // system control register

	isb
	tlbi alle2
	dsb	nsh
    ret