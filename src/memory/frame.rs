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
//! Physical memory allocation.

use alloc::vec::Vec;
use bitmap_allocator::BitAlloc;

use spin::Mutex;

use super::addr::{align_down, align_up, is_aligned, PhysAddr};
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
        f.clear();
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

    /// allocate contigugous frames, and you can specify the alignment, set the lower `align_log2` bits to 0.
    pub fn new_contiguous_with_base(frame_count: usize, align_log2: usize) -> HvResult<Self> {
        let align_mask = (1 << align_log2) - 1;
        // Create a vector to keep track of attempted frames
        let mut attempted_frames = Vec::new();
        loop {
            if let Ok(frame) = Frame::new_contiguous(frame_count, 0) {
                if frame.start_paddr() & align_mask == 0 {
                    // info!(
                    //     "new contiguous success!!! start_paddr:0x{:x}",
                    //     frame.start_paddr()
                    // );
                    return Ok(frame);
                } else {
                    let start_paddr = frame.start_paddr();
                    let next_aligned_addr = (start_paddr + align_mask) & !align_mask;
                    let temp_frame_count = (next_aligned_addr - start_paddr) / PAGE_SIZE;
                    drop(frame);
                    attempted_frames.push(Frame::new_contiguous(temp_frame_count, 0));
                    if let Ok(frame) = Frame::new_contiguous(frame_count, 0) {
                        // info!(
                        //     "new contiguous success!!! start_paddr:0x{:x}",
                        //     frame.start_paddr()
                        // );
                        return Ok(frame);
                    }
                }
            } else {
                return Err(hv_err!(ENOMEM));
            }
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

    pub fn new_16() -> HvResult<Self> {
        let mut v: Vec<Frame> = Vec::new();
        loop {
            let f = Self::new_zero()?;
            if f.start_paddr & 0b11_1111_1111_1111 == 0 {
                v.push(f);
                break;
            }
            v.push(f);
        }
        let f_16 = v.pop().unwrap();
        drop(f_16);
        let ret = Self::new_contiguous(4, 0)?;
        drop(v);
        Ok(ret)
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
        self.start_paddr as *const u8
    }

    /// convert to a mutable raw pointer.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.start_paddr as *mut u8
    }

    /// Fill `self` with `byte`.
    pub fn fill(&mut self, byte: u8) {
        let ptr = self.as_mut_ptr();
        for i in 0..self.size() {
            unsafe {
                *ptr.add(i) = byte;
            }
        }
    }

    /// Fill `self` with zero.
    pub fn clear(&mut self) {
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

    pub fn copy_data_from(&mut self, data: &[u8]) {
        let len = data.len();
        assert!(data.len() <= self.size());
        self.as_slice_mut()[..len].copy_from_slice(data);
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
pub fn init() {
    let mem_pool_start = crate::consts::mem_pool_start();
    let mem_pool_end = align_down(crate::consts::hv_end());
    let mem_pool_size = mem_pool_end - mem_pool_start;
    FRAME_ALLOCATOR.lock().init(mem_pool_start, mem_pool_size);

    info!(
        "Frame allocator initialization finished: {:#x?}",
        mem_pool_start..mem_pool_end
    );
}

pub fn test() {
    let mut v: Vec<Frame> = Vec::new();
    for _ in 0..5 {
        let frame = Frame::new().unwrap();
        // println!("{:x?}", frame);
        v.push(frame);
    }
    v.clear();
    for _ in 0..5 {
        let frame = Frame::new().unwrap();
        // println!("{:x?}", frame);
        v.push(frame);
    }
    drop(v);
    info!("frame_allocator_test passed!");
}
