use crate::error::HvResult;
use crate::pci::pci_struct::PciConfigSpace;
use crate::pci::pci_access::{EndpointField, Bar};
use crate::pci::PciConfigAddress;
use super::{PciConfigAccessStatus, VpciDeviceHandler};
// use crate::memory::frame::Frame;
use crate::pci::pci_struct::ArcRwLockVirtualPciConfigSpace;
use crate::percpu::this_zone;
use crate::memory::MMIOAccess;
use crate::pci::pci_access::PciMemType;

const STANDARD_VENDOR_ID: u16 = 0x110a;
const STANDARD_DEVICE_ID: u16 = 0x4106;
const PCI_STS_CAPS: u16 = 0x10; // bit 4
const PCI_DEV_CLASS_OTHER: u8 = 0xff;
const PCI_CFG_CAPS: usize = 0x34;
const PCI_CAP_ID_VNDR: u8 = 0x09;
const PCI_CAP_ID_MSIX: u8 = 0x11;
const STANDARD_CFG_VNDR_CAP: u8 = 0x40;
const STANDARD_CFG_VNDR_LEN: u8 = 0x20;
const STANDARD_CFG_MSIX_CAP: usize = 0x60; // VNDR_CAP + VNDR_LEN
const STANDARD_MSIX_VECTORS: u16 = 16;
const STANDARD_CFG_SIZE: usize = 0x80;

pub(crate) const DEFAULT_CSPACE_U32: [u32; STANDARD_CFG_SIZE / 4] = {
    let mut arr = [0u32; STANDARD_CFG_SIZE / 4];
    arr[0x00 / 4] = (STANDARD_DEVICE_ID as u32) << 16 | STANDARD_VENDOR_ID as u32;
    arr[0x04 / 4] = (PCI_STS_CAPS as u32) << 16;
    arr[0x08 / 4] = (PCI_DEV_CLASS_OTHER as u32) << 24;
    arr[0x2c / 4] = (STANDARD_DEVICE_ID as u32) << 16 | STANDARD_VENDOR_ID as u32;
    arr[PCI_CFG_CAPS / 4] = STANDARD_CFG_VNDR_CAP as u32;
    arr[STANDARD_CFG_VNDR_CAP as usize / 4] = (STANDARD_CFG_VNDR_LEN as u32) << 16
        | (STANDARD_CFG_MSIX_CAP as u32) << 8
        | PCI_CAP_ID_VNDR as u32;
    arr[STANDARD_CFG_MSIX_CAP / 4] = (0x00u32) << 8 | PCI_CAP_ID_MSIX as u32;
    arr[(STANDARD_CFG_MSIX_CAP + 0x4) / 4] = 1;
    arr[(STANDARD_CFG_MSIX_CAP + 0x8) / 4] = ((0x10 * STANDARD_MSIX_VECTORS) as u32) | 1;
    arr
};

/// Handler for standard virtual PCI devices
pub struct StandardHandler;

impl VpciDeviceHandler for StandardHandler {
    fn read_cfg(&self, dev: ArcRwLockVirtualPciConfigSpace, offset: PciConfigAddress, size: usize) -> HvResult<PciConfigAccessStatus> {
        info!("virt pci standard read_cfg, offset {:#x}, size {:#x}", offset, size);
        let dev_guard = dev.read();
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                warn!("virt pci standard read_cfg, id {:#x}", offset);
                Ok(PciConfigAccessStatus::Perform)
            }
            EndpointField::Bar(0) => {
                let slot = 0;
                let bar = dev_guard.get_bararr()[slot];
                if bar.get_size_read() {
                    let value = bar.get_size_with_flag();
                    drop(dev_guard);
                    dev.write().clear_bar_size_read(slot);
                    Ok(PciConfigAccessStatus::Done(value as usize))
                } else {
                    let value = bar.get_virtual_value();
                    Ok(PciConfigAccessStatus::Done(value as usize))
                }
            }
            _ => {
                Ok(PciConfigAccessStatus::Perform)
            }
        }
    }

    fn write_cfg(&self, dev: ArcRwLockVirtualPciConfigSpace, offset: PciConfigAddress, size: usize, value: usize) -> HvResult<PciConfigAccessStatus> {
        info!("virt pci standard write_cfg, offset {:#x}, size {:#x}, value {:#x}", offset, size, value);
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                Ok(PciConfigAccessStatus::Reject)
            }
            EndpointField::Command => {
                let mut dev_guard = dev.write();
                let space = dev_guard.get_space_mut();
                space.set(EndpointField::Command, value as u32);
                Ok(PciConfigAccessStatus::Done(value))
            }
            EndpointField::Bar(0) => {
                let slot = 0;
                let bar_size = dev.get_bararr()[slot].get_size();
                
                if value == 0xFFFF_FFFF {
                    dev.set_bar_size_read(slot);
                } else {
                    let zone = this_zone();
                    let mut guard = zone.write();
                    warn!("virtual pci standard write_cfg, register mmio region {:#x}, size {:#x}", value, bar_size);
                    guard.mmio_region_register(value as usize, bar_size as usize, mmio_vdev_standard_handler, value);
                }

                Ok(PciConfigAccessStatus::Done(value))
            }
            _ => {
                Ok(PciConfigAccessStatus::Reject)
            }
        }
    }

    fn init_config_space(&self) -> PciConfigSpace {
        let mut space = PciConfigSpace::default();
        let default_cspace = DEFAULT_CSPACE_U32;
        let mut offset = 0;
        for &value in &default_cspace {
            space.get_range_mut(offset, 4).copy_from_slice(&value.to_le_bytes());
            offset += 4;
        }
        
        // Example: update vendor ID
        space.set(EndpointField::ID, 0x12345678);
        
        space
    }

    fn init_bar(&self) -> Bar {
        let your_addr = 0x0;
        let mut bar = Bar::default();
        let size = 0x1000;
        bar[0].config_init(PciMemType::Mem32, false, size as u64, your_addr);
        bar
    }
}

/// Static handler instance for standard virtual PCI devices
pub const HANDLER: StandardHandler = StandardHandler;

pub fn mmio_vdev_standard_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    info!("mmio_vdev_standard_handler {:#x}", mmio.address);
    Ok(())
}