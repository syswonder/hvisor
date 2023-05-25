use crate::error::HvResult;
use crate::percpu::GeneralRegisters;
#[repr(C)]
pub struct Vcpu {
    /// RSP will be loaded from here when handle VM exits.
    regs: GeneralRegisters,
}
impl Vcpu {
    pub fn new() -> HvResult<Self> {
        let mut ret = Self {
            regs: GeneralRegisters,
        };
        Ok(ret)
    }
    pub fn enter(&mut self) -> HvResult {}
}
