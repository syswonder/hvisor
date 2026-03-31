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
//  ForeverYolo <2572131118@qq.com>
#![allow(unused_variables)]
#![allow(dead_code)]
use super::cache::invalidate_dcache_range;
use crate::arch::zone::{Uefi, UefiConfig};
use crate::config::MEM_TYPE_RAM;
use crate::config::{root_zone_config, HvZoneConfig};
use crate::consts::PAGE_SIZE;
use crate::platform::ROOT_ZONE_MEMORY_REGIONS;
use alloc::vec::Vec;
use core::cmp::min;
use core::mem::size_of;
use spin::Once;

const PTR_SIZE: usize = size_of::<usize>();

const EFI_RESERVED_TYPE: u32 = 0;
const EFI_LOADER_CODE: u32 = 1;
const EFI_LOADER_DATA: u32 = 2;
const EFI_BOOT_SERVICES_CODE: u32 = 3;
const EFI_BOOT_SERVICES_DATA: u32 = 4;
const EFI_RUNTIME_SERVICES_CODE: u32 = 5;
pub const EFI_RUNTIME_SERVICES_DATA: u32 = 6;
const EFI_CONVENTIONAL_MEMORY: u32 = 7;
const EFI_UNUSABLE_MEMORY: u32 = 8;
const EFI_ACPI_RECLAIM_MEMORY: u32 = 9;
const EFI_ACPI_MEMORY_NVS: u32 = 10;
const EFI_MEMORY_MAPPED_IO: u32 = 11;
const EFI_MEMORY_MAPPED_IO_PORT_SPACE: u32 = 12;
const EFI_PAL_CODE: u32 = 13;
const EFI_PERSISTENT_MEMORY: u32 = 14;
const EFI_MAX_MEMORY_TYPE: u32 = 15;
const EFI_PAGE_SIZE: u64 = 4096;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EfiMemoryAttributes: u64 {
        const UC         = 0x0000_0000_0000_0001; // Uncached
        const WC         = 0x0000_0000_0000_0002; // Write-coalescing
        const WT         = 0x0000_0000_0000_0004; // Write-through
        const WB         = 0x0000_0000_0000_0008; // Write-back
        const UCE        = 0x0000_0000_0000_0010; // Uncached, exported
        const WP         = 0x0000_0000_0000_1000; // Write-protect
        const RP         = 0x0000_0000_0000_2000; // Read-protect
        const XP         = 0x0000_0000_0000_4000; // Execute-protect
        const NV         = 0x0000_0000_0000_8000; // Non-volatile
        const MORE_RELIABLE = 0x0000_0000_0001_0000; // Higher reliability
        const RO         = 0x0000_0000_0002_0000; // Read-only
        const RUNTIME    = 0x8000_0000_0000_0000; // Range requires runtime mapping
    }
}

