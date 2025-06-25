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
use spin::Once;
use crate::device::irqchip::gicv2::gic::MAX_CPU_NUM;
/// The outer layer is defined using gicv2.
/// author: ForeverYolo
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
use crate::device::irqchip::gicv2::gicc::GICC;
use crate::device::irqchip::gicv2::gicd::GICD;
use crate::device::irqchip::gicv2::gicd::gicd_init;
use crate::device::irqchip::gicv2::gicc::gicc_init;
use crate::device::irqchip::gicv2::gich::gich_init;
use crate::device::irqchip::gicv2::gicv::gicv_init;
use crate::platform::ROOT_ARCH_ZONE_CONFIG;
use crate::zone::Zone;
use crate::arch::zone::{HvArchZoneConfig,GicConfig,Gicv2Config};
// GIC Distributor Definition.
pub mod gicd;

// GIC CPU Interface Definition.
pub mod gicc;

// Virtual GIC realization.
pub mod vgic;

// Physical GIC realization.
pub mod gic;

// GIC Hypervisor Interface Definition.
pub mod gich;

// GIC Virtual CPU Interface Definition.
pub mod gicv;

// GIC Reference warp.
mod gic_ref;

pub struct Gicv2 {
    gicd_base: usize,
    gicc_base: usize,
    gich_base: usize,
    gicv_base: usize,
}

pub static GICV2: Once<Gicv2> = Once::new();
// get base address of GIC and initialize GIC Structs.
pub fn primary_init_early() {
    match ROOT_ARCH_ZONE_CONFIG.gic_config {
        GicConfig::Gicv3(_) => {
            panic!("GICv3 is not supported in this version of hvisor");
        }
        GicConfig::Gicv2(ref gicv2_config) => {
            if ROOT_ARCH_ZONE_CONFIG.gic_version != 2 {
                panic!("GIC version mismatch, expected 2, got {}", ROOT_ARCH_ZONE_CONFIG.gic_version);
            }
            info!("GICv2 detected");
            GICV2.call_once(|| {
                Gicv2 {
                    gicd_base: gicv2_config.gicd_base,
                    gicc_base: gicv2_config.gicc_base + gicv2_config.gicc_offset,
                    gich_base: gicv2_config.gich_base,
                    gicv_base: gicv2_config.gicv_base,
                }
            });
            gicd_init(gicv2_config.gicd_base);
            gicc_init(gicv2_config.gicc_base + gicv2_config.gicc_offset);
            gich_init(gicv2_config.gich_base);
            gicv_init(gicv2_config.gicv_base);
            info!("GIC Distributor base: {:#x}, size: {:#x}", GICV2.get().unwrap().gicd_base, gicv2_config.gicd_size);
            info!("GIC CPU Interface base: {:#x}, size: {:#x}", GICV2.get().unwrap().gicc_base, gicv2_config.gicc_size);
            info!("GIC CPU Interface offset: {:#x}", gicv2_config.gicc_offset);
            info!("GIC Hypervisor Interface base: {:#x}, size: {:#x}", GICV2.get().unwrap().gich_base, gicv2_config.gich_size);
            info!("GIC Virtual CPU Interface base: {:#x}, size: {:#x}", GICV2.get().unwrap().gicv_base, gicv2_config.gicv_size);
        }
    };
    gic::PENDING_VIRQS.call_once(|| gic::PendingIrqs::new(MAX_CPU_NUM));
}

pub fn percpu_init() {
    GICC.get().unwrap().init();
}

pub fn primary_init_late() {
    GICD.get().unwrap().global_init();
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        // todo
        panic!("todo: arch_irqchip_reset")
    }
}
