#![allow(dead_code)]
use spin::Mutex;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

const UART_CLOCK_FACTOR: usize = 16;
const OSC_FREQ: usize = 1_843_200;

lazy_static::lazy_static! {
    static ref COM1: Mutex<Uart16550> = {
        let mut uart = Uart16550::new(0x3f8);
        uart.init(115200);
        Mutex::new(uart)
    };
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
            rhr: PortReadOnly::new(base_port),
            thr: PortWriteOnly::new(base_port),
            ier: PortWriteOnly::new(base_port + 1),
            fcr: PortWriteOnly::new(base_port + 2),
            lcr: PortWriteOnly::new(base_port + 3),
            mcr: PortWriteOnly::new(base_port + 4),
            lsr: PortReadOnly::new(base_port + 5),
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
            while self.lsr.read() & (1 << 5) == 0 {}
            self.thr.write(c);
        }
    }

    fn getchar(&mut self) -> Option<u8> {
        unsafe {
            if self.lsr.read() & 1 != 0 {
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
