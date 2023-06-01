use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

use crate::memory::addr::{PhysAddr, VirtAddr};
use spin::Mutex;

pub const UART_BASE_PHYS: PhysAddr = 0x09000000;
pub const UART_BASE_VIRT: VirtAddr = 0xffffc0000000;

lazy_static! {
    static ref UART: Mutex<Pl011Uart> = {
        let mut uart = Pl011Uart::new(UART_BASE_VIRT);
        uart.init();
        Mutex::new(uart)
    };
}

register_structs! {
    Pl011UartRegs {
        (0x00 => dr: ReadWrite<u32>),
        (0x04 => _reserved0),
        (0x18 => fr: ReadOnly<u32>),
        (0x1c => _reserved1),
        (0x30 => cr: ReadWrite<u32>),
        (0x34 => ifls: ReadWrite<u32>),
        (0x38 => imsc: ReadWrite<u32>),
        (0x3c => ris: ReadOnly<u32>),
        (0x40 => mis: ReadOnly<u32>),
        (0x44 => icr: WriteOnly<u32>),
        (0x48 => @END),
    }
}

struct Pl011Uart {
    base_vaddr: VirtAddr,
}

impl Pl011Uart {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr }
    }

    const fn regs(&self) -> &Pl011UartRegs {
        unsafe { &*(self.base_vaddr as *const _) }
    }

    fn init(&mut self) {
        self.regs().icr.set(0x3ff);
        self.regs().ifls.set(0);
        self.regs().imsc.set(1 << 4);
        self.regs().cr.set((1 << 0) | (1 << 8) | (1 << 9));
    }

    fn putchar(&mut self, c: u8) {
        while self.regs().fr.get() & (1 << 5) != 0 {}
        self.regs().dr.set(c as u32)
    }

    fn getchar(&mut self) -> Option<u8> {
        if self.regs().fr.get() & (1 << 4) == 0 {
            Some(self.regs().dr.get() as u8)
        } else {
            None
        }
    }
}

pub fn console_putchar(c: u8) {
    UART.lock().putchar(c)
}

pub fn console_getchar() -> Option<u8> {
    UART.lock().getchar()
}
