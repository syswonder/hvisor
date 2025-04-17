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
use alloc::sync::Arc;

use super::{gicd::GICD_LOCK, is_spi};
use crate::{
    arch::zone::HvArchZoneConfig,
    consts::MAX_CPU_NUM,
    device::irqchip::gicv3::{
        gicd::*, gicr::*, gits::*, host_gicd_base, host_gicr_base, host_gits_base,
        MAINTENACE_INTERRUPT, PER_GICR_SIZE,
    },
    error::HvResult,
    hypercall::SGI_IPI_ID,
    memory::{mmio_perform_access, MMIOAccess},
    percpu::{get_cpu_data, this_zone},
    zone::{this_zone_id, Zone},
};

pub fn reg_range(base: usize, n: usize, size: usize) -> core::ops::Range<usize> {
    base..(base + (n - 1) * size)
}

impl Zone {
    pub fn vgicv3_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        if arch.gicd_base == 0 || arch.gicr_base == 0 {
            panic!("vgicv3_mmio_init: gicd_base or gicr_base is null");
        }

        self.mmio_region_register(arch.gicd_base, arch.gicd_size, vgicv3_dist_handler, 0);
        self.mmio_region_register(arch.gits_base, arch.gits_size, vgicv3_its_handler, 0);

        for cpu in 0..MAX_CPU_NUM {
            let gicr_base = arch.gicr_base + cpu * PER_GICR_SIZE;
            debug!("registering gicr {} at {:#x?}", cpu, gicr_base);
            self.mmio_region_register(gicr_base, PER_GICR_SIZE, vgicv3_redist_handler, cpu);
        }
    }

    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {
        for irq in irqs {
            self.insert_irq_to_bitmap(*irq);
        }
        for (index, &word) in self.irq_bitmap.iter().enumerate() {
            for bit_position in 0..32 {
                if word & (1 << bit_position) != 0 {
                    let interrupt_number = index * 32 + bit_position;
                    info!(
                        "Found interrupt in Zone {} irq_bitmap: {}",
                        self.id, interrupt_number
                    );
                }
            }
        }
    }

    fn insert_irq_to_bitmap(&mut self, irq: u32) {
        assert!(irq < 1024); // 1024 is the maximum number of interrupts supported by GICv3 (GICD_TYPER.ITLinesNumber)
        let irq_index = irq / 32;
        let irq_bit = irq % 32;
        self.irq_bitmap[irq_index as usize] |= 1 << irq_bit;
    }
}

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

    for irq in 0..irqs_per_reg {
        if zone_r.irq_in_zone((first_irq + irq) as _) {
            trace!("restrict visit irq {}", first_irq + irq);
            access_mask |= irq_bits << (irq * bits_per_irq);
        }
    }

    if !mmio.is_write {
        /* Restrict the read value */
        mmio_perform_access(gicd_base, mmio);
        mmio.value &= access_mask;
        return Ok(());
    }

    if !is_poke {
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
        mmio.value &= access_mask;
        mmio_perform_access(gicd_base, mmio);
    }
    Ok(())
}

