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

use core::fmt::Debug;

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    arch::cpu::this_cpu_id,
    cpu_data::this_zone,
    device::{irqchip::inject_irq, virtio_trampoline::VirtioPCIDataInfo},
    error::HvResult,
    memory::MMIOAccess,
    pci::pci_struct::{AreaInBar, PciCapabilityRegion},
};

pub fn activate_msix() {
    // let data_info = VirtioPCIDataInfo::from_u64(data_req_id);
    // let dev_id = data_info.get_dev_id();
    let zone = this_zone();
    let zone_lock = zone.read();
    let bus = zone_lock.vpci_bus();
    let msix_backend = match bus.get_msix_backend() {
        Some(x) => x,
        None => {
            // warn!("There is no msix backend in this zone's vpci bus!");
            return;
        }
    };
    for (_, i) in bus.read_devs() {
        // if i.get_bdf().requester_id() == dev_id{
        //     i.try_inject_msix_irq();
        // }
        i.try_inject_msix_irq(&msix_backend);
    }
}

#[allow(unreachable_code)]
pub fn get_arch_msix_backend() -> Option<Arc<RwLock<dyn MsixBackend>>> {
    #[cfg(all(target_arch = "aarch64", feature = "gicv3"))]
    {
        use crate::device::irqchip::gicv3::msix_backend::get_gicv3_backend;
        return Some(get_gicv3_backend());
    }

    return None;
}

pub struct MsixCap {
    offset_in_config: usize,
    cap_id: u8,
    cap_next: u8,
    message_control: u16,
    table_bar: u8,
    table_offset: u32,
    pending_bar: u8,
    pending_offset: u32,

    msix_table: Arc<RwLock<MsixTable>>,
}

impl MsixCap {
    pub fn new(
        offset_in_config: usize,
        next: u8,
        table_size: u16,
        msix_table: Arc<RwLock<MsixTable>>,
    ) -> Self {
        let mut res = Self {
            offset_in_config,
            cap_id: 0x11,
            cap_next: next,
            message_control: 0x0,
            table_bar: 0x01,
            table_offset: 0x0000_0000,
            pending_bar: 0x01,
            pending_offset: 0x0000_08000,
            msix_table,
        };
        res.set_table_size(table_size);
        res
    }

    pub fn get_table_size(&self) -> usize {
        ((self.message_control & 0x07ff) as usize) + 1
    }

    pub fn get_table_mesg(&self) -> u32 {
        (self.table_offset << 3) | (self.table_bar as u32)
    }

    pub fn get_pending_mesg(&self) -> u32 {
        (self.pending_offset << 3) | (self.pending_bar as u32)
    }

    pub fn set_table_mesg(&mut self, mesg: u32) {
        self.table_offset = mesg >> 3;
        self.table_bar = (mesg & 0x0000_0003) as u8;
    }
    pub fn set_pending_mesg(&mut self, mesg: u32) {
        self.pending_offset = mesg >> 3;
        self.pending_bar = (mesg & 0x0000_0003) as u8;
    }

    pub fn set_table_size(&mut self, size: u16) {
        if size == 0 || size > 2048 {
            warn!("msix table size must be in 1..=2048");
            return;
        }
        let mask = 0xf800;
        self.message_control &= mask;
        self.message_control |= size - 1;
    }
}

impl PciCapabilityRegion for MsixCap {
    fn read(
        &self,
        offset: crate::pci::PciConfigAddress,
        size: usize,
    ) -> crate::error::HvResult<u32> {
        if offset as usize % size != 0 {
            warn!("cap read is misalign!");
            return Ok(0);
        }
        if size == 1 {
            match offset {
                0x00 => return Ok(self.cap_id as u32),
                0x01 => return Ok(self.cap_next as u32),
                _ => {
                    warn!("read u8 from unexpected area! offset:{}", offset);
                    return Ok(0);
                }
            }
        };
        if size == 2 {
            match offset {
                0x00 => return Ok(self.cap_id as u32 | (self.cap_next as u32) << 8),
                0x02 => return Ok(self.message_control as u32),
                _ => {
                    warn!("read u16 from unexpected area! offset:{}", offset);
                    return Ok(0);
                }
            }
        };
        if size == 4 {
            match offset {
                0x00 => {
                    return Ok((self.cap_id as u32)
                        | (self.cap_next as u32) << 8
                        | (self.message_control as u32) << 16)
                }
                0x04 => return Ok(self.get_table_mesg()),
                0x08 => return Ok(self.get_pending_mesg()),
                _ => {
                    warn!("read u32 from unexpected area! offset:{}", offset);
                    return Ok(0);
                }
            }
        };
        warn!("size is not any of 1,2,4!");
        Ok(0)
    }

