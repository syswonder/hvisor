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
use crate::arch::csr::{write_csr ,read_csr ,CSR_VSISELECT ,CSR_VSIREG ,CSR_VSTOPI ,CSR_VSTOPEI};
pub const IMSIC_VS: usize = 0x2800_1000;
const IMSIC_VS_HART_STRIDE: usize = 0x2000;

const XLEN: usize = usize::BITS as usize;
const XLEN_STRIDE: usize = XLEN / 32;

const EIP: usize = 0x80;

pub const fn imsic_vs(hart: usize) -> usize {
    IMSIC_VS + IMSIC_VS_HART_STRIDE * hart
}
fn imsic_write(reg: usize, val: usize) {
    unsafe {
        match reg {
            CSR_VSISELECT => write_csr!(CSR_VSISELECT, val),
            CSR_VSIREG => write_csr!(CSR_VSIREG, val),
            CSR_VSTOPI => write_csr!(CSR_VSTOPI, val),
            CSR_VSTOPEI => write_csr!(CSR_VSTOPEI, val),
            _ => panic!("Unknown CSR {}", reg),
        }
    }
}

// Read from an IMSIC CSR

fn imsic_read(reg: usize) -> usize {
    let ret: usize;
    unsafe {
        ret = match reg {
            CSR_VSISELECT => read_csr!(CSR_VSISELECT),
            CSR_VSIREG => read_csr!(CSR_VSIREG),
            CSR_VSTOPI => read_csr!(CSR_VSTOPI),
            CSR_VSTOPEI => read_csr!(CSR_VSTOPEI),
            _ => panic!("Unknown CSR {}", reg),
        }
    }
    ret
}
// VS-Mode IMSIC CSRs


pub fn imsic_trigger(hart: u32, guest: u32, eiid: u32) {
    if guest == 1{
        unsafe {
            core::ptr::write_volatile(imsic_vs(hart as usize) as *mut u32, eiid);
        }
    } else {
        panic!(
            "Unknown imsic set hart {} guest {} eiid {}",
            hart, guest, eiid
        );
    }
}

