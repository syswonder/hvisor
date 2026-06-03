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
//      Jingyu Liu <liujingyu24s@ict.ac.cn

use super::cmd::{
    IoDirFunc, IoFenceFunc, IotInvalFunc, RiscvIommuCommand, IODIR_OPCODE, IOFENCE_OPCODE,
    IOTINVAL_OPCODE,
};
use super::iommu_hw::{
    iommu_add_raw_command, iommu_read_ddt_field, iommu_read_reg, iommu_write_ddt_field,
    iommu_write_reg, IommuDdtField, IommuReg,
};
use super::reg_bits::{DDT_FSC, DDT_TC, IOMMU_CAPS, IOMMU_DDTP, IOMMU_XQB};
use crate::consts::MAX_ZONE_NUM;
use crate::consts::{IPI_EVENT_VCPU_RESUME, IPI_EVENT_VCPU_SUSPEND};
use crate::cpu_data::{signal_other_vcpus_resume, wait_for_other_vcpus_suspend};
use crate::error::HvResult;
use crate::event::send_event_to_all;
use crate::memory::{GuestPhysAddr, MMIOAccess, MemoryRegion};
use crate::zone::{find_zone, Zone};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr;
use spin::Mutex;

/// Masked capabilities for the Virtual IOMMU
/// Virtual IOMMU does not support the following capabilities
const VIOMMU_MASKED_CAPS: u64 = (IOMMU_CAPS::SV32X4.mask << IOMMU_CAPS::SV32X4.shift)
    | (IOMMU_CAPS::SV39X4.mask << IOMMU_CAPS::SV39X4.shift)
    | (IOMMU_CAPS::SV48X4.mask << IOMMU_CAPS::SV48X4.shift)
    | (IOMMU_CAPS::SV57X4.mask << IOMMU_CAPS::SV57X4.shift)
    | (IOMMU_CAPS::MSI_FLAT.mask << IOMMU_CAPS::MSI_FLAT.shift)
    | (IOMMU_CAPS::MSI_MRIF.mask << IOMMU_CAPS::MSI_MRIF.shift)
    | (IOMMU_CAPS::PD8.mask << IOMMU_CAPS::PD8.shift)
    | (IOMMU_CAPS::PD17.mask << IOMMU_CAPS::PD17.shift)
    | (IOMMU_CAPS::PD20.mask << IOMMU_CAPS::PD20.shift)
    | (IOMMU_CAPS::ATS.mask << IOMMU_CAPS::ATS.shift)
    | (IOMMU_CAPS::SVPBMT.mask << IOMMU_CAPS::SVPBMT.shift)
    | (IOMMU_CAPS::T2GPA.mask << IOMMU_CAPS::T2GPA.shift)
    | (IOMMU_CAPS::HPM.mask << IOMMU_CAPS::HPM.shift)
    | (IOMMU_CAPS::DBG.mask << IOMMU_CAPS::DBG.shift)
    | (IOMMU_CAPS::END.mask << IOMMU_CAPS::END.shift)
    | (IOMMU_CAPS::IGS.mask << IOMMU_CAPS::IGS.shift);

/// Size of a 1-level DDT page exposed by the vIOMMU.
const VIOMMU_DDT1LVL_SIZE: usize = 0x1000;
const DDT_ENTRY_SIZE: usize = 32;
const CQ_ENTRY_SIZE: usize = 16;

const REG_CAPS_START: usize = 0x0;
const REG_CAPS_END: usize = 0x7;
const REG_FCTL_START: usize = 0x8;
const REG_FCTL_END: usize = 0xB;
const REG_DDTP_START: usize = 0x10;
const REG_DDTP_END: usize = 0x17;
const REG_CQB_START: usize = 0x18;
const REG_CQB_END: usize = 0x1F;
const REG_CQH_START: usize = 0x20;
const REG_CQH_END: usize = 0x23;
const REG_CQT_START: usize = 0x24;
const REG_CQT_END: usize = 0x27;
const REG_FQB_START: usize = 0x28;
const REG_FQB_END: usize = 0x2F;
const REG_FQH_START: usize = 0x30;
const REG_FQH_END: usize = 0x33;
const REG_FQT_START: usize = 0x34;
const REG_FQT_END: usize = 0x37;
const REG_CQCSR_START: usize = 0x48;
const REG_CQCSR_END: usize = 0x4B;
const REG_FQCSR_START: usize = 0x4C;
const REG_FQCSR_END: usize = 0x4F;
const REG_IPSR_START: usize = 0x54;
const REG_IPSR_END: usize = 0x57;
const REG_ICVEC_START: usize = 0x2F8;
const REG_ICVEC_END: usize = 0x2FF;

