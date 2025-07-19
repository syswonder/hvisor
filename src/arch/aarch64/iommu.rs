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
    arch::mm::new_s2_memory_set,
    consts::{MAX_ZONE_NUM, PAGE_SIZE},
    memory::{Frame, GuestPhysAddr, MemFlags, MemoryRegion, MemorySet, PhysAddr},
};
use aarch64_cpu::registers::{Readable, Writeable};
use alloc::vec::Vec;
use spin::mutex::Mutex;
use tock_registers::{
    register_structs,
    registers::{ReadOnly, ReadWrite},
};

use super::Stage2PageTable;

#[allow(dead_code)]
const SMMU_BASE_ADDR: PhysAddr = 0x09050000;
#[allow(dead_code)]
const SMMU_SIZE: PhysAddr = 0x20000;

const CR0_SMMUEN: usize = 1;
const ARM_SMMU_SYNC_TIMEOUT: usize = 0x1000000;
const CR0_CMDQEN: usize = 1 << 3;
// const CR0_EVTQEN:usize = 1 << 2;

const CR1_TABLE_SH_OFF: usize = 10;
const CR1_TABLE_OC_OFF: usize = 8;
const CR1_TABLE_IC_OFF: usize = 6;
const CR1_QUEUE_SH_OFF: usize = 4;
const CR1_QUEUE_OC_OFF: usize = 2;
const CR1_QUEUE_IC_OFF: usize = 0;
const SH_ISH: usize = 3;
const MEMATTR_OIWB: usize = 0xf;
const CR1_CACHE_WB: usize = 1;

const IDR0_S2P_BIT: usize = 1;
const IDR0_S1P_BIT: usize = 1 << 1;
const IDR0_ST_LEVEL_OFF: usize = 27;
const IDR0_ST_LEVEL_LEN: usize = 2;
const IDR0_VMID16_BIT: usize = 1 << 18;

const IDR1_SIDSIZE_OFF: usize = 0;
const IDR1_SIDSIZE_LEN: usize = 6;
const IDR1_CMDQS_OFF: usize = 21;
const IDR1_CMDQS_LEN: usize = 5;
// const IDR1_EVTQS_OFF:usize = 16;
// const IDR1_EVTQS_LEN:usize = 5;
const CMDQ_MAX_SZ_SHIFT: usize = 8;
// const EVTQ_MAX_SZ_SHIFT:usize = 7;

const STRTAB_BASE_OFF: usize = 6;
const STRTAB_BASE_LEN: usize = 46;
const STRTAB_BASE_RA: usize = 1 << 62;

const STRTAB_STE_DWORDS_BITS: usize = 3;
const STRTAB_STE_DWORDS: usize = 1 << STRTAB_STE_DWORDS_BITS;
const STRTAB_STE_SIZE: usize = STRTAB_STE_DWORDS << 3;
const STRTAB_STE_0_V: usize = 1;
const STRTAB_STE_0_INVALID: usize = 0;
const STRTAB_STE_0_CFG_OFF: usize = 1;
const STRTAB_STE_0_CFG_BYPASS: usize = 4;
const STRTAB_STE_0_CFG_S2_TRANS: usize = 6;
const STRTAB_STE_1_SHCFG_OFF: usize = 44;
const STRTAB_STE_1_SHCFG_INCOMING: usize = 1;
const STRTAB_STE_2_VTCR_OFF: usize = 32;
const STRTAB_STE_2_VTCR_LEN: usize = 19;
const STRTAB_STE_2_S2VMID_OFF: usize = 0;
const STRTAB_STE_2_S2PTW: usize = 1 << 54;
const STRTAB_STE_2_S2AA64: usize = 1 << 51;
const STRTAB_STE_2_S2R: usize = 1 << 58;
const STRTAB_STE_3_S2TTB_OFF: usize = 4;
const STRTAB_STE_3_S2TTB_LEN: usize = 48;

const STRTAB_BASE_CFG_FMT_OFF: usize = 16;
const STRTAB_BASE_CFG_FMT_LINEAR: usize = 0 << 16;
const STRTAB_BASE_CFG_LOG2SIZE_OFF: usize = 0;

