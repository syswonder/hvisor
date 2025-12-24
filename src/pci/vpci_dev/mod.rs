use crate::pci::pci_struct::{ConfigValue, ArcRwLockVirtualPciConfigSpace, VirtualPciConfigSpace, Bdf};
use crate::pci::PciConfigAddress;
use crate::error::HvResult;
use crate::pci::pci_access::{Bar, EndpointField};

// Standard virtual device configuration space defaults
pub(crate) const STANDARD_CFG_SIZE: usize = 0x80;
pub(crate) const DEFAULT_CSPACE_U32: [u32; STANDARD_CFG_SIZE / 4] = {
    const STANDARD_VENDOR_ID: u16 = 0x110a;
    const STANDARD_DEVICE_ID: u16 = 0x4106;
    const PCI_STS_CAPS: u16 = 0x10;
    const PCI_DEV_CLASS_OTHER: u8 = 0xff;
    const PCI_CFG_CAPS: usize = 0x34;
    const PCI_CAP_ID_VNDR: u8 = 0x09;
    const PCI_CAP_ID_MSIX: u8 = 0x11;
    const STANDARD_CFG_VNDR_CAP: u8 = 0x40;
    const STANDARD_CFG_VNDR_LEN: u8 = 0x20;
    const STANDARD_CFG_MSIX_CAP: usize = 0x60;
    const STANDARD_MSIX_VECTORS: u16 = 16;
    
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

pub mod standard;

/*
 * PciConfigAccessStatus is used to return the result of the config space access
 * Done(usize): the value is returned in usize
 * Default: use default config space value
 * Reject: the access is rejected
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PciConfigAccessStatus {
    Done(usize),
    Default,
    Reject,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, align(4))]
pub enum VpciDevType {
    #[default]
    Physical=0,
    StandardVdev=1,
    // Add new device types here
}

pub trait VpciDeviceHandler: Sync + Send {
    fn read_cfg(&self, dev: ArcRwLockVirtualPciConfigSpace, offset: PciConfigAddress, size: usize) -> HvResult<PciConfigAccessStatus>;
    fn write_cfg(&self, dev: ArcRwLockVirtualPciConfigSpace, offset: PciConfigAddress, size: usize, value: usize) -> HvResult<PciConfigAccessStatus>;
    fn vdev_init(&self, dev: VirtualPciConfigSpace) -> VirtualPciConfigSpace;
}

/* 
 * Static handler instances for each device type (except Physical).
 * To add a new device type:
 * 1. Add the variant to VpciDevType enum above
 * 2. Add the handler registration here: (&module::HANDLER, VpciDevType::YourType) 
 */
static HANDLERS: &[(&dyn VpciDeviceHandler, VpciDevType)] = &[
    (&standard::HANDLER, VpciDevType::StandardVdev),
];

pub(crate) fn get_handler(dev_type: VpciDevType) -> Option<&'static dyn VpciDeviceHandler> {
    HANDLERS.iter()
        .find(|(_, ty)| *ty == dev_type)
        .map(|(handler, _)| *handler)
}

pub(super) fn vpci_dev_read_cfg(
    dev_type: VpciDevType, 
    node: ArcRwLockVirtualPciConfigSpace, 
    offset: PciConfigAddress, 
    size: usize
) -> HvResult<usize> {
    match dev_type {
        VpciDevType::Physical => {
            warn!("vpci_dev_read_cfg: physical device is not supported");
            Ok(0xFFFF_FFFF)
        }
        _ => {
            if let Some(handler) = get_handler(dev_type) {
                match handler.read_cfg(node.clone(), offset, size) {
                    Ok(status) => {
                        match status {
                            PciConfigAccessStatus::Done(value) => {
                                Ok(value)
                            }
                            PciConfigAccessStatus::Default => {
                                // If this is a standard virtual device, read from DEFAULT_CSPACE_U32
                                if offset < STANDARD_CFG_SIZE as PciConfigAddress {
                                    let u32_offset = (offset as usize) / 4;
                                    let u32_value = DEFAULT_CSPACE_U32[u32_offset];
                                    
                                    // Extract the appropriate bytes based on size and offset within the u32
                                    let byte_offset_in_u32 = (offset as usize) % 4;
                                    let value = match size {
                                        1 => ((u32_value >> (byte_offset_in_u32 * 8)) & 0xFF) as usize,
                                        2 => ((u32_value >> (byte_offset_in_u32 * 8)) & 0xFFFF) as usize,
                                        4 => u32_value as usize,
                                        _ => {
                                            warn!("vpci_dev_read_cfg: invalid size {size}, try read from emu");
                                            let field = EndpointField::from(offset as usize, size);
                                            return node.write().read_emu(field);
                                        }
                                    };
                                    return Ok(value);
                                }
                                // For other device types or out of range, use read_emu
                                let field = EndpointField::from(offset as usize, size);
                                let r = node.write().read_emu(field)?;
                                Ok(r)
                            }
                            PciConfigAccessStatus::Reject => {
                                // warn!("vpci_dev_read_cfg: operation rejected");
                                Ok(0xFFFF_FFFF)
                            }
                        }
                    }
                    Err(e) => {
                        warn!("vpci_dev_read_cfg error: {:?}", e);
                        Ok(0xFFFF_FFFF)
                    }
                }
            } else {
                warn!("vpci_dev_read_cfg: unknown device type");
                Ok(0xFFFF_FFFF)
            }
        }
    }
}

pub(super) fn vpci_dev_write_cfg(
    dev_type: VpciDevType, 
    node: ArcRwLockVirtualPciConfigSpace, 
    offset: PciConfigAddress, 
    size: usize, 
    value: usize
) -> HvResult {
    match dev_type {
        VpciDevType::Physical => {
            warn!("vpci_dev_write_cfg: physical device is not supported");
            Ok(())
        }
        _ => {
            if let Some(handler) = get_handler(dev_type) {
                match handler.write_cfg(node.clone(), offset, size, value) {
                    Ok(status) => {
                        match status {
                            PciConfigAccessStatus::Done(_) => {
                                Ok(())
                            }
                            PciConfigAccessStatus::Default => {
                                warn!("vpci_dev_write_cfg: Default");
                                Ok(())
                            }
                            PciConfigAccessStatus::Reject => {
                                warn!("vpci_dev_write_cfg: operation rejected");
                                Ok(())
                            }
                        }
                    }
                    Err(e) => {
                        warn!("vpci_dev_write_cfg error: {:?}", e);
                        Err(e)
                    }
                }
            } else {
                warn!("vpci_dev_write_cfg: unknown device type");
                Ok(())
            }
        }
    }
}

pub(super) fn virt_dev_init(
    bdf: Bdf,
    base: PciConfigAddress,
    dev_type: VpciDevType,
) -> VirtualPciConfigSpace {
    // Create initial VirtualPciConfigSpace with default values
    let initial_dev = VirtualPciConfigSpace::virt_dev_init_default(
        bdf,
        base,
        dev_type,
        ConfigValue::default(),
        Bar::default()
    );
    
    match dev_type {
        VpciDevType::Physical => {
            // Physical devices use default values
            warn!("virt_dev_init: physical device is not supported");
            initial_dev
        }
        _ => {
            if let Some(handler) = get_handler(dev_type) {
                // Let handler modify and return the device
                handler.vdev_init(initial_dev)
            } else {
                warn!("virt_dev_init: unknown device type");
                initial_dev
            }
        }
    }
}