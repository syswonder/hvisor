use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

use crate::arch::cpu::this_cpu_id;
use crate::arch::s2pt::Stage2PageTable;
use crate::consts::MAX_CPU_NUM;
use crate::control::{resume_cpu, suspend_cpu};
use crate::error::HvResult;
use crate::memory::addr::{align_up, GuestPhysAddr, HostPhysAddr};
use crate::memory::{MMIOConfig, MMIOHandler, MMIORegion, MemFlags, MemoryRegion, MemorySet};
use crate::percpu::{get_cpu_data, CpuSet};
use core::panic;

pub struct Zone {
    pub id: usize,
    pub mmio: Vec<MMIOConfig>,
    pub cpu_set: CpuSet,
    pub irq_bitmap: [u32; 1024 / 32],
    pub gpm: MemorySet<Stage2PageTable>,
}

impl Zone {
    pub fn new(zoneid: usize) -> Self {
        Self {
            id: zoneid,
            gpm: MemorySet::new(),
            cpu_set: CpuSet::new(MAX_CPU_NUM as usize, 0),
            mmio: Vec::new(),
            irq_bitmap: [0; 1024 / 32],
        }
    }

    fn remove_irqs(&mut self, irq_bitmap: &[u32]) {
        for (i, &bitmap) in irq_bitmap.iter().enumerate() {
            self.irq_bitmap[i] &= !bitmap; // 使用位与和取反操作，将对应位置的位清零
        }
    }

    fn init_irq_bitmap(&mut self) {
        todo!();
        // let config = self.config();
        // let irq_chips = config.irq_chips().to_vec();
        // for irq_chip in irq_chips.iter() {
        //     let irq_bitmap_slice = &mut self.irq_bitmap[1..4 + 1]; // 获取可变的 irq_bitmap 切片
        //     irq_bitmap_slice
        //         .iter_mut()
        //         .zip(irq_chip.pin_bitmap.iter())
        //         .for_each(|(dest, src)| {
        //             *dest |= *src; // 对每个元素进行位或操作
        //         });
        // }
        // info!("irq bitmap = {:#x?}", self.irq_bitmap);
    }

    pub fn suspend(&self) {
        trace!("suspending cpu_set = {:#x?}", self.cpu_set);
        self.cpu_set.iter_except(this_cpu_id()).for_each(|cpu_id| {
            trace!("try to suspend cpu_id = {:#x?}", cpu_id);
            suspend_cpu(cpu_id);
        });
        info!("send sgi done!");
    }

    pub fn resume(&self) {
        trace!("resuming cpu_set = {:#x?}", self.cpu_set);
        self.cpu_set.iter_except(this_cpu_id()).for_each(|cpu_id| {
            trace!("try to resume cpu_id = {:#x?}", cpu_id);
            resume_cpu(cpu_id);
        });
    }

    pub fn owns_cpu(&self, id: usize) -> bool {
        self.cpu_set.contains_cpu(id)
    }

    /// Query an ipa from zone's stage 2 page table to get pa.
    pub fn gpm_query(&self, gpa: GuestPhysAddr) -> usize {
        todo!();
        // unsafe { self.gpm.page_table_query(gpa).unwrap().0 }
    }
    /// Map a mem region to a zone. \
    /// If the mem size is aligned to one page, it will be inserted into page table. \
    /// Otherwise into mmio regions.
    pub fn mem_region_map_partial(&mut self, mem: &MemoryRegion<GuestPhysAddr>) {
        todo!();
        // if is_aligned(mem.size) {
        //     self.gpm.map_partial(mem).unwrap();
        // } else {
        //     // Handle subpages
        //     self.mmio_region_register(
        //         mem.start as _,
        //         mem.size as _,
        //         mmio_subpage_handler,
        //         mem.start.wrapping_sub(mem.mapper.offset()) as _,
        //     );
        // }
    }

    /// Unmap a mem region from gpm or mmio regions of the zone.
    // pub fn mem_region_unmap_partial(&mut self, mem: &MemoryRegion<GuestPhysAddr>) {
    //     if is_aligned(mem.size) {
    //         self.gpm.unmap_partial(mem).unwrap();
    //     } else {
    //         // Handle subpages
    //         self.mmio_region_unregister(mem.start);
    //     }
    // }

