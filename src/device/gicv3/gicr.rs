// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.
#![allow(dead_code)]
use crate::{
    device::common::MMIODerefWrapper,
    error::HvResult,
    memory::{mmio_perform_access, MMIOAccess},
    percpu::{get_cpu_data, this_cell},
};
use alloc::sync::Arc;
use tock_registers::{
    interfaces::Readable,
    register_structs,
    registers::{ReadOnly, ReadWrite},
};

use super::gicd::{
    GICD_ICACTIVER, GICD_ICENABLER, GICD_ICFGR, GICD_ICPENDR, GICD_IPRIORITYR, GICD_ISACTIVER,
    GICD_ISENABLER, GICD_ISPENDR,
};

register_structs! {
    #[allow(non_snake_case)]
    GicRedistributorLPIRegs {
        /// Redistributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Implementer Identification Register.
        (0x0004 => IIDR: ReadOnly<u32>),
        /// Redistributor Type Register.
        (0x0008 => TYPER: ReadOnly<u64>),
        /// Error Reporting Status Register, optional
        (0x0010 => STATUSR: ReadOnly<u32>),
        /// Redistributor Wake Register
        (0x0014 => WAKER: ReadWrite<u32>),
        (0x0018 => _reserved_0),
        /// Redistributor Synchronize Register
        (0x00c0=> SYNCR: ReadOnly<u32>),
        (0x00c4 => _reserved_1),
        /// Redistributor Peripheral ID2 Register
        (0xffe8=> PIDR2: ReadOnly<u32>),

        (0xffec => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    GicRedistributorSGIRegs {
        (0x0000 => _reserved_0),
        /// Interrupt Group Register 0.
        (0x0080 => IGROUPR0: ReadWrite<u32>),
        (0x0084 => _reserved_1),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER0: ReadWrite<u32>),
        (0x0104 => _reserved_2),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER0: ReadWrite<u32>),
        (0x0184 => _reserved_3),
        /// , Interrupt Set-Pending Register.
        (0x0200 => ISPENDR0: ReadWrite<u32>),
        (0x0204 => _reserved_4),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR0: ReadWrite<u32>),
        (0x0284 => _reserved_5),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER0: ReadWrite<u32>),
        (0x0304 => _reserved_6),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER0: ReadWrite<u32>),

        (0x0384 => @END),
    }
}

/// Abstraction for the non-banked parts of the associated MMIO registers.
type LpiReg = MMIODerefWrapper<GicRedistributorLPIRegs>;

/// Abstraction for the banked parts of the associated MMIO registers.
type SgiReg = MMIODerefWrapper<GicRedistributorSGIRegs>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

const GICR_CTLR: u64 = 0x0000;
const GICR_IIDR: u64 = 0x0004;
const GICR_TYPER: u64 = 0x0008;
const GICR_STATUSR: u64 = 0x0010;
const GICR_WAKER: u64 = 0x0014;
const GICR_SYNCR: u64 = 0x00c0;
const GICR_PIDR2: u64 = 0xffe8;
const GICR_SGI_BASE: u64 = 0x10000;

const GICR_ISENABLER: u64 = GICD_ISENABLER;
const GICR_ICENABLER: u64 = GICD_ICENABLER;
const GICR_ISPENDR: u64 = GICD_ISPENDR;
const GICR_ICPENDR: u64 = GICD_ICPENDR;
const GICR_ISACTIVER: u64 = GICD_ISACTIVER;
const GICR_ICACTIVER: u64 = GICD_ICACTIVER;
const GICR_IPRIORITYR: u64 = GICD_IPRIORITYR;
const GICR_ICFGR: u64 = GICD_ICFGR;

/// Representation of the GIC Distributor.
pub struct GICR {
    /// Access to shared registers is guarded with a lock.
    gicr_registers0: LpiReg,

    /// Access to banked registers is unguarded.
    gicr_registers1: SgiReg,
}
impl GICR {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            gicr_registers0: LpiReg::new(mmio_start_addr),
            gicr_registers1: SgiReg::new(mmio_start_addr + GICR_SGI_BASE as usize),
        }
    }
    pub fn read_aff(&self) -> u64 {
        self.gicr_registers0.TYPER.get()
    }
}

pub fn gicv3_gicr_mmio_handler(mmio: &mut MMIOAccess, cpu: u64) -> HvResult {
    // info!("gicr({}) mmio = {:#x?}", cpu, mmio);
    let gicr_base = get_cpu_data(cpu).gicr_base;
    match mmio.address as u64 {
        GICR_IIDR | 0xffd0..=0xfffc => {
            // Read-only registers that might be used by a cell to find the redistributor corresponding to a CPU. Keep them accessible.
            mmio_perform_access(gicr_base, mmio);
        }
        GICR_SYNCR => {
            mmio.value = 0;
        }
        _ => {
            if Arc::ptr_eq(&this_cell(), get_cpu_data(cpu).cell.as_ref().unwrap()) {
                // ignore access to foreign redistributors
                mmio_perform_access(gicr_base, mmio);
            }
        }
    }
    HvResult::Ok(())
}
