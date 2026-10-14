/// Architecture-private vCPU state placeholder.
pub struct ArchVcpu;

impl ArchVcpu {
    pub const fn new() -> Self {
        Self
    }

    pub fn clear_irq_state(&self) {}
}
