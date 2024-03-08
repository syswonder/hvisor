use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

use crate::arch::this_cpu_id;
use crate::config::{CellConfig, HvCellDesc, HvConsole, HvSystemConfig};
use crate::control::{resume_cpu, suspend_cpu};
use crate::error::HvResult;
use crate::memory::addr::{is_aligned, GuestPhysAddr, HostPhysAddr};
use crate::memory::{
    mmio_subpage_handler, npages, Frame, MMIOConfig, MMIOHandler, MMIORegion,
    MemFlags, MemoryRegion, MemorySet,
};
use crate::percpu::CpuSet;
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
    pub cell_state: u32,  // volatile
    msg_to_cell: u32,     // volatile
    reply_from_cell: u32, //volatile
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
            cell_state: 0,
            msg_to_cell: 0,
            reply_from_cell: 0,
            flags: 0,
            console: HvConsole::new(),
            gic_version: 0,
            gicd_base: 0,
            gicc_base: 0,
            gicr_base: 0,
        }
    }
}
pub struct Cell {
    /// Communication Page
    pub comm_page: Frame,
    /// Cell configuration.
    pub config_frame: Frame,
    pub mmio: Vec<MMIOConfig>,
    pub cpu_set: CpuSet,
    pub irq_bitmap: [u32; 1024 / 32],
    pub loadable: bool,
    // gpm: MemorySet<Stage2PageTable>,
}

impl Cell {
    fn new_root() -> HvResult<Self> {
        let sys_config = HvSystemConfig::get();
        let cell_config = sys_config.root_cell.config();
        let mut cell = Self::new(cell_config, true)?;

        let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
        let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;
        let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
        let hv_phys_size = sys_config.hypervisor_memory.size as usize;

        todo!();
        // // Back the region of hypervisor core in linux so that shutdown will not trigger violations.
        // cell.mem_region_insert(MemoryRegion::new_with_offset_mapper(
        //     hv_phys_start as GuestPhysAddr,
        //     hv_phys_start as HostPhysAddr,
        //     hv_phys_size as usize,
        //     MemFlags::READ | MemFlags::NO_HUGEPAGES,
        // ));

        // // Map all physical memory regions.
        // let mem_regs = cell.config().mem_regions().to_vec();
        // mem_regs.iter().for_each(|mem| {
        //     cell.mem_region_insert(MemoryRegion::new_with_offset_mapper(
        //         mem.virt_start as GuestPhysAddr,
        //         mem.phys_start as HostPhysAddr,
        //         mem.size as _,
        //         mem.flags,
        //     ))
        // });

        // // TODO: Without this mapping, enabling hypervisor will get an error, maybe now we don't have mmio handlers.
        // cell.mem_region_insert(MemoryRegion::new_with_offset_mapper(
        //     mmcfg_start as GuestPhysAddr,
        //     mmcfg_start as HostPhysAddr,
        //     mmcfg_size as usize,
        //     MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
        // ));

        // trace!("Guest phyiscal memory set: {:#x?}", cell.gpm);
        // Ok(cell)
    }

    pub fn new(config: CellConfig, is_root_cell: bool) -> HvResult<Self> {
        // todo: config page too big
        assert!(npages(config.total_size()) == 1);

        let mut cell: Cell = Self {
            config_frame: {
                let mut config_frame = Frame::new()?;
                config_frame.copy_data_from(config.as_slice());
                config_frame
            },
            // gpm: MemorySet::new(),
            cpu_set: config.cpu_set(),
            loadable: false,
            comm_page: Frame::new()?,
            mmio: vec![],
            irq_bitmap: [0; 1024 / 32],
        };
        todo!();
        // cell.irqchip_register_mmio_handlers();
        // cell.init_irq_bitmap();
        // if !is_root_cell {
        //     let root_cell = root_cell();
        //     let mut root_cell_w = root_cell.write();
        //     root_cell_w.remove_irqs(&cell.irq_bitmap);
        // }
        // Ok(cell)
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


    /// Get cell id
    pub fn id(&self) -> u32 {
        self.config().id()
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

    pub fn owns_cpu(&self, id: u64) -> bool {
        self.cpu_set.contains_cpu(id)
    }

    pub fn config(&self) -> CellConfig {
        // Enable stage 1 translation in el2 changes config_addr from physical address to virtual address
        // with an offset `PHYS_VIRT_OFFSET`, so we need to check whether stage 1 translation is enabled.
        // let config_addr = match INIT_LATE_OK.load(Ordering::Relaxed) {
        //     1 => self.config_frame.as_ptr() as usize,
        //     _ => self.config_frame.start_paddr(),
        // };
        let config_addr = self.config_frame.as_ptr() as usize;
        unsafe { CellConfig::new((config_addr as *const HvCellDesc).as_ref().unwrap()) }
    }

    pub fn gpm_activate(&self) {
        todo!();
        // unsafe { self.gpm.activate() }
    }
    /// Query an ipa from cell's stage 2 page table to get pa.
    pub fn gpm_query(&self, gpa: GuestPhysAddr) -> usize {
        todo!();
        // unsafe { self.gpm.page_table_query(gpa).unwrap().0 }
    }
    /// Map a mem region to a cell. \
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

    /// Unmap a mem region from gpm or mmio regions of the cell.
    // pub fn mem_region_unmap_partial(&mut self, mem: &MemoryRegion<GuestPhysAddr>) {
    //     if is_aligned(mem.size) {
    //         self.gpm.unmap_partial(mem).unwrap();
    //     } else {
    //         // Handle subpages
    //         self.mmio_region_unregister(mem.start);
    //     }
    // }

    /// Insert a mem region to cell. \
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
    /// If irq_id belongs to this cell
    pub fn irq_in_cell(&self, irq_id: u32) -> bool {
        let idx = (irq_id / 32) as usize;
        let bit_pos = (irq_id % 32) as usize;
        (self.irq_bitmap[idx] & (1 << bit_pos)) != 0
    }
}

static ROOT_CELL: spin::Once<Arc<RwLock<Cell>>> = spin::Once::new();
static CELL_LIST: RwLock<Vec<Arc<RwLock<Cell>>>> = RwLock::new(vec![]);
/// Add cell to CELL_LIST
pub fn add_cell(cell: Arc<RwLock<Cell>>) {
    CELL_LIST.write().push(cell);
}
/// Remove cell from CELL_LIST
pub fn remove_cell(cell_id: u32) {
    let mut cell_list = CELL_LIST.write();
    let (idx, _) = cell_list
        .iter()
        .enumerate()
        .find(|(_, cell)| cell.read().config().id() == cell_id)
        .unwrap();
    cell_list.remove(idx);
}

pub fn root_cell() -> Arc<RwLock<Cell>> {
    ROOT_CELL.get().expect("Uninitialized root cell!").clone()
}

pub fn find_cell_by_id(cell_id: u32) -> Option<Arc<RwLock<Cell>>> {
    CELL_LIST
        .read()
        .iter()
        .find(|cell| cell.read().config().id() == cell_id)
        .cloned()
}

pub fn init() -> HvResult {
    info!("Root cell initializing...");
    let root_cell = Arc::new(RwLock::new(Cell::new_root()?));
    info!("Root cell init end.");

    add_cell(root_cell.clone());
    ROOT_CELL.call_once(|| root_cell);
    Ok(())
}
