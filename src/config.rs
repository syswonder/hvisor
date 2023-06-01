use core::fmt::{Debug, Formatter, Result};
use core::{mem::size_of, slice};

use crate::error::HvResult;
use crate::memory::MemFlags;

const CONFIG_SIGNATURE: [u8; 6] = *b"RVMSYS";
const CONFIG_REVISION: u16 = 10;

const HV_CELL_NAME_MAXLEN: usize = 31;
const HV_MAX_IOMMU_UNITS: usize = 8;

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct HvConsole {
    address: u64,
    size: u32,
    console_type: u16,
    flags: u16,
    divider: u32,
    gate_nr: u32,
    clock_reg: u64,
}

/// The jailhouse cell configuration.
///
/// @note Keep Config._HEADER_FORMAT in jailhouse-cell-linux in sync with this
/// structure.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvCellDesc {
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

    console: HvConsole,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
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
#[repr(C, packed)]
pub struct HvIrqChip {
    address: u64,
    id: u32,
    pin_base: u32,
    pin_bitmap: [u32; 4],
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

#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct ArchPlatformInfo {
    gic_version: u8,
    gicd_base: u64,
    gicr_base: u64,
    maintenance_irq: u8,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct PlatformInfo {
    pci_mmconfig_base: u64,
    pci_mmconfig_end_bus: u8,
    pci_is_virtual: u8,
    pci_domain: u16,
    arch: ArchPlatformInfo,
}

/// General descriptor of the system.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct HvSystemConfig {
    pub signature: [u8; 6],
    pub revision: u16,
    flags: u32,

    /// Jailhouse's location in memory
    pub hypervisor_memory: HvMemoryRegion,
    debug_console: HvConsole,
    platform_info: PlatformInfo,
    pub root_cell: HvCellDesc,
    // CellConfigLayout placed here.
}

/// A dummy layout with all variant-size fields empty.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct CellConfigLayout {
    cpus: [u64; 0],
    mem_regions: [HvMemoryRegion; 0],
    cache_regions: [HvCacheRegion; 0],
    irqchips: [HvIrqChip; 0],
    pio_bitmap: [u8; 0],
    pci_devices: [HvPciDevice; 0],
    pci_caps: [HvPciCapability; 0],
}

pub struct CellConfig<'a> {
    desc: &'a HvCellDesc,
}

impl HvCellDesc {
    pub const fn config(&self) -> CellConfig {
        CellConfig::from(self)
    }

    pub const fn config_size(&self) -> usize {
        self.cpu_set_size as usize
            + self.num_memory_regions as usize * size_of::<HvMemoryRegion>()
            + self.num_cache_regions as usize * size_of::<HvCacheRegion>()
            + self.num_irqchips as usize * size_of::<HvIrqChip>()
            + self.pio_bitmap_size as usize
            + self.num_pci_devices as usize * size_of::<HvPciDevice>()
            + self.num_pci_caps as usize * size_of::<HvPciCapability>()
    }
}

impl HvSystemConfig {
    pub fn get<'a>() -> &'a Self {
        unsafe { &*crate::consts::hv_config_ptr() }
    }

    pub const fn size(&self) -> usize {
        size_of::<Self>() + self.root_cell.config_size()
    }

    pub fn check(&self) -> HvResult {
        if self.signature != CONFIG_SIGNATURE {
            return hv_result_err!(EINVAL, "HvSystemConfig signature not matched!");
        }
        if self.revision != CONFIG_REVISION {
            return hv_result_err!(EINVAL, "HvSystemConfig revision not matched!");
        }
        Ok(())
    }
}

impl<'a> CellConfig<'a> {
    const fn from(desc: &'a HvCellDesc) -> Self {
        Self { desc }
    }

    fn config_ptr<T>(&self) -> *const T {
        unsafe { (self.desc as *const HvCellDesc).add(1) as _ }
    }

    pub const fn size(&self) -> usize {
        self.desc.config_size()
    }

    pub fn cpu_set(&self) -> &[u64] {
        // XXX: data may unaligned, which cause panic on debug mode. Same below.
        // See: https://doc.rust-lang.org/src/core/slice/mod.rs.html#6435-6443
        unsafe { slice::from_raw_parts(self.config_ptr(), self.desc.cpu_set_size as usize / 8) }
    }

    pub fn mem_regions(&self) -> &[HvMemoryRegion] {
        unsafe {
            let ptr = self.cpu_set().as_ptr_range().end as _;
            slice::from_raw_parts(ptr, self.desc.num_memory_regions as usize)
        }
    }
}

impl Debug for CellConfig<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = self.desc.name;
        let mut len = 0;
        while name[len] != 0 {
            len += 1;
        }
        f.debug_struct("CellConfig")
            .field("name", &core::str::from_utf8(&name[..len]))
            .field("size", &self.size())
            .field("mem_regions", &self.mem_regions())
            .finish()
    }
}
