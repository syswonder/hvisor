use crate::consts::PER_CPU_SIZE;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    // a0/r4: CPU_ID read from CSR 0x20 CPUID
    core::arch::asm!(
        "
            addi.d      $r4, $zero, 0x21
            bl          print_char
            csrrd       $r4, {CSR_CPUID}
            la.pcrel    $r12, __core_end
            li.d        $r13, {per_cpu_size}
            mul.d       $r14, $r4, $r13
            add.d       $r14, $r13, $r14
            add.d       $r15, $r12, $r14    // this is the stack top of CPU[CPU_ID]
            addi.d      $sp, $r15, 0        // set sp
            la.pcrel    $r12, sbss          // bss start, t0 as ptr
            la.pcrel    $r13, ebss          // bss end
        1:
            st.d        $zero, $r12, 0      // MEM[ptr] = 0
            beq         $r12, $r13, 2f      // break if ptr == ebss, and we just set MEM[ebss] = 0
            addi.d      $r12, $r12, 8       // ptr += 8
            b           1b
        2:
            bl          {rust_main}
        ",
        rust_main = sym crate::rust_main,
        per_cpu_size = const PER_CPU_SIZE,
        CSR_CPUID = const 0x20,
        options(noreturn),
    );
}
