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

use crate::error::HvResult;
use crate::pci::pci_access::{Bar, EndpointField};
use crate::pci::pci_struct::{
    ArcRwLockVirtualPciConfigSpace, Bdf, CapabilityType, ConfigValue, PciCapabilityRegion,
    VirtualPciConfigSpace,
};
use crate::pci::PciConfigAddress;

use bitvec::array::BitArray;
use bitvec::{order::Lsb0, BitArr};

macro_rules! pci_virt_log {
    ($($arg:tt)*) => {
        // info!($($arg)*);
        // To switch to debug level, change the line above to:
        debug!($($arg)*);
    };
}

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
    const STANDARD_CFG_MSIX_CAP: usize = 0x98;
    const STANDARD_MSIX_VECTORS: u16 = 16;

    let mut arr = [0u32; STANDARD_CFG_SIZE / 4];
    arr[0x00 / 4] = (STANDARD_DEVICE_ID as u32) << 16 | STANDARD_VENDOR_ID as u32;
    arr[0x04 / 4] = (PCI_STS_CAPS as u32) << 16;
    arr[0x08 / 4] = (PCI_DEV_CLASS_OTHER as u32) << 24;
    arr[0x2c / 4] = (STANDARD_DEVICE_ID as u32) << 16 | STANDARD_VENDOR_ID as u32;
    // arr[PCI_CFG_CAPS / 4] = STANDARD_CFG_VNDR_CAP as u32;
    // arr[STANDARD_CFG_VNDR_CAP as usize / 4] = (STANDARD_CFG_VNDR_LEN as u32) << 16
    //     | (STANDARD_CFG_MSIX_CAP as u32) << 8
    //     | PCI_CAP_ID_VNDR as u32;
    // arr[STANDARD_CFG_MSIX_CAP / 4] = (0x00u32) << 8 | PCI_CAP_ID_MSIX as u32;
    // arr[(STANDARD_CFG_MSIX_CAP + 0x4) / 4] = 1;
    // arr[(STANDARD_CFG_MSIX_CAP + 0x8) / 4] = ((0x10 * STANDARD_MSIX_VECTORS) as u32) | 1;
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
    Physical = 0,
    StandardVdev = 1,
    // Add new device types here
}

pub trait VpciDeviceHandler: Sync + Send {
    fn read_cfg(
        &self,
        dev: ArcRwLockVirtualPciConfigSpace,
        offset: PciConfigAddress,
        size: usize,
    ) -> HvResult<PciConfigAccessStatus>;
    fn write_cfg(
        &self,
        dev: ArcRwLockVirtualPciConfigSpace,
        offset: PciConfigAddress,
        size: usize,
        value: usize,
    ) -> HvResult<PciConfigAccessStatus>;
    fn vdev_init(&self, dev: VirtualPciConfigSpace) -> VirtualPciConfigSpace;
}

/*
 * Static handler instances for each device type (except Physical).
 * To add a new device type:
 * 1. Add the variant to VpciDevType enum above
 * 2. Add the handler registration here: (&module::HANDLER, VpciDevType::YourType)
 */
static HANDLERS: &[(&dyn VpciDeviceHandler, VpciDevType)] =
    &[(&standard::HANDLER, VpciDevType::StandardVdev)];

pub(crate) fn get_handler(dev_type: VpciDevType) -> Option<&'static dyn VpciDeviceHandler> {
    HANDLERS
        .iter()
        .find(|(_, ty)| *ty == dev_type)
        .map(|(handler, _)| *handler)
}

