use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const BOARD_NAME: &str = "qemu-aia";

pub const PLIC_BASE: usize = 0xc000000;
pub const APLIC_BASE: usize = 0xc000000;
pub const PLIC_MAX_IRQ: usize = 1024;
pub const PLIC_GLOBAL_SIZE: usize = 0x200000;
pub const PLIC_TOTAL_SIZE: usize = 0x400000;
pub const PLIC_MAX_CONTEXT: usize = 64;
pub const PLIC_PRIORITY_BASE: usize = 0x0000;
pub const PLIC_PENDING_BASE: usize = 0x1000;
pub const PLIC_ENABLE_BASE: usize = 0x2000;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x8f000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x90000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x90000000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x83000000,
        virtual_start: 0x83000000,
        size: 0x1D000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10000000,
        virtual_start: 0x10000000,
        size: 0x1000,
    }, // serial
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30000000,
        virtual_start: 0x30000000,
        size: 0x10000000,
    }, // pci
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10001000,
        virtual_start: 0x10001000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10002000,
        virtual_start: 0x10002000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10003000,
        virtual_start: 0x10003000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10004000,
        virtual_start: 0x10004000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10005000,
        virtual_start: 0x10005000,
        size: 0x1000,
    }, // virtio
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x10008000,
        virtual_start: 0x10008000,
        size: 0x1000,
    }, // virtio
];

pub const ROOT_ZONE_IRQS: [u32; 11] = [1, 2, 3, 4, 5, 8, 10, 33, 34, 35, 36]; // ARCH= riscv .It doesn't matter temporarily.

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    plic_base: 0xc000000,
    plic_size: 0x4000000,
    aplic_base: 0xd000000,
    aplic_size: 0x8000,
};
