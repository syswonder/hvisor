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
use alloc::vec::Vec;

use super::{
    pcibar::{BarRegion, PciBar},
    NUM_BAR_REGS_TYPE0,
};

#[derive(Debug)]
pub struct EndpointConfig {
    bars: [PciBar; NUM_BAR_REGS_TYPE0],
    pub bdf: usize,
}

impl EndpointConfig {
    pub fn new(bdf: usize) -> Self {
        let (bars, bdf) = { ([PciBar::default(); NUM_BAR_REGS_TYPE0], bdf) };
        let r = EndpointConfig { bars, bdf };
        r
    }

    pub fn bars_init(&mut self, bar_id: usize, origin_val: u32, val: u32) {
        self.bars[bar_id].init(origin_val, val);
    }

    pub fn get_regions(&self) -> Vec<BarRegion> {
        let mut regions: Vec<BarRegion> = Vec::new();
        let mut bar_id = 0;
        while bar_id < NUM_BAR_REGS_TYPE0 {
            if self.bars[bar_id].is_mutable() {
                if !self.bars[bar_id].mem_type_64() {
                    regions.push(self.bars[bar_id].get_32b_region());
                    bar_id += 1;
                } else {
                    regions.push(
                        self.bars[bar_id + 1].get_64b_region(self.bars[bar_id].get_32b_region()),
                    );
                    bar_id += 2;
                }
            } else {
                bar_id += 1;
            }
        }
        regions
    }

    // pub fn ep_cfg_access(&mut self, mmio: &mut MMIOAccess){
    //     let bar_id = match mmio.address & 0xfff {
    //         0x10 => 0,
    //         0x14 => 1,
    //         0x18 => 2,
    //         0x1c => 3,
    //         0x20 => 4,
    //         0x24 => 5,
    //         _ => 0,
    //     };
    //     match mmio.is_write {
    //         true => {
    //             self.bars[bar_id].write(mmio.value as _);
    //         },
    //         false => {
    //             mmio.value = self.bars[bar_id].read() as _;
    //         }
    //     }
    // }
}