const DDT_FIELD_TC_START: usize = 0x0;
const DDT_FIELD_TC_END: usize = 0x7;
const DDT_FIELD_IOHGATP_START: usize = 0x8;
const DDT_FIELD_IOHGATP_END: usize = 0xF;
const DDT_FIELD_TA_START: usize = 0x10;
const DDT_FIELD_TA_END: usize = 0x17;
const DDT_FIELD_FSC_START: usize = 0x18;
const DDT_FIELD_FSC_END: usize = 0x1F;
const DDT_TC_ALLOWED_WRITE_MASK: usize = (DDT_TC::V.mask << DDT_TC::V.shift) as usize;

const CQ_MAX_ENTRIES: u64 = 256;
const CQ_LOG2SZ_1_CAP: u64 = 0x7;
const DDTP_PPN_TO_GPA_SHIFT: usize = 12;
const CMD_OPCODE_MASK: u64 = 0x7f;
const CMD_FUNC3_SHIFT: u64 = 7;
const CMD_FUNC3_MASK: u64 = 0x7;
const IOTINVAL_GV_SHIFT: u64 = 33;
const IOTINVAL_GSCID_SHIFT: u64 = 44;
const IOTINVAL_GSCID_MASK: u64 = 0xffff;
const MAX_VIOMMU_DDT_DEVICES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IommuMode {
    Off,
    Bare,
    Ddt1Lvl,
    Ddt2Lvl,
    Ddt3Lvl,
}

