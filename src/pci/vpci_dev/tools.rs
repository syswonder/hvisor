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

use core::{marker::PhantomData, mem::size_of};

#[derive(Clone, Copy, Debug)]
pub struct GuestMemory {
    ptr: usize,
    size: usize,
}

impl GuestMemory {
    pub const fn dummy() -> Self {
        Self { ptr: 0, size: 0 }
    }

    pub fn new(ptr: usize, size: usize) -> Self {
        Self { ptr, size }
    }

    pub fn set_ptr(&mut self, ptr: usize) {
        self.ptr = ptr;
    }

    pub fn set_len(&mut self, len: usize) {
        self.size = len;
    }

    pub fn read_obj<T: Copy>(&self, offset: u64) -> T {
        // warn!("read from :0x{:x},offset:{:x}",self.ptr,offset);
        let obj_ptr = self.translate(offset) as *const T;
        unsafe { core::ptr::read_volatile(obj_ptr) }
    }

    pub fn write_obj<T: Copy>(&self, offset: u64, val: T) {
        // warn!("write into :0x{:x},offset:{:x}",self.ptr,offset);
        let obj_ptr = self.translate(offset) as *mut T;
        unsafe {
            core::ptr::write_unaligned(obj_ptr, val);
        }
    }

    fn translate(&self, offset: u64) -> *mut u8 {
        assert!(offset < self.size as u64);
        unsafe {
            let ptr = self.ptr as *mut u8;
            ptr.add(offset as usize)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GuestSlice<T> {
    mem: GuestMemory,
    // base: u64,
    len: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: Copy> GuestSlice<T> {
    pub const fn dummy() -> Self {
        Self {
            mem: GuestMemory::dummy(),
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub fn new(base: usize, len: usize) -> Self {
        let memory_size = len * core::mem::size_of::<T>();
        let mem = GuestMemory::new(base, memory_size);
        Self {
            mem,
            len,
            _phantom: PhantomData,
        }
    }

    pub fn set_ptr(&mut self, ptr: usize) {
        self.mem.set_ptr(ptr);
    }

    pub fn set_memory(&mut self, ptr: usize, len: usize) {
        self.mem.set_ptr(ptr);
        self.mem.set_len(len * size_of::<T>());
        self.len = len;
    }

    pub fn get_addr(&self) -> usize {
        self.mem.ptr
    }

    pub fn get(&self, index: usize) -> T {
        assert!(index < self.len);

        let offset = index * core::mem::size_of::<T>();
        self.mem.read_obj::<T>(offset as u64)
    }

    pub fn set(&self, index: usize, val: T) {
        // info!("current len:{:x},index:{:x}",self.mem.size,index);
        assert!(index < self.len);

        let offset = index * core::mem::size_of::<T>();
        self.mem.write_obj(offset as u64, val);
    }
}
