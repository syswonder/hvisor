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

// TODO:
// - [ ] Remove iommu from arch to device
// - [ ] Add a abstract interface for all-architecture IOMMU
// - [x] Support MSI remapping
// - [ ] Complete command queue and fault queue
// - [ ] Support vIOMMU
// - [ ] Increase more fault tolerance

use super::reg_bits::{
    DDT_DIR, DDT_FSC, DDT_IOHGATP, DDT_TC, IOMMU_CAPS, IOMMU_CQCSR, IOMMU_DDTP, IOMMU_FCTL,
    IOMMU_FQCSR, IOMMU_FQ_TAG, IOMMU_IPSR, IOMMU_XQB,
};
use super::{
    IoDirCommand, IoDirFunc, IoFenceCommand, IoFenceFunc, IotInvalCommand, IotInvalFunc,
    RiscvIommuCommand,
};
use crate::memory::Frame;
use alloc::vec::Vec;
use core::sync::atomic::{fence, Ordering};
use log::{error, info, warn};
use spin::{Mutex, Once};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};

const CQ_ENTRY_SIZE: usize = 16;
const FQ_ENTRY_SIZE: usize = 32;
const CQ_LOG2SZ_1: u32 = 7; // k-1, where k=log2(N), N=256
const CQ_MASK: u32 = CQ_ENTRIES as u32 - 1;
const FQ_LOG2SZ_1: u32 = 6; // k-1, where k=log2(N), N=128
const FQ_MASK: u32 = FQ_ENTRIES as u32 - 1;
const CQ_ENTRIES: usize = 1usize << (CQ_LOG2SZ_1 + 1);
const FQ_ENTRIES: usize = 1usize << (FQ_LOG2SZ_1 + 1);
const QUEUE_ON_TIMEOUT: usize = 1_000_000;
const QUEUE_FULL_TIMEOUT: usize = 1_000_000;
const QUEUE_FENCE_C_TIMEOUT: usize = 1_000_000;
const IOHGATP_MODE_BARE: u64 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
enum IommuDdtMode {
    Off = 0,        // No inbound memory translations are allowed by the IOMMU.
    Bare = 1,       // NO translation or protection.
    OneLevel = 2,   // One-level device-directory table.
    TwoLevel = 3,   // Two-level device-directory table.
    ThreeLevel = 4, // Three-level device-directory table.
}

