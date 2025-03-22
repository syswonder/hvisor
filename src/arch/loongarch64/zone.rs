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
//      Yulong Han <wheatfox17@icloud.com>
//
use crate::{
    config::*,
    consts::PAGE_SIZE,
    device::virtio_trampoline::mmio_virtio_handler,
    error::HvResult,
    memory::{
        addr::{align_down, align_up},
        mmio_generic_handler, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion,
    },
    zone::Zone,
};
use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{fence, Ordering};

impl Zone {
    pub fn pt_init(&mut self, mem_regions: &[HvConfigMemoryRegion]) -> HvResult {
        // use the new zone config type of init
        for region in mem_regions {
            trace!("loongarch64: pt_init: process region: {:#x?}", region);
            let mem_type = region.mem_type;
            match mem_type {
                MEM_TYPE_RAM => {
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        region.virtual_start as GuestPhysAddr,
                        region.physical_start as HostPhysAddr,
                        region.size as _,
                        MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
                    ))?;
                }
                MEM_TYPE_IO => {
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        region.virtual_start as GuestPhysAddr,
                        region.physical_start as HostPhysAddr,
                        region.size as _,
                        MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
                    ))?;
                }
                MEM_TYPE_VIRTIO => {
                    info!(
                        "loongarch64: pt_init: register virtio mmio region: {:#x?}",
                        region
                    );
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        region.virtual_start as GuestPhysAddr,
                        region.physical_start as HostPhysAddr,
                        PAGE_SIZE, // since we only need 0x200 size for virtio mmio, but the minimal size is PAGE_SIZE
                        MemFlags::USER, // we use the USER as a hint flag for invalidating this stage-2 PTE
                    ))?;
                    self.mmio_region_register(
                        region.physical_start as _,
                        region.size as _,
                        mmio_virtio_handler,
                        region.physical_start as _,
                    );
                }
                _ => {
                    error!("loongarch64: pt_init: unknown mem type: {}", mem_type);
                    return hv_result_err!(EINVAL);
                }
            }
        }
        debug!("zone stage-2 memory set: {:#x?}", self.gpm);
        unsafe {
            // test the page table by querying the first page
            if mem_regions.len() > 0 {
                let r = self
                    .gpm
                    .page_table_query(mem_regions[0].virtual_start as GuestPhysAddr);
                debug!("query 0x{:x}: {:#x?}", mem_regions[0].virtual_start, r);
                // check whether the first page is mapped
                let va = mem_regions[0].virtual_start as GuestPhysAddr;
                let result_pa = r.unwrap().0;
                if result_pa != mem_regions[0].physical_start as HostPhysAddr {
                    error!(
                        "loongarch64: pt_init: page table test failed: va: {:#x}, pa: {:#x}, expected pa: {:#x}",
                        va, result_pa, mem_regions[0].physical_start
                    );
                    return hv_result_err!(EINVAL, "page table test failed");
                }
            }
        }
        Ok(())
    }
    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        warn!("loongarch64: mm: isa_init do nothing");
    }
    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {}
}

pub fn disable_hwi_through() {
    info!("loongarch64: disable_hwi_through");
    use crate::arch::register::*;
    gintc::set_hwip(0x0); // stop passing through all 8 HWIs
}

pub fn enable_hwi_through() {
    info!("loongarch64: enable_hwi_through");
    use crate::arch::register::*;
    gintc::set_hwip(0xff); // pass through all HWI7-0
}

