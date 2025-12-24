use crate::error::HvResult;
use crate::pci::pci_struct::VirtualPciConfigSpace;
use crate::pci::pci_access::{EndpointField, DeviceId, VendorId, BaseClass, SubClass, Interface, DeviceRevision};
use crate::pci::PciConfigAddress;
use super::{PciConfigAccessStatus, VpciDeviceHandler};
// use crate::memory::frame::Frame;
use crate::pci::pci_struct::ArcRwLockVirtualPciConfigSpace;
use crate::percpu::this_zone;
use crate::memory::MMIOAccess;
use crate::pci::pci_access::PciMemType;


/// Handler for standard virtual PCI devices
pub struct StandardHandler;

impl VpciDeviceHandler for StandardHandler {
    fn read_cfg(&self, dev: ArcRwLockVirtualPciConfigSpace, offset: PciConfigAddress, size: usize) -> HvResult<PciConfigAccessStatus> {
        info!("virt pci standard read_cfg, offset {:#x}, size {:#x}", offset, size);
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                warn!("virt pci standard read_cfg, id {:#x}", offset);
                Ok(PciConfigAccessStatus::Default)
            }
            EndpointField::Bar(0) => {
                let slot = 0;
                let size_read = dev.with_bar_ref(slot, |bar| bar.get_size_read());
                if size_read {
                    let value = dev.with_bar_ref(slot, |bar| bar.get_size_with_flag());
                    dev.with_bar_ref_mut(slot, |bar| bar.clear_size_read());
                    Ok(PciConfigAccessStatus::Done(value as usize))
                } else {
                    let value = dev.with_bar_ref(slot, |bar| bar.get_virtual_value());
                    Ok(PciConfigAccessStatus::Done(value as usize))
                }
            }
            _ => {
                Ok(PciConfigAccessStatus::Default)
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
                // Command field is written directly to backend, no need for space cache
                Ok(PciConfigAccessStatus::Done(value))
            }
            EndpointField::Bar(0) => {
                let slot = 0;
                let bar_size = dev.with_bar_ref(slot, |bar| bar.get_size());
                
                if value == 0xFFFF_FFFF {
                    dev.with_bar_ref_mut(slot, |bar| bar.set_size_read());
                } else {
                    let zone = this_zone();
                    let mut guard = zone.write();
                    info!("virtual pci standard write_cfg, register mmio region {:#x}, size {:#x}", value, bar_size);
                    guard.mmio_region_register(value as usize, bar_size as usize, mmio_vdev_standard_handler, value);
                }

                Ok(PciConfigAccessStatus::Done(value))
            }
            _ => {
                Ok(PciConfigAccessStatus::Reject)
            }
        }
    }

    fn vdev_init(&self, mut dev: VirtualPciConfigSpace) -> VirtualPciConfigSpace {
        // Set config_value
        let id :(DeviceId, VendorId) = (0x110a, 0x4106);
        let revision: DeviceRevision = 0xFFu8;
        let base_class: BaseClass = 0x0;
        let sub_class: SubClass = 0x0;
        let interface: Interface = 0x0;
        dev.with_config_value_mut(|config_value| {
            config_value.set_id(id);
            config_value.set_class_and_revision_id((
                base_class,
                sub_class,
                interface,
                revision,
            ));
        });
        
        // Set bararr
        let your_addr = 0x0;
        let size = 0x1000;
        dev.with_bararr_mut(|bararr| {
            bararr[0].config_init(PciMemType::Mem32, false, size as u64, your_addr);
        });
        
        dev
    }
}

/// Static handler instance for standard virtual PCI devices
pub const HANDLER: StandardHandler = StandardHandler;

pub fn mmio_vdev_standard_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    info!("mmio_vdev_standard_handler {:#x}", mmio.address);
    Ok(())
}