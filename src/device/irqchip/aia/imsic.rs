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
use super::vimsic::*;
use crate::arch::csr::{read_csr, write_csr};
use crate::platform::__board::{IMSIC_GUEST_INDEX, IMSIC_NUM_IDS};

pub const CSR_SISELECT: usize = 0x150;
pub const CSR_SIREG: usize = 0x151;
pub const CSR_STOPEI: usize = 0x15c;
// pub const CSR_VSISELECT: usize = 0x250;
// pub const CSR_VSIREG: usize = 0x251;
// pub const CSR_VSTOPEI: usize = 0x25C;

pub const IMSIC_EIDELIVERY: usize = 0x70;
pub const IMSIC_EITHRESHOLD: usize = 0x72;
pub const IMSIC_EIP: usize = 0x80; // 0x80..=0xBF
pub const IMSIC_EIE: usize = 0xC0; // 0xC0..=0xFF

/// Init imsic per hart
pub fn imsic_init() {
    // Disable interrupt delivery
    write_csr!(CSR_SISELECT, IMSIC_EIDELIVERY);
    write_csr!(CSR_SIREG, 0);

    // For qemu, num_ids = 0xff
    let num_ids = (IMSIC_NUM_IDS + 63) / 64;

    // For 64bit system, eip1, eip3.. eip63 don't exist.
    for i in (0..num_ids) {
        // Disable all interrupts
        write_csr!(CSR_SISELECT, IMSIC_EIE + i * 2);
        write_csr!(CSR_SIREG, 0);
        // Remove all pending interrupts
        write_csr!(CSR_SISELECT, IMSIC_EIP + i * 2);
        write_csr!(CSR_SIREG, 0);
    }

    // Every interrupt is triggerable
    write_csr!(CSR_SISELECT, IMSIC_EITHRESHOLD);
    write_csr!(CSR_SIREG, 0);

    // Enable interrupt delivery
    write_csr!(CSR_SISELECT, IMSIC_EIDELIVERY);
    write_csr!(CSR_SIREG, 1);
}

/// Write to imsic interrupt file.
pub fn imsic_trigger(hart: u32, guest: u32, eiid: u32) {
    if guest as usize == IMSIC_GUEST_INDEX {
        let addr = imsic_vs_file_addr(hart as usize);
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, eiid);
        }
    } else {
        panic!(
            "Unknown imsic set hart {} guest {} eiid {}",
            hart, guest, eiid
        );
    }
}
