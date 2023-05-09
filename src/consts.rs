use crate::header::HvHeader;
use crate::memory::addr::VirtAddr;
pub use crate::memory::PAGE_SIZE;
/// Size of the hypervisor heap.
pub const HV_HEAP_SIZE: usize = 32 * 1024 * 1024; // 32 MB

/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: usize = 512 * 1024; // 512 KB

/// Start virtual address of the hypervisor memory.
pub const HV_BASE: usize = 0xffffc0200000;

/// Pointer of the `HvHeader` structure.
pub const HV_HEADER_PTR: *const HvHeader = __header_start as _;

/// Pointer of the per-CPU data array.
pub const PER_CPU_ARRAY_PTR: *mut VirtAddr = __core_end as _;

extern "C" {
    fn __header_start();
    fn __core_end();
}
