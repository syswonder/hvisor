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
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::device::irqchip::gicv2::gic_ref::GicRef;
use crate::device::irqchip::gicv2::GICV2;
/// gich layout definition and functions for gich operations.
/// author : ForeverYolo
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};
use spin::Once;
pub const GICV2_MAX_LIST_REGS_NUM: usize = 64;
pub const GICV2_GICH_HCR_EN: u32 = 0x1;
pub const GICV2_GICH_VMCR_VEM: u32 = 0x1 << 9;
pub const GICV2_GICH_VMCR_VMGRP1EN: u32 = 0x1 << 1;
pub const GICV2_GICH_VMCR_VMGRP0EN: u32 = 0x1;
pub const GICV2_GICH_LR_GRP1: u32 = 0x1 << 30;
pub const GICV2_GICH_LR_PENDING_STATE: usize = 0x1 << 28;
pub const GICV2_GICH_LR_HW: usize = 0x1 << 31;
pub const GICV2_GICH_HCR_REG_OFFSET: usize = 0x0000;
pub const GICV2_GICH_VTR_REG_OFFSET: usize = 0x0004;
pub const GICV2_GICH_VMCR_REG_OFFSET: usize = 0x0008;
pub const GICV2_GICH_MISR_REG_OFFSET: usize = 0x0010;
pub const GICV2_GICH_EISR_REG_OFFSET: usize = 0x0020;
pub const GICV2_GICH_ELRSR_REG_OFFSET: usize = 0x0030;
pub const GICV2_GICH_APR_REG_OFFSET: usize = 0x00f0;
pub const GICV2_GICH_LR_REG_OFFSET: usize = 0x0100;
pub const GICV2_GICH_END: usize = 0x1000;
pub const GICV2_GICH_VMCR_PMR_SHIFT: u32 = 27;
pub const GICV2_GICH_LR_CPUID_SHIFT: u32 = 10;
pub const GICV2_GICH_LR_PHYSID_SHIFT: u32 = 10;
pub const GICV2_GICH_HCR_UIE: u32 = 0x1 << 1;

// GICH Register layout.
register_structs! {
    #[allow(non_snake_case)]
    pub GicHypervisorInterface {
        (0x0000 => HCR: ReadWrite<u32>),
        (0x0004 => VTR: ReadOnly<u32>),
        (0x0008 => VMCR: ReadWrite<u32>),
        (0x000c => reserve0),
        (0x0010 => MISR: ReadOnly<u32>),
        (0x0014 => reserve1),
        (0x0020 => EISR: [ReadOnly<u32>; GICV2_MAX_LIST_REGS_NUM / 32]),
        (0x0028 => reserve2),
        (0x0030 => ELRSR: [ReadOnly<u32>; GICV2_MAX_LIST_REGS_NUM / 32]),
        (0x0038 => reserve3),
        (0x00f0 => APR: ReadWrite<u32>),
        (0x00f4 => reserve4),
        (0x0100 => LR: [ReadWrite<u32>; GICV2_MAX_LIST_REGS_NUM]),
        (0x0200 => reserve5),
        (0x1000 => @END),
    }
}
unsafe impl Sync for GicHypervisorInterface {}
// Each CPU holds one GICH.
pub static GICH: Once<GicRef<GicHypervisorInterface>> = Once::new();
    // unsafe { GicRef::new(GICV2.gich_base as *const GicHypervisorInterface) };

pub fn gich_init(gich_base: usize) {
    unsafe {
        GICH.call_once(|| {
            GicRef::new(gich_base as *const GicHypervisorInterface)
        });
    }

}


impl GicHypervisorInterface {
    // init GICH for each CPU.
    pub fn get_lrs_num(&self) -> u32 {
        let vtr = self.VTR.get();
        (vtr & 0b11111) + 1
    }

    pub fn clear_all_lr(&self) {
        for i in 0..self.get_lrs_num() as usize {
            self.LR[i].set(0);
        }
        self.APR.set(0);
    }

    pub fn set_hcr(&self, value: u32) {
        self.HCR.set(value);
    }

    pub fn set_vmcr(&self, value: u32) {
        self.VMCR.set(value);
    }

    pub fn get_lr_num(&self) -> u32 {
        self.VTR.get() & 0b11111
    }

    pub fn get_elrsr(&self, index: usize) -> u32 {
        self.ELRSR[index].get()
    }

    pub fn get_lr(&self, index: usize) -> u32 {
        self.LR[index].get()
    }

    pub fn set_lr(&self, index: usize, value: u32) {
        self.LR[index].set(value);
    }

    pub fn get_hcr(&self) -> u32 {
        self.HCR.get()
    }
}
