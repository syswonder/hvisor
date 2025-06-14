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

use alloc::collections::btree_map::BTreeMap;
use spin::Mutex;

use crate::device::irqchip::set_ispender;
use crate::{
    config::{HvIvcConfig, CONFIG_MAX_IVC_CONGIGS},
    consts::PAGE_SIZE,
    error::HvResult,
    memory::{Frame, GuestPhysAddr, MMIOAccess, MemFlags, MemoryRegion},
    zone::{this_zone_id, Zone},
};

// ivc_id -> ivc_record
static IVC_RECORDS: Mutex<BTreeMap<u32, IvcRecord>> = Mutex::new(BTreeMap::new());
// zone id -> zone's IvcInfo
pub static IVC_INFOS: Mutex<BTreeMap<usize, IvcInfo>> = Mutex::new(BTreeMap::new());

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
/// The ivc info that one zone should first accquire
pub struct IvcInfo {
    /// The number that one zone participates in ivc region
    pub len: u64,
    /// The ivc control table ipa of each ivc region
    ivc_ct_ipas: [u64; CONFIG_MAX_IVC_CONGIGS],
    /// The ivc shared memory ipa of each ivc region
    ivc_shmem_ipas: [u64; CONFIG_MAX_IVC_CONGIGS],
    /// The ivc_id of each ivc region
    ivc_ids: [u32; CONFIG_MAX_IVC_CONGIGS],
    /// The irq number of each ivc region
    ivc_irqs: [u32; CONFIG_MAX_IVC_CONGIGS],
}

impl From<&[HvIvcConfig]> for IvcInfo {
    fn from(configs: &[HvIvcConfig]) -> Self {
        let mut ivc_ids = [0; CONFIG_MAX_IVC_CONGIGS];
        let mut ivc_ct_ipas = [0; CONFIG_MAX_IVC_CONGIGS];
        let mut ivc_shmem_ipas = [0; CONFIG_MAX_IVC_CONGIGS];
        let mut ivc_irqs = [0; CONFIG_MAX_IVC_CONGIGS];
        for i in 0..configs.len() {
            let config = &configs[i];
            ivc_ids[i] = config.ivc_id;
            ivc_ct_ipas[i] = config.control_table_ipa;
            ivc_shmem_ipas[i] = config.shared_mem_ipa;
            ivc_irqs[i] = config.interrupt_num;
        }
        Self {
            len: configs.len() as u64,
            ivc_ids,
            ivc_shmem_ipas,
            ivc_ct_ipas,
            ivc_irqs,
        }
    }
}
fn insert_ivc_record(ivc_config: &HvIvcConfig, zone_id: u32) -> Result<(bool, usize), ()> {
    let mut recs = IVC_RECORDS.lock();
    let ivc_id = ivc_config.ivc_id;
    if let Some(rec) = recs.get_mut(&ivc_id) {
        if rec.max_peers != ivc_config.max_peers
            || rec.rw_sec_size != ivc_config.rw_sec_size
            || rec.out_sec_size != ivc_config.out_sec_size
        {
            error!("ivc config conflicts!!!");
            return Err(());
        }
        if rec.peer_infos.len() == rec.max_peers as _ {
            error!("can't add more peers to ivc_id {}", ivc_id);
            return Err(());
        }
        rec.peer_infos.insert(
            ivc_config.peer_id,
            PeerInfo {
                zone_id,
                irq_num: ivc_config.interrupt_num,
                shared_mem_ipa: ivc_config.shared_mem_ipa,
            },
        );
        Ok((false, rec.shared_mem.start_paddr()))
    } else {
        if ivc_config.rw_sec_size as usize % PAGE_SIZE != 0
            || ivc_config.out_sec_size as usize % PAGE_SIZE != 0
        {
            error!("section size must be page aligned!!!");
            return Err(());
        }
        let mut rec = IvcRecord::from(ivc_config);
        let start_paddr = rec.shared_mem.start_paddr();
        rec.peer_infos.insert(
            ivc_config.peer_id,
            PeerInfo {
                zone_id,
                irq_num: ivc_config.interrupt_num,
                shared_mem_ipa: ivc_config.shared_mem_ipa,
            },
        );
        recs.insert(ivc_id, rec);
        Ok((true, start_paddr))
    }
}

struct IvcRecord {
    max_peers: u32,
    rw_sec_size: u32,
    out_sec_size: u32,
    // peer id -> PeerInfo
    peer_infos: BTreeMap<u32, PeerInfo>,
    shared_mem: Frame,
}

#[allow(unused)]
struct PeerInfo {
    zone_id: u32,
    irq_num: u32,
    shared_mem_ipa: u64,
}

impl From<&HvIvcConfig> for IvcRecord {
    fn from(config: &HvIvcConfig) -> Self {
        let frames = Frame::new_contiguous(
            ((config.rw_sec_size + config.out_sec_size * config.max_peers) / PAGE_SIZE as u32)
                as usize,
            0,
        )
        .unwrap();
        Self {
            max_peers: config.max_peers,
            rw_sec_size: config.rw_sec_size,
            out_sec_size: config.out_sec_size,
            peer_infos: BTreeMap::new(),
            shared_mem: frames,
        }
    }
}

impl Zone {
    pub fn ivc_init(&mut self, ivc_configs: &[HvIvcConfig]) {
        for ivc_config in ivc_configs {
            // is_new is ok to remove
            if let Ok((_, start_paddr)) = insert_ivc_record(ivc_config, self.id as _) {
                info!(
                    "ivc init: zone {}'s shared mem begins at {:x}, ipa is {:x}",
                    self.id, start_paddr, ivc_config.shared_mem_ipa
                );
                let rw_sec_size: usize = ivc_config.rw_sec_size as usize;
                let out_sec_size: usize = ivc_config.out_sec_size as usize;
                self.gpm
                    .insert(MemoryRegion::new_with_offset_mapper(
                        ivc_config.shared_mem_ipa as _,
                        start_paddr,
                        rw_sec_size as _,
                        MemFlags::READ | MemFlags::WRITE,
                    ))
                    .unwrap();
                for i in 0..ivc_config.max_peers as usize {
                    let flags = if i == ivc_config.peer_id as _ {
                        MemFlags::READ | MemFlags::WRITE
                    } else {
                        MemFlags::READ
                    };
                    self.gpm
                        .insert(MemoryRegion::new_with_offset_mapper(
                            ivc_config.shared_mem_ipa as usize + rw_sec_size + i * out_sec_size,
                            start_paddr + rw_sec_size + i * out_sec_size,
                            out_sec_size as _,
                            flags,
                        ))
                        .unwrap();
                }
                self.mmio_region_register(
                    ivc_config.control_table_ipa as _,
                    PAGE_SIZE,
                    mmio_ivc_handler,
                    ivc_config.control_table_ipa as _,
                );
            } else {
                return;
            }
        }
        IVC_INFOS.lock().insert(self.id, IvcInfo::from(ivc_configs));
    }
}

const CT_IVC_ID: GuestPhysAddr = 0x00;
const CT_MAX_PEERS: GuestPhysAddr = 0x04;
const CT_RW_SEC_SIZE: GuestPhysAddr = 0x08;
const CT_OUT_SEC_SIZE: GuestPhysAddr = 0x0C;
const CT_PEER_ID: GuestPhysAddr = 0x10;
const CT_IPI_INVOKE: GuestPhysAddr = 0x14;

pub fn mmio_ivc_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    let zone_id = this_zone_id();
    let is_write = mmio.is_write;
    let ivc_infos = IVC_INFOS.lock();
    let ivc_info = ivc_infos.get(&zone_id).unwrap();
    let ivc_id = (0..ivc_info.len as usize)
        .find(|&i| ivc_info.ivc_ct_ipas[i] == base as _)
        .map(|i| ivc_info.ivc_ids[i])
        .unwrap();
    drop(ivc_infos);
    let recs = IVC_RECORDS.lock();
    let rec = recs.get(&ivc_id).unwrap();
    mmio.value = match mmio.address {
        CT_IVC_ID => ivc_id as usize,
        CT_MAX_PEERS => rec.max_peers as usize,
        CT_RW_SEC_SIZE => rec.rw_sec_size as usize,
        CT_OUT_SEC_SIZE => rec.out_sec_size as usize,
        CT_PEER_ID => {
            let peer_id = rec
                .peer_infos
                .iter()
                .find(|&(_, info)| info.zone_id == zone_id as _)
                .map(|(peer_id, _)| *peer_id)
                .unwrap();
            peer_id as usize
        }
        CT_IPI_INVOKE if is_write => {
            let peer_id = mmio.value as u32;
            let irq_num = match rec.peer_infos.get(&peer_id) {
                Some(info) => info.irq_num,
                None => {
                    error!("zone {} has no peer {}", zone_id, peer_id);
                    return hv_result_err!(EINVAL);
                }
            } as usize;
            set_ispender(irq_num / 32, 1 << (irq_num % 32));
            return Ok(());
        }
        _ => return hv_result_err!(EFAULT),
    };
    Ok(())
}
