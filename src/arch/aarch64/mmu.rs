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
#![allow(unused)]

use core::ptr::addr_of;
use core::{mem, panic};

use aarch64_cpu::registers::CSSELR_EL1::Level::L1;
use aarch64_cpu::registers::PAR_EL1::PA;
use cfg_if::cfg_if;
use cortex_a::registers::SCTLR_EL2;
use tock_registers::interfaces::*;
use tock_registers::*;

use crate::arch::mmu::S1PageAndBlockDescriptor::VALID;
use crate::memory::addr::is_aligned;
use crate::memory::AlignedPage;

register_bitfields! {u64,
    S1PageAndBlockDescriptor [
        RES     OFFSET(55) NUMBITS(4) [],
        UXN     OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        PXN     OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        OAB     OFFSET(12) NUMBITS(36) [],
        NLTA    OFFSET(12) NUMBITS(36) [],
        AF      OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        SH      OFFSET(8) NUMBITS(2) [
            NonShareable = 0b00,
            Reserved = 0b01,
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],
        AP      OFFSET(6) NUMBITS(2) [
            PrivRW = 0b00,
            PrivRW_UnprivRW = 0b01,
            PrivRO = 0b10,
            PrivRO_UnprivRO = 0b11
        ],
        AttrIndx    OFFSET(2) NUMBITS(3) [
            Attr0 = 0b000,
            Attr1 = 0b001,
            Attr2 = 0b010
        ],
        TYPE    OFFSET(1) NUMBITS(1) [
            Block = 0,
            Page = 1
        ],
        VALID   OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

const PAGE_SIZE: usize = 4096;
const PAGE_SHIFT: usize = 12;

const WORD_SIZE: usize = 8;
const ENTRY_PER_PAGE: usize = PAGE_SIZE / WORD_SIZE;

#[derive(PartialEq, Clone, Copy)]
pub enum MemoryType {
    Normal,
    Device,
    Null,
}

enum PTEType {
    Block,
    Page,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct PTEDescriptor(u64);

impl PTEDescriptor {
    fn new(addr: usize, mem_type: MemoryType, pte_type: PTEType) -> PTEDescriptor {
        match pte_type {
            PTEType::Block => PTEDescriptor(
                (S1PageAndBlockDescriptor::OAB.val((addr >> PAGE_SHIFT) as u64)
                    + S1PageAndBlockDescriptor::AF::True
                    + S1PageAndBlockDescriptor::AP::PrivRW
                    + S1PageAndBlockDescriptor::TYPE::Block
                    + S1PageAndBlockDescriptor::VALID::True
                    + match mem_type {
                        MemoryType::Device => {
                            S1PageAndBlockDescriptor::AttrIndx::Attr0
                                + S1PageAndBlockDescriptor::SH::OuterShareable
                        }
                        MemoryType::Normal => {
                            S1PageAndBlockDescriptor::AttrIndx::Attr1
                                + S1PageAndBlockDescriptor::SH::InnerShareable
                        }
                        _ => {
                            S1PageAndBlockDescriptor::AttrIndx::Attr0
                                + S1PageAndBlockDescriptor::SH::OuterShareable
                        }
                    })
                .value,
            ),
            PTEType::Page => PTEDescriptor(
                (S1PageAndBlockDescriptor::NLTA.val((addr >> PAGE_SHIFT) as u64)
                    + S1PageAndBlockDescriptor::VALID::True
                    + S1PageAndBlockDescriptor::TYPE::Page)
                    .value,
            ),
        }
    }

    const fn invalid() -> PTEDescriptor {
        PTEDescriptor(0)
    }
}

const MAX_PT_L1_PAGES: usize = 16;

#[repr(C)]
#[repr(align(4096))]
struct BootPTPagePool([u8; MAX_PT_L1_PAGES * PAGE_SIZE]);

static mut BOOT_PT_L1_PAGE_POOL: BootPTPagePool = BootPTPagePool([0; MAX_PT_L1_PAGES * PAGE_SIZE]);
static mut BOOT_PT_INDEX: usize = 0;

const L1_SHIFT: u64 = 21;
const L0_SHIFT: u64 = 30;

const L1_SZ: u64 = 1 << L1_SHIFT;
const L0_SZ: u64 = 1 << L0_SHIFT;

#[repr(C)]
#[repr(align(4096))]
struct PageTables {
    entry: [PTEDescriptor; ENTRY_PER_PAGE],
}

static mut BOOT_PT_L0: PageTables = PageTables {
    entry: [PTEDescriptor(0); ENTRY_PER_PAGE],
};

fn check_list(list: &[(u64, u64, MemoryType)]) {
    unsafe { assert!(is_aligned(addr_of!(BOOT_PT_L1_PAGE_POOL.0) as _)) };
    assert!(MAX_PT_L1_PAGES > 0);

    for i in 0..list.len() {
        // each addr should align to 2M
        if list[i].0 % L1_SZ != 0 || list[i].1 % L1_SZ != 0 {
            panic!("memory list addr not align to 2M at index {}", i);
        }
        if i < list.len() - 1 && list[i].1 > list[i + 1].0 {
            panic!("memory list addr not sorted at index {}", i);
        }
    }
}

unsafe fn map_l1_page(addr: usize, l0_index: usize, l1_index: usize, mem_type: MemoryType) {
    let l0_entry = &mut BOOT_PT_L0.entry[l0_index];
    let l1_pt_addr = if (l0_entry.0 & 0x1 == 0) {
        // l1 page table not exist, create it
        if BOOT_PT_INDEX >= MAX_PT_L1_PAGES {
            panic!("boot pt page pool is full");
        }
        let l1_pt_addr = BOOT_PT_L1_PAGE_POOL
            .0
            .as_ptr()
            .add(BOOT_PT_INDEX * PAGE_SIZE) as u64;
        BOOT_PT_INDEX += 1;
        *l0_entry = PTEDescriptor::new(l1_pt_addr as _, MemoryType::Null, PTEType::Page);
        l1_pt_addr
    } else {
        l0_entry.0 & !(PAGE_SIZE as u64 - 1)
    };
    assert!(is_aligned(l1_pt_addr as _));
    (*(l1_pt_addr as *mut PageTables)).entry[l1_index] =
        PTEDescriptor::new(addr, mem_type, PTEType::Block);
}

unsafe fn map_l0_page(addr: usize, l0_index: usize, mem_type: MemoryType) {
    assert!(addr as u64 % L0_SZ == 0);
    let l0_entry = &mut BOOT_PT_L0.entry[l0_index];
    *l0_entry = PTEDescriptor::new(addr, mem_type, PTEType::Block);
}

fn map_range(mut start: u64, end: u64, mem_type: MemoryType) {
    assert!(start <= end, "Start address is greater than end address");
    assert!(start % L1_SZ == 0, "Start address is not align to 2M");
    assert!(end % L1_SZ == 0, "End address is not align to 2M");

    while (start < end) {
        let l0_index = start >> L0_SHIFT;
        let l1_index = (start >> L1_SHIFT) & (ENTRY_PER_PAGE as u64 - 1);
        if (l0_index >= ENTRY_PER_PAGE as _) || (l1_index >= ENTRY_PER_PAGE as _) {
            panic!("l0_index or l1_index out of range");
        }

        if (l1_index == 0 && start + L0_SZ <= end) {
            // we can directly map l0 page table here
            unsafe {
                map_l0_page(start as _, l0_index as _, mem_type);
            }
            start += L0_SZ;
        } else {
            unsafe {
                map_l1_page(start as _, l0_index as _, l1_index as _, mem_type);
            }
            start += L1_SZ;
        }
    }
}

fn map_list(list: &[(u64, u64, MemoryType)]) {
    let mut i = 0;

    while i < list.len() {
        let (start, mut end, mem_type) = list[i];
        while i + 1 < list.len() && list[i + 1].0 == end && list[i + 1].2 == mem_type {
            end = list[i].1;
            i += 1;
        }
        // map from start to end with mem_type
        map_range(start, end, mem_type);
        i += 1;
    }
}

pub extern "C" fn boot_pt_init() {
    let phys_memlist = &crate::platform::BOARD_PHYSMEM_LIST;
    check_list(phys_memlist);
    map_list(phys_memlist);
}

pub extern "C" fn mmu_enable() {
    use cortex_a::registers::*;
    MAIR_EL2.write(
        MAIR_EL2::Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + MAIR_EL2::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL2::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL2::Attr2_Normal_Outer::NonCacheable
            + MAIR_EL2::Attr2_Normal_Inner::NonCacheable,
    );

    TTBR0_EL2.set(unsafe { &BOOT_PT_L0.entry } as *const _ as u64);

    TCR_EL2.write(
        TCR_EL2::PS::Bits_48
            + TCR_EL2::SH0::Inner
            + TCR_EL2::TG0::KiB_4
            + TCR_EL2::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL2::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL2::T0SZ.val(64 - 39),
    );
    SCTLR_EL2.modify(SCTLR_EL2::M::Enable + SCTLR_EL2::C::Cacheable + SCTLR_EL2::I::Cacheable);

    unsafe {
        core::arch::asm!("tlbi alle2");
        core::arch::asm!("dsb nsh");
        core::arch::asm!("isb");
    }
}
