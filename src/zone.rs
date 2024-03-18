use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

use crate::arch::cpu::this_cpu_id;
use crate::arch::paging::npages;
use crate::arch::s2pt::Stage2PageTable;
use crate::config::{ZoneConfig, HvZoneDesc, HvConsole, HvSystemConfig};
use crate::consts::MAX_CPU_NUM;
use crate::control::{resume_cpu, suspend_cpu};
use crate::error::HvResult;
use crate::memory::addr::{align_up, is_aligned, GuestPhysAddr, HostPhysAddr};
use crate::memory::{
    mmio_subpage_handler, Frame, MMIOConfig, MMIOHandler, MMIORegion,
    MemFlags, MemoryRegion, MemorySet,
};
use crate::percpu::{get_cpu_data, CpuSet};
use core::panic;

#[repr(C)]
pub struct CommPage {
    pub comm_region: CommRegion,
    // padding: [u8; 4096],
}

impl CommPage {
    #[allow(unused)]
    fn new() -> Self {
        Self {
            comm_region: CommRegion::new(),
        }
    }
    // set CommPage to 0s
    #[allow(unused)]
    pub fn fill_zero(&mut self) {
        unsafe { core::ptr::write_bytes(self as *mut _, 0, 1) }
    }
}
#[repr(C)]
pub struct CommRegion {
    pub signature: [u8; 6],
    pub revision: u16,
    pub zone_state: u32,  // volatile
    msg_to_zone: u32,     // volatile
    reply_from_zone: u32, //volatile
    pub flags: u32,       //volatile
    pub console: HvConsole,
    pub gic_version: u8,
    pub gicd_base: u64,
    pub gicc_base: u64,
    pub gicr_base: u64,
}

