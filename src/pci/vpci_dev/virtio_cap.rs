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

use core::{fmt::Debug, sync::atomic::fence};

// use aarch64_cpu::registers::VTCR_EL2::SH0::Non;
use alloc::{sync::Arc, vec::Vec};
use spin::rwlock::RwLock;

use crate::{
    device::virtio_trampoline::{
        VirtioPCIConfigInfo, VirtioPCIDataInfo, VirtqueueAreaInfo, MAX_DEVS, VIRTIO_PCI_BRIDGE,
    },
    error::HvResult,
    event::{send_event, IPI_EVENT_VIRTIO_PCI_CONFIG, IPI_EVENT_VIRTIO_PCI_DATA},
    hypercall::SGI_IPI_ID,
    memory::MMIOAccess,
    pci::{
        msix::MsixTable,
        pci_struct::{AreaInBar, PciCapabilityRegion},
        vpci_dev::virtio_queue::{AvailRing, DescriptorTable, VirtqUsed},
    },
};

pub type PciCapabilityHandler = fn(&mut MMIOAccess, usize) -> HvResult;

struct VirtioPCIInterface {}

const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTIO_F_VERSION_1: usize = 32;

// #[allow(unused_variables)]
// pub unsafe fn virtio_pci_intercept_its(deviceid: usize, event_id: usize, intid: usize) {
//     #[cfg(feature = "virtio_pci")]
//     unsafe {
//         use crate::pci::msix::MAPTI_INTERCEPTOR;

//         if let Some(x) = MAPTI_INTERCEPTOR.clone() {
//             x.write().intercept_its(deviceid, event_id, intid);
//         }
//     }
// }

// #[allow(unused_variables)]
// pub unsafe fn virtio_pci_add_pending_data_req_id(data_req_id: u64) {
//     #[cfg(feature = "virtio_pci")]
//     unsafe {
//         use crate::pci::msix::VIRTIO_MSIX_MANAGER;

//         VIRTIO_MSIX_MANAGER
//             .write()
//             .add_pending_data_req_id(data_req_id);
//     }
// }

// #[allow(unused_variables)]
// pub unsafe fn virtio_pci_activate_all_pending_irq() {
//     #[cfg(feature = "virtio_pci")]
//     unsafe {
//         use crate::pci::msix::VIRTIO_MSIX_MANAGER;

//         VIRTIO_MSIX_MANAGER.write().activate_all_pending_irq();
//     }
// }

fn put_together(src: (u8, u8, u8, u8)) -> u32 {
    let a = (src.0 as u32) << 24 | (src.1 as u32) << 16 | (src.2 as u32) << 8 | (src.3 as u32);
    a
}

#[derive(Clone, Copy, Debug)]
pub enum VirtioCfgType {
    CommonCfg,
    NotifyCfg(u32),
    IsrCfg,
    DeviceCfg,
    PciCfg,
    SharedMemoryCfg,
    VendorCfg,
}

impl From<VirtioCfgType> for u8 {
    fn from(value: VirtioCfgType) -> Self {
        match value {
            VirtioCfgType::CommonCfg => 1,
            VirtioCfgType::NotifyCfg(_) => 2,
            VirtioCfgType::IsrCfg => 3,
            VirtioCfgType::DeviceCfg => 4,
            VirtioCfgType::PciCfg => 5,
            VirtioCfgType::SharedMemoryCfg => 8,
            VirtioCfgType::VendorCfg => 9,
        }
    }
}

/// Corresponding to the struct 'virtio_pci_cap' defined in virtio manual(virtio-v1.2-csd01)
#[derive(Debug)]
pub struct VirtioPciCap {
    offset_in_config: usize,
    cap_vndr: u8,
    cap_next: u8,
    cap_len: u8,
    cfg_type: VirtioCfgType,
    bar: u8,
    id: u8,
    padding: [u8; 2],
    offset: u32,
    length: u32,

    bar_usage: Arc<RwLock<dyn AreaInBar>>,
}

