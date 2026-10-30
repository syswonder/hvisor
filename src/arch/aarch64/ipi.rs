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
use crate::{arch::cpu::this_cpu_id, cpu_data::this_zone};
#[cfg(irq_gicv3)]
use crate::{
    arch::cpu::{cpuid_to_mpidr_affinity, this_cpu_id},
    cpu_data::this_zone,
};
use bit_field::BitField;

// An ICC_SGI1R_EL1 value written by the guest.
#[derive(Clone, Copy)]
struct Sgi1r(u64);

impl Sgi1r {
    fn target_list(self) -> u16 {
        self.0.get_bits(0..16) as u16
    }

    fn intid(self) -> usize {
        self.0.get_bits(24..28) as usize
    }

    // Interrupt Routing Mode: target every PE except the requester.
    fn irm(self) -> bool {
        self.0.get_bit(40)
    }

    #[cfg(irq_gicv3)]
    fn aff1(self) -> u64 {
        self.0.get_bits(16..24)
    }

    #[cfg(irq_gicv3)]
    fn aff2(self) -> u64 {
        self.0.get_bits(32..40)
    }

    #[cfg(irq_gicv3)]
    fn range_selector(self) -> u64 {
        self.0.get_bits(44..48)
    }

    #[cfg(irq_gicv3)]
    fn aff3(self) -> u64 {
        self.0.get_bits(48..56)
    }

    #[cfg(irq_gicv3)]
    fn targets(self, cpu_id: usize, source_cpu: usize) -> bool {
        if self.irm() {
            return cpu_id != source_cpu;
        }
        let (aff3, aff2, aff1, aff0) = cpuid_to_mpidr_affinity(cpu_id as u64);
        let target_aff0 = self.range_selector() * 16;
        aff3 == self.aff3()
            && aff2 == self.aff2()
            && aff1 == self.aff1()
            && aff0 >= target_aff0
            && aff0 < target_aff0 + 16
            && self.target_list().get_bit((aff0 - target_aff0) as usize)
    }
}

// A GICD_SGIR value written by the guest.
#[cfg(irq_gicv2)]
#[derive(Clone, Copy)]
struct Sgir(u32);

#[cfg(irq_gicv2)]
impl Sgir {
    fn intid(self) -> usize {
        self.0.get_bits(0..4) as usize
    }

    fn target_list(self) -> u8 {
        self.0.get_bits(16..24) as u8
    }

    fn target_list_filter(self) -> u32 {
        self.0.get_bits(24..26)
    }
}

pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    gic_send_event(cpu_id, sgi_num);
}

pub fn arch_notify_event(cpu_id: u64, sgi_num: u64, _event_id: usize, _queue_was_empty: bool) {
    arch_send_event(cpu_id, sgi_num);
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

#[cfg(irq_gicv3)]
pub fn handle_guest_sgi1r(value: u64) {
    let sgi = Sgi1r(value);
    let source_cpu = this_cpu_id();
    for target_cpu in this_zone().cpu_set().iter() {
        if sgi.targets(target_cpu, source_cpu) {
            if !crate::device::irqchip::post_guest_irq(target_cpu, sgi.intid(), None) {
                error!(
                    "failed to post guest SGI {} to CPU{}",
                    sgi.intid(),
                    target_cpu
                );
            }
        }
    }
}

#[cfg(irq_gicv2)]
pub fn handle_guest_sgi1r(value: u64) {
    let sgi = Sgi1r(value);
    post_guest_sgi(sgi.intid(), sgi.target_list() as u8, sgi.irm() as u32);
}

#[cfg(irq_gicv2)]
pub fn handle_guest_sgir(value: u32) {
    let sgi = Sgir(value);
    post_guest_sgi(sgi.intid(), sgi.target_list(), sgi.target_list_filter());
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
