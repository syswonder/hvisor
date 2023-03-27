use core::ptr;

const UART0: *mut u8 = 0x0900_0000 as *mut u8; // QEMU Virt定义的UART0地址为0x09000000，是UART0外设的内存映射地址，即访问该地址就是访问该外设。

pub fn putc(byte: u8) {
    unsafe {
        ptr::write_volatile(UART0, byte); 
    }
}
