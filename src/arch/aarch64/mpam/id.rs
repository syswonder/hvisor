use alloc::vec::Vec;
use spin::{Mutex, Once};

static PARTID_ALLOCATOR: Once<Mutex<IdAllocator>> = Once::new();

struct IdAllocator {
    next: usize,
    free_list: Vec<usize>,
    capacity: usize,
}

impl IdAllocator {
    pub fn new(capacity: usize) -> Self {
        Self {
            // 0 for hvisor
            next: 1,
            free_list: Vec::new(),
            capacity,
        }
    }
    pub fn alloc(&mut self) -> Option<usize> {
        if let Some(id) = self.free_list.pop() {
            return Some(id);
        }
        if self.next < self.capacity {
            let id = self.next;
            self.next += 1;
            return Some(id);
        }
        None
    }
    pub fn dealloc(&mut self, id: usize) {
        assert!(
            id < self.capacity,
            "dealloc id {} out of capacity {}",
            id,
            self.capacity
        );
        self.free_list.push(id);
    }
}

pub fn init_partid_allocator(capacity: usize) {
    PARTID_ALLOCATOR.call_once(|| Mutex::new(IdAllocator::new(capacity)));
}

pub fn alloc_partid() -> Option<usize> {
    let allocator = PARTID_ALLOCATOR
        .get()
        .expect("PARTID_ALLOCATOR not initialized");
    let mut guard = allocator.lock();
    guard.alloc()
}

pub fn dealloc_partid(id: usize) {
    let allocator = PARTID_ALLOCATOR
        .get()
        .expect("PARTID_ALLOCATOR not initialized");
    let mut guard = allocator.lock();
    guard.dealloc(id);
}
