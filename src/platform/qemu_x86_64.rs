use crate::{
    arch::zone::HvArchZoneConfig,
    config::*,
    memory::{GuestPhysAddr, HostPhysAddr},
};

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x00000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x8000; // 0x10_0000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x500_0000; // 0x500_0000;
pub const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xd000;
pub const ROOT_ZONE_BOOT_STACK: GuestPhysAddr = 0x7000;
pub const ROOT_ZONE_INITRD_ADDR: GuestPhysAddr = 0x1500_0000;
pub const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0xc000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0) | (1 << 1) | (1 << 2);

pub const ROOT_ZONE_NAME: &str = "root-linux";
pub const ROOT_ZONE_CMDLINE: &str = "console=ttyS0 earlyprintk=serial rdinit=/init nokaslr\0"; // noapic

pub const MEM_TYPE_ROM: u32 = 3;

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 3] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x500_0000,
        virtual_start: 0x0,
        size: 0x1500_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2020_0000,
        virtual_start: 0x1520_0000,
        size: 0x4000_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_ROM,
        physical_start: 0x2000_0000,
        virtual_start: 0x1500_0000,
        size: 0x20_0000,
    }, // initrd
];

pub const ROOT_ZONE_IRQS: [u32; 32] = [0; 32];
pub const ROOT_IOAPIC_BASE: usize = 0xfec0_0000;
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {
    ioapic_base: ROOT_IOAPIC_BASE,
    ioapic_size: 0x1000,
};

// TODO: temp
pub const GUEST_PT1: GuestPhysAddr = 0x1000;
pub const GUEST_PT2: GuestPhysAddr = 0x2000;
pub const GUEST_ENTRY: GuestPhysAddr = 0x10_0000;
pub const GUEST_STACK_TOP: GuestPhysAddr = 0x7000;
pub const GUEST_PHYS_MEMORY_START: HostPhysAddr = 0x100_0000;

pub fn gpa_as_mut_ptr(guest_paddr: GuestPhysAddr) -> *mut u8 {
    let offset = ROOT_ZONE_KERNEL_ADDR as usize;
    let host_vaddr = guest_paddr + offset;
    host_vaddr as *mut u8
}

#[naked]
pub unsafe extern "C" fn test_guest() -> ! {
    core::arch::asm!(
        "
        mov     rax, 0
        mov     rdi, 2
        mov     rsi, 3
        mov     rdx, 3
        mov     rcx, 3
    2:
        vmcall
        add     rax, 1
        jmp     2b",
        options(noreturn),
    );
}

pub unsafe extern "C" fn test_guest_2() -> ! {
    core::arch::asm!(
        "vmcall",
        inout("rax") 0 => _,
        in("rdi") 2,
        in("rsi") 3,
        in("rdx") 3,
        in("rcx") 3,
    );
    core::arch::asm!("mov qword ptr [$0xffff233], $2333"); // panic
    loop {}
}
