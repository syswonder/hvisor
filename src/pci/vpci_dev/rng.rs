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
use crate::pci::pci_struct::{CapabilityType, PciCapability};
use crate::pci::vpci_dev::virtio_cap::{
    VirtioISRCap, VirtioNotifyCap, VirtioPciCap, VirtioPciCommonCfg, Virtqueue,
};
use crate::{error::HvResult, pci::pci_struct::VirtualPciConfigSpace};

pub struct VirtioRngHandler;

impl VpciDeviceHandler for VirtioRngHandler {
    fn vdev_init(&self, mut dev: VirtualPciConfigSpace) -> VirtualPciConfigSpace {
        let id: (DeviceId, VendorId) = (0x1044, 0x1af4);
        let revision: DeviceRevision = 0xFFu8;
        let base_class: BaseClass = 0x0;
        let sub_class: SubClass = 0x0;
        let interface: Interface = 0x0;
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
                Some(rng_mmio_handler),
            );
            bararr[4].config_init(
                PciMemType::Mem64Low,
                true,
                0x10000,
                0x0,
                Some(rng_mmio_handler),
            );
            bararr[5].config_init(
                PciMemType::Mem64High,
                true,
                0x0,
                0x0,
                Some(rng_mmio_handler),
            );
        });
        let commoncfg_bar = arc_rwlock!(VirtioPciCommonCfg::new(4));
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
        let notify = PciCapability::new_virt(notify_cap.clone());
        dev.set_msix_table(msix_table.clone());

        dev.with_cap_mut(|capabilities| {
            capabilities.insert_cap(commoncfg);
            capabilities.insert_cap(isr);
            capabilities.insert_cap(notify);
            capabilities.insert_cap(msix);
            capabilities.set_capability_pointer(0x74);
        });

        dev.with_access_mut(|access| {
            access.set_bits(0x34..0x38);
            access.set_bits(0x40..0x50);
            access.set_bits(0x50..0x60);
            access.set_bits(0x60..0x70);
            access.set_bits(0x70..0x90);
        });

        let vq: Arc<RwLock<Virtqueue>> = arc_rwlock!(Virtqueue::new(0));
        commoncfg_bar.write().insert_queue(vq.clone());
        notify_bar.write().insert_queue(vq, 0x2000, 0x04);
        dev
    }
}

pub const HANDLER: VirtioRngHandler = VirtioRngHandler;

pub fn rng_mmio_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
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
