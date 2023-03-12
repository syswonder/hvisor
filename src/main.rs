#![no_std] // 禁用标准库链接
#![no_main] // 不使用main入口，使用自己定义实际入口_start，因为我们还没有出事后堆栈指针 
use core::ptr;
use core::arch::global_asm; // 支持内联汇编

mod panic;

global_asm!(include_str!("start.s")); // 内联汇编

#[no_mangle] // 关闭Rust的名称修改功能，让rust在编译时不修改我们定义的函数名，这样在start.s就可以跳入这里
pub extern "C" fn not_main() { // extern "C" 使用C语言的调用约定，即ABI，如参数放置的寄存器约定，返回值的寄存器约定，详见：https://en.wikipedia.org/wiki/Calling_convention 。然后我们就可以在Rust之外调用该函数。
    const UART0: *mut u8 = 0x0900_0000 as *mut u8; // QEMU Virt定义的UART0地址为0x09000000，是UART0外设的内存映射地址，即访问该地址就是访问该外设。
    let out_str = b"AArch64 Bare Metal\n";
    for byte in out_str {
        unsafe {
            ptr::write_volatile(UART0, *byte);
        }
    }
    boot_hypervisor();

}
#[no_mangle]
pub extern "C" fn el3_entry() -> u8 {
    printk_uart0("This is el3_entry......\n")
}
#[no_mangle]
pub extern "C" fn el2_entry() -> u8 {
    printk_uart0("This is el2_entry......\n")
}

fn printk_uart0(str: &str) -> u8 {
    const UART0: *mut u8 = 0x0900_0000 as *mut u8;
    for byte in str.bytes() {
        unsafe {
            ptr::write_volatile(UART0, byte);
        }
    }
    return 0;
}

pub fn boot_hypervisor() -> u8 {
    printk_uart0("Hello Hypervisor...\n");
    /*
     * 1. 配置相关寄存器；
     * 2. 配置页表信息；
     * 3. 其他配置；
     * 4. vcpu_init;
     * 5. ram_init;
     * 6. irq_init;
     * 7. load_image;
     * 8. vcpu_run;
     */
    return 0;
}