const EFI_MEMORY_DESCRIPTOR_VERSION: usize = 1;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct EfiMemoryDescriptor {
    pub memory_type: u32,
    pub pad: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct EfiBootMemMap {
    pub map_size: u64,
    pub desc_size: u64,
    pub desc_ver: u32,
    pub key: u64,
}

pub static UEFI_BOOT_CONFIG: Once<EfiBootMemMap> = Once::new();
pub static MEM_MAP: Once<Vec<EfiMemoryDescriptor>> = Once::new();

pub fn memory_map_init(config_addr: usize) {
    let zone_config = root_zone_config();
    match zone_config.arch_config.uefi_config {
        UefiConfig::NoUefi(_) => {
            panic!("UEFI support is enabled, but no UEFI parameters are passed")
        }
        UefiConfig::Uefi(ref uefi_config) => {
            info!("memory_map_init: config_addr = {:#x}", config_addr);
            UEFI_BOOT_CONFIG.call_once(|| EfiBootMemMap {
                map_size: unsafe { **((config_addr + PTR_SIZE) as *const *const u64) },
                desc_size: unsafe { **((config_addr + 2 * PTR_SIZE) as *const *const u64) },
                desc_ver: unsafe { **((config_addr + 3 * PTR_SIZE) as *const *const u32) },
                key: unsafe { **((config_addr + 4 * PTR_SIZE) as *const *const u64) },
            });
            info!("UEFI_BOOT_CONFIG: {:#x?}", UEFI_BOOT_CONFIG.get().unwrap());
            let mut vs = vec![];
            let mut desc_base = unsafe { **(config_addr as *const *const usize) };
            let bound = desc_base + UEFI_BOOT_CONFIG.get().unwrap().map_size as usize;
            while desc_base < bound {
                let desc = unsafe { &*(desc_base as *const EfiMemoryDescriptor) };
                trace!("desc: {:#x?}", desc);
                vs.push(EfiMemoryDescriptor {
                    memory_type: desc.memory_type,
                    pad: desc.pad,
                    physical_start: desc.physical_start,
                    virtual_start: desc.virtual_start,
                    number_of_pages: desc.number_of_pages,
                    attribute: desc.attribute,
                });
                desc_base += UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize;
                trace!("desc_base : {:#x}", desc_base);
            }
            // if ROOT_ZONE_UEFI_SYS_TABLE_ADDR != 0 {
            //     vs.push(EfiMemoryDescriptor{
            //         memory_type: EFI_RUNTIME_SERVICES_DATA,
            //         pad: 0,
            //         physical_start: ROOT_ZONE_UEFI_SYS_TABLE_ADDR,
            //         virtual_start: 0,
            //         number_of_pages: 1,
            //         attribute: (EfiMemoryAttributes::UC |
            //             EfiMemoryAttributes::WC |
            //             EfiMemoryAttributes::WT |
            //             EfiMemoryAttributes::WB |
            //             EfiMemoryAttributes::RUNTIME).bits()
            //     });
            // }
            vs.sort_by_key(|t| t.physical_start);
            MEM_MAP.call_once(|| vs);
            trace!("MEM_MAP: {:#x?}", MEM_MAP.get().unwrap());
            create_fake_memory_map(zone_config, uefi_config);
        }
    }
}

pub fn create_fake_memory_map(hv_zone_config: &HvZoneConfig, uefi_config: &Uefi) {
    // First, place the entries other than EFI_CONVENTIONAL_MEMORY.
    let mut fake_map_base = uefi_config.memory_map_addr as usize;
    let mut real_size = 0;
    let memory_attrs = (EfiMemoryAttributes::UC
        | EfiMemoryAttributes::WC
        | EfiMemoryAttributes::WT
        | EfiMemoryAttributes::WB)
        .bits();
    for desc in MEM_MAP.get().unwrap().iter() {
        if desc.attribute & EfiMemoryAttributes::RUNTIME.bits() != 0 {
            unsafe {
                let p = fake_map_base as *mut EfiMemoryDescriptor;
                *p = desc.clone();
                info!("fake desc: {:#x?}", *p);
            }
            fake_map_base += UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize;
            real_size += UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize;
            trace!("fake_map_base : {:#x}", fake_map_base);
        }
    }
    trace!("real_size : {:#x}", real_size);
    // Then, fill the gaps with EFI_CONVENTIONAL_MEMORY.
    for region in hv_zone_config.memory_regions().iter() {
        if region.mem_type == MEM_TYPE_RAM {
            let mut base_addr = region.physical_start;
            let bound_addr = region.physical_start + region.size;
            let (mut index, overlap_type) = get_first_overlap_desc(base_addr as usize);
            trace!(
                "region: {:#x?}, index: {}, overlap_type: {}",
                region,
                index,
                overlap_type
            );
            if overlap_type == NO_OVERLAP {
                fake_map_base = add_new_desc(
                    fake_map_base,
                    EFI_CONVENTIONAL_MEMORY,
                    region.physical_start,
                    0,
                    region.size,
                    memory_attrs,
                );
                real_size += UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize;
                trace!("fake_map_base : {:#x}", fake_map_base);
            } else {
                if overlap_type == LEFT_OVERLAP {
                    base_addr = MEM_MAP.get().unwrap()[index].physical_start
                        + (MEM_MAP.get().unwrap()[index].number_of_pages * EFI_PAGE_SIZE);
                    index += 1;
                }
                let map_size = MEM_MAP.get().unwrap().len();
                while index < map_size {
                    if MEM_MAP.get().unwrap()[index].attribute & EfiMemoryAttributes::RUNTIME.bits()
                        == 0
                    {
                        index += 1;
                        continue;
                    }
                    let desc_start = MEM_MAP.get().unwrap()[index].physical_start as usize;
                    let desc_end = desc_start
                        + (MEM_MAP.get().unwrap()[index].number_of_pages * EFI_PAGE_SIZE) as usize;
                    info!(
                        "desc_start: {:#x}, bound_addr: {:#x}, base_addr: {:#x}",
                        desc_start, bound_addr, base_addr
                    );
                    let new_memory_size = min(desc_start, bound_addr as usize) - base_addr as usize;
                    if new_memory_size % EFI_PAGE_SIZE as usize != 0 {
                        panic!("memory is not aligned to EFI_PAGE_SIZE");
                    }
                    if new_memory_size != 0 {
                        fake_map_base = add_new_desc(
                            fake_map_base,
                            EFI_CONVENTIONAL_MEMORY,
                            base_addr,
                            0,
                            new_memory_size as u64,
                            memory_attrs,
                        );
                        real_size += UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize;
                        trace!("fake_map_base : {:#x}", fake_map_base);
                    }
                    index += 1;
                    base_addr = desc_end as u64;
                    if base_addr >= bound_addr {
                        trace!(
                            "break at index: {}, base_addr: {:#x}, bound_addr: {:#x}",
                            index,
                            base_addr,
                            bound_addr
                        );
                        break;
                    }
                }
            }
            if base_addr < bound_addr {
                let new_memory_size = bound_addr as usize - base_addr as usize;
                if new_memory_size % EFI_PAGE_SIZE as usize != 0 {
                    panic!("memory is not aligned to EFI_PAGE_SIZE");
                }
                fake_map_base = add_new_desc(
                    fake_map_base,
                    EFI_CONVENTIONAL_MEMORY,
                    base_addr,
                    0,
                    new_memory_size as u64,
                    memory_attrs,
                );
                real_size += UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize;
                trace!("fake_map_base : {:#x}", fake_map_base);
            }
        }
    }
    info!("final real_size : {:#x}", real_size);
    let memory_map_size = uefi_config.memory_map_size as usize;
    if real_size < memory_map_size {
        // fill zero
        unsafe {
            let p = fake_map_base as *mut u8;
            core::ptr::write_bytes(p, 0, memory_map_size - real_size);
        }
    }
    // We need to write this table to memory, in order to let linux see.
    unsafe {
        invalidate_dcache_range(uefi_config.memory_map_addr as usize, real_size);
    }
    info!("Fake memory map write back done");
}

// case 1:
//                              |___________|
//  what we need to find ->  |________|   LEFT_OVERLAP

// case 2:
//                           |____________|
//  what we need to find ->    |________| RIGHT_OVERLAP
const LEFT_OVERLAP: usize = 0;
const RIGHT_OVERLAP: usize = 1;
const NO_OVERLAP: usize = 2;
pub fn get_first_overlap_desc(start_addr: usize) -> (usize, usize) {
    for (i, desc) in MEM_MAP.get().unwrap().iter().enumerate() {
        if desc.attribute & EfiMemoryAttributes::RUNTIME.bits() != 0 {
            let desc_start = desc.physical_start as usize;
            let desc_end = desc_start + (desc.number_of_pages * EFI_PAGE_SIZE) as usize;
            if desc_start > start_addr {
                return (i, RIGHT_OVERLAP);
            } else if desc_end > start_addr {
                return (i, LEFT_OVERLAP);
            }
        }
    }
    (usize::MAX, NO_OVERLAP)
}

pub fn add_new_desc(
    addr: usize,
    memory_type: u32,
    physical_start: u64,
    virtual_start: u64,
    size: u64,
    attribute: u64,
) -> usize {
    let desc = EfiMemoryDescriptor {
        memory_type,
        pad: 0,
        physical_start,
        virtual_start,
        number_of_pages: size / EFI_PAGE_SIZE,
        attribute,
    };
    unsafe {
        let p = addr as *mut EfiMemoryDescriptor;
        *p = desc;
        info!("new desc: {:#x?}", *p);
    };
    addr + UEFI_BOOT_CONFIG.get().unwrap().desc_size as usize
}
