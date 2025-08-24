OUTPUT_ARCH(i386)
BASE_ADDRESS = 0x8000;

ENTRY(entry16)
SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text .text.*)
    }

    /DISCARD/ : {
        *(.eh_frame) *(.eh_frame_hdr)
    }
}
