// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//

use super::{PciConfigAccessStatus, VpciDeviceHandler};
use crate::error::HvResult;
use crate::pci::pci_access::{
    BaseClass, DeviceId, DeviceRevision, EndpointField, Interface, SubClass, VendorId,
};
use crate::pci::pci_struct::{CapabilityType, PciCapability, VirtualPciConfigSpace};
use crate::pci::PciConfigAddress;
// use crate::memory::frame::Frame;
use crate::memory::MMIOAccess;
use crate::pci::pci_access::PciMemType;
use crate::pci::pci_struct::ArcRwLockVirtualPciConfigSpace;
use crate::pci::pci_struct::PciCapabilityRegion;
use crate::pci::vpci_dev::VirtMsiXCap;
use crate::percpu::this_zone;
use alloc::sync::Arc;
use spin::RwLock;

/// Handler for standard virtual PCI devices
pub struct StandardHandler;

impl VpciDeviceHandler for StandardHandler {
    fn read_cfg(
        &self,
        dev: ArcRwLockVirtualPciConfigSpace,
        offset: PciConfigAddress,
        size: usize,
    ) -> HvResult<PciConfigAccessStatus> {
        pci_virt_log!(
            "virt pci standard read_cfg, offset {:#x}, size {:#x}",
            offset,
            size
        );
        match EndpointField::from(offset as usize, size) {
            EndpointField::ID => {
                let id = dev.with_config_value(|config_value| config_value.get_id());
                Ok(PciConfigAccessStatus::Done(
                    (((id.0 as u32) << 16) | (id.1 as u32)) as usize,
                ))
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
            EndpointField::CapabilityPointer => Ok(PciConfigAccessStatus::Done(0x98)),
            _ => Ok(PciConfigAccessStatus::Default),
        }
    }

    fn write_cfg(
        &self,
        dev: ArcRwLockVirtualPciConfigSpace,
        offset: PciConfigAddress,
        size: usize,
        value: usize,
    ) -> HvResult<PciConfigAccessStatus> {
        pci_virt_log!(
            "virt pci standard write_cfg, offset {:#x}, size {:#x}, value {:#x}",
            offset,
            size,
            value
        );
        match EndpointField::from(offset as usize, size) {
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
                    pci_virt_log!(
                        "virtual pci standard write_cfg, register mmio region {:#x}, size {:#x}",
                        value,
                        bar_size
                    );
                    guard.mmio_region_register(
                        value as usize,
                        bar_size as usize,
                        mmio_vdev_standard_handler,
                        value,
                    );
                }

                Ok(PciConfigAccessStatus::Done(value))
            }
            EndpointField::Bar(_) => Ok(PciConfigAccessStatus::Done(value)),
            _ => {
                warn!(
                    "virt pci standard write_cfg, invalid offset {:#x}, size {:#x}, value {:#x}",
                    offset, size, value
                );
                Ok(PciConfigAccessStatus::Reject)
            }
        }
    }

    fn vdev_init(&self, mut dev: VirtualPciConfigSpace) -> VirtualPciConfigSpace {
        // Set config_value
        let id: (DeviceId, VendorId) = (0x110a, 0x4106);
        let revision: DeviceRevision = 0xFFu8;
        let base_class: BaseClass = 0x0;
        let sub_class: SubClass = 0x0;
        let interface: Interface = 0x0;
        dev.with_config_value_mut(|config_value| {
            config_value.set_id(id);
            config_value.set_class_and_revision_id((base_class, sub_class, interface, revision));
        });

        // Set bararr
        let your_addr = 0x0;
        let size = 0x1000;
        dev.with_bararr_mut(|bararr| {
            bararr[0].config_init(PciMemType::Mem32, false, size as u64, your_addr);
        });

        // 0x98 is an arbitrary value, used here only for demonstration purposes
        // please don't forget to set next cap pointer if next cap exists
        let msi_cap_offset = 0x98;
        let mut msi_cap = VirtMsiXCap::new(msi_cap_offset);
        msi_cap.set_next_cap_pointer(0x00);
        dev.with_access_mut(|access| {
            access.set_bits(
                (msi_cap_offset as usize)..(msi_cap_offset as usize + msi_cap.get_size()) as usize,
            );
        });

        dev.with_cap_mut(|capabilities| {
            capabilities.insert(
                msi_cap_offset,
                PciCapability::new_virt(CapabilityType::MsiX, Arc::new(RwLock::new(msi_cap))),
            );
        });

        dev.with_access_mut(|access| {
            access.set_bits(0x34..0x38);
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