    /// Insert a mem region to zone. \
    /// If the mem size is aligned to one page, it will be inserted into page table. \
    /// Otherwise into mmio regions.
    pub fn mem_region_insert(&mut self, mem: MemoryRegion<GuestPhysAddr>) {
        todo!();
        // if is_aligned(mem.size) {
        //     self.gpm.insert(mem).unwrap();
        // } else {
        //     // Handle subpages
        //     self.mmio_region_register(
        //         mem.start as _,
        //         mem.size as _,
        //         mmio_subpage_handler,
        //         mem.start.wrapping_sub(mem.mapper.offset()) as _,
        //     );
        // }
    }
    /// Register a mmio region and its handler.
    pub fn mmio_region_register(
        &mut self,
        start: GuestPhysAddr,
        size: usize,
        handler: MMIOHandler,
        arg: usize,
    ) {
        if let Some(mmio) = self.mmio.iter().find(|mmio| mmio.region.start == start) {
            panic!("duplicated mmio region {:#x?}", mmio);
        }
        self.mmio.push(MMIOConfig {
            region: MMIORegion { start, size },
            handler,
            arg,
        })
    }
    /// Remove the mmio region beginning at `start`.
    // pub fn mmio_region_unregister(&mut self, start: GuestPhysAddr) {
    //     if let Some((idx, _)) = self
    //         .mmio
    //         .iter()
    //         .enumerate()
    //         .find(|(_, mmio)| mmio.region.start == start)
    //     {
    //         self.mmio.remove(idx);
    //     }
    // }
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

static ROOT_CELL: spin::Once<Arc<RwLock<Zone>>> = spin::Once::new();
static CELL_LIST: RwLock<Vec<Arc<RwLock<Zone>>>> = RwLock::new(vec![]);
/// Add zone to CELL_LIST
pub fn add_zone(zone: Arc<RwLock<Zone>>) {
    CELL_LIST.write().push(zone);
}
/// Remove zone from CELL_LIST
pub fn remove_zone(zone_id: u32) {
    todo!();
    // let mut zone_list = CELL_LIST.write();
    // let (idx, _) = zone_list
    //     .iter()
    //     .enumerate()
    //     .find(|(_, zone)| zone.read().config().id() == zone_id)
    //     .unwrap();
    // zone_list.remove(idx);
}

pub fn root_zone() -> Arc<RwLock<Zone>> {
    ROOT_CELL.get().expect("Uninitialized root zone!").clone()
}

pub fn find_zone_by_id(zone_id: u32) -> Option<Arc<RwLock<Zone>>> {
    todo!();
    // CELL_LIST
    //     .read()
    //     .iter()
    //     .find(|zone| zone.read().config().id() == zone_id)
    //     .cloned()
}

pub fn zone_create(vmid: usize, dtb_ptr: *const u8, dtb_ipa: usize) -> Arc<RwLock<Zone>> {
    // we create the new zone here
    //TODO: create Zone with cpu_set
    let guest_fdt = unsafe { fdt::Fdt::from_ptr(dtb_ptr) }.unwrap();
    let guest_entry = guest_fdt
        .memory()
        .regions()
        .next()
        .unwrap()
        .starting_address as usize;

    debug!("zone fdt guest_addr: {:#b}", guest_entry);
    let mut zone = Zone::new(vmid);
    zone.pt_init(guest_entry, &guest_fdt, dtb_ptr as usize, dtb_ipa)
        .unwrap();
    guest_fdt.cpus().for_each(|cpu| {
        let cpu_id = cpu.ids().all().next().unwrap();
        zone.cpu_set.set_bit(cpu_id as usize);
    });
    //TODO:assign cpu according to cpu_set
    //TODO:set cpu entry
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
            cpu_data.cpu_on_entry = guest_entry;
        });
    }
    add_zone(new_zone_pointer.clone());

    new_zone_pointer
}

// pub fn init() -> HvResult {
//     info!("Root zone initializing...");
//     let root_zone = Arc::new(RwLock::new(Zone::new_root()?));
//     info!("Root zone init end.");

//     add_zone(root_zone.clone());
//     ROOT_CELL.call_once(|| root_zone);
//     Ok(())
// }
