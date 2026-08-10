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
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{zone::HvArchZoneConfig, Stage2PageTable},
    config::{root_zone_config, HvPciConfig, HvZoneConfig, MEM_TYPE_RAM},
    cpu_data::this_zone,
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet, PAGE_SIZE},
    platform::MEM_TYPE_RESERVED,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use bit_field::BitField;
use core::{
    arch::{self, global_asm},
    ffi::{c_char, CStr},
    mem::size_of,
    ptr::{copy, copy_nonoverlapping},
};
use multiboot_tag::{Modules, MultibootTags};
use spin::{Mutex, Once};
use uefi_raw::table::{
    boot::{MemoryAttribute, MemoryDescriptor, MemoryType},
    configuration::ConfigurationTable,
    system::SystemTable,
    Header, Revision,
};
use uguid::{guid, Guid};

const ACPI_20_TABLE_GUID: Guid = guid!("8868E871-E4F1-11D3-BC22-0080C73C8881");

mod multiboot_tag {
    pub const END: u32 = 0;
    pub const MODULES: u32 = 3;
    pub const MEMORY_MAP: u32 = 6;
    pub const FRAMEBUFFER: u32 = 8;
    pub const ACPI_V1: u32 = 14;

    #[repr(C)]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Modules {
        tag_type: u32,
        pub size: u32,
        pub mod_start: u32,
        pub mod_end: u32,
    }

    #[repr(C)]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct MemoryMap {
        tag_type: u32,
        pub size: u32,
        pub entry_size: u32,
        pub entry_version: u32,
    }

    #[repr(C)]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct MemoryMapEntry {
        pub base_addr: u64,
        pub length: u64,
        pub _type: u32,
        reserved: u32,
    }

    #[repr(C)]
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Framebuffer {
        tag_type: u32,
        size: u32,
        pub addr: u64,
        pub pitch: u32,
        pub width: u32,
        pub height: u32,
        pub bpp: u8,
        pub fb_type: u8,
        reserved: u8,
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub struct MultibootTags {
        pub framebuffer: Framebuffer,
        pub memory_map_addr: Option<usize>,
        pub rsdp_addr: Option<usize>,
    }
}

static MULTIBOOT_TAGS: Once<MultibootTags> = Once::new();

const E820_MAX_ENTRIES_ZEROPAGE: usize = 128;

const EFI64_LOADER_SIGNATURE: u32 = 0x34364c45; // EL64

const VIDEO_TYPE_VLFB: u8 = 0x23;
const VIDEO_TYPE_EFI: u8 = 0x70;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    /// https://www.kernel.org/doc/html/latest/arch/x86/boot.html
    pub struct BootLoadFlags: u8 {
        const LOADED_HIGH = 1;
        const KASLR_FLAG = 1 << 1;
        const QUIET_FLAG = 1 << 5;
        const CAN_USE_HEAP = 1 << 7;
    }
}