impl PciCapabilityRegion for VirtioPciCap {
    fn read(
        &self,
        offset: crate::pci::PciConfigAddress,
        size: usize,
    ) -> crate::error::HvResult<u32> {
        // info!("read cap:{:x},size:{}", offset, size);
        if offset as usize % size != 0 {
            warn!("cap read is misalign!");
            return Ok(0);
        }

        // if the capability read is the notifycfg, then we have to provide a multiplier field in the capability space
        // Refer to virtio-v1.2-csd01(Session 4.1.4.4:Notification structure layout)
        if let VirtioCfgType::NotifyCfg(multiplier) = self.cfg_type {
            if (offset, size) == (16, 4) {
                return Ok(multiplier);
            }
        }
        // match self.cfg_type {
        //     VirtioCfgType::NotifyCfg(multiplier) => {
        //         if (offset, size) == (16, 4) {
        //             // warn!("read multiplier!!!");
        //             return Ok(multiplier);
        //         }
        //     }
        //     _ => (),
        // };
        if size == 1 {
            match offset {
                0 => return Ok(self.cap_vndr as u32),
                1 => return Ok(self.cap_next as u32),
                2 => return Ok(self.cap_len as u32),
                3 => return Ok(u8::from(self.cfg_type) as u32),
                4 => return Ok(self.bar as u32),
                5 => return Ok(self.id as u32),
                _ => {
                    warn!("read u8 from unexpected area! offset:{}", offset);
                    return Ok(0);
                }
            }
        };
        if size == 2 {
            match offset {
                0 => return Ok(put_together((0, 0, self.cap_next, self.cap_vndr))),
                2 => return Ok(put_together((0, 0, self.cfg_type.into(), self.cap_len))),
                4 => return Ok(put_together((0, 0, self.id, self.bar))),
                _ => {
                    warn!("read u16 from unexpected area! offset:{}", offset);
                    return Ok(0);
                }
            }
        };
        if size == 4 {
            match offset {
                0 => {
                    return Ok(put_together((
                        self.cfg_type.into(),
                        self.cap_len,
                        self.cap_next,
                        self.cap_vndr,
                    )))
                }
                4 => return Ok(put_together((0, 0, self.id, self.bar))),
                8 => return Ok(self.offset),
                12 => return Ok(self.length),
                _ => {
                    warn!("read u32 from unexpected area! offset:{}", offset);
                    return Ok(0);
                }
            }
        };
        warn!("size is not any of 1,2,4!");
        Ok(0)
    }

    // All fields in virtio pci cap is read-only for guest, thus we need not to implement any write function
    fn write(
        &mut self,
        _offset: crate::pci::PciConfigAddress,
        _size: usize,
        _value: u32,
    ) -> crate::error::HvResult {
        Ok(())
    }

    fn get_offset(&self) -> crate::pci::PciConfigAddress {
        self.offset_in_config as u64
    }

    fn get_size(&self) -> usize {
        self.cap_len as usize
    }

    fn next_cap(&self) -> crate::error::HvResult<crate::pci::PciConfigAddress> {
        Ok(self.cap_next as u64)
    }

    fn bar_area(&self) -> Option<Arc<spin::RwLock<dyn AreaInBar>>> {
        Some(self.bar_usage.clone())
    }

    fn set_bar_area(&mut self, _bar_area: Arc<spin::RwLock<dyn AreaInBar>>) {
        self.bar_usage = _bar_area
    }

    fn bar_usage(&self) -> Option<usize> {
        Some(self.bar as usize)
    }

    fn bar_addr_range(&self) -> Option<core::ops::Range<usize>> {
        Some((self.offset as usize)..(self.offset + self.length) as usize)
    }
}

impl VirtioPciCap {
    pub fn new(
        offset_in_config: usize,
        config_type: VirtioCfgType,
        cap_next: u8,
        cap_len: u8,
        bar: u8,
        offset: u32,
        length: u32,
        bar_usage: Arc<RwLock<dyn AreaInBar>>,
    ) -> Self {
        // According to virtio-v1.2-csd01, every capability specially defined by virtio specification is a vender-specific capability,thus the cap_vndr is 0x09
        // Currently, most usage of bar from virtio capabilities is 0x04, but it seems doesn't matter which bar you use
        // Some device types would use multiple capabilities of a certain type, this field is intended to distinguish these capabilities.
        // If you don't have multiple capabilities of a certain type in single device, this field is useless
        Self {
            cap_vndr: 0x09,
            cap_next,
            cap_len,
            cfg_type: config_type,
            bar,
            id: 0x00,
            padding: [0, 0],
            offset,
            length,
            offset_in_config,
            bar_usage,
        }
    }
}

#[derive(Debug)]
pub struct Virtqueue {
    queue_size: u16,
    queue_msix_vector: u16,
    queue_enable: u16,
    queue_notify_off: u16,
    queue_desc: u64,
    queue_driver: u64,
    queue_device: u64,
    queue_notify_data: u16,
    queue_reset: u16,

