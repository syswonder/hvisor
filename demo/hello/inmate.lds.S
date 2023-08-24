
ENTRY(el1_entry)
CONFIG_INMATE_BASE = 0xa0000000;
SECTIONS {
	. = CONFIG_INMATE_BASE;
	.header		: { KEEP(*(.header)) }

	.bss		: {
		*(.bss)
	}

	. = ALIGN(4);
	.text		: {
		*(.text)
	}

	.rodata		: { *(.rodata .rodata.*) }

	.data		: { *(.data .data.*) *(.got .got.*) }

	. = ALIGN(4096);
	. = . + 0x2000;
	stack_top = .;
}