impl TryFrom<usize> for IommuMode {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value as u8 {
            0 => Ok(Self::Off),
            1 => Ok(Self::Bare),
            2 => Ok(Self::Ddt1Lvl),
            3 => Ok(Self::Ddt2Lvl),
            4 => Ok(Self::Ddt3Lvl),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ViommuRegion {
    Regs,
    Ddt,
}

impl ViommuRegion {
    fn label(self) -> &'static str {
        match self {
            Self::Regs => "",
            Self::Ddt => "ddt ",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MmioAccessType {
    Read,
    Write,
}

impl MmioAccessType {
    fn from_is_write(is_write: bool) -> Self {
        if is_write {
            Self::Write
        } else {
            Self::Read
        }
    }
}

lazy_static! {
    /// Global viommu array, one viommu instance for each zone.
    static ref VIOMMU_ARR: Mutex<Vec<Option<Arc<VirtualIommu>>>> =
        Mutex::new(vec![None; MAX_ZONE_NUM]);
}

fn validate_zone_id(zone_id: usize) -> bool {
    if zone_id >= MAX_ZONE_NUM {
        error!("Invalid zone id: {}", zone_id);
        return false;
    }
    true
}

/// Create one viommu instance for target zone.
pub(super) fn viommu_init(zone_id: usize) {
    if !validate_zone_id(zone_id) {
        return;
    }

    let mut viommu_arr = VIOMMU_ARR.lock();
    if viommu_arr[zone_id].is_some() {
        warn!("Zone {}'s Virtual IOMMU already initialized.", zone_id);
        return;
    }
    viommu_arr[zone_id] = Some(Arc::new(VirtualIommu::new()));
    info!("Zone {}'s Virtual IOMMU initialized.", zone_id);
}

/// Remove one viommu instance for target zone and clear its physical DDT side effects.
pub(super) fn viommu_remove(zone_id: usize) {
    if !validate_zone_id(zone_id) {
        return;
    }

    let viommu = {
        let mut viommu_arr = VIOMMU_ARR.lock();
        viommu_arr[zone_id].take()
    };

    let Some(viommu) = viommu else {
        warn!("Zone {}'s Virtual IOMMU does not exist.", zone_id);
        return;
    };

    // Clean some content stored in memory related to this viommu.
    viommu.cleanup_physical_ddt();
    info!("Zone {}'s Virtual IOMMU removed.", zone_id);
}

/// Register viommu mmio handler for target zone.
pub(super) fn viommu_mmio_handler_register(zone: &Zone, viommu_base: usize, viommu_size: usize) {
    zone.write()
        .mmio_region_register(viommu_base, viommu_size, viommu_emul_handler, zone.id());
}

/// Handle Zone's iommu mmio access.
fn viommu_emul_handler(mmio: &mut MMIOAccess, zone_id: usize) -> HvResult {
    viommu_mmio_emul_handler(mmio, zone_id, ViommuRegion::Regs)
}

fn get_viommu_by_zone_id(zone_id: usize) -> Option<Arc<VirtualIommu>> {
    if !validate_zone_id(zone_id) {
        return None;
    }
    let viommu_array = VIOMMU_ARR.lock();
    // ref, don't move viommu out of the array
    match &viommu_array[zone_id] {
        Some(viommu) => Some(Arc::clone(viommu)),
        None => {
            error!("VirtualIommu for Zone {} does not exist!", zone_id);
            None
        }
    }
}

/// Handle Zone's iommu ddt mmio access.
fn viommu_ddt_emul_handler(mmio: &mut MMIOAccess, zone_id: usize) -> HvResult {
    viommu_mmio_emul_handler(mmio, zone_id, ViommuRegion::Ddt)
}

fn viommu_mmio_emul_handler(
    mmio: &mut MMIOAccess,
    zone_id: usize,
    region: ViommuRegion,
) -> HvResult {
    let Some(viommu) = get_viommu_by_zone_id(zone_id) else {
        warn!(
            "vIOMMU {}mmio access for non-initialized zone {}",
            region.label(),
            zone_id
        );
        return Ok(());
    };
    let Some(zone) = find_zone(zone_id) else {
        warn!(
            "vIOMMU {}mmio access for unknown zone {}",
            region.label(),
            zone_id
        );
        return Ok(());
    };
    let access = MmioAccessType::from_is_write(mmio.is_write);
    let value = match region {
        ViommuRegion::Regs => {
            viommu.viommu_emul_access(&zone, mmio.address, mmio.size, mmio.value, access)
        }
        ViommuRegion::Ddt => {
            viommu.viommu_ddt_emul_access(&zone, mmio.address, mmio.size, mmio.value, access)
        }
    };
    if access == MmioAccessType::Read {
        mmio.value = value as usize;
    }
    Ok(())
}

/// Virtual IOMMU device structure
struct VirtualIommu {
    /// Multithread safe inner structure
    inner: Mutex<VirtualIommuInner>,
}

/// Virtual IOMMU
struct VirtualIommuInner {
    regs: ViommuRegs,
    cq: CommandQueueState,
    fq: FaultQueueState,
    ddt: DdtShadowState,
}

struct ViommuRegs {
    caps: u64,
    fctl: u32,
    ddtp: u64,
}

struct CommandQueueState {
    base: u64,
    head: u32,
    tail: u32,
    num_entries: u32,
    gpa: u64,
}

struct FaultQueueState {
    base: u64,
    head: u32,
    tail: u32,
}

/// Device-directory-table shadow state.
/// It affects the device directory table that the zone sees.
struct DdtShadowState {
    tc: Vec<u64>,
    fsc_written: Vec<bool>,
}

impl VirtualIommu {
    fn new() -> Self {
        Self {
            inner: Mutex::new(VirtualIommuInner::new()),
        }
    }

    /// vIOMMU emul access.
    fn viommu_emul_access(
        &self,
        zone: &Zone,
        offset: usize,
        size: usize,
        value: usize,
        access: MmioAccessType,
    ) -> u64 {
        self.inner
            .lock()
            .viommu_emul_access(zone, offset, size, value, access)
    }

    /// vIOMMU ddt emul access.
    fn viommu_ddt_emul_access(
        &self,
        zone: &Zone,
        offset: usize,
        size: usize,
        value: usize,
        access: MmioAccessType,
    ) -> u64 {
        self.inner
            .lock()
            .viommu_ddt_emul_access(zone, offset, size, value, access)
    }
}

impl ViommuRegs {
    fn new() -> Self {
        let mut caps = iommu_read_reg(IommuReg::Caps);
        caps &= !VIOMMU_MASKED_CAPS;
        // Don't support MSI irq generated by IOMMU self.
        caps |= IOMMU_CAPS::IGS::WSI.value;

        Self {
            caps,
            fctl: 0,
            ddtp: 0,
        }
    }
}

impl CommandQueueState {
    fn new() -> Self {
        Self {
            base: 0,
            head: 0,
            tail: 0,
            num_entries: 0,
            gpa: 0,
        }
    }
}

impl FaultQueueState {
    fn new() -> Self {
        Self {
            base: 0,
            head: 0,
            tail: 0,
        }
    }
}

impl DdtShadowState {
    fn new() -> Self {
        Self {
            tc: vec![0; MAX_VIOMMU_DDT_DEVICES],
            fsc_written: vec![false; MAX_VIOMMU_DDT_DEVICES],
        }
    }

    fn mark_fsc_written(&mut self, device_id: usize) {
        if let Some(written) = self.fsc_written.get_mut(device_id) {
            *written = true;
        }
    }

    fn fsc_needs_cleanup(&self, device_id: usize) -> bool {
        self.fsc_written.get(device_id).copied().unwrap_or(false)
    }
}

impl VirtualIommuInner {
    fn new() -> Self {
        Self {
            regs: ViommuRegs::new(),
            cq: CommandQueueState::new(),
            fq: FaultQueueState::new(),
            ddt: DdtShadowState::new(),
        }
    }

    /// vIOMMU emul access inner.
    fn viommu_emul_access(
        &mut self,
        zone: &Zone,
        offset: usize,
        size: usize,
        value: usize,
        access: MmioAccessType,
    ) -> u64 {
        // The current emulation dispatches by register range; access width is
        // accepted as provided by the common MMIO layer.
        let _ = size;
        let zone_id = zone.id();
        match access {
            MmioAccessType::Read => self.read_reg_access(offset),
            MmioAccessType::Write => {
                self.write_reg_access(zone, zone_id, offset, value);
                0
            }
        }
    }

    fn read_reg_access(&self, offset: usize) -> u64 {
        match offset {
            REG_CAPS_START..=REG_CAPS_END => {
                info!("vIOMMU caps: {:#x}", self.regs.caps);
                self.regs.caps
            }
            REG_FCTL_START..=REG_FCTL_END => self.regs.fctl as u64,
            REG_DDTP_START..=REG_DDTP_END => self.regs.ddtp,
            REG_CQB_START..=REG_CQB_END => self.cq.base,
            REG_CQH_START..=REG_CQH_END => self.cq.head as u64,
            REG_CQT_START..=REG_CQT_END => self.cq.tail as u64,
            REG_FQB_START..=REG_FQB_END => self.fq.base,
            REG_FQH_START..=REG_FQH_END => self.fq.head as u64,
            REG_FQT_START..=REG_FQT_END => self.fq.tail as u64,
            // We only keep minimum compatibility for CSR mirrors now.
            REG_CQCSR_START..=REG_CQCSR_END => iommu_read_reg(IommuReg::Cqcsr),
            REG_FQCSR_START..=REG_FQCSR_END => iommu_read_reg(IommuReg::Fqcsr),
            REG_IPSR_START..=REG_IPSR_END => iommu_read_reg(IommuReg::Ipsr),
            REG_ICVEC_START..=REG_ICVEC_END => iommu_read_reg(IommuReg::Icvec),
            _ => {
                warn!("vIOMMU mmio access offset {:#x} not supported", offset);
                0
            }
        }
    }

    fn write_reg_access(&mut self, zone: &Zone, zone_id: usize, offset: usize, value: usize) {
        match offset {
            REG_CAPS_START..=REG_CAPS_END => {
                error!("Capabilities register is read-only!");
            }
            REG_FCTL_START..=REG_FCTL_END => {
                let host_fctl = iommu_read_reg(IommuReg::Fctl);
                if value != host_fctl as usize {
                    error!(
                        "vIOMMU fctl write value {:#x} not match host fctl {:#x}!",
                        value, host_fctl
                    );
                } else {
                    self.regs.fctl = value as u32;
                }
            }
            REG_DDTP_START..=REG_DDTP_END => {
                self.handle_ddtp_write(zone, value);
            }
            REG_CQB_START..=REG_CQB_END => self.handle_cqb_write(value),
            REG_CQH_START..=REG_CQH_END => {
                error!("vIOMMU cqh is read-only!");
            }
            REG_CQT_START..=REG_CQT_END => self.handle_cqt_write(zone_id, value),
            REG_FQB_START..=REG_FQB_END => {
                self.fq.base = value as u64;
            }
            REG_FQH_START..=REG_FQH_END => {
                self.fq.head = value as u32;
            }
            REG_FQT_START..=REG_FQT_END => {
                self.fq.tail = value as u32;
            }
            // We only keep minimum compatibility for CSR mirrors now.
            REG_CQCSR_START..=REG_CQCSR_END => iommu_write_reg(IommuReg::Cqcsr, value as u64),
            REG_FQCSR_START..=REG_FQCSR_END => iommu_write_reg(IommuReg::Fqcsr, value as u64),
            REG_IPSR_START..=REG_IPSR_END => iommu_write_reg(IommuReg::Ipsr, value as u64),
            REG_ICVEC_START..=REG_ICVEC_END => iommu_write_reg(IommuReg::Icvec, value as u64),
            _ => {
                warn!("vIOMMU mmio access offset {:#x} not supported", offset);
            }
        }
    }

    fn handle_ddtp_write(&mut self, zone: &Zone, value: usize) -> bool {
        info!("vIOMMU ddtp write value: {:#x}", value);
        let mode_raw = ((value as u64 & IOMMU_DDTP::MODE.mask) >> IOMMU_DDTP::MODE.shift) as usize;
        match IommuMode::try_from(mode_raw) {
            Ok(mode) => {
                info!("Guest try to set vIOMMU mode to {:?}", mode);
                match mode {
                    IommuMode::Off | IommuMode::Bare => {
                        self.regs.ddtp = value as u64;
                    }
                    IommuMode::Ddt1Lvl => {
                        if !self.handle_ddt1lvl_mode(zone, value) {
                            return false;
                        }
                        self.regs.ddtp = value as u64;
                    }
                    IommuMode::Ddt2Lvl | IommuMode::Ddt3Lvl => {
                        info!("vIOMMU ddtp mode {:?} not supported yet!", mode);
                    }
                }
            }
            Err(_) => {
                error!("vIOMMU ddtp mode {:#x} not supported!", mode_raw as u8);
            }
        }
        true
    }

    fn handle_ddt1lvl_mode(&mut self, zone: &Zone, value: usize) -> bool {
        let mut zone_inner = zone.write();
        let ppn = ((value as u64 & IOMMU_DDTP::PPN.mask) >> IOMMU_DDTP::PPN.shift) as usize;
        let ddt_gpa = ppn << DDTP_PPN_TO_GPA_SHIFT;
        info!("vIOMMU's DDT Table GPA: {:#x}", ddt_gpa);

        let Some(region) = zone_inner.gpm_mut().get_region(ddt_gpa as GuestPhysAddr) else {
            error!("vIOMMU ddtp region not found in gpm!");
            return false;
        };

        // We unmap this page to trigger a page fault when the guest accesses it.
        let cpu_set = zone_inner.cpu_set();
        send_event_to_all(cpu_set, 0, IPI_EVENT_VCPU_SUSPEND);
        wait_for_other_vcpus_suspend(cpu_set);

        let gpm = zone_inner.gpm_mut();
        if let Err(err) = gpm.delete(region.start, region.size) {
            error!("vIOMMU ddtp region delete failed: {:?}", err);
            send_event_to_all(cpu_set, 0, IPI_EVENT_VCPU_RESUME);
            signal_other_vcpus_resume(cpu_set);
            return false;
        }

        let region_start = region.start;
        let region_end = region.start + region.size;
        if region_start < ddt_gpa {
            let left_region = MemoryRegion::new_with_offset_mapper(
                region_start as GuestPhysAddr,
                region_start,
                ddt_gpa - region_start,
                region.flags,
            );
            if let Err(err) = gpm.insert(left_region) {
                error!("vIOMMU ddtp left region insert failed: {:?}", err);
                send_event_to_all(cpu_set, 0, IPI_EVENT_VCPU_RESUME);
                signal_other_vcpus_resume(cpu_set);
                return false;
            }
        }
        if region_end > ddt_gpa + VIOMMU_DDT1LVL_SIZE {
            // For 1LVL DDT, the region size is one 4KiB page.
            let right_start = ddt_gpa + VIOMMU_DDT1LVL_SIZE;
            let right_region = MemoryRegion::new_with_offset_mapper(
                right_start as GuestPhysAddr,
                right_start,
                region_end - right_start,
                region.flags,
            );
            if let Err(err) = gpm.insert(right_region) {
                error!("vIOMMU ddtp right region insert failed: {:?}", err);
                send_event_to_all(cpu_set, 0, IPI_EVENT_VCPU_RESUME);
                signal_other_vcpus_resume(cpu_set);
                return false;
            }
        }
        info!("gpm after unmap vIOMMU ddtp region: {:#x?}", gpm);
        // SAFETY: flush stage-2 translations after changing guest mappings.
        unsafe { riscv_h::asm::hfence_gvma(0, 0) };
        // Keep zone_id as 0 for now to preserve current behavior.
        zone_inner.mmio_region_register(ddt_gpa, VIOMMU_DDT1LVL_SIZE, viommu_ddt_emul_handler, 0);
        send_event_to_all(cpu_set, 0, IPI_EVENT_VCPU_RESUME);
        signal_other_vcpus_resume(cpu_set);
        true
    }

    fn handle_cqb_write(&mut self, value: usize) {
        let value_u64 = value as u64;
        let ppn = (value_u64 & IOMMU_XQB::PPN.mask) >> IOMMU_XQB::PPN.shift;
        let log2sz_1 = (value_u64 & IOMMU_XQB::LOG2SZ_1.mask) >> IOMMU_XQB::LOG2SZ_1.shift;
        let mut num_entries = 1u64 << (log2sz_1 + 1);
        let mut new_value = value_u64;
        if num_entries > CQ_MAX_ENTRIES {
            new_value = (value_u64 & !IOMMU_XQB::LOG2SZ_1.mask) | CQ_LOG2SZ_1_CAP;
            num_entries = CQ_MAX_ENTRIES;
        }
        self.cq.num_entries = num_entries as u32;
        self.cq.gpa = ppn << DDTP_PPN_TO_GPA_SHIFT;
        self.cq.base = new_value;
        info!(
            "vIOMMU cqb: {:#x}, cq_gpa: {:#x}, cq_num_entries: {}",
            self.cq.base, self.cq.gpa, self.cq.num_entries
        );
    }

    fn handle_cqt_write(&mut self, zone_id: usize, value: usize) {
        if self.cq.num_entries == 0 {
            warn!("vIOMMU cqt write before cqb init, ignoring");
            return;
        }

        let new_tail = value as u32 % self.cq.num_entries;
        let mut cqh = self.cq.head;
        while cqh != new_tail {
            let cqe_addr = (self.cq.gpa as usize)
                + (cqh as usize % self.cq.num_entries as usize) * CQ_ENTRY_SIZE;
            let (dword0, dword1) = Self::read_cq_entry(cqe_addr);
            self.dispatch_cq_command(zone_id, dword0, dword1);
            cqh = (cqh + 1) % self.cq.num_entries;
        }
        self.cq.tail = new_tail;
        self.cq.head = new_tail;
    }

    fn dispatch_cq_command(&self, zone_id: usize, dword0: u64, dword1: u64) {
        let opcode = dword0 & CMD_OPCODE_MASK;
        let func3 = (dword0 >> CMD_FUNC3_SHIFT) & CMD_FUNC3_MASK;
        match opcode {
            op if op == u64::from(IOTINVAL_OPCODE) => {
                let vma_raw = u64::from(IotInvalFunc::Vma.raw());
                let gvma_raw = u64::from(IotInvalFunc::Gvma.raw());
                if func3 == vma_raw {
                    // Guest VMA command is always bound to this zone's GSCID.
                    let mut out0 = dword0;
                    out0 |= 1u64 << IOTINVAL_GV_SHIFT;
                    out0 &= !(IOTINVAL_GSCID_MASK << IOTINVAL_GSCID_SHIFT);
                    out0 |= ((zone_id as u64) & IOTINVAL_GSCID_MASK) << IOTINVAL_GSCID_SHIFT;
                    iommu_add_raw_command(RiscvIommuCommand {
                        dword0: out0,
                        dword1,
                    });
                } else if func3 == gvma_raw {
                    iommu_add_raw_command(RiscvIommuCommand { dword0, dword1 });
                } else {
                    warn!("vIOMMU IOTINVAL func3={} not supported", func3);
                }
            }
            op if op == u64::from(IOFENCE_OPCODE) => {
                if func3 != u64::from(IoFenceFunc::C.raw()) {
                    warn!("vIOMMU IOFENCE func3 unsupported");
                } else {
                    iommu_add_raw_command(RiscvIommuCommand { dword0, dword1 });
                }
            }
            op if op == u64::from(IODIR_OPCODE) => {
                let inval_ddt_raw = u64::from(IoDirFunc::InvalDdt.raw());
                let inval_pdt_raw = u64::from(IoDirFunc::InvalPdt.raw());
                if func3 == inval_ddt_raw {
                    iommu_add_raw_command(RiscvIommuCommand { dword0, dword1 });
                } else if func3 == inval_pdt_raw {
                    warn!("vIOMMU IODIR INVAL_PDT not supported yet");
                } else {
                    warn!("vIOMMU IODIR func3={} not supported", func3);
                }
            }
            _ => warn!("vIOMMU unknown CQ opcode {}", opcode),
        }
    }

    fn read_cq_entry(cqe_addr: usize) -> (u64, u64) {
        // SAFETY: CQE address comes from guest-provided CQB/CQH state and
        // current implementation assumes this memory is directly readable.
        let bytes: [u8; 16] = unsafe { ptr::read(cqe_addr as *const [u8; 16]) };
        let dword0 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let dword1 = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        (dword0, dword1)
    }

    /// Handle Zone's iommu ddt mmio access. (now only support 1LVL DDT)
    fn viommu_ddt_emul_access(
        &mut self,
        zone: &Zone,
        offset: usize,
        size: usize,
        value: usize,
        access: MmioAccessType,
    ) -> u64 {
        // DDT entry fields are 8 bytes wide; keep the current tolerant behavior
        // and dispatch by field offset rather than rejecting other widths here.
        let _ = size;
        // A hypervisor that provides such an emulated IOMMU to the guest may
        //      1.retain control of the MSI page tables used to direct MSIs to guest interrupt files
        //      2.clear the MSI_FLAT and MSI_MRIF fields of the emulated capabilities register.

        // So its Device-Context Format is below:
        //      - Translation Control (tc) 8bytes
        //      - IO Hypervisor guest address translation and protection (iohgatp) 8bytes
        //      - Translation-attributes (ta) 8bytes
        //      - First-stage-context (fsc) 8bytes

        let ddt_index = offset / DDT_ENTRY_SIZE; // each entry is 32 bytes (base format)
        if ddt_index == 0 || ddt_index >= self.ddt.tc.len() {
            warn!(
                "ddt_index {} is invalid for zone {}, ignore the access",
                ddt_index,
                zone.id()
            );
            return 0;
        }

        match offset % DDT_ENTRY_SIZE {
            DDT_FIELD_TC_START..=DDT_FIELD_TC_END => {
                self.handle_tc_access(ddt_index, value, access)
            }
            DDT_FIELD_IOHGATP_START..=DDT_FIELD_IOHGATP_END => {
                self.handle_iohgatp_access(ddt_index, value, access)
            }
            DDT_FIELD_TA_START..=DDT_FIELD_TA_END => {
                self.handle_ta_access(ddt_index, value, access)
            }
            DDT_FIELD_FSC_START..=DDT_FIELD_FSC_END => {
                self.handle_fsc_access(ddt_index, value, access)
            }
            _ => {
                error!(
                    "Unexpected offset value: {:#x}. This should never happen!",
                    offset
                );
                0
            }
        }
    }

    fn handle_tc_access(&mut self, ddt_index: usize, value: usize, access: MmioAccessType) -> u64 {
        match access {
            MmioAccessType::Read => self.ddt.tc[ddt_index],
            MmioAccessType::Write => {
                if value & !DDT_TC_ALLOWED_WRITE_MASK != 0 {
                    unimplemented!(
                        "vIOMMU ddt entry {} tc value {:#x} not supported!",
                        ddt_index,
                        value
                    );
                }
                self.ddt.tc[ddt_index] = value as u64;
                0
            }
        }
    }

    fn handle_iohgatp_access(&self, ddt_index: usize, value: usize, access: MmioAccessType) -> u64 {
        match access {
            MmioAccessType::Read => {
                iommu_read_ddt_field(ddt_index, IommuDdtField::Iohgatp).unwrap_or(0)
            }
            MmioAccessType::Write => {
                if value != 0 {
                    error!(
                        "vIOMMU ddt entry {} iohgatp value {:#x} not supported!",
                        ddt_index, value
                    );
                }
                0
            }
        }
    }

    fn handle_ta_access(&self, ddt_index: usize, value: usize, access: MmioAccessType) -> u64 {
        match access {
            MmioAccessType::Read => {
                warn!("vIOMMU ddt entry {} ta read not supported yet", ddt_index);
                0
            }
            MmioAccessType::Write => {
                if !iommu_write_ddt_field(ddt_index, IommuDdtField::Ta, value as u64) {
                    warn!("vIOMMU ddt entry {} ta write ignored", ddt_index);
                }
                0
            }
        }
    }

    fn handle_fsc_access(&mut self, ddt_index: usize, value: usize, access: MmioAccessType) -> u64 {
        match access {
            MmioAccessType::Read => {
                iommu_read_ddt_field(ddt_index, IommuDdtField::Fsc).unwrap_or(0)
            }
            MmioAccessType::Write => {
                let fsc = value as u64;
                let mode = (fsc & DDT_FSC::MODE.mask) >> DDT_FSC::MODE.shift;
                let mode_ok = mode == DDT_FSC::MODE::BARE.value
                    || mode == DDT_FSC::MODE::SV39.value
                    || mode == DDT_FSC::MODE::SV48.value
                    || mode == DDT_FSC::MODE::SV57.value;
                if !mode_ok {
                    error!(
                        "vIOMMU ddt entry {} fsc mode {:#x} not supported!",
                        ddt_index, mode
                    );
                    return 0;
                }
                if !iommu_write_ddt_field(ddt_index, IommuDdtField::Fsc, fsc) {
                    warn!("vIOMMU ddt entry {} fsc write ignored", ddt_index);
                } else {
                    self.ddt.mark_fsc_written(ddt_index);
                }
                0
            }
        }
    }
}

impl VirtualIommu {
    fn cleanup_physical_ddt(&self) {
        self.inner.lock().cleanup_physical_ddt();
    }
}

impl VirtualIommuInner {
    fn cleanup_physical_ddt(&mut self) {
        for device_id in 1..self.ddt.tc.len() {
            if !self.ddt.fsc_needs_cleanup(device_id) {
                continue;
            }
            if iommu_write_ddt_field(device_id, IommuDdtField::Fsc, 0) {
                self.ddt.fsc_written[device_id] = false;
                info!("vIOMMU cleaned DDT FSC for device {}", device_id);
            } else {
                warn!("vIOMMU failed to clean DDT FSC for device {}", device_id);
            }
        }
    }
}
