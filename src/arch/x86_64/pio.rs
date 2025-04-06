use core::ops::Range;

use crate::{
    arch::vtd::{PCI_CONFIG_ADDR, PCI_CONFIG_DATA},
    error::HvResult,
    memory::{Frame, HostPhysAddr},
};

pub const UART_COM1_BASE_PORT: u16 = 0x3f8;

#[derive(Debug)]
pub struct PortIoBitmap {
    a: Frame,
    b: Frame,
}

impl PortIoBitmap {
    pub fn new(zoneid: usize) -> Self {
        let mut bitmap = Self {
            a: Frame::new_zero().unwrap(),
            b: Frame::new_zero().unwrap(),
        };

        bitmap.a.fill(0xff);
        bitmap.b.fill(0xff);

        // ban i8259a ports
        bitmap.set_intercept(0x20, true);
        bitmap.set_intercept(0x21, true);
        bitmap.set_intercept(0xa0, true);
        bitmap.set_intercept(0xa1, true);

        // ban pci config ports
        // TODO: handle config space operations from io ports
        bitmap.set_intercept(PCI_CONFIG_ADDR, true);
        bitmap.set_intercept(PCI_CONFIG_DATA, true);
        // bitmap.set_range_intercept(0xcf8..0xd00, true);

        if zoneid == 0 {
            // passthrough uart com1
            bitmap.set_range_intercept(0x3f8..0x400, false);
            // FIXME: get port info from ACPI FACP table
            bitmap.set_intercept(0xb2, false);
            bitmap.set_range_intercept(0x600..0x630, false);
        }

        bitmap
    }

    pub fn bitmap_a_addr(&self) -> HostPhysAddr {
        self.a.start_paddr()
    }

    pub fn bitmap_b_addr(&self) -> HostPhysAddr {
        self.b.start_paddr()
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
