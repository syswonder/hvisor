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
//    Hangqi Ren <2572131118@qq.com>
use crate::arch::cpu::this_cpu_id;
use crate::arch::zone::{GicConfig, Gicv2Config, HvArchZoneConfig};
use crate::config::{BitmapWord, CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD};
use crate::cpu_data::{get_cpu_data, this_zone};
use crate::device::irqchip::gicv2::gicd::{
    get_max_int_num, GICD, GICD_CPENDSGIR_REG_OFFSET, GICD_CTRL_REG_OFFSET,
    GICD_ICACTIVER_REG_OFFSET, GICD_ICENABLER_REG_OFFSET, GICD_ICFGR_REG_OFFSET,
    GICD_ICPENDR_REG_OFFSET, GICD_IDENTIFICATION_NUM, GICD_IDENTIFICATION_OFFSET,
    GICD_IGROUPR_REG_OFFSET, GICD_IIDR_REG_OFFSET, GICD_IPRIORITYR_REG_OFFSET,
    GICD_ISACTIVER_REG_OFFSET, GICD_ISENABLER_REG_OFFSET, GICD_ISPENDR_REG_OFFSET,
    GICD_ITARGETSR_REG_OFFSET, GICD_LOCK, GICD_SGIR_REG_OFFSET, GICD_SGIR_ROUTING_SHIFT,
    GICD_SGIR_TARGET_LIST_FILTER_SHIFT, GICD_SPENDSGIR_REG_OFFSET, GICD_TYPER_REG_OFFSET,
    GICV2_CONFIG_REGS_NUM, GICV2_INT_REGS_NUM, GICV2_PRIO_REGS_NUM, GICV2_SGIS_NUM,
    GICV2_SGI_REGS_NUM, GICV2_TARGET_REGS_NUM,
};
use crate::device::irqchip::gicv2::GICV2;
use crate::error::HvResult;
use crate::memory::{mmio_perform_access, MMIOAccess, MemFlags, MemoryRegion};
/// This file defines and implements the functional functions of virtual gicv2.
/// author: ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
use crate::zone::Zone;
use bit_field::BitField;
use core::sync::atomic::Ordering;

const GICV2_REG_WIDTH: usize = 4;
/// SGI bits of GICD_ISPENDR0/ICPENDR0; bits 16..32 are physical PPIs.
const SGI_PENDING_BITS: core::ops::Range<usize> = 0..16;

impl Zone {
    // trap all Guest OS accesses to the GIC Distributor registers.
    pub fn vgicv2_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        let zone_id = self.id();
        let mut inner = self.write();
        match arch.gic_config {
            GicConfig::Gicv3(_) => {
                panic!("GICv3 is not supported in this version of hvisor");
            }
            GicConfig::Gicv2(ref gicv2_config) => {
                if gicv2_config.gicd_base == 0 {
                    panic!("vgicv2_mmio_init: gicd_base is null");
                }
                info!("Initializing GICv2 MMIO regions for zone {}", zone_id);
                inner.mmio_region_register(
                    gicv2_config.gicd_base,
                    gicv2_config.gicd_size,
                    vgicv2_dist_handler,
                    0,
                );
            }
        }
    }

    // remap the GIC CPU interface register address space to point to the GIC virtual CPU interface registers.
    pub fn vgicv2_remap_init(&mut self, arch: &HvArchZoneConfig) {
        let zone_id = self.id();
        let mut inner = self.write();
        match arch.gic_config {
            GicConfig::Gicv3(_) => {
                panic!("GICv3 is not supported in this version of hvisor");
            }
            GicConfig::Gicv2(ref gicv2_config) => {
                if gicv2_config.gicc_base == 0
                    || gicv2_config.gicv_base == 0
                    || gicv2_config.gicc_size == 0
                    || gicv2_config.gicv_size == 0
                {
                    panic!("vgicv2_remap_init: gic related address is null");
                }
                if gicv2_config.gicv_size != gicv2_config.gicc_size {
                    panic!("vgicv2_remap_init: gicv_size not equal to gicc_size");
                }
                info!(
                    "Remaping GICv2 GICV MMIO regions to GICC MMIO regions for zone {}",
                    zone_id
                );
                // map gicv memory region to gicc memory region.
                inner
                    .gpm_mut()
                    .insert(MemoryRegion::new_with_offset_mapper(
                        gicv2_config.gicc_base,
                        gicv2_config.gicv_base,
                        gicv2_config.gicc_size,
                        MemFlags::READ | MemFlags::WRITE,
                    ))
                    .unwrap();
            }
        }
    }

    // store the interrupt number in the irq_bitmap.
    pub fn irq_bitmap_init(&mut self, irqs_bitmap: &[BitmapWord]) {
        let zone_id = self.id();
        let mut inner = self.write();
        // Enable each cpu's sgi and ppi access permission
        inner.irq_bitmap_mut()[0] = 0xffff_ffff;

        for i in 0..irqs_bitmap.len() {
            let word = irqs_bitmap[i];

            for j in 0..CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD {
                if ((word >> j) & 1) == 1 {
                    let irq_id = (i * CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD + j) as u32;
                    assert!(irq_id < get_max_int_num() as u32);
                    let irq_index = irq_id / 32;
                    let irq_bit = irq_id % 32;
                    inner.irq_bitmap_mut()[irq_index as usize] |= 1 << irq_bit;
                }
            }
        }

        for (index, &word) in inner.irq_bitmap().iter().enumerate() {
            for bit_position in 0..32 {
                if word & (1 << bit_position) != 0 {
                    let interrupt_number = index * 32 + bit_position;
                    info!(
                        "Found interrupt in Zone {} irq_bitmap: {}",
                        zone_id, interrupt_number
                    );
                }
            }
        }
    }
}

