use crate::{
    arch::zone::HvArchZoneConfig,
    config::*,
    memory::{GuestPhysAddr, GuestVirtAddr, HostPhysAddr},
};

pub const MEM_TYPE_ROM: u32 = 3;
pub const MEM_TYPE_RAM_NOT_ALLOC: u32 = 4;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x00000000;
pub const ROOT_ZONE_BOOT_STACK: GuestPhysAddr = 0x7000;
pub const ROOT_ZONE_ENTRY: u64 = 0x8000;
pub const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0xc000;
pub const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xd000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x500_0000;
pub const ROOT_ZONE_INITRD_ADDR: GuestPhysAddr = 0x1500_0000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2);

pub const ROOT_ZONE_RSDP_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_ROM,
    physical_start: 0x50e_0000,
    virtual_start: 0xe_0000,
    size: 0x2_0000,
};

pub const ROOT_ZONE_ACPI_REGION: HvConfigMemoryRegion = HvConfigMemoryRegion {
    mem_type: MEM_TYPE_RAM_NOT_ALLOC,
    physical_start: 0x6020_0000, // hpa
    virtual_start: 0x5520_0000,  // gpa
    size: 0xf000,                // modify size accordingly
};

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_CMDLINE: &str =
    "console=ttyS0 earlyprintk=serial nointremap root=/dev/vda rw init=/bin/sh\0";
//"console=ttyS0 earlyprintk=serial rdinit=/init nokaslr nointremap\0"; // noapic

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 9] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x500_0000,
        virtual_start: 0x0,
        size: 0xe_0000,
    }, // ram
    ROOT_ZONE_RSDP_REGION, // rsdp
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x510_0000,
        virtual_start: 0x10_0000,
        size: 0x14f0_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_ROM,
        physical_start: 0x2000_0000,
        virtual_start: 0x1500_0000,
        size: 0x20_0000,
    }, // initrd
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2020_0000,
        virtual_start: 0x1520_0000,
        size: 0x4000_0000,
    }, // ram
    ROOT_ZONE_ACPI_REGION, // acpi
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfed0_0000,
        virtual_start: 0xfed0_0000,
        size: 0x1000,
    }, // hpet
    /*HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xb000_0000,
        virtual_start: 0xb000_0000,
        size: 0x1000_0000,
    }, // TODO: pci config*/
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfea0_0000,
        virtual_start: 0xfea0_0000,
        size: 0x20_0000,
    }, // TODO: pci
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0x70_0000_0000,
        virtual_start: 0x70_0000_0000,
        size: 0x1000_4000,
    }, // FIXME: pci 0000:00:03.0
];

pub const ROOT_ZONE_IRQS: [u32; 32] = [0; 32];
pub const ROOT_ZONE_IOAPIC_BASE: usize = 0xfec0_0000;
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    ioapic_base: ROOT_ZONE_IOAPIC_BASE,
    ioapic_size: 0x1000,
};

pub fn root_zone_gpa_as_mut_ptr(guest_paddr: GuestPhysAddr) -> *mut u8 {
    let offset = ROOT_ZONE_KERNEL_ADDR as usize;
    let host_vaddr = guest_paddr + offset;
    host_vaddr as *mut u8
}
