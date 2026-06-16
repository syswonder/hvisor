# Multiboot2 bootloader for Asterinas zone1
# Transitions from 16-bit real mode to 32-bit protected mode,
# sets up GDT with TSS, then jumps to the kernel entry point.
#
# Entry state (from VMCS guest_regs via unrestricted guest):
#   EAX = Multiboot2 magic (0x36D76289)
#   EBX = Multiboot2 info pointer (GPA)
#   ESI = Kernel entry point (GPA)

.section .text
.code16
.global mb2_entry16
mb2_entry16:
    cli
    cld

    mov     ebp, eax        # save multiboot magic
    mov     edi, ebx        # save multiboot info pointer
    mov     ecx, esi        # save kernel entry point

    xor     ax, ax
    mov     ds, ax
    mov     es, ax
    mov     ss, ax

    lgdt    [mb2_gdt_desc]
    mov     eax, cr0
    or      eax, 0x1
    mov     cr0, eax

    ljmp    0x08, mb2_entry32

.code32
.global mb2_entry32
mb2_entry32:
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax

    # Load TSS (entry 4 at selector 0x20)
    mov     ax, 0x20
    ltr     ax

    # Restore Multiboot2 parameters
    mov     eax, ebp
    mov     ebx, edi

    # Set up stack
    mov     esp, MB2_STACK

    # Jump to kernel entry
    jmp     ecx

.align 16
mb2_gdt:
    .quad   0x0000000000000000    # 0x00: null
    .quad   0x00cf9b000000ffff    # 0x08: 32-bit code
    .quad   0x00cf93000000ffff    # 0x10: 32-bit data
    .quad   0x0000000000000000    # 0x18: unused
    .quad   MB2_TSS_DESCRIPTOR    # 0x20: 32-bit available TSS
mb2_gdt_end:

mb2_gdt_desc:
    .short  mb2_gdt_end - mb2_gdt - 1
    .long   mb2_gdt
