// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICD Driver - GIC Distributor.
//!
//! # Glossary
//!   - SPI - Shared Peripheral Interrupt.
#![allow(dead_code)]
use crate::{
    config::HvSystemConfig,
    device::common::MMIODerefWrapper,
    error::HvResult,
    memory::{mmio_perform_access, MMIOAccess},
    percpu::this_cell,
};
use spin::Mutex;
use tock_registers::{
    register_structs,
    registers::{ReadOnly, ReadWrite},
};

use super::is_spi;

register_structs! {
    #[allow(non_snake_case)]
    GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => IGROUPR: [ReadWrite<u32>; 32]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER: [ReadWrite<u32>;32]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER: [ReadWrite<u32>; 32]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => ISPENDR: [ReadWrite<u32>; 32]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR: [ReadWrite<u32>; 32]),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER: [ReadWrite<u32>;32]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER: [ReadWrite<u32>; 32]),
        /// Interrupt Priority Registers.
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 256]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => ITARGETSR: [ReadWrite<u32>; 256]),
        /// Interrupt Configuration Registers.
        (0x0c00 => ICFGR: [ReadWrite<u32>; 64]),
        (0x0d00 => _reserved_1),
        (0x0f04 => @END),
    }
}
/// Abstraction for the banked parts of the associated MMIO registers.
type DisReg = MMIODerefWrapper<GicDistributorRegs>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the GIC Distributor.
pub struct GICD {
    /// Access to shared registers is guarded with a lock.
    gicd_registers: DisReg,
}

impl GICD {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            gicd_registers: DisReg::new(mmio_start_addr),
        }
    }
}

static GICD_LOCK: Mutex<()> = Mutex::new(());

pub const GICD_CTLR: u64 = 0x0000;
pub const GICD_CTLR_ARE_NS: u64 = 1 << 4;
pub const GICD_TYPER: u64 = 0x0004;
pub const GICD_IIDR: u64 = 0x0008;
pub const GICD_IGROUPR: u64 = 0x0080;
pub const GICD_ISENABLER: u64 = 0x0100;
pub const GICD_ICENABLER: u64 = 0x0180;
pub const GICD_ISPENDR: u64 = 0x0200;
pub const GICD_ICPENDR: u64 = 0x0280;
pub const GICD_ISACTIVER: u64 = 0x0300;
pub const GICD_ICACTIVER: u64 = 0x0380;
pub const GICD_IPRIORITYR: u64 = 0x0400;
pub const GICD_ITARGETSR: u64 = 0x0800;
pub const GICD_ICFGR: u64 = 0x0c00;
pub const GICD_NSACR: u64 = 0x0e00;
pub const GICD_SGIR: u64 = 0x0f00;
pub const GICD_CPENDSGIR: u64 = 0x0f10;
pub const GICD_SPENDSGIR: u64 = 0x0f20;
pub const GICD_IROUTER: u64 = 0x6000;

const GICDV3_CIDR0: u64 = 0xfff0;
const GICDV3_PIDR0: u64 = 0xffe0;
const GICDV3_PIDR2: u64 = 0xffe8;
const GICDV3_PIDR4: u64 = 0xffd0;

fn reg_range(base: u64, n: u64, size: u64) -> core::ops::Range<u64> {
    base..(base + (n - 1) * size)
}

// The return value should be the register value to be read.
fn gicv3_handle_irq_ops(mmio: &mut MMIOAccess, irq: u32) -> HvResult {
    let cell = this_cell();
    let cell_r = cell.read();

    if !is_spi(irq) || !cell_r.irq_in_cell(irq) {
        // info!(
        //     "gicd-mmio: skip irq {} access, reg = {:#x?}",
        //     irq, mmio.address
        // );
        return Ok(());
    }

    mmio_perform_access(HvSystemConfig::get().platform_info.arch.gicd_base, mmio);

    Ok(())
}

fn gicd_misc_access(mmio: &mut MMIOAccess, gicd_base: u64) -> HvResult {
    let reg = mmio.address as u64;
    if reg_range(GICDV3_PIDR0, 4, 4).contains(&reg)
        || reg_range(GICDV3_PIDR4, 4, 4).contains(&reg)
        || reg_range(GICDV3_CIDR0, 4, 4).contains(&reg)
        || reg == GICD_CTLR
        || reg == GICD_TYPER
        || reg == GICD_IIDR
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

pub fn gicv3_gicd_mmio_handler(mmio: &mut MMIOAccess, _arg: u64) -> HvResult {
    // warn!("mmio = {:#x?}", mmio);
    let gicd_base = HvSystemConfig::get().platform_info.arch.gicd_base;
    let reg = mmio.address as u64;

    match reg {
        reg if reg_range(GICD_IROUTER, 1024, 8).contains(&reg) => {
            gicv3_handle_irq_ops(mmio, (reg - GICD_IROUTER) as u32 / 8)
        }
        reg if reg_range(GICD_ITARGETSR, 1024, 1).contains(&reg) => {
            gicv3_handle_irq_ops(mmio, (reg - GICD_ITARGETSR) as u32)
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
        _ => gicd_misc_access(mmio, gicd_base),
    }
}

fn restrict_bitmask_access(
    mmio: &mut MMIOAccess,
    reg_index: u64,
    bits_per_irq: u64,
    is_poke: bool,
    gicd_base: u64,
) -> HvResult {
    let cell = this_cell();
    let cell_r = cell.read();
    let mut access_mask: u64 = 0;
    /*
     * In order to avoid division, the number of bits per irq is limited
     * to powers of 2 for the moment.
     */
    let irqs_per_reg = 32 / bits_per_irq;
    let irq_bits = (1 << bits_per_irq) - 1;
    /* First, extract the first interrupt affected by this access */
    let first_irq = reg_index * irqs_per_reg;

    for irq in 0..irqs_per_reg {
        if cell_r.irq_in_cell((first_irq + irq) as _) {
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
