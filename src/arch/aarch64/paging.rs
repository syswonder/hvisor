use crate::error::{HvError, HvResult};
use crate::memory::addr::is_aligned;
use crate::memory::{Frame, MemFlags, MemoryRegion, PhysAddr, VirtAddr};
use alloc::{sync::Arc, vec::Vec};
use core::{fmt::Debug, marker::PhantomData, slice};
use spin::Mutex;

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

    unsafe fn from_root(root_paddr: PhysAddr, level: usize) -> Self;
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

/// A immutable level-3/4 page table implements `GenericPageTableImmut`.
pub struct HvPageTableImmut<VA, PTE: GenericPTE> {
    /// Root table frame.
    root: Frame,
    level: usize,
    /// Phantom data.
    _phantom: PhantomData<(VA, PTE)>,
}

impl<VA, PTE> HvPageTableImmut<VA, PTE>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
{
    fn new(level: usize) -> Self {
        assert!(level == 3 || level == 4);
        Self {
            root: Frame::new_zero().expect("failed to allocate root frame for host page table"),
            level,
            _phantom: PhantomData,
        }
    }

    fn get_entry_mut(&self, vaddr: VA) -> PagingResult<(&mut PTE, PageSize)> {
        let vaddr = vaddr.into();
        let p3 = if self.level == 4 {
            let p4 = table_of_mut::<PTE>(self.root_paddr());
            let p4e = &mut p4[p4_index(vaddr)];
            next_table_mut(p4e)?
        } else {
            table_of_mut::<PTE>(self.root_paddr())
        };
        let p3e = &mut p3[p3_index(vaddr)];
        if p3e.is_huge() {
            return Ok((p3e, PageSize::Size1G));
        }

        let p2 = next_table_mut(p3e)?;
        let p2e = &mut p2[p2_index(vaddr)];
        if p2e.is_huge() {
            return Ok((p2e, PageSize::Size2M));
        }

        let p1 = next_table_mut(p2e)?;
        let p1e = &mut p1[p1_index(vaddr)];
        Ok((p1e, PageSize::Size4K))
    }

    fn walk(
        &self,
        table: &[PTE],
        level: usize,
        start_vaddr: usize,
        limit: usize,
        func: &impl Fn(usize, usize, usize, &PTE),
    ) {
        let mut n = 0;
        for (i, entry) in table.iter().enumerate() {
            let vaddr = start_vaddr + (i << (12 + (3 - level) * 9));
            if entry.is_present() {
                func(level, i, vaddr, entry);
                if level < 3 {
                    match next_table_mut(entry) {
                        Ok(entry) => self.walk(entry, level + 1, vaddr, limit, func),
                        Err(PagingError::MappedToHugePage) => {}
                        _ => unreachable!(),
                    }
                }
                n += 1;
                if n >= limit {
                    break;
                }
            }
        }
    }

    fn dump(&self, limit: usize) {
        static LOCK: Mutex<()> = Mutex::new(());
        let _lock = LOCK.lock();

        println!("Root: {:x?}", self.root_paddr());
        self.walk(
            table_of(self.root_paddr()),
            self.starting_level(),
            0,
            limit,
            &|level: usize, idx: usize, vaddr: usize, entry: &PTE| {
                for _ in 0..level * 2 {
                    print!(" ");
                }
                println!(
                    "[ADDR:{:#x?} level:{} - idx:{:03}], vaddr:{:08x?}: {:x?}",
                    entry as *const _ as VirtAddr, level, idx, vaddr, entry
                );
            },
        );
    }
}

impl<VA, PTE> GenericPageTableImmut for HvPageTableImmut<VA, PTE>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
{
    type VA = VA;

    unsafe fn from_root(root_paddr: PhysAddr, level: usize) -> Self {
        Self {
            root: Frame::from_paddr(root_paddr),
            level,
            _phantom: PhantomData,
        }
    }

    fn root_paddr(&self) -> PhysAddr {
        self.root.start_paddr()
    }

    fn query(&self, vaddr: VA) -> PagingResult<(PhysAddr, MemFlags, PageSize)> {
        let (entry, size) = self.get_entry_mut(vaddr)?;
        if entry.is_unused() {
            return Err(PagingError::NotMapped);
        }
        let off = size.page_offset(vaddr.into());
        Ok((entry.addr() + off, entry.flags(), size))
    }

    fn level(&self) -> usize {
        self.level
    }

    fn starting_level(&self) -> usize {
        if self.level == 4 {
            0
        } else {
            1
        }
    }
}

/// A extended level-3/4 page table that can change its mapping. It also tracks all intermediate
/// level tables. Locks need to be used if change the same page table concurrently.
struct HvPageTableUnlocked<VA, PTE: GenericPTE, I: PagingInstr> {
    inner: HvPageTableImmut<VA, PTE>,
    /// Intermediate level table frames.
    intrm_tables: Vec<Frame>,
    /// Phantom data.
    _phantom: PhantomData<(VA, PTE, I)>,
}

