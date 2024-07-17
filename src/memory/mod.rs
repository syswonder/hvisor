pub mod addr;
pub mod frame;
pub mod heap;
pub mod mapper;
pub mod mm;
pub mod mmio;

use core::ops::{Deref, DerefMut};

use bitflags::bitflags;

pub use addr::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr, HostVirtAddr, PhysAddr, VirtAddr};
pub use frame::Frame;
pub use mm::{MemoryRegion, MemorySet, PARKING_INST_PAGE};
pub use mmio::*;
use spin::{Once, RwLock};

use crate::arch::{paging, s1pt::Stage1PageTable};

pub const PAGE_SIZE: usize = paging::PageSize::Size4K as usize;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct MemFlags: u64 {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const DMA           = 1 << 3;
        const IO            = 1 << 4;
        const COMMUNICATION = 1 << 5;
        const LOADABLE      = 1 << 6;
        const ROOTSHARED    = 1 << 7;
        const NO_HUGEPAGES  = 1 << 8;
        const USER          = 1 << 9;
    }
}

/// Page table used for hypervisor.
// #[cfg(feature = "plic")]
pub static HV_PT: Once<RwLock<MemorySet<Stage1PageTable>>> = Once::new();
// #[cfg(feature = "plic")]
pub fn hv_page_table<'a>() -> &'a RwLock<MemorySet<Stage1PageTable>> {
    HV_PT.get().expect("Uninitialized hypervisor page table!")
}

// pub fn init_hv_page_table() {
//     let sys_config = HvSystemConfig::get();
//     let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
//     let hv_phys_size = sys_config.hypervisor_memory.size as usize;
//     let trampoline_page = TRAMPOLINE_START as usize - unsafe { PHYS_VIRT_OFFSET };
//     let gicd_base = sys_config.platform_info.arch.gicd_base;
//     let gicr_base = sys_config.platform_info.arch.gicr_base;
//     let gicr_size: u64 = MAX_CPU_NUM as u64 * GICR_SIZE;
//     // let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
//     // let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;

//     let mut hv_pt: MemorySet<Stage1PageTable> = MemorySet::new();

//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         HV_BASE as GuestPhysAddr,
//         hv_phys_start as HostPhysAddr,
//         hv_phys_size as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::NO_HUGEPAGES,
//     ))?;

//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         trampoline_page as GuestPhysAddr,
//         trampoline_page as HostPhysAddr,
//         PAGE_SIZE as usize,
//         MemFlags::READ | MemFlags::WRITE,
//     ))?;

//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         UART_BASE_VIRT,
//         sys_config.debug_console.address as PhysAddr,
//         sys_config.debug_console.size as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     ))?;

//     // add gicd memory map
//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         gicd_base as GuestPhysAddr,
//         gicd_base as HostPhysAddr,
//         GICD_SIZE as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     ))?;

//     //add gicr memory map
//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         gicr_base as GuestPhysAddr,
//         gicr_base as HostPhysAddr,
//         gicr_size as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     ))?;

//     // // Map pci region. Jailhouse doesn't map pci region to el2.
//     // // Now we simplify the complex pci handler and just map it.
//     // hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//     //     mmcfg_start as GuestPhysAddr,
//     //     mmcfg_start as HostPhysAddr,
//     //     mmcfg_size as usize,
//     //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     // ))?;

//     // add virtio map
//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         0xa000000 as GuestPhysAddr,
//         0xa000000 as HostPhysAddr,
//         0x4000 as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     ))?;

//     // add virt gic its
//     hv_pt.insert(MemoryRegion::new_with_offset_mapper(
//         0x8080000 as GuestPhysAddr,
//         0x8080000 as HostPhysAddr,
//         0x20000 as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     ))?;

//     info!("Hypervisor page table init end.");

//     debug!("Hypervisor virtual memory set: {:#x?}", hv_pt);

//     unsafe {
//         hv_pt.activate();
//     }

//     HV_PT.call_once(|| RwLock::new(hv_pt));
// }

#[repr(align(4096))]
pub struct AlignedPage([u8; PAGE_SIZE]);

impl AlignedPage {
    pub const fn new() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Deref for AlignedPage {
    type Target = [u8; PAGE_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