pub fn reg_range(base: usize, n: usize, size: usize) -> core::ops::Range<usize> {
    base..(base + n * size)
}

// extend from gicv3, support half-word and byte access.
fn restrict_bitmask_access(
    mmio: &mut MMIOAccess,
    reg_index: usize,
    bits_per_irq: usize,
    is_poke: bool,
    gicd_base: usize,
) -> HvResult {
    let zone = this_zone();
    let zone_r = zone.read();
    let mut access_mask: usize = 0;
    /*
     * In order to avoid division, the number of bits per irq is limited
     * to powers of 2 for the moment.
     */
    let irqs_per_reg = 32 / bits_per_irq;
    let irq_bits = (1 << bits_per_irq) - 1;
    /* First, extract the first interrupt affected by this access */
    let first_irq = reg_index * irqs_per_reg;
    trace!("restrict_bitmask_access: first_irq: {}", first_irq);
    trace!("mmio.address: {:#x}", mmio.address);
    trace!("mmio.size: {:#x}", mmio.size);
    trace!("mmio.value: {:#x}", mmio.value);
    for irq in 0..irqs_per_reg {
        if zone_r.irq_in_zone((first_irq + irq) as _) {
            trace!("restrict visit irq {}", first_irq + irq);
            access_mask |= irq_bits << (irq * bits_per_irq);
        }
    }

    let mut other_mask: usize = 0;
    for offset in 0..mmio.size {
        other_mask |= 0xff << ((mmio.address + offset) % 4) * 8;
    }
    trace!(
        "access_mask: {:#x}, other_mask: {:#x}",
        access_mask,
        other_mask
    );

    // add the mask of the other bits in the register
    let address = mmio.address & 0xfffffffc;
    let size = 4;
    let offset = mmio.address & 0x3;
    let value = mmio.value << (offset * 8);
    let real_mask = access_mask & other_mask;
    trace!("mmio.iswrite: {}", mmio.is_write);
    trace!(
        "address: {:#x}, size: {:#x}, value: {:#x}, real_mask: {:#x}",
        address,
        size,
        value,
        real_mask
    );

    let offset = mmio.address & 0x3;
    mmio.address = mmio.address & 0xfffffffc;
    mmio.size = 4;
    mmio.value <<= offset * 8;
    access_mask = access_mask & other_mask;

    if !mmio.is_write {
        /* Restrict the read value */
        mmio_perform_access(gicd_base, mmio);
        mmio.value &= access_mask;
        mmio.value >>= offset * 8;
        return Ok(());
    }

    if !is_poke {
        trace!("is_poke is false");
        /*
         * Modify the existing value of this register by first reading
         * it into mmio->value
         * Relies on a spinlock since we need two mmio accesses.
         */
        let access_val = mmio.value;

        let _lock = GICD_LOCK.lock();

        mmio.is_write = false;
        mmio_perform_access(gicd_base, mmio);

        mmio.is_write = true;
        mmio.value &= !access_mask;
        mmio.value |= access_val & access_mask;
        mmio_perform_access(gicd_base, mmio);

        // drop lock automatically here
    } else {
        trace!("poke is true");
        mmio.value &= access_mask;
        mmio_perform_access(gicd_base, mmio);
    }
    Ok(())
}

