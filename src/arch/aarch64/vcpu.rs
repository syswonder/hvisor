use crate::error::HvResult;

#[repr(C)]
pub struct Vcpu {
    /// RSP will be loaded from here when handle VM exits.
    host_stack_top: u64,
}
impl Vcpu {
    pub fn new() -> HvResult<Self> {
        let mut ret = Self {
            host_stack_top: PerCpu::current().stack_top() as _,
        };
        Ok(ret)
    }
    pub fn enter(&mut self) -> HvResult {}
}
