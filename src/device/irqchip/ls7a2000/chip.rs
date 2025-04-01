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
// for 3A5000 board and 7A2000 bridge chip registers
// wheatfox 2024.2.27

use crate::device::common::MMIODerefWrapper;
use alloc::string::String;
use core::ptr::*;
use tock_registers::fields::FieldValue;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

const PHY_ADDR_BITMASK: usize = 0x0000_ffff_ffff_ffff;
const DMW0_PREFIX: usize = 0x8000_0000_0000_0000;
const DMW1_PREFIX: usize = 0x9000_0000_0000_0000;

#[macro_export]
macro_rules! DMW_TO_PHY {
    ($addr:expr) => {
        $addr & 0x0000_ffff_ffff_ffffusize
    };
}

#[macro_export]
macro_rules! PHY_TO_DMW_CACHED {
    ($addr:expr) => {
        $addr | 0x9000_0000_0000_0000usize
    };
}

#[macro_export]
macro_rules! PHY_TO_DMW_UNCACHED {
    ($addr:expr) => {
        $addr | 0x8000_0000_0000_0000usize
    };
}

register_bitfields! [
  u8,
  // Chip Config Version Register
  ChipConfVer [
    VER_NUM OFFSET(0) NUMBITS(8) [],
  ],
];

register_bitfields! [
  // Chip Feature Register
  u16,
  ChipFeature [
    CENTIGRADE OFFSET(0) NUMBITS(1) [],
    NODE_COUNT OFFSET(1) NUMBITS(1) [],
    MSI_SUPPORT OFFSET(2) NUMBITS(1) [],
    EXTIOI_SUPPORT OFFSET(3) NUMBITS(1) [],
    IPI_PERCORE OFFSET(4) NUMBITS(1) [],
    FREQ_PERCORE OFFSET(5) NUMBITS(1) [],
    FREQ_SCALE OFFSET(6) NUMBITS(1) [],
    DVFS_V1_SUPPORT OFFSET(7) NUMBITS(1) [],
    TSENSOR_SUPPORT OFFSET(8) NUMBITS(1) [],
    INT_DECODE OFFSET(9) NUMBITS(1) [],
    LEGACY_MODE OFFSET(10) NUMBITS(1) [],
    GUEST_MODE OFFSET(11) NUMBITS(1) [],
  ],
];

register_bitfields! [
  u64,
  // Manufacturer Name
  ManufacturerName [
    VENDOR OFFSET(0) NUMBITS(64) [],
  ],
  // Chip Name
  ChipName [
    ID OFFSET(0) NUMBITS(64) [],
  ],
  OtherFunctionConfig [
    DISABLE_JTAG OFFSET(0) NUMBITS(1) [],
    DISABLE_JTAG_LA464 OFFSET(1) NUMBITS(1) [],
    DISABLE_LA132 OFFSET(2) NUMBITS(1) [],
    DISABLE_JTAG_LA132 OFFSET(3) NUMBITS(1) [],
    DISABLE_ANITFUSE0 OFFSET(4) NUMBITS(1) [],
    DISABLE_ANITFUSE1 OFFSET(5) NUMBITS(1) [],
    DISABLE_ID OFFSET(6) NUMBITS(1) [],
    EXT_INT_EN OFFSET(48) NUMBITS(1) [],
    INT_ENCODE OFFSET(49) NUMBITS(1) [],
  ],
];

register_structs! {
  #[allow(non_snake_case)]
  pub ChipConfigRegs {
    (0x0000 => pub chip_conf_ver: ReadOnly<u8, ChipConfVer::Register>),
    (0x0001 => _reserved0: [u8; 7]),
    (0x0008 => pub chip_feature: ReadWrite<u16, ChipFeature::Register>),
    (0x000a => _reserved1: [u8; 6]),
    (0x0010 => pub manufacturer_name: ReadOnly<u64, ManufacturerName::Register>),
    (0x0018 => _reserved2: [u8; 8]),
    (0x0020 => pub chip_name: ReadOnly<u64, ChipName::Register>),
    (0x0028 => @END),
  }
}

