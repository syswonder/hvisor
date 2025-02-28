use crate::arch::cpu::this_cpu_id;
use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICV2_SGIS_NUM;
use crate::device::irqchip::gicv2::gich::{
    GICH, GICV2_GICH_HCR_UIE, GICV2_GICH_LR_CPUID_SHIFT, GICV2_GICH_LR_HW,
    GICV2_GICH_LR_PENDING_STATE, GICV2_GICH_LR_PHYSID_SHIFT,
};
use crate::event::check_events;
use crate::hypercall::SGI_IPI_ID;
/// This file defines and implements the functional functions of physical gicv2.
/// author: ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::{Mutex, Once};

pub const MAX_CPU_NUM: usize = 8;
pub const MAINTENACE_INTERRUPT: u64 = 25;

pub fn gicv2_handle_irq() {
    if let Some(irq_id) = get_pending_irq() {
        if irq_id < 8 {
            deactivate_irq(irq_id);
            let mut ipi_handled = false;
            if irq_id == SGI_IPI_ID as _ {
                ipi_handled = check_events();
            }
            if !ipi_handled {
                inject_irq(irq_id, true);
            }
        } else if irq_id < GICV2_SGIS_NUM {
            deactivate_irq(irq_id);
        } else if irq_id == MAINTENACE_INTERRUPT as usize {
            handle_maintenace_interrupt();
        } else {
            deactivate_irq(irq_id);
            inject_irq(irq_id, false);
        }
    }
}

pub fn get_pending_irq() -> Option<usize> {
    let iar = GICC.get_iar() as usize;
    let irq = iar & 0x3ff;
    if irq >= 1023 {
        None
    } else {
        Some(irq)
    }
}

// deactivate irq: GIC doesn't care CPU ID.
pub fn deactivate_irq(irq_id: usize) {
    GICC.set_eoir(irq_id as u32);
    if irq_id < GICV2_SGIS_NUM {
        GICC.set_dir(irq_id as u32);
    }
}

pub fn change_underflow_maintenance(is_enable: bool) {
    trace!("enable_maintenace_interrupt, is_enable is {}", is_enable);
    let mut hcr = GICH.get_hcr();
    trace!("hcr is {}", hcr);
    if is_enable {
        hcr |= GICV2_GICH_HCR_UIE;
    } else {
        hcr &= !GICV2_GICH_HCR_UIE;
    }
    GICH.set_hcr(hcr);
}

fn handle_maintenace_interrupt() {
    info!("handle_maintenace_interrupt");
    while let Some((irq_id, is_sgi)) = PENDING_VIRQS.get().unwrap().fetch_irq() {
        let is_injected: bool = inject_irq(irq_id, is_sgi);
        if is_injected {
            info!("inject pending irq {:#x} in maintenace interrupt", irq_id);
        }
        if !is_injected {
            PENDING_VIRQS.get().unwrap().add_irq(irq_id, is_sgi);
            change_underflow_maintenance(true);
            return;
        }
    }
    change_underflow_maintenance(false);
}

pub fn inject_irq(irq_id: usize, is_sgi: bool) -> bool {
    let elrsr: u64 = (GICH.get_elrsr(1) as u64) << 32 | GICH.get_elrsr(0) as u64;
    let lr_num: isize = GICH.get_lr_num() as isize;
    let lr_pint_mask: usize = 0x3ff << 10;
    let mut free_lr: isize = -1;
    for i in 0..lr_num {
        if (1 << i) & elrsr > 0 {
            free_lr = i;
            continue;
        }
        let lr = GICH.get_lr(i as usize) as usize;
        let pint = (lr & lr_pint_mask) >> 10;
        if pint == irq_id {
            trace!("virtual irq {} enables again", irq_id);
            return true;
        }
    }
    if free_lr == -1 {
        warn!("no free lr");
        for i in 0..lr_num {
            let lr = GICH.get_lr(i as usize) as usize;
            warn!("lr[{}]: {:#x}", i, lr);
        }
        PENDING_VIRQS
            .get()
            .unwrap()
            .add_irq(irq_id, is_sgi)
            .unwrap();
        change_underflow_maintenance(true);
        false
    } else {
        /* inject gruop 0 irq */
        // config vint bit 0-9
        let mut val = irq_id;
        // config pending state bit 31
        val = val | GICV2_GICH_LR_PENDING_STATE;
        if is_sgi {
            // config cpu bit 10-12
            val |= 1 << GICV2_GICH_LR_CPUID_SHIFT;
        } else {
            // config pint bit 10-19
            val = val | (irq_id << GICV2_GICH_LR_PHYSID_SHIFT);
            // config hw bit 31
            val = val | GICV2_GICH_LR_HW;
        }
        GICH.set_lr(free_lr as usize, val as u32);
        true
    }
}

// virtual interrupts waiting to inject
pub static PENDING_VIRQS: Once<PendingIrqs> = Once::new();
pub struct PendingIrqs {
    inner: Vec<Mutex<VecDeque<(usize, bool)>>>,
}

impl PendingIrqs {
    pub fn new(max_cpus: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_cpus {
            let v = Mutex::new(VecDeque::new());
            vs.push(v)
        }
        Self { inner: vs }
    }

    fn add_irq(&self, irq_id: usize, is_sgi: bool) -> Option<()> {
        match self.inner.get(this_cpu_id()) {
            Some(pending_irqs) => {
                let mut irqs = pending_irqs.lock();
                irqs.push_back((irq_id, is_sgi));
                Some(())
            }
            _ => None,
        }
    }

    fn fetch_irq(&self) -> Option<(usize, bool)> {
        match self.inner.get(this_cpu_id()) {
            Some(pending_irqs) => {
                let mut irqs = pending_irqs.lock();
                irqs.pop_front()
            }
            _ => None,
        }
    }
}
