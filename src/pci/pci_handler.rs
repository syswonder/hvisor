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

use crate::cpu_data::this_zone;
use crate::error::HvResult;
use crate::memory::MMIOAccess;
use crate::memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion};
use crate::zone::is_this_root_zone;

use super::pci_access::{BridgeField, EndpointField, HeaderType, PciField, PciMemType};
use super::pci_config::GLOBAL_PCIE_LIST;
use super::pci_struct::{ArcRwLockVirtualPciConfigSpace, BIT_LENTH};
use super::vpci_dev::VpciDevType;
use super::PciConfigAddress;

#[cfg(target_arch = "x86_64")]
use crate::zone::this_zone_id;

#[cfg(feature = "dwc_pcie")]
use crate::{
    memory::mmio_perform_access,
    pci::config_accessors::{
        dwc::DwcConfigRegionBackend,
        dwc_atu::{
            AtuType, AtuUnroll, ATU_BASE, ATU_ENABLE_BIT, ATU_REGION_SIZE, PCIE_ATU_UNR_LIMIT,
            PCIE_ATU_UNR_LOWER_BASE, PCIE_ATU_UNR_LOWER_TARGET, PCIE_ATU_UNR_REGION_CTRL1,
            PCIE_ATU_UNR_REGION_CTRL2, PCIE_ATU_UNR_UPPER_BASE, PCIE_ATU_UNR_UPPER_LIMIT,
            PCIE_ATU_UNR_UPPER_TARGET,
        },
        PciRegionMmio,
    },
};

macro_rules! pci_log {
    ($($arg:tt)*) => {
        // info!($($arg)*);
        // To switch to debug level, change the line above to:
        debug!($($arg)*);
    };
}

