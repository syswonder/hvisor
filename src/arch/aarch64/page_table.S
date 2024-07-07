	PTE_TABLE_FLAGS =0x3 //11
	PAGE_DEFAULT_FLAGS=0x741 //11101000001
	PAGE_FLAG_DEVICE=0x745 //11101000101
	MAIR_FLAG=0x004404ff //10001000000010011111111
	SCTLR_FLAG=0x30c51835 //110000110001010001100000110101
	TCR_FLAG=0x80853510 //10000000100001010011010100010000
	S2_BLOCK_FLAG=0x4fd
.macro virt2phys, register
    add	\register, \register, x11 //-FFFF 4060 0000
.endm

.macro phys2virt, register
    sub	\register, \register, x11 //+FFFF 4060 0000
.endm
/*
 * macros used by init_bootstrap_pt
 */
/* clobbers x8,x9 */
.macro	set_pte table, xidx, xval, flags
	add	x8, \xval, #(\flags)
	adr	x9, \table
	add	x9, x9, \xidx, lsl #3
	str	x8, [x9]
.endm

/* clobbers x8,x9 */
.macro	set_block table, index, addr, lvl
	and	x8, \addr, #(((1 << ((\lvl + 1) * 9)) - 1) << (12 + (3 - \lvl) * 9))
	set_pte \table, \index, x8, PAGE_DEFAULT_FLAGS
.endm

/* clobbers x8,x9 */
.macro	set_s2_block table, index, addr, lvl
	and	x8, \addr, #(((1 << ((\lvl + 1) * 9)) - 1) << (12 + (3 - \lvl) * 9))
	set_pte \table, \index, x8, S2_BLOCK_FLAG
.endm

/* clobbers x8,x9 */
.macro	set_block_dev table, index, addr, lvl
	and	x8, \addr, #(((1 << ((\lvl + 1) * 9)) - 1) << (12 + (3 - \lvl) * 9))
	set_pte \table, \index, x8, (PAGE_DEFAULT_FLAGS|PAGE_FLAG_DEVICE)
.endm

/* clobbers x8,x9 */
.macro	set_table parent, index, child

	adr	x8, \child
	set_pte \parent, \index, x8, PTE_TABLE_FLAGS
.endm

.macro	get_index idx, addr, lvl
	ubfx	\idx, \addr, #(12 + (3 - \lvl) * 9), 9     //get index bit from addr depend on lvl, see arm page
.endm