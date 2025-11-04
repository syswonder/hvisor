use core::usize;

use aarch64_cpu::registers::Readable;

use crate::arch::{
    mpam::{
        config::MpamPartConfig,
        error::MpamResult,
        id::init_partid_allocator,
        info::dump_mpamidr_el1,
        regs::{MpamIdrEL1, MpamRegisters, MPAMF_IDR::PARTID_MAX},
    },
    sysreg::write_sysreg,
};

use super::{
    mpam::msc::{MscNode, ResourceNode},
    sysreg::read_sysreg,
};

mod cache;
mod config;
mod error;
mod id;
mod info;
mod mem;
mod msc;
mod regs;

pub use config::{MPAM_SYSTEM_CONFIG, TOTAL_MEM_BW};
pub use id::{alloc_partid, dealloc_partid};
#[allow(unused)]
pub use info::dump_mpam_registers;
pub use msc::MPAM_NODES;

pub fn mpam_verison() -> u8 {
    let id_aa64pfr0_el1 = read_sysreg!(id_aa64pfr0_el1);
    let major = (id_aa64pfr0_el1 >> 40) & 0xf;
    let id_aa64pfr1_el1 = read_sysreg!(id_aa64pfr1_el1);
    let minor = (id_aa64pfr1_el1 >> 16) & 0xf;
    ((major << 4) | minor) as u8
}

/// x between 0.0 and 1.0
fn simple_round(x: f64) -> f64 {
    let i = x as i64;
    let frac = x - i as f64;
    if x >= 0.0 {
        if frac >= 0.5 {
            (i + 1) as f64
        } else {
            i as f64
        }
    } else {
        if frac <= -0.5 {
            (i - 1) as f64
        } else {
            i as f64
        }
    }
}

pub fn configure_mpam_for_system(
    msc_nodes: &[(MscNode, &[ResourceNode])],
    configs: &[MpamPartConfig],
    total_mem_bw: usize,
) -> MpamResult<()> {
    let mut min_max_partid = u64::MAX;
    for (msc, resources) in msc_nodes {
        let mpam_registers = MpamRegisters::new(msc.base_address());
        let partid_max = mpam_registers.mpamf_idr.read(PARTID_MAX);
        if partid_max < min_max_partid {
            min_max_partid = partid_max;
        }
        for res in *resources {
            match res.locator_type() {
                0x00 => {
                    // Cache
                    for config in configs {
                        if let Some(percentage) = config.cache_percentage {
                            info!(
                                "Configure cache partition: PARTID={}, percentage={}%",
                                config.partid, percentage
                            );
                            mpam_registers.set_part_cmax_ratio(
                                config.partid as u16,
                                res.ris_index() as u8,
                                false,
                                (percentage as f64) / 100.0,
                            )?;
                        }
                    }
                }
                0x01 => {
                    // Memory
                    for config in configs {
                        if let (Some(max_bw), Some(min_bw)) = (config.mem_max_bw, config.mem_min_bw)
                        {
                            info!(
                                "Configure memory partition: PARTID={}, max_bw={}MB/s, min_bw={}MB/s",
                                config.partid, max_bw, min_bw
                            );
                            mpam_registers.set_part_mbw_max_ratio(
                                config.partid as u16,
                                res.ris_index() as u8,
                                false,
                                if max_bw == 0 {
                                    0.0
                                } else {
                                    max_bw as f64 / total_mem_bw as f64
                                },
                                false,
                            )?;
                            mpam_registers.set_part_mbw_min_ratio(
                                config.partid as u16,
                                res.ris_index() as u8,
                                false,
                                if min_bw == 0 {
                                    0.0
                                } else {
                                    min_bw as f64 / total_mem_bw as f64
                                },
                            )?;
                        }
                    }
                }
                _ => {
                    warn!("Unsupported resource locator type: {}", res.locator_type());
                }
            }
        }
    }
    init_partid_allocator(min_max_partid as usize + 1);
    Ok(())
}

#[inline(always)]
fn make_mpam_bundle(mpamen: bool, partid_d: u16, partid_i: u16, pmg_d: u8, pmg_i: u8) -> u64 {
    let mut v = 0u64;
    if mpamen {
        v |= 1 << 63;
    }
    v |= (pmg_i as u64) << 32;
    v |= (pmg_d as u64) << 40;
    v |= (partid_d as u64) << 16;
    v |= partid_i as u64;
    v
}

/// hvisor(EL2)
pub fn mpam2_el2_init_partid0() {
    let mpamidr_el1 = MpamIdrEL1::read();
    if mpamidr_el1.has_hcr() {
        let val = make_mpam_bundle(true, 0, 0, 0, 0);
        let ori_val = read_sysreg!(MPAM2_EL2);
        info!("Original MPAM2_EL2: {:#X}", ori_val);
        info!("Set MPAM2_EL2 for hvisor with PARTID 0");
        write_sysreg!(MPAM2_EL2, val);
        info!("Disable traps for MPAM registers in EL2");
        let mut cur = read_sysreg!(MPAM2_EL2);
        // disable trap
        cur &= !(1 << 49);
        cur &= !(1 << 48);
        cur &= !(1 << 58);
        write_sysreg!(MPAM2_EL2, cur);
    } else {
        warn!(
            "Platform does not implement MPAM hypervisor control (HAS_HCR=0). \
               Skip writing MPAM2_EL2 to avoid UNDEFINED instruction."
        );
    }
}

/// guest OS(EL1)
pub fn mpam_el1_zone_partid_enable(partid_d: u16, partid_i: u16) {
    let val = make_mpam_bundle(true, partid_d, partid_i, 0, 0);
    write_sysreg!(MPAM1_EL1, val);
}

pub fn mpam_enable(partid_d: u16, partid_i: u16) {
    dump_mpamidr_el1();
    mpam2_el2_init_partid0();
    mpam_el1_zone_partid_enable(partid_d, partid_i);
}

pub fn mpam_disable() {
    write_sysreg!(MPAM1_EL1, 0);
}
