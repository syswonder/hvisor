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
//

#![allow(unused)]
use crate::error::{HvError, HvResult};
use crate::memory::addr::is_aligned;
use crate::memory::{Frame, MemFlags, MemoryRegion, PhysAddr, VirtAddr};
use alloc::{sync::Arc, vec::Vec};
use core::{fmt::Debug, marker::PhantomData, slice};
use spin::Mutex;

use hvisor_pt::arch::aarch64::{vmsav8_4k_3level_arch, vmsav8_4k_4level_arch, Aarch64PageTable};
use hvisor_pt::common::{
    addr::PAddrExec,
    arch::PTArchExec,
    frame::{FrameSize, MemAttr},
};
use hvisor_pt::spec::memory::{PageTableMem, PageTableMemExec, Table};
use vstd::prelude::*;

#[derive(Debug)]
pub enum PagingError {
    NoMemory,
    NotMapped,
    AlreadyMapped,
    MappedToHugePage,
}

pub type PagingResult<T = ()> = Result<T, PagingError>;

impl From<PagingError> for HvError {
    fn from(err: PagingError) -> Self {
        match err {
            PagingError::NoMemory => hv_err!(ENOMEM),
            _ => hv_err!(EFAULT, format!("{:?}", err)),
        }
    }
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    Size4K = 0x1000,
    Size2M = 0x20_0000,
    Size1G = 0x4000_0000,
}

#[derive(Debug, Copy, Clone)]
pub struct Page<VA> {
    vaddr: VA,
    size: PageSize,
}

impl PageSize {
    pub const fn is_aligned(self, addr: usize) -> bool {
        self.page_offset(addr) == 0
    }

    pub const fn align_down(self, addr: usize) -> usize {
        addr & !(self as usize - 1)
    }

    pub const fn page_offset(self, addr: usize) -> usize {
        addr & (self as usize - 1)
    }

    pub const fn is_huge(self) -> bool {
        matches!(self, Self::Size1G | Self::Size2M)
    }
}

impl<VA: Into<usize> + Copy> Page<VA> {
    pub fn new_aligned(vaddr: VA, size: PageSize) -> Self {
        debug_assert!(size.is_aligned(vaddr.into()));
        Self { vaddr, size }
    }
}

pub trait GenericPTE: Debug + Clone {
    /// Returns the physical address mapped by this entry.
    fn addr(&self) -> PhysAddr;
    /// Returns the flags of this entry.
    fn flags(&self) -> MemFlags;
    /// Returns whether this entry is zero.
    fn is_unused(&self) -> bool;
    /// Returns whether this entry flag indicates present.
    fn is_present(&self) -> bool;
    /// Returns whether this entry maps to a huge frame.
    fn is_huge(&self) -> bool;
    /// Set physical address for terminal entries.
    fn set_addr(&mut self, paddr: PhysAddr);
    /// Set flags for terminal entries.
    fn set_flags(&mut self, flags: MemFlags, is_huge: bool);
    /// Set physical address and flags for intermediate table entries.
    fn set_table(&mut self, paddr: PhysAddr);
    /// Set this entry to zero.
    fn clear(&mut self);
}

const ENTRY_COUNT: usize = 512;

pub trait PagingInstr {
    unsafe fn activate(root_paddr: PhysAddr);
    fn flush(vaddr: Option<usize>);
}

/// A basic read-only page table for address query only.
pub trait GenericPageTableImmut: Sized {
    type VA: From<usize> + Into<usize> + Copy;

    fn level(&self) -> usize;
    fn starting_level(&self) -> usize;

    fn root_paddr(&self) -> PhysAddr;
    fn query(&self, vaddr: Self::VA) -> PagingResult<(PhysAddr, MemFlags, PageSize)>;
}

/// A extended mutable page table can change mappings.
pub trait GenericPageTable: GenericPageTableImmut {
    fn new(level: usize) -> Self;

    fn map(&mut self, region: &MemoryRegion<Self::VA>) -> HvResult;
    fn unmap(&mut self, region: &MemoryRegion<Self::VA>) -> HvResult;
    fn update(
        &mut self,
        vaddr: Self::VA,
        paddr: PhysAddr,
        flags: MemFlags,
    ) -> PagingResult<PageSize>;

    fn clone(&self) -> Self;

    unsafe fn activate(&self);
    fn flush(&self, vaddr: Option<Self::VA>);
}

