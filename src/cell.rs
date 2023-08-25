use crate::arch::Stage2PageTable;
use crate::config::{CellConfig, HvSystemConfig};
use crate::consts::HV_BASE;
use crate::error::HvResult;
use crate::memory::addr::{GuestPhysAddr, HostPhysAddr};
use crate::memory::{GenericPageTableImmut, MemFlags, MemoryRegion, MemorySet};

#[derive(Debug)]
pub struct Cell<'a> {
    /// Cell configuration.
    pub config: CellConfig<'a>,
    /// Guest physical memory set.
    pub gpm: MemorySet<Stage2PageTable>,
}

impl Cell<'_> {
    fn new_root() -> HvResult<Self> {
        let sys_config = HvSystemConfig::get();
        let cell_config = sys_config.root_cell.config();
        let mmcfg_start = sys_config.platform_info.pci_mmconfig_base;
        let mmcfg_size = (sys_config.platform_info.pci_mmconfig_end_bus + 1) as u64 * 256 * 4096;
        let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
        let hv_phys_size = sys_config.hypervisor_memory.size as usize;

        let mut gpm: MemorySet<Stage2PageTable> = MemorySet::new();
        info!("set gpm for el1");
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0xa0000000 as GuestPhysAddr,
            0x7fa00000 as HostPhysAddr,
            0x00100000 as usize,
            MemFlags::READ | MemFlags::WRITE | MemFlags::NO_HUGEPAGES,
        ))?;

        gpm.insert(MemoryRegion::new_with_offset_mapper(
            hv_phys_start as GuestPhysAddr,
            hv_phys_start as HostPhysAddr,
            hv_phys_size as usize,
            MemFlags::READ | MemFlags::NO_HUGEPAGES,
        ))?;

        // Map all physical memory regions.
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x09000000 as GuestPhysAddr,
            0x09000000 as HostPhysAddr,
            0x37000000 as usize,
            MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x40000000 as GuestPhysAddr,
            0x40000000 as HostPhysAddr,
            0x3fb00000 as usize,
            MemFlags::READ | MemFlags::WRITE,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x7fb00000 as GuestPhysAddr,
            0x7fb00000 as HostPhysAddr,
            0x100000 as usize,
            MemFlags::READ | MemFlags::WRITE,
        ))?;
        //add gicd gicr memory map
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x8000000 as GuestPhysAddr,
            0x8000000 as HostPhysAddr,
            0x0200000 as usize,
            MemFlags::READ | MemFlags::WRITE,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            mmcfg_start as GuestPhysAddr,
            mmcfg_start as HostPhysAddr,
            mmcfg_size as usize,
            MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
        ))?;

        trace!("Guest phyiscal memory set: {:#x?}", gpm);

        Ok(Self {
            config: cell_config,
            gpm,
        })
    }
}

static ROOT_CELL: spin::Once<Cell> = spin::Once::new();

pub fn root_cell<'a>() -> &'a Cell<'a> {
    ROOT_CELL.get().expect("Uninitialized root cell!")
}

pub fn init() -> HvResult {
    let root_cell = Cell::new_root()?;
    info!("Root cell init end.");
    //debug!("{:#x?}", root_cell);

    ROOT_CELL.call_once(|| root_cell);
    Ok(())
}
