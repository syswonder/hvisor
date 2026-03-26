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
//      ForeverYolo <2572131118@qq.com>
//      Jingyu Liu <liujingyu24s@ict.ac.cn>

#![allow(unused)]

use crate::memory::Frame;
use alloc::vec::Vec;
use core::sync::atomic::{fence, Ordering};
use log::{error, info, warn};
use spin::{Once, RwLock};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};

#[inline(always)]
fn io_fence() {
    unsafe {
        core::arch::asm!("fence iorw, iorw", options(nostack, preserves_flags));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum IommuDdtMode {
    Off = 0,
    Bare = 1,
    OneLevel = 2,
    TwoLevel = 3,
    ThreeLevel = 4,
}

// RISC-V IOMMU Spec Chap6.3 IOMMU capabilities
register_bitfields![u64,
    IOMMU_CAPS [
        VERSION OFFSET(0) NUMBITS(8) [
            VERSION_1_0 = 0x10,
        ],
        SV32 OFFSET(8) NUMBITS(1) [],
        SV39 OFFSET(9) NUMBITS(1) [],
        SV48 OFFSET(10) NUMBITS(1) [],
        SV57 OFFSET(11) NUMBITS(1) [],
        SVRSW60T59B OFFSET(14) NUMBITS(1) [],
        SVPBMT OFFSET(15) NUMBITS(1) [],
        SV32X4 OFFSET(16) NUMBITS(1) [],
        SV39X4 OFFSET(17) NUMBITS(1) [],
        SV48X4 OFFSET(18) NUMBITS(1) [],
        SV57X4 OFFSET(19) NUMBITS(1) [],
        AMO_MRIF OFFSET(21) NUMBITS(1) [],
        MSI_FLAT OFFSET(22) NUMBITS(1) [],
        MSI_MRIF OFFSET(23) NUMBITS(1) [],
        AMO_HWAD OFFSET(24) NUMBITS(1) [],
        ATS OFFSET(25) NUMBITS(1) [],
        T2GPA OFFSET(26) NUMBITS(1) [],
        END OFFSET(27) NUMBITS(1) [],
        IGS OFFSET(28) NUMBITS(2) [
            MSI = 0,
            WSI = 1,
            BOTH = 2,
        ],
        HPM OFFSET(30) NUMBITS(1) [],
        DBG OFFSET(31) NUMBITS(1) [],
        PAS OFFSET(32) NUMBITS(6) [],
        PD8 OFFSET(38) NUMBITS(1) [],
        PD17 OFFSET(39) NUMBITS(1) [],
        PD20 OFFSET(40) NUMBITS(1) [],
        QOS OFFSET(41) NUMBITS(1) [],
        NL OFFSET(42) NUMBITS(1) [],
        S OFFSET(43) NUMBITS(1) [],
    ],
    IOMMU_DDTP [
        MODE OFFSET(0) NUMBITS(4) [
            OFF = 0,
            BARE = 1,
            DDT_1LVL = 2,
            DDT_2LVL = 3,
            DDT_3LVL = 4
        ],
        BUSY OFFSET(4) NUMBITS(1) [],
        PPN OFFSET(10) NUMBITS(44) []
    ],
    DDT_TC [
        V OFFSET(0) NUMBITS(1) [],
        EN_PRI OFFSET(4) NUMBITS(1) []
    ],
    DDT_IOHGATP [
        PPN OFFSET(0) NUMBITS(44) [],
        GSCID OFFSET(44) NUMBITS(16) [],
        MODE OFFSET(60) NUMBITS(4) [
            SV39X4 = 8,
            SV48X4 = 9,
            SV57X4 = 10
        ]
    ],
    DDT_FSC [
        MODE OFFSET(0) NUMBITS(4) [
            BARE = 0
        ]
    ],
    DDT_DIR [
        V OFFSET(0) NUMBITS(1) [],
        PPN OFFSET(10) NUMBITS(44) []
    ]
];

register_bitfields![u32,
    IOMMU_FCTL [
        BE OFFSET(0) NUMBITS(1) [],
        WSI OFFSET(1) NUMBITS(1) [],
        GXL OFFSET(2) NUMBITS(1) [],
    ],
    IOMMU_IPSR [
        CIP OFFSET(0) NUMBITS(1) [],
        FIP OFFSET(1) NUMBITS(1) [],
        PMIP OFFSET(2) NUMBITS(1) [],
        PIP OFFSET(3) NUMBITS(1) []
    ]
];

// RISC-V IOMMU Specv1.0: Chap6.1 Register layout
register_structs! {
    #[allow(non_snake_case)]
    IommuHw {
        (0x000 => caps: ReadOnly<u64, IOMMU_CAPS::Register>),
        (0x008 => fctl: ReadWrite<u32, IOMMU_FCTL::Register>),
        (0x00c => _custom1),
        (0x010 => ddtp: ReadWrite<u64, IOMMU_DDTP::Register>),
        (0x018 => cqb: ReadWrite<u64>),
        (0x020 => cqh: ReadWrite<u32>),
        (0x024 => cqt: ReadWrite<u32>),
        (0x028 => fqb: ReadWrite<u64>),
        (0x030 => fqh: ReadWrite<u32>),
        (0x034 => fqt: ReadWrite<u32>),
        (0x038 => pqb: ReadWrite<u64>),
        (0x040 => pqh: ReadWrite<u32>),
        (0x044 => pqt: ReadWrite<u32>),
        (0x048 => cqcsr: ReadWrite<u32>),
        (0x04c => fqcsr: ReadWrite<u32>),
        (0x050 => pqcsr: ReadWrite<u32>),
        (0x054 => ipsr: ReadWrite<u32, IOMMU_IPSR::Register>),
        (0x058 => iocntovf: ReadWrite<u32>),
        (0x05c => iocntinh: ReadWrite<u32>),
        (0x060 => iohpmcycles: ReadWrite<u64>),
        (0x068 => iohpmctr: [ReadWrite<u64>; 31]),
        (0x160 => iohpmevt: [ReadWrite<u64>; 31]),
        (0x258 => tr_req_iova: ReadWrite<u64>),
        (0x260 => tr_req_ctl: ReadWrite<u64>),
        (0x268 => tr_response: ReadOnly<u64>),
        (0x270 => _reserved1),
        (0x2b0 => _custom2),
        (0x2f8 => icvec: ReadWrite<u64>),
        (0x300 => _msi_cfg_tbl),    // TODO: support MSI
        (0x400 => _reserved2),
        (0x1000 => @END),
    }
}

/// Global IOMMU instance
static IOMMU: Once<RwLock<Iommu>> = Once::new();

fn get_iommu<'a>() -> &'a RwLock<Iommu> {
    IOMMU.get().expect("Uninitialized hypervisor iommu!")
}

/// Initialize IOMMU with default mode
pub fn iommu_init() {
    riscv_iommu_init();
}

/// Add a device to IOMMU
pub fn iommu_add_device(vm_id: usize, device_id: usize, root_pt: usize) {
    info!(
        "RV_IOMMU_ADD_DEVICE: root_pt {:#x}, vm_id {}",
        root_pt, vm_id
    );
    let iommu = get_iommu();
    iommu.write().rv_iommu_add_device(device_id, vm_id, root_pt);
}

/// Initialize RISC-V IOMMU with hardware DDTP probing.
fn riscv_iommu_init() {
    assert!(crate::platform::IOMMU_SYS_SIZE == 0x1000, "IOMMU_SYS_SIZE is not 0x1000");
    let iommu = Iommu::new(crate::platform::IOMMU_SYS_BASE);
    IOMMU.call_once(|| RwLock::new(iommu));
    get_iommu().write().rv_iommu_init();
}

impl IommuHw {
    fn try_set_ddtp(&mut self, ddt_addr: usize, mode: IommuDdtMode) -> bool {
        while self.ddtp.is_set(IOMMU_DDTP::BUSY) {}
        self.ddtp.write(
            IOMMU_DDTP::PPN.val((ddt_addr as u64) >> 12) + IOMMU_DDTP::MODE.val(mode as u64),
        );
        // Mode transition completes only when BUSY returns to 0.
        while self.ddtp.is_set(IOMMU_DDTP::BUSY) {}
        self.ddtp.read(IOMMU_DDTP::MODE) == mode as u64
    }

    fn set_ddtp(&mut self, ddt_addr: usize, requested_mode: IommuDdtMode) -> IommuDdtMode {
        let candidates: &[IommuDdtMode] = &[
            IommuDdtMode::ThreeLevel,
            IommuDdtMode::TwoLevel,
            IommuDdtMode::OneLevel,
        ];
        for mode in candidates {
            if self.try_set_ddtp(ddt_addr, *mode) {
                info!("RISC-V IOMMU: DDTP mode set to {:?}", *mode);
                return *mode;
            }
        }
        IommuDdtMode::Off
    }

    fn rv_iommu_check_features(&self) {
        let version = self.caps.read(IOMMU_CAPS::VERSION);
        // Stop and report failure if capabilities.version is not supported.
        if version != IOMMU_CAPS::VERSION::VERSION_1_0.value {
            panic!(
                "RISC-V IOMMU unsupported version: {}, Please check the IOMMU version",
                version
            );
        }
        // Note: here RISCV-IOMMU and CPU share the same stage-2 page table.
        let cpu_s2pt_lvl = unsafe { crate::arch::s2pt::GSTAGE_PT_LEVEL };
        if !self.caps.is_set(IOMMU_CAPS::SV39X4) && cpu_s2pt_lvl == 3 {
            panic!("CPU s2pt is Sv39x4, but IOMMU does not support Sv39x4");
        }
        if !self.caps.is_set(IOMMU_CAPS::SV48X4) && cpu_s2pt_lvl == 4 {
            panic!("CPU s2pt is Sv48x4, but IOMMU does not support Sv48x4");
        }
        if !self.caps.is_set(IOMMU_CAPS::SV57X4) && cpu_s2pt_lvl == 5 {
            panic!("CPU s2pt is Sv57x4, but IOMMU does not support Sv57x4");
        }
        if !self.caps.is_set(IOMMU_CAPS::MSI_FLAT) {
            // Current DDT Entry only supports Extented-for
            todo!("RISC-V IOMMU HW does not support MSI Address Translation (basic-translate mode)");
        }
        if self.caps.read(IOMMU_CAPS::IGS) == IOMMU_CAPS::IGS::MSI.value {
            warn!("RISC-V IOMMU HW does not support WSI generation");
        }
    }

    fn rv_iommu_init(&mut self, ddt_addr: usize, ddt_mode: IommuDdtMode) -> IommuDdtMode {
        // RISC-V IOMMU Spec Chap7.2 Guidelines for initialization
        // Read the capabilities register to discover the capabilities of the IOMMU.
        self.rv_iommu_check_features();
        // Read the feature control register(fctl).
        self.fctl
            .write(IOMMU_FCTL::WSI::SET + IOMMU_FCTL::BE::CLEAR + IOMMU_FCTL::GXL::CLEAR);
        // Clear all IP flags, ipsr is RW1C (Write-1-to-clear status)
        self.ipsr.write(
            IOMMU_IPSR::CIP::SET
                + IOMMU_IPSR::FIP::SET
                + IOMMU_IPSR::PMIP::SET
                + IOMMU_IPSR::PIP::SET,
        );
        // TODO: support MSI
        // TODO: program the command queue
        // TODO: program the fault queue
        // TODO: program the page-request queue

        // Configure ddtp with DDT base address and IOMMU mode
        self.set_ddtp(ddt_addr, ddt_mode)
    }
}

/// Extended-format device-context data structure.
#[repr(C)]
struct DdtEntry {
    tc: ReadWrite<u64, DDT_TC::Register>,
    iohgatp: ReadWrite<u64, DDT_IOHGATP::Register>,
    ta: ReadWrite<u64>,
    fsc: ReadWrite<u64, DDT_FSC::Register>,
    msiptp: ReadWrite<u64>,
    msi_addr_mask: ReadWrite<u64>,
    msi_addr_pattern: ReadWrite<u64>,
    __rsv: ReadWrite<u64>,
}

/// Intermediate device-directory table
#[repr(C)]
struct DdtDirTable {
    entries: [ReadWrite<u64, DDT_DIR::Register>; 512],
}

/// Leaf device-directory table
#[repr(C)]
struct DdtLeafTable {
    dc: [DdtEntry; 64], // 64 = 4KiB / sizeof(DdtEntry)
}

/// Device-directory table
pub struct DdtRootMemory {
    mode: IommuDdtMode,
    root: Frame,
    lower_levels: Vec<Frame>,
}

impl DdtRootMemory {
    const DEV_ID_MAX: usize = (1 << 24) - 1; // device_id supports up to 24 bits

    fn new() -> Self {
        Self {
            mode: IommuDdtMode::Off,
            root: Frame::new_zero().unwrap(),
            lower_levels: Vec::new(),
        }
    }

    fn mode(&self) -> IommuDdtMode {
        self.mode
    }

    fn set_mode(&mut self, mode: IommuDdtMode) {
        self.mode = mode;
    }

    fn root_paddr(&self) -> usize {
        self.root.start_paddr()
    }

    fn alloc_next_level_table(&mut self) -> usize {
        let frame = Frame::new_zero().unwrap();
        let paddr = frame.start_paddr();
        self.lower_levels.push(frame);
        paddr
    }

    fn dir_table_at(paddr: usize) -> &'static mut DdtDirTable {
        unsafe { &mut *(paddr as *mut DdtDirTable) }
    }

    fn leaf_table_at(paddr: usize) -> &'static mut DdtLeafTable {
        unsafe { &mut *(paddr as *mut DdtLeafTable) }
    }

    fn ensure_child_table(&mut self, table_paddr: usize, idx: usize) -> usize {
        let entry = &mut Self::dir_table_at(table_paddr).entries[idx];
        if entry.is_set(DDT_DIR::V) {
            return (entry.read(DDT_DIR::PPN) as usize) << 12;
        }
        let child_paddr = self.alloc_next_level_table();
        entry.write(DDT_DIR::V::SET + DDT_DIR::PPN.val((child_paddr as u64) >> 12));
        child_paddr
    }

    // IOMMU.caps.MSI_FLAT should be 1.
    // Device id split for 3-level DDT (no overlap):
    // [23:15] -> level-1, [14:6] -> level-2, [5:0] -> level-3.
    fn ddt_indices(device_id: usize) -> (usize, usize, usize) {
        let l1 = (device_id >> 15) & 0x1ff;
        let l2 = (device_id >> 6) & 0x1ff;
        let l3 = device_id & 0x3f;
        (l1, l2, l3)
    }

    fn get_or_alloc_leaf_entry(&mut self, device_id: usize) -> Option<&mut DdtEntry> {
        if device_id > Self::DEV_ID_MAX {
            return None;
        }
        let (l1, l2, l3) = Self::ddt_indices(device_id);
        let leaf_table_paddr = match self.mode {
            IommuDdtMode::OneLevel => self.root_paddr(),
            IommuDdtMode::TwoLevel => self.ensure_child_table(self.root_paddr(), l2),
            IommuDdtMode::ThreeLevel => {
                let lvl2_table_paddr = self.ensure_child_table(self.root_paddr(), l1);
                self.ensure_child_table(lvl2_table_paddr, l2)
            }
            _ => return None,
        };
        Some(&mut Self::leaf_table_at(leaf_table_paddr).dc[l3])
    }
}

