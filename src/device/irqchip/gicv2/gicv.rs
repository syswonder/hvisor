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
/// gicv layout definition and functions for gicv operations.
/// reference:
/// 1. gicv2 spec : https://www.cl.cam.ac.uk/research/srg/han/ACS-P35/zynq/arm_gic_architecture_specification.pdf
/// note: We don't actually use it. This is to ensure the integrity of GICV2.
use crate::device::irqchip::gicv2::gic_ref::GicRef;
use crate::device::irqchip::gicv2::gicc::GicCpuInterface;
use crate::device::irqchip::gicv2::GICV2;
use spin::Once;
// Each CPU holds one GICV, and it has the same register layout as GICC.
pub static GICV: Once<GicRef<GicCpuInterface>> = Once::new();

pub fn gicv_init(gicv_base: usize) {
    unsafe {
        GICV.call_once(|| GicRef::new(gicv_base as *const GicCpuInterface));
    }
}
