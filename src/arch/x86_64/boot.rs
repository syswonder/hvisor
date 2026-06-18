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
use alloc::string::{String, ToString};
use bit_field::BitField;
use core::{
    arch::{self, global_asm},
    ffi::CStr,
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
        self.screen_info.orig_video_is_vga = VIDEO_TYPE_EFI;
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

const MAX_MODULES: usize = 16;

#[derive(Clone, Copy)]
struct BootModule {
    /// Address grub loaded the module at.
    src: usize,
    /// Size of the module in bytes.
    size: usize,
    /// Final address of the module. Equal to `src` when the module's command
    /// line is `0`, i.e. it is left where grub placed it.
    dst: usize,
    /// Whether the module still has to be copied to `dst`.
    pending: bool,
}

/// Whether two half-open byte ranges intersect.
fn ranges_overlap(base_a: usize, len_a: usize, base_b: usize, len_b: usize) -> bool {
    base_a < base_b + len_b && base_b < base_a + len_a
}

/// Relocate multiboot2 modules to the physical addresses encoded in their
/// command lines. A command line of `0` keeps the module where grub loaded it.
///
/// Copies are delayed while they would overwrite another pending module's
/// source range.
pub fn module_init(info_addr: usize) {
    let total_size = unsafe { *(info_addr as *const u32) } as usize;
    let info_end = info_addr
        .checked_add(total_size)
        .expect("multiboot info size overflows");

    let mut modules = [BootModule {
        src: 0,
        size: 0,
        dst: 0,
        pending: false,
    }; MAX_MODULES];
    let mut count = 0;

    let mut cur = info_addr + 8;
    // Every tag starts with a u32 type and a u32 size.
    while cur + 8 <= info_end {
        let tag_type = unsafe { *(cur as *const u32) };
        let tag_size = unsafe { *((cur + 4) as *const u32) } as usize;
        if tag_type == multiboot_tag::END {
            break;
        }
        // Tags must hold a header, fit in the info buffer, and advance the loop.
        if tag_size < 8 || cur.checked_add(tag_size).map_or(true, |end| end > info_end) {
            panic!("malformed multiboot tag");
        }
        if tag_type == multiboot_tag::MODULES {
            if count == MAX_MODULES {
                panic!("more than {} boot modules", MAX_MODULES);
            }
            // Validate the module header before dereferencing it.
            if tag_size < size_of::<Modules>() {
                panic!("malformed boot module tag");
            }
            let module = unsafe { &*(cur as *const Modules) };
            let src = module.mod_start as usize;
            // The multiboot2 module end address is exclusive.
            let size = (module.mod_end as usize)
                .checked_sub(src)
                .expect("boot module end precedes start");
            src.checked_add(size).expect("boot module source overflows");

            // Bound the command-line read to the module tag.
            let cmdline_len = tag_size
                .checked_sub(size_of::<Modules>())
                .expect("boot module tag too small");
            let cmdline_bytes = unsafe {
                core::slice::from_raw_parts((cur + size_of::<Modules>()) as *const u8, cmdline_len)
            };
            let cmdline = CStr::from_bytes_until_nul(cmdline_bytes)
                .ok()
                .and_then(|s| s.to_str().ok())
                .expect("boot module command line is not a valid string");
            let dst = match usize::from_str_radix(cmdline.trim(), 16) {
                Ok(0) => src,
                Ok(dst) => dst,
                Err(_) => panic!("boot module command line is not a hex address"),
            };
            dst.checked_add(size)
                .expect("boot module destination overflows");

            modules[count] = BootModule {
                src,
                size,
                dst,
                pending: dst != src,
            };
            count += 1;
        }
        cur += (tag_size + 7) & !7;
    }

    // Two modules must never share a final address range.
    for i in 0..count {
        for j in (i + 1)..count {
            if ranges_overlap(
                modules[i].dst,
                modules[i].size,
                modules[j].dst,
                modules[j].size,
            ) {
                panic!("boot module destinations overlap");
            }
        }
    }

    let mut remaining = (0..count).filter(|&i| modules[i].pending).count();
    while remaining > 0 {
        let mut progressed = false;
        for i in 0..count {
            if !modules[i].pending {
                continue;
            }
            // Do not overwrite the source of a module that still needs copying.
            let blocked = (0..count).any(|j| {
                j != i
                    && modules[j].pending
                    && ranges_overlap(
                        modules[i].dst,
                        modules[i].size,
                        modules[j].src,
                        modules[j].size,
                    )
            });
            if blocked {
                continue;
            }
            unsafe {
                copy(
                    modules[i].src as *const u8,
                    modules[i].dst as *mut u8,
                    modules[i].size,
                );
            }
            modules[i].pending = false;
            remaining -= 1;
            progressed = true;
        }
        if !progressed {
            // Cyclic moves need scratch space that is not available this early.
            panic!("boot module relocation has a cyclic dependency");
        }
    }

    info!("relocated {} boot module(s)", count);
}