pub struct Iommu {
    pub base: usize,
    ddt: DdtRootMemory,
}

impl Iommu {
    fn new(base: usize) -> Self {
        Self {
            base,
            ddt: DdtRootMemory::new(),
        }
    }

    fn iommu(&self) -> &mut IommuHw {
        unsafe { &mut *(self.base as *mut _) }
    }

    fn ddt_root_paddr(&self) -> usize {
        self.ddt.root_paddr()
    }

    fn ddt_mode(&self) -> IommuDdtMode {
        self.ddt.mode()
    }

    fn rv_iommu_init(&mut self) {
        // Always probe from the highest practical mode, then fallback by retention.
        let requested_mode = IommuDdtMode::ThreeLevel;
        let selected_mode = self
            .iommu()
            .rv_iommu_init(self.ddt_root_paddr(), requested_mode);
        if selected_mode != requested_mode {
            warn!(
                "RV IOMMU: DDTP mode downgraded from {:?} to {:?}",
                requested_mode, selected_mode
            );
        }
        self.ddt.set_mode(selected_mode);
    }

    fn rv_iommu_add_device(&mut self, device_id: usize, vm_id: usize, root_pt: usize) {
        let Some(entry) = self.ddt.get_or_alloc_leaf_entry(device_id) else {
            warn!(
                "RV IOMMU: Invalid device ID {} for DDT mode {:?}",
                device_id,
                self.ddt_mode()
            );
            return;
        };

        // Prepare TC without publishing VALID yet.
        entry.tc.write(DDT_TC::V::CLEAR + DDT_TC::EN_PRI::SET);

        // Check riscv stage-2 pt, root pt should be 16KiB aligned.
        if root_pt & ((16 * 1024) - 1) != 0 {
            error!(
                "RV IOMMU: iohgatp root page-table is not 16KiB aligned: {:#x}",
                root_pt
            );
        }
        // Configure the stage-2 page table mode same as cpu.
        let iohgatp_mode = match unsafe { crate::arch::s2pt::GSTAGE_PT_LEVEL } {
            3 => DDT_IOHGATP::MODE::SV39X4,
            4 => DDT_IOHGATP::MODE::SV48X4,
            5 => DDT_IOHGATP::MODE::SV57X4,
            _ => panic!("Invalid stage-2 pt level: {}", unsafe { crate::arch::s2pt::GSTAGE_PT_LEVEL }),
        };
        entry.iohgatp.write(
            DDT_IOHGATP::PPN.val((root_pt as u64) >> 12)
                + DDT_IOHGATP::GSCID.val(vm_id as u64)
                + iohgatp_mode
        );
        // Bare first-stage context.
        entry.fsc.write(DDT_FSC::MODE::BARE);
        entry.tc.write(DDT_TC::V::SET);
        info!(
            "RV IOMMU: Write DDT, add decive context, iohgatp.mode = {:#x?}, ioghatp.ppn = {:#x?}",
            entry.iohgatp.read(DDT_IOHGATP::MODE),
            entry.iohgatp.read(DDT_IOHGATP::PPN)
        );
        // "RV IOMMU: DDT entry updated for device {}, remember to issue IODIR/IOTINVAL commands if entry is changed while IOMMU is active"
    }

    fn rv_iommu_remove_device(&mut self, device_id: usize) {
        let Some(entry) = self.ddt.get_or_alloc_leaf_entry(device_id) else {
            warn!(
                "RV IOMMU: Invalid device ID {} for DDT mode {:?}",
                device_id,
                self.ddt_mode()
            );
            return;
        };
        entry.tc.write(DDT_TC::V::CLEAR);
        info!(
            "RV IOMMU: Write DDT, remove decive context, iohgatp.mode = {:#x?}, ioghatp.ppn = {:#x?}",
            entry.iohgatp.read(DDT_IOHGATP::MODE),
            entry.iohgatp.read(DDT_IOHGATP::PPN)
        );
        // "RV IOMMU: DDT entry updated for device {}, remember to issue IODIR/IOTINVAL commands if entry is changed while IOMMU is active"
    }
}
