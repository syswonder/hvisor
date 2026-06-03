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

use crate::memory::{GuestPhysAddr, MemoryRegion};
use crate::zone::Zone;
use alloc::vec::Vec;

/// IOMMU trait defining the required methods for diffent IOMMU implementations
#[rustfmt::skip]
pub(crate) trait Iommu {
    fn initialize(&self);
    /// Initialize the IOMMU hardware and data structures
    fn initialize_with_base_and_size(&self, iommu_base: usize, iommu_size: usize);
    /// Add a device with the given VMID, device ID and root page table to the IOMMU
    fn add_device_with_root_pt_addr(&self, zone_id: usize, device_id: usize, root_pt_addr: usize);
    /// Add a device with the given VMID and device ID to the IOMMU, device share s2pt with zone
    fn add_device_share_s2pt(&self, zone_id: usize, device_id: usize);
    /// Add a device with exclusive stage 2 page table mappings (i.e. iopagetable)
    fn add_device_exclusive_s2pt(&self, zone_id: usize, device_id: usize, regions: Vec<MemoryRegion<GuestPhysAddr>>);
    /// Remove a device from the IOMMU
    fn remove_device(&self, zone_id: usize, device_id: usize);
    /// Handle IOMMU-related interrupts
    fn interrupt_handler(&self, irq_id: usize);
    /// Initialize the Virtual IOMMU for the Zone
    fn viommu_init(&self, zone_id: usize);
    /// Remove the Virtual IOMMU for the Zone
    fn viommu_remove(&self, zone_id: usize);
    /// Register the Virtual IOMMU MMIO handler for the Zone
    fn viommu_mmio_handler_register(&self, zone: &Zone, viommu_base: usize, viommu_size: usize);
}
