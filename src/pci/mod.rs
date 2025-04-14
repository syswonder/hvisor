use alloc::collections::btree_map::BTreeMap;
use endpoint::VirtEndpointConfig;
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
use spin::Once;

pub mod bridge;
pub mod endpoint;
pub mod pci;
pub mod pcibar;
pub mod phantom_cfg;

pub const CFG_CMD_OFF: usize = 0x4; //status
pub const CFG_CAP_PTR_OFF: usize = 0x34; // capabilities pointer
pub const CFG_CLASS_CODE_OFF: usize = 0x8; // 4 bytes, include revision and class code

pub const NUM_BAR_REGS_TYPE0: usize = 6;
pub const NUM_BAR_REGS_TYPE1: usize = 2;
pub const PHANTOM_DEV_HEADER: u32 = 0x77777777u32;

pub trait VirtPciDev {
    fn read_bar(&self, bar_id: usize) -> u32;
    fn write_bar(&mut self, bar_id: usize, val: u32);
    fn write_cmd(&mut self, val: u16);
    fn read_cmd(&self) -> u16;
}

pub static ECAM_BASE: Once<usize> = Once::new();

pub fn init_ecam_base(ecam_base: usize) {
    ECAM_BASE.call_once(|| ecam_base);
}

pub fn get_ecam_base() -> usize {
    *ECAM_BASE.get().unwrap() as _
}

pub fn cfg_base(bdf: usize, shift: usize) -> usize {
    if cfg!(all(target_arch = "loongarch64", feature = "pci")) && ((bdf >> 8) != 0){
        get_ecam_base() + (bdf << shift) + 0x1000_0000
    }else{
        get_ecam_base() + (bdf << shift)
    }
}

pub static mut VIRT_PCI_EPS: BTreeMap<usize, VirtEndpointConfig> = BTreeMap::new();

// In fact, if the vdev corresponding to the given BDF cannot be found, 
// it indicates that the device will only be used by subsequent VMs. 
// Therefore, minimal handling is required, 
// and it's not an issue if no further actions are taken.
pub fn add_virt_ep(bdf: usize, vep: VirtEndpointConfig) {
    unsafe {
        match VIRT_PCI_EPS.get(&bdf) {
            Some(_vep) => warn!("The BDF already has a virt config struct"),
            None => {
                VIRT_PCI_EPS.insert(bdf, vep.clone());
                info!("add a virt ep: {:#x?}", vep.clone());
            },
        }
    }
}
