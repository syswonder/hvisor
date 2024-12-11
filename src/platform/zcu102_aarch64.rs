use crate::{arch::zone::HvArchZoneConfig, config::*};

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x100000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x200000;
pub const ROOT_ZONE_ENTRY: u64 = 0x200000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 3] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x0,
        virtual_start: 0x0,
        size: 0x8000_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff00_0000,
        virtual_start: 0xff00_0000,
        size: 0x1_0000,
    }, // uart0
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xff01_0000,
        virtual_start: 0xff01_0000,
        size: 0x1_0000,
    }, // uart1
];

pub const ROOT_ZONE_IRQS: [u32; 1] = [0];

// need port to gicv2 on ZCU102
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0x8000000,
    gicd_size: 0x10000,
    gicr_base: 0x80a0000,
    gicr_size: 0xf60000,
    gits_base: 0x8080000,
    gits_size: 0x20000,
};

pub const ROOT_PCI_CONFIG: HvPciConfig = HvPciConfig {
    ecam_base: 0x4010000000,
    ecam_size: 0x10000000,
    io_base: 0x3eff0000,
    io_size: 0x10000,
    pci_io_base: 0x0,
    mem32_base: 0x10000000,
    mem32_size: 0x2eff0000,
    pci_mem32_base: 0x10000000,
    mem64_base: 0x8000000000,
    mem64_size: 0x8000000000,
    pci_mem64_base: 0x8000000000,
};

pub const ROOT_PCI_DEVS: [u64; 1] = [0];

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 0] = [];