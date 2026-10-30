// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
// GICv3 List register management: virtual IRQ injection, guest SGI clearing,
// and the maintenance interrupt.
use super::gich::{list_regs_num, read_lr, write_lr, IchLr, ICH_HCR_UIE};
use super::is_sgi;
use super::pending::PENDING_VIRQS;
use crate::arch::aarch64::sysreg::{read_sysreg, write_sysreg};
use crate::arch::cpu::this_cpu_id;
use crate::cpu_data::this_cpu_data;
use crate::device::irqchip::SGIS_NUM;
use bit_field::BitField;

pub const MAINTENACE_INTERRUPT: u64 = 25;

pub fn flush_current_guest_irqs() {
    let cpu = this_cpu_data();
    if cpu.zone.is_none() || cpu.vcpu.state.is_stopped() {
        return;
    }
    let state = &cpu.vcpu.irq;
    crate::device::irqchip::flush_guest_irq_state(state, install_pending_guest_irq, || {
        enable_maintenace_interrupt(true)
    });
}

fn install_pending_guest_irq(irq_id: usize) -> bool {
    if irq_id >= SGIS_NUM {
        return false;
    }
    try_install_software_sgi(irq_id)
}

fn try_install_software_sgi(irq_id: usize) -> bool {
    let elsr = read_sysreg!(ich_elrsr_el2);
    let mut free_lr = None;
    for index in 0..list_regs_num() {
        if elsr.get_bit(index) {
            free_lr.get_or_insert(index);
            continue;
        }
        let mut lr = IchLr::read(index);
        if lr.vintid() != irq_id {
            continue;
        }
        // Active-only LRs become pending; pending ones coalesce.
        if !lr.is_pending() {
            lr.set_pending(true);
            lr.write(index);
        }
        return true;
    }
    let Some(index) = free_lr else {
        return false;
    };
    IchLr::new_pending(irq_id).write(index);
    true
}

/// Inject virtual interrupt to vCPU, return whether it not needs to add pending queue.
pub fn inject_irq(irq_id: usize, is_hardware: bool) -> bool {
    // mask
    const LR_VIRTIRQ_MASK: usize = (1 << 32) - 1;

    let elsr: u64 = read_sysreg!(ich_elrsr_el2);
    let lr_num = list_regs_num();
    let mut free_ir = -1 as isize;
    for i in 0..lr_num {
        // find a free list register
        if (1 << i) & elsr > 0 {
            if free_ir == -1 {
                free_ir = i as isize;
            }
            continue;
        }
        let lr_val = read_lr(i) as usize;
        // if a virtual interrupt is enabled and equals to the physical interrupt irq_id
        if (lr_val & LR_VIRTIRQ_MASK) == irq_id {
            // A software IRQ the guest has acknowledged but not yet
            // deactivated would otherwise lose this injection.
            let mut lr = IchLr(lr_val as u64);
            if !is_hardware && lr.is_active() && !lr.is_pending() {
                lr.set_pending(true);
                lr.write(i);
            }
            trace!("virtual irq {} enables again", irq_id);
            return true;
        }
    }
    trace!("To Inject IRQ {}, find lr {}", irq_id, free_ir);

    if free_ir == -1 {
        trace!("all list registers are valid, add to pending queue");
        // If all list registers are valid, add this virtual irq to pending queue,
        // and enable an underflow maintenace interrupt. When list registers are
        // all invalid or only one is valid, the maintenace interrupt will occur,
        // hvisor will execute handle_maintenace_interrupt function.
        PENDING_VIRQS
            .get()
            .unwrap()
            .add_irq(irq_id, is_hardware)
            .unwrap();
        enable_maintenace_interrupt(true);
        return false;
    } else {
        let mut val = irq_id as u64; //v intid
        val |= 1 << 60; //group 1
        val |= 1 << 62; //state pending

        if !is_sgi(irq_id as _) && is_hardware {
            val |= 1 << 61; //map hardware
            val |= (irq_id as u64) << 32; //pINTID
        }
        write_lr(free_ir as usize, val);
        return true;
    }
}

pub fn clear_guest_sgi(target_cpu: usize, irq_id: usize) -> bool {
    if irq_id >= SGIS_NUM {
        return false;
    }
    let target = crate::cpu_data::get_cpu_data(target_cpu);
    let control = target.ctrl_lock.lock();
    if !target.vcpu.irq.clear(irq_id) {
        return false;
    }
    drop(control);
    if target_cpu == this_cpu_id() {
        clear_lr_pending_irq(irq_id);
    }
    true
}

pub(crate) fn clear_lr_pending_irq(irq_id: usize) {
    let elsr = read_sysreg!(ich_elrsr_el2);
    for index in 0..list_regs_num() {
        if elsr.get_bit(index) {
            continue;
        }
        let mut lr = IchLr::read(index);
        if lr.vintid() != irq_id || !lr.is_pending() {
            continue;
        }
        if lr.is_active() {
            lr.set_pending(false);
            lr.write(index);
        } else {
            write_lr(index, 0);
        }
    }
}

pub(crate) fn lr_pending_sgi_mask() -> u32 {
    let elsr = read_sysreg!(ich_elrsr_el2);
    let mut pending = 0u32;
    for index in 0..list_regs_num() {
        if elsr.get_bit(index) {
            continue;
        }
        let lr = IchLr::read(index);
        let irq_id = lr.vintid();
        if irq_id < SGIS_NUM && lr.is_pending() {
            pending.set_bit(irq_id, true);
        }
    }
    pending
}

pub(super) fn handle_maintenace_interrupt() {
    trace!("handle_maintenace_interrupt");
    flush_current_guest_irqs();
    let pending_irqs = PENDING_VIRQS.get().unwrap();
    while let Some((irq_id, is_hardware)) = pending_irqs.fetch_irq() {
        let is_injected: bool = inject_irq(irq_id, is_hardware);
        if is_injected {
            trace!("inject pending irq in maintenace interrupt");
        }
        if !is_injected {
            pending_irqs.add_irq(irq_id, is_hardware);
            enable_maintenace_interrupt(true);
            return;
        }
    }
    enable_maintenace_interrupt(has_pending_maintenance_work());
}

fn has_pending_maintenance_work() -> bool {
    this_cpu_data().vcpu.irq.has_pending() || PENDING_VIRQS.get().unwrap().has_pending()
}

// Enable or disable an underflow maintenace interrupt.
fn enable_maintenace_interrupt(is_enable: bool) {
    trace!("enable_maintenace_interrupt, is_enable is {}", is_enable);
    let mut hcr = read_sysreg!(ich_hcr_el2);
    trace!("hcr is {}", hcr);
    if is_enable {
        hcr |= ICH_HCR_UIE;
    } else {
        hcr &= !ICH_HCR_UIE;
    }
    write_sysreg!(ich_hcr_el2, hcr);
}
