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

#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
use spin::Lazy;
#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
use spin::Mutex;

use crate::cpu_data::this_zone;
use crate::error::HvResult;
use crate::memory::{mmio_perform_access, MMIOAccess};
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion};
use crate::pci::pci_struct::{
    CapabilityType, SRIOV_CAP_SIZE, SRIOV_VF_BAR_END, SRIOV_VF_BAR_OFFSET,
};
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

#[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
use super::dwc_msi::{
    PCIE_MSI_ADDR_HI, PCIE_MSI_ADDR_LO, PCIE_MSI_INTR0_ENABLE, PCIE_MSI_INTR0_MASK,
    PCIE_MSI_INTR0_STATUS,
};

#[cfg(not(feature = "dwc_msi"))]
const PCIE_MSI_ADDR_LO: usize = 0x820;
#[cfg(not(feature = "dwc_msi"))]
const PCIE_MSI_INTR0_STATUS: usize = 0x830;

const SRIOV_CTRL_OFFSET: PciConfigAddress = 0x08;
const SRIOV_NUM_VFS_OFFSET: PciConfigAddress = 0x10;
const SRIOV_CTRL_VF_ENABLE: u16 = 1 << 0;

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

fn collect_vf_device_copies(
    vf_host_bdf: super::pci_struct::Bdf,
) -> Vec<ArcRwLockVirtualPciConfigSpace> {
    let mut devices = Vec::new();

    {
        let zone = this_zone();
        let guard = zone.read();
        let vbus = guard.vpci_bus();
        for dev in vbus.devs_ref().values() {
            if dev.get_bdf() == vf_host_bdf {
                devices.push(dev.clone());
            }
        }
    }

    if let Some(dev) = GLOBAL_PCIE_LIST.lock().get(&vf_host_bdf).cloned() {
        devices.push(dev);
    }

    devices
}