#[repr(C)]
#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct LoongArch64ZoneContext {
    pub x: [usize; 32],
    pub sepc: usize,
    // General Control and Status Registers
    pub gcsr_crmd: usize,   // CRMD
    pub gcsr_prmd: usize,   // PRMD
    pub gcsr_euen: usize,   // EUEN
    pub gcsr_misc: usize,   // MISC
    pub gcsr_ectl: usize,   // ECTL
    pub gcsr_estat: usize,  // ESTAT
    pub gcsr_era: usize,    // ERA
    pub gcsr_badv: usize,   // BADV
    pub gcsr_badi: usize,   // BADI
    pub gcsr_eentry: usize, // EENTRY

    // TLB Registers
    pub gcsr_tlbidx: usize,  // TLBIDX
    pub gcsr_tlbehi: usize,  // TLBEHI
    pub gcsr_tlbelo0: usize, // TLBELO0
    pub gcsr_tlbelo1: usize, // TLBELO1

    // Page Table Registers
    pub gcsr_asid: usize, // ASID
    pub gcsr_pgdl: usize, // PGDL
    pub gcsr_pgdh: usize, // PGDH
    pub gcsr_pgd: usize,  // PGD
    pub gcsr_pwcl: usize, // PWCL
    pub gcsr_pwch: usize, // PWCH

    // Second Level TLB Registers
    pub gcsr_stlbps: usize, // STLBPS
    pub gcsr_ravcfg: usize, // RAVCFG

    // Processor Registers
    pub gcsr_cpuid: usize,  // CPUID
    pub gcsr_prcfg1: usize, // PRCFG1
    pub gcsr_prcfg2: usize, // PRCFG2
    pub gcsr_prcfg3: usize, // PRCFG3

    // Saved Registers
    pub gcsr_save0: usize,  // SAVE0
    pub gcsr_save1: usize,  // SAVE1
    pub gcsr_save2: usize,  // SAVE2
    pub gcsr_save3: usize,  // SAVE3
    pub gcsr_save4: usize,  // SAVE4
    pub gcsr_save5: usize,  // SAVE5
    pub gcsr_save6: usize,  // SAVE6
    pub gcsr_save7: usize,  // SAVE7
    pub gcsr_save8: usize,  // SAVE8
    pub gcsr_save9: usize,  // SAVE9
    pub gcsr_save10: usize, // SAVE10
    pub gcsr_save11: usize, // SAVE11
    pub gcsr_save12: usize, // SAVE12
    pub gcsr_save13: usize, // SAVE13
    pub gcsr_save14: usize, // SAVE14
    pub gcsr_save15: usize, // SAVE15

    // Timer Registers
    pub gcsr_tid: usize,   // TID
    pub gcsr_tcfg: usize,  // TCFG
    pub gcsr_tval: usize,  // TVAL
    pub gcsr_cntc: usize,  // CNTC
    pub gcsr_ticlr: usize, // TICLR

    // Load Linked Buffers Registers
    pub gcsr_llbctl: usize, // LLBCTL

    // TLB Read Entry Registers
    pub gcsr_tlbrentry: usize, // TLBRENTRY
    pub gcsr_tlbrbadv: usize,  // TLBRBADV
    pub gcsr_tlbrera: usize,   // TLBRERA
    pub gcsr_tlbrsave: usize,  // TLBRSAVE
    pub gcsr_tlbrelo0: usize,  // TLBRELO0
    pub gcsr_tlbrelo1: usize,  // TLBRELO1
    pub gcsr_tlbrehi: usize,   // TLBREHI
    pub gcsr_tlbrprmd: usize,  // TLBRPRMD

    // Data Memory Write Registers
    pub gcsr_dmw0: usize, // DMW0
    pub gcsr_dmw1: usize, // DMW1
    pub gcsr_dmw2: usize, // DMW2
    pub gcsr_dmw3: usize, // DMW3

    // Pagetable address
    pub pgdl: usize,
    pub pgdh: usize,
}

macro_rules! gprs_getters {
  ($($reg_name:ident, $index:expr),*) => {
      $(
          pub fn $reg_name(&self) -> usize {
              self.x[$index]
          }
      )*
  }
}

macro_rules! gprs_setters {
  ($($set_name:ident, $index:expr),*) => {
      $(
          pub fn $set_name(&mut self, val: usize) {
              self.x[$index] = val;
          }
      )*
  }
}

