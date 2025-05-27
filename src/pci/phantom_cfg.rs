use core::{ptr, usize};

use alloc::collections::btree_map::BTreeMap;

use crate::{
    error::HvResult,
    memory::{mmio_perform_access, MMIOAccess},
    pci::PHANTOM_DEV_HEADER,
    zone::this_zone_id,
};

use super::{
    cfg_base, endpoint::EndpointConfig, extract_reg_addr, pcibar::VirtPciBar, CFG_BAR0, CFG_BAR1,
    CFG_BAR2, CFG_BAR3, CFG_BAR4, CFG_BAR5, CFG_CAP_PTR_OFF, CFG_CLASS_CODE_OFF, CFG_CMD_OFF,
    CFG_EXT_CAP_PTR_OFF, CFG_INT_LINE, CFG_INT_PIN, CFG_IO_BASE, CFG_IO_BASE_UPPER16, CFG_IO_LIMIT,
    CFG_IO_LIMIT_UPPER16, CFG_MEM_BASE, CFG_MEM_LIMIT, CFG_PREF_BASE_UPPER32,
    CFG_PREF_LIMIT_UPPER32, CFG_PREF_MEM_BASE, CFG_PREF_MEM_LIMIT, CFG_PRIMARY_BUS,
    CFG_SECONDARY_BUS, NUM_BAR_REGS_TYPE0, NUM_BAR_REGS_TYPE1, NUM_MAX_BARS,
};

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PhantomCfgType {
    ENDPOINT,
    BRIDGE,
}

#[derive(Debug, Copy, Clone)]
pub struct PhantomCfg {
    pub bdf: usize,
    command: u16,
    status: u16,
    int_line: u8,
    v_bars: [VirtPciBar; NUM_MAX_BARS],
    bar_num: usize,
    cfg_type: PhantomCfgType,
}

impl PhantomCfg {
    pub fn new(bdf: usize, v_bars: [VirtPciBar; NUM_MAX_BARS], cfg_type: PhantomCfgType) -> Self {
        Self {
            bdf,
            command: 0,
            status: 0,
            int_line: 0,
            v_bars: v_bars,
            bar_num: if cfg_type == PhantomCfgType::ENDPOINT {
                NUM_BAR_REGS_TYPE0
            } else {
                NUM_BAR_REGS_TYPE1
            },
            cfg_type: cfg_type,
        }
    }

    pub fn read_bar(&self, bar_id: usize) -> u32 {
        if bar_id >= self.bar_num {
            panic!("bar {} doesn't exists!", bar_id);
        }
        self.v_bars[bar_id].read()
    }
    pub fn write_bar(&mut self, bar_id: usize, val: u32) {
        if bar_id >= self.bar_num {
            panic!("bar {} doesn't exists!", bar_id);
        }
        self.v_bars[bar_id].write(val as _);
    }
    pub fn read_cmd(&self) -> u16 {
        self.command
    }
    pub fn write_cmd(&mut self, command: u16) {
        self.command = command;
    }
    pub fn read_stats(&self) -> u16 {
        self.status
    }
    pub fn write_stats(&mut self, val: u16) {
        self.status = val;
    }
    pub fn read_int_line(&self) -> u8 {
        self.int_line
    }
    pub fn write_int_line(&mut self, val: u8) {
        self.int_line = val;
    }

    pub fn phantom_mmio_handler(
        &mut self,
        mmio: &mut MMIOAccess,
        base: usize,
        zone_id: usize,
    ) -> HvResult {
        match self.cfg_type {
            PhantomCfgType::ENDPOINT => self.phantom_ep_handler(mmio, base, zone_id),
            PhantomCfgType::BRIDGE => self.phantom_bridge_handler(mmio, base, zone_id),
        }
    }

