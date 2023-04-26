#![no_std] // 禁用标准库链接
#![no_main]
// 不使用main入口，使用自己定义实际入口_start，因为我们还没有初始化堆栈指针
#![feature(naked_functions)] //  surpport naked function
use core::arch::global_asm; // 支持内联汇编
use core::result::Result;
#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;
mod header;
mod panic;

fn main() -> Result<(), ()> {
    Ok(())
}

extern "C" fn entry() -> () {
    if let Err(_e) = main() {}
}
