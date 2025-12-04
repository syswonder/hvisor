use crate::pci::pci_struct::{PciConfigSpace, VirtualPciConfigSpace};
use crate::pci::PciConfigAddress;
use crate::error::HvResult;
use crate::pci::pci_access::Bar;

pub mod standard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PciConfigAccessStatus {
    Done(usize),
    Perform,
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
    fn read_cfg(&self, space: &mut PciConfigSpace, offset: PciConfigAddress, size: usize) -> HvResult<PciConfigAccessStatus>;
    fn write_cfg(&self, space: &mut PciConfigSpace, offset: PciConfigAddress, size: usize, value: usize) -> HvResult<PciConfigAccessStatus>;
    fn init_config_space(&self) -> PciConfigSpace;
    fn init_bar(&self) -> Bar;
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
    node: &mut VirtualPciConfigSpace, 
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
                match handler.read_cfg(node.get_space_mut(), offset, size) {
                    Ok(status) => {
                        match status {
                            PciConfigAccessStatus::Done(value) => {
                                Ok(value)
                            }
                            PciConfigAccessStatus::Perform => {
                                warn!("vpci_dev_read_cfg: perform, offset {:#x}, size {:#x}", offset, size);
                                // warn!("vpci_dev_read_cfg: node {:#?}", node.space);
                                let r = node.read_emu(offset, size).unwrap();
                                warn!("vpci_dev_read_cfg: perform result {:#x}", r);
                                Ok(r)
                            }
                            PciConfigAccessStatus::Reject => {
                                warn!("vpci_dev_read_cfg: operation rejected");
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
    node: &mut VirtualPciConfigSpace, 
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
                match handler.write_cfg(node.get_space_mut(), offset, size, value) {
                    Ok(status) => {
                        match status {
                            PciConfigAccessStatus::Done(_) => {
                                Ok(())
                            }
                            PciConfigAccessStatus::Perform => {
                                warn!("vpci_dev_write_cfg: perform");
                                node.write_emu(offset, size, value)
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

pub(super) fn init_config_space_with_type(dev_type: VpciDevType) -> PciConfigSpace {
    match dev_type {
        VpciDevType::Physical => {
            // Physical devices use default (all zeros) space
            PciConfigSpace::new()
        }
        _ => {
            if let Some(handler) = get_handler(dev_type) {
                handler.init_config_space()
            } else {
                warn!("init_config_space_with_type: unknown device type");
                PciConfigSpace::new()
            }
        }
    }
}

pub(super) fn virt_bar_init(dev_type: VpciDevType) -> Bar {
    match dev_type {
        VpciDevType::Physical => {
            // Physical devices use default (all zeros) space
            unreachable!("virt_bar_init: physical device is not supported");
        }
        _ => {
            if let Some(handler) = get_handler(dev_type) {
                handler.init_bar()
            } else {
                warn!("init_config_space_with_type: unknown device type");
                Bar::default()
            }
        }
    }
}