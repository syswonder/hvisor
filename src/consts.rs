use crate::config::HvSystemConfig;
use crate::memory::addr::{align_up, VirtAddr};
pub use crate::memory::PAGE_SIZE;

/// Size of the hypervisor heap.
pub const HV_HEAP_SIZE: usize = 1024 * 1024; // 1 MB

/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: usize = 64 * 1024; // 64KB  //may get bigger when dev

/// Size of the per cpu boot stack
pub const PER_CPU_BOOT_SIZE: usize = 1024; // 1KB
/// Start virtual address of the hypervisor memory.
pub const HV_BASE: usize = 0xffffc0200000;

/// Pointer of the per-CPU data array.
pub const PER_CPU_ARRAY_PTR: *mut VirtAddr = __core_end as _;

/// Pointer of trampoline start.
pub const TRAMPOLINE_START: *mut VirtAddr = __trampoline_start as _;

pub const INVALID_ADDRESS: u64 = u64::MAX;

pub const MAX_CPU_NUM: u64 = 16;

extern "C" {
    fn __root_config();
}

/// Pointer of the `HvSystemConfig` structure.
pub fn hv_config_ptr() -> *const HvSystemConfig {
    __root_config as _
    // (PER_CPU_ARRAY_PTR as usize + HvHeader::get().max_cpus as usize * PER_CPU_SIZE) as _
}

/// Pointer of the free memory pool.
pub fn free_memory_start() -> VirtAddr {
    align_up(hv_config_ptr() as usize + HvSystemConfig::get().size())
}

/// End virtual address of the hypervisor memory.
pub fn hv_end() -> VirtAddr {
    HV_BASE + HvSystemConfig::get().hypervisor_memory.size as usize
}

extern "C" {
    fn __core_end();
    fn __trampoline_start();
}
