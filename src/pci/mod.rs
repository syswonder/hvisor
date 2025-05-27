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
pub const CFG_EXT_CAP_PTR_OFF: usize = 0x100; // extended capabilities pointer
pub const CFG_NEXT_EXT_CAP_OFF: usize = 20;
pub const CFG_CLASS_CODE_OFF: usize = 0x8; // 4 bytes, include revision and class code
pub const CFG_SRIOV_CAP_ID: usize = 0x0010;
pub const CFG_EXT_CAP_ID: usize = 0x10;
pub const CFG_BAR0: usize = 0x10;
pub const CFG_BAR1: usize = 0x14;
pub const CFG_BAR2: usize = 0x18;
pub const CFG_BAR3: usize = 0x1c;
pub const CFG_BAR4: usize = 0x20;
pub const CFG_BAR5: usize = 0x24;
pub const CFG_PRIMARY_BUS: usize = 0x18;
pub const CFG_SECONDARY_BUS: usize = 0x19;
pub const CFG_IO_BASE: usize = 0x1c;
pub const CFG_IO_LIMIT: usize = 0x1d;
pub const CFG_MEM_BASE: usize = 0x20;
pub const CFG_MEM_LIMIT: usize = 0x22;
pub const CFG_PREF_MEM_BASE: usize = 0x24;
pub const CFG_PREF_MEM_LIMIT: usize = 0x26;
pub const CFG_PREF_BASE_UPPER32: usize = 0x28;
pub const CFG_PREF_LIMIT_UPPER32: usize = 0x2c;
pub const CFG_IO_BASE_UPPER16: usize = 0x30;
pub const CFG_IO_LIMIT_UPPER16: usize = 0x32;
pub const CFG_INT_LINE: usize = 0x3d;
pub const CFG_INT_PIN: usize = 0x3d;

pub const NUM_BAR_REGS_TYPE0: usize = 6;
pub const NUM_BAR_REGS_TYPE1: usize = 2;
pub const NUM_MAX_BARS: usize = 6;
pub const PHANTOM_DEV_HEADER: u32 = 0x77777777u32;

pub static ECAM_BASE: Once<usize> = Once::new();

pub static BDF_SHIFT: Once<usize> = Once::new();

pub fn init_ecam_base(ecam_base: usize) {
    ECAM_BASE.call_once(|| ecam_base);
}

pub fn get_ecam_base() -> usize {
    *ECAM_BASE.get().unwrap() as _
}

pub fn init_bdf_shift(bdf_shift: usize) {
    BDF_SHIFT.call_once(|| bdf_shift);
}

pub fn get_bdf_shift() -> usize {
    *BDF_SHIFT.get().unwrap() as _
}

pub fn cfg_base(bdf: usize) -> usize {
    let shift = get_bdf_shift();
    if cfg!(all(target_arch = "loongarch64", feature = "pci")) && ((bdf >> 8) != 0) {
        get_ecam_base() + (bdf << shift) + 0x1000_0000
    } else {
        get_ecam_base() + (bdf << shift)
    }
}

// generate addr with reg addr, example off = 0x123, shift = 0x8
pub fn cfg_reg_addr(bdf: usize, off: usize) -> usize {
    let base = cfg_base(bdf);
    let shift = get_bdf_shift();
    let upper_off = off >> shift; // 0x1
    let lower_off = off & ((1 << shift) - 1); // 0x23
    let addr = (upper_off << (shift + 16)) + base + lower_off;
    addr
}

/// Extracts the PCI config space register offset, compatible with architectures where the offset layout is split (e.g., LoongArch).
/// Low bits are taken from address[0..bdf_shift), high bits from address[(bdf_shift + 16)..).
fn extract_reg_addr(addr: usize) -> usize {
    let bdf_shift = get_bdf_shift();
    let low_mask = (1usize << bdf_shift) - 1;
    let low_bits = addr & low_mask;

    let high_shift = bdf_shift + 16;
    let high_mask = (1usize << (12 - bdf_shift)) - 1;
    let high_bits = ((addr >> high_shift) & high_mask) << bdf_shift;

    high_bits | low_bits
}
