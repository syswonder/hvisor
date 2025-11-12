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
//  Solicey <lzoi_lth@163.com>

use crate::{
    consts::MAX_ZONE_NUM,
    error::HvResult,
    memory::{Frame, HostPhysAddr},
    zone::this_zone_id,
};
use core::ops::Range;
use heapless::FnvIndexMap;

pub const UART_COM1_BASE_PORT: u16 = 0x3f8;
pub const UART_COM1_PORT: Range<u16> = 0x3f8..0x400;
pub const PCI_CONFIG_ADDR_PORT: Range<u16> = 0xcf8..0xcfc;
pub const PCI_CONFIG_DATA_PORT: Range<u16> = 0xcfc..0xd00;

static mut PIO_BITMAP_MAP: Option<FnvIndexMap<usize, PortIoBitmap, MAX_ZONE_NUM>> = None;

pub fn init_pio_bitmap_map() {
    unsafe { PIO_BITMAP_MAP = Some(FnvIndexMap::new()) };
}

pub fn set_pio_bitmap(zone_id: usize) {
    unsafe {
        if let Some(map) = &mut PIO_BITMAP_MAP {
            if map.contains_key(&zone_id) {
                map.remove(&zone_id);
            }
            map.insert(zone_id, PortIoBitmap::new(zone_id));
        }
    }
}

pub fn get_pio_bitmap(zone_id: usize) -> &'static mut PortIoBitmap {
    unsafe {
        PIO_BITMAP_MAP
            .as_mut()
            .expect("PIO_BITMAP_MAP is not initialized!")
            .get_mut(&zone_id)
            .expect("pio bitmap for this Zone does not exist!")
    }
}

#[derive(Debug)]
pub struct PortIoBitmap {
    pub a: Frame,
    pub b: Frame,
    pub pci_config_addr: u32,
}

impl PortIoBitmap {
    pub fn new(zone_id: usize) -> Self {
        let mut bitmap = Self {
            a: Frame::new_zero().unwrap(),
            b: Frame::new_zero().unwrap(),
            pci_config_addr: 0,
        };

        if zone_id == 0 {
            bitmap.a.fill(0);
            bitmap.b.fill(0);
        } else {
            bitmap.a.fill(0xff);
            bitmap.b.fill(0xff);
        }

        // ban i8259a ports
        bitmap.set_intercept(0x20, true);
        bitmap.set_intercept(0x21, true);
        bitmap.set_intercept(0xa0, true);
        bitmap.set_intercept(0xa1, true);

        // pci config ports
        bitmap.set_range_intercept(PCI_CONFIG_ADDR_PORT, true);
        bitmap.set_range_intercept(PCI_CONFIG_DATA_PORT, true);

        if zone_id == 0 {
            #[cfg(feature = "graphics")]
            bitmap.set_range_intercept(UART_COM1_PORT, true);
        }

        // i8042, we won't use it, but intercept its ports might block linux init
        bitmap.set_range_intercept(0x60..0x65, false);

        bitmap
    }

    pub fn set_range_intercept(&mut self, mut ports: Range<u16>, intercept: bool) {
        for port in ports {
            self.set_intercept(port, intercept);
        }
    }

    pub fn set_intercept(&mut self, mut port: u16, intercept: bool) {
        let bitmap = match port <= 0x7fff {
            true => unsafe { core::slice::from_raw_parts_mut(self.a.as_mut_ptr(), 0x1000) },
            false => {
                port -= 0x8000;
                unsafe { core::slice::from_raw_parts_mut(self.b.as_mut_ptr(), 0x1000) }
            }
        };

        let byte = (port / 8) as usize;
        let bits = port % 8;
        if intercept {
            bitmap[byte] |= 1 << bits;
        } else {
            bitmap[byte] &= !(1 << bits);
        }
    }
}
