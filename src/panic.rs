use crate::tests::*;
use core::panic::PanicInfo;

#[panic_handler]

fn on_panic(info: &PanicInfo) -> ! {
    error!("panic occurred: {:#?}", info);
    if cfg!(test) {
        error!("panic occurred when running cargo test, quitting qemu");
        #[cfg(test)]
        crate::tests::quit_qemu(HvUnitTestResult::Failed);
    }
    loop {}
}
