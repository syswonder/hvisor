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

#![allow(dead_code)]

use core::ptr::write_volatile;
use spin::Mutex;

use super::host_gicd_base;

pub static GICD_LOCK: Mutex<()> = Mutex::new(());

pub const GICD_CTLR: usize = 0x0000;
pub const GICD_CTLR_ARE_NS: usize = 1 << 5;
pub const GICD_CTLR_GRP1NS_ENA: usize = 1 << 1;

pub const GICD_TYPER: usize = 0x0004;
pub const GICD_IIDR: usize = 0x0008;
pub const GICD_TYPER2: usize = 0x000c;
pub const GICD_IGROUPR: usize = 0x0080;
pub const GICD_ISENABLER: usize = 0x0100;
pub const GICD_ICENABLER: usize = 0x0180;
pub const GICD_ISPENDR: usize = 0x0200;
pub const GICD_ICPENDR: usize = 0x0280;
pub const GICD_ISACTIVER: usize = 0x0300;
pub const GICD_ICACTIVER: usize = 0x0380;
pub const GICD_IPRIORITYR: usize = 0x0400;
pub const GICD_ITARGETSR: usize = 0x0800;
pub const GICD_ICFGR: usize = 0x0c00;
pub const GICD_NSACR: usize = 0x0e00;
pub const GICD_SGIR: usize = 0x0f00;
pub const GICD_CPENDSGIR: usize = 0x0f10;
pub const GICD_SPENDSGIR: usize = 0x0f20;
pub const GICD_IROUTER: usize = 0x6000;

pub const GICDV3_CIDR0: usize = 0xfff0;
pub const GICDV3_PIDR0: usize = 0xffe0;
pub const GICDV3_PIDR2: usize = 0xffe8;
pub const GICDV3_PIDR4: usize = 0xffd0;

pub fn enable_gic_are_ns() {
    unsafe {
        ((host_gicd_base() + GICD_CTLR) as *mut u32)
            .write_volatile(GICD_CTLR_ARE_NS as u32 | GICD_CTLR_GRP1NS_ENA as u32);
    }
}

pub fn set_ispender(index: usize, value: u32) {
    unsafe {
        write_volatile(
            (host_gicd_base() + GICD_ISPENDR + index * 4) as *mut u32,
            value,
        );
    }
}