    this_dev_id: u16,
    queue_id: u16,
    // msix_table: Arc<RwLock<MsixTable>>,
    desc_table: Option<DescriptorTable>,
    used_area: Option<VirtqUsed>,
    avail_area: Option<AvailRing>,
    last_avail: usize,
}

impl Virtqueue {
    pub fn new(queue_id: u16) -> Self {
        Self {
            queue_size: 256,
            queue_msix_vector: u16::MAX,
            queue_enable: 0,
            queue_notify_off: 0,
            queue_desc: 0,
            queue_driver: 0,
            queue_device: 0,
            queue_notify_data: 0,
            queue_reset: 0,

            this_dev_id: MAX_DEVS as u16,
            queue_id,
            // msix_table: msix,
            desc_table: None,
            used_area: None,
            avail_area: None,
            last_avail: 0,
        }
    }

    pub fn set_dev_id(&mut self, dev_id: u16) {
        self.this_dev_id = dev_id;
    }

    pub fn set_queue_id(&mut self, queue_id: u16) {
        self.queue_id = queue_id;
    }

    // pub fn notify_driver(&self) {
    //     self.msix_table
    //         .read()
    //         .inject_irq(self.queue_msix_vector as usize);
    // }

    pub fn set_desc_area(&mut self) {
        let base = self.queue_desc as usize;
        match self.desc_table {
            Some(ref mut x) => {
                warn!("this arm should not be entered");
                x.set_ptr(base);
            }
            None => {
                let desc = DescriptorTable::new(base, self.queue_size as usize);
                self.desc_table = Some(desc);
            }
        }
    }

    pub fn set_avail_area(&mut self) {
        let base = self.queue_driver as usize;
        match self.avail_area {
            Some(ref mut x) => {
                warn!("this arm should not be entered");
                x.set_ptr(base);
            }
            None => {
                let avail = AvailRing::new(base, self.queue_size);
                self.avail_area = Some(avail);
            }
        }
    }

    pub fn set_used_area(&mut self) {
        let base = self.queue_device as usize;
        match self.used_area {
            Some(ref mut x) => {
                warn!("this arm should not be entered");
                x.set_ptr(base);
            }
            None => {
                let desc = VirtqUsed::new(base, self.queue_size);
                self.used_area = Some(desc);
            }
        }
    }

    pub fn get_area_info(&self) -> VirtqueueAreaInfo {
        VirtqueueAreaInfo::new(
            self.queue_desc,
            self.queue_driver,
            self.queue_device,
            self.queue_size as u64,
        )
    }

    pub fn get_data_info(&self) -> VirtioPCIDataInfo {
        VirtioPCIDataInfo::new(self.this_dev_id, self.queue_id, self.queue_msix_vector)
    }
}

#[derive(Debug)]
pub struct VirtioPciCommonCfg {
    device_feature_select: u32,
    device_feature: (u32, u32),
    driver_feature_select: u32,
    driver_feature: (u32, u32),
    dtype: u16,
    config_msix_vector: u16,
    num_queue: u16,
    device_status: u8,
    config_generation: u8,

    queue_select: usize,
    queue_list: Vec<Arc<RwLock<Virtqueue>>>,
    config_changed: bool,
}

impl VirtioPciCommonCfg {
    pub fn new(dtype: u16) -> Self {
        let cfg = VirtioPciCommonCfg {
            device_feature_select: 0,
            device_feature: (0, 65),
            // device_feature:(0,1),
            driver_feature_select: 0,
            driver_feature: (0, 0),
            dtype,
            config_msix_vector: u16::MAX,
            num_queue: 0,
            device_status: 0,
            config_generation: 0,
            queue_select: 0,
            queue_list: Vec::new(),
            config_changed: false,
        };
        cfg
    }

    pub fn insert_queue(&mut self, qu: Arc<RwLock<Virtqueue>>) {
        self.queue_list.push(qu);
        self.num_queue += 1;
    }
}

