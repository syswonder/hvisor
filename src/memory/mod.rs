//! Hypervisor Memory Layout
//!
//!     +--------------------------------------+ - HV_BASE: 0xffff_ff00_0000_0000 (lower address)
//!     | HvHeader                             |
//!     +--------------------------------------+
//!     | Text Segment                         |
//!     |                                      |
//!     +--------------------------------------+
//!     | Read-only Data Segment               |
//!     |                                      |
//!     +--------------------------------------+
//!     | Data Segment                         |
//!     |                                      |
//!     +--------------------------------------+
//!     | BSS Segment                          |
//!     | (includes hypervisor heap)           |
//!     |                                      |
//!     +--------------------------------------+ - PER_CPU_ARRAY_PTR (core_end)
//!     |  +--------------------------------+  |
//!     |  | Per-CPU Data 0                 |  |
//!     |  +--------------------------------+  |
//!     |  | Per-CPU Stack 0                |  |
//!     |  +--------------------------------+  | - PER_CPU_ARRAY_PTR + PER_CPU_SIZE
//!     |  | Per-CPU Data 1                 |  |
//!     |  +--------------------------------+  |
//!     |  | Per-CPU Stack 1                |  |
//!     |  +--------------------------------+  |
//!     :  :                                :  :
//!     :  :                                :  :
//!     |  +--------------------------------+  |
//!     |  | Per-CPU Data n-1               |  |
//!     |  +--------------------------------+  |
//!     |  | Per-CPU Stack n-1              |  |
//!     |  +--------------------------------+  | - hv_config_ptr
//!     |  | HvSystemConfig                 |  |
//!     |  | +----------------------------+ |  |
//!     |  | | CellConfigLayout           | |  |
//!     |  | |                            | |  |
//!     |  | +----------------------------+ |  |
//!     |  +--------------------------------+  |
//!     +--------------------------------------| - free_memory_start
//!     |  Dynamic Page Pool                   |
//!     :                                      :
//!     :                                      :
//!     |                                      |
//!     +--------------------------------------+ - hv_end (higher address)
//!
mod frame;
mod heap;
mod mapper;
mod mm;
mod paging;

pub mod addr;

use core::ops::{Deref, DerefMut};

use bitflags::bitflags;
use crate::config::HvSystemConfig;
use crate::consts::HV_BASE;


pub use addr::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr, HostVirtAddr, PhysAddr, VirtAddr};
pub use frame::Frame;
pub use mm::{MemoryRegion, MemorySet};
pub use paging::{GenericPTE, PagingInstr};
pub use paging::{GenericPageTable, GenericPageTableImmut, Level4PageTable, Level4PageTableImmut, npages};

pub const PAGE_SIZE: usize = paging::PageSize::Size4K as usize;

//pub use mm::{MemoryRegion, MemorySet};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct MemFlags: u64 {
        const READ          = 1 << 0;
        const WRITE         = 1 << 1;
        const EXECUTE       = 1 << 2;
        const DMA           = 1 << 3;
        const IO            = 1 << 4;
        const NO_HUGEPAGES  = 1 << 8;
        const USER          = 1 << 9;
    }
}

pub fn init_heap() {
    // Set PHYS_VIRT_OFFSET early.
    unsafe {
        addr::PHYS_VIRT_OFFSET =
            HV_BASE - HvSystemConfig::get().hypervisor_memory.phys_start as usize
    };
    heap::init();
}

pub fn init_frame_allocator() {
    frame::init();
}

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