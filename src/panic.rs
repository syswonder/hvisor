use core::panic::PanicInfo;

#[panic_handler]

fn on_panic(info: &PanicInfo) -> ! {
    error!("panic occurred: {:#?}", info);
    loop {}
}
