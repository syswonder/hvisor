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

use core::mem::size_of;

use crate::pci::vpci_dev::tools::{GuestMemory, GuestSlice};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

pub type DescriptorTable = GuestSlice<VirtqDesc>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AvailRing {
    mem: GuestMemory,
    // base: u64,
    queue_size: u16,
}

impl AvailRing {
    pub fn new(base: usize, queue_size: u16) -> Self {
        let memory_size = 4 + queue_size as usize * 2;
        let mem = GuestMemory::new(base, memory_size);
        Self { mem, queue_size }
    }

    pub fn set_ptr(&mut self, ptr: usize) {
        self.mem.set_ptr(ptr);
    }

    pub fn get_idx(&self) -> u16 {
        self.mem.read_obj::<u16>(2)
    }

    pub fn get_ring_content(&self, idx: usize) -> u16 {
        let offset = 4 + idx * 2;
        self.mem.read_obj::<u16>(offset as u64)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtqUsedElem {
    pub id: u32,
    pub len: u32,
}

impl VirtqUsedElem {
    pub fn new(id: u32, len: u32) -> Self {
        Self { id, len }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VirtqUsed {
    mem: GuestMemory,
    queue_size: u16,
}

impl VirtqUsed {
    pub fn new(base: usize, queue_size: u16) -> Self {
        let memory_size = 4 + (queue_size as usize) * size_of::<VirtqUsedElem>();
        let mem = GuestMemory::new(base, memory_size);
        Self { mem, queue_size }
    }

    pub fn set_ptr(&mut self, ptr: usize) {
        self.mem.set_ptr(ptr);
    }

    pub fn write_ring(&self, idx: usize, val: VirtqUsedElem) {
        let offset = 4 + idx * size_of::<VirtqUsedElem>();
        self.mem.write_obj(offset as u64, val);
    }

    pub fn get_idx(&self) -> u16 {
        self.mem.read_obj::<u16>(2)
    }

    pub fn set_idx(&self, val: u16) {
        self.mem.write_obj::<u16>(2, val);
    }
}