// RISC-V IOMMU Specv1.0: Chap6.1 Register layout
register_structs! {
    #[allow(non_snake_case)]
    IommuHw {
        (0x000 => caps: ReadOnly<u64, IOMMU_CAPS::Register>),
        (0x008 => fctl: ReadWrite<u32, IOMMU_FCTL::Register>),
        (0x00c => _custom1),
        (0x010 => ddtp: ReadWrite<u64, IOMMU_DDTP::Register>),
        (0x018 => cqb: ReadWrite<u64, IOMMU_XQB::Register>),
        (0x020 => cqh: ReadWrite<u32>),
        (0x024 => cqt: ReadWrite<u32>),
        (0x028 => fqb: ReadWrite<u64, IOMMU_XQB::Register>),
        (0x030 => fqh: ReadWrite<u32>),
        (0x034 => fqt: ReadWrite<u32>),
        (0x038 => pqb: ReadWrite<u64>),
        (0x040 => pqh: ReadWrite<u32>),
        (0x044 => pqt: ReadWrite<u32>),
        (0x048 => cqcsr: ReadWrite<u32, IOMMU_CQCSR::Register>),
        (0x04c => fqcsr: ReadWrite<u32, IOMMU_FQCSR::Register>),
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
static IOMMU: Once<Mutex<Iommu>> = Once::new();

fn get_iommu<'a>() -> &'a Mutex<Iommu> {
    IOMMU.get().expect("Uninitialized hypervisor iommu!")
}

/// Initialize IOMMU with default mode
pub fn iommu_init() {
    #[cfg(feature = "iommu")]
    riscv_iommu_init();
    #[cfg(not(feature = "iommu"))]
    info!("RISC-V IOMMU: do nothing now");
}

/// Add a device to IOMMU
pub fn iommu_add_device(vm_id: usize, device_id: usize, root_pt: usize) {
    #[cfg(feature = "iommu")]
    {
        info!(
            "RV IOMMU: Add device, root_pt {:#x}, vm_id {}, device_id {}",
            root_pt, vm_id, device_id
        );
        let iommu = get_iommu();
        iommu.lock().rv_iommu_add_device(device_id, vm_id, root_pt);
    }
    #[cfg(not(feature = "iommu"))]
    info!("RISC-V: iommu_add_device do nothing now");
}

/// Remove a device from IOMMU (reserved for future hot-unplug paths).
pub fn iommu_remove_device(vm_id: usize, device_id: usize) {
    #[cfg(feature = "iommu")]
    {
        info!(
            "RV IOMMU: Remove device, vm_id {}, device_id {}",
            vm_id, device_id
        );
        let iommu = get_iommu();
        iommu.lock().rv_iommu_remove_device(device_id);
    }
    #[cfg(not(feature = "iommu"))]
    info!("RISC-V: iommu_remove_device do nothing now");
}

pub fn iommu_msi_pt_tlb_invalid(gscid: u16, msi_gpa: usize) {
    #[cfg(feature = "iommu")]
    {
        // If software changes a MSI page-table entry identified by interrupt file number I that corresponds to an
        //  untranslated MSI address A then the following invalidations must be performed:
        //      IOTINVAL.GVMA with GV=AV=1, ADDR[63:12]=A[63:12] and GSCID=DC.iohgatp.GSCID
        //
        // Between a change to the MSI PTE and when an invalidation command to invalidate the cached PTE is
        //  processed by the IOMMU, the IOMMU may use the old PTE value or the new PTE value.
        //
        // An IOFENCE.C command with PW=1 may be used to to ensure that all previous writes, including MSI writes, that have been
        //  command with PW=1 may be used to to ensure that all previous writes, including MSI writes, that have been
        //  previously processed by the IOMMU are committed to a global ordering point such that they can be
        //  observed by all RISC-V harts and IOMMUs in the system.
        info!(
            "RV IOMMU: Invalidate MSI PT, msi_gpa {:#x}, gscid {}",
            msi_gpa, gscid
        );
        let iommu = get_iommu();
        iommu.lock().rv_iommu_msi_pt_tlb_invalid(gscid, msi_gpa);
    }
    #[cfg(not(feature = "iommu"))]
    info!("RISC-V: iommu_msi_pt_tlb_invalid do nothing now");
}

/// Initialize RISC-V IOMMU with hardware DDTP probing.
fn riscv_iommu_init() {
    assert!(
        crate::platform::IOMMU_SYS_SIZE == 0x1000,
        "IOMMU_SYS_SIZE is not 0x1000"
    );
    let iommu = Iommu::new(crate::platform::IOMMU_SYS_BASE);
    IOMMU.call_once(|| Mutex::new(iommu));
    get_iommu().lock().rv_iommu_init();
}

impl IommuHw {
    fn wait_cq_on(&self) {
        let mut loops = 0usize;
        while !self.cqcsr.is_set(IOMMU_CQCSR::CQON) {
            core::hint::spin_loop();
            loops += 1;
            if loops >= QUEUE_ON_TIMEOUT {
                panic!("RISC-V IOMMU: timeout waiting for CQON");
            }
        }
    }

    fn wait_fq_on(&self) {
        let mut loops = 0usize;
        while !self.fqcsr.is_set(IOMMU_FQCSR::FQON) {
            core::hint::spin_loop();
            loops += 1;
            if loops >= QUEUE_ON_TIMEOUT {
                panic!("RISC-V IOMMU: timeout waiting for FQON");
            }
        }
    }

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
        let candidates: &[IommuDdtMode] = match requested_mode {
            IommuDdtMode::Off => &[],
            IommuDdtMode::Bare => &[IommuDdtMode::Bare],
            IommuDdtMode::OneLevel => &[IommuDdtMode::OneLevel],
            IommuDdtMode::TwoLevel => &[IommuDdtMode::TwoLevel, IommuDdtMode::OneLevel],
            IommuDdtMode::ThreeLevel => &[
                IommuDdtMode::ThreeLevel,
                IommuDdtMode::TwoLevel,
                IommuDdtMode::OneLevel,
            ],
        };
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
        if cpu_s2pt_lvl == 3 && !self.caps.is_set(IOMMU_CAPS::SV39X4) {
            panic!("CPU s2pt is Sv39x4, but IOMMU does not support Sv39x4");
        }
        if cpu_s2pt_lvl == 4 && !self.caps.is_set(IOMMU_CAPS::SV48X4) {
            panic!("CPU s2pt is Sv48x4, but IOMMU does not support Sv48x4");
        }
        if cpu_s2pt_lvl == 5 && !self.caps.is_set(IOMMU_CAPS::SV57X4) {
            panic!("CPU s2pt is Sv57x4, but IOMMU does not support Sv57x4");
        }
        // If capabilities.MSI_FLAT is 1 then the Extended Format is used else the Base Format is used.
        if !self.caps.is_set(IOMMU_CAPS::MSI_FLAT) {
            // Current DDT Entry only supports Extented-for
            todo!("To support Base-format DDT Entry");
        }
        if self.caps.read(IOMMU_CAPS::IGS) == IOMMU_CAPS::IGS::MSI.value {
            warn!("RISC-V IOMMU HW does not support WSI generation");
        }
    }

    fn rv_iommu_init(
        &mut self,
        ddt_addr: usize,
        ddt_mode: IommuDdtMode,
        cq_addr: usize,
        fq_addr: usize,
    ) -> IommuDdtMode {
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
        // TODO: program icvec

        // Program command queue:
        // Here use static one frame for command queue.
        let cq_size = CQ_ENTRIES * CQ_ENTRY_SIZE;
        self.cqb.write(
            IOMMU_XQB::LOG2SZ_1.val(CQ_LOG2SZ_1 as u64)
                + IOMMU_XQB::PPN.val((cq_addr as u64) >> 12),
        );
        self.cqt.set(0x0);
        self.cqcsr.write(IOMMU_CQCSR::CQEN::SET);
        self.wait_cq_on(); // Poll cqcsr.cqon until it reads 1

        // Program fault queue:
        // Here use static one frame for fault queue.
        let fq_size = FQ_ENTRIES * FQ_ENTRY_SIZE;
        self.fqb.write(
            IOMMU_XQB::LOG2SZ_1.val(FQ_LOG2SZ_1 as u64)
                + IOMMU_XQB::PPN.val((fq_addr as u64) >> 12),
        );
        self.fqh.set(0x0);
        self.fqcsr.write(IOMMU_FQCSR::FQEN::SET);
        self.wait_fq_on(); // Poll fqcsr.fqon until it reads 1

        // Do not support page-request queue.
        self.pqb.set(0x0);
        self.pqh.set(0x0);
        self.pqt.set(0x0);

        // Configure ddtp with DDT base address and IOMMU mode
        self.set_ddtp(ddt_addr, ddt_mode)
    }

    fn cq_is_empty(&self) -> bool {
        // If cqh == cqt, the command-queue is empty.
        self.cqh.get() & CQ_MASK == self.cqt.get() & CQ_MASK
    }

    fn cq_is_full(&self) -> bool {
        // If cqt == (cqh - 1) the command-queue is full.
        (self.cqt.get() & CQ_MASK) == (self.cqh.get().wrapping_sub(1) & CQ_MASK)
    }

    fn advance_cqt(&mut self) {
        self.cqt.set(self.cqt.get().wrapping_add(1) & CQ_MASK);
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

/// Non-leaf device-directory table
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
struct DdtRootMemory {
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

    /// Check if the ddt entry is valid, if not, allocate a new child table and return the new table address.
    fn ensure_child_table(&mut self, table_paddr: usize, idx: usize) -> (usize, bool) {
        let entry = &mut Self::dir_table_at(table_paddr).entries[idx];
        if entry.is_set(DDT_DIR::V) {
            return ((entry.read(DDT_DIR::PPN) as usize) << 12, false);
        }
        let child_paddr = self.alloc_next_level_table();
        entry.write(DDT_DIR::V::SET + DDT_DIR::PPN.val((child_paddr as u64) >> 12));
        (child_paddr, true)
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

    // For external users to get or allocate a leaf entry.
    fn get_or_alloc_leaf_entry(&mut self, device_id: usize) -> Option<(&mut DdtEntry, bool)> {
        if device_id > Self::DEV_ID_MAX {
            return None;
        }
        let (l1, l2, l3) = Self::ddt_indices(device_id);
        // Get leaf-table
        let (leaf_table_paddr, non_leaf_updated) = match self.mode {
            IommuDdtMode::OneLevel => (self.root_paddr(), false),
            IommuDdtMode::TwoLevel => self.ensure_child_table(self.root_paddr(), l2),
            IommuDdtMode::ThreeLevel => {
                let (lvl2_table_paddr, updated_l1) = self.ensure_child_table(self.root_paddr(), l1);
                let (leaf_table_paddr, updated_l2) = self.ensure_child_table(lvl2_table_paddr, l2);
                (leaf_table_paddr, updated_l1 || updated_l2)
            }
            _ => return None,
        };
        // Get DDT Entry
        Some((
            &mut Self::leaf_table_at(leaf_table_paddr).dc[l3],
            non_leaf_updated,
        ))
    }
}

/// Command queue entry, RISC-V IOMMU Spec v1.0 Chap4.1 Command-queue
#[repr(C)]
struct CqEntry {
    cmd: ReadWrite<u128>, // TODO: split into detailed fields
}

/// Fault queue entry, RISC-V IOMMU Spec v1.0 Chap4.2 Fault/Event-Queue
#[repr(C)]
struct FqEntry {
    tags: ReadWrite<u64, IOMMU_FQ_TAG::Register>,
    __rsv: ReadWrite<u64>,
    iotval: ReadWrite<u64>,
    iotval2: ReadWrite<u64>,
}

/// Global IOMMU structure
struct Iommu {
    base: usize,
    ddt: DdtRootMemory, // device-directory table
    cq: Frame,          // command queue
    fq: Frame,          // fault queue
}

impl Iommu {
    fn new(base: usize) -> Self {
        Self {
            base,
            ddt: DdtRootMemory::new(),
            cq: Frame::new_zero().unwrap(),
            fq: Frame::new_zero().unwrap(),
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
        let selected_mode = self.iommu().rv_iommu_init(
            self.ddt_root_paddr(),
            requested_mode,
            self.cq.start_paddr(),
            self.fq.start_paddr(),
        );
        if selected_mode != requested_mode {
            warn!(
                "RV IOMMU: DDTP mode downgraded from {:?} to {:?}",
                requested_mode, selected_mode
            );
        }
        self.ddt.set_mode(selected_mode);
    }

    // Used for adding a new device context to the DDT, enable IOMMU translation for specific device.
    fn rv_iommu_add_device(&mut self, device_id: usize, vm_id: usize, root_pt: usize) {
        if device_id == 0 {
            info!("Skip Device with device_id = 0");
            return;
        }
        // Check riscv stage-2 pt, root pt should be 16KiB aligned.
        if root_pt & ((16 * 1024) - 1) != 0 {
            error!(
                "RV IOMMU: iohgatp root page-table is not 16KiB aligned: {:#x}",
                root_pt
            );
            return;
        }

        let (entry_ptr, non_leaf_updated) = {
            let Some((entry, non_leaf_updated)) = self.ddt.get_or_alloc_leaf_entry(device_id)
            else {
                warn!(
                    "RV IOMMU: Invalid device ID {} for DDT mode {:?}",
                    device_id,
                    self.ddt_mode()
                );
                return;
            };
            (entry as *mut DdtEntry, non_leaf_updated)
        };

        // RISC-V IOMMU Spec v1.0 Chap7.3.1: non-leaf updates should perform invalidation.
        if non_leaf_updated {
            // If software changes a non-leaf-level DDT entry the following invalidations must be performed:
            //  IODIR.INVAL_DDT with DV=0
            self.enqueue_iodir_inval_ddt(false, 0);
            // Wait IODIR_INVAL has been executed done by IOMMU.
            self.sync_previous_commands(true, true);
        }

        // Convert pointer to reference.
        let entry = unsafe { &mut *entry_ptr };

        // Prepare TC without publishing VALID yet.
        entry.tc.set(0x0);

        let gstage_pt_level = unsafe { crate::arch::s2pt::GSTAGE_PT_LEVEL };
        // Configure the stage-2 page table mode same as cpu.
        let iohgatp_mode = match gstage_pt_level {
            3 => DDT_IOHGATP::MODE::SV39X4,
            4 => DDT_IOHGATP::MODE::SV48X4,
            5 => DDT_IOHGATP::MODE::SV57X4,
            _ => {
                error!("RV IOMMU: Invalid stage-2 pt level: {}", gstage_pt_level);
                return;
            }
        };

        entry.iohgatp.write(
            DDT_IOHGATP::PPN.val((root_pt as u64) >> 12)
                + DDT_IOHGATP::GSCID.val(vm_id as u64)
                + iohgatp_mode,
        );
        // Bare first-stage context.
        entry.fsc.set(0x0);
        entry.tc.write(DDT_TC::V::SET);
        let (dc_mode, dc_gscid) = (
            entry.iohgatp.read(DDT_IOHGATP::MODE),
            entry.iohgatp.read(DDT_IOHGATP::GSCID) as u16,
        );

        // RISC-V IOMMU Spec v1.0 Chap7.3.1: leaf updates should perform invalidation.
        //  IODIR.INVAL_DDT with DV=1 and DID=D
        //      If DC.iohgatp.MODE != Bare
        //          IOTINVAL.VMA with GV=1, AV=PSCV=0, and GSCID=DC.iohgatp.GSCID
        //          IOTINVAL.GVMA with GV=1, AV=0, and GSCID=DC.iohgatp.GSCID
        self.enqueue_leaf_ddt_invalidations(device_id as u32, dc_mode, dc_gscid);
        // Wait IODIR_INVAL has been executed done by IOMMU.
        self.sync_previous_commands(true, true);

        info!(
            "RV IOMMU: Write DDT, add decive context, device_id {}, mode {}, gscid {}",
            device_id, dc_mode, dc_gscid
        );
    }

    fn rv_iommu_remove_device(&mut self, device_id: usize) {
        if device_id == 0 {
            info!("Skip Device with device_id = 0");
            return;
        }
        let Some((entry, _)) = self.ddt.get_or_alloc_leaf_entry(device_id) else {
            warn!(
                "RV IOMMU: Invalid device ID {} for DDT mode {:?}",
                device_id,
                self.ddt_mode()
            );
            return;
        };
        let dc_mode = entry.iohgatp.read(DDT_IOHGATP::MODE);
        let dc_gscid = entry.iohgatp.read(DDT_IOHGATP::GSCID) as u16;
        // Update DDT Entry
        entry.tc.write(DDT_TC::V::CLEAR);
        self.enqueue_leaf_ddt_invalidations(device_id as u32, dc_mode, dc_gscid);
        self.sync_previous_commands(true, true);

        info!(
            "RV IOMMU: Write DDT, remove decive context, device_id {}, mode {}, gscid {}",
            device_id, dc_mode, dc_gscid
        );
    }

    fn rv_iommu_msi_pt_tlb_invalid(&mut self, gscid: u16, msi_gpa: usize) {
        self.enqueue_iotinval(IotInvalFunc::Gvma, gscid, true, msi_gpa);
        self.enqueue_iofence_c(false, true);
    }

    fn enqueue_leaf_ddt_invalidations(&mut self, device_id: u32, dc_mode: u64, dc_gscid: u16) {
        // Leaf DDT entry updated: always invalidate this DID.
        self.enqueue_iodir_inval_ddt(true, device_id);
        // If DC.iohgatp.MODE != Bare, issue both global VMA and global GVMA invalidations.
        if dc_mode != IOHGATP_MODE_BARE {
            // IOTINVAL.VMA with GV=1, AV=PSCV=0, GSCID=DC.iohgatp.GSCID
            self.enqueue_iotinval(IotInvalFunc::Vma, dc_gscid, false, 0);
            // IOTINVAL.GVMA with GV=1, AV=0, and GSCID=DC.iohgatp.GSCID
            self.enqueue_iotinval(IotInvalFunc::Gvma, dc_gscid, false, 0);
        }
    }

    fn enqueue_iodir_inval_ddt(&mut self, dv: bool, did: u32) {
        let iodir = IoDirCommand {
            func: IoDirFunc::InvalDdt,
            pid: 0,
            dv,
            did,
        };
        match iodir.encode() {
            Ok(cmd) => self.rv_iommu_add_command(cmd),
            Err(err) => error!("RV IOMMU: build IODIR command failed: {:?}", err),
        }
    }

    fn enqueue_iotinval(&mut self, func: IotInvalFunc, gscid: u16, av: bool, addr: usize) {
        let iotinval = IotInvalCommand {
            func,
            av,
            pscid: 0,
            pscv: false,
            gv: true,
            nl: false,
            gscid,
            s: false,
            addr: addr as u64,
        };
        match iotinval.encode() {
            Ok(cmd) => self.rv_iommu_add_command(cmd),
            Err(err) => error!("RV IOMMU: build IOTINVAL command failed: {:?}", err),
        }
    }

    fn enqueue_iofence_c(&mut self, pr: bool, pw: bool) {
        let iofence = IoFenceCommand {
            func: IoFenceFunc::C,
            av: false,
            wsi: false,
            pr,
            pw,
            data: 0,
            addr: 0,
        };
        match iofence.encode() {
            Ok(cmd) => self.rv_iommu_add_command(cmd),
            Err(err) => error!("RV IOMMU: build IOFENCE command failed: {:?}", err),
        }
    }

    fn sync_previous_commands(&mut self, pr: bool, pw: bool) {
        // Add IOFENCE.C command to CQ.
        self.enqueue_iofence_c(pr, pw);
        // Wait previous commands to be executed.
        let mut loops = 0usize;
        // Only one hart will handle CQ, here wait cq_is_empty is okay.
        while !self.iommu().cq_is_empty() {
            core::hint::spin_loop();
            loops += 1;
            if loops >= QUEUE_FENCE_C_TIMEOUT {
                error!("RV IOMMU: command sync timeout waiting CQ empty");
                panic!("RV IOMMU: timeout waiting command synchronization");
            }
        }
    }

    fn rv_iommu_add_command(&mut self, command: RiscvIommuCommand) {
        let raw = u128::from(command.dword0) | (u128::from(command.dword1) << 64);
        // Wait for CQ not full.
        let mut loops = 0usize;
        while self.iommu().cq_is_full() {
            core::hint::spin_loop();
            loops += 1;
            if loops >= QUEUE_FULL_TIMEOUT {
                error!("RV IOMMU: command queue full timeout");
                panic!("RV IOMMU: timeout waiting command queue not full");
            }
        }
        // Get current cqt index.
        let cqt_idx = (self.iommu().cqt.get() & CQ_MASK) as usize;
        let cq_entries = unsafe {
            core::slice::from_raw_parts_mut(self.cq.as_mut_ptr() as *mut CqEntry, CQ_ENTRIES)
        };
        // Write command to queue tail.
        cq_entries[cqt_idx].cmd.set(raw);
        // Make sure the ring buffer update (whether in normal or I/O memory) is
        //  completed and visible before signaling the tail doorbell to fetch
        //  the next command. 'fence ow, ow'
        unsafe { core::arch::asm!("fence ow, ow", options(nomem, nostack)) };
        self.iommu().advance_cqt();
    }
}
