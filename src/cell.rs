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
        //let hv_virt_start = sys_config.hypervisor_memory.virt_start as usize;
        let hv_phys_size = sys_config.hypervisor_memory.size as usize;

        let mut gpm: MemorySet<Stage2PageTable> = MemorySet::new();

        // Map hypervisor memory to the empty page.
        // gpm.insert(MemoryRegion::new_with_empty_mapper(
        //     hv_phys_start,
        //     hv_phys_size,
        //     MemFlags::READ | MemFlags::NO_HUGEPAGES,
        // ))?;

        // gpm.insert(MemoryRegion::new_with_offset_mapper(
        //     0xffffc0200000 as GuestPhysAddr,
        //     0x7fc00000 as HostPhysAddr,
        //     0x00400000 as usize,
        //     MemFlags::READ | MemFlags::NO_HUGEPAGES,
        // ))?;
        info!("set gpm 1");
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x7fc00000 as GuestPhysAddr,
            0x7fc00000 as HostPhysAddr,
            0x00100000 as usize,
            MemFlags::READ | MemFlags::NO_HUGEPAGES,
        ))?;

        info!("gpm1 ok");
        info!("set gpm 2");
        //use ldr to test  different offset
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x7fd00000 as GuestPhysAddr,
            0x7fc00000 as HostPhysAddr,
            0x00200000 as usize,
            MemFlags::READ,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0xf7fd00000 as GuestPhysAddr,
            0x07fc00000 as HostPhysAddr,
            0x000200000 as usize,
            MemFlags::READ,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0xff7fd00000 as GuestPhysAddr,
            0x07fc00000 as HostPhysAddr,
            0x0002000000 as usize,
            MemFlags::READ,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0xfff7fd00000 as GuestPhysAddr,
            0x07fc00000 as HostPhysAddr,
            0x00000200000 as usize,
            MemFlags::READ,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            //bad map at here 0xfffffffff
            0xfffffe00000 as GuestPhysAddr,
            0x07fc00000 as HostPhysAddr,
            0x00000200000 as usize,
            MemFlags::READ,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x1fffffffffff as GuestPhysAddr,
            0x07fc00000 as HostPhysAddr,
            0x00000200000 as usize,
            MemFlags::READ,
        ))?;
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0xffff00000000 as GuestPhysAddr,
            0x07fc00000 as HostPhysAddr,
            0x000000200000 as usize,
            MemFlags::READ,
        ))?;
        info!("gpm2 ok");
        info!("set gpm 3");
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0xffffc0200000 as GuestPhysAddr,
            0x00007fc00000 as HostPhysAddr,
            0x000000200000 as usize,
            MemFlags::READ,
        ))?;
        info!("gpm3 ok");
        // gpm.insert(MemoryRegion::new_with_offset_mapper(
        //     HV_BASE as GuestPhysAddr,
        //     hv_phys_start as HostPhysAddr,
        //     hv_phys_size as usize,
        //     MemFlags::READ | MemFlags::NO_HUGEPAGES,
        // ))?;

        // Map all physical memory regions.
        gpm.insert(MemoryRegion::new_with_offset_mapper(
            0x09000000 as GuestPhysAddr,
            0x09000000 as HostPhysAddr,
            0x37000000 as usize,
            MemFlags::READ | MemFlags::WRITE,
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
        //add test_el1 memory Map
        // gpm.insert(MemoryRegion::new_with_offset_mapper(
        //     0xffffc0200000 as GuestPhysAddr,
        //     0x00007fc00000 as HostPhysAddr,
        //     0x000000400000 as usize,
        //     MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        // ))?;
        // for region in cell_config.mem_regions() {
        //     gpm.insert(MemoryRegion::new_with_offset_mapper(
        //         region.virt_start as GuestPhysAddr,
        //         region.phys_start as HostPhysAddr,
        //         region.size as usize,
        //         region.flags,
        //     ))?;
        // }

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
