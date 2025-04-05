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

#[cfg(target_arch = "loongarch64")]
pub const MAX_CPU_NUM: usize = 4;
#[cfg(target_arch = "aarch64")]
pub const MAX_CPU_NUM: usize = 32;
#[cfg(target_arch = "riscv64")]
pub const MAX_CPU_NUM: usize = 4;

pub const MAX_ZONE_NUM: usize = 3;

pub static mut NCPU: usize = MAX_CPU_NUM;

pub fn core_end() -> VirtAddr {
    __core_end as _
}

pub fn mem_pool_start() -> VirtAddr {
    core_end() + unsafe {NCPU} * PER_CPU_SIZE
}

pub fn hv_end() -> VirtAddr {
    mem_pool_start() + HV_MEM_POOL_SIZE
}

extern "C" {
    fn __core_end();
}
