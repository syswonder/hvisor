
/// This file defines and implements the functional functions of physical gicv2.
/// author: ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf

use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICV2_SGIS_NUM;
use crate::device::irqchip::gicv2::gich::{GICH, GICV2_GICH_LR_CPUID_SHIFT, GICV2_GICH_LR_HW, GICV2_GICH_LR_PENDING_STATE, GICV2_GICH_LR_PHYSID_SHIFT};
use crate::hypercall::SGI_IPI_ID;
use crate::event::check_events;

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


pub fn inject_irq(irq_id: usize, is_sgi: bool) {
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
            return;
        }
    }
    if free_lr == -1 {
        for i in 0..lr_num {
            let lr = GICH.get_lr(i as usize) as usize;
            warn!("lr[{}]: {:#x}", i, lr);
        }
        warn!("elrsr: {:#x}", elrsr);
        panic!("no free lr");
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
    }
}