verus! {

/// Memory that stores page tables.
pub struct PageTableMemory {
    arch: PTArchExec,
    tables: Vec<Frame>,
}

impl PageTableMemExec for PageTableMemory {
    spec fn view(self) -> PageTableMem {
        PageTableMem {
            tables: Seq::new(self.tables.len() as nat, |i| Table {
                base: self.frames[i].start_paddr(),
                size: FrameSize::Size4K,
                level: 0,
            }),
            arch: self.arch@,
        }
    }

    fn root(&self) -> PAddrExec {
        PAddrExec(self.tables[0].start_paddr())
    }

    fn new_init(arch: PTArchExec) -> Self {
        Self {
            arch,
            tables: vec![Frame::new().unwrap()],
        }
    }

    fn is_table_empty(&self, base: PAddrExec) -> bool {
        let table = self.tables.iter().find(|t| t.start_paddr() == base.0).unwrap();
        let contents = unsafe {
            core::slice::from_raw_parts(base.0 as *const u8, table.size())
        };
        contents.iter().all(|&b| b == 0)
    }

    fn alloc_table(&mut self, level: usize) -> (PAddrExec, FrameSize) {
        let frame = Frame::new().unwrap();
        let res = (PAddrExec(frame.start_paddr()), FrameSize::Size4K);
        self.tables.push(frame);
        res
    }

    fn dealloc_table(&mut self, base: PAddrExec) {
        let index = self.tables.iter().position(|f| f.start_paddr() == base.0).unwrap();
        self.tables.remove(index);
    }

    fn read(&self, base:PAddrExec, index:usize) -> u64 {
        unsafe { (base.0 as *const u64).offset(index as isize).read_volatile() }
    }

    fn write(&mut self, base:PAddrExec, index:usize, val:u64) {
        unsafe { (base.0 as *mut u64).offset(index as isize).write_volatile(val) }
    }
}

}

/// Page table implementation for aarch64.
pub struct HvPageTable<VA: From<usize> + Into<usize> + Copy, I: PagingInstr> {
    inner: Aarch64PageTable<PageTableMemory>,
    clonee_lock: Arc<Mutex<()>>,
    _phantom: PhantomData<(VA, I)>,
}

impl<VA, I> HvPageTable<VA, I>
where
    VA: From<usize> + Into<usize> + Copy,
    I: PagingInstr,
{
    pub fn clone_from(src: &impl GenericPageTableImmut) -> Self {
        // XXX: The clonee won't track intermediate tables, must ensure it lives shorter than the
        // original page table.
        let pt = Self::new(src.level());
        let dst_p4_table =
            unsafe { slice::from_raw_parts_mut(pt.root_paddr() as *mut u64, ENTRY_COUNT) };
        let src_p4_table =
            unsafe { slice::from_raw_parts(src.root_paddr() as *const u64, ENTRY_COUNT) };
        dst_p4_table.clone_from_slice(src_p4_table);
        pt
    }
}

impl<VA, I> GenericPageTableImmut for HvPageTable<VA, I>
where
    VA: From<usize> + Into<usize> + Copy,
    I: PagingInstr,
{
    type VA = VA;

    fn level(&self) -> usize {
        self.inner.arch().level_count()
    }

    fn starting_level(&self) -> usize {
        0
    }

    fn root_paddr(&self) -> PhysAddr {
        self.inner.root()
    }

    fn query(&self, vaddr: Self::VA) -> PagingResult<(PhysAddr, MemFlags, PageSize)> {
        let _lock = self.clonee_lock.lock();
        info!("query {:#x}", vaddr.into());
        self.inner
            .query(vaddr.into())
            .map(|(vb, pb, sz, attr)| (pb, attr_to_flags(attr), frame_size_to_page_size(sz)))
            .map_err(|_| PagingError::NotMapped)
    }
}