// general GIC Distributor register access.
fn vgicv2_dist_misc_access(mmio: &mut MMIOAccess, gicd_base: usize) -> HvResult {
    let reg = mmio.address;
    if reg == GICD_CTRL_REG_OFFSET
        || reg == GICD_TYPER_REG_OFFSET
        || reg == GICD_IIDR_REG_OFFSET
        || reg_range(
            GICD_IDENTIFICATION_OFFSET,
            GICD_IDENTIFICATION_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg)
    {
        if !mmio.is_write {
            // ignore write
            mmio_perform_access(gicd_base, mmio);
        }
    } else {
        todo!()
    }

    Ok(())
}

pub fn set_sgi_irq(irq_id: usize, target_list: usize, routing_mode: usize) {
    let val = irq_id
        | target_list << GICD_SGIR_TARGET_LIST_FILTER_SHIFT
        | routing_mode << GICD_SGIR_ROUTING_SHIFT;
    trace!(
        "set_sgi_irq: irq_id: {}, target_list: {}, routing_mode: {}",
        irq_id,
        target_list,
        routing_mode
    );
    trace!("ISENABLER: {:#x}", GICD.get().unwrap().get_isenabler(0));
    GICD.get().unwrap().set_sgir(val as u32);
}

fn read_guest_sgi_source_mask(target_cpu: usize, irq_id: usize, lr_source_mask: u8) -> u8 {
    let target = get_cpu_data(target_cpu);
    let software = if target.vcpu.irq.pending_mask(irq_id, 1) != 0 {
        target.vcpu.arch.sgi_source[irq_id].load(core::sync::atomic::Ordering::Acquire)
    } else {
        0
    };
    software | lr_source_mask
}

fn handle_gicd_sgi_pending_access(mmio: &mut MMIOAccess, is_clear: bool) -> HvResult {
    let target_cpu = this_cpu_id();
    let word_offset = mmio.address & 0x3;
    let shift = word_offset * 8;
    let width_mask = if mmio.size == 4 {
        u32::MAX
    } else {
        ((1u32 << (mmio.size * 8)) - 1) << shift
    };
    if mmio.is_write {
        let value = ((mmio.value as u32) << shift) & width_mask;
        for byte in 0..4 {
            if width_mask & (0xff << (byte * 8)) == 0 {
                continue;
            }
            let source_mask = ((value >> (byte * 8)) & 0xff) as u8;
            let base = if is_clear {
                GICD_CPENDSGIR_REG_OFFSET
            } else {
                GICD_SPENDSGIR_REG_OFFSET
            };
            let irq_id = ((mmio.address & !3) - base) / 4 * 4 + byte;
            if irq_id >= 16 {
                continue;
            }
            if is_clear {
                crate::device::irqchip::gicv2::gic::clear_guest_sgi_pending_sources(
                    target_cpu,
                    irq_id,
                    source_mask,
                );
            } else {
                for source in 0..8 {
                    if source_mask.get_bit(source) {
                        crate::device::irqchip::post_guest_irq(target_cpu, irq_id, Some(source));
                    }
                }
            }
        }
        return Ok(());
    }

    let lr_summary = crate::device::irqchip::gicv2::gic::read_pending_sgi_lr_summary();
    let mut value = 0u32;
    for byte in 0..4 {
        let irq_id = first_irq + byte;
        value.set_bits(
            byte * 8..byte * 8 + 8,
            read_guest_sgi_source_mask(target_cpu, irq_id, lr_summary.source_masks[irq_id]) as u32,
        );
    }
    mmio.value = ((value & width_mask) >> shift) as usize;
    Ok(())
}

fn handle_gicd_spendsgir_access(mmio: &mut MMIOAccess) -> HvResult {
    handle_gicd_sgi_pending_access(mmio, false)
}

fn handle_gicd_cpendsgir_access(mmio: &mut MMIOAccess) -> HvResult {
    handle_gicd_sgi_pending_access(mmio, true)
}

