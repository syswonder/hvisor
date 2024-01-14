// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.
#![allow(dead_code)]
use crate::{
    error::HvResult,
    memory::{mmio_perform_access, MMIOAccess},
    percpu::{get_cpu_data, this_cell, this_cpu_data}, hypercall::SGI_EVENT_ID,
};
use alloc::sync::Arc;

use super::gicd::{
    GICD_ICACTIVER, GICD_ICENABLER, GICD_ICFGR, GICD_ICPENDR, GICD_IPRIORITYR, GICD_ISACTIVER,
    GICD_ISENABLER, GICD_ISPENDR, GICD_IGROUPR,
};

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

const GICR_IGROUPR: u64 = GICD_IGROUPR;
const GICR_ISENABLER: u64 = GICD_ISENABLER;
const GICR_ICENABLER: u64 = GICD_ICENABLER;
const GICR_ISPENDR: u64 = GICD_ISPENDR;
const GICR_ICPENDR: u64 = GICD_ICPENDR;
const GICR_ISACTIVER: u64 = GICD_ISACTIVER;
const GICR_ICACTIVER: u64 = GICD_ICACTIVER;
const GICR_IPRIORITYR: u64 = GICD_IPRIORITYR;
const GICR_ICFGR: u64 = GICD_ICFGR;
const GICR_TYPER_LAST: u64 = 1 << 4;

pub fn enable_ipi() {
    let base = this_cpu_data().gicr_base + GICR_SGI_BASE;

    unsafe {
        // 配置 IPI 为非安全中断
        let gicr_igroupr0 = (base + GICR_IGROUPR) as *mut u32;
        gicr_igroupr0.write_volatile(gicr_igroupr0.read_volatile() & !(1 << SGI_EVENT_ID));

        // 启用 IPI
        let gicr_isenabler0 = (base + GICR_ISENABLER) as *mut u32;
        gicr_isenabler0.write_volatile(1 << SGI_EVENT_ID);

        let gicr_ipriorityr0 = (base + GICR_IPRIORITYR) as *mut u32;
        {
            let reg = SGI_EVENT_ID / 4;
            let offset = SGI_EVENT_ID % 4 * 8;
            let mask = ((1 << 8) - 1) << offset;
            info!("reg={}, offset={}", reg, offset);
            let p = gicr_ipriorityr0.add(reg as _);
            let prio = p.read_volatile();
            info!("p={:#x?}, prio={:#x?}, new={:#x?}", p, prio, (prio & !mask) | (0xa0 << offset));

            p.write_volatile((prio & !mask) | (0xa0 << offset));
        }

        let gicr_waker = (base + GICR_WAKER) as *mut u32;
        gicr_waker.write_volatile(gicr_waker.read_volatile() & !0x02);
    }
}

pub fn gicv3_gicr_mmio_handler(mmio: &mut MMIOAccess, cpu: u64) -> HvResult {
    info!("gicr({}) mmio = {:#x?}", cpu, mmio);
    let gicr_base = get_cpu_data(cpu).gicr_base;
    match mmio.address as u64 {
        GICR_TYPER => {
            mmio_perform_access(gicr_base, mmio);
            if cpu == this_cell().read().max_cpu_id {
                debug!("this is the last gicr");
                mmio.value |= GICR_TYPER_LAST;
            }
        }
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
            } else {
                trace!("*** gicv3_gicr_mmio_handler: ignore access to foreign redistributors ***");
            }
        }
    }
    HvResult::Ok(())
}