impl LoongArch64ZoneContext {
    pub const fn new() -> LoongArch64ZoneContext {
        LoongArch64ZoneContext {
            x: [0; 32],
            sepc: 0,
            gcsr_crmd: 0,
            gcsr_prmd: 0,
            gcsr_euen: 0,
            gcsr_misc: 0,
            gcsr_ectl: 0,
            gcsr_estat: 0,
            gcsr_era: 0,
            gcsr_badv: 0,
            gcsr_badi: 0,
            gcsr_eentry: 0,
            gcsr_tlbidx: 0,
            gcsr_tlbehi: 0,
            gcsr_tlbelo0: 0,
            gcsr_tlbelo1: 0,
            gcsr_asid: 0,
            gcsr_pgdl: 0,
            gcsr_pgdh: 0,
            gcsr_pgd: 0,
            gcsr_pwcl: 0,
            gcsr_pwch: 0,
            gcsr_stlbps: 0,
            gcsr_ravcfg: 0,
            gcsr_cpuid: 0,
            gcsr_prcfg1: 0,
            gcsr_prcfg2: 0,
            gcsr_prcfg3: 0,
            gcsr_save0: 0,
            gcsr_save1: 0,
            gcsr_save2: 0,
            gcsr_save3: 0,
            gcsr_save4: 0,
            gcsr_save5: 0,
            gcsr_save6: 0,
            gcsr_save7: 0,
            gcsr_save8: 0,
            gcsr_save9: 0,
            gcsr_save10: 0,
            gcsr_save11: 0,
            gcsr_save12: 0,
            gcsr_save13: 0,
            gcsr_save14: 0,
            gcsr_save15: 0,
            gcsr_tid: 0,
            gcsr_tcfg: 0,
            gcsr_tval: 0,
            gcsr_cntc: 0,
            gcsr_ticlr: 0,
            gcsr_llbctl: 0,
            gcsr_tlbrentry: 0,
            gcsr_tlbrbadv: 0,
            gcsr_tlbrera: 0,
            gcsr_tlbrsave: 0,
            gcsr_tlbrelo0: 0,
            gcsr_tlbrelo1: 0,
            gcsr_tlbrehi: 0,
            gcsr_tlbrprmd: 0,
            gcsr_dmw0: 0,
            gcsr_dmw1: 0,
            gcsr_dmw2: 0,
            gcsr_dmw3: 0,
            // pagetable of zone
            pgdl: 0,
            pgdh: 0,
        }
    }

