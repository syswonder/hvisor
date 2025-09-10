use core::{fmt::Debug, ops::Range};

pub type Mem32Address = u32;
pub type Mem64Address = u64;

trait Algin {
    fn align_up(self, align: Self) -> Self;
}

impl Algin for Mem32Address {
    fn align_up(self, align: Self) -> Self {
        (self + align - 1) & !(align - 1)
    }
}

impl Algin for Mem64Address {
    fn align_up(self, align: Self) -> Self {
        (self + align - 1) & !(align - 1)
    }
}

pub trait BarAllocator: Debug {
    fn alloc_memory32(&mut self, size: Mem32Address) -> Option<Mem32Address>;
    fn alloc_memory64(&mut self, size: Mem64Address) -> Option<Mem64Address>;
}

#[derive(Default, Debug)]
pub struct BaseAllocator {
    mem32: Range<Mem32Address>,
    mem32_used: Mem32Address,
    mem64: Range<Mem64Address>,
    mem64_used: Mem64Address,
}

impl BaseAllocator {
    pub fn set_mem32(&mut self, start: Mem32Address, size: Mem32Address) {
        self.mem32 = start..start + size;
        self.mem32_used = start;
    }

    pub fn set_mem64(&mut self, start: Mem64Address, size: Mem64Address) {
        self.mem64 = start..start + size;
        self.mem64_used = start;
    }
}

impl BarAllocator for BaseAllocator {
    fn alloc_memory32(&mut self, size: Mem32Address) -> Option<Mem32Address> {
        let ptr = self.mem32_used.align_up(size);

        if self.mem32.contains(&ptr) && ptr + size <= self.mem32.end {
            self.mem32_used = ptr + size;
            // debug!("alloc mem32 {:x} {}", ptr, size);
            Some(ptr)
        } else {
            None
        }
    }

    fn alloc_memory64(&mut self, size: Mem64Address) -> Option<Mem64Address> {
        let ptr = self.mem64_used.align_up(size);
        if self.mem64.contains(&ptr) && ptr + size <= self.mem64.end {
            self.mem64_used = ptr + size;
            // debug!("alloc mem64 {:x} {}", ptr, size);
            Some(ptr)
        } else {
            None
        }
    }
}
