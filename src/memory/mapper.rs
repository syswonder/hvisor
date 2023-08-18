use super::addr::{align_down, virt_to_phys};
use super::{AlignedPage, MemFlags, MemoryRegion, PhysAddr};

static EMPTY_PAGE: AlignedPage = AlignedPage::new();

#[derive(Clone, Debug)]
pub(super) enum Mapper {
    Offset(usize),
    Fixed(usize),
}

impl Mapper {
    pub fn map_fn<VA: Into<usize>>(&self, vaddr: VA) -> PhysAddr {
        match self {
            Self::Offset(ref off) => vaddr.into() - *off,
            Self::Fixed(ref paddr) => *paddr,
        }
    }
}

impl<VA: From<usize> + Into<usize> + Copy> MemoryRegion<VA> {
    pub fn new_with_empty_mapper(start: VA, size: usize, flags: MemFlags) -> Self {
        let paddr = virt_to_phys(EMPTY_PAGE.as_ptr() as usize);
        Self::new(start, size, flags, Mapper::Fixed(paddr))
    }

    pub fn new_with_offset_mapper(
        start_vaddr: VA,
        start_paddr: PhysAddr,
        size: usize,
        flags: MemFlags,
    ) -> Self {
        let start_vaddr = align_down(start_vaddr.into());
        let start_paddr = align_down(start_paddr);
        // bug: vaddr > paddr?
        let phys_virt_offset = start_vaddr - start_paddr;
        Self::new(
            start_vaddr.into(),
            size,
            flags,
            Mapper::Offset(phys_virt_offset),
        )
    }
}
