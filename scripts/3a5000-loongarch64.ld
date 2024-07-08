ENTRY(arch_entry)
BASE_ADDRESS = 0x9000000080000000;


SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    .trap_entry : {
        *(.trap_entry)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)

        /* .bss inside .data for UEFI environment */
        /* TODO: dynamically clear .bss after boot for smaller image size */
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        ebss = .;
    }

    . = ALIGN(4K);
    edata = .;

    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
    . = ALIGN(4K);
	__core_end = .;
}