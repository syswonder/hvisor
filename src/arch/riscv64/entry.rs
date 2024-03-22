use core::arch::global_asm;
use crate::consts::PER_CPU_SIZE;

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn arch_entry() -> i32 {
    //a0=cpuid,a1=dtb addr
    core::arch::asm!(
        "
        la t0, __core_end                // t0 = core_end
        li t1, {per_cpu_size}            // t1 = per_cpu_size
        mul t2, a0, t1                   // t2 = cpuid * per_cpu_size
        add t2, t1, t2                   // t2 = cpuid * per_cpu_size+per_cpu_size
        add sp, t0, t2                   // sp = core_end + cpuid * per_cpu_size + per_cpu_size
        call {rust_main}
        ",
        rust_main = sym crate::rust_main,
        per_cpu_size=const PER_CPU_SIZE,
        options(noreturn),
    );
}

// global_asm!("
//     .section \".rootcfg\", \"a\"
//     .incbin \"imgs/config/qemu-riscv64.zone\"

//     // .section \".nrcfg1\", \"a\"
//     // .incbin \"imgs/config/qemu-arm64-linux-demo.zone\"
// ");
