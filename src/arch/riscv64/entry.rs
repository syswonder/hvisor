use core::arch::global_asm;

global_asm!(include_str!("arch_entry.S"));

extern "C" {
    pub fn arch_entry();
}

// global_asm!("
//     .section \".rootcfg\", \"a\"
//     .incbin \"imgs/config/qemu-riscv64.cell\"

//     // .section \".nrcfg1\", \"a\"
//     // .incbin \"imgs/config/qemu-arm64-linux-demo.cell\"
// ");
