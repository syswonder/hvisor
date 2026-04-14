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

use alloc::string::String;
#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
use spin::Lazy;
#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
use spin::Mutex;

use crate::cpu_data::this_zone;
use crate::error::HvResult;
use crate::memory::{mmio_perform_access, MMIOAccess};
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion};
use crate::pci::pci_struct::CapabilityType;
use crate::zone::is_this_root_zone;

use super::pci_access::{BridgeField, EndpointField, HeaderType, PciField, PciMemType};
use super::pci_config::GLOBAL_PCIE_LIST;
use super::pci_struct::{ArcRwLockVirtualPciConfigSpace, BIT_LENTH};
use super::vpci_dev::VpciDevType;
use super::PciConfigAddress;

#[cfg(target_arch = "x86_64")]
use crate::zone::this_zone_id;

#[cfg(feature = "dwc_pcie")]
use crate::pci::config_accessors::{
    dwc::DwcConfigRegionBackend,
    dwc_atu::{
        AtuType, AtuUnroll, ATU_BASE, ATU_ENABLE_BIT, ATU_REGION_SIZE, PCIE_ATU_UNR_LIMIT,
        PCIE_ATU_UNR_LOWER_BASE, PCIE_ATU_UNR_LOWER_TARGET, PCIE_ATU_UNR_REGION_CTRL1,
        PCIE_ATU_UNR_REGION_CTRL2, PCIE_ATU_UNR_UPPER_BASE, PCIE_ATU_UNR_UPPER_LIMIT,
        PCIE_ATU_UNR_UPPER_TARGET,
    },
    PciRegionMmio,
};

#[cfg(feature = "dwc_msi")]
use super::dwc_msi::{
    PCIE_MSI_ADDR_HI, PCIE_MSI_ADDR_LO, PCIE_MSI_INTR0_ENABLE, PCIE_MSI_INTR0_MASK,
    PCIE_MSI_INTR0_STATUS,
};

#[cfg(not(feature = "dwc_msi"))]
const PCIE_MSI_ADDR_LO: usize = 0x820;
#[cfg(not(feature = "dwc_msi"))]
const PCIE_MSI_INTR0_STATUS: usize = 0x830;

macro_rules! pci_log {
    ($($arg:tt)*) => {
        // info!($($arg)*);
        // To switch to debug level, change the line above to:
        debug!($($arg)*);
    };
}

fn handle_virtio_pci_request(
    dev: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
    value: usize,
    is_write: bool,
) -> HvResult<Option<usize>> {
    // info!(
    //     "offset:0x{:x},size:0x{:x},value:0x{:x},is_write:{}",
    //     offset, size, value, is_write
    // );
    let res = if is_write {
        handle_virtio_pci_write(dev, offset, size, value)
    } else {
        handle_virtio_pci_read(dev, offset, size)
    };
    // info!("result:{:x?}", res);
    res
}

fn handle_virtio_pci_read(
    dev: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
) -> HvResult<Option<usize>> {
    match EndpointField::from(offset as usize, size) {
        EndpointField::ID => dev.with_config_value(|x| {
            let id = x.get_id();
            let res = ((id.0 as usize) << 16) | (id.1 as usize);
            Ok(Some(res))
        }),

        EndpointField::Bar(n) => dev.with_bar_ref(n, |x| Ok(Some(x.read() as usize))),

        EndpointField::Status => {
            // enable capability list
            Ok(Some(0x0010))
        }

        EndpointField::Command => {
            // This is necessary for virtio pci
            Ok(Some(0x0010_0406))
        }

        EndpointField::RevisionIDAndClassCode => Ok(Some(0xff00_0000)),

        EndpointField::CapabilityPointer => {
            dev.with_cap(|x| Ok(Some(x.get_capability_pointer() as usize)))
        }

        _ => {
            dev.with_cap(|x| Ok(x.try_read_cap(offset, size)))
            // Ok(None)
        }
    }
}

fn handle_virtio_pci_write(
    dev: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
    value: usize,
) -> HvResult<Option<usize>> {
    match EndpointField::from(offset as usize, size) {
        EndpointField::Bar(n) => dev.with_bar_ref_mut(n, |x| {
            x.write(value as u32);
            Ok(Some(0))
        }),
        _ => {
            // TODO: Add some warning here in case try write cap failed
            dev.with_cap(|x| Ok(x.try_write_cap(offset, size, value)))
            // Ok(None)
        }
    }
}

// fn handle_virt_pci_request(
//     dev: ArcRwLockVirtualPciConfigSpace,
//     offset: PciConfigAddress,
//     size: usize,
//     value: usize,
//     is_write: bool,
//     dev_type: VpciDevType,
// ) -> HvResult<Option<usize>> {
//     /*
//      * The capability is located in the upper part of the configuration space,
//      * and there is no other message. So the max cap_offset which is less than
//      * offset is the correct cap we need.
//      */
//     let result = dev.with_cap(|capabilities| {
//         if let Some((cap_offset, cap)) = capabilities
//             .cap_in_config_ref()
//             .range(..=offset)
//             .next_back()
//         {
//             let end = *cap_offset + cap.get_size() as u64;
//             if offset >= end {
//                 return hv_result_err!(
//                     ERANGE,
//                     format!(
//                         "virt pci cap rw offset {:#x} out of range [{:#x}..{:#x})",
//                         offset, *cap_offset, end
//                     )
//                 );
//             }
//             let relative_offset = offset - *cap_offset;

//             if is_write {
//                 cap.with_region_mut(|region| {
//                     match region.write(relative_offset, size, value as u32) {
//                         Ok(()) => Ok(0),
//                         Err(e) => {
//                             warn!(
//                                 "Failed to write capability at offset 0x{:x}: {:?}",
//                                 offset, e
//                             );
//                             Err(e)
//                         }
//                     }
//                 })
//             } else {
//                 cap.with_region(|region| match region.read(relative_offset, size) {
//                     Ok(val) => Ok(val),
//                     Err(e) => {
//                         warn!(
//                             "Failed to read capability at offset 0x{:x}: {:?}",
//                             offset, e
//                         );
//                         Err(e)
//                     }
//                 })
//             }
//         } else {
//             hv_result_err!(ENOENT)
//         }
//     });

//     match result {
//         Ok(val) => {
//             if !is_write {
//                 Ok(Some(val as usize))
//             } else {
//                 Ok(None)
//             }
//         }
//         Err(_) => {
//             if is_write {
//                 super::vpci_dev::vpci_dev_write_cfg(dev_type, dev.clone(), offset, size, value)?;
//                 Ok(None)
//             } else {
//                 Ok(Some(super::vpci_dev::vpci_dev_read_cfg(
//                     dev_type,
//                     dev.clone(),
//                     offset,
//                     size,
//                 )?))
//             }
//         }
//     }
// }

fn handle_cap_access(
    dev: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
    value: usize,
    is_write: bool,
    is_dev_belong_to_zone: bool,
) -> HvResult<Option<usize>> {
    // Handle capability region access (offset >= 0x34)
    if offset == 0x34 {
        // Cap Pointer register (may be accessed as different sizes)
        if is_dev_belong_to_zone {
            // Direct pass through to hardware
            if is_write {
                dev.write_hw(offset, size, value)?;
                Ok(None)
            } else {
                Ok(Some(dev.read_hw(offset, size)?))
            }
        } else {
            // Device not belong to zone, return 0 (no capability)
            if is_write {
                Ok(None)
            } else {
                Ok(Some(0))
            }
        }
    } else {
        // Other capability region offsets
        // Try to find the capability that contains this offset
        let cap_info = dev.with_cap(|capabilities| {
            capabilities
                .range(..=offset as u64)
                .next_back()
                .map(|(cap_offset, cap)| (*cap_offset, cap.get_type()))
        });

        if let Some((cap_offset, cap_type)) = cap_info {
            let cap_offset = cap_offset as usize;
            let relative_offset = offset as usize - cap_offset;

            if cap_type == CapabilityType::Msi {
                let vbdf = dev.get_vbdf();
                let domain_id = vbdf.domain();

                let is_msi_64 = dev.with_cap(|capabilities| {
                    capabilities
                        .get(&(cap_offset as u64))
                        .and_then(|cap| cap.with_region(|region| region.read(0x02, 2).ok()))
                        .map(|ctrl| (ctrl & (1 << 7)) != 0)
                        .unwrap_or(false)
                });

                let is_addr_low = matches!(relative_offset, 4 | 5 | 6 | 7);
                let is_addr_high = is_msi_64 && matches!(relative_offset, 8 | 9 | 10 | 11);
                let is_msg_data = if is_msi_64 {
                    matches!(relative_offset, 12 | 13)
                } else {
                    matches!(relative_offset, 8 | 9)
                };

                #[cfg(feature = "dwc_msi")]
                {
                    if is_write {
                        if is_addr_low {
                            dev.with_msi_info_mut(|msi_info| {
                                let current = msi_info.msi_doorbell & 0xffffffff00000000;
                                msi_info.set_doorbell(current | (value as u64));
                            });
                            let hw_paddr =
                                crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);
                            dev.write_hw(offset, size, (hw_paddr & 0xffffffff) as usize)?;
                            return Ok(None);
                        }
                        if is_addr_high {
                            dev.with_msi_info_mut(|msi_info| {
                                let current = msi_info.msi_doorbell & 0xffffffff;
                                msi_info.set_doorbell(current | ((value as u64) << 32));
                            });
                            let hw_paddr =
                                crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);
                            dev.write_hw(offset, size, ((hw_paddr >> 32) & 0xffffffff) as usize)?;
                            return Ok(None);
                        }
                        if is_msg_data {
                            let zone = this_zone();
                            let guard = zone.read();
                            let vbus = guard.vpci_bus();
                            if let Some(domain_msi_info) = vbus.domain_msi_info().get(&domain_id) {
                                let hw_value =
                                    (value as u32).wrapping_add(domain_msi_info.hwirq_bit);
                                dev.write_hw(offset, size, hw_value as usize)?;
                            } else {
                                dev.write_hw(offset, size, value)?;
                            }
                            return Ok(None);
                        }
                    } else {
                        if is_addr_low {
                            let vm_doorbell = dev
                                .read()
                                .get_msi_info()
                                .map(|msi_info| msi_info.msi_doorbell)
                                .unwrap_or(0);
                            return Ok(Some((vm_doorbell & 0xffffffff) as usize));
                        }
                        if is_addr_high {
                            let vm_doorbell = dev
                                .read()
                                .get_msi_info()
                                .map(|msi_info| msi_info.msi_doorbell)
                                .unwrap_or(0);
                            return Ok(Some(((vm_doorbell >> 32) & 0xffffffff) as usize));
                        }
                        if is_msg_data {
                            let hw_value = dev.read_hw(offset, size)?;
                            let zone = this_zone();
                            let guard = zone.read();
                            let vbus = guard.vpci_bus();
                            if let Some(domain_msi_info) = vbus.domain_msi_info().get(&domain_id) {
                                let hwirq_bit = domain_msi_info.hwirq_bit;
                                let hw_vec = hw_value as u32;
                                let virq_bit = if hw_vec >= hwirq_bit {
                                    hw_vec - hwirq_bit
                                } else {
                                    hw_vec
                                };
                                return Ok(Some(virq_bit as usize));
                            }
                            return Ok(Some(hw_value));
                        }
                    }
                }
            }

            // Direct pass through to hardware for all cap access
            if is_write {
                dev.write_hw(offset, size, value)?;
                Ok(None)
            } else {
                Ok(Some(dev.read_hw(offset, size)?))
            }
        } else {
            // No capability found at this offset
            Ok(None)
        }
    }
}

