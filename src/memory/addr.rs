//! Definition of phyical and virtual addresses.

#![allow(dead_code)]

use crate::consts::PAGE_SIZE;

pub type VirtAddr = usize;
pub type PhysAddr = usize;

pub type GuestVirtAddr = usize;
pub type GuestPhysAddr = usize;

pub type HostVirtAddr = VirtAddr;
pub type HostPhysAddr = PhysAddr;

pub const fn align_down(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

pub const fn align_up(addr: usize) -> usize {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn is_aligned(addr: usize) -> bool {
    page_offset(addr) == 0
}

pub const fn page_count(size: usize) -> usize {
    align_up(size) / PAGE_SIZE
}

pub const fn page_offset(addr: usize) -> usize {
    addr & (PAGE_SIZE - 1)
}
