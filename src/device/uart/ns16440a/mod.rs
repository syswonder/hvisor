#![allow(dead_code)]
use crate::device::common::MMIODerefWrapper;
use crate::memory::addr::{PhysAddr, VirtAddr};
use core::arch::global_asm;
use spin::Mutex;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

pub const UART_BASE_PHYS: PhysAddr = 0x1fe001e0;
pub const UART_BASE_VIRT: VirtAddr = 0x80000000_1fe001e0;

const UART_REF_CLK: usize = 100000000; // 100MHz for 3A5000's SYS_CLK
const UART_DIV_HI: usize = ((UART_REF_CLK + (115200 * 8)) / (115200 * 16)) >> 8;
const UART_DIV_LO: usize = ((UART_REF_CLK + (115200 * 8)) / (115200 * 16)) & 0xff;

const BOARD_UART0_VADDR: usize = 0x8000_0000_1fe0_01e0;
const BOARD_UART1_VADDR: usize = BOARD_UART0_VADDR + 0x100;
const BOARD_UART2_VADDR: usize = BOARD_UART0_VADDR + 0x200;
const BOARD_UART3_VADDR: usize = BOARD_UART0_VADDR + 0x300;

global_asm!(
  include_str!("uart.S"),
  CONSOLE_BASE_ADDR = const BOARD_UART0_VADDR,
  UART_DIV_HI = const UART_DIV_HI,
  UART_DIV_LO = const UART_DIV_LO
);

extern "C" {
    fn init_serial();
    fn print_char(c: u8);
    fn get_char() -> u8;
}

register_bitfields! {
  u8, // register width

  // UART DATA REGISTER
  DAT [
    TxFIFO OFFSET(0) NUMBITS(8) []
  ],

  // UART INTERRUPT ENABLE REGISTER
  IER [
    IME OFFSET(3) NUMBITS(1) [
      Disable = 0,
      Enable = 1
    ],
    ILE OFFSET(2) NUMBITS(1) [
      Disable = 0,
      Enable = 1
    ],
    ITxE OFFSET(1) NUMBITS(1) [
      Disable = 0,
      Enable = 1
    ],
    IRxE OFFSET(0) NUMBITS(1) [
      Disable = 0,
      Enable = 1
    ]
  ],

  // UART INTERRUPT INDICATOR REGISTER
  IIR [
    II OFFSET(1) NUMBITS(3) [
      RecvLineStatus = 0b011,
      RecvDataAvail = 0b010,
      CharTimeout = 0b110,
      THREmpty = 0b001,
      ModemStatus = 0b000,
    ],
    INTp OFFSET(0) NUMBITS(1) [],
  ],

  // UART FIFO CONTROL REGISTER
  FCR [
    TL OFFSET(6) NUMBITS(2) [],
    Txset OFFSET(2) NUMBITS(1) [],
    Rxset OFFSET(1) NUMBITS(1) [],
  ],

  // UART LINE CONTROL REGISTER
  LCR [
    dlab OFFSET(7) NUMBITS(1) [],
    bcb OFFSET(6) NUMBITS(1) [],
    spb OFFSET(5) NUMBITS(1) [],
    eps OFFSET(4) NUMBITS(1) [],
    pe OFFSET(3) NUMBITS(1) [],
    sb OFFSET(2) NUMBITS(1) [],
    bec OFFSET(1) NUMBITS(1) [],
  ],

  // UART MODEM CONTROL REGISTER
  MCR [
    Loop OFFSET(4) NUMBITS(1) [],
    OUT2 OFFSET(3) NUMBITS(1) [],
    OUT1 OFFSET(2) NUMBITS(1) [],
    RTSC OFFSET(1) NUMBITS(1) [],
    DTRC OFFSET(0) NUMBITS(1) [],
  ],

  // UART LINE STATUS REGISTER
  LSR [
    ERROR OFFSET(7) NUMBITS(1) [],
    TE OFFSET(6) NUMBITS(1) [],
    TFE OFFSET(5) NUMBITS(1) [],
    BI OFFSET(4) NUMBITS(1) [],
    FE OFFSET(3) NUMBITS(1) [],
    PE OFFSET(2) NUMBITS(1) [],
    OE OFFSET(1) NUMBITS(1) [],
    DR OFFSET(0) NUMBITS(1) [],
  ],

  // UART MODEM STATUS REGISTER
  MSR [
    CDCD OFFSET(7) NUMBITS(1) [],
    CRI OFFSET(6) NUMBITS(1) [],
    CDSR OFFSET(5) NUMBITS(1) [],
    CCTS OFFSET(4) NUMBITS(1) [],
    DDCD OFFSET(3) NUMBITS(1) [],
    TERI OFFSET(2) NUMBITS(1) [],
    DDSR OFFSET(1) NUMBITS(1) [],
    DCTS OFFSET(0) NUMBITS(1) [],
  ],
}

register_structs!(
  UartRegs {
    (0x00 => dat: ReadWrite<u8, DAT::Register>),
    (0x01 => ier: ReadWrite<u8, IER::Register>),
    (0x02 => iir: ReadOnly<u8, IIR::Register>),
    (0x03 => fcr: WriteOnly<u8, FCR::Register>),
    (0x04 => lcr: ReadWrite<u8, LCR::Register>),
    (0x05 => mcr: ReadWrite<u8, MCR::Register>),
    (0x06 => lsr: ReadOnly<u8, LSR::Register>),
    (0x07 => msr: ReadOnly<u8, MSR::Register>),
    (0x08 => @END),
  }
);
#[allow(dead_code)]
pub struct Uart {
    base_addr: usize,
    regs: MMIODerefWrapper<UartRegs>,
}

impl Uart {
    pub const fn new(base_addr: usize) -> Self {
        Self {
            base_addr,
            regs: unsafe { MMIODerefWrapper::new(base_addr as usize) },
        }
    }
    pub fn init(&mut self) {
        unsafe {
            init_serial();
        }
    }
    pub fn putchar(&mut self, c: u8) {
        unsafe {
            print_char(c);
        }
    }
    pub fn send_str(&mut self, s: &str) {
        for c in s.bytes() {
            self.putchar(c);
            if c == b'\n' {
                self.putchar(b'\r');
            }
        }
    }
    pub fn getchar(&mut self) -> u8 {
        unsafe { get_char() }
    }
}

pub static UART0: Mutex<Uart> = Mutex::new(Uart::new(BOARD_UART0_VADDR));

pub fn console_putchar(c: u8) {
    UART0.lock().putchar(c);
}

pub fn console_getchar() -> Option<u8> {
    UART0.lock().getchar().into()
}