register_bitfields![
  u32,
  pub Intisr [
    // please refer to manual for detailed description, the field name is simplified
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Inten [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Intenset [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Intenclr [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Intenedge [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Core0Intisr [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Core1Intisr [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Core2Intisr [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
  pub Core3Intisr [
    SC0 OFFSET(0) NUMBITS(1) [],
    SC1 OFFSET(1) NUMBITS(1) [],
    SC2 OFFSET(2) NUMBITS(1) [],
    SC3 OFFSET(3) NUMBITS(1) [],
    GPIO28 OFFSET(4) NUMBITS(1) [],
    GPIO29 OFFSET(5) NUMBITS(1) [],
    GPIO30 OFFSET(6) NUMBITS(1) [],
    GPIO31 OFFSET(7) NUMBITS(1) [],
    I2C0 OFFSET(8) NUMBITS(1) [],
    I2C1 OFFSET(9) NUMBITS(1) [],
    UART0 OFFSET(10) NUMBITS(1) [],
    MC0 OFFSET(11) NUMBITS(1) [],
    MC1 OFFSET(12) NUMBITS(1) [],
    SPI OFFSET(13) NUMBITS(1) [],
    THSENS OFFSET(14) NUMBITS(1) [],
    UART1 OFFSET(15) NUMBITS(1) [],
    HT0 OFFSET(16) NUMBITS(8) [],
    HT1 OFFSET(24) NUMBITS(8) [],
  ],
];

// 3A5000 manual p73
register_structs! {
  #[allow(non_snake_case)]
  pub ChipLegacyIntCtrlRegs {
    (0x0000 => pub int_isr: ReadWrite<u32, Intisr::Register>),
    (0x0004 => pub int_en: ReadOnly<u32, Inten::Register>),
    (0x0008 => pub int_en_set: ReadWrite<u32, Intenset::Register>),
    (0x000c => pub int_en_clr: ReadWrite<u32, Intenclr::Register>),
    (0x0010 => _reserved0: [u8; 4]),
    (0x0014 => pub int_en_edge: ReadWrite<u32, Intenedge::Register>),
    (0x0018 => _reserved1: [u8; 8]),
    (0x0020 => pub core0_intisr: ReadWrite<u32, Core0Intisr::Register>),
    (0x0024 => _reserved2: [u8; 4]),
    (0x0028 => pub core1_intisr: ReadWrite<u32, Core1Intisr::Register>),
    (0x002c => _reserved3: [u8; 4]),
    (0x0030 => pub core2_intisr: ReadWrite<u32, Core2Intisr::Register>),
    (0x0034 => _reserved4: [u8; 4]),
    (0x0038 => pub core3_intisr: ReadWrite<u32, Core3Intisr::Register>),
    (0x003c => @END),
  }
}

register_structs! {
  #[allow(non_snake_case)]
  pub ChipOtherFunctionRegs {
    (0x0000 => pub other_function_config: ReadWrite<u64, OtherFunctionConfig::Register>),
    (0x0008 => @END),
  }
}

register_structs! {
  #[allow(non_snake_case)]
  pub ChipLegacyIntRouteRegs {
    // [3:0] route cpu core id (CPU0-CPU3 on 3A5000), for example, 4'b0101 means CPU0 and CPU2
    // [7:4] route cpu core int pin id (INT0-INT3 on 3A5000), for example, 4'b0010 means INT1
    (0x0000 => pub entry0: ReadWrite<u8>), // GPIO24/16/8/0
    (0x0001 => pub entry1: ReadWrite<u8>), // GPIO25/17/9/1
    (0x0002 => pub entry2: ReadWrite<u8>), // GPIO26/18/10/2
    (0x0003 => pub entry3: ReadWrite<u8>), // GPIO27/19/11/3
    (0x0004 => pub entry4: ReadWrite<u8>), // GPIO28/20/12/4
    (0x0005 => pub entry5: ReadWrite<u8>), // GPIO29/21/13/5
    (0x0006 => pub entry6: ReadWrite<u8>), // GPIO30/22/14/6
    (0x0007 => pub entry7: ReadWrite<u8>), // GPIO31/23/15/7
    (0x0008 => pub entry8: ReadWrite<u8>), // I2C0
    (0x0009 => pub entry9: ReadWrite<u8>), // I2C1
    (0x000a => pub entry10: ReadWrite<u8>), // UART0
    (0x000b => pub entry11: ReadWrite<u8>), // MC0
    (0x000c => pub entry12: ReadWrite<u8>), // MC1
    (0x000d => pub entry13: ReadWrite<u8>), // SPI
    (0x000e => pub entry14: ReadWrite<u8>), // THSENS
    (0x000f => pub entry15: ReadWrite<u8>), // UART1
    (0x0010 => pub entry16: ReadWrite<u8>), // HT0-INT0
    (0x0011 => pub entry17: ReadWrite<u8>), // HT0-INT1
    (0x0012 => pub entry18: ReadWrite<u8>), // HT0-INT2
    (0x0013 => pub entry19: ReadWrite<u8>), // HT0-INT3
    (0x0014 => pub entry20: ReadWrite<u8>), // HT0-INT4
    (0x0015 => pub entry21: ReadWrite<u8>), // HT0-INT5
    (0x0016 => pub entry22: ReadWrite<u8>), // HT0-INT6
    (0x0017 => pub entry23: ReadWrite<u8>), // HT0-INT7
    (0x0018 => pub entry24: ReadWrite<u8>), // HT1-INT0
    (0x0019 => pub entry25: ReadWrite<u8>), // HT1-INT1
    (0x001a => pub entry26: ReadWrite<u8>), // HT1-INT2
    (0x001b => pub entry27: ReadWrite<u8>), // HT1-INT3
    (0x001c => pub entry28: ReadWrite<u8>), // HT1-INT4
    (0x001d => pub entry29: ReadWrite<u8>), // HT1-INT5
    (0x001e => pub entry30: ReadWrite<u8>), // HT1-INT6
    (0x001f => pub entry31: ReadWrite<u8>), // HT1-INT7
    (0x0020 => @END),
  }
}

register_bitfields![
  u64,
  Extioi_en0 [
    EXTIOI_EN0 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_en1 [
    EXTIOI_EN1 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_en2 [
    EXTIOI_EN2 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_en3 [
    EXTIOI_EN3 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_sr0 [
    EXTIOI_SR0 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_sr1 [
    EXTIOI_SR1 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_sr2 [
    EXTIOI_SR2 OFFSET(0) NUMBITS(63) []
  ],
  Extioi_sr3 [
    EXTIOI_SR3 OFFSET(0) NUMBITS(63) []
  ],
];

// 3A5000 manual p75
register_structs! {
  #[allow(non_snake_case)]
  pub ChipExtioiEnableRegs {
    (0x0000 => pub extioi_en0: ReadWrite<u64, Extioi_en0::Register>),
    (0x0008 => pub extioi_en1: ReadWrite<u64, Extioi_en1::Register>),
    (0x0010 => pub extioi_en2: ReadWrite<u64, Extioi_en2::Register>),
    (0x0018 => pub extioi_en3: ReadWrite<u64, Extioi_en3::Register>),
    (0x0020 => @END),
  }
}

register_structs! {
  #[allow(non_snake_case)]
  pub ChipExtioiBounceRegs {
    (0x0000 => pub extioi_bounce0: ReadWrite<u64>),
    (0x0008 => pub extioi_bounce1: ReadWrite<u64>),
    (0x0010 => pub extioi_bounce2: ReadWrite<u64>),
    (0x0018 => pub extioi_bounce3: ReadWrite<u64>),
    (0x0020 => @END),
  }
}

register_structs! {
  #[allow(non_snake_case)]
  pub ChipExtioiStatusRegs {
    (0x0000 => pub extioi_sr0: ReadOnly<u64, Extioi_sr0::Register>),
    (0x0008 => pub extioi_sr1: ReadOnly<u64, Extioi_sr1::Register>),
    (0x0010 => pub extioi_sr2: ReadOnly<u64, Extioi_sr2::Register>),
    (0x0018 => pub extioi_sr3: ReadOnly<u64, Extioi_sr3::Register>),
    (0x0020 => @END),
  }
}

register_structs! {
  #[allow(non_snake_case)]
  pub ChipExtioiRouteRegs {
    (0x0000 => pub extioi_map0: ReadWrite<u8>),
    (0x0001 => pub extioi_map1: ReadWrite<u8>),
    (0x0002 => pub extioi_map2: ReadWrite<u8>),
    (0x0003 => pub extioi_map3: ReadWrite<u8>),
    (0x0004 => pub extioi_map4: ReadWrite<u8>),
    (0x0005 => pub extioi_map5: ReadWrite<u8>),
    (0x0006 => pub extioi_map6: ReadWrite<u8>),
    (0x0007 => pub extioi_map7: ReadWrite<u8>),
    (0x0008 => @END),
  }
}

const MMIO_BASE: usize = PHY_TO_DMW_UNCACHED!(0x1fe0_0000);

const CHIP_CONFIG_BASE: usize = MMIO_BASE;
pub static CHIP_CONFIG: MMIODerefWrapper<ChipConfigRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_CONFIG_BASE as usize) };

const CHIP_LEGACY_INT_CTRL_BASE: usize = MMIO_BASE + 0x1420;
pub static CHIP_LEGACY_INT_CTRL: MMIODerefWrapper<ChipLegacyIntCtrlRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_LEGACY_INT_CTRL_BASE as usize) };

const CHIP_OTHER_FUNCTION_BASE: usize = MMIO_BASE + 0x420;
pub static CHIP_OTHER_FUNCTION: MMIODerefWrapper<ChipOtherFunctionRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_OTHER_FUNCTION_BASE as usize) };

const CHIP_LEGACY_INT_ROUTE_BASE: usize = MMIO_BASE + 0x1400;
pub static CHIP_LEGACY_INT_ROUTE: MMIODerefWrapper<ChipLegacyIntRouteRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_LEGACY_INT_ROUTE_BASE as usize) };

const CHIP_EXTIOI_ENABLE_BASE: usize = MMIO_BASE + 0x1600;
pub static CHIP_EXTIOI_ENABLE: MMIODerefWrapper<ChipExtioiEnableRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_EXTIOI_ENABLE_BASE as usize) };

const CHIP_EXTIOI_STATUS_BASE: usize = MMIO_BASE + 0x1700;
pub static CHIP_EXTIOI_STATUS: MMIODerefWrapper<ChipExtioiStatusRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_EXTIOI_STATUS_BASE as usize) };

// this indicates the configs for irq routing to which INT pin, not target cpu core
// the 256 irqs are grouped into 8 group to control the target INT pin - wheatfox
const CHIP_EXTIOI_ROUTE_BASE: usize = MMIO_BASE + 0x14c2;
pub static CHIP_EXTIOI_ROUTE: MMIODerefWrapper<ChipExtioiRouteRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_EXTIOI_ROUTE_BASE as usize) };

