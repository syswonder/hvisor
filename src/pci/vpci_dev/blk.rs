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

use alloc::sync::Arc;
use spin::rwlock::RwLock;

use super::VpciDeviceHandler;
use crate::cpu_data::this_zone;
use crate::memory::MMIOAccess;
use crate::pci::msix::{MsixCap, MsixTable};
use crate::pci::pci_access::{
    BaseClass, DeviceId, DeviceRevision, Interface, PciMemType, SubClass, VendorId,
};
use crate::pci::pci_struct::{AreaInBar, CapabilityType, PciCapability};
use crate::pci::vpci_dev::virtio_cap::{
    VirtioISRCap, VirtioNotifyCap, VirtioPciCap, VirtioPciCommonCfg, Virtqueue,
};
use crate::{error::HvResult, pci::pci_struct::VirtualPciConfigSpace};

const VIRTIO_BLK_CAPACITY_SECTORS: u64 = 64 * 2 * 1024;
const VIRTIO_BLK_SECTOR_SIZE: u32 = 512;
const VIRTIO_BLK_CFG_OFFSET: usize = 0x3000;

#[derive(Debug)]
struct VirtioBlkCfg {
    offset_in_bar: usize,
    capacity_sectors: u64,
    blk_size: u32,
}

impl VirtioBlkCfg {
    fn new() -> Self {
        Self {
            offset_in_bar: VIRTIO_BLK_CFG_OFFSET,
            capacity_sectors: VIRTIO_BLK_CAPACITY_SECTORS,
            blk_size: VIRTIO_BLK_SECTOR_SIZE,
        }
    }

    fn read_field(&self, offset: usize, size: usize) -> usize {
        let Some(offset) = offset.checked_sub(self.offset_in_bar) else {
            warn!(
                "virtio-blk device cfg read below range: offset={offset:#x} base={:#x} size={size}",
                self.offset_in_bar
            );
            return 0;
        };
        let mut buf = [0u8; 24];
        buf[0..8].copy_from_slice(&self.capacity_sectors.to_le_bytes());
        buf[8..12].copy_from_slice(&0u32.to_le_bytes()); // size_max
        buf[12..16].copy_from_slice(&0u32.to_le_bytes()); // seg_max
        buf[20..24].copy_from_slice(&self.blk_size.to_le_bytes());

        let end = offset.saturating_add(size);
        if end > buf.len() {
            warn!("virtio-blk device cfg read out of range: offset={offset:#x} size={size}");
            return 0;
        }

        match size {
            1 => buf[offset] as usize,
            2 => u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize,
            4 => u32::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]) as usize,
            8 => u64::from_le_bytes([
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
                buf[offset + 4],
                buf[offset + 5],
                buf[offset + 6],
                buf[offset + 7],
            ]) as usize,
            _ => {
                warn!("virtio-blk device cfg unsupported read size {size}");
                0
            }
        }
    }
}

impl AreaInBar for VirtioBlkCfg {
    fn read(&mut self, mmio_ac: &mut MMIOAccess) -> HvResult {
        mmio_ac.value = self.read_field(mmio_ac.address, mmio_ac.size);
        Ok(())
    }

    fn write(&mut self, mmio_ac: &MMIOAccess) -> HvResult {
        warn!(
            "virtio-blk device cfg is read-only: addr={:#x} size={}",
            mmio_ac.address, mmio_ac.size
        );
        Ok(())
    }
}

pub struct VirtioBlkHandler;