fn handle_virt_pci_request(
    dev: ArcRwLockVirtualPciConfigSpace,
    offset: PciConfigAddress,
    size: usize,
    value: usize,
    is_write: bool,
    dev_type: VpciDevType,
) -> HvResult<Option<usize>> {
    pci_log!(
        "virt pci standard rw offset {:#x}, size {:#x}",
        offset,
        size
    );

    /*
     * The capability is located in the upper part of the configuration space,
     * and there is no other message. So the max cap_offset which is less than
     * offset is the correct cap we need.
     */
    let result = dev.with_cap(|capabilities| {
        if let Some((cap_offset, cap)) = capabilities.range(..=offset).next_back() {
            pci_log!(
                "find cap at offset {:#x}, cap {:#?}",
                cap_offset,
                cap.get_type()
            );
            let end = *cap_offset + cap.get_size() as u64;
            if offset >= end {
                return hv_result_err!(
                    ERANGE,
                    format!(
                        "virt pci cap rw offset {:#x} out of range [{:#x}..{:#x})",
                        offset, *cap_offset, end
                    )
                );
            }
            let relative_offset = offset - *cap_offset;

            if is_write {
                cap.with_region_mut(|region| {
                    match region.write(relative_offset, size, value as u32) {
                        Ok(()) => Ok(0),
                        Err(e) => {
                            warn!(
                                "Failed to write capability at offset 0x{:x}: {:?}",
                                offset, e
                            );
                            Err(e)
                        }
                    }
                })
            } else {
                cap.with_region(|region| match region.read(relative_offset, size) {
                    Ok(val) => Ok(val),
                    Err(e) => {
                        warn!(
                            "Failed to read capability at offset 0x{:x}: {:?}",
                            offset, e
                        );
                        Err(e)
                    }
                })
            }
        } else {
            hv_result_err!(ENOENT)
        }
    });

    match result {
        Ok(val) => {
            if !is_write {
                Ok(Some(val as usize))
            } else {
                Ok(None)
            }
        }
        Err(_) => {
            if is_write {
                super::vpci_dev::vpci_dev_write_cfg(dev_type, dev.clone(), offset, size, value)?;
                Ok(None)
            } else {
                Ok(Some(super::vpci_dev::vpci_dev_read_cfg(
                    dev_type,
                    dev.clone(),
                    offset,
                    size,
                )?))
            }
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

                                let paddr = if is_root {
                                    dev.with_bar_ref_mut(slot, |bar| bar.set_value(new_vaddr));
                                    if bar_type == PciMemType::Mem64High {
                                        dev.with_bar_ref_mut(slot - 1, |bar| {
                                            bar.set_value(new_vaddr)
                                        });
                                    }
                                    new_vaddr as HostPhysAddr
                                } else {
                                    dev.with_bar_ref(slot, |bar| bar.get_value64()) as HostPhysAddr
                                };
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
                                let gpm = &mut guard.gpm;

                                if !gpm
                                    .try_delete(old_vaddr.try_into().unwrap(), bar_size as usize)
                                    .is_ok()
                                {
                                    // warn!("delete bar {}: can not found 0x{:x}", slot, old_vaddr);
                                }
                                gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                    new_vaddr as GuestPhysAddr,
                                    paddr as HostPhysAddr,
                                    bar_size as _,
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

                        if value & 0xfffff800 != 0xfffff800 {
                            let old_vaddr =
                                dev.with_rom_ref(|rom| rom.get_virtual_value64()) & !0xf;
                            let new_vaddr = (value as u64) & !0xf;

                            dev.with_rom_ref_mut(|rom| rom.set_virtual_value(new_vaddr));

                            let paddr = if is_root {
                                dev.with_rom_ref_mut(|rom| rom.set_value(new_vaddr));
                                new_vaddr as HostPhysAddr
                            } else {
                                dev.with_rom_ref(|rom| rom.get_value64()) as HostPhysAddr
                            };

                            let rom_size = {
                                let size = dev.with_rom_ref(|rom| rom.get_size());
                                if crate::memory::addr::is_aligned(size as usize) {
                                    size
                                } else {
                                    crate::memory::PAGE_SIZE as u64
                                }
                            };
                            let new_vaddr = if !crate::memory::addr::is_aligned(new_vaddr as usize)
                            {
                                crate::memory::addr::align_up(new_vaddr as usize) as u64
                            } else {
                                new_vaddr as u64
                            };

                            let zone = this_zone();
                            let mut guard = zone.write();
                            let gpm = &mut guard.gpm;

                            if !gpm
                                .try_delete(old_vaddr.try_into().unwrap(), rom_size as usize)
                                .is_ok()
                            {
                                // warn!("delete rom bar: can not found 0x{:x}", old_vaddr);
                            }
                            gpm.try_insert_quiet(MemoryRegion::new_with_offset_mapper(
                                new_vaddr as GuestPhysAddr,
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
    _dev: ArcRwLockVirtualPciConfigSpace,
    _field: BridgeField,
    _is_write: bool,
) -> HvResult<Option<usize>> {
    Ok(None)
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
                            HeaderType::PciBridge => {
                                if let Some(val) = handle_pci_bridge_access(
                                    dev,
                                    BridgeField::from(offset as usize, size),
                                    is_write,
                                )? {
                                    mmio.value = val;
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
                            handle_virt_pci_request(dev, offset, size, value, is_write, dev_type)?
                        {
                            mmio.value = val;
                        }
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
        let mut guard = zone.write();
        let vbus = &mut guard.vpci_bus;
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
            .atu_configs
            .get_atu_by_io_base(_base as PciConfigAddress)
            .and_then(|atu| {
                guard
                    .atu_configs
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
        .atu_configs
        .get_atu_by_cfg_base(_base as PciConfigAddress)
        .and_then(|atu| {
            guard
                .atu_configs
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
            let vbus = &mut guard.vpci_bus;
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

    /* 0x0-0x100 is outbound atu0 reg
     * 0x100-0x200 is inbound atu0 reg just handle outbound right now
     * so MAX is ATU_BASE + ATU_REGION_SIZE/2
     */
    if mmio.address >= ATU_BASE && mmio.address < ATU_BASE + ATU_REGION_SIZE / 2 {
        let zone = this_zone();
        let mut guard = zone.write();
        let ecam_base = _base;
        let atu_offset = mmio.address - ATU_BASE;

        // warn!("set atu0 register {:#X} value {:#X}", atu_offset, mmio.value);

        let atu = guard.atu_configs.get_atu_by_ecam_mut(ecam_base).unwrap();

        // info!("atu config write {:#?}", atu);

        if mmio.is_write {
            if mmio.size == 4 {
                match atu_offset {
                    PCIE_ATU_UNR_REGION_CTRL1 => {
                        // info!("set atu0 region ctrl1 value {:#X}", mmio.value);
                        atu.set_atu_type(AtuType::from_u8((mmio.value & 0xff) as u8));
                    }
                    PCIE_ATU_UNR_REGION_CTRL2 => {
                        // Enable bit is written here, but we just track it
                        // The actual enable is handled by the driver
                    }
                    PCIE_ATU_UNR_LOWER_BASE => {
                        // info!("set atu0 lower base value {:#X}", mmio.value);
                        atu.set_cpu_base(
                            (atu.cpu_base() & !0xffffffff) | (mmio.value as PciConfigAddress),
                        );
                    }
                    PCIE_ATU_UNR_UPPER_BASE => {
                        // info!("set atu0 upper base value {:#X}", mmio.value);
                        atu.set_cpu_base(
                            (atu.cpu_base() & 0xffffffff)
                                | ((mmio.value as PciConfigAddress) << 32),
                        );
                    }
                    PCIE_ATU_UNR_LIMIT => {
                        // info!("set atu0 limit value {:#X}", mmio.value);
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
                        // info!("set atu0 lower target value {:#X}", mmio.value);
                        atu.set_pci_target(
                            (atu.pci_target() & !0xffffffff) | (mmio.value as PciConfigAddress),
                        );
                    }
                    PCIE_ATU_UNR_UPPER_TARGET => {
                        // info!("set atu0 upper target value {:#X}", mmio.value);
                        atu.set_pci_target(
                            (atu.pci_target() & 0xffffffff)
                                | ((mmio.value as PciConfigAddress) << 32),
                        );
                    }
                    _ => {
                        warn!("invalid atu0 write {:#x} + {:#x}", atu_offset, mmio.size);
                    }
                }
            } else {
                warn!("invalid atu0 read size {:#x}", mmio.size);
            }
        } else {
            // Read from virtual ATU
            // warn!("read atu0 {:#x}", atu_offset);
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
                    warn!("invalid atu0 read {:#x}", atu_offset);
                    mmio_perform_access(_base, mmio);
                }
            }
        }
    } else if mmio.address > ATU_BASE + ATU_REGION_SIZE / 2 {
        mmio_perform_access(_base, mmio);
    } else if mmio.address >= BIT_LENTH {
        // dbi read
        mmio_perform_access(_base, mmio);
    } else {
        warn!("mmio_vpci_handler_dbi read {:#x}", mmio.address);
        let offset = (mmio.address & 0xfff) as PciConfigAddress;
        let zone = this_zone();
        let mut is_dev_belong_to_zone = false;

        let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;

        let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
            let mut guard = zone.write();
            let vbus = &mut guard.vpci_bus;
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

        handle_config_space_access(dev, mmio, offset, is_direct, is_root, is_dev_belong_to_zone)?;
    }

    Ok(())
}

pub fn mmio_vpci_direct_handler(mmio: &mut MMIOAccess, _base: usize) -> HvResult {
    let zone = this_zone();
    let offset = (mmio.address & 0xfff) as PciConfigAddress;
    let base = mmio.address as PciConfigAddress - offset + _base as PciConfigAddress;
    let mut is_dev_belong_to_zone = false;

    let dev: Option<ArcRwLockVirtualPciConfigSpace> = {
        let mut guard = zone.write();
        let vbus = &mut guard.vpci_bus;
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
