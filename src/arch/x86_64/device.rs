use crate::{
    device::{irqchip::pic::i8259a::VirtDualI8259a, uart::VirtUart16550a},
    error::HvResult,
};
use alloc::{sync::Arc, vec, vec::Vec};

pub const PIC_MASTER_BASE_PORT: u16 = 0x20;
pub const PIC_SLAVE_BASE_PORT: u16 = 0xa0;
pub const UART_COM1_BASE_PORT: u16 = 0x3f8;

#[allow(non_snake_case)]
pub mod DeviceMsg {
    pub const UPDATE_IRQ_LOW: u8 = 0x0;
    pub const UPDATE_IRQ_HIGH: u8 = 0x1;
}

pub trait PortIoDevice: Send + Sync {
    fn port_range(&self) -> &Vec<core::ops::Range<u16>>;
    fn read(&self, port: u16, msg: u8) -> HvResult<u32>;
    fn write(&self, port: u16, value: u32, msg: u8) -> HvResult;
}

pub struct VirtDeviceList {
    port_io_devices: Vec<Arc<dyn PortIoDevice>>,
}

impl VirtDeviceList {
    pub fn find_port_io_device(&self, port: u16) -> Option<&Arc<dyn PortIoDevice>> {
        self.port_io_devices.iter().find(|dev| {
            dev.port_range()
                .iter()
                .find(|range| range.contains(&port))
                .is_some()
        })
    }

    pub fn send_msg(&self, port: u16, value: u32, msg: u8) {
        if let Some(device) = self.find_port_io_device(port) {
            /*info!(
                "SEND MSG! port: {:x}, value: {:x}, msg: {:x}",
                port, value, msg
            );*/
            device.write(port, value, msg).unwrap();
        }
    }
}

lazy_static::lazy_static! {
    static ref VIRT_DEVICES : VirtDeviceList = VirtDeviceList {
        port_io_devices: vec![
            Arc::new(VirtDualI8259a::new(PIC_MASTER_BASE_PORT, PIC_SLAVE_BASE_PORT)), // Dual PIC
            Arc::new(VirtUart16550a::new(UART_COM1_BASE_PORT)), // COM1
        ],
    };
}

pub fn all_virt_devices() -> &'static VirtDeviceList {
    &VIRT_DEVICES
}
