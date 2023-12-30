use core::fmt::{Debug, Formatter, Result};

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

extern "C" {
    fn __entry_offset();
    fn __core_size();
    fn __core_end();
}

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