pub fn vgicv3_redist_handler(mmio: &mut MMIOAccess, cpu: usize) -> HvResult {
    trace!("gicr({}) mmio = {:#x?}", cpu, mmio);
    let gicr_base = host_gicr_base(cpu);
    match mmio.address {
        GICR_CTLR => {
            if get_cpu_data(cpu).zone.is_none() {
                if !mmio.is_write {
                    mmio_perform_access(gicr_base, mmio);
                }
            } else if Arc::ptr_eq(&this_zone(), get_cpu_data(cpu).zone.as_ref().unwrap()) {
                mmio_perform_access(gicr_base, mmio);
            } else {
                if !mmio.is_write {
                    mmio_perform_access(gicr_base, mmio);
                }
            }
        }
        GICR_TYPER => {
            mmio_perform_access(gicr_base, mmio);
            if cpu == MAX_CPU_NUM - 1 {
                mmio.value |= GICR_TYPER_LAST;
            }
        }
        GICR_IIDR | 0xffd0..=0xfffc => {
            // Read-only registers that might be used by a zone to find the redistributor corresponding to a CPU. Keep them accessible.
            mmio_perform_access(gicr_base, mmio);
        }
        GICR_PENDBASER => {
            // every redist have its own pending tbl
            mmio_perform_access(gicr_base, mmio);
            if mmio.is_write {
                trace!("write pending tbl base : 0x{:x}", mmio.value);
            } else {
                trace!("read pending tbl base : 0x{:x}", mmio.value);
            }
        }
        GICR_PROPBASER => {
            // all the redist share one prop tbl
            // mmio_perform_access(gicr_base, mmio);
            if mmio.is_write {
                set_prop_baser(mmio.value);
                trace!("write prop tbl base : 0x{:x}!", mmio.value);
            } else {
                mmio.value = read_prop_baser();
                trace!("read prop tbl base : 0x{:x}", mmio.value);
            }
        }
        GICR_SYNCR => {
            mmio.value = 0;
        }
        GICR_SETLPIR => {
            mmio_perform_access(gicr_base, mmio);
        }
        reg if reg == GICR_CLRLPIR || reg == GICR_INVALLR => {
            mmio_perform_access(gicr_base, mmio);
        }
        GICR_INVLPIR => {
            // Presume that this write is to enable an LPI.
            // Or we need to check all the proptbl created by vm.
            enable_one_lpi((mmio.value & 0xffffffff) - 8192);
        }
        reg if reg == GICR_STATUSR
            || reg == GICR_WAKER
            || reg == GICR_SGI_BASE + GICR_ISENABLER
            || reg == GICR_SGI_BASE + GICR_ICENABLER
            || reg == GICR_SGI_BASE + GICR_ISPENDR
            || reg == GICR_SGI_BASE + GICR_ICPENDR
            || reg == GICR_SGI_BASE + GICR_ISACTIVER
            || reg == GICR_SGI_BASE + GICR_ICACTIVER
            || reg_range(GICR_SGI_BASE + GICR_IPRIORITYR, 8, 4).contains(&reg)
            || reg_range(GICR_SGI_BASE + GICR_ICFGR, 2, 4).contains(&reg) =>
        {
            if Arc::ptr_eq(&this_zone(), get_cpu_data(cpu).zone.as_ref().unwrap()) {
                // avoid linux disable maintenance interrupt
                if reg == GICR_SGI_BASE + GICR_ICENABLER {
                    mmio.value &= !(1 << MAINTENACE_INTERRUPT);
                    mmio.value &= !(1 << SGI_IPI_ID);
                }
                // ignore access to foreign redistributors
                mmio_perform_access(gicr_base, mmio);
            } else {
                trace!("*** gicv3_gicr_mmio_handler: ignore access to foreign redistributors ***");
            }
        }
        _ => {}
    }
    HvResult::Ok(())
}

// The return value should be the register value to be read.
fn vgicv3_handle_irq_ops(mmio: &mut MMIOAccess, irq: u32) -> HvResult {
    let zone = this_zone();
    let zone_r = zone.read();

    if !is_spi(irq) || !zone_r.irq_in_zone(irq) {
        debug!(
            "gicd-mmio: skip irq {} access, reg = {:#x?}",
            irq, mmio.address
        );
        return Ok(());
    }

    mmio_perform_access(host_gicd_base(), mmio);

    Ok(())
}

