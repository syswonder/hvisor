use crate::{
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
    pub fn uninit() -> Self {
        Self {
            a: unsafe { Frame::from_paddr(0) },
            b: unsafe { Frame::from_paddr(0) },
        }
    }

    pub fn intercept_def() -> HvResult<Self> {
        let mut bitmap = Self {
            a: Frame::new_zero()?,
            b: Frame::new_zero()?,
        };
        bitmap.a.fill(0xff);
        bitmap.b.fill(0xff);

        for port in UART_COM1_BASE_PORT..UART_COM1_BASE_PORT + 8 {
            bitmap.set_intercept(port, false);
        }

        Ok(bitmap)
    }

    pub fn bitmap_a_addr(&self) -> HostPhysAddr {
        self.a.start_paddr()
    }

    pub fn bitmap_b_addr(&self) -> HostPhysAddr {
        self.b.start_paddr()
    }

    fn set_intercept(&mut self, mut port: u16, intercept: bool) {
        let bitmap = match port <= 0x7fff {
            true => unsafe { core::slice::from_raw_parts_mut(self.a.as_mut_ptr(), 1024) },
            false => {
                port -= 0x8000;
                unsafe { core::slice::from_raw_parts_mut(self.b.as_mut_ptr(), 1024) }
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