const CHIP_EXTIOI_BOUNCE_BASE: usize = MMIO_BASE + 0x1680;
pub static CHIP_EXTIOI_BOUNCE: MMIODerefWrapper<ChipExtioiBounceRegs> =
    unsafe { MMIODerefWrapper::new(CHIP_EXTIOI_BOUNCE_BASE as usize) };

const CHIP_EXTIOI_DEBUG_SEND_BASE: usize = MMIO_BASE + 0x1140;

// this is the target cpu core for all 256 irq sources - wheatfox
const CHIP_EXTIOI_ROUTE_CORE_BASE: usize = MMIO_BASE + 0x1c00;
const CHIP_EXTIOI_NODE_TYPE_BASE: usize = MMIO_BASE + 0x14a0;

// 3A5000 manual p118
const CHIP_HT_CONFIG_BASE: usize = PHY_TO_DMW_UNCACHED!(0xfd_fb00_0000);
const CHIP_HT_INT_VECTOR_BASE: usize = CHIP_HT_CONFIG_BASE + 0x80;
const CHIP_HT_INT_EN_BASE: usize = CHIP_HT_CONFIG_BASE + 0xa0;

/******************************************** */
/*             SOME BASIC FUCNTIONS           */
/******************************************** */

pub fn get_chip_conf_ver() -> u64 {
    CHIP_CONFIG.chip_conf_ver.read(ChipConfVer::VER_NUM) as u64
}

