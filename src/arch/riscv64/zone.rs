use crate::{
    error::HvResult,
    arch::iommu::{BLK_PCI_ID, PCI_MAP_BEG, PCI_MAP_SIZE, PCIE_MMIO_BEG, PCIE_MMIO_SIZE},
    memory::{
        addr::align_up, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion,
    },
    percpu::get_cpu_data,
    zone::Zone,
};
impl Zone {
    pub fn pt_init(
        &mut self,
        _vm_paddr_start: usize,
        fdt: &fdt::Fdt,
        guest_dtb: usize,
        dtb_ipa: usize,
    ) -> HvResult {
        //debug!("fdt: {:?}", fdt);
        // The first memory region is used to map the guest physical memory.
        let mem_region = fdt.memory().regions().next().unwrap();
        info!("map mem_region: {:?}", mem_region);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            mem_region.starting_address as GuestPhysAddr,
            mem_region.starting_address as HostPhysAddr,
            mem_region.size.unwrap(),
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        ))?;
        // map guest dtb
        info!("map guest dtb: {:#x}", dtb_ipa);
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(
            dtb_ipa as GuestPhysAddr,
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
            // PCIE_ECAM
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as HostPhysAddr;
                let size = reg.size.unwrap();
                println!("map pci addr: {:#x}, size: {:#x}", paddr, size);
                self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                    paddr as GuestPhysAddr,
                    paddr,
                    size,
                    MemFlags::READ | MemFlags::WRITE,
                ))?;
            }
            // PCIE_MMIO
            let paddr = PCIE_MMIO_BEG;
            let size = PCIE_MMIO_SIZE;
            println!("map pci addr: {:#x}, size: {:#x}", paddr, size);
            self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                 paddr,
                 paddr,
                 size,
                 MemFlags::READ | MemFlags::WRITE,
             ))?;
            // add another region
            let paddr = PCI_MAP_BEG;
            let size = PCI_MAP_SIZE;
            println!("map pci addr: {:#x}, size: {:#x}", paddr, size);
            self.gpm.insert(MemoryRegion::new_with_offset_mapper(
                paddr,
                paddr,
                size,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }

        info!("VM stage 2 memory set: {:#x?}", self.gpm);
        Ok(())
    }
    pub fn mmio_init(&mut self, _fdt: &fdt::Fdt) {
        //TODO
    }
    pub fn irq_bitmap_init(&mut self, _fdt: &fdt::Fdt) {}
    pub fn isa_init(&mut self, fdt: &fdt::Fdt) {
        let cpu_set = self.cpu_set;
        cpu_set.iter().for_each(|cpuid| {
            let cpu_data = get_cpu_data(cpuid);
            let cpu_isa = fdt
                .cpus()
                .find(|cpu| cpu.ids().all().next().unwrap() == cpuid)
                .unwrap()
                .properties()
                .find(|p| p.name == "riscv,isa")
                .unwrap();
            if cpu_isa.as_str().unwrap().contains("sstc") {
                println!("cpu{} support sstc", cpuid);
                cpu_data.arch_cpu.sstc = true;
            }
        })
    }
}