const Q_BASE_RWA: usize = 1 << 62;
const Q_BASE_ADDR_OFF: usize = 5;
const Q_BASE_ADDR_LEN: usize = 47;
const Q_BASE_LOG2SIZE_OFF: usize = 0;
const Q_BASE_LOG2SIZE_LEN: usize = 5;

const CMDQ_ENT_DWORDS: usize = 2;
const CMDQ_ENT_SIZE: usize = CMDQ_ENT_DWORDS << 3;

const CMDQ_OP_CMD_SYNC: usize = 0x46;
// const CMDQ_SYNC_0_CS_SEV:usize = 2;
// const CMDQ_SYNC_0_CS_OFF:usize = 12;
const CMDQ_SYNC_0_MSH_OFF: usize = 22;
const CMDQ_SYNC_0_MSI_ATTR_OFF: usize = 24;

const CMDQ_OP_CFGI_STE: usize = 3;
const CMDQ_CFGI_0_SID_OFF: usize = 32;
const CMDQ_CFGI_1_LEAF: usize = 1;

const DEFAULT_VCTR: usize =
    20 + (2 << 6) + (1 << 8) + (1 << 10) + (3 << 12) + (0 << 14) + (4 << 16);

// page0 + page1
register_structs! {
    #[allow(non_snake_case)]
    pub RegisterPage{
        (0x0000 => IDR0:ReadOnly<u32>),
        (0x0004 => IDR1:ReadOnly<u32>),
        (0x0008 => IDR2:ReadOnly<u32>),
        (0x000c => IDR3:ReadOnly<u32>),
        (0x0010 => IDR4:ReadOnly<u32>),
        (0x0014 => IDR5:ReadOnly<u32>),
        (0x0018 => IIDR:ReadOnly<u32>),
        (0x001c => AIDR:ReadOnly<u32>),
        (0x0020 => CR0:ReadWrite<u32>),
        (0x0024 => CR0ACK:ReadOnly<u32>),
        (0x0028 => CR1:ReadWrite<u32>),
        (0x002c => CR2:ReadWrite<u32>),
        (0x0030 => _reserved0),
        (0x0050 => IRQ_CTRL:ReadWrite<u32>),
        (0x0054 => IRQ_CTRLACK:ReadOnly<u32>),
        (0x0058 => _reserved1),
        (0x0060 => GERROR:ReadOnly<u32>),
        (0x0064 => GERRORN:ReadWrite<u32>),
        (0x0068 => GERROR_IRQ_CFG0:ReadWrite<u64>),
        (0x0070 => _reserved2),
        (0x0080 => STRTAB_BASE:ReadWrite<u64>),
        (0x0088 => STRTAB_BASE_CFG:ReadWrite<u32>),
        (0x008c => _reserved3),
        (0x0090 => CMDQ_BASE:ReadWrite<u64>),
        (0x0098 => CMDQ_PROD:ReadWrite<u32>),
        (0x009c => CMDQ_CONS:ReadWrite<u32>),
        (0x00a0 => EVENTQ_BASE:ReadWrite<u64>),
        (0x00a8 => _reserved4),
        (0x00b0 => EVENTQ_IRQ_CFG0:ReadWrite<u64>),
        (0x00b8 => EVENTQ_IRQ_CFG1:ReadWrite<u32>),
        (0x00bc => EVENTQ_IRQ_CFG2:ReadWrite<u32>),
        (0x00c0 => _reserved5),
        (0x100a8 => EVENTQ_PROD:ReadWrite<u32>),
        (0x100ac => EVENTQ_CONS:ReadWrite<u32>),
        (0x100b0 => _reserved6),
        (0x20000 => @END),
    }
}

unsafe impl Sync for RegisterPage {}

pub fn extract_bits(value: usize, start: usize, length: usize) -> usize {
    let mask = (1 << length) - 1;
    (value >> start) & mask
}

pub struct StreamTableEntry([u64; STRTAB_STE_DWORDS]);

pub struct LinearStreamTable {
    base: PhysAddr,
    sid_max_bits: usize,
    frames: Vec<Frame>,
}

impl LinearStreamTable {
    fn new() -> Self {
        Self {
            base: 0,
            sid_max_bits: 0,
            frames: vec![],
        }
    }

    fn init_with_base(&mut self, base: PhysAddr, frame: Frame) {
        self.base = base;
        self.frames.push(frame);
    }

    fn get_base(&self) -> PhysAddr {
        self.base
    }

    fn set_max_sid(&mut self, sid_max_bits: usize) {
        self.sid_max_bits = sid_max_bits;
    }

    fn get_max_sid(&self) -> usize {
        self.sid_max_bits
    }

    fn ste(&self, sid: usize) -> &mut StreamTableEntry {
        let base = self.base + sid * STRTAB_STE_SIZE;
        unsafe { &mut *(base as *mut StreamTableEntry) }
    }

    fn init_bypass_ste(&self, sid: usize) {
        let tab = self.ste(sid);
        let mut val: usize = 0;
        val |= STRTAB_STE_0_INVALID;
        val |= STRTAB_STE_0_V;
        val |= STRTAB_STE_0_CFG_BYPASS << STRTAB_STE_0_CFG_OFF;
        tab.0[0] = val as _;
        tab.0[1] = (STRTAB_STE_1_SHCFG_INCOMING << STRTAB_STE_1_SHCFG_OFF) as _;
    }

    fn write_ste(&self, sid: usize, vmid: usize, root_pt: usize) {
        info!(
            "write ste, sid: 0x{:x}, vmid: 0x{:x}, ste_addr:0x{:x}, root_pt: 0x{:x}",
            sid,
            vmid,
            self.base + sid * STRTAB_STE_SIZE,
            root_pt
        );
        let tab = self.ste(sid);
        let mut val0: usize = 0;
        val0 |= STRTAB_STE_0_V;
        val0 |= STRTAB_STE_0_CFG_S2_TRANS << STRTAB_STE_0_CFG_OFF;
        let mut val2: usize = 0;
        val2 |= vmid << STRTAB_STE_2_S2VMID_OFF;
        val2 |= STRTAB_STE_2_S2PTW;
        val2 |= STRTAB_STE_2_S2AA64;
        val2 |= STRTAB_STE_2_S2R;
        let vtcr = DEFAULT_VCTR;
        let v = extract_bits(vtcr as _, 0, STRTAB_STE_2_VTCR_LEN);
        val2 |= v << STRTAB_STE_2_VTCR_OFF;
        let vttbr = extract_bits(root_pt, STRTAB_STE_3_S2TTB_OFF, STRTAB_STE_3_S2TTB_LEN);
        tab.0[0] |= val0 as u64;
        tab.0[2] |= val2 as u64;
        tab.0[3] |= (vttbr << STRTAB_STE_3_S2TTB_OFF) as u64;
    }
}

pub struct CmdQueue {
    base_reg: usize,
    base: PhysAddr,
    prod: u32,
    cons: u32,
    max_n_shift: u32,
    q_frame: Frame,
}

pub struct Cmd([u64; CMDQ_ENT_DWORDS]);

impl Cmd {
    fn new() -> Self {
        Cmd([0; CMDQ_ENT_DWORDS])
    }
}

impl CmdQueue {
    fn new() -> Self {
        let r = Self {
            base_reg: 0,
            base: 0,
            prod: 0,
            cons: 0,
            max_n_shift: 0,
            q_frame: Frame::new_zero().unwrap(),
        };
        r
    }

    fn init(&mut self, shift_bits: u32) {
        self.base = self.q_frame.start_paddr();
        self.base_reg = Q_BASE_RWA;
        self.max_n_shift = shift_bits;
        let addr_mask = extract_bits(self.q_frame.start_paddr(), Q_BASE_ADDR_OFF, Q_BASE_ADDR_LEN);
        self.base_reg |= addr_mask << Q_BASE_ADDR_OFF;
        self.base_reg |= extract_bits(
            self.max_n_shift as _,
            Q_BASE_LOG2SIZE_OFF,
            Q_BASE_LOG2SIZE_LEN,
        );
    }

    fn q_idx(&self, reg: u32) -> u32 {
        (reg) & ((1 << (self.max_n_shift)) - 1)
    }