fn handle_endpoint_access(
    dev: ArcRwLockVirtualPciConfigSpace,
    field: EndpointField,
    value: usize,
    is_write: bool,
    is_direct: bool,
    is_root: bool,
    is_dev_belong_to_zone: bool,
) -> HvResult<Option<usize>> {
    match field {
        EndpointField::ID => {
            if !is_write && is_dev_belong_to_zone {
                Ok(Some(dev.read_emu(EndpointField::ID)?))
            } else if !is_write && is_direct && is_root {
                /* just an id no one used now
                 * here let root allocate resources but not drive the device
                 */
                const ROOT_UNUSED_DEVICE_ID: usize = 0xFFFD_4106;
                Ok(Some(ROOT_UNUSED_DEVICE_ID))
            } else {
                // id is readonly (when is_write is true)
                // warn!(
                //     "vbdf {:#?}: unhandled {:#?} {}",
                //     dev.get_vbdf(),
                //     field,
                //     if is_write { "write" } else { "read" }
                // );
                Ok(None)
            }
        }
        EndpointField::RevisionIDAndClassCode => {
            if !is_write && is_dev_belong_to_zone {
                Ok(Some(dev.read_emu(EndpointField::RevisionIDAndClassCode)?))
            } else if !is_write && is_direct && is_root {
                const ROOT_DEFAULT_CLASS_AND_REVISION: usize = 0xff00_0000;
                Ok(Some(ROOT_DEFAULT_CLASS_AND_REVISION))
            } else {
                warn!(
                    "vbdf {:#?}: unhandled {:#?} {}",
                    dev.get_vbdf(),
                    field,
                    if is_write { "write" } else { "read" }
                );
                Ok(None)
            }
        }
        EndpointField::Bar(slot) => {
            /*
             * hw: the physical reg
             * value: same with physical reg, the paddr for pt
             * virt_value: the vaddr for pt
             * config_value: the virtual reg for zone, directly rw
             *
             * The virt_value cache of vaddr is required because mem64 bar updates are
             * split between mem64high and mem64low registers. The Hvisor must wait
             * for both updates to complete before using old_vaddr for page table maintenance
             *
             * In typical operation, tmp_value maintains parity with virt_value; the sole exception occurs
             * when exclusively updating mem64low while leaving mem64high unmodified,
             * as previously described
             */
            let bar_type = dev.with_bar_ref(slot, |bar| bar.get_type());

            // Check if this BAR contains MSIX table (only when dwc_msi feature is enabled)
            #[cfg(feature = "dwc_msi")]
            let is_msix_bar = {
                let msix_check_slot = if bar_type == PciMemType::Mem64High && slot > 0 {
                    slot - 1
                } else {
                    slot
                };

                dev.read()
                    .get_msi_info()
                    .and_then(|msi_info| {
                        msi_info
                            .msix_info
                            .as_ref()
                            .map(|msix| msix.bar_id == msix_check_slot as u8)
                    })
                    .unwrap_or(false)
            };

            #[cfg(not(feature = "dwc_msi"))]
            let is_msix_bar = false;

            if bar_type != PciMemType::default() {
                if is_write {
                    if is_direct && is_root {
                        // direct mode and root zone, update resources directly
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_bar_value(slot, value as u32);
                        });
                        if (value & 0xfffffff0) != 0xfffffff0 {
                            dev.write_hw(
                                field.to_offset() as PciConfigAddress,
                                field.size(),
                                value,
                            )?;
                            if (bar_type == PciMemType::Mem32)
                                | (bar_type == PciMemType::Mem64High)
                                | (bar_type == PciMemType::Io)
                            {
                                let old_vaddr =
                                    dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                let new_vaddr = {
                                    if bar_type == PciMemType::Mem64High {
                                        /* last 4bit is flag, not address and need ignore
                                         * flag will auto add when set_value and set_virtual_value
                                         * Read from config_value.bar_value cache instead of space
                                         */
                                        let low_value = dev
                                            .with_config_value(|cv| cv.get_bar_value(slot - 1))
                                            as u64;
                                        let high_value = (value as u32 as u64) << 32;
                                        (low_value | high_value) & !0xf
                                    } else {
                                        (value as u64) & !0xf
                                    }
                                };

                                // set virt_value
                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                }

                                // set value
                                dev.with_bar_ref_mut(slot, |bar| bar.set_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| bar.set_value(new_vaddr));
                                }

                                let paddr = {
                                    let raw = dev.with_bar_ref(slot, |bar| bar.get_value64())
                                        as HostPhysAddr;
                                    if bar_type == PciMemType::Io {
                                        raw & !0x3
                                    } else {
                                        raw & !0xf
                                    }
                                };

                                if is_msix_bar {
                                    let msix_slot = if bar_type == PciMemType::Mem64High {
                                        slot - 1
                                    } else {
                                        slot
                                    };
                                    dev.with_msi_info_mut(|msi_info| {
                                        if let Some(msix) = msi_info.msix_info.as_mut() {
                                            if msix.bar_id as usize == msix_slot {
                                                msix.bar_paddr = paddr as u64;
                                            }
                                        }
                                    });
                                }

                                let bar_size = {
                                    let size = dev.with_bar_ref(slot, |bar| bar.get_size());
                                    if crate::memory::addr::is_aligned(size as usize) {
                                        size
                                    } else {
                                        crate::memory::PAGE_SIZE as u64
                                    }
                                };
                                let new_vaddr_aligned =
                                    if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                    } else {
                                        new_vaddr as u64
                                    };

                                let zone = this_zone();
                                let mut guard = zone.write();

                                if is_msix_bar {
                                    let msix_bar_id = if bar_type == PciMemType::Mem64High {
                                        slot - 1
                                    } else {
                                        slot
                                    };
                                    warn!(
                                        "Register MSI-X BAR handler: vbdf {:#?}, bar_id {}, gpa {:#x}, size {:#x}, paddr {:#x}",
                                        dev.get_vbdf(),
                                        msix_bar_id,
                                        new_vaddr_aligned,
                                        bar_size,
                                        paddr
                                    );
                                    guard.mmio_region_remove(old_vaddr as GuestPhysAddr);
                                    guard.mmio_region_register(
                                        new_vaddr_aligned as GuestPhysAddr,
                                        bar_size as usize,
                                        mmio_msix_table_handler,
                                        paddr as usize,
                                    );
                                } else {
                                    let gpm = guard.gpm_mut();
                                    if !gpm
                                        .try_delete(
                                            old_vaddr.try_into().unwrap(),
                                            bar_size as usize,
                                        )
                                        .is_ok()
                                    {}
                                    gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                        new_vaddr_aligned as GuestPhysAddr,
                                        paddr as HostPhysAddr,
                                        bar_size as _,
                                        MemFlags::READ | MemFlags::WRITE,
                                    ))?;
                                }
                                drop(guard);
                                #[cfg(target_arch = "aarch64")]
                                unsafe {
                                    core::arch::asm!("isb");
                                    core::arch::asm!("tlbi vmalls12e1is");
                                    core::arch::asm!("dsb nsh");
                                }
                                #[cfg(target_arch = "x86_64")]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::arch::iommu::flush(
                                        this_zone_id(),
                                        vbdf.bus,
                                        (vbdf.device << 3) + vbdf.function,
                                    );
                                }
                            }
                        }
                    } else if is_dev_belong_to_zone {
                        // normal mod, update virt resources
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_bar_value(slot, value as u32);
                        });
                        if (value & 0xfffffff0) != 0xfffffff0 {
                            if (bar_type == PciMemType::Mem32)
                                | (bar_type == PciMemType::Mem64High)
                                | (bar_type == PciMemType::Io)
                            {
                                let old_vaddr =
                                    dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                let new_vaddr = {
                                    if bar_type == PciMemType::Mem64High {
                                        /* last 4bit is flag, not address and need ignore
                                         * flag will auto add when set_value and set_virtual_value
                                         * Read from config_value.bar_value cache instead of space
                                         */
                                        let low_value = dev
                                            .with_config_value(|cv| cv.get_bar_value(slot - 1))
                                            as u64;
                                        let high_value = (value as u32 as u64) << 32;
                                        (low_value | high_value) & !0xf
                                    } else {
                                        (value as u64) & !0xf
                                    }
                                };

                                // info!("new_vaddr: {:#x}", new_vaddr);
                                // info!("old_vaddr: {:#x}", old_vaddr);
                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                }

                                let paddr = {
                                    let raw = dev.with_bar_ref(slot, |bar| bar.get_value64())
                                        as HostPhysAddr;
                                    if bar_type == PciMemType::Io {
                                        raw & !0x3
                                    } else {
                                        raw & !0xf
                                    }
                                };

                                if is_msix_bar {
                                    dev.with_msi_info_mut(|msi_info| {
                                        if let Some(msix) = msi_info.msix_info.as_mut() {
                                            let msix_slot = if bar_type == PciMemType::Mem64High {
                                                slot - 1
                                            } else {
                                                slot
                                            };
                                            if msix.bar_id as usize == msix_slot {
                                                msix.bar_paddr = paddr as u64;
                                            }
                                        }
                                    });
                                }
                                let bar_size = {
                                    let size = dev.with_bar_ref(slot, |bar| bar.get_size());
                                    if crate::memory::addr::is_aligned(size as usize) {
                                        size
                                    } else {
                                        crate::memory::PAGE_SIZE as u64
                                    }
                                };
                                let new_vaddr =
                                    if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                    } else {
                                        new_vaddr as u64
                                    };

                                let zone = this_zone();
                                let mut guard = zone.write();

                                if is_msix_bar {
                                    let msix_bar_id = if bar_type == PciMemType::Mem64High {
                                        slot - 1
                                    } else {
                                        slot
                                    };
                                    warn!(
                                        "Register MSI-X BAR handler: vbdf {:#?}, bar_id {}, gpa {:#x}, size {:#x}, paddr {:#x}",
                                        dev.get_vbdf(),
                                        msix_bar_id,
                                        new_vaddr,
                                        bar_size,
                                        paddr
                                    );
                                    // Remove old MSIX handler if it exists
                                    guard.mmio_region_remove(old_vaddr as GuestPhysAddr);
                                    // Register new MSIX handler at new address
                                    guard.mmio_region_register(
                                        new_vaddr as GuestPhysAddr,
                                        bar_size as usize,
                                        mmio_msix_table_handler,
                                        paddr as usize,
                                    );
                                } else {
                                    // Delete old gpm mapping if it exists
                                    let gpm = guard.gpm_mut();
                                    if !gpm
                                        .try_delete(
                                            old_vaddr.try_into().unwrap(),
                                            bar_size as usize,
                                        )
                                        .is_ok()
                                    {
                                        // warn!("delete bar {}: can not found 0x{:x}", slot, old_vaddr);
                                    }
                                    // Insert new gpm mapping at new address
                                    gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                        new_vaddr as GuestPhysAddr,
                                        paddr as HostPhysAddr,
                                        bar_size as _,
                                        MemFlags::READ | MemFlags::WRITE,
                                    ))?;
                                }
                                drop(guard);
                                /* after update gpm, mem barrier is needed
                                 */
                                #[cfg(target_arch = "aarch64")]
                                unsafe {
                                    core::arch::asm!("isb");
                                    core::arch::asm!("tlbi vmalls12e1is");
                                    core::arch::asm!("dsb nsh");
                                }
                                /* after update gpm, need to flush iommu table
                                 * in x86_64
                                 */
                                #[cfg(all(target_arch = "x86_64", feature = "intel_vtd"))]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::device::iommu::flush(
                                        this_zone_id(),
                                        vbdf.bus,
                                        (vbdf.device << 3) + vbdf.function,
                                    );
                                }
                                #[cfg(target_arch = "riscv64")]
                                unsafe {
                                    // TOOD: add remote fence support (using sbi rfence spec?)
                                    core::arch::asm!("hfence.gvma");
                                }
                            }
                        }
                    }
                    Ok(None)
                } else {
                    // read bar
                    if (dev.with_config_value(|configvalue| configvalue.get_bar_value(slot))
                        & 0xfffffff0)
                        == 0xfffffff0
                    {
                        /*
                         * tmp_value being 0xFFFF_FFFF means that Linux is attempting to determine the BAR size.
                         * The value of tmp_value is used directly here because Linux will rewrite this register later,
                         * so the Hvisor does not need to preserve any additional state.
                         */
                        Ok(Some(
                            dev.with_bar_ref(slot, |bar| bar.get_size_with_flag()) as usize
                        ))
                    } else {
                        Ok(Some(
                            dev.with_config_value(|configvalue| configvalue.get_bar_value(slot))
                                as usize,
                        ))
                    }
                }
            } else {
                Ok(None)
            }
        }
        EndpointField::ExpansionRomBar => {
            // rom is same with bar
            let rom_type = dev.with_rom_ref(|rom| rom.get_type());
            if rom_type == PciMemType::Rom {
                if is_write {
                    if is_direct && is_root {
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_rom_value(value as u32);
                        });
                        if value & 0xfffff800 != 0xfffff800 {
                            dev.write_hw(
                                field.to_offset() as PciConfigAddress,
                                field.size(),
                                value,
                            )?;

                            let new_vaddr = (value as u64) & !0xf;

                            // set virt_value
                            dev.with_rom_ref_mut(|rom| rom.set_virtual_value(new_vaddr));

                            // set value
                            dev.with_rom_ref_mut(|rom| rom.set_value(new_vaddr));
                        }
                    } else if is_dev_belong_to_zone {
                        // normal mode, update virt resources
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_rom_value(value as u32);
                        });

                        // Check if this is size probe (all 1s in BA field, bits 31-11)
                        let is_size_probe = (value & 0xfffff800) == 0xfffff800;
                        // Check if ROM enable bit (bit 0) is set
                        let rom_enabled = (value & 0x1) != 0;

                        if !is_size_probe {
                            let old_vaddr =
                                dev.with_rom_ref(|rom| rom.get_virtual_value64()) & !0xf;
                            let new_vaddr = (value as u64) & !0xf;

                            // Only perform mapping operations if ROM enable bit is set
                            if rom_enabled {
                                // set new_value not new_vaddr, because `set_virtual_value` will not add enable flag automatically
                                dev.with_rom_ref_mut(|rom| rom.set_virtual_value(value as _));

                                // Write to hardware with enable bit set
                                // Get the current ROM value from hardware and set bit 0
                                // And not to use rom.set_value()
                                let hw_value = dev.with_rom_ref(|rom| rom.get_value64());
                                let hw_value_enabled = hw_value | 0x1; // Set enable bit
                                dev.write_hw(
                                    field.to_offset() as PciConfigAddress,
                                    field.size(),
                                    hw_value_enabled as usize,
                                )?;
                                dev.with_rom_ref_mut(|rom| rom.set_value(hw_value_enabled));

                                let paddr =
                                    dev.with_rom_ref(|rom| rom.get_value64()) as HostPhysAddr;

                                let rom_size = {
                                    let size = dev.with_rom_ref(|rom| rom.get_size());
                                    if crate::memory::addr::is_aligned(size as usize) {
                                        size
                                    } else {
                                        crate::memory::PAGE_SIZE as u64
                                    }
                                };
                                let new_vaddr_aligned =
                                    if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                    } else {
                                        new_vaddr as u64
                                    };

                                let zone = this_zone();
                                let mut guard = zone.write();
                                let gpm = guard.gpm_mut();

                                if !gpm
                                    .try_delete(old_vaddr.try_into().unwrap(), rom_size as usize)
                                    .is_ok()
                                {
                                    // warn!("delete rom bar: can not found 0x{:x}", old_vaddr);
                                }
                                gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                    new_vaddr_aligned as GuestPhysAddr,
                                    paddr as HostPhysAddr,
                                    rom_size as _,
                                    MemFlags::READ | MemFlags::WRITE,
                                ))?;
                                drop(guard);
                                /* after update gpm, mem barrier is needed
                                 */
                                #[cfg(target_arch = "aarch64")]
                                unsafe {
                                    core::arch::asm!("isb");
                                    core::arch::asm!("tlbi vmalls12e1is");
                                    core::arch::asm!("dsb nsh");
                                }
                                /* after update gpm, need to flush iommu table
                                 * in x86_64
                                 */
                                #[cfg(all(target_arch = "x86_64", feature = "intel_vtd"))]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::device::iommu::flush(
                                        this_zone_id(),
                                        vbdf.bus,
                                        (vbdf.device << 3) + vbdf.function,
                                    );
                                }
                                #[cfg(target_arch = "riscv64")]
                                unsafe {
                                    // TOOD: add remote fence support (using sbi rfence spec?)
                                    core::arch::asm!("hfence.gvma");
                                }
                            } else {
                                // ROM disabled
                            }
                        }
                    }
                    Ok(None)
                } else {
                    // read rom bar
                    if (dev.with_config_value(|configvalue| configvalue.get_rom_value()))
                        & 0xfffff800
                        == 0xfffff800
                    {
                        /*
                         * config_value being 0xFFFF_FFFF means that Linux is attempting to determine the ROM size.
                         * The value is used directly here because Linux will rewrite this register later,
                         * so the Hvisor does not need to preserve any additional state.
                         */
                        Ok(Some(
                            dev.with_rom_ref(|rom| rom.get_size_with_flag()) as usize
                        ))
                    } else {
                        Ok(Some(
                            dev.with_config_value(|configvalue| configvalue.get_rom_value())
                                as usize,
                        ))
                    }
                }
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

