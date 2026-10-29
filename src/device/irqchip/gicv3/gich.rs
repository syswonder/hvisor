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
// GICv3 virtual CPU interface control (ICH_*_EL2) and List register values.
use crate::arch::aarch64::sysreg::{read_sysreg, write_sysreg};
use bit_field::BitField;

pub const ICH_HCR_UIE: u64 = 1 << 1;

/// An ICH_LR<n>_EL2 value.
#[derive(Clone, Copy)]
pub struct IchLr(pub u64);

impl IchLr {
    pub fn new_pending(vintid: usize) -> Self {
        let mut lr = Self(0);
        lr.0.set_bits(0..32, vintid as u64);
        lr.0.set_bit(60, true);
        lr.set_pending(true);
        lr
    }

    pub fn read(index: usize) -> Self {
        Self(read_lr(index))
    }

    pub fn write(self, index: usize) {
        write_lr(index, self.0);
    }

    pub fn vintid(self) -> usize {
        self.0.get_bits(0..32) as usize
    }

    pub fn is_pending(self) -> bool {
        self.0.get_bit(62)
    }

    pub fn set_pending(&mut self, pending: bool) {
        self.0.set_bit(62, pending);
    }

    pub fn is_active(self) -> bool {
        self.0.get_bit(63)
    }
}

/// Number of implemented List registers.
//
// ICH_VTR_EL2.ListRegs[4:0] holds that number minus one. GICv3 defines at
// most 16 (`ICH_LR0..15_EL2`), which is also all `read_lr()`/`write_lr()` can
// reach.
pub fn list_regs_num() -> usize {
    (read_sysreg!(ich_vtr_el2).get_bits(0..5) as usize + 1).min(16)
}

pub fn read_lr(id: usize) -> u64 {
    let id = id as u64;
    match id {
        //TODO get lr size from gic reg
        0 => read_sysreg!(ich_lr0_el2),
        1 => read_sysreg!(ich_lr1_el2),
        2 => read_sysreg!(ich_lr2_el2),
        3 => read_sysreg!(ich_lr3_el2),
        4 => read_sysreg!(ich_lr4_el2),
        5 => read_sysreg!(ich_lr5_el2),
        6 => read_sysreg!(ich_lr6_el2),
        7 => read_sysreg!(ich_lr7_el2),
        8 => read_sysreg!(ich_lr8_el2),
        9 => read_sysreg!(ich_lr9_el2),
        10 => read_sysreg!(ich_lr10_el2),
        11 => read_sysreg!(ich_lr11_el2),
        12 => read_sysreg!(ich_lr12_el2),
        13 => read_sysreg!(ich_lr13_el2),
        14 => read_sysreg!(ich_lr14_el2),
        15 => read_sysreg!(ich_lr15_el2),
        _ => {
            error!("lr over");
            loop {}
        }
    }
}

pub fn write_lr(id: usize, val: u64) {
    let id = id as u64;
    match id {
        0 => write_sysreg!(ich_lr0_el2, val),
        1 => write_sysreg!(ich_lr1_el2, val),
        2 => write_sysreg!(ich_lr2_el2, val),
        3 => write_sysreg!(ich_lr3_el2, val),
        4 => write_sysreg!(ich_lr4_el2, val),
        5 => write_sysreg!(ich_lr5_el2, val),
        6 => write_sysreg!(ich_lr6_el2, val),
        7 => write_sysreg!(ich_lr7_el2, val),
        8 => write_sysreg!(ich_lr8_el2, val),
        9 => write_sysreg!(ich_lr9_el2, val),
        10 => write_sysreg!(ich_lr10_el2, val),
        11 => write_sysreg!(ich_lr11_el2, val),
        12 => write_sysreg!(ich_lr12_el2, val),
        13 => write_sysreg!(ich_lr13_el2, val),
        14 => write_sysreg!(ich_lr14_el2, val),
        15 => write_sysreg!(ich_lr15_el2, val),
        _ => {
            error!("lr over");
            loop {}
        }
    }
}
