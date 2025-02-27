use crate::{arch::device::PortIoDevice, error::HvResult};
use spin::Mutex;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

const DATA_REG: u16 = 0;
const INT_EN_REG: u16 = 1;
const FIFO_CTRL_REG: u16 = 2;
const LINE_CTRL_REG: u16 = 3;
const MODEM_CTRL_REG: u16 = 4;
const LINE_STATUS_REG: u16 = 5;
const MODEM_STATUS_REG: u16 = 6;
const SCRATCH_REG: u16 = 7;

const UART_CLOCK_FACTOR: usize = 16;
const UART_FIFO_CAPACITY: usize = 16;
const OSC_FREQ: usize = 1_843_200;

lazy_static::lazy_static! {
    static ref COM1: Mutex<Uart16550> = {
        let mut uart = Uart16550::new(0x3f8);
        uart.init(115200);
        Mutex::new(uart)
    };
}

bitflags::bitflags! {
    /// Line status flags
    struct LineStatusFlags: u8 {
        const RECEIVER_DATA_READY = 1;
        const TRANSMIT_HOLD_REG_EMPTY = 1 << 5;
        const TRANSMITTER_EMPTY = 1 << 6;
    }
}

/// FIFO queue for caching bytes read.
struct Fifo<const CAP: usize> {
    buf: [u8; CAP],
    head: usize,
    num: usize,
}

impl<const CAP: usize> Fifo<CAP> {
    const fn new() -> Self {
        Self {
            buf: [0; CAP],
            head: 0,
            num: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.num == 0
    }

    fn is_full(&self) -> bool {
        self.num == CAP
    }

    fn push(&mut self, value: u8) {
        assert!(self.num < CAP);
        self.buf[(self.head + self.num) % CAP] = value;
        self.num += 1;
    }

    fn pop(&mut self) -> u8 {
        assert!(self.num > 0);
        let ret = self.buf[self.head];
        self.head += 1;
        self.head %= CAP;
        self.num -= 1;
        ret
    }
}

struct Uart16550 {
    rhr: PortReadOnly<u8>,  // receive holding
    thr: PortWriteOnly<u8>, // transmit holding
    ier: PortWriteOnly<u8>, // interrupt enable
    fcr: PortWriteOnly<u8>, // fifo control
    lcr: PortWriteOnly<u8>, // line control
    mcr: PortWriteOnly<u8>, // modem control
    lsr: PortReadOnly<u8>,  // line status
}

impl Uart16550 {
    const fn new(base_port: u16) -> Self {
        Self {
            rhr: PortReadOnly::new(base_port + DATA_REG),
            thr: PortWriteOnly::new(base_port + DATA_REG),
            ier: PortWriteOnly::new(base_port + INT_EN_REG),
            fcr: PortWriteOnly::new(base_port + FIFO_CTRL_REG),
            lcr: PortWriteOnly::new(base_port + LINE_CTRL_REG),
            mcr: PortWriteOnly::new(base_port + MODEM_CTRL_REG),
            lsr: PortReadOnly::new(base_port + LINE_STATUS_REG),
        }
    }

    fn init(&mut self, baud_rate: usize) {
        unsafe {
            // disable interrupts
            self.ier.write(0x00);

            // enable DLAB, set baud rate
            let divisor = OSC_FREQ / (baud_rate * UART_CLOCK_FACTOR);
            self.lcr.write(0x80);
            self.thr.write((divisor & 0xff) as u8);
            self.ier.write((divisor >> 8) as u8);

            // disable DLAB, set word length to 8 bits
            self.lcr.write(0x03);

            // enable fifo, clear tx/rx queues
            // set interrupt level to 14 bytes
            self.fcr.write(0xC7);

            // data terminal ready, request to send
            // enable option 2 output (used as interrupt line for CPU)
            self.mcr.write(0x0B);
        }
    }

    fn putchar(&mut self, c: u8) {
        unsafe {
            while self.lsr.read() & LineStatusFlags::TRANSMIT_HOLD_REG_EMPTY.bits() == 0 {}
            self.thr.write(c);
        }
    }

    fn getchar(&mut self) -> Option<u8> {
        unsafe {
            if self.lsr.read() & LineStatusFlags::RECEIVER_DATA_READY.bits() != 0 {
                Some(self.rhr.read())
            } else {
                None
            }
        }
    }
}

pub struct VirtUart16550 {
    base_port: u16,
    fifo: Mutex<Fifo<UART_FIFO_CAPACITY>>,
}

impl VirtUart16550 {
    pub fn new(base_port: u16) -> Self {
        Self {
            base_port,
            fifo: Mutex::new(Fifo::new()),
        }
    }
}

impl PortIoDevice for VirtUart16550 {
    fn port_range(&self) -> core::ops::Range<u16> {
        self.base_port..self.base_port + 8
    }

    fn read(&self, port: u16, access_size: u8) -> HvResult<u32> {
        if access_size != 1 {
            error!("Invalid serial port I/O read size: {} != 1", access_size);
            return hv_result_err!(EIO);
        }
        let ret = match port - self.base_port {
            DATA_REG => {
                // read a byte from FIFO
                let mut fifo = self.fifo.lock();
                if fifo.is_empty() {
                    0
                } else {
                    fifo.pop()
                }
            }
            LINE_STATUS_REG => {
                // check if the physical serial port has an available byte, and push it to FIFO.
                let mut fifo = self.fifo.lock();
                if !fifo.is_full() {
                    if let Some(c) = console_getchar() {
                        fifo.push(c);
                    }
                }
                let mut lsr =
                    LineStatusFlags::TRANSMIT_HOLD_REG_EMPTY | LineStatusFlags::TRANSMITTER_EMPTY;
                if !fifo.is_empty() {
                    lsr |= LineStatusFlags::RECEIVER_DATA_READY;
                }
                lsr.bits()
            }
            FIFO_CTRL_REG => {
                0xc0 // FIFO enabled
            }
            INT_EN_REG | LINE_CTRL_REG | MODEM_CTRL_REG | MODEM_STATUS_REG | SCRATCH_REG => {
                debug!("Unimplemented serial port I/O read: {:#x}", port); // unimplemented
                0
            }
            _ => unreachable!(),
        };
        Ok(ret as u32)
    }

    fn write(&self, port: u16, access_size: u8, value: u32) -> HvResult {
        if access_size != 1 {
            error!("Invalid serial port I/O write size: {} != 1", access_size);
            return hv_result_err!(EIO);
        }
        match port - self.base_port {
            DATA_REG => console_putchar(value as u8),
            INT_EN_REG | FIFO_CTRL_REG | LINE_CTRL_REG | MODEM_CTRL_REG | SCRATCH_REG => {
                debug!("Unimplemented serial port I/O write: {:#x}", port); // unimplemented
            }
            LINE_STATUS_REG => {} // ignore
            _ => unreachable!(),
        }
        Ok(())
    }
}

pub fn console_putchar(c: u8) {
    COM1.lock().putchar(c);
}

pub fn console_getchar() -> Option<u8> {
    COM1.lock().getchar()
}
