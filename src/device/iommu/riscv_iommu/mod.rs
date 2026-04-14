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
// - [x] Remove iommu from arch to device
// - [x] Add a abstract interface for all-architecture IOMMU
// - [x] Support MSI remapping
// - [ ] Complete command queue and fault queue
// - [ ] Support vIOMMU
// - [ ] Increase more fault tolerance

mod iommu_hw;

use super::Iommu;
use crate::cpu_data::this_zone;
use crate::memory::Frame;
use crate::zone::Zone;
use alloc::vec::Vec;
use iommu_hw::*;
use log::{error, info, warn};
use spin::{Mutex, Once};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};

pub(super) struct RiscvIommu;

unsafe impl Send for RiscvIommu {}
unsafe impl Sync for RiscvIommu {}

impl Iommu for RiscvIommu {
    fn initialize(&self) {
        iommu_init();
    }
    fn initialize_with_base_and_size(&self, iommu_base: usize, iommu_size: usize) {
        todo!("RiscvIommu initialize with base and size not implemented yet.");
    }
    fn add_device_with_root_pt_addr(&self, zone_id: usize, device_id: usize, root_pt: usize) {
        iommu_add_device(zone_id, device_id, root_pt);
    }
    fn add_device_share_s2pt(&self, zone_id: usize, device_id: usize) {
        todo!(
            "RiscvIommu add device with share S2PT for device id {} and VMID {}",
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
            "RiscvIommu add device with exclusive S2PT for device id {} and VMID {}",
            device_id,
            zone_id
        );
    }
    fn remove_device(&self, zone_id: usize, device_id: usize) {
        todo!(
            "RiscvIommu remove device for device id {} and VMID {}",
            device_id,
            zone_id
        );
    }
    fn interrupt_handler(&self, irq_id: usize) {
        todo!(
            "RiscvIommu interrupt handler for irq id {} not implemented yet.",
            irq_id
        );
    }
    fn viommu_init(&self, zone_id: usize) {
        todo!(
            "RiscvIommu viommu init for zone id {} not implemented yet.",
            zone_id
        );
    }
    fn viommu_mmio_handler(&self, zone: &mut Zone, viommu_base: usize, viommu_size: usize) {
        todo!(
            "RiscvIommu viommu handler for zone id {} not implemented yet.",
            zone.id()
        );
    }
}
