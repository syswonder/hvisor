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
#![allow(dead_code)]

use core::ptr;

use crate::memory::addr::{PhysAddr, VirtAddr};
// use spin::Mutex;

pub const UART_BASE_PHYS: PhysAddr = 0x30890000;
// pub const UART_BASE_VIRT: VirtAddr = 0xffffc0090000;

const UTS: usize = 0xb4;
const UTXD: usize = 0x40;
const UTS_TXEMPTY: u32 = 1 << 6;

// lazy_static! {
static mut UART: ImxUart = {
    ImxUart::new(UART_BASE_PHYS)
    // Mutex::new(uart)
};
// }

struct ImxUart {
    base_vaddr: VirtAddr,
}

impl ImxUart {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr }
    }

    fn is_busy(&self) -> bool {
        let uts_addr = (self.base_vaddr + UTS) as *mut u32;
        unsafe {
            return (ptr::read_volatile(uts_addr) & UTS_TXEMPTY) == 0;
        }
    }
    fn putchar(&mut self, c: u8) {
        unsafe {
            while self.is_busy() {}
            ptr::write_volatile((self.base_vaddr + UTXD) as *mut u32, c as u32);
        }
    }
    fn getchar(&mut self) -> Option<u8> {
        todo!()
    }
}

pub fn console_putchar(c: u8) {
    unsafe { UART.putchar(c) }
}

pub fn console_getchar() -> Option<u8> {
    unsafe { UART.getchar() }
}