    fn phantom_ep_handler(
        &mut self,
        mmio: &mut MMIOAccess,
        base: usize,
        zone_id: usize,
    ) -> HvResult {
        let reg_addr = extract_reg_addr(mmio.address);
        match reg_addr {
            0 => {
                // phantom device
                let header_addr = base + mmio.address;
                let bdf = self.bdf;
                let function = bdf & 0x7;
                let device = (bdf >> 3) & 0b11111;
                let bus = bdf >> 8;
                let header_val = unsafe { ptr::read_volatile(header_addr as *mut u32) };
                warn!(
                    "{:x}:{:x}.{:x} exists but we don't show it to vm {:x}:{:x}",
                    bus,
                    device,
                    function,
                    header_val & 0xffff,
                    (header_val >> 16) & 0xffff
                );
                mmio.value = PHANTOM_DEV_HEADER as _;
            }
            CFG_CMD_OFF => {
                if mmio.is_write {
                    self.write_cmd(mmio.value as _);
                } else {
                    mmio.value = self.read_cmd() as _;
                }
            }
            CFG_CAP_PTR_OFF => {
                // can't see any capabilities
                mmio.value = 0x0;
            }
            CFG_EXT_CAP_PTR_OFF => {
                mmio.value = 0x0;
            }
            CFG_CLASS_CODE_OFF => {
                mmio.value = 0x1f0000;
            }
            CFG_BAR0 => {
                if zone_id == 0 {
                    mmio_perform_access(base, mmio);
                }
                if mmio.is_write {
                    self.write_bar(0, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(0) as _;
                }
            }
            CFG_BAR1 => {
                if zone_id == 0 {
                    mmio_perform_access(base, mmio);
                }
                if mmio.is_write {
                    self.write_bar(1, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(1) as _;
                }
            }
            CFG_BAR2 => {
                if zone_id == 0 {
                    mmio_perform_access(base, mmio);
                }
                if mmio.is_write {
                    self.write_bar(2, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(2) as _;
                }
            }
            CFG_BAR3 => {
                if zone_id == 0 {
                    mmio_perform_access(base, mmio);
                }
                if mmio.is_write {
                    self.write_bar(3, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(3) as _;
                }
            }
            CFG_BAR4 => {
                if zone_id == 0 {
                    mmio_perform_access(base, mmio);
                }
                if mmio.is_write {
                    self.write_bar(4, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(4) as _;
                }
            }
            CFG_BAR5 => {
                if zone_id == 0 {
                    mmio_perform_access(base, mmio);
                }
                if mmio.is_write {
                    self.write_bar(5, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(5) as _;
                }
            }
            CFG_INT_PIN => {
                mmio.value = 0;
            }
            CFG_INT_LINE => {
                if mmio.is_write {
                    self.write_int_line(mmio.value as _);
                } else {
                    mmio.value = self.read_int_line() as _;
                }
            }
            _ => {
                mmio_perform_access(base, mmio);
                // if self.bdf >> 8 == 8 {
                //     info!(
                //         "{:x}:{:x}.{:x} access {:#x} {:?} {} -> {:#x}",
                //         self.bdf >> 8,
                //         (self.bdf >> 3) & 0b11111,
                //         self.bdf & 0b111,
                //         reg_addr,
                //         if mmio.is_write {"W"} else {"R"},
                //         mmio.size,
                //         mmio.value
                //     );
                // }
            }
        }
        Ok(())
    }

    fn phantom_bridge_handler(
        &mut self,
        mmio: &mut MMIOAccess,
        base: usize,
        _zone_id: usize,
    ) -> HvResult {
        let reg_addr = extract_reg_addr(mmio.address);
        match reg_addr {
            0 => {
                // phantom device
                let header_addr = base + mmio.address;
                let header_val = unsafe { ptr::read_volatile(header_addr as *mut u32) };
                let bdf = self.bdf;
                let function = bdf & 0x7;
                let device = (bdf >> 3) & 0b11111;
                let bus = bdf >> 8;
                warn!(
                    "{:x}:{:x}.{:x} exists but we don't show it to vm {:x}:{:x}",
                    bdf >> 8,
                    device,
                    function,
                    header_val & 0xffff,
                    (header_val >> 16) & 0xffff
                );
                mmio.value = PHANTOM_DEV_HEADER as _;
            }
            CFG_CMD_OFF => {
                if mmio.is_write {
                    self.write_cmd(mmio.value as _);
                } else {
                    mmio.value = self.read_cmd() as _;
                }
            }
            CFG_CAP_PTR_OFF => {
                // can't see any capabilities
                mmio.value = 0x0;
            }
            CFG_EXT_CAP_PTR_OFF => {
                mmio.value = 0x0;
            }
            CFG_BAR0 => {
                if mmio.is_write {
                    self.write_bar(0, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(0) as _;
                }
            }
            CFG_BAR1 => {
                if mmio.is_write {
                    self.write_bar(1, mmio.value as _);
                } else {
                    mmio.value = self.read_bar(1) as _;
                }
            }
            CFG_INT_PIN => {
                mmio.value = 0;
            }
            CFG_INT_LINE => {
                if mmio.is_write {
                    self.write_int_line(mmio.value as _);
                } else {
                    mmio.value = self.read_int_line() as _;
                }
            }
            _ => {
                // if !mmio.is_write {
                mmio_perform_access(base, mmio);
                // }
            }
        }
        Ok(())
    }
}

pub static mut PHANTOM_DEVS: BTreeMap<usize, PhantomCfg> = BTreeMap::new();

pub fn add_phantom_devices(phantom_dev: PhantomCfg) {
    unsafe {
        let bdf = phantom_dev.bdf;
        if !PHANTOM_DEVS.contains_key(&bdf) {
            info!(
                "Add a new virt pci device: {:x}:{:x}.{:x}",
                &phantom_dev.bdf >> 8,
                (&phantom_dev.bdf >> 3) & 0b11111,
                &phantom_dev.bdf & 0b111
            );
            PHANTOM_DEVS.insert(bdf, phantom_dev);
        } else {
            warn!(
                "Phantom device with BDF {:#x} already exists, skipping",
                bdf
            );
        }
    }
}

pub fn find_phantom_dev(bdf: usize) -> PhantomCfg {
    unsafe {
        match PHANTOM_DEVS.get(&bdf) {
            Some(device) => device.clone(),
            None => generate_vep_by_bdf(bdf), // root will generate all virt bridges so we don't need to actively generate vbridges
        }
    }
}

pub fn generate_vep_by_bdf(bdf: usize) -> PhantomCfg {
    let mut tmp_ep = EndpointConfig::new(bdf);
    let cfg_base = cfg_base(bdf);
    let offsets: [usize; NUM_BAR_REGS_TYPE0] = [0x10, 0x14, 0x18, 0x1c, 0x20, 0x24];
    for bar_id in 0..NUM_BAR_REGS_TYPE0 {
        unsafe {
            let reg_ptr = (cfg_base + offsets[bar_id]) as *mut u32;
            let origin_val = *reg_ptr;
            *reg_ptr = 0xffffffffu32;
            let new_val = *reg_ptr;
            tmp_ep.bars_init(bar_id, origin_val, new_val);
            *reg_ptr = origin_val;
        }
    }
    let pdev = tmp_ep.generate_vep();
    add_phantom_devices(pdev);
    // info!("generate a pdev: {:#x?}", &pdev);
    pdev
}