fn handle_pci_bridge_access(
    dev: ArcRwLockVirtualPciConfigSpace,
    field: BridgeField,
    value: usize,
    is_write: bool,
    is_direct: bool,
    is_root: bool,
    is_dev_belong_to_zone: bool,
) -> HvResult<Option<usize>> {
    match field {
        BridgeField::Bar(slot) => {
            let bar_type = dev.with_bar_ref(slot, |bar| bar.get_type());

            // Check if this BAR contains MSIX table (only when dwc_msi feature is enabled)
            #[cfg(feature = "dwc_msi")]
            let is_msix_bar = {
                let msix_check_slot = if bar_type == PciMemType::Mem64High && slot > 0 {
                    slot - 1
                } else {
                    slot
                };

                dev.read()
                    .get_msi_info()
                    .and_then(|msi_info| {
                        msi_info
                            .msix_info
                            .as_ref()
                            .map(|msix| msix.bar_id == msix_check_slot as u8)
                    })
                    .unwrap_or(false)
            };

            #[cfg(not(feature = "dwc_msi"))]
            let is_msix_bar = false;

            if bar_type != PciMemType::default() {
                if is_write {
                    if is_direct && is_root {
                        // direct mode and root zone, update resources directly
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_bar_value(slot, value as u32);
                        });
                        if (value & 0xfffffff0) != 0xfffffff0 {
                            dev.write_hw(
                                field.to_offset() as PciConfigAddress,
                                field.size(),
                                value,
                            )?;
                            if (bar_type == PciMemType::Mem32)
                                | (bar_type == PciMemType::Mem64High)
                                | (bar_type == PciMemType::Io)
                            {
                                let old_vaddr =
                                    dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                let new_vaddr = {
                                    if bar_type == PciMemType::Mem64High {
                                        let low_value = dev
                                            .with_config_value(|cv| cv.get_bar_value(slot - 1))
                                            as u64;
                                        let high_value = (value as u32 as u64) << 32;
                                        (low_value | high_value) & !0xf
                                    } else {
                                        (value as u64) & !0xf
                                    }
                                };

                                // set virt_value
                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                }

                                // set value
                                dev.with_bar_ref_mut(slot, |bar| bar.set_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| bar.set_value(new_vaddr));
                                }

                                let paddr = {
                                    let raw = dev.with_bar_ref(slot, |bar| bar.get_value64())
                                        as HostPhysAddr;
                                    if bar_type == PciMemType::Io {
                                        raw & !0x3
                                    } else {
                                        raw & !0xf
                                    }
                                };

                                if is_msix_bar {
                                    let msix_slot = if bar_type == PciMemType::Mem64High {
                                        slot - 1
                                    } else {
                                        slot
                                    };
                                    dev.with_msi_info_mut(|msi_info| {
                                        if let Some(msix) = msi_info.msix_info.as_mut() {
                                            if msix.bar_id as usize == msix_slot {
                                                msix.bar_paddr = paddr as u64;
                                            }
                                        }
                                    });
                                }

                                let bar_size = {
                                    let size = dev.with_bar_ref(slot, |bar| bar.get_size());
                                    if crate::memory::addr::is_aligned(size as usize) {
                                        size
                                    } else {
                                        crate::memory::PAGE_SIZE as u64
                                    }
                                };
                                let new_vaddr_aligned =
                                    if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                    } else {
                                        new_vaddr as u64
                                    };

                                let zone = this_zone();
                                let mut guard = zone.write();

                                if is_msix_bar {
                                    let msix_bar_id = if bar_type == PciMemType::Mem64High {
                                        slot - 1
                                    } else {
                                        slot
                                    };
                                    warn!(
                                        "Register MSI-X BAR handler: vbdf {:#?}, bar_id {}, gpa {:#x}, size {:#x}, paddr {:#x}",
                                        dev.get_vbdf(),
                                        msix_bar_id,
                                        new_vaddr_aligned,
                                        bar_size,
                                        paddr
                                    );
                                    guard.mmio_region_remove(old_vaddr as GuestPhysAddr);
                                    guard.mmio_region_register(
                                        new_vaddr_aligned as GuestPhysAddr,
                                        bar_size as usize,
                                        mmio_msix_table_handler,
                                        paddr as usize,
                                    );
                                } else {
                                    let gpm = guard.gpm_mut();
                                    if !gpm
                                        .try_delete(
                                            old_vaddr.try_into().unwrap(),
                                            bar_size as usize,
                                        )
                                        .is_ok()
                                    {}
                                    gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                        new_vaddr_aligned as GuestPhysAddr,
                                        paddr as HostPhysAddr,
                                        bar_size as _,
                                        MemFlags::READ | MemFlags::WRITE,
                                    ))?;
                                }
                                drop(guard);
                                #[cfg(target_arch = "aarch64")]
                                unsafe {
                                    core::arch::asm!("isb");
                                    core::arch::asm!("tlbi vmalls12e1is");
                                    core::arch::asm!("dsb nsh");
                                }
                                #[cfg(target_arch = "x86_64")]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::arch::iommu::flush(
                                        this_zone_id(),
                                        vbdf.bus,
                                        (vbdf.device << 3) + vbdf.function,
                                    );
                                }
                            }
                        }
                    } else if is_dev_belong_to_zone {
                        // normal mode, update virt resources
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_bar_value(slot, value as u32);
                        });
                        if (value & 0xfffffff0) != 0xfffffff0 {
                            if (bar_type == PciMemType::Mem32)
                                | (bar_type == PciMemType::Mem64High)
                                | (bar_type == PciMemType::Io)
                            {
                                let old_vaddr =
                                    dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                let new_vaddr = {
                                    if bar_type == PciMemType::Mem64High {
                                        let low_value = dev
                                            .with_config_value(|cv| cv.get_bar_value(slot - 1))
                                            as u64;
                                        let high_value = (value as u32 as u64) << 32;
                                        (low_value | high_value) & !0xf
                                    } else {
                                        (value as u64) & !0xf
                                    }
                                };

                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                }

                                let paddr = {
                                    let raw = dev.with_bar_ref(slot, |bar| bar.get_value64())
                                        as HostPhysAddr;
                                    if bar_type == PciMemType::Io {
                                        raw & !0x3
                                    } else {
                                        raw & !0xf
                                    }
                                };

                                if is_msix_bar {
                                    dev.with_msi_info_mut(|msi_info| {
                                        if let Some(msix) = msi_info.msix_info.as_mut() {
                                            let msix_slot = if bar_type == PciMemType::Mem64High {
                                                slot - 1
                                            } else {
                                                slot
                                            };
                                            if msix.bar_id as usize == msix_slot {
                                                msix.bar_paddr = paddr as u64;
                                            }
                                        }
                                    });
                                }
                                let bar_size = {
                                    let size = dev.with_bar_ref(slot, |bar| bar.get_size());
                                    if crate::memory::addr::is_aligned(size as usize) {
                                        size
                                    } else {
                                        crate::memory::PAGE_SIZE as u64
                                    }
                                };
                                let new_vaddr_aligned =
                                    if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                    } else {
                                        new_vaddr as u64
                                    };

                                let zone = this_zone();
                                let mut guard = zone.write();

                                if is_msix_bar {
                                    let msix_bar_id = if bar_type == PciMemType::Mem64High {
                                        slot - 1
                                    } else {
                                        slot
                                    };
                                    warn!(
                                        "Register MSI-X BAR handler: vbdf {:#?}, bar_id {}, gpa {:#x}, size {:#x}, paddr {:#x}",
                                        dev.get_vbdf(),
                                        msix_bar_id,
                                        new_vaddr_aligned,
                                        bar_size,
                                        paddr
                                    );
                                    // Remove old MSIX handler if it exists
                                    guard.mmio_region_remove(old_vaddr as GuestPhysAddr);
                                    // Register new MSIX handler at new address
                                    guard.mmio_region_register(
                                        new_vaddr_aligned as GuestPhysAddr,
                                        bar_size as usize,
                                        mmio_msix_table_handler,
                                        paddr as usize,
                                    );
                                } else {
                                    // Delete old gpm mapping if it exists
                                    let gpm = guard.gpm_mut();
                                    if !gpm
                                        .try_delete(
                                            old_vaddr.try_into().unwrap(),
                                            bar_size as usize,
                                        )
                                        .is_ok()
                                    {
                                        // warn!("delete bar {}: can not found 0x{:x}", slot, old_vaddr);
                                    }
                                    // Insert new gpm mapping at new address
                                    gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                        new_vaddr_aligned as GuestPhysAddr,
                                        paddr as HostPhysAddr,
                                        bar_size as _,
                                        MemFlags::READ | MemFlags::WRITE,
                                    ))?;
                                }
                                drop(guard);
                                /* after update gpm, mem barrier is needed
                                 */
                                #[cfg(target_arch = "aarch64")]
                                unsafe {
                                    core::arch::asm!("isb");
                                    core::arch::asm!("tlbi vmalls12e1is");
                                    core::arch::asm!("dsb nsh");
                                }
                                /* after update gpm, need to flush iommu table
                                 * in x86_64
                                 */
                                #[cfg(target_arch = "x86_64")]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::arch::iommu::flush(
                                        this_zone_id(),
                                        vbdf.bus,
                                        (vbdf.device << 3) + vbdf.function,
                                    );
                                }
                            }
                        }
                    }
                    Ok(None)
                } else {
                    // read bar
                    if (dev.with_config_value(|configvalue| configvalue.get_bar_value(slot))
                        & 0xfffffff0)
                        == 0xfffffff0
                    {
                        /*
                         * tmp_value being 0xFFFF_FFFF means that Linux is attempting to determine the BAR size.
                         * The value of tmp_value is used directly here because Linux will rewrite this register later,
                         * so the Hvisor does not need to preserve any additional state.
                         */
                        Ok(Some(
                            dev.with_bar_ref(slot, |bar| bar.get_size_with_flag()) as usize
                        ))
                    } else {
                        Ok(Some(
                            dev.with_config_value(|configvalue| configvalue.get_bar_value(slot))
                                as usize,
                        ))
                    }
                }
            } else {
                Ok(None)
            }
        }
        BridgeField::ExpansionRomBar => {
            // rom is same with bar
            let rom_type = dev.with_rom_ref(|rom| rom.get_type());
            if rom_type == PciMemType::Rom {
                if is_write {
                    if is_direct && is_root {
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_rom_value(value as u32);
                        });
                        if value & 0xfffff800 != 0xfffff800 {
                            dev.write_hw(
                                field.to_offset() as PciConfigAddress,
                                field.size(),
                                value,
                            )?;

                            let new_vaddr = (value as u64) & !0xf;

                            // set virt_value
                            dev.with_rom_ref_mut(|rom| rom.set_virtual_value(new_vaddr));

                            // set value
                            dev.with_rom_ref_mut(|rom| rom.set_value(new_vaddr));
                        }
                    } else if is_dev_belong_to_zone {
                        // normal mode, update virt resources
                        dev.with_config_value_mut(|configvalue| {
                            configvalue.set_rom_value(value as u32);
                        });

                        // Check if this is size probe (all 1s in BA field, bits 31-11)
                        let is_size_probe = (value & 0xfffff800) == 0xfffff800;
                        // Check if ROM enable bit (bit 0) is set
                        let rom_enabled = (value & 0x1) != 0;

                        if !is_size_probe {
                            let old_vaddr =
                                dev.with_rom_ref(|rom| rom.get_virtual_value64()) & !0xf;
                            let new_vaddr = (value as u64) & !0xf;

                            // Only perform mapping operations if ROM enable bit is set
                            if rom_enabled {
                                // set new_value not new_vaddr, because `set_virtual_value` will not add enable flag automatically
                                dev.with_rom_ref_mut(|rom| rom.set_virtual_value(value as _));

                                // Write to hardware with enable bit set
                                // Get the current ROM value from hardware and set bit 0
                                // And not to use rom.set_value()
                                let hw_value = dev.with_rom_ref(|rom| rom.get_value64());
                                let hw_value_enabled = hw_value | 0x1; // Set enable bit
                                dev.write_hw(
                                    field.to_offset() as PciConfigAddress,
                                    field.size(),
                                    hw_value_enabled as usize,
                                )?;
                                dev.with_rom_ref_mut(|rom| rom.set_value(hw_value_enabled));

                                let paddr =
                                    dev.with_rom_ref(|rom| rom.get_value64()) as HostPhysAddr;

                                let rom_size = {
                                    let size = dev.with_rom_ref(|rom| rom.get_size());
                                    if crate::memory::addr::is_aligned(size as usize) {
                                        size
                                    } else {
                                        crate::memory::PAGE_SIZE as u64
                                    }
                                };
                                let new_vaddr_aligned =
                                    if !crate::memory::addr::is_aligned(new_vaddr as usize) {
                                        crate::memory::addr::align_up(new_vaddr as usize) as u64
                                    } else {
                                        new_vaddr as u64
                                    };

                                let zone = this_zone();
                                let mut guard = zone.write();
                                let gpm = guard.gpm_mut();

                                if !gpm
                                    .try_delete(old_vaddr.try_into().unwrap(), rom_size as usize)
                                    .is_ok()
                                {
                                    // warn!("delete rom bar: can not found 0x{:x}", old_vaddr);
                                }
                                gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                    new_vaddr_aligned as GuestPhysAddr,
                                    paddr as HostPhysAddr,
                                    rom_size as _,
                                    MemFlags::READ | MemFlags::WRITE,
                                ))?;
                                drop(guard);
                                /* after update gpm, mem barrier is needed
                                 */
                                #[cfg(target_arch = "aarch64")]
                                unsafe {
                                    core::arch::asm!("isb");
                                    core::arch::asm!("tlbi vmalls12e1is");
                                    core::arch::asm!("dsb nsh");
                                }
                                /* after update gpm, need to flush iommu table
                                 * in x86_64
                                 */
                                #[cfg(target_arch = "x86_64")]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::arch::iommu::flush(
                                        this_zone_id(),
                                        vbdf.bus,
                                        (vbdf.device << 3) + vbdf.function,
                                    );
                                }
                                #[cfg(target_arch = "riscv64")]
                                unsafe {
                                    // TOOD: add remote fence support (using sbi rfence spec?)
                                    core::arch::asm!("hfence.gvma");
                                }
                            } else {
                                // ROM disabled
                            }
                        }
                    }
                    Ok(None)
                } else {
                    // read rom bar
                    if (dev.with_config_value(|configvalue| configvalue.get_rom_value()))
                        & 0xfffff800
                        == 0xfffff800
                    {
                        /*
                         * config_value being 0xFFFF_FFFF means that Linux is attempting to determine the ROM size.
                         * The value is used directly here because Linux will rewrite this register later,
                         * so the Hvisor does not need to preserve any additional state.
                         */
                        Ok(Some(
                            dev.with_rom_ref(|rom| rom.get_size_with_flag()) as usize
                        ))
                    } else {
                        Ok(Some(
                            dev.with_config_value(|configvalue| configvalue.get_rom_value())
                                as usize,
                        ))
                    }
                }
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/*
 * is_direct: if true, root can allocate resource for device belonging
 *            to ohter zone but can't drive it
 * is_root: if the access is from the root zone
 * is_dev_belong_to_zone: if the access is from the device that belongs to the zone
 */
