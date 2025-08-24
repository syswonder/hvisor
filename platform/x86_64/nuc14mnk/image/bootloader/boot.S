.section .text
.code16
.global entry16
entry16:
    cli
    cld

    mov     ecx, eax
    xor     ax, ax
    mov     ds, ax
    mov     es, ax
    mov     ss, ax

    lgdt    [prot_gdt_desc]
    mov     eax, cr0
    or      eax, 0x1
    mov     cr0, eax

    ljmp    0x8, entry32

.code32
.global entry32
entry32:
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax

    jmp     ecx

.balign 16
prot_gdt:
    .quad 0x0000000000000000    # 0x00: null
    .quad 0x00cf9b000000ffff    # 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    # 0x10: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)

prot_gdt_desc:
    .short  prot_gdt_desc - prot_gdt - 1    # limit
    .long   prot_gdt                        # base
