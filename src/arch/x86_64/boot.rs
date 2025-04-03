use crate::{
    config::{root_zone_config, HvZoneConfig, MEM_TYPE_RAM},
    error::HvResult,
    memory::{GuestPhysAddr, HostPhysAddr},
    platform::qemu_x86_64::root_zone_gpa_as_mut_ptr,
};
use alloc::string::{String, ToString};
use core::{
    ffi::{c_char, CStr},
    ptr::copy_nonoverlapping,
};
use spin::Mutex;

const E820_MAX_ENTRIES_ZEROPAGE: usize = 128;

lazy_static::lazy_static! {
    static ref CMDLINE: Mutex<String> = Mutex::new(String::new());
}

pub fn cmdline() -> &'static Mutex<String> {
    &CMDLINE
}

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
    pad0: [u8; 0x1e8],
    e820_entries: u8,
    pad1: [u8; 0x8],
    setup_sects: u8,
    pad2: [u8; 0x1b],
    kernel_version: u16,
    type_of_loader: u8,
    loadflags: BootLoadFlags,
    setup_move_size: u16,
    code32_start: u32,
    ramdisk_image: u32,
    ramdisk_size: u32,
    bootsect_kludge: u32,
    heap_end_ptr: u16,
    pad3: [u8; 2],
    cmd_line_ptr: u32,
    pad4: [u8; 12],
    cmdline_size: u32,
    pad5: [u8; 12],
    payload_offset: u32,
    payload_length: u32,
    pad6: [u8; 128],
    e820_table: [BootE820Entry; E820_MAX_ENTRIES_ZEROPAGE],
    pad7: [u8; 0x330],
}

impl BootParams {
    pub fn fill(
        setup_addr: GuestPhysAddr,
        initrd_addr: GuestPhysAddr,
        root_cmdline_addr: GuestPhysAddr,
        root_cmdline: &str,
    ) -> HvResult {
        let boot_params_hpa = root_zone_gpa_as_mut_ptr(setup_addr) as HostPhysAddr;
        let boot_params = unsafe { &mut *(boot_params_hpa as *mut BootParams) };

        // TODO: get kernel version

        // set bootloader type as undefined
        boot_params.type_of_loader = 0xff;
        let mut loadflags = boot_params.loadflags;
        // print early messages
        loadflags &= !BootLoadFlags::QUIET_FLAG;
        // disable heap_ptr
        loadflags &= !BootLoadFlags::CAN_USE_HEAP;
        boot_params.loadflags = loadflags;

        // TODO: tmp command
        unsafe {
            copy_nonoverlapping(
                root_cmdline.as_ptr(),
                root_zone_gpa_as_mut_ptr(root_cmdline_addr),
                root_cmdline.len(),
            )
        };
        boot_params.cmd_line_ptr = root_cmdline_addr as _;

        // set e820
        // TODO: zone config
        boot_params.set_e820_entries(&root_zone_config());

        // parse cmdline
        let hv_cmdline = CMDLINE.lock().clone();
        for param in hv_cmdline.split_whitespace() {
            let mut parts = param.splitn(2, '=');
            let key = parts.next().unwrap().to_string();
            let value = parts.next().map(|s| s.to_string());
            match key.as_str() {
                "initrd_size" => {
                    boot_params.set_initrd(initrd_addr as _, value.unwrap().parse::<u32>().unwrap())
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn set_e820_entries(&mut self, config: &HvZoneConfig) {
        let mut index = 0;
        for mem_region in config.memory_regions().iter() {
            match mem_region.mem_type {
                MEM_TYPE_RAM => {
                    self.e820_table[index] = BootE820Entry {
                        addr: mem_region.virtual_start,
                        size: mem_region.size,
                        _type: E820Type::E820_RAM,
                    };
                    index += 1;
                }
                /* FIXME: reserved?
                _ => {
                    self.e820_table[index] = BootE820Entry {
                        addr: mem_region.virtual_start,
                        size: mem_region.size,
                        _type: E820Type::E820_RESERVED,
                    };
                    index += 1;
                }*/
                _ => {}
            }
        }
        self.e820_entries = index as _;
    }

    fn set_initrd(&mut self, ramdisk_image: u32, ramdisk_size: u32) {
        // FIXME:
        return;
        self.ramdisk_image = ramdisk_image;
        self.ramdisk_size = ramdisk_size;
        info!("initrd size: {}", self.ramdisk_size);
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

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    pub mods_count: u32,
    pub mods_addr: u32,
}

impl MultibootInfo {
    fn new(addr: usize) -> Self {
        let multiboot_info = unsafe { &*(addr as *const MultibootInfo) };
        multiboot_info.clone()
    }

    pub fn init(info_addr: usize) {
        let boot_info = MultibootInfo::new(info_addr);
        println!("{:#x?}", boot_info);

        let cmd_ptr = boot_info.cmdline as *const c_char;
        let cmd_cstr = unsafe { CStr::from_ptr(cmd_ptr) };
        let cmd_str = cmd_cstr.to_str().unwrap();
        CMDLINE.lock().push_str(cmd_str);

        println!("cmdline: {}", CMDLINE.lock().as_str());
    }
}
