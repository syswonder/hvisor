.equ MULTIBOOT_HEADER_MAGIC, 0x1BADB002
.equ MULTIBOOT_HEADER_FLAGS, 0x00010002
.equ MULTIBOOT_CHECKSUM, -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)

.section .text.entry

.section .text.entry32
.code32

.balign 4
.type multiboot_header, STT_OBJECT
multiboot_header:
    .int    MULTIBOOT_HEADER_MAGIC
    .int    MULTIBOOT_HEADER_FLAGS
    .int    MULTIBOOT_CHECKSUM
    .int    multiboot_header - {offset}        // header_addr
    .int    skernel - {offset}                 // load_addr
    .int    edata - {offset}                   // load_end
    .int    ebss - {offset}                    // bss_end_addr
    .int    arch_entry - {offset}              // entry_addrs

entry32:
    // load the temporary GDT
    lgdt    [.Ltmp_gdt_desc_phys - {offset}]
    mov     ax, 0x18    // data segment selector
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

    // long return to the 64-bit entry
    push    0x10    // code64 segment selector
    lea     eax, [entry64 - {offset}]
    push    eax
    retf

.section .text.entry64
.code64

entry64:
    // reload GDT by high address
    movabs  rax, offset .Ltmp_gdt_desc
    lgdt    [rax]

    // clear segment selectors
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    // set stack and jump to rust_main
    movabs  rsp, offset boot_stack_top
    movabs  rax, offset {main_entry}
    call    rax
1:  jmp     1b

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
.Ltmp_gdt_end:

.balign 4096
.Ltmp_pml4:
    // 0x0000_0000 ~ 0x8000_0000
    .quad .Ltmp_pdpt_low - {offset} + 0x3   // PRESENT | WRITABLE | paddr(tmp_pdpt)
    .zero 8 * 510
    // 0xffff_ff80_0000_0000 ~ 0xffff_ff80_8000_0000
    .quad .Ltmp_pdpt_high - {offset} + 0x3  // PRESENT | WRITABLE | paddr(tmp_pdpt)

.Ltmp_pdpt_low:
    .quad 0x0000 | 0x83                 // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .quad 0x40000000 | 0x83             // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 510

.Ltmp_pdpt_high:
    .quad 0x0000 | 0x83                 // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .quad 0x40000000 | 0x83             // PRESENT | WRITABLE | HUGE_PAGE | paddr(0x0)
    .zero 8 * 510

.section .bss.stack
.balign 4096
boot_stack:
    .space {boot_stack_size}
boot_stack_top:
