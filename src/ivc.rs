use alloc::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use spin::Mutex;

use crate::{config::HvIvcConfig, consts::PAGE_SIZE, memory::{Frame, MemFlags, MemoryRegion}, zone::Zone};
// ivc_id -> ivc_record
static IVC_RECORDS: Mutex<BTreeMap<u32, IvcRecord>> = Mutex::new(BTreeMap::new());

fn insert_ivc_record(ivc_config: &HvIvcConfig, zone_id: u32) -> Result<(bool, usize), ()> {
    let mut recs = IVC_RECORDS.lock();
    let ivc_id = ivc_config.ivc_id;
    if let Some(rec) = recs.get_mut(&ivc_id) {
        if rec.max_peers != ivc_config.max_peers || rec.rw_sec_size != ivc_config.rw_sec_size ||
            rec.out_sec_size != ivc_config.out_sec_size {
                error!("ivc config conflicts!!!");
                return Err(());
            }
        if rec.id2irq.keys().len() == rec.max_peers as _{
            error!("can't add more peers to ivc_id {}", ivc_id);
            return Err(());
        }
        rec.id2irq.insert(zone_id, ivc_config.interrupt_num);
        Ok((false, rec.shared_mem.start_paddr()))
    } else {
        if ivc_config.rw_sec_size as usize % PAGE_SIZE != 0 || ivc_config.out_sec_size as usize % PAGE_SIZE != 0 {
            error!("section size must be page aligned!!!");
            return Err(());
        }
        let mut rec = IvcRecord::from(ivc_config);
        let start_paddr = rec.shared_mem.start_paddr();
        rec.id2irq.insert(zone_id, ivc_config.interrupt_num);
        recs.insert(ivc_id, rec);
        Ok((true, start_paddr))
    }
}

struct IvcRecord {
    max_peers: u32,
    rw_sec_size: u32,
    out_sec_size: u32,
    id2irq: BTreeMap<u32, u32>, // zone id -> irq number
    shared_mem: Frame,
}

impl From<&HvIvcConfig> for IvcRecord {
    fn from(config: &HvIvcConfig) -> Self {
        let frames = Frame::new_contiguous(((config.rw_sec_size + config.out_sec_size * config.max_peers) / PAGE_SIZE as u32) as usize, 0).unwrap();
        Self {
            max_peers: config.max_peers,
            rw_sec_size: config.rw_sec_size,
            out_sec_size: config.out_sec_size,
            id2irq: BTreeMap::new(),
            shared_mem: frames,
        }
    }
}

impl Zone {
    pub fn ivc_init(&mut self, ivc_configs: &[HvIvcConfig]) {
        for ivc_config in ivc_configs {
            // add interrupt.
            // is_new is ok to remove
            if let Ok((is_new, start_paddr)) = insert_ivc_record(ivc_config, self.id as _) {
                info!("ivc init: zone {}'s shared mem begins at {:x}, ipa is {:x}", self.id, start_paddr, ivc_config.shared_mem_ipa);
                let max_peers = ivc_config.max_peers;
                let rw_sec_size: usize = ivc_config.rw_sec_size as usize;
                let out_sec_size: usize = ivc_config.out_sec_size as usize;
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    ivc_config.shared_mem_ipa as _,
                    start_paddr,
                    rw_sec_size as _,
                    MemFlags::READ | MemFlags::WRITE)).unwrap();
                for i in 0..ivc_config.max_peers as usize{
                    let flags = if i == ivc_config.peer_id as _{
                        MemFlags::READ | MemFlags::WRITE
                    } else {
                        MemFlags::READ
                    };
                    self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                        ivc_config.shared_mem_ipa as usize + rw_sec_size + i * out_sec_size,
                        start_paddr + rw_sec_size + i * out_sec_size,
                        out_sec_size as _,
                        flags)).unwrap();
                }
            } else {
                return ;
            }
        }
    }
}