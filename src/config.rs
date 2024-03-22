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

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvCacheRegion {
    start: u32,
    size: u32,
    cache_type: u8,
    _padding: u8,
    flags: u16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HvIrqChip {
    pub address: u64,
    pub id: u32,
    pub pin_base: u32,
    pub pin_bitmap: [u32; 4],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvPciDevice {
    pci_device_type: u8,
    iommu: u8,
    domain: u16,
    bdf: u16,
    bar_mask: [u32; 6],
    caps_start: u16,
    num_caps: u16,
    num_msi_vectors: u8,
    msi_64bits: u8,
    num_msix_vectors: u16,
    msix_region_size: u16,
    msix_address: u64,
    /// Memory region index of virtual shared memory device.
    shmem_region: u32,
    /// PCI subclass and interface ID of virtual shared memory device.
    shmem_protocol: u16,
    _padding: [u8; 2],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvPciCapability {
    id: u16,
    start: u16,
    len: u16,
    flags: u16,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct HvIommu {
    base: u64,
    size: u32,
    amd_bdf: u16,
    amd_base_cap: u8,
    amd_msi_cap: u8,
    amd_features: u32,
}

#[repr(C)]
#[repr(align(4096))]
pub struct DTBBlob1([u8; include_bytes!("../tenants/linux3.dtb").len()]);

#[link_section = ".dtb"]
/// the tenant dtb file
pub static TENANT1_DTB: DTBBlob1 = DTBBlob1(*include_bytes!("../tenants/linux3.dtb"));
#[link_section = ".initrd"]
/// the tenant kernel file
pub static TENANT1: [u8; include_bytes!("../tenants/Image-62").len()] =
    *include_bytes!("../tenants/Image-62");
// pub static TENANTS: [(&'static [u8], &'static [u8]); 1] = [(&TENANT1, &TENANT1_DTB)];
// #[link_section = ".dtb"]
// /// the tenant dtb file
// pub static TENANT_DTB: [u8; include_bytes!("../../tenants/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb")
//     .len()] = *include_bytes!("../../tenants/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb");
// #[link_section = ".initrd"]
// static TENANT: [u8; include_bytes!("../../tenants/rCore-Tutorial-v3/rCore-Tutorial-v3.bin").len()] =
//     *include_bytes!("../../tenants/rCore-Tutorial-v3/rCore-Tutorial-v3.bin");

#[repr(C)]
#[repr(align(4096))]
pub struct DTBBlob2([u8; include_bytes!("../tenants/os_ch5.dtb").len()]);

#[link_section = ".dtb"]
/// the tenant dtb file
pub static TENANT2_DTB: DTBBlob2 = DTBBlob2(*include_bytes!("../tenants/os_ch5.dtb"));

#[link_section = ".rcore"]
/// the tenant kernel file
pub static TENANT2: [u8; include_bytes!("../tenants/os.bin").len()] =
    *include_bytes!("../tenants/os.bin");
pub static TENANTS: [(&'static [u8], &'static [u8]); 1] = [(&TENANT2, &TENANT2_DTB.0)];
// pub static TENANTS: [(&'static [u8], &'static [u8]); 2] =
//     [(&TENANT1, &TENANT1_DTB.0), (&TENANT2, &TENANT2_DTB.0)];
