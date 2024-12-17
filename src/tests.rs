/// this module is for unittests of hvisor
/// since this is a baremetal program
/// all unittests are performed when running hvisor on qemu
/// you will need to use `make test` to run the unittests
use core::ptr::write_volatile;
use qemu_exit::QEMUExit;

#[test_case]
fn simple_test() {
    assert_eq!(1, 1);
}

// base trait for hvisor unittests
pub trait HvUnitTest {
    fn run(&self);
}

impl<T> HvUnitTest for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("    unittest: {:?}...", core::any::type_name::<T>());
        self();
        println!("[OK]");
    }
}

pub enum HvUnitTestResult {
    Passed,
    Failed,
}

pub fn quit_qemu(result: HvUnitTestResult) {
    #[cfg(target_arch = "aarch64")]
    let qemu_exit_handle = qemu_exit::AArch64::new();
    match result {
        HvUnitTestResult::Passed => qemu_exit_handle.exit_success(),
        HvUnitTestResult::Failed => qemu_exit_handle.exit_failure(),
    }
}

#[cfg(test)]
#[no_mangle]
pub fn test_main(tests: &[&dyn HvUnitTest]) {
    info!("Running {} unit tests", tests.len());
    for test in tests {
        test.run();
    }
    info!("All tests passed without panic! [ALL OK]");
    quit_qemu(HvUnitTestResult::Passed);
    loop {}
}
