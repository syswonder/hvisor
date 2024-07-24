use alloc::sync::Arc;
use alloc::vec::Vec;
use psci::error::INVALID_ADDRESS;
use spin::RwLock;

use crate::arch::mm::new_s2_memory_set;
use crate::arch::s2pt::Stage2PageTable;
use crate::config::{HvZoneConfig, CONFIG_NAME_MAXLEN};
use crate::consts::MAX_CPU_NUM;

use crate::error::HvResult;
use crate::memory::addr::GuestPhysAddr;
use crate::memory::{MMIOConfig, MMIOHandler, MMIORegion, MemorySet};
use crate::percpu::{get_cpu_data, this_zone, CpuSet};
use core::panic;

pub struct Zone {
    pub name: [u8; CONFIG_NAME_MAXLEN],
    pub id: usize,
    pub mmio: Vec<MMIOConfig>,
    pub cpu_set: CpuSet,
    pub irq_bitmap: [u32; 1024 / 32],
    pub gpm: MemorySet<Stage2PageTable>,
}

impl Zone {
    pub fn new(zoneid: usize, name: &[u8]) -> Self {
        Self {
            name: name.try_into().unwrap(),
            id: zoneid,
            gpm: new_s2_memory_set(),
            cpu_set: CpuSet::new(MAX_CPU_NUM as usize, 0),
            mmio: Vec::new(),
            irq_bitmap: [0; 1024 / 32],
        }
    }

    // pub fn suspend(&self) {
    //     trace!("suspending cpu_set = {:#x?}", self.cpu_set);
    //     self.cpu_set.iter_except(this_cpu_id()).for_each(|cpu_id| {
    //         trace!("try to suspend cpu_id = {:#x?}", cpu_id);
    //         suspend_cpu(cpu_id);
    //     });
    //     info!("send sgi done!");
    // }

    // pub fn resume(&self) {
    //     trace!("resuming cpu_set = {:#x?}", self.cpu_set);
    //     self.cpu_set.iter_except(this_cpu_id()).for_each(|cpu_id| {
    //         trace!("try to resume cpu_id = {:#x?}", cpu_id);
    //         resume_cpu(cpu_id);
    //     });
    // }

    // pub fn owns_cpu(&self, id: usize) -> bool {
    //     self.cpu_set.contains_cpu(id)
    // }

    /// Register a mmio region and its handler.
    pub fn mmio_region_register(
        &mut self,
        start: GuestPhysAddr,
        size: usize,
        handler: MMIOHandler,
        arg: usize,
    ) {
        if let Some(mmio) = self.mmio.iter_mut().find(|mmio| mmio.region.start == start) {
            warn!("duplicated mmio region {:#x?}", mmio);
            if mmio.region.size != size {
                panic!("duplicated mmio region size not match");
            }
            mmio.handler = handler;
            mmio.arg = arg;
        } else {
            self.mmio.push(MMIOConfig {
                region: MMIORegion { start, size },
                handler,
                arg,
            })
        }
    }
    #[allow(dead_code)]
    /// Remove the mmio region beginning at `start`.
    pub fn mmio_region_remove(&mut self, start: GuestPhysAddr) {
        if let Some((idx, _)) = self
            .mmio
            .iter()
            .enumerate()
            .find(|(_, mmio)| mmio.region.start == start)
        {
            self.mmio.remove(idx);
        }
    }
    /// Find the mmio region contains (addr..addr+size).
    pub fn find_mmio_region(
        &self,
        addr: GuestPhysAddr,
        size: usize,
    ) -> Option<(MMIORegion, MMIOHandler, usize)> {
        self.mmio
            .iter()
            .find(|cfg| cfg.region.contains_region(addr, size))
            .map(|cfg| (cfg.region, cfg.handler, cfg.arg))
    }
    /// If irq_id belongs to this zone
    pub fn irq_in_zone(&self, irq_id: u32) -> bool {
        let idx = (irq_id / 32) as usize;
        let bit_pos = (irq_id % 32) as usize;
        (self.irq_bitmap[idx] & (1 << bit_pos)) != 0
    }
}

static ZONE_LIST: RwLock<Vec<Arc<RwLock<Zone>>>> = RwLock::new(vec![]);

