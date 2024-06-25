#![allow(dead_code)]
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

use crate::memory::addr::{PhysAddr, VirtAddr};
use spin::Mutex;

pub const UART_BASE_PHYS: PhysAddr = 0x1fe001e0;
pub const UART_BASE_VIRT: VirtAddr = 0x80000000_1fe001e0;

pub fn console_putchar(c: u8) {
    todo!()
}

pub fn console_getchar() -> Option<u8> {
    todo!()
}