fn handle_config_space_access(
    dev: ArcRwLockVirtualPciConfigSpace,
    mmio: &mut MMIOAccess,
    offset: PciConfigAddress,
    is_direct: bool,
    is_root: bool,
    is_dev_belong_to_zone: bool,
) -> HvResult {
    let is_write = mmio.is_write;

    // the lenth of access and control bits are limited by BIT_LENTH
    if (offset as usize) >= BIT_LENTH {
        warn!("invalid pci offset {:#x}", offset);
        if !is_write {
            mmio.value = 0;
        }
        return Ok(());
    }

    let size = mmio.size;
    let value = mmio.value;

    let vbdf = dev.get_bdf();
    let dev_type = dev.get_dev_type();

    if is_root || is_dev_belong_to_zone {
        match dev.access(offset, size) {
            false => {
                // Hardware access path
                pci_log!(
                    "hw vbdf {:#?} reg 0x{:x} try {} {}",
                    vbdf,
                    offset,
                    if is_write { "write" } else { "read" },
                    if is_write {
                        format!("0x{:x}", mmio.value)
                    } else {
                        String::new()
                    }
                );
                if is_write {
                    dev.write_hw(offset, size, value)?;
                } else {
                    mmio.value = dev.read_hw(offset, size).unwrap();
                }
            }
            true => {
                // Emulation access path
                pci_log!(
                    "emu vbdf {:#?} reg 0x{:x} try {} {}",
                    vbdf,
                    offset,
                    if is_write { "write" } else { "read" },
                    if is_write {
                        format!(" 0x{:x}", mmio.value)
                    } else {
                        String::new()
                    }
                );
                match dev_type {
                    VpciDevType::Physical => {
                        let config_type = dev.get_config_type();
                        match config_type {
                            HeaderType::Endpoint => {
                                // Check if this is capability region access (offset >= 0x40)
                                if (offset >= 0x40 && offset < 0x100) || (offset == 0x34) {
                                    if let Some(val) = handle_cap_access(
                                        dev,
                                        offset,
                                        size,
                                        value,
                                        is_write,
                                        is_dev_belong_to_zone,
                                    )? {
                                        mmio.value = val;
                                    }
                                } else {
                                    if let Some(val) = handle_endpoint_access(
                                        dev,
                                        EndpointField::from(offset as usize, size),
                                        value,
                                        is_write,
                                        is_direct,
                                        is_root,
                                        is_dev_belong_to_zone,
                                    )? {
                                        mmio.value = val;
                                    }
                                }
                            }
                            HeaderType::PciBridge => {
                                // Check if this is capability region access (offset >= 0x40)
                                if (offset >= 0x40 && offset < 0x100) || (offset == 0x34) {
                                    if let Some(val) = handle_cap_access(
                                        dev,
                                        offset,
                                        size,
                                        value,
                                        is_write,
                                        is_dev_belong_to_zone,
                                    )? {
                                        mmio.value = val;
                                    }
                                } else {
                                    if let Some(val) = handle_pci_bridge_access(
                                        dev,
                                        BridgeField::from(offset as usize, size),
                                        value,
                                        is_write,
                                        is_direct,
                                        is_root,
                                        is_dev_belong_to_zone,
                                    )? {
                                        mmio.value = val;
                                    }
                                }
                            }
                            _ => {
                                mmio.value = 0;
                            }
                        }
                    }
                    _ => {
                        // virt pci dev
                        if let Some(val) =
                            handle_virtio_pci_request(dev, offset, size, value, is_write)?
                        {
                            mmio.value = val
                        }
                        // if let Some(val) =
                        //     handle_virt_pci_request(dev, offset, size, value, is_write, dev_type)?
                        // {
                        //     mmio.value = val;
                        // }
                    }
                }
            }
        }
    }

    pci_log!(
        "vbdf {:#?} reg 0x{:x} {} 0x{:x}",
        vbdf,
        offset,
        if is_write { "write" } else { "read" },
        mmio.value
    );
    Ok(())
}

