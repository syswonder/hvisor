#![no_std] // 禁用标准库链接
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)] //  surpport naked function
#![feature(default_alloc_error_handler)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
mod logging;
mod device;
mod entry;
mod panic;
#[no_mangle]
extern "C" fn start() {
    logging::init();
    info!("Logging is enabled.");
    main();
}
fn main() {
    info!("Hello, world!");
    loop {}
}
