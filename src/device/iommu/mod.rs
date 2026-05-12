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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>

#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_mut)]

mod iommu_impl;
mod iommu_trait;

use crate::consts::MAX_ZONE_NUM;
use iommu_impl::iommu_impl;
use iommu_trait::Iommu;

#[cfg(feature = "arm_smmu")]
mod arm_smmu;
#[cfg(not(any(feature = "arm_smmu", feature = "intel_vtd", feature = "riscv_iommu")))]
mod dummy_iommu;
#[cfg(feature = "intel_vtd")]
mod intel_vtd;
#[cfg(feature = "riscv_iommu")]
mod riscv_iommu;

fn check_zone_id(zone_id: usize) -> Result<(), &'static str> {
    if zone_id >= MAX_ZONE_NUM {
        return Err("Invalid zone id for IOMMU operation");
    }
    Ok(())
}

/// Public interface for IOMMU initialization and device addition
/// It is only called during hypervisor initialization by master cpu
pub fn iommu_init() {
    iommu_impl().initialize();
}

/// Public interface for adding a device to the IOMMU
/// It can be call during VM creation, the concret implementation need
pub fn iommu_add_device_with_root_pt_addr(zone_id: usize, did: usize, root_pt_addr: usize) {
    match check_zone_id(zone_id) {
        Ok(()) => {
            iommu_impl().add_device_with_root_pt_addr(zone_id, did, root_pt_addr);
        }
        Err(e) => {
            warn!("{}", e);
        }
    }
}

/////////////////////////////////////////////////////////////////////////
// Below pub apis are used for compatibility for old code(for x86_64) //
// These apis will be replaced by IOMMU trait later.                  //
////////////////////////////////////////////////////////////////////////
#[cfg(feature = "intel_vtd")]
pub fn clear_dma_translation_tables(zone_id: usize) {
    intel_vtd::clear_dma_translation_tables(zone_id);
}

#[cfg(feature = "intel_vtd")]
pub fn fill_dma_translation_tables(zone_id: usize, zone_s2pt_hpa: crate::memory::HostPhysAddr) {
    intel_vtd::fill_dma_translation_tables(zone_id, zone_s2pt_hpa);
}

/// should be called after gpm is activated
#[cfg(feature = "intel_vtd")]
pub fn activate() {
    intel_vtd::activate();
}

#[cfg(feature = "intel_vtd")]
pub fn flush(zone_id: usize, bus: u8, dev_func: u8) {
    intel_vtd::flush(zone_id, bus, dev_func);
}

#[cfg(feature = "riscv_iommu")]
pub fn iommu_msi_pt_tlb_invalid(zone_id: u16, msi_gpa: usize) {
    riscv_iommu::iommu_msi_pt_tlb_invalid(zone_id, msi_gpa);
}
