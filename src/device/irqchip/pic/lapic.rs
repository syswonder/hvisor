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
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{
        cpu::{this_apic_id, this_cpu_id},
        idt::IdtVector,
        ipi,
        msr::Msr::{self, *},
    },
    device::irqchip::pic::pop_vector,
    error::HvResult,
    memory::Frame,
    percpu::this_cpu_data,
};
use bit_field::BitField;
use core::{ops::Range, u32};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerMode};

pub struct VirtLocalApic {
    pub phys_lapic: LocalApic,
    pub virt_timer_vector: u8,
    virt_lvt_timer_bits: u32,
}

impl VirtLocalApic {
    pub fn new() -> Self {
        Self {
            phys_lapic: Self::new_phys_lapic(
                IdtVector::APIC_TIMER_VECTOR as _,
                IdtVector::APIC_ERROR_VECTOR as _,
                IdtVector::APIC_SPURIOUS_VECTOR as _,
            ),
            virt_timer_vector: IdtVector::APIC_TIMER_VECTOR as _,
            virt_lvt_timer_bits: (1 << 16) as _, // masked
        }
    }

    fn new_phys_lapic(timer: usize, error: usize, spurious: usize) -> LocalApic {
        let mut lapic = LocalApicBuilder::new()
            .timer_vector(timer)
            .error_vector(error)
            .spurious_vector(spurious)
            .build()
            .unwrap();
        unsafe {
            lapic.enable();
            lapic.disable_timer();
        }
        lapic
    }

    pub const fn msr_range() -> Range<u32> {
        0x800..0x840
    }

    pub fn phys_local_apic<'a>() -> &'a mut LocalApic {
        &mut this_cpu_data().arch_cpu.virt_lapic.phys_lapic
    }

    pub fn rdmsr(&mut self, msr: Msr) -> HvResult<u64> {
        match msr {
            IA32_X2APIC_APICID => {
                // info!("apicid: {:x}", this_cpu_id());
                Ok(this_apic_id() as u64)
            }
            IA32_X2APIC_LDR => Ok(this_apic_id() as u64), // logical apic id
            IA32_X2APIC_ISR0 | IA32_X2APIC_ISR1 | IA32_X2APIC_ISR2 | IA32_X2APIC_ISR3
            | IA32_X2APIC_ISR4 | IA32_X2APIC_ISR5 | IA32_X2APIC_ISR6 | IA32_X2APIC_ISR7 => {
                // info!("isr!");
                Ok(0)
            }
            IA32_X2APIC_IRR0 | IA32_X2APIC_IRR1 | IA32_X2APIC_IRR2 | IA32_X2APIC_IRR3
            | IA32_X2APIC_IRR4 | IA32_X2APIC_IRR5 | IA32_X2APIC_IRR6 | IA32_X2APIC_IRR7 => {
                // info!("irr!");
                Ok(0)
            }
            IA32_X2APIC_LVT_TIMER => Ok(self.virt_lvt_timer_bits as _),
            _ => hv_result_err!(ENOSYS),
        }
    }

    pub fn wrmsr(&mut self, msr: Msr, value: u64) -> HvResult {
        match msr {
            IA32_X2APIC_EOI => {
                // info!("eoi");
                pop_vector(this_cpu_id());
                Ok(())
            }
            IA32_X2APIC_ICR => {
                // info!("ICR value: {:x}", value);
                ipi::send_ipi(value);
                Ok(())
            }
            IA32_X2APIC_LVT_TIMER => {
                self.virt_lvt_timer_bits = value as u32;
                let timer = value.get_bits(0..=7) as u8;
                if timer != self.virt_timer_vector {
                    self.virt_timer_vector = timer;
                    self.phys_lapic = Self::new_phys_lapic(
                        timer as _,
                        IdtVector::APIC_ERROR_VECTOR as _,
                        IdtVector::APIC_SPURIOUS_VECTOR as _,
                    )
                }
                unsafe {
                    self.phys_lapic
                        .set_timer_mode(match value.get_bits(17..19) {
                            0 => TimerMode::OneShot,
                            1 => TimerMode::Periodic,
                            _ => TimerMode::TscDeadline,
                        });
                    if value.get_bit(16) {
                        self.phys_lapic.disable_timer();
                    } else {
                        self.phys_lapic.enable_timer();
                    }
                }
                Ok(())
            }
            _ => hv_result_err!(ENOSYS),
        }
    }
}
