// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.
use crate::device::common::MMIODerefWrapper;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
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

const GICR_SGI_BASE: usize = 0x10000;
//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

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
            gicr_registers1: SgiReg::new(mmio_start_addr + GICR_SGI_BASE),
        }
    }
    pub fn read_aff(&self) -> u64 {
        self.gicr_registers0.TYPER.get()
    }
}
