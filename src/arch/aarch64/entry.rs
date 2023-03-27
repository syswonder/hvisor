use core::arch::global_asm; // 支持内联汇编
use crate::percpu::PerCpu;



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
pub unsafe  extern "C" fn arch_entry(linux_sp: usize) -> i32 {
    println!("Welcome AArch64 Bare Metal Hypervisor\n");
    let cpu_data = match PerCpu::new() {
        OK(c) => c,
        Err(e) => return e.code(),
    };
    let hv_sp = cpu_data.stack_top();
    core::arch::asm!("
        bl {entry}",
        entry = sym crate::entry,
        in("x0") cpu_data,
        in("x1") linux_sp,
    );
    0

}