pub fn get_chip_ht_device_id() -> usize {
    // offset 0x00, size 0x2 bytes
    let mut device_id: usize;
    unsafe {
        device_id = read_volatile((CHIP_HT_CONFIG_BASE + 0x00) as *const u16) as usize;
    }
    device_id
}

pub fn get_chip_ht_vendor_id() -> usize {
    // offset 0x02, size 0x2 bytes
    let mut vendor_id: usize;
    unsafe {
        vendor_id = read_volatile((CHIP_HT_CONFIG_BASE + 0x02) as *const u16) as usize;
    }
    vendor_id
}

pub fn debug_set_extioi_intvec(irq: u8) {
    unsafe {
        write_volatile((CHIP_EXTIOI_DEBUG_SEND_BASE + 0x00) as *mut u8, irq);
    }
}

pub fn get_ipi_percore() -> bool {
    CHIP_CONFIG.chip_feature.read(ChipFeature::IPI_PERCORE) != 0
}

pub fn get_guest_mode() -> bool {
    CHIP_CONFIG.chip_feature.read(ChipFeature::GUEST_MODE) != 0
}

pub fn set_guest_mode() {
    CHIP_CONFIG
        .chip_feature
        .modify(ChipFeature::GUEST_MODE::SET);
}

pub fn clear_guest_mode() {
    CHIP_CONFIG
        .chip_feature
        .modify(ChipFeature::GUEST_MODE::CLEAR);
}

fn u64tostr(x: u64) -> String {
    // 0x00003030_30354133 to 3A5000
    let mut s = String::new();
    for i in 0..8 {
        let c = (x >> (i * 8)) & 0xff;
        if c == 0 {
            break;
        }
        s.push(c as u8 as char);
    }
    s
}

#[no_mangle]
pub fn print_chip_info() {
    info!(
        "loongarch64: print_chip_info: chip config version: {:#x}",
        get_chip_conf_ver()
    );
    info!(
        "loongarch64: print_chip_info: chip feature extioi support: {}",
        CHIP_CONFIG.chip_feature.read(ChipFeature::EXTIOI_SUPPORT) != 0
    );
    info!(
        "loongarch64: print_chip_info: manufacturer name: {}",
        u64tostr(CHIP_CONFIG.manufacturer_name.read(ManufacturerName::VENDOR))
    );
    info!(
        "loongarch64: print_chip_info: chip name: {}",
        u64tostr(CHIP_CONFIG.chip_name.read(ChipName::ID))
    );
}

/******************************************** */
/*            LEGACY INT FUCNTIONS            */
/******************************************** */

pub fn legacy_int_enable_all() {
    CHIP_LEGACY_INT_CTRL.int_en_set.modify(Intenset::UART0::SET);
    // CHIP_LEGACY_INT_CTRL.int_en_set.set(0xffff_ffff);
}

