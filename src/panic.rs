#[cfg(test)]
use crate::tests::*;

use core::panic::PanicInfo;

#[panic_handler]

fn on_panic(info: &PanicInfo) -> ! {
    error!("panic occurred: {:#?}", info);
    #[cfg(test)]
    {
        error!("panic occurred when running cargo test, quitting qemu");
        crate::tests::quit_qemu(HvUnitTestResult::Failed);
    }
    loop {}
}
