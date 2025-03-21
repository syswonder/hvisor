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
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::device::irqchip::gicv2::gic_ref::GicRef;
use crate::device::irqchip::gicv2::gicd::GICD;
use crate::device::irqchip::gicv2::gich::{
    GICH, GICV2_GICH_HCR_EN, GICV2_GICH_VMCR_PMR_SHIFT, GICV2_GICH_VMCR_VEM,
    GICV2_GICH_VMCR_VMGRP0EN,
};
use crate::device::irqchip::gicv2::GICV2;
/// gicc layout definition and functions for gicc operations.
/// author : ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

pub const GICV2_PREEMPPTION_MAX: usize = 128;

pub const GICV2_GICC_APR_REGS_NUM: usize = GICV2_PREEMPPTION_MAX / 32;

pub const GICV2_GICC_CTRL_GRPEN1: u32 = 0x1;
pub const GICV2_GICC_CTRL_EOIMODES: u32 = 0x1 << 9;
pub const GICV2_GICC_CTLR_REG_OFFSET: usize = 0x0000;
pub const GICV2_GICC_PMR_REG_OFFSET: usize = 0x0004;
pub const GICV2_GICC_BPR_REG_OFFSET: usize = 0x0008;
pub const GICV2_GICC_IAR_REG_OFFSET: usize = 0x000C;
pub const GICV2_GICC_EOIR_REG_OFFSET: usize = 0x0010;
pub const GICV2_GICC_RPR_REG_OFFSET: usize = 0x0014;
pub const GICV2_GICC_HPPIR_REG_OFFSET: usize = 0x0018;
pub const GICV2_GICC_ABPR_REG_OFFSET: usize = 0x001C;
pub const GICV2_GICC_AIAR_REG_OFFSET: usize = 0x0020;
pub const GICV2_GICC_AEOIR_REG_OFFSET: usize = 0x0024;
pub const GICV2_GICC_AHPPIR_REG_OFFSET: usize = 0x0028;
pub const GICV2_GICC_APR_REG_OFFSET: usize = 0x00D0;
pub const GICV2_GICC_NSAPR_REG_OFFSET: usize = 0x00E0;
pub const GICV2_GICC_IIDR_REG_OFFSET: usize = 0x00FC;
pub const GICV2_GICC_DIR_REG_OFFSET: usize = 0x1000;
pub const GICV2_GICC_END: usize = 0x2000;
pub const GICV2_GICC_PMR_DEFAULT: u32 = 0xf0;

// GICC Register layout.
register_structs! {
    #[allow(non_snake_case)]
    pub GicCpuInterface {
        (0x0000 => CTLR: ReadWrite<u32>),
        (0x0004 => PMR: ReadWrite<u32>),
        (0x0008 => BPR: ReadWrite<u32>),
        (0x000C => IAR: ReadOnly<u32>),
        (0x0010 => EOIR: WriteOnly<u32>),
        (0x0014 => RPR: ReadOnly<u32>),
        (0x0018 => HPPIR: ReadOnly<u32>),
        (0x001C => ABPR: ReadWrite<u32>),
        (0x0020 => AIAR: ReadOnly<u32>),
        (0x0024 => AEOIR: WriteOnly<u32>),
        (0x0028 => AHPPIR: ReadOnly<u32>),
        (0x002c => reserved0),
        (0x00D0 => APR: [ReadWrite<u32>; GICV2_GICC_APR_REGS_NUM]),
        (0x00E0 => NSAPR: [ReadWrite<u32>; GICV2_GICC_APR_REGS_NUM]),
        (0x00f0 => reserved1),
        (0x00FC => IIDR: ReadOnly<u32>),
        (0x0100 => reserved2),
        (0x1000 => DIR: WriteOnly<u32>),
        (0x1004 => reserved3),
        (0x2000 => @END),
    }
}
unsafe impl Sync for GicCpuInterface {}

// Each CPU holds one GICC.
pub static GICC: GicRef<GicCpuInterface> =
    unsafe { GicRef::new(GICV2.gicc_base as *const GicCpuInterface) };

impl GicCpuInterface {
    // init GICC for each CPU.
    pub fn init(&self) {
        // Ensure all SGIs disabled.
        GICD.set_icenabler(0, 0x0000FFFF);
        // get ctrl and pmr value
        let gicc_ctrl = self.CTLR.get();
        let gicc_pmr = self.PMR.get();
        // interrupt completion is divided into two steps to improve hypervisor performance.
        self.CTLR
            .set(GICV2_GICC_CTRL_GRPEN1 | GICV2_GICC_CTRL_EOIMODES);
        // Set the priority mask register to default value
        self.PMR.set(GICV2_GICC_PMR_DEFAULT);
        // VMCR only conyains 5 bits of priority
        let mut vmcr = (gicc_pmr >> 3) << GICV2_GICH_VMCR_PMR_SHIFT;
        // GICV layout equal to GICC without security extensions.
        if gicc_ctrl & GICV2_GICC_CTRL_GRPEN1 != 0 {
            vmcr |= GICV2_GICH_VMCR_VMGRP0EN;
        }
        if gicc_ctrl & GICV2_GICC_CTRL_EOIMODES != 0 {
            vmcr |= GICV2_GICH_VMCR_VEM;
        }
        GICH.set_vmcr(vmcr);
        // Enable virtual CPU interface operation.
        GICH.set_hcr(GICV2_GICH_HCR_EN);
        // Clear all lr registers in GICH.
        GICH.clear_all_lr();
        // Deactivate all active and pending SGIS
        let gicd_isactive = GICD.get_isactiver(0);
        let gicd_ispend = GICD.get_spendsgir(0);
        GICD.set_icactiver(0, gicd_isactive & 0xffff);
        GICD.set_cpendsgir(0, gicd_ispend & 0xffff);
        // re-enable all SGIs
        GICD.set_isenabler(0, 0x0000FFFF);
        info!("GICV2: GICC init done.");
    }

    pub fn get_iar(&self) -> u32 {
        self.IAR.get()
    }

    pub fn set_eoir(&self, value: u32) {
        self.EOIR.set(value);
    }

    pub fn set_dir(&self, value: u32) {
        self.DIR.set(value);
    }
}
