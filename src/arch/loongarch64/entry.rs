use crate::consts::PER_CPU_SIZE;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    // a0/r4: CPU_ID read from CSR 0x20 CPUID
    core::arch::asm!(
        "
        csrrd   $r4, {CSR_CPUID}
        li.d    $r12, __core_end
        li.d    $r13, {per_cpu_size}
        mul.d   $r14, $r4, $r13
        add.d   $r14, $r13, $r14
        add.d   $r15, $r12, $r14    // this is the stack top of CPU[CPU_ID]
        addi.d  $sp, $r15, 0        // set sp
        bl {rust_main}
        ",
        rust_main = sym crate::rust_main,
        per_cpu_size = const PER_CPU_SIZE,
        CSR_CPUID = const 0x20,
        options(noreturn),
    );
}