fn handle_gicd_intid_pending_access(
    mmio: &mut MMIOAccess,
    is_clear: bool,
    gicd_base: usize,
) -> HvResult {
    let target_cpu = this_cpu_id();
    let shift = (mmio.address & 0x3) * 8;
    let width_mask = if mmio.size == 4 {
        u32::MAX
    } else {
        ((1u32 << (mmio.size * 8)) - 1) << shift
    };
    if mmio.is_write {
        let value = ((mmio.value as u32) << shift) & width_mask;
        for irq_id in 0..16 {
            if value & (1 << irq_id) == 0 {
                continue;
            }
            if is_clear {
                crate::device::irqchip::gicv2::gic::clear_guest_sgi_pending(target_cpu, irq_id);
            } else {
                crate::device::irqchip::post_guest_irq(target_cpu, irq_id, Some(target_cpu));
            }
        }
        if mmio.size == 4 && shift == 0 {
            mmio.value = (value & !0xffff) as usize;
            mmio_perform_access(gicd_base, mmio);
        } else if shift >= 16 {
            mmio_perform_access(gicd_base, mmio);
        }
        return Ok(());
    }
    mmio_perform_access(gicd_base, mmio);
    let physical_value = ((mmio.value as u32) << shift) & width_mask;
    let lr_pending = crate::device::irqchip::gicv2::gic::read_pending_sgi_lr_summary();
    let mut value = get_cpu_data(target_cpu).vcpu.irq.pending_mask(0, 16) as u32;
    value |= lr_pending.pending_mask;
    mmio.value = ((((physical_value & !0xffff) | value) & width_mask) >> shift) as usize;
    Ok(())
}

fn handle_gicd_ispendr_access(mmio: &mut MMIOAccess, gicd_base: usize) -> HvResult {
    handle_gicd_intid_pending_access(mmio, false, gicd_base)
}

fn handle_gicd_icpendr_access(mmio: &mut MMIOAccess, gicd_base: usize) -> HvResult {
    handle_gicd_intid_pending_access(mmio, true, gicd_base)
}

// Handle GIC Distributor register accesses.
pub fn vgicv2_dist_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    let gicd_base = GICV2.get().unwrap().gicd_base;
    let reg = mmio.address;

    match reg {
        reg if reg == GICD_SGIR_REG_OFFSET => {
            if !mmio.is_write {
                return Ok(());
            }
            crate::arch::ipi::handle_guest_sgir(mmio.value as u32);
            Ok(())
        }
        reg if reg_range(GICD_ISPENDR_REG_OFFSET, 1, GICV2_REG_WIDTH).contains(&reg) => {
            handle_gicd_ispendr_access(mmio, gicd_base)
        }
        reg if reg_range(GICD_ICPENDR_REG_OFFSET, 1, GICV2_REG_WIDTH).contains(&reg) => {
            handle_gicd_icpendr_access(mmio, gicd_base)
        }
        reg if reg_range(
            GICD_CPENDSGIR_REG_OFFSET,
            GICV2_SGI_REGS_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg) =>
        {
            handle_gicd_cpendsgir_access(mmio)
        }
        reg if reg_range(
            GICD_SPENDSGIR_REG_OFFSET,
            GICV2_SGI_REGS_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg) =>
        {
            handle_gicd_spendsgir_access(mmio)
        }
        reg if reg_range(
            GICD_ITARGETSR_REG_OFFSET,
            GICV2_TARGET_REGS_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0x3ff) / 4, 8, false, gicd_base)
        }
        reg if reg_range(
            GICD_ICENABLER_REG_OFFSET,
            GICV2_INT_REGS_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg)
            || reg_range(
                GICD_ISENABLER_REG_OFFSET,
                GICV2_INT_REGS_NUM,
                GICV2_REG_WIDTH,
            )
            .contains(&reg)
            || reg_range(GICD_ICPENDR_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH)
                .contains(&reg)
            || reg_range(GICD_ISPENDR_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH)
                .contains(&reg)
            || reg_range(
                GICD_ICACTIVER_REG_OFFSET,
                GICV2_INT_REGS_NUM,
                GICV2_REG_WIDTH,
            )
            .contains(&reg)
            || reg_range(
                GICD_ISACTIVER_REG_OFFSET,
                GICV2_INT_REGS_NUM,
                GICV2_REG_WIDTH,
            )
            .contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, true, gicd_base)
        }
        reg if reg_range(GICD_IGROUPR_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH)
            .contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, false, gicd_base)
        }
        reg if reg_range(
            GICD_ICFGR_REG_OFFSET,
            GICV2_CONFIG_REGS_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0xff) / 4, 2, false, gicd_base)
        }
        reg if reg_range(
            GICD_IPRIORITYR_REG_OFFSET,
            GICV2_PRIO_REGS_NUM,
            GICV2_REG_WIDTH,
        )
        .contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0x3ff) / 4, 8, false, gicd_base)
        }
        _ => vgicv2_dist_misc_access(mmio, gicd_base),
    }
}
