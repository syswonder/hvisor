use crate::arch::pio::UART_COM1_BASE_PORT;
use spin::Mutex;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

#[allow(non_snake_case)]
pub mod UartReg {
    pub const RCVR_BUFFER: u16 = 0;
    pub const XMIT_BUFFER: u16 = 0;
    pub const INTR_ENABLE: u16 = 1;
    pub const INTR_IDENT: u16 = 2;
    pub const FIFO_CTRL: u16 = 2;
    pub const LINE_CTRL: u16 = 3;
    pub const MODEM_CTRL: u16 = 4;
    pub const LINE_STATUS: u16 = 5;
    pub const MODEM_STATUS: u16 = 6;
    pub const SCRATCH: u16 = 7;
}

const UART_CLOCK_FACTOR: usize = 16;
const OSC_FREQ: usize = 1_843_200;

lazy_static::lazy_static! {
    static ref COM1: Mutex<Uart16550a> = {
        let mut uart = Uart16550a::new(UART_COM1_BASE_PORT);
        uart.init(115200);
        Mutex::new(uart)
    };
}

bitflags::bitflags! {
    struct InterruptEnableFlags: u8 {
        const ENABLE_RCVR_DATA_AVAIL_INTR = 1 << 0;
        const ENABLE_XMIT_HOLD_REG_EMPTY_INTR = 1 << 1;
        const _ = !0;
    }

    struct InterruptIdentFlags: u8 {
        const NO_INTR_IS_PENDING = 1 << 0;
        const XMIT_HOLD_REG_EMPTY = 0x2;
        const RCVR_DATA_AVAIL = 0x4;
        const FIFO_ENABLED_16550_MODE = 0xc0;
        const _ = !0;
    }

    struct LineControlFlags: u8 {
        const WORD_LENGTH_SELECT_8_BITS = 0x3;
        const DIVISOR_LATCH_ACCESS_BIT = 1 << 7;
        const _ = !0;
    }

    struct LineStatusFlags: u8 {
        const RCVR_DATA_READY = 1;
        const XMIT_HOLD_REG_EMPTY = 1 << 5;
        const XMIT_EMPTY = 1 << 6;
        const _ = !0;
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

struct Uart16550a {
    rhr: PortReadOnly<u8>,  // receive holding
    thr: PortWriteOnly<u8>, // transmit holding
    ier: PortWriteOnly<u8>, // interrupt enable
    fcr: PortWriteOnly<u8>, // fifo control
    lcr: PortWriteOnly<u8>, // line control
    mcr: PortWriteOnly<u8>, // modem control
    lsr: PortReadOnly<u8>,  // line status
}

impl Uart16550a {
    const fn new(base_port: u16) -> Self {
        Self {
            rhr: PortReadOnly::new(base_port + UartReg::RCVR_BUFFER),
            thr: PortWriteOnly::new(base_port + UartReg::XMIT_BUFFER),
            ier: PortWriteOnly::new(base_port + UartReg::INTR_ENABLE),
            fcr: PortWriteOnly::new(base_port + UartReg::FIFO_CTRL),
            lcr: PortWriteOnly::new(base_port + UartReg::LINE_CTRL),
            mcr: PortWriteOnly::new(base_port + UartReg::MODEM_CTRL),
            lsr: PortReadOnly::new(base_port + UartReg::LINE_STATUS),
        }
    }

    fn init(&mut self, baud_rate: usize) {
        unsafe {
            // enable read available interrupts
            self.ier
                .write(InterruptEnableFlags::ENABLE_RCVR_DATA_AVAIL_INTR.bits());

            // enable DLAB, set baud rate
            let divisor = OSC_FREQ / (baud_rate * UART_CLOCK_FACTOR);
            self.lcr
                .write(LineControlFlags::DIVISOR_LATCH_ACCESS_BIT.bits());
            self.thr.write((divisor & 0xff) as u8);
            self.ier.write((divisor >> 8) as u8);

            // disable DLAB, set word length to 8 bits
            self.lcr
                .write(LineControlFlags::WORD_LENGTH_SELECT_8_BITS.bits());

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
            while self.lsr.read() & LineStatusFlags::XMIT_HOLD_REG_EMPTY.bits() == 0 {}
            self.thr.write(c);
        }
    }

    fn getchar(&mut self) -> Option<u8> {
        unsafe {
            if self.lsr.read() & LineStatusFlags::RCVR_DATA_READY.bits() != 0 {
                Some(self.rhr.read())
            } else {
                None
            }
        }
    }
}

pub fn console_putchar(c: u8) {
    COM1.lock().putchar(c);
}

pub fn console_getchar() -> Option<u8> {
    COM1.lock().getchar()
}
