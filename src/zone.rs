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
use alloc::vec::Vec;
// use psci::error::INVALID_ADDRESS;
use crate::consts::{INVALID_ADDRESS, MAX_CPU_NUM};
use crate::pci::pci_struct::VirtualRootComplex;
use spin::RwLock;

use crate::arch::mm::new_s2_memory_set;
use crate::arch::s2pt::Stage2PageTable;
use crate::config::{HvZoneConfig, CONFIG_NAME_MAXLEN};

use crate::error::HvResult;
use crate::memory::addr::GuestPhysAddr;
use crate::memory::{MMIOConfig, MMIOHandler, MMIORegion, MemorySet};
use crate::percpu::{get_cpu_data, this_zone, CpuSet};
use core::panic;

pub struct Zone {
    pub name: [u8; CONFIG_NAME_MAXLEN],
    pub id: usize,
    pub mmio: Vec<MMIOConfig>,
    pub cpu_num: usize,
    pub cpu_set: CpuSet,
    pub irq_bitmap: [u32; 1024 / 32],
    pub gpm: MemorySet<Stage2PageTable>,
    pub is_err: bool,
    pub vpci_bus: VirtualRootComplex,
}

impl Zone {
    pub fn new(zoneid: usize, name: &[u8]) -> Self {
        Self {
            name: name.try_into().unwrap(),
            id: zoneid,
            gpm: new_s2_memory_set(),
            cpu_num: 0,
            cpu_set: CpuSet::new(MAX_CPU_NUM as usize, 0),
            mmio: Vec::new(),
            irq_bitmap: [0; 1024 / 32],
            is_err: false,
            vpci_bus: VirtualRootComplex::new(),
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
                error!("duplicated mmio region size not match, PLEASE CHECK!!!");
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

    zone_list
        .iter()
        .map(|zone| {
            let zone_lock = zone.read();
            ZoneInfo {
                zone_id: zone_lock.id as u32,
                cpus: zone_lock.cpu_set.bitmap,
                name: zone_lock.name.clone(),
                is_err: zone_lock.is_err as u8,
            }
        })
        .collect()
}

pub fn this_zone_id() -> usize {
    this_zone().read().id
}

pub fn zone_create(config: &HvZoneConfig) -> HvResult<Arc<RwLock<Zone>>> {
    // we create the new zone here
    // TODO: create Zone with cpu_set
    let zone_id = config.zone_id as usize;

    if find_zone(zone_id).is_some() {
        return hv_result_err!(
            EINVAL,
            format!("Failed to create zone: zone_id {} already exists", zone_id)
        );
    }

    let mut zone = Zone::new(zone_id, &config.name);
    zone.pt_init(config.memory_regions()).unwrap();
    zone.mmio_init(&config.arch_config);

    let _ = zone.virtual_pci_mmio_init(&config.pci_config, config.num_pci_bus);
    let _ = zone.guest_pci_init(&config.alloc_pci_devs, config.num_pci_devs);
    
    // #[cfg(target_arch = "aarch64")]
    // zone.ivc_init(config.ivc_config());

    /* loongarch page table emergency */
    /* Kai: Maybe unnecessary but i can't boot vms on my 3A6000 PC without this function. */
    // #[cfg(target_arch = "loongarch64")]
    // zone.page_table_emergency(
    //     config.pci_config[0].ecam_base as _,
    //     config.pci_config[0].ecam_size as _,
    // )?;

    let mut cpu_num = 0;

    for cpu_id in config.cpus().iter() {
        if let Some(zone) = get_cpu_data(*cpu_id as _).zone.clone() {
            return hv_result_err!(
                EBUSY,
                format!(
                    "Failed to create zone: cpu {} already belongs to zone {}",
                    cpu_id,
                    zone.read().id
                )
            );
        }
        zone.cpu_set.set_bit(*cpu_id as _);
        cpu_num += 1;
    }

    zone.cpu_num = cpu_num;
    // Initialize the virtual interrupt controller, it needs zone.cpu_num
    zone.virqc_init(config);

    zone.irq_bitmap_init(config.interrupts());

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
            #[cfg(target_arch = "aarch64")]
            {
                cpu_data.arch_cpu.is_aarch32 = config.arch_config.is_aarch32 != 0;
            }
        });
    }

    Ok(new_zone_pointer)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ZoneInfo {
    zone_id: u32,
    cpus: u64,
    name: [u8; CONFIG_NAME_MAXLEN],
    is_err: u8,
}
// Be careful about dead lock for zone.write()
pub fn zone_error() {
    if is_this_root_zone() {
        panic!("root zone has some error");
    }
    let zone = this_zone();
    let zone_id = zone.read().id;
    error!("zone {} has some error, please shut down it", zone_id);

    let mut zone_w = zone.write();
    zone_w.is_err = true;

    drop(zone_w);
    drop(zone);
}

#[test_case]
fn test_add_and_remove_zone() {
    let zone_count = 50;
    let zone_count_before = ZONE_LIST.read().len();
    for i in 0..zone_count {
        let u8name_array = [i as u8; CONFIG_NAME_MAXLEN];
        let zone = Zone::new(i, &u8name_array);
        ZONE_LIST.write().push(Arc::new(RwLock::new(zone)));
    }
    for i in 0..zone_count {
        remove_zone(i);
    }
    assert_eq!(ZONE_LIST.read().len(), zone_count_before);
}
