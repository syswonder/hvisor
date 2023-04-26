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
	.bss		: { *(.bss .bss.*) *(COMMON) }

	. = ALIGN(4K);
	__core_end = .;

	__entry_offset = arch_entry - BASE_ADDRESS;
	__core_size = __core_end - BASE_ADDRESS;

	/DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}