numeric_enum_macro::numeric_enum! {
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum E820Type {
    E820_DEFAULT = 0,
    E820_RAM = 1,
    E820_RESERVED = 2,
    E820_ACPI = 3,
    E820_NVS = 4,
    E820_UNUSABLE = 5,
    E820_PMEM = 7,
}
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// The so-called "zeropage"
pub struct BootParams {
    screen_info: ScreenInfo,
    pad9: [u8; 0x180],
    efi_info: EfiInfo,
    pad0: [u8; 0x8],
    e820_entries: u8,
    pad1: [u8; 0x8],
    setup_sects: u8,
    root_flags: u16,
    syssize: u32,
    ramsize: u16,
    vid_mode: u16,
    pad2: [u8; 0x9],
    boot_proto_version: u16,
    pad3: [u8; 0x6],
    kernel_version: u16,
    type_of_loader: u8,
    loadflags: BootLoadFlags,
    setup_move_size: u16,
    code32_start: u32,
    ramdisk_image: u32,
    ramdisk_size: u32,
    bootsect_kludge: u32,
    heap_end_ptr: u16,
    pad4: [u8; 2],
    cmd_line_ptr: u32,
    pad5: [u8; 12],
    cmdline_size: u32,
    pad6: [u8; 12],
    payload_offset: u32,
    payload_length: u32,
    pad7: [u8; 128],
    e820_table: [BootE820Entry; E820_MAX_ENTRIES_ZEROPAGE],
    pad8: [u8; 0x330],
}

impl BootParams {
    pub fn fill(config: &HvZoneConfig, gpm: &mut MemorySet<Stage2PageTable>) -> HvResult {
        if config.arch_config.setup_load_gpa == 0 {
            panic!("setup addr not set yet!");
        }

        let boot_params_hpa = unsafe {
            gpm.page_table_query(config.arch_config.setup_load_gpa)
                .unwrap()
                .0
        } as HostPhysAddr;
        let boot_params = unsafe { &mut *(boot_params_hpa as *mut BootParams) };

        info!("boot_proto_version: {:x?}", boot_params.boot_proto_version);
        if boot_params.boot_proto_version < 0x0204 {
            panic!("kernel boot protocol version older than 2.04 not supported!");
        }

        // set bootloader type as undefined
        boot_params.type_of_loader = 0xff;

        let mut loadflags = boot_params.loadflags;
        // print early messages
        loadflags &= !BootLoadFlags::QUIET_FLAG;
        // disable heap_ptr
        loadflags &= !BootLoadFlags::CAN_USE_HEAP;
        boot_params.loadflags = loadflags;

        boot_params.cmd_line_ptr = config.arch_config.cmdline_load_gpa as _;
        // copy cmdline manually for root zone
        if config.zone_id == 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    crate::platform::ROOT_ZONE_CMDLINE.as_ptr(),
                    gpm.page_table_query(config.arch_config.cmdline_load_gpa)
                        .unwrap()
                        .0 as *mut u8,
                    crate::platform::ROOT_ZONE_CMDLINE.len(),
                )
            };
        }

        // set e820
        boot_params.set_e820_entries(config);

        // set initrd
        if config.arch_config.initrd_load_gpa != 0 {
            boot_params.set_initrd(
                config.arch_config.initrd_load_gpa as _,
                config.arch_config.initrd_size as _,
            );
        }

        // set screen
        if config.arch_config.screen_base != 0 {
            boot_params.set_screen_info(config, gpm);
        }

        // set efi_info
        // if (config.zone_id == 0) {
        boot_params.set_uefi_info(config);
        // }

        Ok(())
    }

    fn set_e820_entries(&mut self, config: &HvZoneConfig) {
        let mut index = 0;
        for i in 0..config.memory_regions().len() {
            let mem_region = &config.memory_regions()[i];
            let mut e820_type = E820Type::E820_DEFAULT;

            if i == config.arch_config.rsdp_memory_region_id
                || i == config.arch_config.acpi_memory_region_id
            {
                e820_type = E820Type::E820_ACPI;
            } else if i == config.arch_config.uefi_memory_region_id {
                e820_type = E820Type::E820_RESERVED;
            } else if mem_region.mem_type == MEM_TYPE_RAM {
                e820_type = E820Type::E820_RAM;
            }

            if e820_type != E820Type::E820_DEFAULT {
                self.e820_table[index] = BootE820Entry {
                    addr: mem_region.virtual_start,
                    size: mem_region.size,
                    _type: e820_type,
                };
                index += 1;
            }
        }

        self.e820_table[index] = BootE820Entry {
            addr: config.pci_config[0].ecam_base as _,
            size: config.pci_config[0].ecam_size as _,
            _type: E820Type::E820_RESERVED,
        };
        index += 1;

        self.e820_entries = index as _;
    }

    fn set_efi_mem_map(&mut self, config: &HvZoneConfig, paddr: usize) -> usize {
        let mut cnt = 0;
        let mem_map_cnt = config.memory_regions().len();
        let mut mem_map = unsafe { paddr as *mut MemoryDescriptor };
        for i in 0..mem_map_cnt {
            let mem_region = &config.memory_regions()[i];
            let mem_desc = unsafe { &mut *mem_map };

            let mut mem_desc_type = MemoryType::RESERVED;

            if i == config.arch_config.rsdp_memory_region_id
                || i == config.arch_config.acpi_memory_region_id
            {
                mem_desc_type = MemoryType::ACPI_RECLAIM;
            } else if i == config.arch_config.uefi_memory_region_id {
                mem_desc_type = MemoryType::RUNTIME_SERVICES_DATA;
            } else if mem_region.mem_type == MEM_TYPE_RAM {
                mem_desc_type = MemoryType::CONVENTIONAL;
            }

            if mem_desc_type != MemoryType::RESERVED {
                *mem_desc = MemoryDescriptor {
                    ty: mem_desc_type,
                    phys_start: mem_region.virtual_start,
                    virt_start: mem_region.virtual_start,
                    page_count: mem_region.size / (PAGE_SIZE as u64),
                    att: MemoryAttribute::WRITE_BACK,
                };
                cnt += 1;
                mem_map = mem_map.wrapping_add(1);
            }
        }

        let mem_desc = unsafe { &mut *mem_map };
        *mem_desc = MemoryDescriptor {
            ty: MemoryType::MMIO,
            phys_start: config.pci_config[0].ecam_base,
            virt_start: config.pci_config[0].ecam_base,
            page_count: config.pci_config[0].ecam_size / (PAGE_SIZE as u64),
            att: MemoryAttribute::UNCACHEABLE,
        };
        cnt += 1;

        cnt
    }

    fn set_initrd(&mut self, ramdisk_image: u32, ramdisk_size: u32) {
        self.ramdisk_image = ramdisk_image;
        self.ramdisk_size = ramdisk_size;
        info!("initrd size: {}", self.ramdisk_size);
    }

    fn set_uefi_info(&mut self, config: &HvZoneConfig) {
        self.efi_info.loader_signature = EFI64_LOADER_SIGNATURE;

        let uefi_region = &config.memory_regions()[config.arch_config.uefi_memory_region_id];
        let mut vaddr = uefi_region.virtual_start as usize;
        let mut paddr = uefi_region.physical_start as usize;

        // set system table
        self.efi_info.systab = vaddr.get_bits(0..32) as _;
        self.efi_info.systab_hi = vaddr.get_bits(32..64) as _;
        let system_table = unsafe { &mut *(paddr as usize as *mut SystemTable) };

        let system_table_header = Header {
            signature: SystemTable::SIGNATURE,
            revision: Revision::EFI_2_90,
            size: size_of::<SystemTable>() as u32,
            crc: 0,
            reserved: 0,
        };

        // start of the efi memmap
        vaddr += size_of::<SystemTable>();
        paddr += size_of::<SystemTable>();

        let mem_desc_cnt = self.set_efi_mem_map(config, paddr);
        let mem_map_tot_size = size_of::<MemoryDescriptor>() * mem_desc_cnt;
        self.efi_info.memmap = vaddr.get_bits(0..32) as _;
        self.efi_info.memmap_hi = vaddr.get_bits(32..64) as _;
        self.efi_info.memdesc_size = size_of::<MemoryDescriptor>() as _;
        self.efi_info.memmap_size = mem_map_tot_size as _;
        self.efi_info.memdesc_version = MemoryDescriptor::VERSION;

        // start of the config table
        vaddr += size_of::<MemoryDescriptor>() * mem_desc_cnt;
        paddr += size_of::<MemoryDescriptor>() * mem_desc_cnt;

        const CONFIG_TABLE_ENTRIES: usize = 1;
        let config_table =
            unsafe { &mut *(paddr as *mut [ConfigurationTable; CONFIG_TABLE_ENTRIES]) };

        // ACPI_20_TABLE_GUID
        let rsdp_region = &config.memory_regions()[config.arch_config.rsdp_memory_region_id];
        config_table[0].vendor_guid = ACPI_20_TABLE_GUID;
        config_table[0].vendor_table = unsafe { rsdp_region.virtual_start as _ };

        *system_table = SystemTable {
            header: system_table_header,
            firmware_vendor: core::ptr::null_mut(),
            firmware_revision: 0,
            stdin_handle: core::ptr::null_mut(),
            stdin: core::ptr::null_mut(),
            stdout_handle: core::ptr::null_mut(),
            stdout: core::ptr::null_mut(),
            stderr_handle: core::ptr::null_mut(),
            stderr: core::ptr::null_mut(),
            runtime_services: core::ptr::null_mut(),
            boot_services: core::ptr::null_mut(),
            number_of_configuration_table_entries: CONFIG_TABLE_ENTRIES,
            configuration_table: unsafe { vaddr as *mut ConfigurationTable },
        };
    }

    fn set_screen_info(&mut self, config: &HvZoneConfig, gpm: &mut MemorySet<Stage2PageTable>) {
        let fb_info = &get_multiboot_tags().framebuffer;

        let bytes_per_pixel = (fb_info.bpp as usize) / 8;
        let width = fb_info.width as usize;

        #[cfg(not(split_screen))]
        let height = fb_info.height as usize;
        #[cfg(all(split_screen))]
        let height = (fb_info.height / 2) as usize;

        self.screen_info.lfb_base = config.arch_config.screen_base as _;
        self.screen_info.lfb_width = width as _;
        self.screen_info.lfb_height = height as _;
        self.screen_info.lfb_depth = fb_info.bpp as _;
        self.screen_info.lfb_size = (bytes_per_pixel * width * height) as _;
        self.screen_info.lfb_linelength = (bytes_per_pixel * width) as _;

        // TODO: custom
        self.screen_info.blue_size = 8;
        self.screen_info.blue_pos = 0;
        self.screen_info.green_size = 8;
        self.screen_info.green_pos = 8;
        self.screen_info.red_size = 8;
        self.screen_info.red_pos = 16;
        self.screen_info.alpha_size = 8;
        self.screen_info.alpha_pos = 24;
        self.screen_info.orig_video_is_vga = VIDEO_TYPE_VLFB;
        self.screen_info.capabilities = 0;
        self.vid_mode = 0xffff;

        gpm.insert(MemoryRegion::new_with_offset_mapper(
            config.arch_config.screen_base as GuestPhysAddr,
            fb_info.addr as HostPhysAddr,
            self.screen_info.lfb_size as _,
            MemFlags::READ | MemFlags::WRITE,
        ));
    }
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
/// The E820 memory region entry of the boot protocol ABI:
pub struct BootE820Entry {
    addr: u64,
    size: u64,
    _type: E820Type,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct ScreenInfo {
    pad0: [u8; 0x0f],
    orig_video_is_vga: u8,
    pad1: u16,
    lfb_width: u16,
    lfb_height: u16,
    lfb_depth: u16,
    lfb_base: u32,
    lfb_size: u32,
    pad2: [u16; 2],
    lfb_linelength: u16,
    red_size: u8,
    red_pos: u8,
    green_size: u8,
    green_pos: u8,
    blue_size: u8,
    blue_pos: u8,
    alpha_size: u8,
    alpha_pos: u8,
    pad3: [u8; 4],
    pages: u16,
    vesa_attributes: u16,
    capabilities: u32,
    pad4: [u8; 6],
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct EfiInfo {
    loader_signature: u32,
    systab: u32,
    memdesc_size: u32,
    memdesc_version: u32,
    memmap: u32,
    memmap_size: u32,
    systab_hi: u32,
    memmap_hi: u32,
}

pub fn multiboot_init(info_addr: usize) {
    let mut cur = info_addr;
    let total_size = unsafe { *(cur as *const u32) } as usize;
    let mut multiboot_tags = MultibootTags::default();

    // println!("{:#x?}", total_size);
    cur += 8;
    while cur < info_addr + total_size {
        let tag_type = unsafe { *(cur as *const u32) };
        if tag_type == multiboot_tag::END {
            break;
        }

        // println!("{:#x?}", tag_type);
        match tag_type {
            multiboot_tag::MODULES => {}
            multiboot_tag::MEMORY_MAP => {
                multiboot_tags.memory_map_addr = Some(cur);
            }
            multiboot_tag::FRAMEBUFFER => {
                multiboot_tags.framebuffer =
                    unsafe { *(cur as *const multiboot_tag::Framebuffer) }.clone();
            }
            multiboot_tag::ACPI_V1 => {
                multiboot_tags.rsdp_addr = Some(cur + 8);
            }
            _ => {}
        }
        cur += ((unsafe { *((cur + 4) as *const u32) } as usize + 7) & (!7));
    }

    MULTIBOOT_TAGS.call_once(|| multiboot_tags);
}

pub fn get_multiboot_tags() -> &'static multiboot_tag::MultibootTags {
    MULTIBOOT_TAGS.get().unwrap()
}

pub fn print_memory_map() {
    let map_addr = get_multiboot_tags().memory_map_addr.unwrap();
    let mem_map = unsafe { *(map_addr as *const multiboot_tag::MemoryMap) };
    let mem_map_size = size_of::<multiboot_tag::MemoryMap>();
    let cnt = ((mem_map.size as usize) - mem_map_size) / (mem_map.entry_size as usize);

    let mut entry_addr = map_addr + mem_map_size;
    println!("---------- MEMORY MAP ----------");
    for i in 0..cnt {
        let entry = unsafe { *(entry_addr as *const multiboot_tag::MemoryMapEntry) };
        println!(
            "base: {:x}, len: {:x}, type: {:x}",
            entry.base_addr, entry.length, entry._type
        );
        entry_addr += size_of::<multiboot_tag::MemoryMapEntry>();
    }
}

/// Construct Multiboot2 info structure in guest memory at `multiboot_info_paddr`.
pub fn multiboot2_info_fill(
    config: &HvZoneConfig,
    gpm: &mut MemorySet<Stage2PageTable>,
    multiboot_info_paddr: GuestPhysAddr,
) -> HvResult {
    let info_gpa = multiboot_info_paddr;
    let (info_hpa, _, _) = unsafe {
        gpm.page_table_query(info_gpa)
            .expect("multiboot2 info GPA unmapped")
    };

    // Read raw cmdline from guest memory FIRST, before writing tags
    // (cmdline is at the same GPA as multiboot info, so writing tags
    // would overwrite it)
    let mut cmdline_buf = [0u8; 256];
    let cmdline_len = {
        let cmdline_gpa = config.arch_config.cmdline_load_gpa as GuestPhysAddr;
        if cmdline_gpa != 0 {
            if let Ok((cmdline_hpa, _, _)) = unsafe { gpm.page_table_query(cmdline_gpa) } {
                let src = cmdline_hpa as *const u8;
                let mut len = 0usize;
                while len < 255 && unsafe { *src.add(len) } != 0 {
                    cmdline_buf[len] = unsafe { *src.add(len) };
                    len += 1;
                }
                len
            } else {
                0
            }
        } else {
            0
        }
    };

    let info_ptr = info_hpa as *mut u8;

    const TAG_END: u32 = 0;
    const TAG_CMDLINE: u32 = 1;
    const TAG_BASIC_MEMINFO: u32 = 4;
    const TAG_MMAP: u32 = 6;
    const MB_MEM_AVAILABLE: u32 = 1;
    const LOW_MEM_TOP: u64 = 0xC0000000;
    const LOW_MMIO_TOP: u64 = 0x100000000;

    let mut offset = 8usize;

    // Tag 1: Basic memory info
    let total_ram: u64 = config
        .memory_regions()
        .iter()
        .filter(|r| r.mem_type == MEM_TYPE_RAM)
        .map(|r| r.size)
        .sum();
    let mem_upper_kb = if total_ram > 0x100000 {
        ((total_ram - 0x100000) / 1024) as u32
    } else {
        0
    };
    unsafe {
        let t = info_ptr.add(offset);
        *(t as *mut u32) = TAG_BASIC_MEMINFO;
        *(t.add(4) as *mut u32) = 16;
        *(t.add(8) as *mut u32) = 640;
        *(t.add(12) as *mut u32) = mem_upper_kb;
    }
    offset += 16;

    // Tag 2: Memory map. Available RAM is carved around reserved regions so
    // Asterinas cannot reclaim the bootloader, kernel, cmdline, info buffer,
    // or initramfs pages.
    const BOOTLOADER_RESERVED_END: u64 = 0x10000;

    let entry_size = 24usize;
    let mmap_hdr = offset;
    offset += 16;
    let mut entry_count = 0u32;

    let mut reserved: Vec<(u64, u64)> = Vec::new();
    if config.kernel_size > 0 {
        let kernel_start = config.kernel_load_paddr;
        reserved.push((kernel_start, kernel_start + config.kernel_size));
    }
    reserved.push((info_gpa as u64, info_gpa as u64 + PAGE_SIZE as u64));
    if config.arch_config.cmdline_load_gpa != 0
        && config.arch_config.cmdline_load_gpa as u64 != info_gpa as u64
    {
        let cmdline_gpa = config.arch_config.cmdline_load_gpa as u64;
        reserved.push((cmdline_gpa, cmdline_gpa + PAGE_SIZE as u64));
    }
    let initrd_gpa = config.arch_config.initrd_load_gpa as u64;
    let initrd_size = config.arch_config.initrd_size as u64;
    if initrd_size > 0 {
        reserved.push((initrd_gpa, initrd_gpa + initrd_size));
    }
    reserved.push((0x8000, BOOTLOADER_RESERVED_END));
    reserved.sort_unstable();

    let mut available: Vec<(u64, u64)> = Vec::new();
    for r in config
        .memory_regions()
        .iter()
        .filter(|r| r.mem_type == MEM_TYPE_RAM)
    {
        let start = r.virtual_start;
        let end = start + r.size;
        let mut cursor = start;
        for &(reserved_start, reserved_end) in &reserved {
            if reserved_end <= cursor || reserved_start >= end {
                continue;
            }
            let s = reserved_start.max(cursor);
            if s > cursor {
                available.push((cursor, s.min(end)));
            }
            cursor = reserved_end.max(cursor);
            if cursor >= end {
                break;
            }
        }
        if cursor < end {
            available.push((cursor, end));
        }
    }

    for (start, end) in available {
        if start < LOW_MEM_TOP {
            let s = start;
            let e = end.min(LOW_MEM_TOP);
            if e > s {
                unsafe {
                    let p = info_ptr.add(offset);
                    *(p as *mut u64) = s;
                    *(p.add(8) as *mut u64) = e - s;
                    *(p.add(16) as *mut u32) = MB_MEM_AVAILABLE;
                    *(p.add(20) as *mut u32) = 0;
                }
                offset += entry_size;
                entry_count += 1;
            }
        }
        if end > LOW_MMIO_TOP {
            let s = start.max(LOW_MMIO_TOP);
            if end > s {
                unsafe {
                    let p = info_ptr.add(offset);
                    *(p as *mut u64) = s;
                    *(p.add(8) as *mut u64) = end - s;
                    *(p.add(16) as *mut u32) = MB_MEM_AVAILABLE;
                    *(p.add(20) as *mut u32) = 0;
                }
                offset += entry_size;
                entry_count += 1;
            }
        }
    }

    let mmap_tag_size = (16 + entry_count as usize * entry_size) as u32;
    unsafe {
        *(info_ptr.add(mmap_hdr) as *mut u32) = TAG_MMAP;
        *(info_ptr.add(mmap_hdr + 4) as *mut u32) = mmap_tag_size;
        *(info_ptr.add(mmap_hdr + 8) as *mut u32) = entry_size as u32;
        *(info_ptr.add(mmap_hdr + 12) as *mut u32) = 0;
    }
    offset = (offset + 7) & !7;

    // Tag 3: Command line (from pre-read buffer)
    if cmdline_len > 0 {
        let tag_size = ((8 + cmdline_len + 1 + 7) & !7) as u32;
        unsafe {
            *(info_ptr.add(offset) as *mut u32) = TAG_CMDLINE;
            *(info_ptr.add(offset + 4) as *mut u32) = tag_size;
            for i in 0..cmdline_len {
                *info_ptr.add(offset + 8 + i) = cmdline_buf[i];
            }
            *info_ptr.add(offset + 8 + cmdline_len) = 0;
        }
        offset += tag_size as usize;
    }

    // Tag 4: Module (initramfs)
    let initrd_gpa = config.arch_config.initrd_load_gpa as u64;
    let initrd_size = config.arch_config.initrd_size as u64;
    if initrd_size > 0 {
        const TAG_MODULE: u32 = 3;
        let mod_cmd = b"./initramfs.cpio.gz\0";
        let mod_cmd_len = mod_cmd.len();
        let tag_size = ((8 + 8 + mod_cmd_len + 7) & !7) as u32;
        unsafe {
            *(info_ptr.add(offset) as *mut u32) = TAG_MODULE;
            *(info_ptr.add(offset + 4) as *mut u32) = tag_size;
            *(info_ptr.add(offset + 8) as *mut u32) = initrd_gpa as u32;
            *(info_ptr.add(offset + 12) as *mut u32) = (initrd_gpa + initrd_size) as u32;
            for i in 0..mod_cmd_len {
                *info_ptr.add(offset + 16 + i) = mod_cmd[i];
            }
        }
        offset += tag_size as usize;
    }

    // End tag
    unsafe {
        *(info_ptr.add(offset) as *mut u32) = TAG_END;
        *(info_ptr.add(offset + 4) as *mut u32) = 8;
    }
    offset += 8;

    // Header
    unsafe {
        *(info_ptr as *mut u32) = offset as u32;
        *(info_ptr.add(4) as *mut u32) = 0;
    }

    info!(
        "Multiboot2 info: {} bytes, {} mmap entries, RAM {:#x}",
        offset, entry_count, total_ram
    );
    Ok(())
}

/// copy kernel modules to the right place
pub fn module_init(info_addr: usize) {
    println!("module_init");

    const MAX_MODULES: usize = 16;

    #[derive(Clone, Copy)]
    struct ModuleInfo {
        start: usize,
        end: usize,
        dst: usize,
        string_ptr: usize,
    }

    let mut modules = [ModuleInfo {
        start: 0,
        end: 0,
        dst: 0,
        string_ptr: 0,
    }; MAX_MODULES];
    let mut module_count = 0;

    let mut cur = info_addr;
    let total_size = unsafe { *(cur as *const u32) } as usize;
    cur += 8;
    while cur < info_addr + total_size && module_count < MAX_MODULES {
        let tag_type = unsafe { *(cur as *const u32) };
        if tag_type == multiboot_tag::END {
            break;
        }
        if tag_type == multiboot_tag::MODULES {
            let ptr = cur as *const multiboot_tag::Modules;
            let module = unsafe { *ptr };
            let string_ptr = (ptr as usize) + size_of::<Modules>();
            modules[module_count] = ModuleInfo {
                start: module.mod_start as usize,
                end: module.mod_end as usize,
                dst: 0, // parse later
                string_ptr,
            };
            module_count += 1;
        }
        cur += ((unsafe { *((cur + 4) as *const u32) } as usize + 7) & (!7));
    }

    // parse dst
    for i in 0..module_count {
        let cstr = unsafe { CStr::from_ptr(modules[i].string_ptr as *const c_char) };
        modules[i].dst = usize::from_str_radix(cstr.to_str().unwrap(), 16).unwrap();
        println!(
            "module: start={:#x}, end={:#x}, dst={:#x}",
            modules[i].start, modules[i].end, modules[i].dst
        );
    }

    // now move in order
    let mut moved = [false; MAX_MODULES];
    let mut moved_count = 0;
    while moved_count < module_count {
        let mut found = false;
        for i in 0..module_count {
            if moved[i] {
                continue;
            }
            let dst = modules[i].dst;
            let dst_end = dst + (modules[i].end - modules[i].start);
            let mut can_move = true;
            for j in 0..module_count {
                if moved[j] || i == j {
                    continue;
                }
                let start = modules[j].start;
                let end = modules[j].end;
                if dst < end && dst_end > start {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                if modules[i].dst != 0 {
                    let size = modules[i].end - modules[i].start + 1;
                    let dst_end = modules[i].dst + size;
                    let overlaps_self =
                        modules[i].dst < modules[i].end && dst_end > modules[i].start;
                    unsafe {
                        if overlaps_self {
                            core::ptr::copy(
                                modules[i].start as *const u8,
                                modules[i].dst as *mut u8,
                                size,
                            );
                        } else {
                            core::ptr::copy_nonoverlapping(
                                modules[i].start as *const u8,
                                modules[i].dst as *mut u8,
                                size,
                            );
                        }
                    }
                }
                moved[i] = true;
                moved_count += 1;
                found = true;
                break;
            }
        }
        if !found {
            panic!("Cannot move modules due to overlapping addresses");
        }
    }

    println!("module cnt: {:x}", module_count);
}
