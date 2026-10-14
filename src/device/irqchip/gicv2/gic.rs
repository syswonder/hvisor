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
//      Hangqi Ren <2572131118@qq.com>
use crate::arch::cpu::this_cpu_id;
use crate::cpu_data::this_cpu_data;
use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICV2_SGIS_NUM;
use crate::device::irqchip::gicv2::gich::{
    GICH, GICV2_GICH_HCR_UIE, GICV2_GICH_LR_ACTIVE_STATE, GICV2_GICH_LR_CPUID_SHIFT,
    GICV2_GICH_LR_HW, GICV2_GICH_LR_PENDING_STATE, GICV2_GICH_LR_PHYSID_SHIFT,
};
use crate::event::{check_events, dispatch_host_events};
use crate::hypercall::SGI_IPI_ID;
/// This file defines and implements the functional functions of physical gicv2.
/// author: ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::{Mutex, Once};

// gicv2 support max 8 cpus
pub(crate) const MAX_CPU_NUM: usize = 8;
pub const MAINTENACE_INTERRUPT: u64 = 25;

pub fn gicv2_handle_irq() {
    if let Some(irq_id) = get_pending_irq() {
        if irq_id < GICV2_SGIS_NUM {
            deactivate_irq(irq_id);
            if irq_id == SGI_IPI_ID as _ {
                dispatch_host_events(|| while check_events() {});
            } else {
                warn!("unexpected physical SGI {}", irq_id);
            }
        } else if irq_id == MAINTENACE_INTERRUPT as usize {
            handle_maintenace_interrupt();
        } else {
            deactivate_irq(irq_id);
            inject_irq(irq_id, false);
        }
    }
}

pub fn get_pending_irq() -> Option<usize> {
    let iar = GICC.get().unwrap().get_iar() as usize;
    let irq = iar & 0x3ff;
    if irq >= 1023 {
        None
    } else {
        Some(irq)
    }
}

// deactivate irq: GIC doesn't care CPU ID.
pub fn deactivate_irq(irq_id: usize) {
    GICC.get().unwrap().set_eoir(irq_id as u32);
    if irq_id < GICV2_SGIS_NUM {
        GICC.get().unwrap().set_dir(irq_id as u32);
    }
}

pub fn change_underflow_maintenance(is_enable: bool) {
    trace!("enable_maintenace_interrupt, is_enable is {}", is_enable);
    let mut hcr = GICH.get().unwrap().get_hcr();
    trace!("hcr is {}", hcr);
    if is_enable {
        hcr |= GICV2_GICH_HCR_UIE;
    } else {
        hcr &= !GICV2_GICH_HCR_UIE;
    }
    GICH.get().unwrap().set_hcr(hcr);
}

fn handle_maintenace_interrupt() {
    info!("handle_maintenace_interrupt");
    flush_current_guest_irqs();
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
    change_underflow_maintenance(has_pending_maintenance_work());
}

fn has_pending_maintenance_work() -> bool {
    this_cpu_data().vcpu.irq.has_pending() || PENDING_VIRQS.get().unwrap().has_pending()
}

const VINTID_MASK: usize = 0x3ff;

fn try_install_software_sgi(irq_id: usize, source_cpu: usize) -> bool {
    let gich = GICH.get().unwrap();
    let elrsr = (gich.get_elrsr(1) as u64) << 32 | gich.get_elrsr(0) as u64;
    let lr_num = gich.get_lrs_num() as usize;
    let mut free_lr = None;
    for index in 0..lr_num {
        if elrsr & (1u64 << index) != 0 {
            free_lr.get_or_insert(index);
            continue;
        }
        let lr = gich.get_lr(index) as usize;
        let lr_source = (lr >> GICV2_GICH_LR_CPUID_SHIFT) & 0x7;
        if lr & VINTID_MASK != irq_id || lr_source != source_cpu {
            continue;
        }
        if lr & GICV2_GICH_LR_PENDING_STATE != 0 {
            return true;
        }
        gich.set_lr(
            index,
            (lr | GICV2_GICH_LR_PENDING_STATE | (source_cpu << GICV2_GICH_LR_CPUID_SHIFT)) as u32,
        );
        return true;
    }
    let Some(index) = free_lr else {
        return false;
    };
    gich.set_lr(
        index,
        (irq_id | GICV2_GICH_LR_PENDING_STATE | (source_cpu << GICV2_GICH_LR_CPUID_SHIFT)) as u32,
    );
    true
}

fn install_pending_guest_irq(cpu: &crate::cpu_data::PerCpu, irq_id: usize) -> bool {
    if irq_id >= GICV2_SGIS_NUM {
        return false;
    }
    let source_mask = cpu.vcpu.arch.sgi_source[irq_id].load(core::sync::atomic::Ordering::Acquire);
    let source_cpu = source_mask.trailing_zeros() as usize;
    if source_cpu >= 8 {
        return false;
    }
    cpu.vcpu.arch.sgi_source[irq_id]
        .fetch_and(!(1u8 << source_cpu), core::sync::atomic::Ordering::AcqRel);
    if !try_install_software_sgi(irq_id, source_cpu) {
        cpu.vcpu.arch.sgi_source[irq_id]
            .fetch_or(1u8 << source_cpu, core::sync::atomic::Ordering::AcqRel);
        return false;
    }
    if cpu.vcpu.arch.sgi_source[irq_id].load(core::sync::atomic::Ordering::Acquire) != 0 {
        cpu.vcpu.irq.set(irq_id);
    }
    true
}

