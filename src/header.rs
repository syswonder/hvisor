use core::fmt::{Debug, Formatter, Result};

use crate::consts::{HV_HEADER_PTR, PER_CPU_SIZE};

const HEADER_SIGNATURE: [u8; 8] = *b"HVISORIM";

#[repr(C)]
pub struct HvHeader {
    pub signature: [u8; 8],
    pub core_size: usize,
    pub percpu_size: usize,
    pub entry: usize,
    pub console_page: usize,
    pub gcov_info_head: usize,
    pub max_cpus: u32,
    pub online_cpus: u32,
    pub debug_console_base: usize,
    pub arm_linux_hyp_vectors: u64,
    pub arm_linux_hyp_abi: u32,
}

impl HvHeader {
    pub fn get<'a>() -> &'a Self {
        unsafe { &*HV_HEADER_PTR }
    }
}

#[repr(C)]
pub struct HvHeaderStuff {
    signature: [u8; 8],
    core_size: unsafe extern "C" fn(),
    percpu_size: usize,
    entry: unsafe extern "C" fn(),
    console_page: usize,
    gcov_info_head: usize,
    max_cpus: u32,
    online_cpus: u32,
    debug_console_base: usize,
    pub arm_linux_hyp_vectors: u64,
    arm_linux_hyp_abi: u32,
}

extern "C" {
    fn __entry_offset();
    fn __core_size();
    fn __header_start();
    fn __core_end();

}

#[used]
#[link_section = ".header"]
pub static mut HEADER_STUFF: HvHeaderStuff = HvHeaderStuff {
    signature: HEADER_SIGNATURE,
    core_size: __core_size,
    percpu_size: PER_CPU_SIZE,
    entry: __entry_offset,
    console_page: 0,
    gcov_info_head: 0,
    max_cpus: 0,
    online_cpus: 0,
    debug_console_base: 0,
    arm_linux_hyp_vectors: 0,
    arm_linux_hyp_abi: 0,
};
impl Debug for HvHeader {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("HvHeader")
            .field("signature", &core::str::from_utf8(&self.signature))
            .field("core_size", &self.core_size)
            .field("percpu_size", &self.percpu_size)
            .field("entry", &self.entry)
            .field("max_cpus", &self.max_cpus)
            .field("online_cpus", &self.online_cpus)
            .finish()
    }
}
