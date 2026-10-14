#[cfg(irq_gicv2)]
use core::sync::atomic::{AtomicU8, Ordering};

/// Architecture-private vCPU state, separate from the host pCPU's `ArchCpu`.
pub struct ArchVcpu {
    #[cfg(irq_gicv2)]
    pub sgi_source: [AtomicU8; 16],
}

impl ArchVcpu {
    pub const fn new() -> Self {
        #[cfg(irq_gicv2)]
        const ZERO_SOURCE: AtomicU8 = AtomicU8::new(0);
        Self {
            #[cfg(irq_gicv2)]
            sgi_source: [ZERO_SOURCE; 16],
        }
    }

    pub fn clear_irq_state(&self) {
        #[cfg(irq_gicv2)]
        for source in &self.sgi_source {
            source.store(0, Ordering::Release);
        }
    }
}