impl<VA, PTE, I> HvPageTableUnlocked<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    fn new(level: usize) -> Self {
        Self {
            inner: HvPageTableImmut::new(level),
            intrm_tables: Vec::new(),
            _phantom: PhantomData,
        }
    }

    unsafe fn from_root(root_paddr: PhysAddr, level: usize) -> Self {
        Self {
            inner: HvPageTableImmut::from_root(root_paddr, level),
            intrm_tables: Vec::new(),
            _phantom: PhantomData,
        }
    }

    fn alloc_intrm_table(&mut self) -> HvResult<PhysAddr> {
        let frame = Frame::new_zero()?;
        let paddr = frame.start_paddr();
        self.intrm_tables.push(frame);
        Ok(paddr)
    }

    fn _dealloc_intrm_table(&mut self, _paddr: PhysAddr) {}

    fn get_entry_mut_or_create(&mut self, page: Page<VA>) -> PagingResult<&mut PTE> {
        let vaddr: usize = page.vaddr.into();
        let p3 = if self.inner.level == 4 {
            let p4 = table_of_mut::<PTE>(self.inner.root_paddr());
            let p4e = &mut p4[p4_index(vaddr)];

            next_table_mut_or_create(p4e, || self.alloc_intrm_table())?
        } else {
            table_of_mut::<PTE>(self.inner.root_paddr())
        };
        let p3e = &mut p3[p3_index(vaddr)];
        if page.size == PageSize::Size1G {
            return Ok(p3e);
        }

        let p2 = next_table_mut_or_create(p3e, || self.alloc_intrm_table())?;
        let p2e = &mut p2[p2_index(vaddr)];
        if page.size == PageSize::Size2M {
            return Ok(p2e);
        }

        let p1 = next_table_mut_or_create(p2e, || self.alloc_intrm_table())?;
        let p1e = &mut p1[p1_index(vaddr)];
        Ok(p1e)
    }

    fn map_page(
        &mut self,
        page: Page<VA>,
        paddr: PhysAddr,
        flags: MemFlags,
    ) -> PagingResult<&mut PTE> {
        let entry: &mut PTE = self.get_entry_mut_or_create(page)?;
        if !entry.is_unused() {
            return Err(PagingError::AlreadyMapped);
        }
        entry.set_addr(page.size.align_down(paddr));
        entry.set_flags(flags, page.size.is_huge());
        Ok(entry)
    }

    fn unmap_page(&mut self, vaddr: VA) -> PagingResult<(PhysAddr, PageSize)> {
        let (entry, size) = self.inner.get_entry_mut(vaddr)?;
        if entry.is_unused() {
            return Err(PagingError::NotMapped);
        }
        let paddr = entry.addr();
        entry.clear();
        Ok((paddr, size))
    }

    fn update(&mut self, vaddr: VA, paddr: PhysAddr, flags: MemFlags) -> PagingResult<PageSize> {
        let (entry, size) = self.inner.get_entry_mut(vaddr)?;
        entry.set_addr(paddr);
        entry.set_flags(flags, size.is_huge());
        Ok(size)
    }
}

/// A extended level-3/4 page table implements `GenericPageTable`. It use locks to avoid data
/// racing between it and its clonees.
pub struct HvPageTable<VA, PTE: GenericPTE, I: PagingInstr> {
    inner: HvPageTableUnlocked<VA, PTE, I>,
    /// Make sure all accesses to the page table and its clonees is exclusive.
    clonee_lock: Arc<Mutex<()>>,
}

impl<VA, PTE, I> HvPageTable<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    #[allow(dead_code)]
    pub fn dump(&self, limit: usize) {
        self.inner.inner.dump(limit)
    }

    /// Clone only the top level page table mapping from `src`.
    pub fn clone_from(src: &impl GenericPageTableImmut) -> Self {
        // XXX: The clonee won't track intermediate tables, must ensure it lives shorter than the
        // original page table.
        let pt = Self::new(src.level());
        let dst_p4_table =
            unsafe { slice::from_raw_parts_mut(pt.root_paddr() as *mut PTE, ENTRY_COUNT) };
        let src_p4_table =
            unsafe { slice::from_raw_parts(src.root_paddr() as *const PTE, ENTRY_COUNT) };
        dst_p4_table.clone_from_slice(src_p4_table);
        pt
    }
}

impl<VA, PTE, I> GenericPageTableImmut for HvPageTable<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    type VA = VA;

    unsafe fn from_root(root_paddr: PhysAddr, level: usize) -> Self {
        Self {
            inner: HvPageTableUnlocked::from_root(root_paddr, level),
            clonee_lock: Arc::new(Mutex::new(())),
        }
    }

    fn root_paddr(&self) -> PhysAddr {
        self.inner.inner.root_paddr()
    }

    fn query(&self, vaddr: VA) -> PagingResult<(PhysAddr, MemFlags, PageSize)> {
        let _lock = self.clonee_lock.lock();
        self.inner.inner.query(vaddr)
    }

    fn level(&self) -> usize {
        self.inner.inner.level()
    }

    fn starting_level(&self) -> usize {
        self.inner.inner.starting_level()
    }
}

