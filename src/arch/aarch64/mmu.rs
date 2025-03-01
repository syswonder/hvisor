#![allow(unused)]

use cfg_if::cfg_if;
use cortex_a::registers::{MAIR_EL1, SCTLR_EL2};
use tock_registers::interfaces::*;
use tock_registers::*;

register_bitfields! {u64,
    pub S1PageAndBlockDescriptor [
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

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

pub const ENTRY_PER_PAGE: usize = PAGE_SIZE / 8;

pub const WORD_SIZE: usize = 8;
pub const PTE_PER_PAGE: usize = PAGE_SIZE / WORD_SIZE;

enum MemoryType {
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

#[repr(C)]
#[repr(align(4096))]
pub struct PageTables {
    entry: [PTEDescriptor; ENTRY_PER_PAGE],
}

// l1 page table
pub static BOOT_PT_L0: PageTables = PageTables {
    entry: [PTEDescriptor(0); ENTRY_PER_PAGE],
};

// l2 page table
pub static BOOT_PT_L1: PageTables = PageTables {
    entry: [PTEDescriptor(0); ENTRY_PER_PAGE],
};

//TODO: use memset from crate
pub unsafe fn memset(s: *mut u8, c: i32, n: usize) {
    if (s as usize) < 0x1000 {
        panic!("illegal addr for memset s {:x}", s as usize);
    }
    core::ptr::write_bytes(s, c as u8, n);
}

// #[link_section = ".text.boot"]
pub extern "C" fn boot_pt_init(l0_pt: &mut PageTables, l1_pt: &mut PageTables) {
    let l0_pt_entry: usize = l0_pt as *const _ as usize;
    let l1_pt_entry: usize = l1_pt as *const _ as usize;

    unsafe {
        memset(l0_pt_entry as *mut u8, 0, PAGE_SIZE);
        memset(l1_pt_entry as *mut u8, 0, PAGE_SIZE);
    }
    cfg_if! {
        if #[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))] {
            l0_pt.entry[0] = PTEDescriptor::new(0x0, MemoryType::Device, PTEType::Block);
            for i in 1..ENTRY_PER_PAGE {
                l0_pt.entry[i] = PTEDescriptor::new(0x40000000*i, MemoryType::Normal, PTEType::Block);
            }
        } else {
            l0_pt.entry[0] = PTEDescriptor::new(0x0, MemoryType::Device, PTEType::Block);
            for i in 1..7 {
                l0_pt.entry[i] = PTEDescriptor::new(0x40000000*i, MemoryType::Normal, PTEType::Block);
            }
            for i in 8..ENTRY_PER_PAGE {
                l0_pt.entry[i] = PTEDescriptor::invalid();
            }
        }
    }
}

// init mmu
// #[link_section = ".text.boot"]
#[no_mangle]
pub extern "C" fn mmu_init(pt: &PageTables) {
    use cortex_a::registers::*;
    MAIR_EL2.write(
        MAIR_EL2::Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + MAIR_EL2::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL2::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL2::Attr2_Normal_Outer::NonCacheable
            + MAIR_EL2::Attr2_Normal_Inner::NonCacheable,
    );
    TTBR0_EL2.set(&pt.entry as *const _ as u64);

    TCR_EL2.write(
        TCR_EL2::PS::Bits_48
            + TCR_EL2::SH0::Inner
            + TCR_EL2::TG0::KiB_4
            + TCR_EL2::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL2::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL2::T0SZ.val(64 - 39),
    );
}

// #[link_section = ".text.boot"]
pub extern "C" fn mmu_enable() {
    SCTLR_EL2.modify(SCTLR_EL2::M::Enable + SCTLR_EL2::C::Cacheable + SCTLR_EL2::I::Cacheable);
}
