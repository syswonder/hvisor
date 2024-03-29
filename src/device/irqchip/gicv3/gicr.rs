// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.

use super::gicd::{
    GICD_ICACTIVER, GICD_ICENABLER, GICD_ICFGR, GICD_ICPENDR, GICD_IGROUPR, GICD_IPRIORITYR,
    GICD_ISACTIVER, GICD_ISENABLER, GICD_ISPENDR,
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

#[allow(unused)]
pub fn enable_ipi() {
    todo!();
    // let base = this_cpu_data().gicr_base + GICR_SGI_BASE;

    // unsafe {
    //     // 配置 IPI 为非安全中断
    //     let gicr_igroupr0 = (base + GICR_IGROUPR) as *mut u32;
    //     gicr_igroupr0.write_volatile(gicr_igroupr0.read_volatile() & !(1 << SGI_EVENT_ID));

    //     // 启用 IPI
    //     let gicr_isenabler0 = (base + GICR_ISENABLER) as *mut u32;
    //     gicr_isenabler0.write_volatile(1 << SGI_EVENT_ID);

    //     let gicr_ipriorityr0 = (base + GICR_IPRIORITYR) as *mut u32;
    //     {
    //         let reg = SGI_EVENT_ID / 4;
    //         let offset = SGI_EVENT_ID % 4 * 8;
    //         let mask = ((1 << 8) - 1) << offset;
    //         let p = gicr_ipriorityr0.add(reg as _);
    //         let prio = p.read_volatile();

    //         p.write_volatile((prio & !mask) | (0xa0 << offset));
    //     }

    //     let gicr_waker = (base + GICR_WAKER) as *mut u32;
    //     gicr_waker.write_volatile(gicr_waker.read_volatile() & !0x02);
    // }
}