    fn q_wrap(&self, reg: u32) -> u32 {
        (reg) & (1 << (self.max_n_shift))
    }

    fn q_ovf(&self, reg: u32) -> u32 {
        reg & (1 << 31)
    }

    fn q_empty(&self) -> bool {
        (self.q_idx(self.prod) == self.q_idx(self.cons))
            && (self.q_wrap(self.prod) == self.q_wrap(self.cons))
    }

    fn q_full(&self) -> bool {
        (self.q_idx(self.prod) == self.q_idx(self.cons))
            && (self.q_wrap(self.prod) != self.q_wrap(self.cons))
    }

    fn sync_cons(&mut self, cons: u32) {
        self.cons = cons;
    }

    fn inc_prod(&mut self) -> u32 {
        let prod: u32 = (self.q_wrap(self.prod) | self.q_idx(self.prod)) + 1;
        self.prod = self.q_ovf(self.prod) | self.q_wrap(prod) | self.q_idx(prod);
        self.prod
    }

    fn queue_entry(&self, reg: u32) -> usize {
        let entry = self.base + ((self.q_idx(reg) as usize) * CMDQ_ENT_SIZE);
        entry
    }

    fn queue_write(&mut self, cmd: Cmd) {
        unsafe {
            let entry = &mut *(self.queue_entry(self.prod) as *mut Cmd);
            entry.0[0] = cmd.0[0];
            entry.0[1] = cmd.0[1];
        }
    }

    // CMD_SYNC
    fn build_sync_cmd(&self) -> Cmd {
        let mut cmd: Cmd = Cmd::new();
        cmd.0[0] |= CMDQ_OP_CMD_SYNC as u64;
        cmd.0[0] |= (SH_ISH << CMDQ_SYNC_0_MSH_OFF) as u64;
        cmd.0[0] |= (MEMATTR_OIWB << CMDQ_SYNC_0_MSI_ATTR_OFF) as u64;
        cmd
    }

    // CFGI_STE
    fn build_cfgi_cmd(&self, sid: usize) -> Cmd {
        let mut cmd: Cmd = Cmd::new();
        cmd.0[0] |= (sid << CMDQ_CFGI_0_SID_OFF) as u64;
        cmd.0[0] |= CMDQ_OP_CFGI_STE as u64;
        cmd.0[1] |= CMDQ_CFGI_1_LEAF as u64;
        cmd
    }
}

pub struct Smmuv3 {
    rp: &'static RegisterPage,
    strtab: LinearStreamTable,
    iommu_pt_list: Vec<MemorySet<Stage2PageTable>>,
    cmdq: CmdQueue,
}

impl Smmuv3 {
    fn new() -> Self {
        let rp = unsafe { &*(SMMU_BASE_ADDR as *const RegisterPage) };
        let mut r = Self {
            rp: rp,
            strtab: LinearStreamTable::new(),
            iommu_pt_list: vec![],
            cmdq: CmdQueue::new(),
        };

        for _ in 0..MAX_ZONE_NUM {
            r.iommu_pt_list.push(new_s2_memory_set());
        }

        info!("pagetables for iommu, init done!");

        r.check_env();
        r.init_limited_pt();
        r.init_structures();
        r.device_reset();
        r
    }

    fn check_env(&mut self) {
        let idr0 = self.rp.IDR0.get() as usize;
        info!("Smmuv3 IDR0:{:b}", idr0);
        // supported types of stream tables.
        let stb_support = extract_bits(idr0, IDR0_ST_LEVEL_OFF, IDR0_ST_LEVEL_LEN);
        match stb_support {
            0 => info!("Smmuv3 Linear Stream Table Supported."),
            1 => info!("Smmuv3 2-level Stream Table Supoorted."),
            _ => error!("Smmuv3 don't support any stream table."),
        }
        // supported address translation stages.
        let s1p_support = idr0 & IDR0_S1P_BIT;
        match s1p_support {
            0 => info!("Smmuv3 Stage-1 translation not supported."),
            _ => info!("Smmuv3 Stage-1 translation supported."),
        }
        let s2p_support = idr0 & IDR0_S2P_BIT;
        match s2p_support {
            0 => error!("Smmuv3 Stage-2 translation not supported."),
            _ => info!("Smmuv3 Stage-2 translation supported."),
        }
        // 16-bit VMID supported.
        match idr0 & IDR0_VMID16_BIT {
            0 => info!("Smmuv3 16-bit VMID not supported."),
            _ => info!("Smmuv3 16-bit VMID supported."),
        }
        let idr1 = self.rp.IDR1.get() as usize;
        let sid_max_bits = extract_bits(idr1, IDR1_SIDSIZE_OFF, IDR1_SIDSIZE_LEN);
        info!("Smmuv3 SID_MAX_BITS:{:?}", sid_max_bits);
        self.strtab.set_max_sid(sid_max_bits);
        if sid_max_bits >= 7 && extract_bits(idr0, IDR0_ST_LEVEL_OFF, IDR0_ST_LEVEL_LEN) == 0 {
            error!("Smmuv3 the system must support for 2-level table");
        }
        if sid_max_bits <= 8 {
            info!("Smmuv3 must use linear stream table!");
        }
    }

