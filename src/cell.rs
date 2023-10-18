use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

use crate::arch::Stage2PageTable;
use crate::config::{CellConfig, HvCellDesc, HvSystemConfig};
use crate::control::{resume_cpu, suspend_cpu};
use crate::error::HvResult;
use crate::memory::addr::{GuestPhysAddr, HostPhysAddr};
use crate::memory::{npages, Frame, MemFlags, MemoryRegion, MemorySet};
use crate::percpu::{this_cpu_data, CpuSet};

pub struct Cell {
    /// Communication Page
    pub comm_page: Frame,
    /// Cell configuration.
    pub config_frame: Frame,
    /// Guest physical memory set.
    pub gpm: MemorySet<Stage2PageTable>,
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

        cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
            hv_phys_start as GuestPhysAddr,
            hv_phys_start as HostPhysAddr,
            hv_phys_size as usize,
            MemFlags::READ | MemFlags::NO_HUGEPAGES,
        ))?;

        // Map all physical memory regions.
        /* MMIO (permissive) */
        {
            cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
                0x09000000 as GuestPhysAddr,
                0x09000000 as HostPhysAddr,
                0x37000000 as usize,
                MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
            ))?;
            /* RAM */
            cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
                0x40000000 as GuestPhysAddr,
                0x40000000 as HostPhysAddr,
                0x3fb00000 as usize,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
            /* "physical" PCI ECAM */
            cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
                0x7fb00000 as GuestPhysAddr,
                0x7fb00000 as HostPhysAddr,
                0x100000 as usize,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
            //add gicd gicr memory map
            cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
                0x8000000 as GuestPhysAddr,
                0x8000000 as HostPhysAddr,
                0x0200000 as usize,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
            cell.gpm.insert(MemoryRegion::new_with_offset_mapper(
                mmcfg_start as GuestPhysAddr,
                mmcfg_start as HostPhysAddr,
                mmcfg_size as usize,
                MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
            ))?;

            trace!("Guest phyiscal memory set: {:#x?}", cell.gpm);
        }
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

        let mut comm_page = Frame::new()?;

        Ok(Self {
            config_frame,
            gpm,
            cpu_set: CpuSet::new(cpu_set.len() as u64 * 8 - 1, cpu_set_long),
            loadable: false,
            comm_page,
        })
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
        unsafe {
            CellConfig::new(
                &(self.config_frame.start_paddr() as *const HvCellDesc)
                    .as_ref()
                    .unwrap(),
            )
        }
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