fn handle_device_not_found(mmio: &mut MMIOAccess, offset: PciConfigAddress) {
    /* if the dev is None, just return 0xFFFF_FFFF when read ID */
    if !mmio.is_write {
        match EndpointField::from(offset as usize, mmio.size) {
            EndpointField::ID => {
                mmio.value = 0xFFFF_FFFF;
            }
            _ => {
                // warn!("unhandled pci mmio read, addr: {:#x?}", mmio.address);
                mmio.value = 0;
            }
        }
    }
}

pub fn mmio_vpci_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    // info!("mmio_vpci_handler {:#x}", mmio.address);
    let zone = this_zone();
    let offset = (mmio.address & 0xfff) as PciConfigAddress;
    let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;

    let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
        let guard = zone.read();
        let vbus = guard.vpci_bus();
        vbus.get_device_by_base(base)
    };

    let is_root = is_this_root_zone();

    if let Some(dev) = dev {
        handle_config_space_access(dev, mmio, offset, false, is_root, true)?;
    } else {
        handle_device_not_found(mmio, offset);
    }

    Ok(())
}

#[cfg(feature = "dwc_pcie")]
pub fn mmio_dwc_io_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    {
        let zone = this_zone();
        let guard = zone.read();

        let atu_config = guard
            .atu_configs()
            .get_atu_by_io_base(_base as PciConfigAddress)
            .and_then(|atu| {
                guard
                    .atu_configs()
                    .get_ecam_by_io_base(_base as PciConfigAddress)
                    .map(|ecam| (*atu, ecam))
            });

        drop(guard);

        if let Some((atu, ecam_base)) = atu_config {
            use crate::platform;
            if let Some(extend_config) = platform::ROOT_DWC_ATU_CONFIG
                .iter()
                .find(|cfg| cfg.ecam_base == ecam_base as u64)
            {
                // Create DBI backend
                let dbi_base = extend_config.dbi_base as PciConfigAddress;
                let dbi_size = extend_config.dbi_size;
                let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
                let dbi_backend = DwcConfigRegionBackend::new(dbi_region);

                // Call AtuUnroll to program the ATU
                AtuUnroll::dw_pcie_prog_outbound_atu_unroll(&dbi_backend, &atu)?;
            }
            mmio_perform_access(atu.pci_target() as usize, mmio);
        } else {
            warn!("No ATU config yet, do nothing");
        }
    }
    Ok(())
}

#[cfg(feature = "dwc_pcie")]
pub fn mmio_dwc_cfg_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    // info!("mmio_dwc_cfg_handler {:#x}", mmio.address + _base);
    let zone = this_zone();
    let guard = zone.read();

    let atu_config = guard
        .atu_configs()
        .get_atu_by_cfg_base(_base as PciConfigAddress)
        .and_then(|atu| {
            guard
                .atu_configs()
                .get_ecam_by_cfg_base(_base as PciConfigAddress)
                .map(|ecam| (*atu, ecam))
        });

    drop(guard);

    if let Some((atu, ecam_base)) = atu_config {
        // Get dbi_base from platform config (usually dbi_base == ecam_base)
        use crate::platform;
        if let Some(extend_config) = platform::ROOT_DWC_ATU_CONFIG
            .iter()
            .find(|cfg| cfg.ecam_base == ecam_base as u64)
        {
            // Create DBI backend
            let dbi_base = extend_config.dbi_base as PciConfigAddress;
            let dbi_size = extend_config.dbi_size;
            let dbi_region = PciRegionMmio::new(dbi_base, dbi_size);
            let dbi_backend = DwcConfigRegionBackend::new(dbi_region);

            // warn!("atu config {:#?}", atu);

            // Call AtuUnroll to program the ATU
            AtuUnroll::dw_pcie_prog_outbound_atu_unroll(&dbi_backend, &atu)?;
        }

        let offset = (mmio.address & 0xfff) as PciConfigAddress;
        let zone = this_zone();
        let mut is_dev_belong_to_zone = false;

        let base = mmio.address as PciConfigAddress - offset + atu.pci_target();

        let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
            let mut guard = zone.write();
            let vbus = guard.vpci_bus_mut();
            if let Some(dev) = vbus.get_device_by_base(base) {
                is_dev_belong_to_zone = true;
                Some(dev)
            } else {
                drop(guard);
                // Clone Arc first while holding GLOBAL_PCIE_LIST lock, then release it
                // This avoids holding multiple locks simultaneously
                let dev_clone = {
                    let global_pcie_list = GLOBAL_PCIE_LIST.lock();
                    global_pcie_list
                        .values()
                        .find(|dev| {
                            let dev_guard = dev.read();
                            dev_guard.get_base() == base
                        })
                        .cloned()
                };
                dev_clone
            }
        };

        let dev = match dev {
            Some(dev) => dev,
            None => {
                handle_device_not_found(mmio, offset);
                return Ok(());
            }
        };

        let is_root = is_this_root_zone();
        let is_direct = true; // dwc_cfg_handler uses direct mode

        handle_config_space_access(dev, mmio, offset, is_direct, is_root, is_dev_belong_to_zone)?;
    } else {
        warn!("No ATU config yet, do nothing");
    }
    Ok(())
}

