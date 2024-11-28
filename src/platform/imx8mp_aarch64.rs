use crate::{arch::zone::HvArchZoneConfig, config::*};
// gpu on non root linux
pub const ROOT_ZONE_DTB_ADDR: u64 = 0xa0000000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0xa0400000;
pub const ROOT_ZONE_ENTRY: u64 = 0xa0400000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 6] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x50000000,
        virtual_start: 0x50000000,
        size: 0x80000000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30000000,
        virtual_start: 0x30000000,
        size: 0x400000,
    }, // bus@30000000
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x30800000,
        virtual_start: 0x30800000,
        size: 0x400000,
    }, 
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32f00000,
        virtual_start: 0x32f00000,
        size: 0x10000,
    }, // pcie phy
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x33800000,
        virtual_start: 0x33800000,
        size: 0x400000,
    }, // pcie dbi
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x32f10000,
        virtual_start: 0x32f10000,
        size: 0x1000,
    }, // iomuxc
    // bus@30800000
       // HvConfigMemoryRegion {
       //     mem_type: MEM_TYPE_IO,
       //     physical_start: 0x30890000,
       //     virtual_start: 0x30890000,
       //     size: 0x1000,
       // }, // serial
];

pub const ROOT_ZONE_IRQS: [u32; 26] = [
    36, 52, 55, 59, 64, 65, 67, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 150, 151, 152, 155, 156, 157, 158, 172, 159
];

pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    gicd_base: 0x38800000,
    gicd_size: 0x10000,
    gicr_base: 0x38880000,
    gicr_size: 0xc0000,
    gits_base: 0,
    gits_size: 0,
};

pub const ROOT_ZONE_IVC_CONFIG: [HvIvcConfig; 1] = [
    HvIvcConfig {
        ivc_id: 0,
        peer_id: 0,
        control_table_ipa: 0xd000_0000,
        shared_mem_ipa: 0xd000_1000,
        rw_sec_size: 0,
        out_sec_size: 0x1000,
        interrupt_num: 0x21 + 32,
        max_peers: 2,
    }
];

pub const ROOT_PCI_CONFIG: HvPciConfig = HvPciConfig {
    ecam_base: 0x1ff00000,
    ecam_size: 0x80000,
    io_base: 0x1ff80000,
    io_size: 0x10000,
    pci_io_base: 0x0,
    mem32_base: 0x0,
    mem32_size: 0x0,
    pci_mem32_base: 0x0,
    mem64_base: 0x18000000,
    mem64_size: 0x7f00000,
    pci_mem64_base: 0x18000000,
};

pub const ROOT_PCI_DEVS: [u64; 2] = [0, 1 << 8];