    fn init_limited_pt(&mut self) {
        // its
        for pt in self.iommu_pt_list.iter_mut() {
            pt.insert(MemoryRegion::new_with_offset_mapper(
                0x8080000 as GuestPhysAddr,
                0x8080000,
                0x20000,
                MemFlags::READ | MemFlags::WRITE,
            ))
            .ok();
        }

        // ram
        self.iommu_pt_list[0]
            .insert(MemoryRegion::new_with_offset_mapper(
                0x80000000 as GuestPhysAddr,
                0x80000000,
                0x50000000,
                MemFlags::READ | MemFlags::WRITE,
            ))
            .ok();

        self.iommu_pt_list[1]
            .insert(MemoryRegion::new_with_offset_mapper(
                0x50000000 as GuestPhysAddr,
                0x50000000,
                0x30000000,
                MemFlags::READ | MemFlags::WRITE,
            ))
            .ok();

        self.iommu_pt_list[2]
            .insert(MemoryRegion::new_with_offset_mapper(
                0x80000000 as GuestPhysAddr,
                0x80000000,
                0x10000000,
                MemFlags::READ | MemFlags::WRITE,
            ))
            .ok();
    }

    fn init_structures(&mut self) {
        self.init_strtab();
        self.init_queues();
    }

    fn init_strtab(&mut self) {
        // linear stream table is our priority
        self.init_linear_strtab();
    }

    // strtab
    fn init_linear_strtab(&mut self) {
        info!("Smmuv3 initing linear stream table");
        // The lower (5 + self.sid_max_bits) bits must be 0.
        let tab_size = (1 << self.strtab.sid_max_bits) * STRTAB_STE_SIZE;
        let frame_count = tab_size / PAGE_SIZE; // need 4MB
        info!(
            "stream table frame cnts:{}, align is {}",
            frame_count,
            5 + self.strtab.sid_max_bits
        );
        if let Ok(frame) =
            Frame::new_contiguous_with_base(frame_count, 5 + self.strtab.sid_max_bits)
        {
            self.strtab.init_with_base(frame.start_paddr(), frame);
        } else {
            error!("stream table frames alloc err!!!")
        }
        info!("strtab_base:0x{:x}", self.strtab.get_base());
        let mut base = extract_bits(self.strtab.get_base(), STRTAB_BASE_OFF, STRTAB_BASE_LEN);
        base = base << STRTAB_BASE_OFF;
        base |= STRTAB_BASE_RA;
        self.rp.STRTAB_BASE.set(base as _);
        // strtab_base_cfg
        let mut cfg: usize = 0;
        cfg |= STRTAB_BASE_CFG_FMT_LINEAR << STRTAB_BASE_CFG_FMT_OFF;
        cfg |= self.strtab.get_max_sid() << STRTAB_BASE_CFG_LOG2SIZE_OFF;
        self.rp.STRTAB_BASE_CFG.set(cfg as _);
        self.init_bypass_stes();
    }

    fn init_bypass_stes(&mut self) {
        let entry_num: usize = 1 << self.strtab.get_max_sid();
        for sid in 0..entry_num {
            self.strtab.init_bypass_ste(sid);
        }
    }

