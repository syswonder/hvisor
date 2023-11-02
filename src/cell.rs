use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

use crate::arch::Stage2PageTable;
use crate::config::{CellConfig, HvCellDesc, HvConsole, HvSystemConfig};
use crate::control::{resume_cpu, suspend_cpu};
use crate::device::gicv3::{gicv3_mmio_handler, GICD_SIZE};
use crate::error::HvResult;
use crate::memory::addr::{GuestPhysAddr, HostPhysAddr};
use crate::memory::{
    npages, Frame, MMIOConfig, MMIOHandler, MMIORegion, MemFlags, MemoryRegion, MemorySet,
};
use crate::percpu::{this_cpu_data, CpuSet};
use crate::INIT_LATE_OK;
use core::sync::atomic::Ordering;

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
    /// Guest physical memory set.
    pub gpm: MemorySet<Stage2PageTable>,
    pub mmio: Vec<MMIOConfig>,
    pub cpu_set: CpuSet,
    pub loadable: bool,
}

impl Cell {
    fn new_root() -> HvResult<Self> {
        let sys_config = HvSystemConfig::get();
        let cell_config = sys_config.root_cell.config();
        let mut cell = Self::new(cell_config)?;

        let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
        let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;
        let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
        let hv_phys_size = sys_config.hypervisor_memory.size as usize;

        // Back the region of hypervisor core in linux so that shutdown will not trigger violations.
        cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
            hv_phys_start as GuestPhysAddr,
            hv_phys_start as HostPhysAddr,
            hv_phys_size as usize,
            MemFlags::READ | MemFlags::NO_HUGEPAGES,
        ))?;

        // Map all physical memory regions.
        let mem_regs = cell.config().mem_regions().to_vec();
        mem_regs.iter().for_each(|mem| {
            cell.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    mem.virt_start as GuestPhysAddr,
                    mem.phys_start as HostPhysAddr,
                    mem.size as _,
                    mem.flags,
                ))
                .unwrap()
        });

        // TODO: Without this mapping, enable hypervisor will get an error, maybe now we don't have mmio handlers.
        cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
            mmcfg_start as GuestPhysAddr,
            mmcfg_start as HostPhysAddr,
            mmcfg_size as usize,
            MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
        ))?;

        // TODO: Without this mapping, create a new cell will get warnings because we don't have mmio handlers now.
        cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x800_0000 as GuestPhysAddr,
            0x800_0000 as HostPhysAddr,
            0x020_0000 as usize,
            MemFlags::READ | MemFlags::WRITE,
        ))?;
        trace!("Guest phyiscal memory set: {:#x?}", cell.gpm);
        Ok(cell)
    }

    pub fn new(config: CellConfig) -> HvResult<Self> {
        let gpm: MemorySet<Stage2PageTable> = MemorySet::new();
        let cpu_set = config.cpu_set();
        if cpu_set.len() != 8 {
            todo!("Cpu_set should be 8 bytes!");
        }
        let cpu_set_long: u64 = cpu_set
            .iter()
            .enumerate()
            .fold(0, |acc, (i, x)| acc | (*x as u64) << (i * 8));

        let config_size = config.total_size();
        let config_pages = npages(config_size);

        // todo: config page too big
        assert!(config_pages == 1);

        // copy config to the newly allocated frame
        let mut config_frame = Frame::new()?;
        config_frame.copy_data_from(config.as_slice());

        let comm_page = Frame::new()?;

        let mut cell = Self {
            config_frame,
            gpm,
            cpu_set: CpuSet::new(cpu_set.len() as u64 * 8 - 1, cpu_set_long),
            loadable: false,
            comm_page,
            mmio: vec![],
        };

        cell.mmio_region_register(
            HvSystemConfig::get().platform_info.arch.gicd_base as _,
            GICD_SIZE,
            gicv3_mmio_handler,
        );

        Ok(cell)
    }

    pub fn suspend(&self) {
        info!("suspending cpu_set = {:#x?}", self.cpu_set);
        self.cpu_set
            .iter_except(this_cpu_data().id)
            .for_each(|cpu_id| {
                info!("try to suspend cpu_id = {:#x?}", cpu_id);
                suspend_cpu(cpu_id);
            });
        info!("send sgi done!");
    }

    pub fn resume(&self) {
        info!("resuming cpu_set = {:#x?}", self.cpu_set);
        self.cpu_set
            .iter_except(this_cpu_data().id)
            .for_each(|cpu_id| {
                info!("try to resume cpu_id = {:#x?}", cpu_id);
                resume_cpu(cpu_id);
            });
    }

    pub fn owns_cpu(&self, id: u64) -> bool {
        self.cpu_set.contains_cpu(id)
    }

    pub fn config(&self) -> CellConfig {
        // Enable stage 1 translation in el2 changes config_addr from physical address to virtual address
        // with an offset `PHYS_VIRT_OFFSET`, so we need to check whether stage 1 translation is enabled.
        let config_addr = match INIT_LATE_OK.load(Ordering::Relaxed) {
            1 => self.config_frame.as_ptr() as usize,
            _ => self.config_frame.start_paddr(),
        };
        unsafe { CellConfig::new(&(config_addr as *const HvCellDesc).as_ref().unwrap()) }
    }

    pub fn mmio_region_register(&mut self, start: GuestPhysAddr, size: u64, handler: MMIOHandler) {
        self.mmio.push(MMIOConfig {
            region: MMIORegion { start, size },
            handler,
        })
    }

    pub fn find_mmio_region(
        &self,
        addr: GuestPhysAddr,
        size: u64,
    ) -> Option<(MMIORegion, MMIOHandler)> {
        self.mmio
            .iter()
            .find(|cfg| cfg.region.contains_region(addr, size))
            .map(|cfg| (cfg.region, cfg.handler))
    }
}

static ROOT_CELL: spin::Once<Arc<RwLock<Cell>>> = spin::Once::new();
static CELL_LIST: RwLock<Vec<Arc<RwLock<Cell>>>> = RwLock::new(vec![]);

pub fn add_cell(cell: Arc<RwLock<Cell>>) {
    CELL_LIST.write().push(cell);
}

pub fn root_cell() -> Arc<RwLock<Cell>> {
    ROOT_CELL.get().expect("Uninitialized root cell!").clone()
}

pub fn find_cell_by_id(cell_id: u32) -> Option<Arc<RwLock<Cell>>> {
    CELL_LIST
        .read()
        .iter()
        .find(|cell| cell.read().config().id() == cell_id)
        .map(|cell| cell.clone())
}

pub fn init() -> HvResult {
    let root_cell = Arc::new(RwLock::new(Cell::new_root()?));
    info!("Root cell init end.");
    //debug!("{:#x?}", root_cell);

    add_cell(root_cell.clone());
    ROOT_CELL.call_once(|| root_cell);
    Ok(())
}
