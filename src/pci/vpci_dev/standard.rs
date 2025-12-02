use crate::error::HvResult;
use crate::pci::pci_struct::PciConfigSpace;
use crate::pci::pci_access::EndpointField;
use crate::pci::PciConfigAddress;
use super::{PciConfigAccessStatus, VpciDeviceHandler};

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
    fn read_cfg(&self, _space: &mut PciConfigSpace, offset: PciConfigAddress, size: usize) -> HvResult<PciConfigAccessStatus> {
        info!("virt pci standard read_cfg, offset {:#x}, size {:#x}", offset, size);
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                
                Ok(PciConfigAccessStatus::Done(_space.get(EndpointField::ID) as usize))
            }
            _ => {
                Ok(PciConfigAccessStatus::Perform)
            }
        }
    }

    fn write_cfg(&self, space: &mut PciConfigSpace, offset: PciConfigAddress, size: usize, value: usize) -> HvResult<PciConfigAccessStatus> {
        info!("virt pci standard write_cfg, offset {:#x}, size {:#x}, value {:#x}", offset, size, value);
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                Ok(PciConfigAccessStatus::Reject)
            }
            EndpointField::Command => {
                space.set(EndpointField::Command, value as u32);
                Ok(PciConfigAccessStatus::Done(value))
            }
            _ => {
                Ok(PciConfigAccessStatus::Reject)
            }
        }
    }

    fn init_config_space(&self) -> PciConfigSpace {
        let mut space = PciConfigSpace::new();
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
}

/// Static handler instance for standard virtual PCI devices
pub const HANDLER: StandardHandler = StandardHandler;