pub fn csr_disable_new_codec() {
    // set CSR[0x420][49] to false to use legacy vector mask route
    use core::arch::asm;
    let mut tmp_: usize;
    unsafe {
        asm!("csrrd {}, 0x420", out(reg) tmp_);
    }
    tmp_ &= !(1 << 49);
    unsafe {
        asm!("csrwr {}, 0x420", in(reg) tmp_);
    }
}

pub fn legacy_int_route_all() {
    // route to CPU0 INT0 for testing
    let cpu_mask = 0b0001; // route to CPU0
    let int_mask = 0b0001; // route to INT0
    let mask = (int_mask << 4) | cpu_mask;
    CHIP_LEGACY_INT_ROUTE.entry10.set(mask);
    CHIP_LEGACY_INT_ROUTE.entry31.set(mask);
    info!("(legacy_int_route_all) route all legacy int to CPU0 INT0");
}

pub fn legacy_int_dump() {
    info!(
        "(legacy_int_dump) int_isr_raw = 0x{:x}",
        CHIP_LEGACY_INT_CTRL.int_isr.get()
    );
    info!(
        "(legacy_int_dump) int_en_raw = 0x{:x}",
        CHIP_LEGACY_INT_CTRL.int_en.get()
    );
    info!(
        "(legacy_int_dump) int_en_set_raw = 0x{:x}",
        CHIP_LEGACY_INT_CTRL.int_en_set.get()
    );
    info!(
        "(legacy_int_dump) int_en_clr_raw = 0x{:x}",
        CHIP_LEGACY_INT_CTRL.int_en_clr.get()
    );
    info!(
        "(legacy_int_dump) int_en_edge_raw = 0x{:x}",
        CHIP_LEGACY_INT_CTRL.int_en_edge.get()
    );
    info!(
        "(legacy_int_dump) core0_intisr_raw = 0x{:x}",
        CHIP_LEGACY_INT_CTRL.core0_intisr.get()
    );
}

/******************************************** */
/*            EXTIOI FUCNTIONS               */
/******************************************** */

pub fn extioi_mode_enable() {
    CHIP_OTHER_FUNCTION
        .other_function_config
        .modify(OtherFunctionConfig::EXT_INT_EN::SET);
}

pub fn extioi_mode_disable() {
    CHIP_OTHER_FUNCTION
        .other_function_config
        .modify(OtherFunctionConfig::EXT_INT_EN::CLEAR);
}

pub fn extioi_is_enabled() -> bool {
    let status = CHIP_OTHER_FUNCTION
        .other_function_config
        .read(OtherFunctionConfig::EXT_INT_EN);
    status != 0
}

pub fn extioi_int_enable_all() {
    CHIP_EXTIOI_ENABLE.extioi_en0.set(0xffff_ffff_ffff_ffff);
    CHIP_EXTIOI_ENABLE.extioi_en1.set(0xffff_ffff_ffff_ffff);
    CHIP_EXTIOI_ENABLE.extioi_en2.set(0xffff_ffff_ffff_ffff);
    CHIP_EXTIOI_ENABLE.extioi_en3.set(0xffff_ffff_ffff_ffff);
    #[cfg(feature = "extioi_debug")]
    {
        // dump ht int vector and enable
        /*
        info!("(extioi_int_enable_all) ht int vector dump:");
        for i in 0..8 {
          let addr = CHIP_HT_INT_VECTOR_BASE + i * 32;
          let mut val: u32;
          unsafe {
            val = read_volatile(addr as *const u32);
          }
          info!("(extioi_int_enable_all) ht int vector[{}]: 0x{:x}", i, val);
        }
        */
        // enable all int in HT config reg
        for i in 0..8 {
            let addr = CHIP_HT_INT_EN_BASE + i * 32;
            unsafe {
                write_volatile(addr as *mut u32, 0xffff_ffff);
            }
        }
        info!("(extioi_int_enable_all) ht int enable dump:");
        for i in 0..8 {
            let addr = CHIP_HT_INT_EN_BASE + i * 32;
            let mut val: u32;
            unsafe {
                val = read_volatile(addr as *const u32);
            }
            info!("(extioi_int_enable_all) ht int enable[{}]: 0x{:x}", i, val);
        }
    }
}

// rcore refenrence
// https://github.com/Godones/rCoreloongArch/blob/master/kernel/src/loongarch/extioi.rs
pub fn extioi_int_route_pin_all() {
    csr_disable_new_codec(); // use legacy vector codec
    let mask = 0b0000_0001u8; // INT1
    CHIP_EXTIOI_ROUTE.extioi_map0.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map1.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map2.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map3.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map4.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map5.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map6.set(mask);
    CHIP_EXTIOI_ROUTE.extioi_map7.set(mask);
}