pub(super) fn vpci_dev_read_cfg(
    dev_type: VpciDevType,
    node: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
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
                            PciConfigAccessStatus::Done(value) => Ok(value),
                            PciConfigAccessStatus::Default => {
                                // If this is a standard virtual device, read from DEFAULT_CSPACE_U32
                                pci_virt_log!("vpci_dev_read_cfg: default config space read, offset {:#x}, size {:#x}", offset, size);
                                if offset < STANDARD_CFG_SIZE as PciConfigAddress {
                                    let u32_offset = (offset as usize) / 4;
                                    let u32_value = DEFAULT_CSPACE_U32[u32_offset];

                                    // Extract the appropriate bytes based on size and offset within the u32
                                    let byte_offset_in_u32 = (offset as usize) % 4;
                                    let value = match size {
                                        1 => {
                                            ((u32_value >> (byte_offset_in_u32 * 8)) & 0xFF)
                                                as usize
                                        }
                                        2 => {
                                            ((u32_value >> (byte_offset_in_u32 * 8)) & 0xFFFF)
                                                as usize
                                        }
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
    value: usize,
) -> HvResult {
    match dev_type {
        VpciDevType::Physical => {
            warn!("vpci_dev_write_cfg: physical device is not supported");
            Ok(())
        }
        _ => {
            if let Some(handler) = get_handler(dev_type) {
                match handler.write_cfg(node.clone(), offset, size, value) {
                    Ok(status) => match status {
                        PciConfigAccessStatus::Done(_) => Ok(()),
                        PciConfigAccessStatus::Default => {
                            warn!("vpci_dev_write_cfg: Default");
                            Ok(())
                        }
                        PciConfigAccessStatus::Reject => {
                            warn!("vpci_dev_write_cfg: operation rejected");
                            Ok(())
                        }
                    },
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
        Bar::default(),
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

pub struct VirtMsiCap {
    offset: PciConfigAddress,
    next_cap_pointer: u16,
    control_bits: BitArr!(for 16, in u16, Lsb0),
    message_address_lower: u32,
    message_address_upper: u32,
    message_data: u16,
}

impl PciCapabilityRegion for VirtMsiCap {
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<u32> {
        match (offset, size) {
            (0x0, 2) => {
                Ok((CapabilityType::Msi.to_id() as u32) | (self.next_cap_pointer << 8) as u32)
            }
            (0x2, 2) => Ok(self.control_bits.as_raw_slice()[0] as u32),
            (0x4, 4) => Ok(self.message_address_lower),
            (0x8, 4) => Ok(self.message_address_upper),
            (0xC, 2) => Ok(self.message_data as u32),
            _ => {
                warn!(
                    "VirtMsiCap invalid read offset 0x{:x} size {}",
                    offset, size
                );
                Ok(0)
            }
        }
    }
    fn write(&mut self, offset: PciConfigAddress, size: usize, value: u32) -> HvResult {
        match (offset, size) {
            (0x2, 2) => self.control_bits = BitArray::new([value as u16]),
            (0x4, 4) => self.message_address_lower = value,
            (0x8, 4) => self.message_address_upper = value,
            (0xC, 2) => self.message_data = value as u16,
            _ => {
                warn!(
                    "VirtMsiCap invalid write offset 0x{:x} size {}",
                    offset, size
                );
            }
        }
        Ok(())
    }
    fn get_offset(&self) -> PciConfigAddress {
        self.offset
    }
    fn get_size(&self) -> usize {
        128
    }
}

impl VirtMsiCap {
    pub fn new(offset: PciConfigAddress) -> Self {
        let mut bits = BitArray::ZERO;
        bits.set(0, true);
        Self {
            offset,
            next_cap_pointer: 0,
            control_bits: bits,
            message_address_lower: 0,
            message_address_upper: 0,
            message_data: 0,
        }
    }

    pub fn set_next_cap_pointer(&mut self, next_cap_pointer: u16) {
        self.next_cap_pointer = next_cap_pointer;
    }
}

pub struct VirtMsiXCap {
    offset: PciConfigAddress,
    next_cap_pointer: u16,
    control_bits: BitArr!(for 16, in u16, Lsb0),
    table: u32,
    pba: u32,
}

impl PciCapabilityRegion for VirtMsiXCap {
    fn read(&self, offset: PciConfigAddress, size: usize) -> HvResult<u32> {
        match (offset, size) {
            (0x0, 4) => Ok(((CapabilityType::MsiX.to_id() as u32)
                | (self.next_cap_pointer << 8) as u32)
                | (((self.control_bits.as_raw_slice()[0] as u32) << 16) as u32)),
            (0x0, 2) => {
                Ok((CapabilityType::MsiX.to_id() as u32) | (self.next_cap_pointer << 8) as u32)
            }
            (0x2, 2) => Ok(self.control_bits.as_raw_slice()[0] as u32),
            (0x4, 4) => Ok(self.table),
            (0x8, 4) => Ok(self.pba),
            _ => {
                warn!(
                    "VirtMsiXCap invalid read offset 0x{:x} size {}",
                    offset, size
                );
                Ok(0)
            }
        }
    }
    fn write(&mut self, offset: PciConfigAddress, size: usize, value: u32) -> HvResult {
        match (offset, size) {
            (0x2, 2) => self.control_bits = BitArray::new([value as u16]),
            (0x4, 4) => self.table = value,
            (0x8, 4) => self.pba = value,
            _ => {
                warn!(
                    "VirtMsiXCap invalid write offset 0x{:x} size {}",
                    offset, size
                );
            }
        }

        Ok(())
    }

    fn get_offset(&self) -> PciConfigAddress {
        self.offset
    }
    fn get_size(&self) -> usize {
        96
    }
}

impl VirtMsiXCap {
    pub fn new(offset: PciConfigAddress) -> Self {
        let bits = BitArray::ZERO;
        Self {
            offset,
            next_cap_pointer: 0,
            control_bits: bits,
            table: 1,
            pba: ((0x10 * 16) as u32) | 1,
        }
    }

    pub fn set_next_cap_pointer(&mut self, next_cap_pointer: u16) {
        self.next_cap_pointer = next_cap_pointer;
    }
}
