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
#![allow(dead_code)]
use crate::device::common::MMIODerefWrapper;
use crate::memory::addr::{PhysAddr, VirtAddr};
use core::arch::global_asm;
use spin::Mutex;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

const UART_CPU_REF_CLK: usize = 100000000; // 100MHz for 3A5000's SYS_CLK
const UART_CPU_DIV_HI: usize = ((UART_CPU_REF_CLK + (115200 * 8)) / (115200 * 16)) >> 8;
const UART_CPU_DIV_LO: usize = ((UART_CPU_REF_CLK + (115200 * 8)) / (115200 * 16)) & 0xff;

const UART_COM_REF_CLK: usize = 50000000; // 50MHz for 7A2000 COM DB9 RS232 (115200 8n1)
const UART_COM_DIV_HI: usize = ((UART_COM_REF_CLK + (115200 * 8)) / (115200 * 16)) >> 8;
const UART_COM_DIV_LO: usize = ((UART_COM_REF_CLK + (115200 * 8)) / (115200 * 16)) & 0xff;

const BOARD_UART0_VADDR: usize = 0x8000_0000_1fe0_01e0;
const BOARD_UART1_VADDR: usize = 0x8000_0000_1008_0000;
const BOARD_UART2_VADDR: usize = 0x8000_0000_1008_0100;
const BOARD_UART3_VADDR: usize = 0x8000_0000_1008_0200;

global_asm!(
  include_str!("uart0.S"),
  CONSOLE_BASE_ADDR = const BOARD_UART0_VADDR,
  UART_DIV_HI = const UART_CPU_DIV_HI,
  UART_DIV_LO = const UART_CPU_DIV_LO
);

global_asm!(
  include_str!("uart1.S"),
  CONSOLE_BASE_ADDR = const BOARD_UART1_VADDR,
  UART_DIV_HI = const UART_COM_DIV_HI,
  UART_DIV_LO = const UART_COM_DIV_LO
);

extern "C" {
    // UART0
    fn uart0_init();
    fn uart0_putchar(c: u8);
    fn uart0_getchar() -> u8;
    // UART1
    fn uart1_init();
    fn uart1_putchar(c: u8);
    fn uart1_getchar() -> u8;
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
    port: usize, // 0 for UART0, 1 for UART1
    base_addr: usize,
    regs: MMIODerefWrapper<UartRegs>,
}

impl Uart {
    pub const fn new(port: usize) -> Self {
        let base_addr = match port {
            0 => BOARD_UART0_VADDR,
            1 => BOARD_UART1_VADDR,
            _ => panic!("Invalid UART port"),
        };
        Self {
            port,
            base_addr,
            regs: unsafe { MMIODerefWrapper::new(base_addr) },
        }
    }
    pub fn init(&mut self) {
        match self.port {
            0 => unsafe { uart0_init() },
            1 => unsafe { uart1_init() },
            _ => panic!("Invalid UART port"),
        }
    }
    pub fn putchar(&mut self, c: u8) {
        match self.port {
            0 => unsafe { uart0_putchar(c) },
            1 => unsafe { uart1_putchar(c) },
            _ => panic!("Invalid UART port"),
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
        match self.port {
            0 => unsafe { uart0_getchar() },
            1 => unsafe { uart1_getchar() },
            _ => panic!("Invalid UART port"),
        }
    }
}

pub static UART0: Mutex<Uart> = Mutex::new(Uart::new(0));
pub static UART1: Mutex<Uart> = Mutex::new(Uart::new(1));

pub fn console_putchar(c: u8) {
    UART0.lock().putchar(c);
}

pub fn console_getchar() -> Option<u8> {
    UART0.lock().getchar().into()
}

pub fn __test_uart1() {
    info!("loongarch: uart: __test_uart1");
    let mut uart1 = UART1.lock();
    uart1.init();
    info!("loongarch: uart: __test_uart1 init done");
    let s = "Hello, UART1!\n";
    uart1.send_str(s);
    info!("loongarch: uart: __test_uart1 send_str test done");
}