pub fn extioi_int_route_core_all() {
    // first disable all extioi bounce
    CHIP_EXTIOI_BOUNCE.extioi_bounce0.set(0);
    CHIP_EXTIOI_BOUNCE.extioi_bounce1.set(0);
    CHIP_EXTIOI_BOUNCE.extioi_bounce2.set(0);
    CHIP_EXTIOI_BOUNCE.extioi_bounce3.set(0);
    // write CHIP_EXTIOI_NODE_TYPE_BASE mmio (each with 2 bytes)
    unsafe {
        // set to 16'b0000_0000_0000_0001
        // which means only trigger node0
        core::ptr::write_volatile(CHIP_EXTIOI_NODE_TYPE_BASE as *mut u16, 0x0001);
    }
    // from CHIP_EXTIOI_ROUTE_CORE_BASE tp CHIP_EXTIOI_ROUTE_CORE_BASE + 0xff
    // for each MMIO byte, set to cput core 0, data = 8'b0000_0001
    // which is [EXT_IOI_node_type0][CPU0 MASK]
    let mask = 0b0000_0001u8;
    for i in 0..256 {
        let addr = CHIP_EXTIOI_ROUTE_CORE_BASE + i;
        unsafe {
            core::ptr::write_volatile(addr as *mut u8, mask);
        }
    }
}

pub fn extioi_dump() {
    info!(
        "(extioi_dump) extioi_en0=0x{:x}",
        CHIP_EXTIOI_ENABLE.extioi_en0.get()
    );
    info!(
        "(extioi_dump) extioi_en1=0x{:x}",
        CHIP_EXTIOI_ENABLE.extioi_en1.get()
    );
    info!(
        "(extioi_dump) extioi_en2=0x{:x}",
        CHIP_EXTIOI_ENABLE.extioi_en2.get()
    );
    info!(
        "(extioi_dump) extioi_en3=0x{:x}",
        CHIP_EXTIOI_ENABLE.extioi_en3.get()
    );
    info!(
        "(extioi_dump) extioi_bounce0=0x{:x}",
        CHIP_EXTIOI_BOUNCE.extioi_bounce0.get()
    );
    info!(
        "(extioi_dump) extioi_bounce1=0x{:x}",
        CHIP_EXTIOI_BOUNCE.extioi_bounce1.get()
    );
    info!(
        "(extioi_dump) extioi_bounce2=0x{:x}",
        CHIP_EXTIOI_BOUNCE.extioi_bounce2.get()
    );
    info!(
        "(extioi_dump) extioi_bounce3=0x{:x}",
        CHIP_EXTIOI_BOUNCE.extioi_bounce3.get()
    );
    info!(
        "(extioi_dump) extioi_sr0=0x{:x}",
        CHIP_EXTIOI_STATUS.extioi_sr0.get()
    );
    info!(
        "(extioi_dump) extioi_sr1=0x{:x}",
        CHIP_EXTIOI_STATUS.extioi_sr1.get()
    );
    info!(
        "(extioi_dump) extioi_sr2=0x{:x}",
        CHIP_EXTIOI_STATUS.extioi_sr2.get()
    );
    info!(
        "(extioi_dump) extioi_sr3=0x{:x}",
        CHIP_EXTIOI_STATUS.extioi_sr3.get()
    );
}

/******************************************** */
/*             PCI STUFFS :)                  */
/******************************************** */

const PCI_STANDARD_CONFIG_BASE_ALT: usize = 0x8000_0000_1a00_0000;
const PCI_STANDARD_CONFIG_BASE: usize = 0x8000_0efd_fe00_0000;
const PCI_RESERVED_CONFIG_BASE: usize = 0x8000_0efe_0000_0000;

/**

Standard PCI config space:
TYPE0: [15:11] Device Number, [10:8] Function Number, [7:0] Offset
TYPE1: [23:16] Bus Number, [15:11] Device Number, [10:8] Function Number, [7:0] Offset

Reserved PCI config space:
TYPE0: [27:24] Offset[11:8], [15:11] Device Number, [10:8] Function Number, [7:0] Offset[7:0]
TYPE1: [27:24] Offset[11:8], [23:16] Bus Number, [15:11] Device Number, [10:8] Function Number, [7:0] Offset[7:0]

*/

pub fn probe_pci_config_standard_ecam(
    bus: u8,
    device: u8,
    function: u8,
    offset: u8,
    size: u8,
) -> usize {
    let mut addr: usize;
    let mut data: usize;
    addr = PCI_STANDARD_CONFIG_BASE_ALT
        | ((bus as usize) << 16)
        | ((device as usize) << 11)
        | ((function as usize) << 8)
        | (offset as usize);
    data = 0;
    for i in 0..size {
        let byte_addr = addr + i as usize;
        let byte_data: u8;
        unsafe {
            byte_data = read_volatile(byte_addr as *const u8);
        }
        data |= (byte_data as usize) << (i * 8);
    }
    data
}