impl VpciDeviceHandler for VirtioBlkHandler {
    fn vdev_init(&self, mut dev: VirtualPciConfigSpace) -> VirtualPciConfigSpace {
        let id: (DeviceId, VendorId) = (0x1042, 0x1af4);
        let revision: DeviceRevision = 0xFFu8;
        let base_class: BaseClass = 0x01;
        let sub_class: SubClass = 0x80;
        let interface: Interface = 0x00;
        dev.with_config_value_mut(|config_value| {
            config_value.set_id(id);
            config_value.set_class_and_revision_id((base_class, sub_class, interface, revision));
        });
        dev.with_bararr_mut(|bararr| {
            bararr[1].config_init(
                PciMemType::Mem32,
                false,
                0x4000,
                0x0,
                Some(blk_mmio_handler),
            );
            bararr[4].config_init(
                PciMemType::Mem64Low,
                true,
                0x10000,
                0x0,
                Some(blk_mmio_handler),
            );
            bararr[5].config_init(
                PciMemType::Mem64High,
                true,
                0x0,
                0x0,
                Some(blk_mmio_handler),
            );
        });

        let commoncfg_bar = arc_rwlock!(VirtioPciCommonCfg::new(3));
        let commoncfg_cap = arc_rwlock!(VirtioPciCap::new(
            0x40,
            super::virtio_cap::VirtioCfgType::CommonCfg,
            0,
            0x10,
            0x04,
            0x0,
            0x1000,
            commoncfg_bar.clone()
        ));
        let commoncfg = PciCapability::new_virt(commoncfg_cap);

        let isr_bar: Arc<RwLock<VirtioISRCap>> = Arc::new(RwLock::new(VirtioISRCap::new()));
        let isr_cap = arc_rwlock!(VirtioPciCap::new(
            0x50,
            super::virtio_cap::VirtioCfgType::IsrCfg,
            0x40,
            0x10,
            0x04,
            0x1000,
            0x1000,
            isr_bar.clone()
        ));
        let isr = PciCapability::new_virt(isr_cap);
        isr_bar.write().set_isr(0);

        let msix_table: Arc<RwLock<MsixTable>> = Arc::new(RwLock::new(MsixTable::new(
            0x10,
            dev.get_bdf().requester_id() as usize,
        )));
        let msix_cap = arc_rwlock!(MsixCap::new(0x74, 0x60, 0x10, msix_table.clone()));
        let msix = PciCapability::new_cap(CapabilityType::MsiX, msix_cap);

        let notify_bar = arc_rwlock!(VirtioNotifyCap::new(msix_table.clone()));
        let notify_cap = arc_rwlock!(VirtioPciCap::new(
            0x60,
            crate::pci::vpci_dev::virtio_cap::VirtioCfgType::NotifyCfg(0x04),
            0x50,
            0x14,
            0x04,
            0x2000,
            0x1000,
            notify_bar.clone()
        ));
        let notify = PciCapability::new_virt(notify_cap);

        let devcfg_bar = arc_rwlock!(VirtioBlkCfg::new());
        let devcfg_cap = arc_rwlock!(VirtioPciCap::new(
            0x88,
            crate::pci::vpci_dev::virtio_cap::VirtioCfgType::DeviceCfg,
            0x74,
            0x10,
            0x04,
            VIRTIO_BLK_CFG_OFFSET as u32,
            0x1000,
            devcfg_bar
        ));
        let devcfg = PciCapability::new_virt(devcfg_cap);

        dev.set_msix_table(msix_table.clone());

        dev.with_cap_mut(|capabilities| {
            capabilities.insert_cap(commoncfg);
            capabilities.insert_cap(isr);
            capabilities.insert_cap(notify);
            capabilities.insert_cap(msix);
            capabilities.insert_cap(devcfg);
            capabilities.set_capability_pointer(0x88);
        });

        dev.with_access_mut(|access| {
            access.set_bits(0x34..0x38);
            access.set_bits(0x40..0x50);
            access.set_bits(0x50..0x60);
            access.set_bits(0x60..0x70);
            access.set_bits(0x70..0x90);
            access.set_bits(0x88..0x98);
        });

        let vq: Arc<RwLock<Virtqueue>> = arc_rwlock!(Virtqueue::new(0));
        commoncfg_bar.write().insert_queue(vq.clone());
        notify_bar.write().insert_queue(vq, 0x2000, 0x04);
        dev
    }
}

pub const HANDLER: VirtioBlkHandler = VirtioBlkHandler;

pub fn blk_mmio_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    let zone = this_zone();
    let zone_lock = zone.read();
    let bus = zone_lock.vpci_bus();
    let (mut dev, mut bar) = (None, 0);
    for (_, i) in bus.read_devs() {
        if let Some(res) = i.is_my_bar_addr(base) {
            dev = Some(i.clone());
            bar = res;
            break;
        }
    }
    if let Some(found_dev) = dev {
        return found_dev.bar_mmio_distribute(bar, mmio);
    }
    Ok(())
}
