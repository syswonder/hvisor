use crate::{arch::device::PortIoDevice, error::HvResult};

pub struct VirtI8259Pic {
    port_base: u16,
}

impl PortIoDevice for VirtI8259Pic {
    fn port_range(&self) -> core::ops::Range<u16> {
        self.port_base..self.port_base + 2
    }

    fn read(&self, port: u16, access_size: u8) -> HvResult<u32> {
        hv_result_err!(EIO) // report error for read
    }

    fn write(&self, port: u16, access_size: u8, value: u32) -> HvResult {
        Ok(()) // ignore write
    }
}

impl VirtI8259Pic {
    pub const fn new(port_base: u16) -> Self {
        Self { port_base }
    }
}
