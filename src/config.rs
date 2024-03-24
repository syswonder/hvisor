use core::fmt::Debug;
use crate::memory::MemFlags;

const CONFIG_SIGNATURE: [u8; 6] = *b"HVISOR";
const CONFIG_REVISION: u16 = 10;

const HV_CELL_NAME_MAXLEN: usize = 31;
const HV_MAX_IOMMU_UNITS: usize = 8;

pub const DTB_ADDR: usize = 0xbfe00000;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvConsole {
    pub address: u64,
    pub size: u32,
    console_type: u16,
    pub flags: u16,
    divider: u32,
    gate_nr: u32,
    clock_reg: u64,
}

impl HvConsole {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            address: 0,
            size: 0,
            console_type: 0,
            flags: 0,
            divider: 0,
            gate_nr: 0,
            clock_reg: 0,
        }
    }
}
/// The jailhouse zone configuration.
///
/// @note Keep Config._HEADER_FORMAT in jailhouse-zone-linux in sync with this
/// structure.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvZoneDesc {
    signature: [u8; 6],
    revision: u16,

    name: [u8; HV_CELL_NAME_MAXLEN + 1],
    id: u32, // set by the driver
    flags: u32,

    pub cpu_set_size: u32,
    pub num_memory_regions: u32,
    pub num_cache_regions: u32,
    pub num_irqchips: u32,
    pub pio_bitmap_size: u32,
    pub num_pci_devices: u32,
    pub num_pci_caps: u32,

    vpci_irq_base: u32,

    cpu_reset_address: u64,
    msg_reply_timeout: u64,

    pub console: HvConsole,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HvMemoryRegion {
    pub phys_start: u64,
    pub virt_start: u64,
    pub size: u64,
    pub flags: MemFlags,
}