    fn write(
        &mut self,
        offset: crate::pci::PciConfigAddress,
        size: usize,
        value: u32,
    ) -> crate::error::HvResult {
        if size == 1 {
            warn!("there is no writeable field with size 1!")
        }
        if size == 2 {
            match offset {
                0x02 => self.message_control = value as u16,
                _ => {
                    warn!("write into unexpected area! offset:{}", offset)
                }
            }
            return Ok(());
        }

        if size == 4 {
            match offset {
                0x04 => self.set_table_mesg(value),
                0x08 => self.set_pending_mesg(value),
                _ => {
                    warn!("write into unexpected area! offset:{}", offset)
                }
            }
            return Ok(());
        }
        Ok(())
    }

    fn get_offset(&self) -> crate::pci::PciConfigAddress {
        self.offset_in_config as u64
    }

    fn get_size(&self) -> usize {
        0x0c
    }

    fn next_cap(&self) -> crate::error::HvResult<crate::pci::PciConfigAddress> {
        Ok(self.cap_next as u64)
    }

    fn bar_usage(&self) -> Option<usize> {
        Some(self.table_bar as usize)
    }

    fn bar_area(&self) -> Option<Arc<RwLock<dyn AreaInBar>>> {
        Some(self.msix_table.clone())
    }

    fn bar_addr_range(&self) -> Option<core::ops::Range<usize>> {
        let table_offset = self.table_offset as usize;
        Some(table_offset..table_offset + self.get_table_size() * 16)
    }
}

#[derive(Clone, Debug)]
pub struct MsixTableEntry {
    pub message_address: u32,
    pub message_upper_address: u32,
    pub msg_data: u32,
    pub vector_control: u32,
    pub intid: Option<usize>,
}

impl MsixTableEntry {
    pub fn activate_irq(&self) {
        // info!("entry:{:x?}", self);
        match self.intid {
            Some(x) => {
                inject_irq(x, false);
            }
            None => {
                warn!("this msix vector has not gotten a intid:{:x?}", self);
            }
        }
    }

    pub fn get_msg_data(&self) -> u32 {
        return self.msg_data;
    }
}

impl MsixTableEntry {
    pub fn dummy() -> Self {
        Self {
            message_address: 0,
            message_upper_address: 0,
            msg_data: 0,
            vector_control: 0,
            intid: None,
        }
    }
}

#[derive(Debug)]
pub struct MsixTable {
    table: Vec<MsixTableEntry>,
    device_id: usize,
    event_id: Vec<(usize, usize)>,
    pending_msix_idx: Vec<usize>,
}

impl MsixTable {
    pub fn new(size: usize, deviceid: usize) -> Self {
        let mut vec = Vec::new();
        vec.resize(size, MsixTableEntry::dummy());
        Self {
            table: vec,
            device_id: deviceid,
            event_id: Vec::new(),
            pending_msix_idx: Vec::new(),
        }
    }

    pub fn get_entry(&self, vector_index: usize) -> Option<MsixTableEntry> {
        if vector_index >= self.table.len() {
            return None;
        }
        Some(self.table[vector_index].clone())
    }

    pub fn intercept_its(&mut self, deviceid: usize, event_id: usize, intid: usize) {
        if deviceid == self.device_id {
            warn!("MAPTI's deviceid != current deviceid!");
        }
        self.event_id.push((event_id, intid));
    }

    pub fn init_msix_intid(&mut self, index: usize) {
        let selected_vector = &mut self.table[index];
        for i in self.event_id.iter() {
            if selected_vector.msg_data as usize == i.0 {
                selected_vector.intid = Some(i.1);
                break;
            }
        }
    }

