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
use crate::memory::addr::VirtAddr;
pub use crate::memory::PAGE_SIZE;

/// Size of the hypervisor heap.
pub const HV_HEAP_SIZE: usize = 1024 * 1024; // 1 MB
pub const HV_MEM_POOL_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: usize = 512 * 1024; // 512KB  //may get bigger when dev

/// Pointer of the per-CPU data array.
pub const PER_CPU_ARRAY_PTR: *mut VirtAddr = __core_end as _;

pub const INVALID_ADDRESS: usize = usize::MAX;

pub const MAX_CPU_NUM: usize = 32;

pub const MAX_ZONE_NUM: usize = 3;

pub static mut NCPU: usize = MAX_CPU_NUM;

pub const MAX_WAIT_TIMES: usize = 10000000;

pub fn core_end() -> VirtAddr {
    __core_end as _
}

pub fn mem_pool_start() -> VirtAddr {
    core_end() + unsafe { NCPU } * PER_CPU_SIZE
}

pub fn hv_end() -> VirtAddr {
    mem_pool_start() + HV_MEM_POOL_SIZE
}

extern "C" {
    fn __core_end();
}