fn vgicv3_dist_misc_access(mmio: &mut MMIOAccess, gicd_base: usize) -> HvResult {
    let reg = mmio.address;
    if reg_range(GICDV3_PIDR0, 4, 4).contains(&reg)
        || reg_range(GICDV3_PIDR4, 4, 4).contains(&reg)
        || reg_range(GICDV3_CIDR0, 4, 4).contains(&reg)
        || reg == GICD_CTLR
        || reg == GICD_TYPER
        || reg == GICD_IIDR
        || reg == GICD_TYPER2
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

pub fn vgicv3_dist_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    trace!("gicd mmio = {:#x?}", mmio);
    let gicd_base = host_gicd_base();
    let reg = mmio.address;

    match reg {
        reg if reg_range(GICD_IROUTER, 1024, 8).contains(&reg) => {
            vgicv3_handle_irq_ops(mmio, (reg - GICD_IROUTER) as u32 / 8)
        }
        reg if reg_range(GICD_ITARGETSR, 1024, 1).contains(&reg) => {
            vgicv3_handle_irq_ops(mmio, (reg - GICD_ITARGETSR) as u32)
        }
        reg if reg_range(GICD_ICENABLER, 32, 4).contains(&reg)
            || reg_range(GICD_ISENABLER, 32, 4).contains(&reg)
            || reg_range(GICD_ICPENDR, 32, 4).contains(&reg)
            || reg_range(GICD_ISPENDR, 32, 4).contains(&reg)
            || reg_range(GICD_ICACTIVER, 32, 4).contains(&reg)
            || reg_range(GICD_ISACTIVER, 32, 4).contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, true, gicd_base)
        }
        reg if reg_range(GICD_IGROUPR, 32, 4).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, false, gicd_base)
        }
        reg if reg_range(GICD_ICFGR, 64, 4).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0xff) / 4, 2, false, gicd_base)
        }
        reg if reg_range(GICD_IPRIORITYR, 255, 4).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x3ff) / 4, 8, false, gicd_base)
        }
        _ => vgicv3_dist_misc_access(mmio, gicd_base),
    }
}

pub fn vgicv3_its_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    let gits_base = host_gits_base();
    let reg = mmio.address;

    // mmio_perform_access(gits_base, mmio);
    match reg {
        GITS_CTRL => {
            mmio_perform_access(gits_base, mmio);
            if mmio.is_write {
                trace!("write GITS_CTRL: {:#x}", mmio.value);
            } else {
                trace!("read GITS_CTRL: {:#x}", mmio.value);
            }
        }
        GITS_CBASER => {
            if this_zone_id() == 0 {
                mmio_perform_access(gits_base, mmio);
            }
            if mmio.is_write {
                set_cbaser(mmio.value);
            } else {
                mmio.value = read_cbaser();
            }
            if mmio.is_write {
                info!("write GITS_CBASER: {:#x}", mmio.value);
            } else {
                info!("read GITS_CBASER: {:#x}", mmio.value);
            }
        }
        GITS_BASER => {
            if this_zone_id() == 0 {
                mmio_perform_access(gits_base, mmio);
            } else {
                if mmio.is_write {
                    set_dt_baser(mmio.value);
                } else {
                    mmio.value = read_dt_baser();
                }
            }
            if mmio.is_write {
                trace!("write GITS_BASER: 0x{:016x}", mmio.value);
            } else {
                trace!("read GITS_BASER: 0x{:016x}", mmio.value);
            }
        }
        GITS_COLLECTION_BASER => {
            if this_zone_id() == 0 {
                mmio_perform_access(gits_base, mmio);
            } else {
                if mmio.is_write {
                    set_ct_baser(mmio.value);
                } else {
                    mmio.value = read_ct_baser();
                }
            }
            if mmio.is_write {
                trace!("write GITS_COLL_BASER: 0x{:016x}", mmio.value);
            } else {
                trace!("read GITS_COLL_BASER: 0x{:016x}", mmio.value);
            }
        }
        GITS_CWRITER => {
            if mmio.is_write {
                trace!("write GITS_CWRITER: {:#x}", mmio.value);
                set_cwriter(mmio.value);
            } else {
                mmio.value = read_cwriter();
                trace!("read GITS_CWRITER: {:#x}", mmio.value);
            }
        }
        GITS_CREADR => {
            mmio.value = read_creadr();
            trace!("read GITS_CREADER: {:#x}", mmio.value);
        }
        GITS_TYPER => {
            mmio_perform_access(gits_base, mmio);
            trace!("GITS_TYPER: {:#x}", mmio.value);
        }
        _ => {
            mmio_perform_access(gits_base, mmio);
            if mmio.is_write {
                trace!(
                    "write GITS offset: {:#x}, 0x{:016x}",
                    mmio.address,
                    mmio.value
                );
            } else {
                trace!(
                    "read GITS offset: {:#x}, 0x{:016x}",
                    mmio.address,
                    mmio.value
                );
            }
        }
    }
    Ok(())
}