pub fn probe_pci_config_standard(bus: u8, device: u8, function: u8, offset: u8, size: u8) -> usize {
    let mut addr: usize;
    let mut data: usize;
    addr = PCI_STANDARD_CONFIG_BASE
        | ((bus as usize) << 16)
        | ((device as usize) << 11)
        | ((function as usize) << 8)
        | (offset as usize);
    data = 0;
    for i in 0..size {
        let byte_addr = addr + i as usize;
        let byte_data: u8;
        unsafe {
            byte_data = read_volatile(byte_addr as *const u8);
        }
        data |= (byte_data as usize) << (i * 8);
    }
    data
}
pub fn probe_pci_config_reserved(
    bus: u8,
    device: u8,
    function: u8,
    offset: usize,
    size: u8,
) -> usize {
    let mut addr: usize;
    let mut data: usize;
    let offset_low = offset & 0xff;
    let offset_high = (offset >> 8) & 0xf;
    addr = PCI_RESERVED_CONFIG_BASE
        | ((bus as usize) << 16)
        | ((device as usize) << 11)
        | ((function as usize) << 8)
        | (offset_low as usize)
        | (offset_high << 24);
    data = 0;
    for i in 0..size {
        let byte_addr = addr + i as usize;
        let byte_data: u8;
        unsafe {
            byte_data = read_volatile(byte_addr as *const u8);
        }
        data |= (byte_data as usize) << (i * 8);
    }
    data
}

// https://admin.pci-ids.ucw.cz/read/PC/0014

const PCI_VENDOR_ID_LOONGSON: usize = 0x0014;
const PCI_DEVICE_ID_HT_BRIDGE: usize = 0x7a00;
const PCI_DEVICE_ID_APB: usize = 0x7a02;
const PCI_DEVICE_ID_GIGE: usize = 0x7a03;
const PCI_DEVICE_ID_OTG_USB: usize = 0x7a04;
const PCI_DEVICE_ID_GPU: usize = 0x7a05;
const PCI_DEVICE_ID_DC: usize = 0x7a06;
const PCI_DEVICE_ID_HDA: usize = 0x7a07;
const PCI_DEVICE_ID_SATA: usize = 0x7a08;
const PCI_DEVICE_ID_PCI_BRIDGE: usize = 0x7a09;
const PCI_DEVICE_ID_SPI: usize = 0x7a0b;
const PCI_DEVICE_ID_LPC: usize = 0x7a0c;
const PCI_DEVICE_ID_DMA: usize = 0x7a0f;
const PCI_DEVICE_ID_HT_BRIDGE2: usize = 0x7a10;
const PCI_DEVICE_ID_PCH_GIGE: usize = 0x7a13;
const PCI_DEVICE_ID_EHCI_USB: usize = 0x7a14;
const PCI_DEVICE_ID_GPU2: usize = 0x7a15;
const PCI_DEVICE_ID_SATA3: usize = 0x7a18;
const PCI_DEVICE_ID_PCI_BRIDGE2: usize = 0x7a19;
const PCI_DEVICE_ID_SPI2: usize = 0x7a1b;
const PCI_DEVICE_ID_OHCI_USB: usize = 0x7a24;
const PCI_DEVICE_ID_LG100_GPU: usize = 0x7a25;
const PCI_DEVICE_ID_I2S: usize = 0x7a27;
const PCI_DEVICE_ID_PCI_BRIDGE3: usize = 0x7a29;
const PCI_DEVICE_ID_XHCI_USB: usize = 0x7a34;
const PCI_DEVICE_ID_DC2: usize = 0x7a36;
const PCI_DEVICE_ID_PCIE_X1: usize = 0x7a39;
const PCI_DEVICE_ID_PCIE_X4: usize = 0x7a49;
const PCI_DEVICE_ID_PCIE_X8: usize = 0x7a59;
const PCI_DEVICE_ID_PCIE_X16: usize = 0x7a69;

