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

#![allow(unused)]

mod smmu_hw;

use super::Iommu;
use crate::cpu_data::this_zone;
use crate::memory::Frame;
use crate::zone::Zone;
use alloc::vec::Vec;
use log::{error, info, warn};
use smmu_hw::*;
use spin::{Mutex, Once};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};

pub(super) struct ArmSmmu;

unsafe impl Send for ArmSmmu {}
unsafe impl Sync for ArmSmmu {}

impl Iommu for ArmSmmu {
    fn initialize(&self) {
        iommu_init();
    }
    fn initialize_with_base_and_size(&self, iommu_base: usize, iommu_size: usize) {
        todo!("ArmSmmu initialize with base and size not implemented yet.");
    }
    fn add_device_with_root_pt_addr(&self, zone_id: usize, device_id: usize, root_pt: usize) {
        iommu_add_device(zone_id, device_id, root_pt);
    }
    fn add_device_share_s2pt(&self, zone_id: usize, device_id: usize) {
        todo!(
            "ArmSmmu add device with share S2PT for device id {} and VMID {}",
            device_id,
            zone_id
        );
    }
    fn add_device_exclusive_s2pt(
        &self,
        zone_id: usize,
        device_id: usize,
        regions: alloc::vec::Vec<crate::memory::MemoryRegion<crate::memory::GuestPhysAddr>>,
    ) {
        todo!(
            "ArmSmmu add device with exclusive S2PT for device id {} and VMID {}",
            device_id,
            zone_id
        );
    }
    fn remove_device(&self, zone_id: usize, device_id: usize) {
        todo!(
            "ArmSmmu remove device for device id {} and VMID {}",
            device_id,
            zone_id
        );
    }
    fn interrupt_handler(&self, irq_id: usize) {
        todo!(
            "ArmSmmu interrupt handler for irq id {} not implemented yet.",
            irq_id
        );
    }
    fn viommu_init(&self, zone_id: usize) {
        todo!(
            "ArmSmmu viommu init for zone id {} not implemented yet.",
            zone_id
        );
    }
    fn viommu_remove(&self, zone_id: usize) {
        todo!(
            "ArmSmmu viommu remove for zone id {} not implemented yet.",
            zone_id
        );
    }
    fn viommu_mmio_handler_register(&self, zone: &Zone, _viommu_base: usize, _viommu_size: usize) {
        todo!(
            "ArmSmmu viommu handler for zone id {} not implemented yet.",
            zone.id()
        );
    }
}
