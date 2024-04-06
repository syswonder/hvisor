// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.

use crate::{arch::cpu::this_cpu_id, hypercall::SGI_IPI_ID};

use super::{
    gicd::{
        GICD_ICACTIVER, GICD_ICENABLER, GICD_ICFGR, GICD_ICPENDR, GICD_IGROUPR, GICD_IPRIORITYR,
        GICD_ISACTIVER, GICD_ISENABLER, GICD_ISPENDR,
    },
    host_gicr_base,
};

pub const GICR_CTLR: usize = 0x0000;
pub const GICR_IIDR: usize = 0x0004;
pub const GICR_TYPER: usize = 0x0008;
pub const GICR_STATUSR: usize = 0x0010;
pub const GICR_WAKER: usize = 0x0014;
pub const GICR_SYNCR: usize = 0x00c0;
pub const GICR_PIDR2: usize = 0xffe8;
pub const GICR_SGI_BASE: usize = 0x10000;

pub const GICR_IGROUPR: usize = GICD_IGROUPR;
pub const GICR_ISENABLER: usize = GICD_ISENABLER;
pub const GICR_ICENABLER: usize = GICD_ICENABLER;
pub const GICR_ISPENDR: usize = GICD_ISPENDR;
pub const GICR_ICPENDR: usize = GICD_ICPENDR;
pub const GICR_ISACTIVER: usize = GICD_ISACTIVER;
pub const GICR_ICACTIVER: usize = GICD_ICACTIVER;
pub const GICR_IPRIORITYR: usize = GICD_IPRIORITYR;
pub const GICR_ICFGR: usize = GICD_ICFGR;
pub const GICR_TYPER_LAST: usize = 1 << 4;

pub fn enable_ipi() {
    let base = host_gicr_base(this_cpu_id()) + GICR_SGI_BASE;

    unsafe {
        let gicr_waker = (base + GICR_WAKER) as *mut u32;
        gicr_waker.write_volatile(gicr_waker.read_volatile() & !0x02);
        while gicr_waker.read_volatile() & 0x04 != 0 {}

        let gicr_igroupr0 = (base + GICR_IGROUPR) as *mut u32;
        gicr_igroupr0.write_volatile(gicr_igroupr0.read_volatile() | (1 << SGI_IPI_ID));

        let gicr_isenabler0 = (base + GICR_ISENABLER) as *mut u32;
        gicr_isenabler0.write_volatile(1 << SGI_IPI_ID);

        let gicr_ipriorityr0 = (base + GICR_IPRIORITYR) as *mut u32;
        {
            let reg = SGI_IPI_ID / 4;
            let offset = SGI_IPI_ID % 4 * 8;
            let mask = ((1 << 8) - 1) << offset;
            let p = gicr_ipriorityr0.add(reg as _);
            let prio = p.read_volatile();

            p.write_volatile((prio & !mask) | (0x01 << offset));
        }
    }
}
