use crate::error::HvResult;
use crate::pci::pci_struct::ConfigValue;
use crate::pci::pci_access::{EndpointField, Bar, DeviceId, VendorId, BaseClass, SubClass, Interface, DeviceRevision};
use crate::pci::PciConfigAddress;
use super::{PciConfigAccessStatus, VpciDeviceHandler};
// use crate::memory::frame::Frame;
use crate::pci::pci_struct::ArcRwLockVirtualPciConfigSpace;
use crate::percpu::this_zone;
use crate::memory::MMIOAccess;
use crate::pci::pci_access::PciMemType;

// const STANDARD_VENDOR_ID: u16 = 0x110a;
// const STANDARD_DEVICE_ID: u16 = 0x4106;
// const PCI_STS_CAPS: u16 = 0x10; // bit 4
// const PCI_DEV_CLASS_OTHER: u8 = 0xff;
// const PCI_CFG_CAPS: usize = 0x34;
// const PCI_CAP_ID_VNDR: u8 = 0x09;
// const PCI_CAP_ID_MSIX: u8 = 0x11;
// const STANDARD_CFG_VNDR_CAP: u8 = 0x40;
// const STANDARD_CFG_VNDR_LEN: u8 = 0x20;
// const STANDARD_CFG_MSIX_CAP: usize = 0x60; // VNDR_CAP + VNDR_LEN
// const STANDARD_MSIX_VECTORS: u16 = 16;
// const STANDARD_CFG_SIZE: usize = 0x80;

// pub(crate) const DEFAULT_CSPACE_U32: [u32; STANDARD_CFG_SIZE / 4] = {
//     let mut arr = [0u32; STANDARD_CFG_SIZE / 4];
//     arr[0x00 / 4] = (STANDARD_DEVICE_ID as u32) << 16 | STANDARD_VENDOR_ID as u32;
//     arr[0x04 / 4] = (PCI_STS_CAPS as u32) << 16;
//     arr[0x08 / 4] = (PCI_DEV_CLASS_OTHER as u32) << 24;
//     arr[0x2c / 4] = (STANDARD_DEVICE_ID as u32) << 16 | STANDARD_VENDOR_ID as u32;
//     arr[PCI_CFG_CAPS / 4] = STANDARD_CFG_VNDR_CAP as u32;
//     arr[STANDARD_CFG_VNDR_CAP as usize / 4] = (STANDARD_CFG_VNDR_LEN as u32) << 16
//         | (STANDARD_CFG_MSIX_CAP as u32) << 8
//         | PCI_CAP_ID_VNDR as u32;
//     arr[STANDARD_CFG_MSIX_CAP / 4] = (0x00u32) << 8 | PCI_CAP_ID_MSIX as u32;
//     arr[(STANDARD_CFG_MSIX_CAP + 0x4) / 4] = 1;
//     arr[(STANDARD_CFG_MSIX_CAP + 0x8) / 4] = ((0x10 * STANDARD_MSIX_VECTORS) as u32) | 1;
//     arr
// };

/// Handler for standard virtual PCI devices
pub struct StandardHandler;

impl VpciDeviceHandler for StandardHandler {
    fn read_cfg(&self, dev: ArcRwLockVirtualPciConfigSpace, offset: PciConfigAddress, size: usize) -> HvResult<PciConfigAccessStatus> {
        info!("virt pci standard read_cfg, offset {:#x}, size {:#x}", offset, size);
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                warn!("virt pci standard read_cfg, id {:#x}", offset);
                Ok(PciConfigAccessStatus::Perform)
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

    fn init_config_space(&self) -> ConfigValue {
        let id :(DeviceId, VendorId) = (0x110a, 0x4106);
        let revision: DeviceRevision = 0xFFu8;
        let base_class: BaseClass = 0x0;
        let sub_class: SubClass = 0x0;
        let interface: Interface = 0x0;
        ConfigValue::new(
            id,
            (
                base_class as BaseClass,
                sub_class as SubClass,
                interface as Interface,
                revision as DeviceRevision,
            ),
        )
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