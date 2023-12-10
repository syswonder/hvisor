ENTRY(arch_entry)

BASE_ADDRESS = 0xffffc0200000;

SECTIONS
{
	. = BASE_ADDRESS;

	.header		: {
		__header_start = .;
		KEEP(*(.header))
	}

	. = ALIGN(4K);
	.text		: { *(.text .text.*) }

	. = ALIGN(4K);
	.rodata		: { *(.rodata .rodata.*) }

	. = ALIGN(4K);
	.data		: { *(.data .data.*) *(.got .got.*) }

	. = ALIGN(4K);
	.bootstrap_page_tables : { bootstrap_pt_l0 = .; 
							. = . + (1 << 12); bootstrap_pt_l1_hyp_uart = .; 
							. = . + (1 << 12); bootstrap_pt_l1_trampoline = .; 
							. = . + (1 << 12); bootstrap_pt_l2_hyp_uart = .; 
							. = . + (1 << 12); bootstrap_pt_l0_s2 = .;
							. = . + (1 << 12); bootstrap_pt_l1_s2 = .; 
							. = . + (1 << 12); } .trampoline : { __trampoline_start = .; *(.trampoline) }

	. = ALIGN(4K);
	.bss		: { *(.bss .bss.*) *(COMMON) 
	. = ALIGN(4K);
	__boot_stack = .; 
	. = . + 0x4000; 
    __boot_stack_top = .; 
	}

	

	. = ALIGN(4K);
	__core_end = .;

	__entry_offset = arch_entry - BASE_ADDRESS;
	__core_size = __core_end - BASE_ADDRESS;

	/DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}