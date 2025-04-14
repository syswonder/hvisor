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
    pcibar::{BarRegion, PciBar, VirtPciBar},
    VirtPciDev, NUM_BAR_REGS_TYPE0, NUM_BAR_REGS_TYPE1,
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

    // after we get bar regions, we should generate a virtual device instance that mirrors this device for use by other VMs
    pub fn generate_vep(&self) -> VirtEndpointConfig {
        let mut v_bars: [VirtPciBar; NUM_BAR_REGS_TYPE0] =
            [VirtPciBar::default(); NUM_BAR_REGS_TYPE0];
        for i in 0..NUM_BAR_REGS_TYPE0 {
            v_bars[i] = self.bars[i].generate_vbar();
        }
        VirtEndpointConfig::new(self.bdf, v_bars)
    }
}

#[derive(Clone, Debug)]
pub struct VirtEndpointConfig {
    pub bdf: usize,
    command: u16,
    v_bars: [VirtPciBar; NUM_BAR_REGS_TYPE0],
}

impl VirtEndpointConfig {
    pub fn new(bdf: usize, v_bars: [VirtPciBar; NUM_BAR_REGS_TYPE0]) -> Self {
        Self {
            bdf,
            command: 0,
            v_bars: v_bars,
        }
    }
}

impl VirtPciDev for VirtEndpointConfig {
    fn read_bar(&self, bar_id: usize) -> u32 {
        self.v_bars[bar_id].read()
    }
    fn write_bar(&mut self, bar_id: usize, val: u32) {
        self.v_bars[bar_id].write(val as _);
    }
    fn read_cmd(&self) -> u16 {
        self.command
    }
    fn write_cmd(&mut self, command: u16) {
        self.command = command;
    }
}
