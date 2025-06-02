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
use crate::{
    arch::zone::HvArchZoneConfig,
    config::*,
    memory::{GuestPhysAddr, HostPhysAddr},
};

pub const BOARD_NAME: &str = "qemu";

pub const BOARD_NCPUS: usize = 4;

pub const ROOT_ZONE_DTB_ADDR: u64 = 0x00000000;
pub const ROOT_ZONE_ENTRY: u64 = 0x8000; // 0x10_0000;
pub const ROOT_ZONE_KERNEL_ADDR: u64 = 0x500_0000; // 0x500_0000;
pub const ROOT_ZONE_SETUP_ADDR: GuestPhysAddr = 0xd000;
pub const ROOT_ZONE_BOOT_STACK: GuestPhysAddr = 0x7000;
pub const ROOT_ZONE_INITRD_ADDR: GuestPhysAddr = 0x1500_0000;
pub const ROOT_ZONE_CMDLINE_ADDR: GuestPhysAddr = 0xc000;
pub const ROOT_ZONE_CPUS: u64 = (1 << 0);

pub const ROOT_ZONE_NAME: &str = "root-linux";

pub const ROOT_ZONE_MEMORY_REGIONS: [HvConfigMemoryRegion; 7] = [
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x500_0000,
        virtual_start: 0x0,
        size: 0x1_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x501_0000,
        virtual_start: 0x1_0000,
        size: 0x14ff_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2020_0000,
        virtual_start: 0x1520_0000,
        size: 0x4000_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_RAM,
        physical_start: 0x2000_0000,
        virtual_start: 0x1500_0000,
        size: 0x20_0000,
    }, // ram
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfec0_0000,
        virtual_start: 0xfec0_0000,
        size: 0x1000,
    }, // io apic
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfed0_0000,
        virtual_start: 0xfed0_0000,
        size: 0x1000,
    }, // hpet
    HvConfigMemoryRegion {
        mem_type: MEM_TYPE_IO,
        physical_start: 0xfee0_0000,
        virtual_start: 0xfee0_0000,
        size: 0x1000,
    }, // local apic
];

pub const ROOT_ZONE_IRQS: [u32; 32] = [0; 32];
pub const ROOT_ARCH_ZONE_CONFIG: HvArchZoneConfig = HvArchZoneConfig {};

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