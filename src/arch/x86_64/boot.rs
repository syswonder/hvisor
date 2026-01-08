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
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, MemorySet},
    percpu::this_zone,
    platform::MEM_TYPE_RESERVED,
};
use alloc::string::{String, ToString};
use core::{
    arch::{self, global_asm},
    ffi::{c_char, CStr},
    mem::size_of,
    ptr::{copy, copy_nonoverlapping},
};
use multiboot_tag::{Modules, MultibootTags};
use spin::{Mutex, Once};

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
    pad0: [u8; 0x1a8],
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

        // info!("boot_proto_version: {:x?}", boot_params.boot_proto_version);
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

        Ok(())
    }

    fn set_e820_entries(&mut self, config: &HvZoneConfig) {
        let mut index = 0;
        for i in 0..config.memory_regions().len() {
            let mem_region = config.memory_regions()[i];
            let mut e820_type = E820Type::E820_DEFAULT;

            if i == config.arch_config.rsdp_memory_region_id
                || i == config.arch_config.acpi_memory_region_id
            {
                e820_type = E820Type::E820_ACPI;
            } else if mem_region.mem_type == MEM_TYPE_RAM {
                e820_type = E820Type::E820_RAM;
            } /*
              else if config.arch_config.initrd_load_gpa != 0
                    && i == config.arch_config.initrd_memory_region_id
              {
              }  */

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

    fn set_initrd(&mut self, ramdisk_image: u32, ramdisk_size: u32) {
        self.ramdisk_image = ramdisk_image;
        self.ramdisk_size = ramdisk_size;
        info!("initrd size: {}", self.ramdisk_size);
    }

    fn set_screen_info(&mut self, config: &HvZoneConfig, gpm: &mut MemorySet<Stage2PageTable>) {
        let fb_info = &get_multiboot_tags().framebuffer;

        let bytes_per_pixel = (fb_info.bpp as usize) / 8;
        let width = fb_info.width as usize;
        let height = fb_info.height as usize;

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
        self.screen_info.orig_video_is_vga = 0x23; // VESA
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

/// copy kernel modules to the right place
pub fn module_init(info_addr: usize) {
    println!("module_init");
    let mut cur = info_addr;
    let total_size = unsafe { *(cur as *const u32) } as usize;

    let mut cnt = 0;
    cur += 8;
    while cur < info_addr + total_size {
        let tag_type = unsafe { *(cur as *const u32) };
        let ptr = cur as *const multiboot_tag::Modules;
        cur += ((unsafe { *((cur + 4) as *const u32) } as usize + 7) & (!7));

        if tag_type == multiboot_tag::END {
            break;
        }
        if tag_type != multiboot_tag::MODULES {
            continue;
        }

        let module = unsafe { *ptr };
        let dst = unsafe {
            usize::from_str_radix(
                CStr::from_ptr(((ptr as usize) + size_of::<Modules>()) as *const c_char)
                    .to_str()
                    .unwrap(),
                16,
            )
            .unwrap()
        };
        println!("module: {:#x?}, addr: {:#x?}", module, dst);
        cnt += 1;

        if dst == 0x0 {
            continue;
        }

        unsafe {
            core::ptr::copy(
                module.mod_start as *mut u8,
                dst as *mut u8,
                (module.mod_end - module.mod_start + 1) as usize,
            )
        };
    }
    println!("module cnt: {:x}", cnt);
}
