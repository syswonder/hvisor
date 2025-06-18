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
use core::ptr::read_volatile;

use super::{
    cfg_base, cfg_reg_addr,
    pcibar::{BarRegion, PciBar, VirtPciBar},
    phantom_cfg::{PhantomCfg, PhantomCfgType},
    CFG_CAP_PTR_OFF, CFG_EXT_CAP_ID, CFG_EXT_CAP_PTR_OFF, CFG_NEXT_EXT_CAP_OFF, CFG_SRIOV_CAP_ID,
    NUM_BAR_REGS_TYPE0, NUM_MAX_BARS,
};

#[derive(Debug)]
pub struct EndpointConfig {
    bars: [PciBar; NUM_BAR_REGS_TYPE0],
    pub bdf: usize,
    pub node_before_sriov: usize,
    pub node_after_sriov: usize,
}

impl EndpointConfig {
    pub fn new(bdf: usize) -> Self {
        let (bars, bdf) = { ([PciBar::default(); NUM_BAR_REGS_TYPE0], bdf) };
        let mut r = EndpointConfig {
            bars,
            bdf,
            node_before_sriov: 0xfff,
            node_after_sriov: 0xfff,
        };
        if r.ext_cap_exists() {
            r.find_sriov();
        }
        r
    }

    pub fn ext_cap_exists(&self) -> bool {
        let cap_ptr_addr = cfg_reg_addr(self.bdf, CFG_CAP_PTR_OFF);
        let mut cur_cap_ptr = unsafe { read_volatile(cap_ptr_addr as *const u8) } as usize;

        if cur_cap_ptr == 0 {
            return false;
        }

        while cur_cap_ptr != 0 {
            let cap_addr = cfg_reg_addr(self.bdf, cur_cap_ptr);
            let cap_val = unsafe { read_volatile(cap_addr as *const u16) };

            let cap_id = (cap_val & 0xff) as u8;
            let next_cap_ptr = ((cap_val >> 8) & 0xff) as usize;

            if (cap_id as usize) == CFG_EXT_CAP_ID {
                info!(
                    "{:x}:{:x}.{:x} is a PCI Express device!",
                    self.bdf >> 8,
                    (self.bdf >> 3) & 0b11111,
                    self.bdf & 0b111
                );
                return true;
            }

            cur_cap_ptr = next_cap_ptr;
        }

        false
    }

    pub fn find_sriov(&mut self) {
        info!("finding sriov");

        let mut prev_cap_ptr = 0;
        let mut curr_cap_ptr = CFG_EXT_CAP_PTR_OFF; // start from 0x100

        // init to invalid offset value, to check if we find sriov
        self.node_before_sriov = 0xfff;
        self.node_after_sriov = 0xfff;

        while curr_cap_ptr != 0 {
            let cap_addr = cfg_reg_addr(self.bdf, curr_cap_ptr);
            let cap_val = unsafe { read_volatile(cap_addr as *const u32) }; // each ext cap is 8 bytes

            let cap_id = (cap_val & 0xffff) as u16;
            let next_cap_ptr = ((cap_val >> 20) & 0xfff) as usize;

            if (cap_id as usize) == CFG_SRIOV_CAP_ID {
                self.node_before_sriov = prev_cap_ptr;
                self.node_after_sriov = next_cap_ptr;
                info!(
                    "{:x}:{:x}.{:x} SR-IOV off: {:#x}, prev_node_off: {:#x}, next_node_off: {:#x}",
                    self.bdf >> 8,
                    (self.bdf >> 3) & 0b11111,
                    self.bdf & 0b111,
                    curr_cap_ptr,
                    prev_cap_ptr,
                    next_cap_ptr
                );
                break;
            }

            prev_cap_ptr = curr_cap_ptr;
            curr_cap_ptr = next_cap_ptr;
        }
    }

    pub fn skip_sriov(&self, cur_cap_hdr: usize) -> usize {
        (cur_cap_hdr & 0x000fffff) | (self.node_after_sriov << CFG_NEXT_EXT_CAP_OFF)
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
    pub fn generate_vep(&self) -> PhantomCfg {
        let mut v_bars: [VirtPciBar; NUM_MAX_BARS] = [VirtPciBar::default(); NUM_MAX_BARS];
        for i in 0..NUM_BAR_REGS_TYPE0 {
            v_bars[i] = self.bars[i].generate_vbar();
        }
        PhantomCfg::new(self.bdf, v_bars, PhantomCfgType::ENDPOINT)
    }
}