impl CommRegion {
    fn new() -> Self {
        Self {
            signature: [0; 6],
            revision: 0,
            zone_state: 0,
            msg_to_zone: 0,
            reply_from_zone: 0,
            flags: 0,
            console: HvConsole::new(),
            gic_version: 0,
            gicd_base: 0,
            gicc_base: 0,
            gicr_base: 0,
        }
    }
}
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

    pub fn pt_init(
        &mut self,
        vm_paddr_start: usize,
        fdt: &fdt::Fdt,
        guest_dtb: usize,
        dtb_addr: usize,
    ) -> HvResult {
        //debug!("fdt: {:?}", fdt);
        // The first memory region is used to map the guest physical memory.
        let mem_region = fdt.memory().regions().next().unwrap();
        info!("map mem_region: {:?}", mem_region);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            mem_region.starting_address as GuestPhysAddr,
            vm_paddr_start as HostPhysAddr,
            mem_region.size.unwrap(),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // map guest dtb
        info!("map guest dtb: {:#x}", dtb_addr);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            dtb_addr as GuestPhysAddr,
            guest_dtb as HostPhysAddr,
            align_up(fdt.total_size()),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // probe virtio mmio device
        for node in fdt.find_all_nodes("/soc/virtio_mmio") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                info!("map virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe virt test
        for node in fdt.find_all_nodes("/soc/test") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap() + 0x1000;
                info!("map test addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
                ))?;
            }
        }

        // probe uart device
        for node in fdt.find_all_nodes("/soc/uart") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = align_up(reg.size.unwrap());
                info!("map uart addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe clint(core local interrupter)
        for node in fdt.find_all_nodes("/soc/clint") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                info!("map clint addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        // probe plic
        //TODO: remove plic map from vm
        // for node in fdt.find_all_nodes("/soc/plic") {
        //     if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        //         let paddr = reg.starting_address as HostPhysAddr;
        //         //let size = reg.size.unwrap();
        //         let size = PLIC_GLOBAL_SIZE; //
        //         debug!("map plic addr: {:#x}, size: {:#x}", paddr, size);
        //         self.gpm.insert(MemoryRegion::new_with_offset_mapper(
        //             paddr as GuestPhysAddr,
        //             paddr,
        //             size,
        //             MemFlags::READ | MemFlags::WRITE,
        //         ))?;
        //     }
        // }

        for node in fdt.find_all_nodes("/soc/pci") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                info!("map pci addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
        }

        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }

    fn remove_irqs(&mut self, irq_bitmap: &[u32]) {
        for (i, &bitmap) in irq_bitmap.iter().enumerate() {
            self.irq_bitmap[i] &= !bitmap; // 使用位与和取反操作，将对应位置的位清零
        }
    }

    fn init_irq_bitmap(&mut self) {
        let config = self.config();
        let irq_chips = config.irq_chips().to_vec();
        for irq_chip in irq_chips.iter() {
            let irq_bitmap_slice = &mut self.irq_bitmap[1..4 + 1]; // 获取可变的 irq_bitmap 切片
            irq_bitmap_slice
                .iter_mut()
                .zip(irq_chip.pin_bitmap.iter())
                .for_each(|(dest, src)| {
                    *dest |= *src; // 对每个元素进行位或操作
                });
        }
        // info!("irq bitmap = {:#x?}", self.irq_bitmap);
    }

    pub fn suspend(&self) {
        trace!("suspending cpu_set = {:#x?}", self.cpu_set);
        self.cpu_set
            .iter_except(this_cpu_id())
            .for_each(|cpu_id| {
                trace!("try to suspend cpu_id = {:#x?}", cpu_id);
                suspend_cpu(cpu_id);
            });
        info!("send sgi done!");
    }

    pub fn resume(&self) {
        trace!("resuming cpu_set = {:#x?}", self.cpu_set);
        self.cpu_set
            .iter_except(this_cpu_id())
            .for_each(|cpu_id| {
                trace!("try to resume cpu_id = {:#x?}", cpu_id);
                resume_cpu(cpu_id);
            });
    }

    pub fn owns_cpu(&self, id: usize) -> bool {
        self.cpu_set.contains_cpu(id)
    }

    pub fn config(&self) -> ZoneConfig {
        // Enable stage 1 translation in el2 changes config_addr from physical address to virtual address
        // with an offset `PHYS_VIRT_OFFSET`, so we need to check whether stage 1 translation is enabled.
        // let config_addr = match INIT_LATE_OK.load(Ordering::Relaxed) {
        //     1 => self.config_frame.as_ptr() as usize,
        //     _ => self.config_frame.start_paddr(),
        // };
        todo!()
        // let config_addr = self.config_frame.as_ptr() as usize;
        // unsafe { ZoneConfig::new((config_addr as *const HvZoneDesc).as_ref().unwrap()) }
    }

    pub fn gpm_activate(&self) {
        unsafe { self.gpm.activate() }
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
        size: u64,
        handler: MMIOHandler,
        arg: u64,
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
        size: u64,
    ) -> Option<(MMIORegion, MMIOHandler, u64)> {
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
    let mut zone_list = CELL_LIST.write();
    let (idx, _) = zone_list
        .iter()
        .enumerate()
        .find(|(_, zone)| zone.read().config().id() == zone_id)
        .unwrap();
    zone_list.remove(idx);
}

pub fn root_zone() -> Arc<RwLock<Zone>> {
    ROOT_CELL.get().expect("Uninitialized root zone!").clone()
}

pub fn find_zone_by_id(zone_id: u32) -> Option<Arc<RwLock<Zone>>> {
    CELL_LIST
        .read()
        .iter()
        .find(|zone| zone.read().config().id() == zone_id)
        .cloned()
}

pub fn zone_create(
    vmid: usize,
    vm_paddr_start: usize,
    dtb_ptr: *const u8,
    dtb_addr: usize,
) -> Arc<RwLock<Zone>> {
    // we create the new zone here
    //TODO: create Zone with cpu_set
    let guest_fdt = unsafe { fdt::Fdt::from_ptr(dtb_ptr) }.unwrap();
    let guest_entry = guest_fdt
        .memory()
        .regions()
        .next()
        .unwrap()
        .starting_address as usize;
    let mut zone = Zone::new(vmid);
    zone.pt_init(vm_paddr_start, &guest_fdt, dtb_ptr as usize, dtb_addr)
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