    pub fn print_zone_context(&self) {
        info!("=============ZONE CONTEXT============");
        // get self addr in memory
        let self_addr = self as *const _ as usize;
        info!("self addr: {:#x}", self_addr);
        for (index, &register) in self.x.iter().enumerate() {
            info!("$r[{}]: {:#x}", index, register);
        }
        info!("sepc: {:#x}", self.sepc);
        info!("gcsr_crmd: {:#x}", self.gcsr_crmd);
        info!("gcsr_prmd: {:#x}", self.gcsr_prmd);
        info!("gcsr_euen: {:#x}", self.gcsr_euen);
        info!("gcsr_misc: {:#x}", self.gcsr_misc);
        info!("gcsr_ectl: {:#x}", self.gcsr_ectl);
        info!("gcsr_estat: {:#x}", self.gcsr_estat);
        info!("gcsr_era: {:#x}", self.gcsr_era);
        info!("gcsr_badv: {:#x}", self.gcsr_badv);
        info!("gcsr_badi: {:#x}", self.gcsr_badi);
        info!("gcsr_eentry: {:#x}", self.gcsr_eentry);
        info!("gcsr_tlbidx: {:#x}", self.gcsr_tlbidx);
        info!("gcsr_tlbehi: {:#x}", self.gcsr_tlbehi);
        info!("gcsr_tlbelo0: {:#x}", self.gcsr_tlbelo0);
        info!("gcsr_tlbelo1: {:#x}", self.gcsr_tlbelo1);
        info!("gcsr_asid: {:#x}", self.gcsr_asid);
        info!("gcsr_pgdl: {:#x}", self.gcsr_pgdl);
        info!("gcsr_pgdh: {:#x}", self.gcsr_pgdh);
        info!("gcsr_pgd: {:#x}", self.gcsr_pgd);
        info!("gcsr_pwcl: {:#x}", self.gcsr_pwcl);
        info!("gcsr_pwch: {:#x}", self.gcsr_pwch);
        info!("gcsr_stlbps: {:#x}", self.gcsr_stlbps);
        info!("gcsr_ravcfg: {:#x}", self.gcsr_ravcfg);
        info!("gcsr_cpuid: {:#x}", self.gcsr_cpuid);
        info!("gcsr_prcfg1: {:#x}", self.gcsr_prcfg1);
        info!("gcsr_prcfg2: {:#x}", self.gcsr_prcfg2);
        info!("gcsr_prcfg3: {:#x}", self.gcsr_prcfg3);
        info!("gcsr_save0: {:#x}", self.gcsr_save0);
        info!("gcsr_save1: {:#x}", self.gcsr_save1);
        info!("gcsr_save2: {:#x}", self.gcsr_save2);
        info!("gcsr_save3: {:#x}", self.gcsr_save3);
        info!("gcsr_save4: {:#x}", self.gcsr_save4);
        info!("gcsr_save5: {:#x}", self.gcsr_save5);
        info!("gcsr_save6: {:#x}", self.gcsr_save6);
        info!("gcsr_save7: {:#x}", self.gcsr_save7);
        info!("gcsr_save8: {:#x}", self.gcsr_save8);
        info!("gcsr_save9: {:#x}", self.gcsr_save9);
        info!("gcsr_save10: {:#x}", self.gcsr_save10);
        info!("gcsr_save11: {:#x}", self.gcsr_save11);
        info!("gcsr_save12: {:#x}", self.gcsr_save12);
        info!("gcsr_save13: {:#x}", self.gcsr_save13);
        info!("gcsr_save14: {:#x}", self.gcsr_save14);
        info!("gcsr_save15: {:#x}", self.gcsr_save15);
        info!("gcsr_tid: {:#x}", self.gcsr_tid);
        info!("gcsr_tcfg: {:#x}", self.gcsr_tcfg);
        info!("gcsr_tval: {:#x}", self.gcsr_tval);
        info!("gcsr_cntc: {:#x}", self.gcsr_cntc);
        info!("gcsr_ticlr: {:#x}", self.gcsr_ticlr);
        info!("gcsr_llbctl: {:#x}", self.gcsr_llbctl);
        info!("gcsr_tlbrentry: {:#x}", self.gcsr_tlbrentry);
        info!("gcsr_tlbrbadv: {:#x}", self.gcsr_tlbrbadv);
        info!("gcsr_tlbrera: {:#x}", self.gcsr_tlbrera);
        info!("gcsr_tlbrsave: {:#x}", self.gcsr_tlbrsave);
        info!("gcsr_tlbrelo0: {:#x}", self.gcsr_tlbrelo0);
        info!("gcsr_tlbrelo1: {:#x}", self.gcsr_tlbrelo1);
        info!("gcsr_tlbrehi: {:#x}", self.gcsr_tlbrehi);
        info!("gcsr_tlbrprmd: {:#x}", self.gcsr_tlbrprmd);
        info!("gcsr_dmw0: {:#x}", self.gcsr_dmw0);
        info!("gcsr_dmw1: {:#x}", self.gcsr_dmw1);
        info!("gcsr_dmw2: {:#x}", self.gcsr_dmw2);
        info!("gcsr_dmw3: {:#x}", self.gcsr_dmw3);
        info!("pgdl: {:#x}", self.pgdl);
        info!("pgdh: {:#x}", self.pgdh);
    }

    gprs_getters!(
        get_ra, 1, get_a0, 4, get_a1, 5, get_a2, 6, get_a3, 7, get_a4, 8, get_a5, 9, get_a6, 10,
        get_a7, 11
    );
    gprs_setters!(set_a0, 4);
}

pub type ZoneContext = LoongArch64ZoneContext;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HvArchZoneConfig {
    pub dummy: usize,
}
