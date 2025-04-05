// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! GICC Driver - GIC CPU interface.

use core::ptr;

use alloc::vec::Vec;
use spin::{mutex::Mutex, Once};

use crate::{
    arch::cpu::this_cpu_id,
    consts,
    consts::{MAX_CPU_NUM, MAX_ZONE_NUM, PAGE_SIZE},
    hypercall::SGI_IPI_ID,
    memory::Frame,
    zone::this_zone_id,
};

use super::{
    gicd::{
        GICD_ICACTIVER, GICD_ICENABLER, GICD_ICFGR, GICD_ICPENDR, GICD_IGROUPR, GICD_IPRIORITYR,
        GICD_ISACTIVER, GICD_ISENABLER, GICD_ISPENDR, GICD_TYPER,
    },
    host_gicd_base, host_gicr_base, MAINTENACE_INTERRUPT,
};

pub const GICR_CTLR: usize = 0x0000;
pub const GICR_IIDR: usize = 0x0004;
pub const GICR_TYPER: usize = 0x0008;
pub const GICR_STATUSR: usize = 0x0010;
pub const GICR_WAKER: usize = 0x0014;
pub const GICR_SETLPIR: usize = 0x0040;
pub const GICR_CLRLPIR: usize = 0x0048;
pub const GICR_INVLPIR: usize = 0x00a0;
pub const GICR_INVALLR: usize = 0x00b0;
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

pub const GICR_PROPBASER: usize = 0x0070;
pub const GICR_PENDBASER: usize = 0x0078;

pub fn enable_ipi() {
    let base = host_gicr_base(this_cpu_id()) + GICR_SGI_BASE;

    unsafe {
        let gicr_waker = (base + GICR_WAKER) as *mut u32;
        gicr_waker.write_volatile(gicr_waker.read_volatile() & !0x02);
        while gicr_waker.read_volatile() & 0x04 != 0 {}

        let gicr_igroupr0 = (base + GICR_IGROUPR) as *mut u32;
        gicr_igroupr0.write_volatile(gicr_igroupr0.read_volatile() | (1 << SGI_IPI_ID));

        let gicr_isenabler0 = (base + GICR_ISENABLER) as *mut u32;
        gicr_isenabler0.write_volatile(1 << SGI_IPI_ID | 1 << MAINTENACE_INTERRUPT);
        trace!("gicr_isenabler0: {}", gicr_isenabler0.read_volatile());
        let gicr_ipriorityr0 = (base + GICR_IPRIORITYR) as *mut u32;
        for irq_id in [SGI_IPI_ID, MAINTENACE_INTERRUPT] {
            let reg = irq_id / 4;
            let offset = irq_id % 4 * 8;
            let mask = ((1 << 8) - 1) << offset;
            let p = gicr_ipriorityr0.add(reg as _);
            let prio = p.read_volatile();

            p.write_volatile((prio & !mask) | (0x01 << offset));
        }
    }
}

pub struct LpiPropTable {
    phy_addr: usize,
    frame: Frame,
    baser_list: [usize; MAX_ZONE_NUM],
}

impl LpiPropTable {
    fn new() -> Self {
        let gicd_typer =
            unsafe { ptr::read_volatile((host_gicd_base() + GICD_TYPER) as *const u32) };
        let id_bits = (gicd_typer >> 19) & 0x1f;
        let page_num: usize = ((1 << (id_bits + 1)) - 8192) / PAGE_SIZE;
        let f = Frame::new_contiguous(page_num, 0).unwrap();
        let propreg = f.start_paddr() | 0x78f;
        for id in 0..unsafe {consts::NCPU} {
            let propbaser = host_gicr_base(id) + GICR_PROPBASER;
            unsafe {
                ptr::write_volatile(propbaser as *mut u64, propreg as _);
            }
        }
        Self {
            phy_addr: f.start_paddr(),
            frame: f,
            baser_list: [0; MAX_ZONE_NUM],
        }
    }

    fn set_prop_baser(&mut self, zone_id: usize, value: usize) {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.baser_list[zone_id] = value;
    }

    fn read_prop_baser(&self, zone_id: usize) -> usize {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.baser_list[zone_id]
    }

    fn enable_one_lpi(&self, lpi: usize) {
        let addr = self.phy_addr + lpi;
        let val: u8 = 0b1;
        // no priority
        unsafe {
            ptr::write_volatile(addr as *mut u8, val as _);
        }
    }
}

pub static LPT: Once<Mutex<LpiPropTable>> = Once::new();

pub fn init_lpi_prop() {
    LPT.call_once(|| Mutex::new(LpiPropTable::new()));
}

pub fn set_prop_baser(value: usize) {
    let mut lpt = LPT.get().unwrap().lock();
    lpt.set_prop_baser(this_zone_id(), value);
}

pub fn read_prop_baser() -> usize {
    let lpt = LPT.get().unwrap().lock();
    lpt.read_prop_baser(this_zone_id())
}

pub fn enable_one_lpi(lpi: usize) {
    let lpt = LPT.get().unwrap().lock();
    lpt.enable_one_lpi(lpi);
}
