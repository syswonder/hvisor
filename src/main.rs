#![no_std] // 禁用标准库链接
#![no_main] // 不使用main入口，使用自己定义实际入口_start，因为我们还没有出事后堆栈指针 

use core::arch::global_asm; // 支持内联汇编

mod panic;
mod driver;
mod lib;
global_asm!(include_str!("arch/aarch64/start.s")); // 内联汇编

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::lib::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[no_mangle]
pub extern "C" fn init(cpu_id: usize) {
    println!("Welcome AArch64 Bare Metal Hypervisor\n");
    boot_hypervisor(cpu_id);
}

pub fn boot_hypervisor(cpu_id: usize) {
    println!("Hello Hypervisor...\n");
    /* 原始方案：(deprecated)
     * 1. 配置相关寄存器；
     * 2. 配置页表信息；
     * 3. 其他配置；
     * 4. vcpu_init;
     * 5. ram_init;
     * 6. irq_init;
     * 7. load_image;
     * 8. vcpu_run;
     */

    /*
     * 1. 检查是否是core_0
     * 2.
     */
    // printk_uart0(usize);
    println!("cpu_id: {}", cpu_id);
    if cpu_id == 0 {
        println!( "Welcome to RVM hypervisor...\n");
        // heap::init();
        // mem_init();
    }
    loop {}
}