impl VirtioPciCommonCfg {
    // This function initialize the virtqueue in root linux
    pub fn init_virtqueue_shared_space(&self) -> HvResult {
        // info!("queue_info:{:x?}", queue_info);
        let mut config_info = VirtioPCIConfigInfo::dummy();
        config_info.set_features(self.get_features());
        config_info.set_dtype(self.dtype);
        config_info.set_num_of_queues(self.num_queue);
        for i in 0..self.num_queue as usize {
            config_info.set_vqs(i, self.queue_list[i].read().get_area_info());
        }
        let mut pci_bridge = VIRTIO_PCI_BRIDGE.lock();
        let request_id = pci_bridge.push_config_req(config_info)?;
        send_event(0, SGI_IPI_ID as usize, IPI_EVENT_VIRTIO_PCI_CONFIG);
        let dev_id = pci_bridge.til_config_finish(request_id)?;
        for i in self.queue_list.iter() {
            i.write().set_dev_id(dev_id);
        }
        Ok(())
    }

    fn get_features(&self) -> u64 {
        (self.driver_feature.1 as u64) << 32 | (self.driver_feature.0 as u64)
    }

    pub fn write_into(&mut self, mmio_ac: &MMIOAccess) -> bool {
        let addr = mmio_ac.address;
        let size = mmio_ac.size;
        let value = mmio_ac.value;
        if size == 1 {
            match addr {
                0x14 => {
                    // we use FEATURES_OK to confirm that initialization is completed
                    if value & 0x04 != 0 {
                        if let Err(err) = self.init_virtqueue_shared_space() {
                            error!("virtio pci queue init failed: {:?}", err);
                            return false;
                        }
                    }
                    self.device_status = value as u8;
                    return true;
                }
                _ => {
                    warn!("write:size is misalign!");
                    return false;
                }
            }
        }

        if size == 2 {
            match addr {
                0x10 => {
                    self.config_msix_vector = value as u16;
                    return true;
                }
                0x16 => {
                    self.queue_select = value;
                    return true;
                }
                0x18 => {
                    self.queue_list[self.queue_select].write().queue_size = value as u16;
                    return true;
                }
                0x1a => {
                    // info!(
                    //     "queue No.{} has msix vector: 0x{:x}",
                    //     self.queue_select, value
                    // );
                    self.queue_list[self.queue_select].write().queue_msix_vector = value as u16;
                    return true;
                }
                0x1c => {
                    self.queue_list[self.queue_select].write().queue_enable = value as u16;
                }
                // 0x1e is read-only for driver
                _ => {
                    return false;
                }
            }
        }

        if size == 4 {
            match addr {
                0x00 => {
                    self.device_feature_select = value as u32;
                    return true;
                }
                0x08 => {
                    self.driver_feature_select = value as u32;
                    return true;
                }
                0x0c => {
                    if self.driver_feature_select == 0 {
                        self.driver_feature.0 = value as u32;
                    } else {
                        self.driver_feature.1 = value as u32;
                    }
                    return true;
                }
                0x20 => {
                    let mut queue = self.queue_list[self.queue_select].write();
                    queue.queue_desc = value as u64;
                    return true;
                }
                0x24 => {
                    let mut queue = self.queue_list[self.queue_select].write();
                    queue.queue_desc |= (value as u64) << 32;
                    queue.set_desc_area();
                    return true;
                }
                0x28 => {
                    self.queue_list[self.queue_select].write().queue_driver = value as u64;
                    return true;
                }
                0x2c => {
                    let mut queue = self.queue_list[self.queue_select].write();
                    queue.queue_driver |= (value as u64) << 32;
                    queue.set_avail_area();
                    return true;
                }
                0x30 => {
                    self.queue_list[self.queue_select].write().queue_device = value as u64;
                    return true;
                }
                0x34 => {
                    let mut queue = self.queue_list[self.queue_select].write();
                    queue.queue_device |= (value as u64) << 32;
                    queue.set_used_area();
                    return true;
                }
                _ => {
                    warn!("write:not implement yet!addr:{:x}", addr);
                    return false;
                }
            }
        }
        false
    }
}

