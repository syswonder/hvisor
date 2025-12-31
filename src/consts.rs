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

//! This module defines core constants and memory layout for the hypervisor.
//!
//! It contains definitions for:
//! - Memory pool sizes and offsets
//! - Per-CPU data configuration
//! - Architecture-specific event codes
//! - Memory boundary access functions
//! - External assembly symbols for linker interaction

pub use crate::memory::PAGE_SIZE;
use crate::{memory::addr::VirtAddr, platform::BOARD_NCPUS};
use core::arch::global_asm;

/// Size of the hypervisor heap.
pub const HV_HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

/// Size of the hypervisor memory pool used for dynamic allocation.
pub const HV_MEM_POOL_SIZE: usize = 64 * 1024 * 1024; // 64 MiB

/// Size of the per-CPU data area, including stack and CPU-local data.
///
/// This area is allocated for each CPU core and may increase in size during
/// development.
pub const PER_CPU_SIZE: usize = 512 * 1024; // 512 KiB

/// Pointer to the beginning of the per-CPU data array.
///
/// This array starts immediately after the core memory area ends.
pub const PER_CPU_ARRAY_PTR: *mut VirtAddr = __core_end as _;

/// Represents an invalid memory address.
pub const INVALID_ADDRESS: usize = usize::MAX;

/// Maximum number of supported CPU cores.
///
/// This is determined by the board-specific configuration.
pub const MAX_CPU_NUM: usize = BOARD_NCPUS;

/// Maximum number of memory zones supported.
pub const MAX_ZONE_NUM: usize = 4;

/// Maximum number of spin-wait iterations before timing out.
pub const MAX_WAIT_TIMES: usize = 100000000;

/// Returns the address of the hypervisor entry point.
#[allow(dead_code)]
pub fn hv_start() -> VirtAddr {
    skernel as _
}

/// Returns the address marking the end of the core memory area.
///
/// This is the boundary between statically linked code/data
/// and the memory allocated for per-CPU data and memory pool.
pub fn core_end() -> VirtAddr {
    __core_end as _
}

/// Returns the address of the memory pool start.
///
/// The memory pool follows immediately after all per-CPU data areas.
pub fn mem_pool_start() -> VirtAddr {
    core_end() + MAX_CPU_NUM * PER_CPU_SIZE
}

/// Returns the address marking the end of the entire hypervisor memory.
pub fn hv_end() -> VirtAddr {
    mem_pool_start() + HV_MEM_POOL_SIZE
}

/// arch specific constants
pub const IPI_EVENT_CLEAR_INJECT_IRQ: usize = 4;
pub const IPI_EVENT_UPDATE_HART_LINE: usize = 5;
pub const IPI_EVENT_SEND_IPI: usize = 6;

extern "C" {
    /// Entry point of the hypervisor written in assembly.
    ///
    /// This is the first function executed when the hypervisor boots.
    #[allow(dead_code)]
    fn skernel();

    /// Symbol marking the end of the core memory area.
    ///
    /// This is defined by the linker script and represents the boundary
    /// between statically linked code/data and dynamic memory areas.
    #[allow(dead_code)]
    fn __core_end();
}

/// Total size of the extended memory area from the core end to the end of
/// the hypervisor memory.
///
/// This is calculated as (MAX_CPU_NUM * PER_CPU_SIZE) + HV_MEM_POOL_SIZE and
/// represents the total additional memory required beyond the core static area.
#[allow(dead_code)]
pub const HV_EXTENDED_SIZE: usize = MAX_CPU_NUM * PER_CPU_SIZE + HV_MEM_POOL_SIZE;

// Expose HV_EXTENDED_SIZE as a global assembly symbol for linker accessibility.
// This makes it easier for the linker script to directly reference this constant value.
global_asm!(
    ".global HV_EXTENDED_SIZE",
    ".equ HV_EXTENDED_SIZE, {hv_extended_size}",
    hv_extended_size = const HV_EXTENDED_SIZE
);