fn sync_sriov_vf_bar_state(
    pf_dev: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
    value: usize,
) -> HvResult<bool> {
    if size != 4 {
        return Ok(false);
    }

    let Some((cap_offset, vf_bdfs)) =
        pf_dev.with_sriov_info(|sriov_info| (sriov_info.cap_offset, sriov_info.vf_bdfs.clone()))
    else {
        return Ok(false);
    };

    let vf_bar_start = cap_offset + SRIOV_VF_BAR_OFFSET;
    if offset < vf_bar_start || offset >= cap_offset + SRIOV_VF_BAR_END {
        return Ok(false);
    }

    let relative = offset - vf_bar_start;
    if (relative & 0x3) != 0 {
        return Ok(false);
    }
    let slot = (relative / 4) as usize;

    let Some((bar_type, bar_size)) = pf_dev.with_sriov_info(|sriov_info| {
        (
            sriov_info.vf_bars[slot].get_type(),
            sriov_info.vf_bars[slot].get_size(),
        )
    }) else {
        return Ok(false);
    };

    if (value & 0xfffffff0) == 0xfffffff0 {
        // PF-side SR-IOV BAR probing only queries the template in the ext cap.
        // VF BAR state is meaningful only after PF programs a valid BAR value.
        return Ok(true);
    }

    let pf_value = match bar_type {
        PciMemType::Mem64Low => {
            let low = pf_dev.read_hw(offset, size)? as u64;
            let high = pf_dev.read_hw(offset + 4, size)? as u64;
            (low | (high << 32)) & !0xf
        }
        PciMemType::Mem64High => {
            let low = pf_dev.read_hw(offset - 4, size)? as u64;
            let high = pf_dev.read_hw(offset, size)? as u64;
            (low | (high << 32)) & !0xf
        }
        PciMemType::Io => (pf_dev.read_hw(offset, size)? as u64) & !0x3,
        PciMemType::Mem32 => (pf_dev.read_hw(offset, size)? as u64) & !0xf,
        _ => return Ok(false),
    };

    for (vf_index, vf_bdf) in vf_bdfs.into_iter().enumerate() {
        let vf_value = pf_value.saturating_add((vf_index as u64).saturating_mul(bar_size));
        let propagated_value = match bar_type {
            PciMemType::Mem64Low => vf_value as u32 as usize,
            PciMemType::Mem64High => (vf_value >> 32) as u32 as usize,
            PciMemType::Io | PciMemType::Mem32 => vf_value as u32 as usize,
            _ => continue,
        };

        for vf_dev in collect_vf_device_copies(vf_bdf) {
            let is_root = is_this_root_zone();
            let is_dev_belong_to_zone = {
                let base = vf_dev.read().get_base();
                let zone = this_zone();
                let mut guard = zone.write();
                let vbus = guard.vpci_bus_mut();
                vbus.get_device_by_base(base).is_some()
            };

            // let vf_id = vf_dev
            //     .read()
            //     .get_sriov_vf_info()
            //     .map(|vf_info| vf_info.vf_index)
            //     .unwrap_or(vf_index as u16);

            let _ = handle_endpoint_access(
                vf_dev,
                EndpointField::Bar(slot),
                propagated_value,
                true,
                true,
                is_root,
                is_dev_belong_to_zone,
            )?;
        }
    }

    Ok(true)
}

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
    } else if offset >= 0x100 {
        #[cfg(feature = "sriov")]
        if let Some(cap_offset) = dev.with_sriov_info(|sriov_info| sriov_info.cap_offset) {
            if offset >= cap_offset && offset < cap_offset + SRIOV_CAP_SIZE {
                if is_write {
                    dev.write_hw(offset, size, value)?;

                    let _ = sync_sriov_vf_bar_state(dev.clone(), offset, size, value)?;
                    return Ok(None);
                }
                let read_value = dev.read_hw(offset, size)?;
                return Ok(Some(read_value));
            }
        }

        // When `sriov` feature is disabled, hide the SR-IOV extended capability
        // from guest VMs by patching the ext-cap linked list on the fly.
        #[cfg(not(feature = "sriov"))]
        if let Some(hide) = dev.with_hide_sriov(|h| h.clone()) {
            use bit_field::BitField;
            // Accesses inside the SR-IOV cap range: return 0 / silently drop writes.
            if offset >= hide.sriov_cap_offset && offset < hide.sriov_cap_offset + SRIOV_CAP_SIZE {
                if is_write {
                    return Ok(None);
                } else {
                    return Ok(Some(0));
                }
            }

            // Access to the first DWORD of the preceding cap node: patch the
            // `next` pointer so it skips over the SR-IOV cap.
            if let Some(prev_offset) = hide.prev_cap_offset {
                if offset >= prev_offset && offset < prev_offset + 4 {
                    if is_write {
                        // Pass writes through unchanged; the physical `next`
                        // pointer still points to SR-IOV which is fine for host.
                        dev.write_hw(offset, size, value)?;
                        return Ok(None);
                    } else {
                        // Always read the full DWORD, patch bits[31:20], then
                        // return the sub-slice the guest asked for.
                        let mut dw = dev.read_hw(prev_offset, 4)? as u32;
                        dw.set_bits(20..32, hide.sriov_cap_next as u32);
                        let byte_offset = (offset - prev_offset) as usize;
                        let result = match size {
                            1 => ((dw >> (byte_offset * 8)) & 0xFF) as usize,
                            2 => ((dw >> (byte_offset * 8)) & 0xFFFF) as usize,
                            _ => dw as usize,
                        };
                        return Ok(Some(result));
                    }
                }
            }
            // If SR-IOV is the first ext cap (prev_cap_offset == None) and the
            // guest reads offset 0x100, the cap header has already been zeroed
            // above so the guest sees no extended capabilities at all.
        }

        if is_write {
            dev.write_hw(offset, size, value)?;
            Ok(None)
        } else {
            Ok(Some(dev.read_hw(offset, size)?))
        }
    } else {
        // Other capability region offsets
        // Try to find the capability that contains this offset
        let cap_info = dev.with_cap(|capabilities| {
            capabilities
                .cap_in_config_ref()
                .range(..=offset as u64)
                .next_back()
                .map(|(cap_offset, cap)| (*cap_offset, cap.get_type()))
        });

        if let Some((cap_offset, cap_type)) = cap_info {
            let cap_offset = cap_offset as usize;
            let relative_offset = offset as usize - cap_offset;

            if cap_type == CapabilityType::Msi {
                let vbdf = dev.get_vbdf();
                let _domain_id = vbdf.domain();

                let is_msi_64 = dev.with_cap(|capabilities| {
                    capabilities
                        .cap_in_config_ref()
                        .get(&(cap_offset as u64))
                        .and_then(|cap| cap.with_region(|region| region.read(0x02, 2).ok()))
                        .map(|ctrl| (ctrl & (1 << 7)) != 0)
                        .unwrap_or(false)
                });

                let _is_addr_low = matches!(relative_offset, 4 | 5 | 6 | 7);
                let _is_addr_high = is_msi_64 && matches!(relative_offset, 8 | 9 | 10 | 11);
                let _is_msg_data = if is_msi_64 {
                    matches!(relative_offset, 12 | 13)
                } else {
                    matches!(relative_offset, 8 | 9)
                };

                #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
                {
                    if is_write {
                        if _is_addr_low {
                            dev.with_msi_info_mut(|msi_info| {
                                let current = msi_info.msi_doorbell & 0xffffffff00000000;
                                msi_info.set_doorbell(current | (value as u64));
                            });
                            let hw_paddr =
                                crate::pci::dwc_msi::get_domain_doorbell_paddr(_domain_id);
                            dev.write_hw(offset, size, (hw_paddr & 0xffffffff) as usize)?;
                            return Ok(None);
                        }
                        if _is_addr_high {
                            dev.with_msi_info_mut(|msi_info| {
                                let current = msi_info.msi_doorbell & 0xffffffff;
                                msi_info.set_doorbell(current | ((value as u64) << 32));
                            });
                            let hw_paddr =
                                crate::pci::dwc_msi::get_domain_doorbell_paddr(_domain_id);
                            dev.write_hw(offset, size, ((hw_paddr >> 32) & 0xffffffff) as usize)?;
                            return Ok(None);
                        }
                        if _is_msg_data {
                            let zone = this_zone();
                            let guard = zone.read();
                            let vbus = guard.vpci_bus();
                            if let Some(domain_msi_info) = vbus.domain_msi_info().get(&_domain_id) {
                                let hw_value =
                                    (value as u32).wrapping_add(domain_msi_info.hwirq_bit);
                                dev.write_hw(offset, size, hw_value as usize)?;
                            } else {
                                dev.write_hw(offset, size, value)?;
                            }
                            return Ok(None);
                        }
                    } else {
                        if _is_addr_low {
                            let vm_doorbell = dev
                                .read()
                                .get_msi_info()
                                .map(|msi_info| msi_info.msi_doorbell)
                                .unwrap_or(0);
                            return Ok(Some((vm_doorbell & 0xffffffff) as usize));
                        }
                        if _is_addr_high {
                            let vm_doorbell = dev
                                .read()
                                .get_msi_info()
                                .map(|msi_info| msi_info.msi_doorbell)
                                .unwrap_or(0);
                            return Ok(Some(((vm_doorbell >> 32) & 0xffffffff) as usize));
                        }
                        if _is_msg_data {
                            let hw_value = dev.read_hw(offset, size)?;
                            let zone = this_zone();
                            let guard = zone.read();
                            let vbus = guard.vpci_bus();
                            if let Some(domain_msi_info) = vbus.domain_msi_info().get(&_domain_id) {
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
            #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
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
                                | (bar_type == PciMemType::Mem64Low)
                                | (bar_type == PciMemType::Mem64High)
                                | (bar_type == PciMemType::Io)
                            {
                                let old_vaddr =
                                    dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                let new_vaddr = {
                                    match bar_type {
                                        PciMemType::Mem64Low => {
                                            let low_value = value as u32 as u64;
                                            let high_value = (dev
                                                .with_config_value(|cv| cv.get_bar_value(slot + 1))
                                                as u64)
                                                << 32;
                                            (low_value | high_value) & !0xf
                                        }
                                        PciMemType::Mem64High => {
                                            /* last 4bit is flag, not address and need ignore
                                             * flag will auto add when set_value and set_virtual_value
                                             * Read from config_value.bar_value cache instead of space
                                             */
                                            let low_value = dev
                                                .with_config_value(|cv| cv.get_bar_value(slot - 1))
                                                as u64;
                                            let high_value = (value as u32 as u64) << 32;
                                            (low_value | high_value) & !0xf
                                        }
                                        _ => (value as u64) & !0xf,
                                    }
                                };

                                // set virt_value
                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                } else if bar_type == PciMemType::Mem64Low {
                                    dev.with_bar_ref_mut(slot + 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                }

                                // set value
                                dev.with_bar_ref_mut(slot, |bar| bar.set_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| bar.set_value(new_vaddr));
                                } else if bar_type == PciMemType::Mem64Low {
                                    dev.with_bar_ref_mut(slot + 1, |bar| bar.set_value(new_vaddr));
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
                                #[cfg(all(target_arch = "x86_64", feature = "intel_vtd"))]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::device::iommu::flush(
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
                                | (bar_type == PciMemType::Mem64Low)
                                | (bar_type == PciMemType::Mem64High)
                                | (bar_type == PciMemType::Io)
                            {
                                let old_vaddr =
                                    dev.with_bar_ref(slot, |bar| bar.get_virtual_value64()) & !0xf;
                                let new_vaddr = {
                                    match bar_type {
                                        PciMemType::Mem64Low => {
                                            let low_value = value as u32 as u64;
                                            let high_value = (dev
                                                .with_config_value(|cv| cv.get_bar_value(slot + 1))
                                                as u64)
                                                << 32;
                                            (low_value | high_value) & !0xf
                                        }
                                        PciMemType::Mem64High => {
                                            /* last 4bit is flag, not address and need ignore
                                             * flag will auto add when set_value and set_virtual_value
                                             * Read from config_value.bar_value cache instead of space
                                             */
                                            let low_value = dev
                                                .with_config_value(|cv| cv.get_bar_value(slot - 1))
                                                as u64;
                                            let high_value = (value as u32 as u64) << 32;
                                            (low_value | high_value) & !0xf
                                        }
                                        _ => (value as u64) & !0xf,
                                    }
                                };

                                // info!("new_vaddr: {:#x}", new_vaddr);
                                // info!("old_vaddr: {:#x}", old_vaddr);
                                dev.with_bar_ref_mut(slot, |bar| bar.set_virtual_value(new_vaddr));
                                if bar_type == PciMemType::Mem64High {
                                    dev.with_bar_ref_mut(slot - 1, |bar| {
                                        bar.set_virtual_value(new_vaddr)
                                    });
                                } else if bar_type == PciMemType::Mem64Low {
                                    dev.with_bar_ref_mut(slot + 1, |bar| {
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
            #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
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
                                #[cfg(all(target_arch = "x86_64", feature = "intel_vtd"))]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::device::iommu::flush(
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
                                #[cfg(all(target_arch = "x86_64", feature = "intel_vtd"))]
                                {
                                    let vbdf = dev.get_vbdf();
                                    crate::device::iommu::flush(
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

    if !is_root && dev.read().get_sriov_vf_info().is_some() {
        if offset == 0x100 {
            mmio.value = 0x0;
            return Ok(());
        }
    }

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
                                if (offset >= 0x40 && offset < 0x100)
                                    || (offset == 0x34)
                                    || (offset >= 0x100)
                                {
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
                                if (offset >= 0x40 && offset < 0x100)
                                    || (offset == 0x34)
                                    || (offset >= 0x100)
                                {
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

            let pci_target = atu.pci_target();
            let target_bus = ((pci_target >> 24) & 0xff) as u8;
            let target_device = ((pci_target >> 19) & 0x1f) as u8;
            let target_function = ((pci_target >> 16) & 0x7) as u8;

            let mapped_target = {
                let zone_guard = zone.read();
                let vbus = zone_guard.vpci_bus();
                vbus.devs_ref().values().find_map(|dev| {
                    let vbdf = dev.get_vbdf();
                    if vbdf.bus() == target_bus
                        && vbdf.device() == target_device
                        && vbdf.function() == target_function
                    {
                        Some((dev.get_bdf(), dev.get_parent_bus()))
                    } else {
                        None
                    }
                })
            };

            let mut hw_pci_target = pci_target;
            let mut atu_type = atu.atu_type();
            let mut config_base = atu.cpu_base();
            let mut cpu_limit = atu.cpu_limit();
            if let Some((host_bdf, parent_bus)) = mapped_target {
                hw_pci_target = ((host_bdf.bus() as u64) << 24)
                    + ((host_bdf.device() as u64) << 19)
                    + ((host_bdf.function() as u64) << 16);
                (config_base, atu_type) = if parent_bus == 0 {
                    (extend_config.cfg_base, AtuType::Cfg0)
                } else {
                    (
                        extend_config.cfg_base + (extend_config.cfg_size / 2),
                        AtuType::Cfg1,
                    )
                };
                cpu_limit = config_base + (extend_config.cfg_size / 2) - 1;
            }

            // Program hardware ATU with translated host target when remap exists.
            let mut hw_atu = atu;
            hw_atu.set_pci_target(hw_pci_target);
            hw_atu.set_atu_type(atu_type);
            hw_atu.set_cpu_base(config_base);
            hw_atu.set_cpu_limit(cpu_limit);
            AtuUnroll::dw_pcie_prog_outbound_atu_unroll(&dbi_backend, &hw_atu)?;
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

        handle_config_space_access(
            dev.clone(),
            mmio,
            offset,
            is_direct,
            is_root,
            is_dev_belong_to_zone,
        )?;
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

    #[cfg(all(feature = "pci_init_delay", feature = "dwc_pcie"))]
    {
        // Delay mode semantics:
        // - Before init-done, accesses to non-zero DBI regs are normally passed through.
        // - For dwc_msi, MSI_ADDR_LO/HI are intercepted early so VM doorbell writes are cached.
        // - Access to DBI reg 0 triggers hvisor PCI init, then normal DBI virtualization continues.
        if !is_pci_init_done(domain_id) {
            if mmio.address != 0 {
                #[cfg(feature = "dwc_msi")]
                match mmio.address {
                    PCIE_MSI_ADDR_LO | PCIE_MSI_ADDR_HI => {
                        let zone = this_zone();
                        let mut guard = zone.write();
                        let vbus = guard.vpci_bus_mut();

                        if vbus.domain_msi_info().get(&domain_id).is_none() {
                            vbus.add_msi_count_for_domain(domain_id, 1, 0);
                        }

                        if let Some(domain_msi_info) =
                            vbus.domain_msi_info_mut().get_mut(&domain_id)
                        {
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
                    _ => {}
                }

                mmio_perform_access(ecam_base, mmio);
                return Ok(());
            }

            let root_config = platform::platform_root_zone_config();
            let num_pci_bus = root_config.num_pci_bus as usize;

            crate::pci::pci_config::hvisor_pci_init(&root_config.pci_config[..num_pci_bus])?;

            let zone = crate::zone::root_zone();
            let mut inner = zone.write();
            inner.virtual_pci_mmio_init_delay(&root_config.pci_config, num_pci_bus);
            inner.guest_pci_init_delay(
                0,
                &root_config.alloc_pci_devs,
                root_config.num_pci_devs,
                &root_config.pci_config,
                num_pci_bus,
            )?;

            #[cfg(feature = "dwc_msi")]
            {
                // Why this is inside init-delay only:
                // before init-done, VM may have already written MSI_ADDR_LO/HI and those writes were
                // cached (virtual doorbell) but did not program final hardware state.
                // After hvisor_pci_init() completes, force HW LO/HI to hvisor-allocated doorbell.
                // In non-delay mode, writes go through the normal MSI register handler below,
                // and first LO/HI writes are translated/synced there, so this extra sync is unnecessary.
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

            set_pci_init_done(domain_id);
            info!(
                "Hvisor PCI initialization complete for domain {}",
                domain_id
            );
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
            #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
            {
                // Non-delay path (or delay after init-done) uses this handler for MSI DBI regs.
                // LO/HI writes are virtualized and synchronized with hvisor-managed doorbell here.
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
static DBI_PCI_INIT_DONE: Lazy<Mutex<BTreeMap<u8, bool>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
pub fn is_pci_init_done(domain_id: u8) -> bool {
    DBI_PCI_INIT_DONE
        .lock()
        .get(&domain_id)
        .copied()
        .unwrap_or(false)
}

#[cfg(all(feature = "dwc_pcie", feature = "pci_init_delay"))]
fn set_pci_init_done(domain_id: u8) {
    DBI_PCI_INIT_DONE.lock().insert(domain_id, true);
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
    let (device_info, _domain_id) = {
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
            panic!(
                "MSIX table handler could not find device in current zone vPCI bus for BAR base {:#x}",
                base_aligned
            );
        }
        (result, domain_id)
    };

    // Check if this access is within the MSIX table range
    if let Some((dev, msix_offset, entry_count)) = device_info {
        // let vbdf = dev.get_vbdf();

        let msix_table_size = (entry_count as u64) * 16; // Each entry is 16 bytes
        let msix_table_end = msix_offset + msix_table_size;

        if access_offset >= msix_offset && access_offset < msix_table_end {
            // This is a MSIX table access, record it with detailed information
            let offset_in_entry = access_offset - msix_offset;
            // let entry_index = offset_in_entry / 16;
            let field_offset = offset_in_entry % 16;
            // let host_bdf = dev.get_bdf();
            // let field_name = match field_offset {
            //     0..=3 => "msg_addr_lo",
            //     4..=7 => "msg_addr_hi",
            //     8..=11 => "msg_data",
            //     12..=15 => "vector_ctrl",
            //     _ => "unknown",
            // };

            if mmio.is_write {
                // let vm_value = mmio.value;
                match field_offset {
                    0..=3 => {
                        // Save VM's doorbell low 32 bits
                        dev.with_msi_info_mut(|msi_info| {
                            let current = msi_info.msi_doorbell & 0xffffffff00000000;
                            msi_info.set_doorbell(current | (mmio.value as u64));
                        });

                        // Replace with hvisor's doorbell before writing to hardware
                        #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
                        {
                            if _domain_id != 0xFF {
                                let hw_paddr =
                                    crate::pci::dwc_msi::get_domain_doorbell_paddr(_domain_id);
                                let hw_doorbell_lo = (hw_paddr & 0xffffffff) as usize;
                                mmio.value = hw_doorbell_lo;
                            }
                        }
                    }
                    4..=7 => {
                        // Save VM's doorbell high 32 bits
                        dev.with_msi_info_mut(|msi_info| {
                            let current = msi_info.msi_doorbell & 0xffffffff;
                            msi_info.set_doorbell(current | ((mmio.value as u64) << 32));
                        });

                        // Replace with hvisor's doorbell before writing to hardware
                        #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
                        {
                            if _domain_id != 0xFF {
                                let hw_paddr =
                                    crate::pci::dwc_msi::get_domain_doorbell_paddr(_domain_id);
                                let hw_doorbell_hi = ((hw_paddr >> 32) & 0xffffffff) as usize;
                                mmio.value = hw_doorbell_hi;
                            }
                        }
                    }
                    8..=11 => {
                        // Convert VM vector index to hardware vector index.
                        #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
                        {
                            if _domain_id != 0xFF {
                                let zone = this_zone();
                                let guard = zone.read();
                                let vbus = guard.vpci_bus();
                                if let Some(domain_msi_info) =
                                    vbus.domain_msi_info().get(&_domain_id)
                                {
                                    let virq_bit = mmio.value as u32;
                                    let hwirq_bit = domain_msi_info.hwirq_bit;
                                    let hw_value = virq_bit.wrapping_add(hwirq_bit);
                                    mmio.value = hw_value as usize;
                                }
                            }
                        }
                    }
                    12..=15 => {}
                    _ => {}
                }

                mmio_perform_access(base, mmio);
                return Ok(());
            } else {
                let mut hw_mmio = MMIOAccess {
                    address: mmio.address,
                    value: 0,
                    size: mmio.size,
                    is_write: false,
                };
                mmio_perform_access(base, &mut hw_mmio);
                let hw_value = hw_mmio.value;

                match field_offset {
                    0..=3 => {
                        let dev_vm_doorbell = dev
                            .read()
                            .get_msi_info()
                            .map(|msi| msi.msi_doorbell)
                            .unwrap_or(0);
                        mmio.value = (dev_vm_doorbell & 0xffffffff) as usize;
                    }
                    4..=7 => {
                        let dev_vm_doorbell = dev
                            .read()
                            .get_msi_info()
                            .map(|msi| msi.msi_doorbell)
                            .unwrap_or(0);
                        mmio.value = ((dev_vm_doorbell >> 32) & 0xffffffff) as usize;
                    }
                    8..=11 => {
                        mmio.value = hw_value;
                        #[cfg(all(feature = "dwc_msi", feature = "dwc_pcie"))]
                        {
                            if _domain_id != 0xFF {
                                let zone = this_zone();
                                let guard = zone.read();
                                let vbus = guard.vpci_bus();
                                if let Some(domain_msi_info) =
                                    vbus.domain_msi_info().get(&_domain_id)
                                {
                                    let hwirq_bit = domain_msi_info.hwirq_bit;
                                    let hw_vec = hw_value as u32;
                                    let virq_bit = if hw_vec >= hwirq_bit {
                                        hw_vec - hwirq_bit
                                    } else {
                                        hw_vec
                                    };
                                    mmio.value = virq_bit as usize;
                                }
                            }
                        }
                    }
                    12..=15 => {
                        mmio.value = hw_value;
                    }
                    _ => {
                        mmio.value = hw_value;
                    }
                }
                return Ok(());
            }
        }
    }

    mmio_perform_access(base, mmio);

    Ok(())
}
