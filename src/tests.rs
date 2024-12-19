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
        // Get the test name
        let test_name = core::any::type_name::<T>();

        // Print a clean start message with a header and test name
        print!("\n--- Running test: {} ---\n", test_name);

        // Execute the test function
        self();

        // Print a success message after the test
        println!("Result: PASSED");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HvUnitTestResult {
    Success,
    Failed,
}

pub fn quit_qemu(result: HvUnitTestResult) {
    warn!("quitting qemu, result: {:?}", result);
    #[cfg(target_arch = "aarch64")]
    let qemu_exit_handle = qemu_exit::AArch64::new();
    match result {
        HvUnitTestResult::Success => qemu_exit_handle.exit_success(),
        HvUnitTestResult::Failed => qemu_exit_handle.exit_failure(),
    }
}

#[cfg(test)]
#[no_mangle]
pub fn test_main(tests: &[&dyn HvUnitTest]) {
    info!("Running {} unit tests", tests.len());
    println!("\nTotal {} tests to run", tests.len());
    for test in tests {
        test.run();
    }
    println!("\nAll tests passed without panic which is good");
    quit_qemu(HvUnitTestResult::Success);
    loop {}
}
