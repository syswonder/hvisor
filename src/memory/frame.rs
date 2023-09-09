//! Physical memory allocation.

use bitmap_allocator::BitAlloc;

use spin::Mutex;

use super::addr::{align_down, align_up, is_aligned, phys_to_virt, virt_to_phys, PhysAddr};
use crate::consts::PAGE_SIZE;
use crate::error::HvResult;

// Support max 1M * 4096 = 1GB memory.
type FrameAlloc = bitmap_allocator::BitAlloc1M;

struct FrameAllocator {
    base: PhysAddr,
    inner: FrameAlloc,
}

/// A safe wrapper for physical frame allocation.
#[derive(Debug)]
pub struct Frame {
    start_paddr: PhysAddr,
    frame_count: usize,
}

static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::empty());

impl FrameAllocator {
    const fn empty() -> Self {
        Self {
            base: 0,
            inner: FrameAlloc::DEFAULT,
        }
    }

    fn init(&mut self, base: PhysAddr, size: usize) {
        self.base = align_up(base);
        let page_count = align_up(size) / PAGE_SIZE;
        self.inner.insert(0..page_count);
    }

    /// # Safety
    ///
    /// This function is unsafe because you need to deallocate manually.
    unsafe fn alloc(&mut self) -> Option<PhysAddr> {
        let ret = self.inner.alloc().map(|idx| idx * PAGE_SIZE + self.base);
        trace!("Allocate frame: {:x?}", ret);
        ret
    }

    /// # Safety
    ///
    /// This function is unsafe because your need to deallocate manually.
    unsafe fn alloc_contiguous(
        &mut self,
        frame_count: usize,
        align_log2: usize,
    ) -> Option<PhysAddr> {
        let ret = self
            .inner
            .alloc_contiguous(frame_count, align_log2)
            .map(|idx| idx * PAGE_SIZE + self.base);
        trace!(
            "Allocate {} frames with alignment {}: {:x?}",
            frame_count,
            1 << align_log2,
            ret
        );
        ret
    }

    /// # Safety
    ///
    /// This function is unsafe because the frame must have been allocated.
    unsafe fn dealloc(&mut self, target: PhysAddr) {
        trace!("Deallocate frame: {:x}", target);
        self.inner.dealloc((target - self.base) / PAGE_SIZE)
    }

    /// # Safety
    ///
    /// This function is unsafe because the frames must have been allocated.
    unsafe fn dealloc_contiguous(&mut self, target: PhysAddr, frame_count: usize) {
        trace!("Deallocate {} frames: {:x}", frame_count, target);
        let start_idx = (target - self.base) / PAGE_SIZE;
        for i in start_idx..start_idx + frame_count {
            self.inner.dealloc(i)
        }
    }
}

#[allow(dead_code)]
impl Frame {
    /// Allocate one physical frame.
    pub fn new() -> HvResult<Self> {
        unsafe {
            FRAME_ALLOCATOR
                .lock()
                .alloc()
                .map(|start_paddr| Self {
                    start_paddr,
                    frame_count: 1,
                })
                .ok_or(hv_err!(ENOMEM))
        }
    }

    /// Allocate one physical frame and fill with zero.
    pub fn new_zero() -> HvResult<Self> {
        let mut f = Self::new()?;
        f.zero();
        Ok(f)
    }

    /// Allocate contiguous physical frames.
    pub fn new_contiguous(frame_count: usize, align_log2: usize) -> HvResult<Self> {
        unsafe {
            FRAME_ALLOCATOR
                .lock()
                .alloc_contiguous(frame_count, align_log2)
                .map(|start_paddr| Self {
                    start_paddr,
                    frame_count,
                })
                .ok_or(hv_err!(ENOMEM))
        }
    }

    /// Constructs a frame from a raw physical address without automatically calling the destructor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the user must ensure that this is an available physical
    /// frame.
    pub unsafe fn from_paddr(start_paddr: PhysAddr) -> Self {
        assert!(is_aligned(start_paddr));
        Self {
            start_paddr,
            frame_count: 0,
        }
    }

    /// Get the start physical address of this frame.
    pub fn start_paddr(&self) -> PhysAddr {
        self.start_paddr
    }

    /// Get the total size (in bytes) of this frame.
    pub fn size(&self) -> usize {
        self.frame_count * PAGE_SIZE
    }

    /// convert to raw a pointer.
    pub fn as_ptr(&self) -> *const u8 {
        phys_to_virt(self.start_paddr) as *const u8
    }

    /// convert to a mutable raw pointer.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        phys_to_virt(self.start_paddr) as *mut u8
    }

    /// Fill `self` with `byte`.
    pub fn fill(&mut self, byte: u8) {
        unsafe { core::ptr::write_bytes(self.as_mut_ptr(), byte, self.size()) }
    }

    /// Fill `self` with zero.
    pub fn zero(&mut self) {
        self.fill(0)
    }

    /// Forms a slice that can read data.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.size()) }
    }

    /// Forms a mutable slice that can write data.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size()) }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            match self.frame_count {
                0 => {} // Do not deallocate when use Frame::from_paddr()
                1 => FRAME_ALLOCATOR.lock().dealloc(self.start_paddr),
                _ => FRAME_ALLOCATOR
                    .lock()
                    .dealloc_contiguous(self.start_paddr, self.frame_count),
            }
        }
    }
}

/// Initialize the physical frame allocator.
pub(super) fn init() {
    let mem_pool_start = crate::consts::free_memory_start();
    let mem_pool_end = align_down(crate::consts::hv_end());
    let mem_pool_size = mem_pool_end - mem_pool_start;
    FRAME_ALLOCATOR
        .lock()
        .init(virt_to_phys(mem_pool_start), mem_pool_size);

    info!(
        "Frame allocator init end: {:#x?}",
        mem_pool_start..mem_pool_end
    );
}
