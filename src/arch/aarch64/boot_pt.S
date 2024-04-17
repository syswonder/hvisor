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
    .zero 0x1000

    .section .text
    .global set_boot_pt
boot_pt_init:
    adrp x1, boot_pt_l0
    adrp x2, boot_pt_l1
    add x2, x2, PTE_TABLE_FLAGS
    str x2, [x1]                      // set boot_pt_l0, index=0 -> boot_pt_l1

    adrp x1, boot_pt_l1
    ldr x2, =(0x0 | PAGE_DEVICE_FLAG) // device memory at 0x0 ~ 0x40000000
    str x2, [x1]
    ldr x2, =(0x40000000 | PAGE_DEFAULT_FLAG) // RAM at 0x40000000 ~ 0x80000000
    str x2, [x1, #0x8]
    ldr x2, =(0x80000000 | PAGE_DEFAULT_FLAG) // RAM at 0x80000000 ~ 0xC0000000
    str x2, [x1, #0x10]
    ldr x2, =(0xC0000000 | PAGE_DEFAULT_FLAG) // RAM at 0xC0000000 ~ 0x100000000
    str x2, [x1, #0x18]
    ret

enable_boot_pt:
    adrp x1, boot_pt_l0
    msr ttbr0_el2, x1

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
