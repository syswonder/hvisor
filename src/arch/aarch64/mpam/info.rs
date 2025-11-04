#![allow(unused)]

use crate::arch::sysreg::read_sysreg;

use super::cache::{MPAMF_CCAP_IDR, MPAMF_CPOR_IDR};
use super::mem::MPAMF_MBW_IDR;
use aarch64_cpu::registers::Readable;

use super::{msc::mpam_get_msc_node_base_address_iter, regs::*};

pub fn dump_mpam_registers() {
    let base_addresses = mpam_get_msc_node_base_address_iter();
    for base in base_addresses {
        let mpam_mmio_registers = MpamRegisters::new(base);
        let mpamf_idr = &mpam_mmio_registers.mpamf_idr;
        info!("MPAMF_IDR at base address {:#X}:", base);
        info!("\tPARTID_NRW: {}", mpamf_idr.read(MPAMF_IDR::PARTID_NRW));
        info!("\tMSMON: {}", mpamf_idr.read(MPAMF_IDR::MSMON));
        info!("\tIMPL_IDR: {}", mpamf_idr.read(MPAMF_IDR::IMPL_IDR));
        info!("\tEXT: {}", mpamf_idr.read(MPAMF_IDR::EXT));
        info!("\tPRI_PART: {}", mpamf_idr.read(MPAMF_IDR::PRI_PART));
        info!("\tMBW_PART: {}", mpamf_idr.read(MPAMF_IDR::MBW_PART));
        info!("\tCPOR_PART: {}", mpamf_idr.read(MPAMF_IDR::CPOR_PART));
        info!("\tCCAP_PART: {}", mpamf_idr.read(MPAMF_IDR::CCAP_PART));
        info!("\tPMG_MAX: {}", mpamf_idr.read(MPAMF_IDR::PMG_MAX));
        info!("\tPARTID_MAX: {}", mpamf_idr.read(MPAMF_IDR::PARTID_MAX));

        if mpamf_idr.is_set(MPAMF_IDR::CPOR_PART) {
            info!("MPAMF_CPOR_IDR:");
            let mpamf_cpor_idr = &mpam_mmio_registers.mpamf_cpor_idr;
            info!(
                "\tCPBM_WD: {}",
                mpamf_cpor_idr.read(MPAMF_CPOR_IDR::CPBM_WD)
            );
        }

        if mpamf_idr.is_set(MPAMF_IDR::CCAP_PART) {
            let mpamf_ccap_idr = &mpam_mmio_registers.mpamf_ccap_idr;
            info!("\tMPAMF_CCPR_IDR:");
            info!(
                "\tHAS_CMAX_SOFTLIM: {}",
                mpamf_ccap_idr.read(MPAMF_CCAP_IDR::HAS_CMAX_SOFTLIM)
            );
            info!(
                "\tNO_CMAX: {}",
                mpamf_ccap_idr.read(MPAMF_CCAP_IDR::NO_CMAX)
            );
            info!(
                "\tHAS_CMIN: {}",
                mpamf_ccap_idr.read(MPAMF_CCAP_IDR::HAS_CMIN)
            );
            info!(
                "\tHAS_CASSOC: {}",
                mpamf_ccap_idr.read(MPAMF_CCAP_IDR::HAS_CASSOC)
            );
            info!(
                "\tCASSOC_WD: {}",
                mpamf_ccap_idr.read(MPAMF_CCAP_IDR::CASSOC_WD)
            );
            info!(
                "\tCMAX_WD: {}",
                mpamf_ccap_idr.read(MPAMF_CCAP_IDR::CMAX_WD)
            );
        }

        if mpamf_idr.is_set(MPAMF_IDR::MBW_PART) {
            let mpamf_mbw_idr = &mpam_mmio_registers.mpamf_mbw_idr;
            info!("MPAMF_MBW_IDR:");
            info!(
                "\tBWPBM_WD: {}",
                mpamf_mbw_idr.read(MPAMF_MBW_IDR::BWPBM_WD)
            );
            info!("\tWINDWR: {}", mpamf_mbw_idr.read(MPAMF_MBW_IDR::WINDWR));
            info!(
                "\tHAS_PROP: {}",
                mpamf_mbw_idr.read(MPAMF_MBW_IDR::HAS_PROP)
            );
            info!("\tHAS_PBM: {}", mpamf_mbw_idr.read(MPAMF_MBW_IDR::HAS_PBM));
            info!("\tHAS_MAX: {}", mpamf_mbw_idr.read(MPAMF_MBW_IDR::HAS_MAX));
            info!("\tHAS_MIN: {}", mpamf_mbw_idr.read(MPAMF_MBW_IDR::HAS_MIN));
            info!("\tMAX_LIM: {}", mpamf_mbw_idr.read(MPAMF_MBW_IDR::MAX_LIM));
            info!("\tBWA_WD: {}", mpamf_mbw_idr.read(MPAMF_MBW_IDR::BWA_WD));
        }
    }
}

pub fn dump_mpamidr_el1() {
    let mpamidr_el1 = read_sysreg!(MPAMIDR_EL1);
    info!("MPAMIDR_EL1: {:#X}", mpamidr_el1);
    info!("\tHAS_SDEFLT: {}", (mpamidr_el1 >> 61) & 0x1);
    info!("\tHAS_FORCE_NS: {}", (mpamidr_el1 >> 60) & 0x1);
    info!("\tHAS_TIDR: {}", (mpamidr_el1 >> 58) & 0x1);
    info!("\tPMG_MAX: {}", (mpamidr_el1 >> 32) & 0xff);
    info!("\tVPMR_MAX: {}", (mpamidr_el1 >> 18) & 0x7);
    info!("\tHAS_HCR: {}", (mpamidr_el1 >> 17) & 0x1);
    info!("\tPARTID_MAX: {}", mpamidr_el1 & 0xffff);
}
