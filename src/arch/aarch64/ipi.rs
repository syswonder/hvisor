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

use crate::device::irqchip::gic_send_event;

#[cfg(irq_gicv2)]
use crate::{
    arch::cpu::this_cpu_id,
    cpu_data::this_zone,
    device::irqchip::gicv2::gicd::{GICD_SGIR_ROUTING_SHIFT, GICD_SGIR_TARGET_LIST_FILTER_SHIFT},
};

#[cfg(irq_gicv3)]
use crate::{
    arch::cpu::{cpuid_to_mpidr_affinity, this_cpu_id},
    cpu_data::this_zone,
};

#[cfg(irq_gicv3)]
const ICC_SGI1R_TARGET_LIST_MASK: u64 = 0xffff;

#[cfg(irq_gicv3)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Sgi1r {
    target_list: u16,
    aff1: u8,
    intid: usize,
    aff2: u8,
    irm: bool,
    range_selector: u8,
    aff3: u8,
}

#[cfg(irq_gicv3)]
impl Sgi1r {
    fn decode(value: u64) -> Self {
        Self {
            target_list: (value & ICC_SGI1R_TARGET_LIST_MASK) as u16,
            aff1: ((value >> 16) & 0xff) as u8,
            intid: ((value >> 24) & 0xf) as usize,
            aff2: ((value >> 32) & 0xff) as u8,
            irm: value & (1 << 40) != 0,
            range_selector: ((value >> 44) & 0xf) as u8,
            aff3: ((value >> 48) & 0xff) as u8,
        }
    }

    fn targets(self, cpu_id: usize, source_cpu: usize) -> bool {
        if self.irm {
            return cpu_id != source_cpu;
        }
        let (aff3, aff2, aff1, aff0) = cpuid_to_mpidr_affinity(cpu_id as u64);
        let target_aff0 = self.range_selector as u64 * 16;
        aff3 == self.aff3 as u64
            && aff2 == self.aff2 as u64
            && aff1 == self.aff1 as u64
            && aff0 >= target_aff0
            && aff0 < target_aff0 + 16
            && self.target_list & (1 << (aff0 - target_aff0)) != 0
    }
}

pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    gic_send_event(cpu_id, sgi_num);
}

pub fn arch_notify_event(cpu_id: u64, sgi_num: u64, _event_id: usize, _queue_was_empty: bool) {
    arch_send_event(cpu_id, sgi_num);
}

#[cfg(irq_gicv3)]
pub fn handle_guest_sgi1r(value: u64) {
    let sgi = Sgi1r::decode(value);
    let source_cpu = this_cpu_id();
    for target_cpu in this_zone().cpu_set().iter() {
        if sgi.targets(target_cpu, source_cpu) {
            if !crate::device::irqchip::post_guest_irq(target_cpu, sgi.intid, None) {
                error!(
                    "failed to post guest SGI {} to CPU{}",
                    sgi.intid, target_cpu
                );
            }
        }
    }
}

#[cfg(irq_gicv2)]
pub fn handle_guest_sgi1r(value: u64) {
    let target_list = (value & 0xffff) as usize;
    let routing_mode = if value & (1 << 40) != 0 { 1 } else { 0 };
    let irq_id = ((value >> 24) & 0xf) as usize;
    handle_guest_sgir(
        irq_id as u32
            | ((target_list as u32) << GICD_SGIR_TARGET_LIST_FILTER_SHIFT)
            | ((routing_mode as u32) << GICD_SGIR_ROUTING_SHIFT),
    );
}

#[cfg(irq_gicv2)]
pub fn handle_guest_sgir(value: u32) {
    let source_cpu = this_cpu_id();
    let irq_id = (value & 0xf) as usize;
    let target_list = ((value >> GICD_SGIR_TARGET_LIST_FILTER_SHIFT) & 0xff) as usize;
    let routing_mode = (value >> GICD_SGIR_ROUTING_SHIFT) & 0x3;
    for target_cpu in this_zone().cpu_set().iter() {
        let selected = match routing_mode {
            0 => target_list & (1 << target_cpu) != 0,
            1 => target_cpu != source_cpu,
            2 => target_cpu == source_cpu,
            _ => false,
        };
        if selected && !crate::device::irqchip::post_guest_irq(target_cpu, irq_id, Some(source_cpu))
        {
            warn!("failed to post guest SGI {} to CPU{}", irq_id, target_cpu);
        }
    }
}

pub fn arch_check_events(event: Option<usize>) {
    match event {
        _ => {
            info!(
                "aarch64: arch_check_events: event={:#x?} do nothing now",
                event
            );
        }
    }
}
