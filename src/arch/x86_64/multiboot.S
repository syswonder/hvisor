.equ boot_stack_top, __core_end + {per_cpu_size}

.section .text.entry

.section .text.entry32
.code32

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

.macro ENTRY32_COMMON
    // set data segment selectors
    mov     ax, 0x18
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    // set PAE, PGE bit in CR4
    mov     eax, {cr4}
    mov     cr4, eax
    
    // load the temporary page table
    lea     eax, [.Ltmp_pml4 - {offset}]
    mov     cr3, eax

    // set LME, NXE bit in IA32_EFER
    mov     ecx, {efer_msr}
    mov     edx, 0
    mov     eax, {efer}
    wrmsr

    // set protected mode, write protect, paging bit in CR0
    mov     eax, {cr0}
    mov     cr0, eax
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
    // load the temporary GDT
    lgdt    [.Ltmp_gdt_desc_phys - {offset}]
    ENTRY32_COMMON

    // long return to the 64-bit entry
    push    0x10    // code64 segment selector
    lea     eax, [bsp_entry64 - {offset}]
    push    eax
    retf

.global ap_entry32
ap_entry32:
    ENTRY32_COMMON

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
    mov     ax, 0x20
    ltr     ax

    ENTRY64_COMMON

    // set stack and jump to rust_entry
    movabs  rsp, offset boot_stack_top
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
    // 0x0000_0000 ~ 0x1_0000_0000
    .quad .Ltmp_pdpt_low - {offset} + 0x3   // PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 510
    // 0xffff_ff80_0000_0000 ~ 0xffff_ff80_8000_0000
    .quad .Ltmp_pdpt_high - {offset} + 0x3  // PRESENT | WRITABLE | paddr(tmp_pdpt)

.Ltmp_pdpt_low:
    .quad 0x0000 | 0x83         // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .quad 0x40000000 | 0x83     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .quad 0x80000000 | 0x83     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .quad 0xc0000000 | 0x83     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .zero 8 * 508

.Ltmp_pdpt_high:
    .quad 0x0000 | 0x83         // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .quad 0x40000000 | 0x83     // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x4000_0000)
    .zero 8 * 510
