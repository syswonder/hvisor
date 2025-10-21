.equ    bsp_boot_stack_top,     __core_end + {per_cpu_size}
.equ    multiboot2_header_len,  multiboot2_header_end - multiboot2_header    

.equ    multiboot2_header_tag_end, 0
.equ    multiboot2_header_tag_address, 2
.equ    multiboot2_header_tag_entry_address, 3
.equ    multiboot2_header_tag_framebuffer, 5

.section .text.header

.balign 4
.type multiboot_header, STT_OBJECT
multiboot_header:
    .int    {multiboot_header_magic}
    .int    {multiboot_header_flags}
    .int    -({multiboot_header_magic} + {multiboot_header_flags})
    .int    multiboot_header - {offset}        // header_addr
    .int    skernel - {offset}                 // load_addr
    .int    edata - {offset}                   // load_end
    .int    ebss - {offset}                    // bss_end_addr
    .int    arch_entry - {offset}              // entry_addrs

.align 8
.type multiboot2_header STT_OBJECT
multiboot2_header:
    .int    {multiboot2_header_magic}
    .int    {multiboot2_arch_i386}
    .int    multiboot2_header_len
    .int    -({multiboot2_header_magic} + {multiboot2_arch_i386} + multiboot2_header_len)

.align 8
.type tag_address STT_OBJECT
tag_address:
    .short  multiboot2_header_tag_address
    .short  0
    .int    24
    .int    multiboot2_header - {offset}        // header_addr
    .int    skernel - {offset}                  // load_addr
    .int    edata - {offset}                    // load_end_addr
    .int    bsp_boot_stack_top - {offset}       // bss_end_addr

.align 8
.type tag_entry_address STT_OBJECT
tag_entry_address:
    .short  multiboot2_header_tag_entry_address
    .short  0
    .int    12
    .int    arch_entry - {offset}               // entry_addr

.align 8
.type tag_framebuffer STT_OBJECT
tag_framebuffer:
    .short  multiboot2_header_tag_framebuffer
    .short  0
    .int    20
    .int    1024                                // width
    .int    768                                 // height
    .int    32                                  // depth

.align 8
.type tag_end STT_OBJECT
tag_end:
    .short  multiboot2_header_tag_end
    .short  0
    .int    8

multiboot2_header_end:

.section .text.entry

.section .text.entry32
.code32

.macro ENTRY32_COMMON_1
    // disable paging (UEFI may turn it on)
    mov     eax, cr0
    mov     ebx, (1 << 31)
    not     ebx
    and     eax, ebx
    mov     eax, cr0
    
    // load the temporary page table
    lea     eax, [.Ltmp_pml4 - {offset}]
    mov     cr3, eax

    // set PAE, PGE bit in CR4
    mov     eax, {cr4}
    mov     cr4, eax

    // set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    mov     edx, 0
    mov     eax, {efer}
    wrmsr

    // set protected mode, write protect, paging bit in CR0
    mov     eax, {cr0}
    mov     cr0, eax
.endm

.macro ENTRY32_COMMON_2
    // set data segment selectors
    mov     ax, 0x18
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax
.endm

.macro ENTRY64_COMMON
    // clear segment selectors
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax
.endm

bsp_entry32:
    ENTRY32_COMMON_1

    // set up GDT
    lgdt    [.Ltmp_gdt_desc_phys - {offset}]
    
    ENTRY32_COMMON_2

    // long return to the 64-bit entry
    push    0x10    // code64 segment selector
    lea     eax, [bsp_entry64 - {offset}]
    push    eax
    retf

.global ap_entry32
ap_entry32:
    ENTRY32_COMMON_1
    ENTRY32_COMMON_2

    // long return to the 64-bit entry
    push    0x10    // code64 segment selector
    lea     eax, [ap_entry64 - {offset}]
    push    eax
    retf

.section .text.entry64
.code64

bsp_entry64:
    // reload GDT by high address
    movabs  rax, offset .Ltmp_gdt_desc
    lgdt    [rax]

    // load task register
    mov     ax, 0x20
    ltr     ax
    
    ENTRY64_COMMON

    // set stack and jump to rust_entry
    movabs  rsp, offset bsp_boot_stack_top
    movabs  rax, offset {rust_entry}
    call    rax
    jmp     .Lhlt

ap_entry64:
    ENTRY64_COMMON
    // set rsp to high address
    mov     rax, {offset}
    add     rsp, rax

    // jump to rust_entry_secondary
    movabs  rax, offset {rust_entry_secondary}
    call    rax
    jmp     .Lhlt

.Lhlt:
    hlt
    jmp     .Lhlt

.section .rodata
.balign 8
.Ltmp_gdt_desc_phys:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   // limit
    .long   .Ltmp_gdt - {offset}            // base

.balign 8
.Ltmp_gdt_desc:
    .short  .Ltmp_gdt_end - .Ltmp_gdt - 1   // limit
    .quad   .Ltmp_gdt                       // base

.section .data
.balign 16
.Ltmp_gdt:
    .quad 0x0000000000000000    // 0x00: null
    .quad 0x00cf9b000000ffff    // 0x08: code segment (base=0, limit=0xfffff, type=32bit code exec/read, DPL=0, 4k)
    .quad 0x00af9b000000ffff    // 0x10: code segment (base=0, limit=0xfffff, type=64bit code exec/read, DPL=0, 4k)
    .quad 0x00cf93000000ffff    // 0x18: data segment (base=0, limit=0xfffff, type=32bit data read/write, DPL=0, 4k)
    .quad 0x00008934ee800067    // 0x20: tss low
    .quad 0x00000000ffffff80    // 0x28: tss high
.Ltmp_gdt_end:

.balign 4096
.Ltmp_pml4:
    // 0x0000_0000 ~ 0x4000_0000
    .quad Ltmp_pdpt_low - {offset} + 0x3    // PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 510
    // 0xffff_ff80_0000_0000 ~ 0xffff_ff80_4000_0000
    .quad Ltmp_pdpt_high - {offset} + 0x3   // PRESENT | WRITABLE | paddr(tmp_pdpt)

.global Ltmp_pdpt_low
Ltmp_pdpt_low:
    .quad 0x00000000 | 0x83                 // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 511

.global Ltmp_pdpt_high
Ltmp_pdpt_high:
    .quad 0x00000000 | 0x83                 // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 511
