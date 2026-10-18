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

use bit_field::BitField;

#[cfg(irq_gicv2)]
use crate::{arch::cpu::this_cpu_id, cpu_data::this_zone};

#[cfg(irq_gicv3)]
use crate::{
    arch::cpu::{cpuid_to_mpidr_affinity, this_cpu_id},
    cpu_data::this_zone,
};

// ICC_SGI1R_EL1 fields.
const ICC_SGI1R_TARGET_LIST_BITS: core::ops::Range<usize> = 0..16;
#[cfg(irq_gicv3)]
const ICC_SGI1R_AFF1_BITS: core::ops::Range<usize> = 16..24;
const ICC_SGI1R_INTID_BITS: core::ops::Range<usize> = 24..28;
#[cfg(irq_gicv3)]
const ICC_SGI1R_AFF2_BITS: core::ops::Range<usize> = 32..40;
const ICC_SGI1R_IRM_BIT: usize = 40;
#[cfg(irq_gicv3)]
const ICC_SGI1R_RS_BITS: core::ops::Range<usize> = 44..48;
#[cfg(irq_gicv3)]
const ICC_SGI1R_AFF3_BITS: core::ops::Range<usize> = 48..56;

// GICD_SGIR fields.
#[cfg(irq_gicv2)]
const GICD_SGIR_INTID_BITS: core::ops::Range<usize> = 0..4;
#[cfg(irq_gicv2)]
const GICD_SGIR_TARGET_LIST_BITS: core::ops::Range<usize> = 16..24;
#[cfg(irq_gicv2)]
const GICD_SGIR_TARGET_LIST_FILTER_BITS: core::ops::Range<usize> = 24..26;

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
            target_list: value.get_bits(ICC_SGI1R_TARGET_LIST_BITS) as u16,
            aff1: value.get_bits(ICC_SGI1R_AFF1_BITS) as u8,
            intid: value.get_bits(ICC_SGI1R_INTID_BITS) as usize,
            aff2: value.get_bits(ICC_SGI1R_AFF2_BITS) as u8,
            irm: value.get_bit(ICC_SGI1R_IRM_BIT),
            range_selector: value.get_bits(ICC_SGI1R_RS_BITS) as u8,
            aff3: value.get_bits(ICC_SGI1R_AFF3_BITS) as u8,
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
            && self.target_list.get_bit((aff0 - target_aff0) as usize)
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
    post_guest_sgi(
        value.get_bits(ICC_SGI1R_INTID_BITS) as usize,
        value.get_bits(ICC_SGI1R_TARGET_LIST_BITS) as u8,
        value.get_bit(ICC_SGI1R_IRM_BIT) as u32,
    );
}

#[cfg(irq_gicv2)]
pub fn handle_guest_sgir(value: u32) {
    post_guest_sgi(
        value.get_bits(GICD_SGIR_INTID_BITS) as usize,
        value.get_bits(GICD_SGIR_TARGET_LIST_BITS) as u8,
        value.get_bits(GICD_SGIR_TARGET_LIST_FILTER_BITS),
    );
}

#[cfg(irq_gicv2)]
fn post_guest_sgi(irq_id: usize, target_list: u8, filter: u32) {
    let source_cpu = this_cpu_id();
    for target_cpu in this_zone().cpu_set().iter() {
        let selected = match filter {
            0 => target_cpu < 8 && target_list.get_bit(target_cpu),
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