impl AreaInBar for VirtioPciCommonCfg {
    fn read(&mut self, mmio_ac: &mut MMIOAccess) -> HvResult {
        let addr = mmio_ac.address;
        let size = mmio_ac.size;
        if size == 1 {
            match addr {
                0x14 => {
                    mmio_ac.value = self.device_status as usize;
                }
                0x15 => {
                    mmio_ac.value = self.config_generation as usize;
                }
                _ => {
                    warn!("read:size is misalign!");
                    return Ok(());
                }
            }
            // info!("read from common cfg:0x{:x}",mmio_ac.value);
        }

        if size == 2 {
            match addr {
                0x10 => {
                    mmio_ac.value = self.config_msix_vector as usize;
                }
                0x12 => {
                    mmio_ac.value = self.num_queue as usize;
                }
                0x18 => {
                    mmio_ac.value = self.queue_list[self.queue_select].read().queue_size as usize;
                }
                0x1a => {
                    mmio_ac.value =
                        self.queue_list[self.queue_select].read().queue_msix_vector as usize;
                }
                0x1c => {
                    mmio_ac.value = self.queue_list[self.queue_select].read().queue_enable as usize;
                }
                0x1e => {
                    mmio_ac.value =
                        self.queue_list[self.queue_select].read().queue_notify_off as usize;
                }
                _ => {
                    warn!("read:size is misalign!");
                    return Ok(());
                }
            }
            // info!("read from common cfg:0x{:x}",mmio_ac.value);
        }

        if size == 4 {
            match addr {
                0x04 => {
                    if self.device_feature_select == 0 {
                        mmio_ac.value = self.device_feature.0 as usize;
                    } else {
                        mmio_ac.value = self.device_feature.1 as usize;
                    }
                }
                _ => {
                    warn!("read:not implement yet!addr:{:x}", mmio_ac.value);
                    return Ok(());
                }
            }
            // info!("read from common cfg:0x{:x}",mmio_ac.value);
        }
        Ok(())
    }

    fn write(&mut self, mmio_ac: &MMIOAccess) -> HvResult {
        if self.write_into(mmio_ac) {
            return Ok(());
        }
        warn!("the write has not reached the common config!");
        Ok(())
    }
}

#[derive(Debug)]
pub struct VirtioISRCap {
    isr: u32,
}

impl VirtioISRCap {
    pub fn new() -> Self {
        Self { isr: 0 }
    }

    pub fn set_isr(&mut self, value: u32) {
        self.isr = value
    }
}

impl AreaInBar for VirtioISRCap {
    fn read(&mut self, mmio_ac: &mut MMIOAccess) -> HvResult {
        let offset = mmio_ac.address;
        match offset {
            0x00 => {
                mmio_ac.value = self.isr as usize;
                self.isr = 0;
            }
            _ => {
                warn!("illegal isr address");
            }
        };
        return Ok(());
    }

    fn write(&mut self, _mmio_ac: &MMIOAccess) -> HvResult {
        warn!("isr should not be write");
        return Ok(());
    }
}

#[derive(Debug)]
pub struct VirtioNotifyCap {
    // cap: VirtioPciCap,
    queue_list: Vec<(usize, Arc<RwLock<Virtqueue>>)>,
    msix_table: Arc<RwLock<MsixTable>>,
}

impl VirtioNotifyCap {
    pub fn new(msix_table: Arc<RwLock<MsixTable>>) -> Self {
        Self {
            queue_list: Vec::new(),
            msix_table,
        }
    }

    pub fn insert_queue(
        &mut self,
        qu: Arc<RwLock<Virtqueue>>,
        cap_notify_offset: u32,
        multiplier: u32,
    ) {
        let queue_notify_off = qu.read().queue_notify_off as u32;
        let offset = cap_notify_offset + queue_notify_off * multiplier;
        self.queue_list.push((offset as usize, qu));
        // if let VirtioCfgType::NotifyCfg(multiplier) = self.cap.cfg_type {
        //     let offset = self.cap.offset + queue_notify_off * multiplier;
        //     self.queue_list.push((offset as usize, qu));
        // } else {
        //     error!("Notify cap has to have NotifyCfg type!");
        // }
    }

    fn get_queues(&self, offset: usize) -> impl Iterator<Item = &Arc<RwLock<Virtqueue>>> {
        self.queue_list
            .iter()
            .filter(move |(x, _)| *x == offset)
            .map(|(_, res)| res)
    }
}

impl AreaInBar for VirtioNotifyCap {
    fn read(&mut self, _mmio_ac: &mut MMIOAccess) -> HvResult {
        warn!("Notify read has not been implemented yet");
        Ok(())
    }

    fn write(&mut self, mmio_ac: &MMIOAccess) -> HvResult {
        let offset = mmio_ac.address;
        for i in self.get_queues(offset) {
            let info = i.read().get_data_info();
            self.msix_table
                .write()
                .add_pending_msix(info.get_msix_vector_idx() as usize);
            if let Err(err) = VIRTIO_PCI_BRIDGE.lock().push_data_req(info) {
                error!("virtio pci data enqueue failed: {:?}", err);
                return Err(err);
            }
            send_event(0, SGI_IPI_ID as usize, IPI_EVENT_VIRTIO_PCI_DATA);
            fence(core::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}