    pub fn add_pending_msix(&mut self, msix_idx: usize) {
        self.pending_msix_idx.push(msix_idx);
    }

    pub fn get_pending_msix_vector(&mut self) -> Option<Vec<MsixTableEntry>> {
        if self.pending_msix_idx.is_empty() {
            return None;
        }
        let mut res = Vec::new();
        loop {
            let idx = self.pending_msix_idx.pop();
            match idx {
                Some(x) => {
                    res.push(self.table[x].clone());
                }
                None => {
                    break;
                }
            }
        }
        return Some(res);
    }
}

impl AreaInBar for MsixTable {
    fn read(&mut self, mmio_ac: &mut MMIOAccess) -> HvResult {
        let offset = mmio_ac.address;
        // 16 is the size of entry
        let index = offset / 16;
        let offset_in_entry = offset % 16;
        if index >= self.table.len() {
            warn!("msix table read index {} out of range", index);
            return Ok(());
        }
        match offset_in_entry {
            0x00 => mmio_ac.value = self.table[index].message_address as usize,
            0x04 => mmio_ac.value = self.table[index].message_upper_address as usize,
            0x08 => mmio_ac.value = self.table[index].msg_data as usize,
            0x0c => mmio_ac.value = self.table[index].vector_control as usize,
            _ => {
                warn!("access address is misalign!");
            }
        }
        Ok(())
    }

    fn write(&mut self, mmio_ac: &MMIOAccess) -> HvResult {
        if mmio_ac.size != 4 {
            warn!("only write with size of 4 would work correctly");
        }
        let offset = mmio_ac.address;
        let index = offset / 16;
        let offset_in_entry = offset % 16;
        if index >= self.table.len() {
            warn!("msix table write index {} out of range", index);
            return Ok(());
        }
        let value = mmio_ac.value;
        match offset_in_entry {
            0x00 => self.table[index].message_address = value as u32,
            0x04 => self.table[index].message_upper_address = value as u32,
            0x08 => self.table[index].msg_data = value as u32,
            0x0c => self.table[index].vector_control = value as u32,
            _ => {
                warn!("access address is misalign!");
            }
        }
        self.init_msix_intid(index);
        Ok(())
    }
}

pub struct VirtioPCIMsixManager {
    table: BTreeMap<u64, MsixTableEntry>,
    pending: Vec<u64>,
}

impl VirtioPCIMsixManager {
    pub const fn new() -> Self {
        Self {
            table: BTreeMap::new(),
            pending: Vec::new(),
        }
    }

    pub fn insert(&mut self, data_info: VirtioPCIDataInfo, entry: MsixTableEntry) {
        // info!("Msix Manager insert:{:x?}", data_info);
        let data_req_id = data_info.get_identifier();
        self.table.insert(data_req_id, entry);
    }

    pub fn add_pending_data_req_id(&mut self, data_req_id: u64) {
        // info!("pending data req id add:0x{:x}", data_req_id);
        self.pending.push(data_req_id);
    }

    pub fn activate_all_pending_irq(&mut self) {
        let cpu_id = (this_cpu_id() as u64) << 32;
        let mut target = Vec::new();
        let mut i = 0;
        while i < self.pending.len() {
            if (self.pending[i] & 0x0000_ffff_0000_0000) == cpu_id {
                target.push(self.pending.swap_remove(i));
            } else {
                i += 1;
            }
        }

        for j in target {
            self.activate_irq(j);
        }
    }

    pub fn activate_irq(&mut self, data_req_id: u64) {
        let entry = self.table.remove(&data_req_id);
        // info!(
        //     "irq activate!!! entry:{:x?},data_req_id:0x{:x}",
        //     entry, data_req_id
        // );
        match entry {
            Some(x) => {
                x.activate_irq();
            }
            None => {
                return;
            }
        }
    }
}

pub trait MsixBackend: Debug + Sync + Send {
    fn activate_irq(&self, device_id: usize, vector: &MsixTableEntry);
    fn enable(&mut self);
    fn disable(&mut self);
}
