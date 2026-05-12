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

use super::Iommu;
use crate::memory::{GuestPhysAddr, MemoryRegion};
use crate::zone::Zone;
use alloc::vec::Vec;

/// Dummy IOMMU implementation for systems without IOMMU support
pub(super) struct DummyIommu;

#[rustfmt::skip]
impl Iommu for DummyIommu {
    fn initialize(&self) {
        info!("No IOMMU implementation available, skipping initialization");
    }
    fn initialize_with_base_and_size(&self, iommu_base: usize, iommu_size: usize) {
        info!("No IOMMU implementation available, skipping initialization, base: {:#x}, size: {:#x}", iommu_base, iommu_size);
    }
    fn add_device_with_root_pt_addr(&self, zone_id: usize, device_id: usize, root_pt_addr: usize) {
        info!("No IOMMU implementation available, cannot add device id {} for VMID {} with root page table {:#x}", device_id, zone_id, root_pt_addr);
    }
    fn add_device_share_s2pt(&self, zone_id: usize, device_id: usize) {
        info!("No IOMMU implementation available, cannot add device id {} for VMID {}", device_id, zone_id);
    }
    fn add_device_exclusive_s2pt(&self, zone_id: usize, device_id: usize, _regions: Vec<MemoryRegion<GuestPhysAddr>>) {
        info!("No IOMMU implementation available, cannot add device id {} for VMID {} with exclusive S2PT", device_id, zone_id);
    }
    fn remove_device(&self, zone_id: usize, device_id: usize) {
        info!("No IOMMU implementation available, cannot remove device id {} for VMID {}", device_id, zone_id);
    }
    fn interrupt_handler(&self, irq_id: usize) {
        info!("No IOMMU implementation available, cannot handle interrupt id {}", irq_id);
    }
    fn viommu_init(&self, zone_id: usize) {
        info!("No IOMMU implementation available, cannot initialize VIOMMU for Zone id {}", zone_id);
    }
    fn viommu_mmio_handler(&self, zone: &mut Zone, _viommu_base: usize, _viommu_size: usize) {
        info!("No IOMMU implementation available, cannot handle VIOMMU MMIO for Zone id {}", zone.id());
    }
}