    fn init_queues(&mut self) {
        self.init_cmdq();
    }

    fn init_cmdq(&mut self) {
        let idr1: usize = self.rp.IDR1.get() as _;
        let shift = extract_bits(idr1, IDR1_CMDQS_OFF, IDR1_CMDQS_LEN);
        if shift > CMDQ_MAX_SZ_SHIFT {
            self.cmdq.init(CMDQ_MAX_SZ_SHIFT as _);
        } else {
            self.cmdq.init(shift as _);
        }
        self.rp.CMDQ_BASE.set(self.cmdq.base_reg as _);
        self.rp.CMDQ_CONS.set(self.cmdq.cons);
        self.rp.CMDQ_PROD.set(self.cmdq.prod);
    }

    fn sync_write_cr0(&mut self, value: usize) {
        self.rp.CR0.set(value as _);
        for _timeout in 0..ARM_SMMU_SYNC_TIMEOUT {
            let val = self.rp.CR0ACK.get() as usize;
            if val == value {
                return;
            }
        }
        error!("CRO write err!");
    }

    fn device_reset(&mut self) {
        /* CR1 (table and queue memory attributes) */
        let mut reg = SH_ISH << CR1_TABLE_SH_OFF;
        reg |= CR1_CACHE_WB << CR1_TABLE_OC_OFF;
        reg |= CR1_CACHE_WB << CR1_TABLE_IC_OFF;
        reg |= SH_ISH << CR1_QUEUE_SH_OFF;
        reg |= CR1_CACHE_WB << CR1_QUEUE_OC_OFF;
        reg |= CR1_CACHE_WB << CR1_QUEUE_IC_OFF;
        self.rp.CR1.set(reg as _);
        let mut cr0 = CR0_SMMUEN;
        cr0 |= CR0_CMDQEN;
        self.sync_write_cr0(cr0);
    }

    // s1 bypass and s2 translate
    fn write_ste(&mut self, sid: usize, vmid: usize) {
        self.sync_ste(sid);

        assert!(vmid < MAX_ZONE_NUM, "Invalid zone id!");

        self.strtab
            .write_ste(sid, vmid, self.iommu_pt_list[vmid].root_paddr());
    }

    // invalidate the ste
    fn sync_ste(&mut self, sid: usize) {
        let cmd = self.cmdq.build_cfgi_cmd(sid);
        self.cmd_insert(cmd);
        self.sync_issue();
    }

    fn cmd_insert(&mut self, cmd: Cmd) {
        while self.cmdq.q_full() {
            self.cmdq.sync_cons(self.rp.CMDQ_CONS.get() as _);
        }
        self.cmdq.queue_write(cmd);
        self.rp.CMDQ_PROD.set(self.cmdq.inc_prod() as _);
        while !self.cmdq.q_empty() {
            self.cmdq.sync_cons(self.rp.CMDQ_CONS.get() as _);
        }
    }

    fn sync_issue(&mut self) {
        let cmd = self.cmdq.build_sync_cmd();
        self.cmd_insert(cmd);
    }
}

static SMMUV3: spin::Once<Mutex<Smmuv3>> = spin::Once::new();

/// smmuv3 init
pub fn iommu_init() {
    #[cfg(feature = "iommu")] {
        info!("Smmuv3 init...");
        SMMUV3.call_once(|| Mutex::new(Smmuv3::new()));
    }
    #[cfg(not(feature = "iommu"))]
    info!("Smmuv3 init: do nothing now");
}

/// smmuv3_base
#[allow(dead_code)]
pub fn smmuv3_base() -> usize {
    SMMU_BASE_ADDR.into()
}

/// smmuv3_size
#[allow(dead_code)]
pub fn smmuv3_size() -> usize {
    SMMU_SIZE.into()
}

/// write ste
pub fn iommu_add_device(vmid: usize, sid: usize) {
    #[cfg(feature = "iommu")] {
        let mut smmu = SMMUV3.get().unwrap().lock();
        smmu.write_ste(sid as _, vmid as _);
    }
    #[cfg(not(feature = "iommu"))]
    info!("aarch64: iommu_add_device: do nothing now, vmid: {}, sid: {}", vmid, sid);
}