pub fn root_zone() -> Arc<RwLock<Zone>> {
    ZONE_LIST.read().get(0).cloned().unwrap()
}

pub fn is_this_root_zone() -> bool {
    Arc::ptr_eq(&this_zone(), &root_zone())
}

/// Add zone to CELL_LIST
pub fn add_zone(zone: Arc<RwLock<Zone>>) {
    ZONE_LIST.write().push(zone);
}

/// Remove zone from ZONE_LIST
pub fn remove_zone(zone_id: usize) {
    let mut zone_list = ZONE_LIST.write();
    let (idx, _) = zone_list
        .iter()
        .enumerate()
        .find(|(_, zone)| zone.read().id == zone_id)
        .unwrap();
    let removed_zone = zone_list.remove(idx);
    assert_eq!(Arc::strong_count(&removed_zone), 1);
}

pub fn find_zone(zone_id: usize) -> Option<Arc<RwLock<Zone>>> {
    ZONE_LIST
        .read()
        .iter()
        .find(|zone| zone.read().id == zone_id)
        .cloned()
}

pub fn all_zones_info() -> Vec<ZoneInfo> {
    let zone_list = ZONE_LIST.read();

    zone_list.iter().map(|zone| {
        let zone_lock = zone.read();
        ZoneInfo {
            zone_id: zone_lock.id as u32,
            cpus: zone_lock.cpu_set.bitmap,
            name: zone_lock.name.clone(),
        }
    }).collect()
}

pub fn this_zone_id() -> usize {
    this_zone().read().id
}

// #[repr(C)]
// #[derive(Debug, Clone)]
// pub struct ZoneConfig {
//     pub zone_id: u32,
//     pub cpus: u64,
//     pub num_memory_regions: u32,
//     pub memory_regions: [MemoryRegion; CONFIG_MAX_MEMORY_REGIONS],
//     pub num_interrupts: u32,
//     pub interrupts: [u32; CONFIG_MAX_INTERRUPTS],
//     pub entry_point: u64,
//     pub dtb_load_paddr: u64,
// }

pub fn zone_create(config: &HvZoneConfig) -> HvResult<Arc<RwLock<Zone>>> {
    // we create the new zone here
    // TODO: create Zone with cpu_set
    let zone_id = config.zone_id as usize;

    if find_zone(zone_id).is_some() {
        return hv_result_err!(EEXIST);
    }

    let mut zone = Zone::new(zone_id, &config.name);
    zone.pt_init(config.memory_regions()).unwrap();
    zone.mmio_init(&config.arch_config);
    zone.irq_bitmap_init(config.interrupts());

    config.cpus().iter().for_each(|cpu_id| {
        zone.cpu_set.set_bit(*cpu_id as _);
    });

    // pub struct HvConfigMemoryRegion {
    //     pub mem_type: u32,
    //     pub physical_start: u64,
    //     pub virtual_start: u64,
    //     pub size: u64,
    // }
    let mut dtb_ipa = INVALID_ADDRESS as u64;
    for region in config.memory_regions() {
        // region contains config.dtb_load_paddr?
        if region.physical_start <= config.dtb_load_paddr
            && region.physical_start + region.size > config.dtb_load_paddr
        {
            dtb_ipa = region.virtual_start + config.dtb_load_paddr - region.physical_start;
        }
    }
    info!("zone cpu_set: {:#b}", zone.cpu_set.bitmap);
    let cpu_set = zone.cpu_set;

    let new_zone_pointer = Arc::new(RwLock::new(zone));
    {
        cpu_set.iter().for_each(|cpuid| {
            let cpu_data = get_cpu_data(cpuid);
            cpu_data.zone = Some(new_zone_pointer.clone());
            //chose boot cpu
            if cpuid == cpu_set.first_cpu().unwrap() {
                cpu_data.boot_cpu = true;
            }
            cpu_data.cpu_on_entry = config.entry_point as _;
            cpu_data.dtb_ipa = dtb_ipa as _;
        });
    }
    add_zone(new_zone_pointer.clone());

    Ok(new_zone_pointer)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    zone_id: u32,
    cpus: u64,
    name: [u8; CONFIG_NAME_MAXLEN]
}
