use core::ops::Range;

use crate::{
    error::HvResult,
    memory::{Frame, HostPhysAddr},
    zone::this_zone_id,
};

pub const UART_COM1_BASE_PORT: u16 = 0x3f8;
pub const PCI_CONFIG_ADDR_PORT: Range<u16> = 0xcf8..0xcfc;
pub const PCI_CONFIG_DATA_PORT: Range<u16> = 0xcfc..0xd00;

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

        bitmap.a.fill(0xff);
        bitmap.b.fill(0xff);

        // ban i8259a ports
        bitmap.set_intercept(0x20, true);
        bitmap.set_intercept(0x21, true);
        bitmap.set_intercept(0xa0, true);
        bitmap.set_intercept(0xa1, true);

        // ban pci config ports
        // TODO: handle config space operations from io ports
        bitmap.set_range_intercept(PCI_CONFIG_ADDR_PORT, true);
        bitmap.set_range_intercept(PCI_CONFIG_DATA_PORT, true);

        // FIXME: uart & i8254
        if zone_id == 0 {
            bitmap.set_range_intercept(0x60..0x65, false);
            bitmap.set_range_intercept(0x3f8..0x400, false);
        }
        // bitmap.set_range_intercept(0x3f8..0x400, false);

        // FIXME: get port info from ACPI FACP table
        bitmap.set_intercept(0xb2, false);
        bitmap.set_range_intercept(0x600..0x630, false);

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