pub fn flush_current_guest_irqs() {
    let cpu = this_cpu_data();
    if cpu.zone.is_none() || cpu.vcpu_state.is_stopped() {
        return;
    }
    let state = &cpu.vcpu.irq;
    crate::device::irqchip::flush_guest_irq_state(
        state,
        |irq_id| install_pending_guest_irq(cpu, irq_id),
        || change_underflow_maintenance(true),
    );
}

pub fn clear_guest_sgi_pending(target_cpu: usize, irq_id: usize) -> bool {
    clear_guest_sgi_pending_sources(target_cpu, irq_id, u8::MAX)
}

pub fn clear_guest_sgi_pending_sources(target_cpu: usize, irq_id: usize, source_mask: u8) -> bool {
    if irq_id >= GICV2_SGIS_NUM {
        return false;
    }
    let target = crate::cpu_data::get_cpu_data(target_cpu);
    let control = target.ctrl_lock.lock();
    target.vcpu.arch.sgi_source[irq_id]
        .fetch_and(!source_mask, core::sync::atomic::Ordering::AcqRel);
    let pending_cleared =
        target.vcpu.arch.sgi_source[irq_id].load(core::sync::atomic::Ordering::Acquire) == 0
            && target.vcpu.irq.clear(irq_id);
    drop(control);
    if target_cpu == this_cpu_id() {
        clear_lr_pending_sgi(irq_id, source_mask);
    }
    pending_cleared
}

fn clear_lr_pending_sgi(irq_id: usize, source_mask: u8) {
    let gich = GICH.get().unwrap();
    let elrsr = (gich.get_elrsr(1) as u64) << 32 | gich.get_elrsr(0) as u64;
    for index in 0..gich.get_lrs_num() as usize {
        if elrsr & (1u64 << index) != 0 {
            continue;
        }
        let lr = gich.get_lr(index) as usize;
        let source = (lr >> GICV2_GICH_LR_CPUID_SHIFT) & 0x7;
        if lr & VINTID_MASK != irq_id
            || lr & GICV2_GICH_LR_PENDING_STATE == 0
            || source_mask & (1 << source) == 0
        {
            continue;
        }
        if lr & GICV2_GICH_LR_ACTIVE_STATE != 0 {
            gich.set_lr(index, (lr & !GICV2_GICH_LR_PENDING_STATE) as u32);
        } else {
            gich.set_lr(index, 0);
        }
    }
}

pub(crate) struct PendingSgiLrSummary {
    pub pending_mask: u32,
    pub source_masks: [u8; GICV2_SGIS_NUM],
}

pub(crate) fn read_pending_sgi_lr_summary() -> PendingSgiLrSummary {
    let gich = GICH.get().unwrap();
    let elrsr = (gich.get_elrsr(1) as u64) << 32 | gich.get_elrsr(0) as u64;
    let mut summary = PendingSgiLrSummary {
        pending_mask: 0,
        source_masks: [0; GICV2_SGIS_NUM],
    };
    for index in 0..gich.get_lrs_num() as usize {
        if elrsr & (1u64 << index) != 0 {
            continue;
        }
        let lr = gich.get_lr(index) as usize;
        let irq_id = lr & VINTID_MASK;
        if irq_id < GICV2_SGIS_NUM && lr & GICV2_GICH_LR_PENDING_STATE != 0 {
            summary.pending_mask |= 1 << irq_id;
            summary.source_masks[irq_id] |= 1 << ((lr >> GICV2_GICH_LR_CPUID_SHIFT) & 0x7);
        }
    }
    summary
}

pub fn read_lr_pending_sgi_source_mask(irq_id: usize) -> u8 {
    if irq_id >= GICV2_SGIS_NUM {
        return 0;
    }
    read_pending_sgi_lr_summary().source_masks[irq_id]
}

pub fn read_lr_pending_sgi_mask() -> u32 {
    read_pending_sgi_lr_summary().pending_mask
}

pub fn inject_irq(irq_id: usize, is_sgi: bool) -> bool {
    let elrsr: u64 =
        (GICH.get().unwrap().get_elrsr(1) as u64) << 32 | GICH.get().unwrap().get_elrsr(0) as u64;
    let lr_num: isize = GICH.get().unwrap().get_lr_num() as isize;
    let lr_pint_mask: usize = 0x3ff << 10;
    let mut free_lr: isize = -1;
    for i in 0..lr_num {
        if (1 << i) & elrsr > 0 {
            free_lr = i;
            continue;
        }
        let lr = GICH.get().unwrap().get_lr(i as usize) as usize;
        let pint = (lr & lr_pint_mask) >> 10;
        if pint == irq_id {
            trace!("virtual irq {} enables again", irq_id);
            return true;
        }
    }
    if free_lr == -1 {
        warn!("no free lr");
        for i in 0..lr_num {
            let lr = GICH.get().unwrap().get_lr(i as usize) as usize;
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
        // config pending state bit 28
        val = val | GICV2_GICH_LR_PENDING_STATE;
        if !is_sgi {
            // config pint bit 10-19
            val = val | (irq_id << GICV2_GICH_LR_PHYSID_SHIFT);
            // config hw bit 31
            val = val | GICV2_GICH_LR_HW;
        }
        GICH.get().unwrap().set_lr(free_lr as usize, val as u32);
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

    fn has_pending(&self) -> bool {
        self.inner
            .get(this_cpu_id())
            .map(|pending_irqs| !pending_irqs.lock().is_empty())
            .unwrap_or(false)
    }
}