#[cfg(feature = "dwc_pcie")]
pub fn mmio_vpci_handler_dbi(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    // info!("mmio_vpci_handler_dbi {:#x}", mmio.address);

    use crate::platform;

    // Decode domain_id and ecam_base from arg:
    // arg = ecam_base + domain_id
    // Since ecam_base is 4KB aligned (low 12 bits are 0),
    // low bits contain domain_id, high bits contain ecam_base
    let domain_id = (_base & 0xF) as u8;
    let ecam_base = _base - (domain_id as usize);

    #[cfg(all(feature = "pci_init_delay", feature = "dwc_msi"))]
    {
        if !*DBI_PCI_INIT_DONE.lock() && mmio.address != 0 {
            match mmio.address {
                PCIE_MSI_ADDR_LO | PCIE_MSI_ADDR_HI => {
                    let zone = this_zone();
                    let mut guard = zone.write();
                    let vbus = guard.vpci_bus_mut();

                    if vbus.domain_msi_info().get(&domain_id).is_none() {
                        vbus.add_msi_count_for_domain(domain_id, 1, 0);
                    }

                    if let Some(domain_msi_info) = vbus.domain_msi_info_mut().get_mut(&domain_id) {
                        if mmio.is_write {
                            let vm_doorbell = domain_msi_info.get_vm_doorbell();
                            let new_val = if mmio.address == PCIE_MSI_ADDR_LO {
                                (vm_doorbell & 0xffffffff00000000) | (mmio.value as u64)
                            } else {
                                (vm_doorbell & 0xffffffff) | ((mmio.value as u64) << 32)
                            };
                            domain_msi_info.set_vm_doorbell(new_val);
                        } else {
                            let vm_doorbell = domain_msi_info.get_vm_doorbell();
                            mmio.value = if mmio.address == PCIE_MSI_ADDR_LO {
                                (vm_doorbell & 0xffffffff) as usize
                            } else {
                                ((vm_doorbell >> 32) & 0xffffffff) as usize
                            };
                        }
                    }

                    return Ok(());
                }
                _ => {
                    mmio_perform_access(ecam_base, mmio);
                    return Ok(());
                }
            }
        }
    }

    #[cfg(feature = "pci_init_delay")]
    {
        if !*DBI_PCI_INIT_DONE.lock() && mmio.address == 0 {
            let root_config = platform::platform_root_zone_config();
            let num_pci_bus = root_config.num_pci_bus as usize;

            crate::pci::pci_config::hvisor_pci_init(&root_config.pci_config[..num_pci_bus])?;

            let zone = crate::zone::root_zone();
            let mut inner = zone.write();
            inner.virtual_pci_mmio_init(&root_config.pci_config, num_pci_bus);
            inner.guest_pci_init(
                0,
                &root_config.alloc_pci_devs,
                root_config.num_pci_devs,
                &root_config.pci_config,
                num_pci_bus,
            )?;

            #[cfg(feature = "dwc_msi")]
            {
                let hw_paddr = crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);
                if hw_paddr != 0 {
                    let mut hw_lo_write = MMIOAccess {
                        address: PCIE_MSI_ADDR_LO,
                        value: (hw_paddr & 0xffffffff) as usize,
                        size: 4,
                        is_write: true,
                    };
                    let mut hw_hi_write = MMIOAccess {
                        address: PCIE_MSI_ADDR_HI,
                        value: ((hw_paddr >> 32) & 0xffffffff) as usize,
                        size: 4,
                        is_write: true,
                    };
                    mmio_perform_access(ecam_base, &mut hw_lo_write);
                    mmio_perform_access(ecam_base, &mut hw_hi_write);
                }
            }

            *DBI_PCI_INIT_DONE.lock() = true;
            info!("Hvisor PCI initialization complete");
        }
    }

    // Read extend_config to get io_atu_index
    let extend_config = platform::ROOT_DWC_ATU_CONFIG
        .iter()
        .find(|cfg| cfg.ecam_base == ecam_base as u64);

    if let Some(extend_config) = extend_config {
        let io_atu_index = extend_config.io_atu_index as usize;
        let atu_base = ATU_BASE + io_atu_index * ATU_REGION_SIZE;

        /* Calculate outbound atu registers address range based on io_atu_index
         * Each ATU has: 0x0-0x100 for outbound, 0x100-0x200 for inbound
         * We only handle outbound now, so MAX is atu_base + ATU_REGION_SIZE/2
         */
        if mmio.address >= atu_base && mmio.address < atu_base + ATU_REGION_SIZE / 2 {
            let zone = this_zone();
            let mut guard = zone.write();
            let atu_offset = mmio.address - atu_base;

            // warn!("set atu{} register {:#X} value {:#X}", io_atu_index, atu_offset, mmio.value);

            let atu = guard
                .atu_configs_mut()
                .get_atu_by_ecam_mut(ecam_base)
                .unwrap();

            // info!("atu config write {:#?}", atu);

            if mmio.is_write {
                if mmio.size == 4 {
                    match atu_offset {
                        PCIE_ATU_UNR_REGION_CTRL1 => {
                            // info!("set atu{} region ctrl1 value {:#X}", io_atu_index, mmio.value);
                            atu.set_atu_type(AtuType::from_u8((mmio.value & 0xff) as u8));
                        }
                        PCIE_ATU_UNR_REGION_CTRL2 => {
                            // Enable bit is written here, but we just track it
                            // The actual enable is handled by the driver
                        }
                        PCIE_ATU_UNR_LOWER_BASE => {
                            // info!("set atu{} lower base value {:#X}", io_atu_index, mmio.value);
                            atu.set_cpu_base(
                                (atu.cpu_base() & !0xffffffff) | (mmio.value as PciConfigAddress),
                            );
                        }
                        PCIE_ATU_UNR_UPPER_BASE => {
                            // info!("set atu{} upper base value {:#X}", io_atu_index, mmio.value);
                            atu.set_cpu_base(
                                (atu.cpu_base() & 0xffffffff)
                                    | ((mmio.value as PciConfigAddress) << 32),
                            );
                        }
                        PCIE_ATU_UNR_LIMIT => {
                            // info!("set atu{} limit value {:#X}", io_atu_index, mmio.value);
                            atu.set_cpu_limit(
                                (atu.cpu_limit() & !0xffffffff) | (mmio.value as PciConfigAddress),
                            );
                        }
                        PCIE_ATU_UNR_UPPER_LIMIT => {
                            // Update the upper 32 bits of cpu_limit
                            atu.set_cpu_limit(
                                (atu.cpu_limit() & 0xffffffff)
                                    | ((mmio.value as PciConfigAddress) << 32),
                            );
                        }
                        PCIE_ATU_UNR_LOWER_TARGET => {
                            // info!("set atu{} lower target value {:#X}", io_atu_index, mmio.value);
                            atu.set_pci_target(
                                (atu.pci_target() & !0xffffffff) | (mmio.value as PciConfigAddress),
                            );
                        }
                        PCIE_ATU_UNR_UPPER_TARGET => {
                            // info!("set atu{} upper target value {:#X}", io_atu_index, mmio.value);
                            atu.set_pci_target(
                                (atu.pci_target() & 0xffffffff)
                                    | ((mmio.value as PciConfigAddress) << 32),
                            );
                        }
                        _ => {
                            warn!(
                                "invalid atu{} write {:#x} + {:#x}",
                                io_atu_index, atu_offset, mmio.size
                            );
                        }
                    }
                } else {
                    warn!("invalid atu{} read size {:#x}", io_atu_index, mmio.size);
                }
            } else {
                // Read from virtual ATU
                // warn!("read atu{} {:#x}", io_atu_index, atu_offset);
                match atu_offset {
                    PCIE_ATU_UNR_REGION_CTRL1 => {
                        mmio.value = atu.atu_type() as usize;
                    }
                    PCIE_ATU_UNR_REGION_CTRL2 => {
                        mmio.value = ATU_ENABLE_BIT as usize;
                    }
                    PCIE_ATU_UNR_LOWER_BASE => {
                        mmio.value = (atu.cpu_base() & 0xffffffff) as usize;
                    }
                    PCIE_ATU_UNR_UPPER_BASE => {
                        mmio.value = ((atu.cpu_base() >> 32) & 0xffffffff) as usize;
                    }
                    PCIE_ATU_UNR_LIMIT => {
                        let limit_value = (atu.cpu_limit() & 0xffffffff) as usize;
                        mmio.value = if limit_value == 0 {
                            atu.limit_hw_value() as usize
                        } else {
                            limit_value
                        };
                    }
                    PCIE_ATU_UNR_UPPER_LIMIT => {
                        let upper_limit = ((atu.cpu_limit() >> 32) & 0xffffffff) as usize;
                        mmio.value = if upper_limit == 0xffffffff {
                            atu.upper_limit_hw_value() as usize
                        } else {
                            upper_limit
                        };
                    }
                    PCIE_ATU_UNR_LOWER_TARGET => {
                        mmio.value = (atu.pci_target() & 0xffffffff) as usize;
                    }
                    PCIE_ATU_UNR_UPPER_TARGET => {
                        mmio.value = ((atu.pci_target() >> 32) & 0xffffffff) as usize;
                    }
                    _ => {
                        warn!("invalid atu{} read {:#x}", io_atu_index, atu_offset);
                        mmio_perform_access(_base, mmio);
                    }
                }
            }
        } else if mmio.address > ATU_BASE {
            mmio_perform_access(ecam_base, mmio);
        } else if mmio.address >= BIT_LENTH
            && !(mmio.address >= PCIE_MSI_ADDR_LO && mmio.address <= PCIE_MSI_INTR0_STATUS)
        {
            // dbi read
            mmio_perform_access(ecam_base, mmio);
        } else if mmio.address >= PCIE_MSI_ADDR_LO && mmio.address <= PCIE_MSI_INTR0_STATUS {
            // Handle MSI registers - virtuize only if dwc_msi feature enabled
            #[cfg(feature = "dwc_msi")]
            {
                // Handle MSI registers
                let dbi_offset = mmio.address;
                let zone = this_zone();

                let mut guard = zone.write();
                let vbus = guard.vpci_bus_mut();

                if let Some(domain_msi_info) = vbus.domain_msi_info_mut().get_mut(&domain_id) {
                    match dbi_offset {
                        PCIE_MSI_ADDR_LO => {
                            if mmio.is_write {
                                // VM writes low 32 bits of doorbell address
                                let new_doorbell = (domain_msi_info.get_vm_doorbell()
                                    & 0xffffffff00000000)
                                    | (mmio.value as u64);
                                domain_msi_info.set_vm_doorbell(new_doorbell);

                                // Check if hardware doorbell matches hvisor's allocation from DW_MSI_DOMAINS
                                // Read current hardware ADDR_LO and ADDR_HI to get full doorbell address
                                let mut hw_hi_mmio = MMIOAccess {
                                    address: PCIE_MSI_ADDR_HI,
                                    value: 0,
                                    size: 4,
                                    is_write: false,
                                };
                                // After VM writes LO, hardware still has old LO value
                                // We'll use the new LO from VM write and existing HI from hardware
                                mmio_perform_access(ecam_base, &mut hw_hi_mmio);
                                let hw_doorbell =
                                    ((hw_hi_mmio.value as u64) << 32) | (mmio.value as u64);

                                // Get the authoritative doorbell from DW_MSI_DOMAINS
                                // Actually vm set the doorbell only when this board doesn't support arch MSI
                                let hw_paddr =
                                    crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);

                                // If hardware doorbell doesn't match hvisor's allocation, sync it
                                if hw_doorbell != hw_paddr && hw_paddr != 0 {
                                    let hw_paddr_lo = (hw_paddr & 0xffffffff) as u32;
                                    let hw_paddr_hi = ((hw_paddr >> 32) & 0xffffffff) as u32;

                                    // Write hvisor's doorbell LO
                                    let mut hw_lo_write = MMIOAccess {
                                        address: PCIE_MSI_ADDR_LO,
                                        value: hw_paddr_lo as usize,
                                        size: 4,
                                        is_write: true,
                                    };
                                    mmio_perform_access(ecam_base, &mut hw_lo_write);

                                    // Write hvisor's doorbell HI (only if needed)
                                    if hw_paddr_hi != (hw_hi_mmio.value as u32) {
                                        let mut hw_hi_write = MMIOAccess {
                                            address: PCIE_MSI_ADDR_HI,
                                            value: hw_paddr_hi as usize,
                                            size: 4,
                                            is_write: true,
                                        };
                                        mmio_perform_access(ecam_base, &mut hw_hi_write);
                                    }

                                    info!(
                                    "Domain {} MSI ADDR_LO write: {:#x}, synced hw doorbell to {:#x}",
                                    domain_id, mmio.value, hw_paddr
                                );
                                } else {
                                    info!(
                                    "Domain {} MSI ADDR_LO write: {:#x}, hw doorbell already matches {:#x}",
                                    domain_id, mmio.value, hw_doorbell
                                );
                                }
                            } else {
                                // Return the low 32 bits of VM doorbell
                                mmio.value =
                                    (domain_msi_info.get_vm_doorbell() & 0xffffffff) as usize;
                            }
                        }
                        PCIE_MSI_ADDR_HI => {
                            if mmio.is_write {
                                // VM writes high 32 bits of doorbell address
                                let new_doorbell = (domain_msi_info.get_vm_doorbell() & 0xffffffff)
                                    | ((mmio.value as u64) << 32);
                                domain_msi_info.set_vm_doorbell(new_doorbell);

                                // Check if hardware doorbell matches hvisor's allocation from DW_MSI_DOMAINS
                                // Read current hardware ADDR_LO and ADDR_HI to get full doorbell address
                                let mut hw_lo_mmio = MMIOAccess {
                                    address: PCIE_MSI_ADDR_LO,
                                    value: 0,
                                    size: 4,
                                    is_write: false,
                                };
                                mmio_perform_access(ecam_base, &mut hw_lo_mmio);
                                let hw_doorbell =
                                    ((mmio.value as u64) << 32) | (hw_lo_mmio.value as u64);

                                // Get the authoritative doorbell from DW_MSI_DOMAINS
                                let hw_paddr =
                                    crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);

                                // If hardware doorbell doesn't match hvisor's allocation, sync it
                                if hw_doorbell != hw_paddr && hw_paddr != 0 {
                                    let hw_paddr_lo = (hw_paddr & 0xffffffff) as u32;
                                    let hw_paddr_hi = ((hw_paddr >> 32) & 0xffffffff) as u32;

                                    // Write hvisor's doorbell HI
                                    let mut hw_hi_write = MMIOAccess {
                                        address: PCIE_MSI_ADDR_HI,
                                        value: hw_paddr_hi as usize,
                                        size: 4,
                                        is_write: true,
                                    };
                                    mmio_perform_access(ecam_base, &mut hw_hi_write);

                                    // Write hvisor's doorbell LO (only if needed)
                                    if hw_paddr_lo != (hw_lo_mmio.value as u32) {
                                        let mut hw_lo_write = MMIOAccess {
                                            address: PCIE_MSI_ADDR_LO,
                                            value: hw_paddr_lo as usize,
                                            size: 4,
                                            is_write: true,
                                        };
                                        mmio_perform_access(ecam_base, &mut hw_lo_write);
                                    }

                                    info!(
                                    "Domain {} MSI ADDR_HI write: {:#x}, synced hw doorbell to {:#x}",
                                    domain_id, mmio.value, hw_paddr
                                );
                                } else {
                                    info!(
                                    "Domain {} MSI ADDR_HI write: {:#x}, hw doorbell already matches {:#x}",
                                    domain_id, mmio.value, hw_doorbell
                                );
                                }
                            } else {
                                // Return the high 32 bits of VM doorbell
                                mmio.value = ((domain_msi_info.get_vm_doorbell() >> 32)
                                    & 0xffffffff)
                                    as usize;
                            }
                        }
                        PCIE_MSI_INTR0_ENABLE | PCIE_MSI_INTR0_MASK | PCIE_MSI_INTR0_STATUS => {
                            // All three registers use the same bit shifting and masking logic
                            let hwirq_bit = domain_msi_info.hwirq_bit;
                            let vm_mask = domain_msi_info.get_msi_mask();

                            if mmio.is_write {
                                // VM writes from virqbit 0-based perspective
                                // Convert to hardware perspective by left-shifting by hwirq_bit
                                let hw_value_vm =
                                    (mmio.value as u32 & vm_mask).wrapping_shl(hwirq_bit);

                                if dbi_offset == PCIE_MSI_INTR0_STATUS {
                                    // Status register: write 1 to clear semantics
                                    // Mask first to ensure VM can only clear its own bits
                                    // No need to read hardware value - just write the mapped bits
                                    // Hardware will clear only the bits we write as 1
                                    // Other domains' pending interrupts remain unaffected
                                    let mut hw_mmio_write = MMIOAccess {
                                        address: mmio.address,
                                        value: hw_value_vm as usize,
                                        size: 4,
                                        is_write: true,
                                    };
                                    mmio_perform_access(ecam_base, &mut hw_mmio_write);

                                    info!(
                                        "Domain {} MSI INTR0_STATUS write (virq): {:#x}, hw: {:#x}",
                                        domain_id, mmio.value, hw_value_vm
                                    );
                                } else {
                                    // For ENABLE/MASK registers: need to preserve other domain's bits
                                    // Read current hardware value
                                    let mut hw_mmio = MMIOAccess {
                                        address: mmio.address,
                                        value: 0,
                                        size: 4,
                                        is_write: false,
                                    };
                                    mmio_perform_access(ecam_base, &mut hw_mmio);
                                    let hw_value = hw_mmio.value as u32;

                                    // Create mask for this domain's MSI bits
                                    let domain_mask = vm_mask.wrapping_shl(hwirq_bit);

                                    // Update hardware value: clear domain bits, then set new ones
                                    let new_hw_value =
                                        (hw_value & !domain_mask) | (hw_value_vm & domain_mask);

                                    let mut hw_mmio_write = MMIOAccess {
                                        address: mmio.address,
                                        value: new_hw_value as usize,
                                        size: 4,
                                        is_write: true,
                                    };
                                    mmio_perform_access(ecam_base, &mut hw_mmio_write);

                                    let reg_name = match dbi_offset {
                                        PCIE_MSI_INTR0_ENABLE => "ENABLE",
                                        PCIE_MSI_INTR0_MASK => "MASK",
                                        _ => "UNKNOWN",
                                    };
                                    info!(
                                        "Domain {} MSI INTR0_{} write (virq): {:#x}, hw: {:#x}, mask: {:#x}",
                                        domain_id, reg_name, mmio.value, new_hw_value, domain_mask
                                    );
                                }
                            } else {
                                // Read and convert from hardware perspective to VM perspective
                                // Read hardware value
                                let mut hw_mmio = MMIOAccess {
                                    address: mmio.address,
                                    value: 0,
                                    size: 4,
                                    is_write: false,
                                };
                                mmio_perform_access(ecam_base, &mut hw_mmio);
                                let hw_value = hw_mmio.value as u32;

                                // Right shift to get VM perspective and mask
                                let vm_value = hw_value.wrapping_shr(hwirq_bit) & vm_mask;
                                mmio.value = vm_value as usize;
                            }
                        }
                        _ => {
                            // Other DBI registers
                            mmio_perform_access(ecam_base, mmio);
                        }
                    }
                } else {
                    warn!("No MSI domain info found for domain {}", domain_id);
                    mmio_perform_access(ecam_base, mmio);
                }
            }

            #[cfg(not(feature = "dwc_msi"))]
            {
                // Without dwc_msi feature, directly pass through MSI register access
                mmio_perform_access(ecam_base, mmio);
            }
        } else {
            // warn!("mmio_vpci_handler_dbi read {:#x}", mmio.address);
            let offset = (mmio.address & 0xfff) as PciConfigAddress;
            let zone = this_zone();
            let mut is_dev_belong_to_zone = false;

            let base = mmio.address as PciConfigAddress - offset + ecam_base as PciConfigAddress;

            let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
                let mut guard = zone.write();
                let vbus = guard.vpci_bus_mut();
                if let Some(dev) = vbus.get_device_by_base(base) {
                    is_dev_belong_to_zone = true;
                    Some(dev)
                } else {
                    drop(guard);
                    // Clone Arc first while holding GLOBAL_PCIE_LIST lock, then release it
                    // This avoids holding multiple locks simultaneously
                    let dev_clone = {
                        let global_pcie_list = GLOBAL_PCIE_LIST.lock();
                        global_pcie_list
                            .values()
                            .find(|dev| {
                                let dev_guard = dev.read();
                                dev_guard.get_base() == base
                            })
                            .cloned()
                    };
                    dev_clone
                }
            };

            let dev = match dev {
                Some(dev) => dev,
                None => {
                    handle_device_not_found(mmio, offset);
                    return Ok(());
                }
            };

            let is_root = is_this_root_zone();
            let is_direct = true; // dbi handler uses direct mode

            handle_config_space_access(
                dev,
                mmio,
                offset,
                is_direct,
                is_root,
                is_dev_belong_to_zone,
            )?;
        }
    } else {
        warn!("No extend config found for ecam_base {:#x}", _base);
    }

    Ok(())
}