pub fn parse_vendor_device_id(vendor_id: usize, device_id: usize) -> String {
    let mut name = String::new();
    if vendor_id == PCI_VENDOR_ID_LOONGSON {
        name.push_str(format!("[{}] ", "Loongson Technology LLC").as_str());
        match device_id {
            PCI_DEVICE_ID_HT_BRIDGE => name.push_str("Hyper Transport Bridge Controller	"),
            PCI_DEVICE_ID_APB => name.push_str("APB (Advanced Peripheral Bus) Controller"),
            PCI_DEVICE_ID_GIGE => name.push_str("Gigabit Ethernet Controller"),
            PCI_DEVICE_ID_OTG_USB => name.push_str("OTG USB Controller"),
            PCI_DEVICE_ID_GPU => name.push_str("Vivante GPU"),
            PCI_DEVICE_ID_DC => name.push_str("Display Controller"),
            PCI_DEVICE_ID_HDA => name.push_str("HDA (High Definition Audio) Controller"),
            PCI_DEVICE_ID_SATA => name.push_str("SATA AHCI Controller"),
            PCI_DEVICE_ID_PCI_BRIDGE => name.push_str("PCI-to-PCI Bridge"),
            PCI_DEVICE_ID_SPI => name.push_str("SPI Controller"),
            PCI_DEVICE_ID_LPC => name.push_str("LPC Controller"),
            PCI_DEVICE_ID_DMA => name.push_str("DMA (Direct Memory Access) Controller"),
            PCI_DEVICE_ID_HT_BRIDGE2 => name.push_str("Hyper Transport Bridge Controller"),
            PCI_DEVICE_ID_PCH_GIGE => name.push_str("7A2000 PCH Gigabit Ethernet Controller"),
            PCI_DEVICE_ID_EHCI_USB => name.push_str("EHCI USB Controller"),
            PCI_DEVICE_ID_GPU2 => name.push_str("Vivante GPU"),
            PCI_DEVICE_ID_SATA3 => name.push_str("SATA 3 AHCI Controller"),
            PCI_DEVICE_ID_PCI_BRIDGE2 => name.push_str("PCI-to-PCI Bridge"),
            PCI_DEVICE_ID_SPI2 => name.push_str("SPI Controller"),
            PCI_DEVICE_ID_OHCI_USB => name.push_str("OHCI USB Controller"),
            PCI_DEVICE_ID_LG100_GPU => name.push_str("LG100 GPU"),
            PCI_DEVICE_ID_I2S => name.push_str("7A2000 PCH I2S Controller"),
            PCI_DEVICE_ID_PCI_BRIDGE3 => name.push_str("PCI-to-PCI Bridge"),
            PCI_DEVICE_ID_XHCI_USB => name.push_str("xHCI USB Controller"),
            PCI_DEVICE_ID_DC2 => name.push_str("Display Controller"),
            PCI_DEVICE_ID_PCIE_X1 => name.push_str("PCIe x1 Root Port"),
            PCI_DEVICE_ID_PCIE_X4 => name.push_str("PCIe x4 Root Port"),
            PCI_DEVICE_ID_PCIE_X8 => name.push_str("PCIe x8 Root Port"),
            PCI_DEVICE_ID_PCIE_X16 => name.push_str("PCIe x16 Root Port"),
            _ => name.push_str("Unknown Device"),
        }
    }
    if name.is_empty() {
        name.push_str("Unknown");
    }
    name
}

pub fn probe_pci() {
    // probe 12 devices using standard config space
    warn!(
        "loongarch64: probe_pci: probing PCI devices @ 0x{:x}",
        PCI_STANDARD_CONFIG_BASE_ALT
    );
    let mut num = 64;
    for i in 0..num {
        // dump vendor id and device id
        let vendor_id = probe_pci_config_standard_ecam(0, i, 0, 0, 2);
        let device_id = probe_pci_config_standard_ecam(0, i, 0, 2, 2);
        if vendor_id == 0xffff && device_id == 0xffff {
            continue;
        }
        info!(
            "loongarch64: probe_pci: device {}: vendor id = {:#x}, device id = {:#x}, name = {}",
            i,
            vendor_id,
            device_id,
            parse_vendor_device_id(vendor_id, device_id)
        );
    }
    // probe 12 devices using standard config space
    warn!(
        "loongarch64: probe_pci: probing PCI devices @ 0x{:x}",
        PCI_STANDARD_CONFIG_BASE
    );
    for i in 0..num {
        // dump vendor id and device id
        let vendor_id = probe_pci_config_standard(0, i, 0, 0, 2);
        let device_id = probe_pci_config_standard(0, i, 0, 2, 2);
        if vendor_id == 0xffff && device_id == 0xffff {
            continue;
        }
        info!(
            "loongarch64: probe_pci: device {}: vendor id = {:#x}, device id = {:#x}, name = {}",
            i,
            vendor_id,
            device_id,
            parse_vendor_device_id(vendor_id, device_id)
        );
    }
    // probe 12 devices using reserved config space
    warn!(
        "loongarch64: probe_pci: probing PCI devices @ 0x{:x}",
        PCI_RESERVED_CONFIG_BASE
    );
    for i in 0..num {
        // dump vendor id and device id
        let vendor_id = probe_pci_config_reserved(0, i, 0, 0, 2);
        let device_id = probe_pci_config_reserved(0, i, 0, 2, 2);
        if vendor_id == 0xffff && device_id == 0xffff {
            continue;
        }
        info!(
            "loongarch64: probe_pci: device {}: vendor id = {:#x}, device id = {:#x}, name = {}",
            i,
            vendor_id,
            device_id,
            parse_vendor_device_id(vendor_id, device_id)
        );
    }
}
