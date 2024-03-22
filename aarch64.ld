ENTRY(arch_entry)
BASE_ADDRESS = 0x40000000;

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
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
    . = ALIGN(4K);
	__core_end = .;
    . = . + 0x2000000;

    .dtb : {
        *(.dtb)
    }

    . = 0x60000000;
    __vmimg1 = .;
    .vmimg1 : {
        *(.vmimg1)
    }
    . = 0x70000000;
    __vmimg2 = .;
    .vmimg2 : {
        *(.vmimg2)
    }
    
}