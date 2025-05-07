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
    pcibar::{BarRegion, PciBar, VirtPciBar}, phantom_cfg::{PhantomCfg, PhantomCfgType}, NUM_BAR_REGS_TYPE1, NUM_MAX_BARS
};

#[derive(Debug)]
pub struct BridgeConfig {
    bars: [PciBar; NUM_BAR_REGS_TYPE1],
    pub bdf: usize,
}

impl BridgeConfig {
    pub fn new(bdf: usize) -> Self {
        Self {
            bars: [PciBar::default(); NUM_BAR_REGS_TYPE1],
            bdf: bdf,
        }
    }

    pub fn bars_init(&mut self, bar_id: usize, origin_val: u32, val: u32) {
        self.bars[bar_id].init(origin_val, val);
    }

    pub fn get_regions(&self) -> Vec<BarRegion> {
        let mut regions: Vec<BarRegion> = Vec::new();
        let mut bar_id = 0;
        while bar_id < NUM_BAR_REGS_TYPE1 {
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

    // after we get bar regions, we should generate a virtual device instance that mirrors this device for use by other VMs
    pub fn generate_vbridge(&self) -> PhantomCfg {
        let mut v_bars: [VirtPciBar; NUM_MAX_BARS] = [VirtPciBar::default(); NUM_MAX_BARS];
        for i in 0..NUM_BAR_REGS_TYPE1 {
            v_bars[i] = self.bars[i].generate_vbar();
        }
        PhantomCfg::new(self.bdf, v_bars, PhantomCfgType::BRIDGE)
    }
}
