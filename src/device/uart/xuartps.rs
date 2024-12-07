/// UART driver for Xilinx Zynq Ultrascale+ MPSoC ZCU102 board.
/// author: wheatfox (enkerewpo@hotmail.com)
/// references:
/// 1. Zynq UltraScale+ Device TRM UG1085 (v2.4) December 21, 2023 Chapter 21
/// 2. https://github.com/Xilinx/linux-xlnx :: drivers/tty/serial/xilinx_uartps.c
/// 3. https://github.com/torvalds/linux/blob/master/drivers/tty/serial/xilinx_uartps.c
use crate::memory::addr::{PhysAddr, VirtAddr};
use spin::Mutex;
use tock_registers::{
    interfaces::{Readable, Writeable, ReadWriteable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

pub const UART0_BASE: PhysAddr = 0xff000000;
pub const UART1_BASE: PhysAddr = 0xff010000;

pub const UART_FIFO_SIZE: usize = 64;
pub const UART_REGS_REGION_SIZE: usize = 0x1000;
pub const UART_TX_TIMEOUT: usize = 500000;
pub const UART_DEFAULT_BAUDRATE: u32 = 115200;
pub const UART_REF_CLK: u32 = 50_000_000; // 50 MHz
pub const UART_BAUDRATE_ACTUAL_CD: u32 = 62;
pub const UART_BAUDRATE_BDIV: u32 = 6;

lazy_static! {
    static ref UART0: Mutex<ZynqUart> = {
        let mut uart = ZynqUart::new(UART0_BASE);
        uart.init();
        Mutex::new(uart)
    };
}

lazy_static! {
    static ref UART1: Mutex<ZynqUart> = {
        let mut uart = ZynqUart::new(UART1_BASE);
        uart.init();
        Mutex::new(uart)
    };
}

register_structs! {
    ZynqUartRegs {
        (0x00 => cr: ReadWrite<u32, CR::Register>), // Control Register
        (0x04 => mr: ReadWrite<u32, MR::Register>), // Mode Register
        (0x08 => ier: WriteOnly<u32, IXR::Register>), // Interrupt Enable Register
        (0x0C => idr: WriteOnly<u32, IXR::Register>), // Interrupt Disable Register
        (0x10 => imr: ReadOnly<u32, IXR::Register>), // Interrupt Mask Register
        (0x14 => isr: ReadOnly<u32, IXR::Register>), // Interrupt Status Register
        (0x18 => baudgen: ReadWrite<u32>), // Baud Rate Generator
        (0x1C => rxtout: ReadWrite<u32>), // RX Timeout
        (0x20 => rxwm: ReadWrite<u32>), // RX FIFO Trigger Level
        (0x24 => modemcr: ReadWrite<u32, MODEMCR::Register>), // Modem Control Register
        (0x28 => modemsr: ReadOnly<u32, MODEMSR::Register>), // Modem Status Register
        (0x2C => sr: ReadOnly<u32, SR::Register>), // Channel Status Register
        (0x30 => fifo: ReadWrite<u32>), // FIFO
        (0x34 => bauddiv: ReadWrite<u32>), // Baud Rate Divider
        (0x38 => flowdel: ReadWrite<u32>), // Flow Delay
        (0x3C => irrx_pwidth: ReadWrite<u32>), // IR Min Received Pulse Width
        (0x40 => irtx_pwidth: ReadWrite<u32>), // IR Transmitted pulse Width
        (0x44 => txwm: ReadWrite<u32>), // TX FIFO Trigger Level
        (0x48 => rxbs: ReadOnly<u32, RXBS::Register>), // RX FIFO byte status register
        (0x4C => @END),
    }
}

register_bitfields! {
    u32,
    CR [
        stop_break OFFSET(8) NUMBITS(1) [],
        start_break OFFSET(7) NUMBITS(1) [],
        tx_disable OFFSET(5) NUMBITS(1) [],
        tx_enable OFFSET(4) NUMBITS(1) [],
        rx_disable OFFSET(3) NUMBITS(1) [],
        rx_enable OFFSET(2) NUMBITS(1) [],
        tx_reset OFFSET(1) NUMBITS(1) [],
        rx_reset OFFSET(0) NUMBITS(1) []
    ],
    MR [
        clock_select OFFSET(0) NUMBITS(1) [
            Normal = 0,
            Div8 = 1
        ],
        char_length OFFSET(1) NUMBITS(2) [
            Bits8 = 0b00,
            Bits7 = 0b10,
            Bits6 = 0b11
        ],
        parity OFFSET(3) NUMBITS(3) [
            Even = 0b000,
            Odd = 0b001,
            Space = 0b010,
            Mark = 0b011,
            None = 0b100
        ],
        stop_mode OFFSET(7) NUMBITS(1) [
            OneStopBit = 0,
            TwoStopBits = 1
        ],
        channel_mode OFFSET(8) NUMBITS(2) [
            Normal = 0b00,
            LocalLoopback = 0b10
        ]
    ],
    RXBS [
        parity_error OFFSET(0) NUMBITS(1) [],
        frame_error OFFSET(1) NUMBITS(1) [],
        overrun_error OFFSET(2) NUMBITS(1) [],
    ],
    IXR [
        // all IER, IDR, IMR, ISR use the same bitfields
        tout OFFSET(8) NUMBITS(1) [],
        parity OFFSET(7) NUMBITS(1) [],
        framing OFFSET(6) NUMBITS(1) [],
        overrun OFFSET(5) NUMBITS(1) [],
        txfull OFFSET(4) NUMBITS(1) [],
        txempty OFFSET(3) NUMBITS(1) [],
        rxfull OFFSET(2) NUMBITS(1) [],
        rxempty OFFSET(1) NUMBITS(1) [],
        rxtrig OFFSET(0) NUMBITS(1) [],
    ],
    MODEMCR [
        fcm OFFSET(5) NUMBITS(1) [],
        rts OFFSET(1) NUMBITS(1) [],
        dtr OFFSET(0) NUMBITS(1) []
    ],
    MODEMSR [
        dcd OFFSET(7) NUMBITS(1) [],
        ri OFFSET(6) NUMBITS(1) [],
        dsr OFFSET(5) NUMBITS(1) [],
        cts OFFSET(4) NUMBITS(1) []
    ],
    SR [
        sr_txfull OFFSET(4) NUMBITS(1) [],
        sr_txempty OFFSET(3) NUMBITS(1) [],
        sr_rxempty OFFSET(1) NUMBITS(1) [],
        sr_rxtrig OFFSET(0) NUMBITS(1) [],
        tactive OFFSET(11) NUMBITS(1) []
    ]
}

struct ZynqUart {
    base_vaddr: VirtAddr,
}

impl ZynqUart {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr }
    }
    const fn regs(&self) -> &ZynqUartRegs {
        unsafe { &*(self.base_vaddr as *const _) }
    }
    /// the UART initialization sequence according to the manual
    /// the baudrate is set to 115200, 8bit data, no parity, 1 stop bit
    fn init(&mut self) {
        // read clock_select from MR, assert it to 0 because we don't use 9600 baudrate
        assert_eq!(
            self.regs().mr.read(MR::clock_select),
            MR::clock_select::Normal.into()
        );
        self.regs()
            .cr
            .modify(CR::tx_disable::SET + CR::rx_disable::SET);
        self.regs().baudgen.set(UART_BAUDRATE_ACTUAL_CD);
        self.regs().bauddiv.set(UART_BAUDRATE_BDIV);
        self.regs().cr.modify(CR::tx_reset::SET + CR::rx_reset::SET);
        self.regs()
            .cr
            .modify(CR::tx_enable::SET + CR::rx_enable::SET);
        self.regs().mr.write(
            MR::char_length::Bits8
                + MR::parity::None
                + MR::stop_mode::OneStopBit
                + MR::channel_mode::Normal,
        );
        self.regs().rxwm.set(0x8);
        self.regs().rxtout.set(0x1);
        self.regs().idr.set(0x1fff);
    }
    fn self_test(&mut self) {
        todo!();
    }
    fn putchar(&mut self, c: u8) {
        // disable interrupts txempty, txfull
        self.regs().idr.write(IXR::txempty::SET + IXR::txfull::SET);
        // check sr that if tx fifo is full, send nothing
        while self.regs().sr.is_set(SR::sr_txfull) {}
        self.regs().fifo.set(c as u32);
    }
    fn getchar(&mut self) -> Option<u8> {
        todo!();
    }
}

pub fn console_putchar(c: u8) {
    UART0.lock().putchar(c)
}

pub fn console_getchar() -> Option<u8> {
    UART0.lock().getchar()
}