#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
static DBI_PCI_INIT_DONE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
pub fn is_pci_init_done() -> bool {
    *DBI_PCI_INIT_DONE.lock()
}

pub fn mmio_vpci_direct_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    let zone = this_zone();
    let offset = (mmio.address & 0xfff) as PciConfigAddress;
    let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;
    let mut is_dev_belong_to_zone = false;

    let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
        let mut guard = zone.write();
        let vbus = guard.vpci_bus_mut();
        if let Some(dev) = vbus.get_device_by_base(base) {
            is_dev_belong_to_zone = true;
            Some(dev)
        } else {
            drop(guard);
            let global_pcie_list = GLOBAL_PCIE_LIST.lock();
            global_pcie_list
                .values()
                .find(|dev| dev.read().get_base() == base)
                .cloned()
        }
    };

    let dev = match dev {
        Some(dev) => dev,
        None => {
            handle_device_not_found(mmio, offset);
            return Ok(());
        }
    };

    let is_root = is_this_root_zone();
    let is_direct = true; // direct handler uses direct mode

    handle_config_space_access(dev, mmio, offset, is_direct, is_root, is_dev_belong_to_zone)?;

    Ok(())
}

/// Handle MMIO access to MSIX table in BAR memory
pub fn mmio_msix_table_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    let access_offset = mmio.address as u64;
    let base_aligned = (base as u64) & !0xf;

    // Find the device matching this BAR's physical address and get domain_id from BDF
    let (device_info, domain_id) = {
        let zone = this_zone();
        let guard = zone.read();
        let vbus = guard.vpci_bus();

        // Find the device whose MSIX BAR paddr matches the handler base
        let mut result = None;
        let mut domain_id = 0xFF;
        for dev in vbus.devs_ref().values() {
            if let Some(msi_info) = dev.read().get_msi_info() {
                if let Some(msix) = &msi_info.msix_info {
                    let msix_bar_aligned = msix.bar_paddr & !0xf;
                    if msix_bar_aligned == base_aligned {
                        // Get domain_id from device's BDF
                        domain_id = dev.read().get_bdf().domain();
                        result = Some((dev.clone(), msix.offset, msix.entry_count));
                        break;
                    }
                }
            }
        }

        if result.is_none() {
            let global_pcie_list = GLOBAL_PCIE_LIST.lock();
            for dev in global_pcie_list.values() {
                if let Some(msi_info) = dev.read().get_msi_info() {
                    if let Some(msix) = &msi_info.msix_info {
                        let msix_bar_aligned = msix.bar_paddr & !0xf;
                        if msix_bar_aligned == base_aligned {
                            domain_id = dev.read().get_bdf().domain();
                            result = Some((dev.clone(), msix.offset, msix.entry_count));
                            break;
                        }
                    }
                }
            }
        }
        (result, domain_id)
    };

    // Check if this access is within the MSIX table range
    if let Some((dev, msix_offset, entry_count)) = device_info {
        let vbdf = dev.get_vbdf();

        let msix_table_size = (entry_count as u64) * 16; // Each entry is 16 bytes
        let msix_table_end = msix_offset + msix_table_size;

        if access_offset >= msix_offset && access_offset < msix_table_end {
            // This is a MSIX table access, record it with detailed information
            let offset_in_entry = access_offset - msix_offset;
            let entry_index = offset_in_entry / 16;
            let field_offset = offset_in_entry % 16;

            if mmio.is_write {
                match field_offset {
                    0..=3 => {
                        let vm_lo = mmio.value;
                        info!(
                            "MSIX[vbdf {:#?}][entry {}] Message Address (Low) write: {:#x}",
                            vbdf, entry_index, mmio.value
                        );
                        // Save VM's doorbell low 32 bits
                        dev.with_msi_info_mut(|msi_info| {
                            let current = msi_info.msi_doorbell & 0xffffffff00000000;
                            msi_info.set_doorbell(current | (mmio.value as u64));
                        });

                        // Replace with hvisor's doorbell before writing to hardware
                        #[cfg(feature = "dwc_msi")]
                        {
                            if domain_id != 0xFF {
                                let hw_paddr =
                                    crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);
                                let hw_doorbell_lo = (hw_paddr & 0xffffffff) as usize;
                                let dev_vm_doorbell = dev
                                    .read()
                                    .get_msi_info()
                                    .map(|msi| msi.msi_doorbell)
                                    .unwrap_or(0);
                                let domain_vm_doorbell = {
                                    let zone = this_zone();
                                    let guard = zone.read();
                                    guard
                                        .vpci_bus()
                                        .domain_msi_info()
                                        .get(&domain_id)
                                        .map(|i| i.get_vm_doorbell())
                                        .unwrap_or(0)
                                };
                                warn!(
                                    "MSIX[vbdf {:#?}][entry {}] doorbell_lo map: vm={:#x} -> hv={:#x}, dev_vm_doorbell={:#x}, domain_vm_doorbell={:#x}",
                                    vbdf,
                                    entry_index,
                                    vm_lo,
                                    hw_doorbell_lo,
                                    dev_vm_doorbell,
                                    domain_vm_doorbell
                                );
                                mmio.value = hw_doorbell_lo;
                            }
                        }
                    }
                    4..=7 => {
                        let vm_hi = mmio.value;
                        info!(
                            "MSIX[vbdf {:#?}][entry {}] Message Address (High) write: {:#x}",
                            vbdf, entry_index, mmio.value
                        );
                        // Save VM's doorbell high 32 bits
                        dev.with_msi_info_mut(|msi_info| {
                            let current = msi_info.msi_doorbell & 0xffffffff;
                            msi_info.set_doorbell(current | ((mmio.value as u64) << 32));
                        });

                        // Replace with hvisor's doorbell before writing to hardware
                        #[cfg(feature = "dwc_msi")]
                        {
                            if domain_id != 0xFF {
                                let hw_paddr =
                                    crate::pci::dwc_msi::get_domain_doorbell_paddr(domain_id);
                                let hw_doorbell_hi = ((hw_paddr >> 32) & 0xffffffff) as usize;
                                let dev_vm_doorbell = dev
                                    .read()
                                    .get_msi_info()
                                    .map(|msi| msi.msi_doorbell)
                                    .unwrap_or(0);
                                let domain_vm_doorbell = {
                                    let zone = this_zone();
                                    let guard = zone.read();
                                    guard
                                        .vpci_bus()
                                        .domain_msi_info()
                                        .get(&domain_id)
                                        .map(|i| i.get_vm_doorbell())
                                        .unwrap_or(0)
                                };
                                warn!(
                                    "MSIX[vbdf {:#?}][entry {}] doorbell_hi map: vm={:#x} -> hv={:#x}, dev_vm_doorbell={:#x}, domain_vm_doorbell={:#x}",
                                    vbdf,
                                    entry_index,
                                    vm_hi,
                                    hw_doorbell_hi,
                                    dev_vm_doorbell,
                                    domain_vm_doorbell
                                );
                                mmio.value = hw_doorbell_hi;
                            }
                        }
                    }
                    8..=11 => {
                        info!(
                            "MSIX[vbdf {:#?}][entry {}] Message Data write: {:#x}",
                            vbdf, entry_index, mmio.value
                        );
                        // Convert VM vector index to hardware vector index.
                        #[cfg(feature = "dwc_msi")]
                        {
                            if domain_id != 0xFF {
                                let zone = this_zone();
                                let guard = zone.read();
                                let vbus = guard.vpci_bus();
                                if let Some(domain_msi_info) =
                                    vbus.domain_msi_info().get(&domain_id)
                                {
                                    let virq_bit = mmio.value as u32;
                                    let hwirq_bit = domain_msi_info.hwirq_bit;
                                    let hw_value = virq_bit.wrapping_add(hwirq_bit);
                                    mmio.value = hw_value as usize;

                                    info!(
                                        "MSIX[vbdf {:#?}][entry {}] Message Data converted: virq {:#x} -> hw {:#x} (hwirq_bit={})",
                                        vbdf, entry_index, virq_bit, hw_value, hwirq_bit
                                    );
                                }
                            }
                        }
                    }
                    12..=15 => {
                        info!(
                            "MSIX[vbdf {:#?}][entry {}] Vector Control write: {:#x} (masked={})",
                            vbdf,
                            entry_index,
                            mmio.value,
                            (mmio.value & 0x1) != 0
                        );
                    }
                    _ => {}
                }
            } else {
                // Handle read operations
                match field_offset {
                    0..=3 => {
                        // Return VM's doorbell low 32 bits
                        let dev_vm_doorbell = dev
                            .read()
                            .get_msi_info()
                            .map(|msi| msi.msi_doorbell)
                            .unwrap_or(0);
                        mmio.value = (dev_vm_doorbell & 0xffffffff) as usize;
                        #[cfg(feature = "dwc_msi")]
                        {
                            let domain_vm_doorbell = {
                                let zone = this_zone();
                                let guard = zone.read();
                                guard
                                    .vpci_bus()
                                    .domain_msi_info()
                                    .get(&domain_id)
                                    .map(|i| i.get_vm_doorbell())
                                    .unwrap_or(0)
                            };
                            warn!(
                                "MSIX[vbdf {:#?}][entry {}] doorbell_lo read: return={:#x}, dev_vm_doorbell={:#x}, domain_vm_doorbell={:#x}",
                                vbdf,
                                entry_index,
                                mmio.value,
                                dev_vm_doorbell,
                                domain_vm_doorbell
                            );
                        }
                        return Ok(());
                    }
                    4..=7 => {
                        // Return VM's doorbell high 32 bits
                        let dev_vm_doorbell = dev
                            .read()
                            .get_msi_info()
                            .map(|msi| msi.msi_doorbell)
                            .unwrap_or(0);
                        mmio.value = ((dev_vm_doorbell >> 32) & 0xffffffff) as usize;
                        #[cfg(feature = "dwc_msi")]
                        {
                            let domain_vm_doorbell = {
                                let zone = this_zone();
                                let guard = zone.read();
                                guard
                                    .vpci_bus()
                                    .domain_msi_info()
                                    .get(&domain_id)
                                    .map(|i| i.get_vm_doorbell())
                                    .unwrap_or(0)
                            };
                            warn!(
                                "MSIX[vbdf {:#?}][entry {}] doorbell_hi read: return={:#x}, dev_vm_doorbell={:#x}, domain_vm_doorbell={:#x}",
                                vbdf,
                                entry_index,
                                mmio.value,
                                dev_vm_doorbell,
                                domain_vm_doorbell
                            );
                        }
                        return Ok(());
                    }
                    8..=11 => {
                        // Read hardware value first
                        let mut hw_mmio = MMIOAccess {
                            address: mmio.address,
                            value: 0,
                            size: mmio.size,
                            is_write: false,
                        };
                        mmio_perform_access(base, &mut hw_mmio);

                        // Convert from hardware hwirq bit back to VM's virq bit
                        #[cfg(feature = "dwc_msi")]
                        {
                            if domain_id != 0xFF {
                                let zone = this_zone();
                                let guard = zone.read();
                                let vbus = guard.vpci_bus();
                                if let Some(domain_msi_info) =
                                    vbus.domain_msi_info().get(&domain_id)
                                {
                                    let hwirq_bit = domain_msi_info.hwirq_bit;
                                    let hw_value = hw_mmio.value as u32;
                                    let virq_bit = if hw_value >= hwirq_bit {
                                        hw_value - hwirq_bit
                                    } else {
                                        hw_value
                                    };
                                    mmio.value = virq_bit as usize;

                                    info!(
                                        "MSIX[vbdf {:#?}][entry {}] Message Data read converted: hw {:#x} -> virq {:#x} (hwirq_bit={})",
                                        vbdf, entry_index, hw_value, virq_bit, hwirq_bit
                                    );
                                }
                            }
                        }
                        return Ok(());
                    }
                    12..=15 => {
                        // Vector Control - just pass through
                    }
                    _ => {}
                }
            }
        }
    }

    mmio_perform_access(base, mmio);

    Ok(())
}