impl<VA, PTE, I> GenericPageTable for HvPageTable<VA, PTE, I>
where
    VA: From<usize> + Into<usize> + Copy,
    PTE: GenericPTE,
    I: PagingInstr,
{
    fn new(level: usize) -> Self {
        Self {
            inner: HvPageTableUnlocked::new(level),
            clonee_lock: Arc::new(Mutex::new(())),
        }
    }

    fn map(&mut self, region: &MemoryRegion<VA>) -> HvResult {
        assert!(
            is_aligned(region.start.into()),
            "region.start = {:#x?}",
            region.start.into()
        );
        assert!(is_aligned(region.size), "region.size = {:#x?}", region.size);
        trace!(
            "create mapping in {}: {:#x?}",
            core::any::type_name::<Self>(),
            region
        );
        let _lock = self.clonee_lock.lock();
        let mut vaddr = region.start.into();
        let mut size = region.size;
        while size > 0 {
            let paddr = region.mapper.map_fn(vaddr);
            let page_size = if PageSize::Size1G.is_aligned(vaddr)
                && PageSize::Size1G.is_aligned(paddr)
                && size >= PageSize::Size1G as usize
                && !region.flags.contains(MemFlags::NO_HUGEPAGES)
            {
                PageSize::Size1G
            } else if PageSize::Size2M.is_aligned(vaddr)
                && PageSize::Size2M.is_aligned(paddr)
                && size >= PageSize::Size2M as usize
                && !region.flags.contains(MemFlags::NO_HUGEPAGES)
            {
                PageSize::Size2M
            } else {
                PageSize::Size4K
            };
            let page = Page::new_aligned(vaddr.into(), page_size);
            self.inner
                .map_page(page, paddr, region.flags)
                .map_err(|e: PagingError| {
                    error!(
                        "failed to map page: {:#x?}({:?}) -> {:#x?}, {:?}",
                        vaddr, page_size, paddr, e
                    );
                    e
                })?;
            vaddr += page_size as usize;
            size -= page_size as usize;
        }
        Ok(())
    }

    fn unmap(&mut self, region: &MemoryRegion<VA>) -> HvResult {
        trace!(
            "destroy mapping in {}: {:#x?}",
            core::any::type_name::<Self>(),
            region
        );
        let _lock = self.clonee_lock.lock();
        let mut vaddr = region.start.into();
        let mut size = region.size;
        while size > 0 {
            let (_, page_size) = self.inner.unmap_page(vaddr.into()).map_err(|e| {
                error!("failed to unmap page: {:#x?}, {:?}", vaddr, e);
                e
            })?;
            if !page_size.is_aligned(vaddr) {
                error!("error vaddr={:#x?}", vaddr);
                loop {}
            }
            assert!(page_size.is_aligned(vaddr));
            assert!(page_size as usize <= size);
            vaddr += page_size as usize;
            size -= page_size as usize;
        }
        Ok(())
    }

    fn update(&mut self, vaddr: VA, paddr: PhysAddr, flags: MemFlags) -> PagingResult<PageSize> {
        let _lock = self.clonee_lock.lock();
        self.inner.update(vaddr, paddr, flags)
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

const fn p4_index(vaddr: usize) -> usize {
    (vaddr >> (12 + 27)) & (ENTRY_COUNT - 1)
}

const fn p3_index(vaddr: usize) -> usize {
    (vaddr >> (12 + 18)) & (ENTRY_COUNT - 1)
}

const fn p2_index(vaddr: usize) -> usize {
    (vaddr >> (12 + 9)) & (ENTRY_COUNT - 1)
}

const fn p1_index(vaddr: usize) -> usize {
    (vaddr >> 12) & (ENTRY_COUNT - 1)
}

fn table_of<'a, E>(paddr: PhysAddr) -> &'a [E] {
    let ptr = paddr as *const E;
    unsafe { slice::from_raw_parts(ptr, ENTRY_COUNT) }
}

fn table_of_mut<'a, E>(paddr: PhysAddr) -> &'a mut [E] {
    let ptr = paddr as *mut E;
    unsafe { slice::from_raw_parts_mut(ptr, ENTRY_COUNT) }
}

fn next_table_mut<'a, E: GenericPTE>(entry: &E) -> PagingResult<&'a mut [E]> {
    if !entry.is_present() {
        Err(PagingError::NotMapped)
    } else if entry.is_huge() {
        Err(PagingError::MappedToHugePage)
    } else {
        Ok(table_of_mut(entry.addr()))
    }
}

fn next_table_mut_or_create<'a, E: GenericPTE>(
    entry: &mut E,
    mut allocator: impl FnMut() -> HvResult<PhysAddr>,
) -> PagingResult<&'a mut [E]> {
    if entry.is_unused() {
        let paddr = allocator().map_err(|_| PagingError::NoMemory)?;
        entry.set_table(paddr);
        Ok(table_of_mut(paddr))
    } else {
        next_table_mut(entry)
    }
}
