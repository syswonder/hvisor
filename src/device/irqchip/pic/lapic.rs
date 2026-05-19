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
        acpi::get_apic_id,
        cpu::{this_apic_id, this_cpu_id},
        idt::IdtVector,
        ipi,
        msr::Msr::{self, *},
        zone::HvArchZoneConfig,
    },
    cpu_data::this_cpu_data,
    device::irqchip::pic::pop_vector,
    error::HvResult,
    memory::Frame,
    memory::MMIOAccess,
    zone::Zone,
};
use bit_field::BitField;
use core::{ops::Range, u32};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerMode};

// APIC Delivery Mode constants
const APIC_DM_INIT: u64 = 0x00500;
const APIC_DM_STARTUP: u64 = 0x00600;
const APIC_INT_LEVELTRIG: u64 = 0x08000;
const APIC_INT_ASSERT: u64 = 0x04000;

/// Convert CPU ID to LDR value for flat mode
/// In flat mode, CPU ID (0-7) maps to bit position (24-31) in LDR
pub fn cpu_id_to_flat_ldr(cpu_id: u32) -> u32 {
    if cpu_id >= 8 {
        panic!("CPU ID must be less than 8 for flat mode");
    }
    // Set the bit corresponding to the CPU ID in the logical destination field
    // CPU ID 0 maps to bit 24, CPU ID 1 to bit 25, etc.
    1 << (24 + cpu_id)
}

/// Convert LDR value to CPU IDs for flat mode
/// Returns an array of CPU IDs corresponding to each set bit in the LDR
pub fn flat_ldr_to_cpu_ids(ldr_value: u32) -> alloc::vec::Vec<u32> {
    let mut cpu_ids = alloc::vec::Vec::new();

    // Extract the logical destination field (bits 24-31)
    let logical_field = ldr_value & 0xFF000000;

    // Check each bit in the logical field
    for i in 0..8 {
        if (logical_field & (1 << (24 + i))) != 0 {
            cpu_ids.push(i);
        }
    }

    cpu_ids
}

pub struct VirtLocalApic {
    pub phys_lapic: LocalApic,
    pub virt_timer_vector: u8,
    virt_lvt_timer_bits: u32,
    icr_high: u64,
    is_icr_high_set: bool,
    is_flat_mode: bool,
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
            icr_high: 0,
            is_icr_high_set: false,
            is_flat_mode: false,
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

    pub fn write_icr_high(&mut self, value: u32) {
        self.icr_high = value as u64;
        self.is_icr_high_set = true;
    }

    pub fn rdmsr(&mut self, msr: Msr, is_mmio: bool) -> HvResult<u64> {
        match msr {
            IA32_X2APIC_APICID => {
                // info!("apicid: {:x}", this_cpu_id());
                Ok(this_apic_id() as u64)
            }
            IA32_X2APIC_LDR => {
                if is_mmio {
                    if self.is_flat_mode {
                        return Ok(cpu_id_to_flat_ldr(this_cpu_id() as u32) as u64);
                    }
                    Ok((this_apic_id() << 24).get_bits(0..32) as u64)
                } else {
                    Ok(this_apic_id() as u64)
                }
            }
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
            _ => Ok(msr.read()),
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
                if self.is_icr_high_set {
                    let dest_array = if self.is_flat_mode {
                        // Check if value contains specific APIC bits
                        let value_lower = value & 0xffff_ffff;
                        let has_special_bits = (value_lower & APIC_DM_INIT != 0)
                            || (value_lower & APIC_DM_STARTUP != 0)
                            || (value_lower & APIC_INT_LEVELTRIG != 0)
                            || (value_lower & APIC_INT_ASSERT != 0);

                        if has_special_bits {
                            alloc::vec![(self.icr_high as u64) << 8]
                        } else {
                            // Use flat_ldr_to_cpu_ids and shift each element left 32 bits
                            let cpu_ids = flat_ldr_to_cpu_ids(self.icr_high as u32);
                            if !cpu_ids.is_empty() {
                                cpu_ids
                                    .iter()
                                    .map(|&id| (get_apic_id(id as usize) as u64) << 32)
                                    .collect()
                            } else {
                                alloc::vec![(self.icr_high as u64) << 8] // fallback
                            }
                        }
                    } else {
                        alloc::vec![(self.icr_high as u64) << 8]
                    };
                    self.is_icr_high_set = false;

                    // Send IPI to each destination in the array
                    for &dest in &dest_array {
                        let icr_value = (dest & !0xffff_ffff) | (value & 0xffff_ffff);
                        ipi::send_ipi(icr_value)?;
                    }
                } else {
                    ipi::send_ipi(value);
                }
                Ok(())
            }
            IA32_X2APIC_LDR => {
                if !self.is_flat_mode {
                    unsafe { msr.write(value) };
                }
                Ok(())
            }
            IA32_X2APIC_DFR => {
                self.is_flat_mode = value == 0xffffffff;
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
            _ => {
                unsafe { msr.write(value) };
                Ok(())
            }
        }
    }
}

fn offset_to_lapic_msr(offset: usize) -> Result<Msr, u32> {
    // 0x0, 0x10, 0x20, ... mapped to x2apic MSR 0x800, 0x810, ...
    let msr = 0x800 + ((offset & 0xff0) >> 4);
    if msr == 0x831 {
        return Msr::try_from(0x830);
    }
    Msr::try_from(msr as u32)
}

pub fn mmio_lapic_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    warn!(
        "LAPIC MMIO access: addr={:#x} value={:#x} is_write={}",
        mmio.address, mmio.value, mmio.is_write
    );
    if let Ok(msr) = offset_to_lapic_msr(mmio.address) {
        /*info!(
            "lapic msr access: msr={:#x} value={:#x} is_write={}",
            msr as u32, mmio.value, mmio.is_write
        );*/
        let virt_lapic = &mut this_cpu_data().arch_cpu.virt_lapic;
        if msr == Msr::IA32_X2APIC_ICR && mmio.address == 0x310 && mmio.is_write {
            virt_lapic.write_icr_high(mmio.value as u32);
            return Ok(());
        }
        if mmio.is_write {
            virt_lapic.wrmsr(msr, mmio.value as u64)
        } else {
            mmio.value = virt_lapic.rdmsr(msr, true)?.try_into().unwrap_or(0);
            Ok(())
        }
    } else {
        /*info!(
            "unhandled mmio lapic access: addr={:#x} value={:#x} is_write={}",
            mmio.address, mmio.value, mmio.is_write
        );*/
        Ok(())
    }
}

impl Zone {
    pub fn lapic_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        let lapic_base = 0xfee0_0000;
        let lapic_size = 0x1000;
        self.write()
            .mmio_region_register(lapic_base, lapic_size, mmio_lapic_handler, lapic_base);
    }
}
