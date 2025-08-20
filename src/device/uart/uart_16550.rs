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
//      Yulong Han <wheatfox17@icloud.com>
//

use crate::memory::addr::{PhysAddr, VirtAddr};
use crate::platform::BOARD_UART_BASE;
use core::ptr;
use spin::Mutex;
use tock_registers::interfaces::*;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::*;

/// Register struct representing the UART registers.
register_structs! {
    /// Representation of the UART registers.
    #[allow(non_snake_case)]
    pub Ns16550a {
        (0x00 => pub THR_RBR_DLL: ReadWrite<u32>),
        (0x04 => pub IER_DLH: ReadWrite<u32>),
        (0x08 => pub IIR_FCR: ReadWrite<u32>),
        (0x0c => pub LCR: ReadWrite<u32>),
        (0x10 => pub MCR: ReadWrite<u32>),
        (0x14 => pub LSR: ReadOnly<u32>),
        (0x18 => pub MSR: ReadWrite<u32>),
        (0x1c => pub SR: ReadWrite<u32>),
        (0x20 => @END),
    }
}

#[allow(dead_code)]
pub struct Uart16550 {
    base_addr: usize,
}

impl Uart16550 {
    pub const fn new(base_addr: VirtAddr) -> Self {
        Self { base_addr }
    }

    #[inline]
    pub const fn regs(&self) -> &Ns16550a {
        unsafe { &*(self.base_addr as *const _) }
    }

    pub fn init(&mut self) {
        self.regs().LCR.set(0x3 << 0);
        self.regs().IER_DLH.set(0);
        self.regs().MCR.set(0);

        self.regs().LSR.get();
        self.regs().MSR.set(0);

        self.regs().IIR_FCR.set(0x1 << 0);
    }

    #[inline]
    pub fn putchar(&mut self, c: u8) {
        while self.regs().LSR.get() & (1 << 5) == 0 {}
        self.regs().THR_RBR_DLL.set(c as u32);
    }

    #[inline]
    fn getchar(&mut self) -> Option<u8> {
        todo!()
    }
}

static mut UART: Uart16550 = Uart16550::new(BOARD_UART_BASE as _);

#[inline]
pub fn console_putchar(c: u8) {
    unsafe { UART.putchar(c) }
}

#[inline]
pub fn console_getchar() -> Option<u8> {
    unsafe { UART.getchar() }
}