impl<VA, I> GenericPageTable for HvPageTable<VA, I>
where
    VA: From<usize> + Into<usize> + Copy,
    I: PagingInstr,
{
    fn new(level: usize) -> Self {
        assert!(level == 3 || level == 4);
        let arch = if level == 4 {
            vmsav8_4k_4level_arch()
        } else {
            vmsav8_4k_3level_arch()
        };
        Self {
            inner: Aarch64PageTable::new(arch, 0x0, 0x80000000),
            clonee_lock: Arc::new(Mutex::new(())),
            _phantom: PhantomData,
        }
    }

    fn map(&mut self, region: &MemoryRegion<Self::VA>) -> HvResult {
        info!(
            "create mapping in {}: {:#x?}",
            core::any::type_name::<Self>(),
            region
        );
        let _lock = self.clonee_lock.lock();
        let mut vaddr = region.start.into();
        let mut size = region.size;
        while size > 0 {
            let paddr = region.mapper.map_fn(vaddr);
            let frame_size = if PageSize::Size1G.is_aligned(vaddr)
                && PageSize::Size1G.is_aligned(paddr)
                && size >= PageSize::Size1G as usize
                && !region.flags.contains(MemFlags::NO_HUGEPAGES)
            {
                FrameSize::Size1G
            } else if PageSize::Size2M.is_aligned(vaddr)
                && PageSize::Size2M.is_aligned(paddr)
                && size >= PageSize::Size2M as usize
                && !region.flags.contains(MemFlags::NO_HUGEPAGES)
            {
                FrameSize::Size2M
            } else {
                FrameSize::Size4K
            };
            self.inner
                .map(vaddr, paddr, frame_size, flags_to_attr(region.flags))
                .map_err(|_| PagingError::AlreadyMapped)?;
            vaddr += frame_size.as_usize();
            size -= frame_size.as_usize();
        }
        Ok(())
    }

    fn unmap(&mut self, region: &MemoryRegion<Self::VA>) -> HvResult {
        info!(
            "remove mapping in {}: {:#x?}",
            core::any::type_name::<Self>(),
            region
        );
        let _lock = self.clonee_lock.lock();
        let mut vaddr = region.start.into();
        let mut size = region.size;
        while size > 0 {
            let page_size = self
                .inner
                .query(vaddr.into())
                .map(|(_, _, sz, _)| frame_size_to_page_size(sz))
                .map_err(|_| PagingError::NotMapped)?;
            self.inner
                .unmap(vaddr.into())
                .map_err(|_| PagingError::NotMapped)?;
            if !page_size.is_aligned(vaddr) {
                error!("error vaddr={:#x?}", vaddr);
                loop {}
            }
            vaddr += page_size as usize;
            size -= page_size as usize;
        }
        Ok(())
    }

    fn update(
        &mut self,
        vaddr: Self::VA,
        paddr: PhysAddr,
        flags: MemFlags,
    ) -> PagingResult<PageSize> {
        let _lock = self.clonee_lock.lock();
        let page_size = self
            .inner
            .query(vaddr.into())
            .map(|(_, _, sz, _)| frame_size_to_page_size(sz))
            .map_err(|_| PagingError::NotMapped)?;
        self.inner
            .protect(vaddr.into(), flags_to_attr(flags))
            .map(|_| page_size)
            .map_err(|_| PagingError::NotMapped)
    }

    fn clone(&self) -> Self {
        let mut pt = Self::clone_from(self);
        // clone with lock to avoid data racing between it and its clonees.
        pt.clonee_lock = self.clonee_lock.clone();
        pt
    }

    unsafe fn activate(&self) {
        I::activate(self.root_paddr())
    }

    fn flush(&self, vaddr: Option<Self::VA>) {
        I::flush(vaddr.map(Into::into))
    }
}

fn attr_to_flags(attr: MemAttr) -> MemFlags {
    let mut flags = MemFlags::empty();
    if attr.readable {
        flags |= MemFlags::READ;
    }
    if attr.writable {
        flags |= MemFlags::WRITE;
    }
    if attr.executable {
        flags |= MemFlags::EXECUTE;
    }
    if attr.device {
        flags |= MemFlags::IO;
    }
    if attr.user_accessible {
        flags |= MemFlags::USER;
    }
    flags
}

fn flags_to_attr(flags: MemFlags) -> MemAttr {
    MemAttr {
        readable: flags.contains(MemFlags::READ),
        writable: flags.contains(MemFlags::WRITE),
        executable: flags.contains(MemFlags::EXECUTE),
        device: flags.contains(MemFlags::IO),
        user_accessible: flags.contains(MemFlags::USER),
    }
}

fn page_size_to_frame_size(size: PageSize) -> FrameSize {
    match size {
        PageSize::Size4K => FrameSize::Size4K,
        PageSize::Size2M => FrameSize::Size2M,
        PageSize::Size1G => FrameSize::Size1G,
    }
}

fn frame_size_to_page_size(size: FrameSize) -> PageSize {
    match size {
        FrameSize::Size4K => PageSize::Size4K,
        FrameSize::Size2M => PageSize::Size2M,
        FrameSize::Size1G => PageSize::Size1G,
        _ => panic!("Unsupported frame size"),
    }
}
