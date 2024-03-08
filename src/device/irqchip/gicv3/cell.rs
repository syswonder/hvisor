impl CellIrqchipTrait {
    pub fn irqchip_register_mmio_handlers(&mut self) {
        // add gicd handler
        self.mmio_region_register(
            HvSystemConfig::get().platform_info.arch.gicd_base as _,
            GICD_SIZE,
            gicv3_gicd_mmio_handler,
            0,
        );

        // add gicr handler
        for cpu in 0..MAX_CPU_NUM {
            let gicr_base = get_cpu_data(cpu).gicr_base as _;
            warn!("registering gicr {} at {:#x?}", cpu, gicr_base);
            if gicr_base == 0 {
                continue;
            }
            self.mmio_region_register(gicr_base, GICR_SIZE, gicv3_gicr_mmio_handler, cpu as _);
        }
        self.mmio_region_register(0x8080000, 0x20000, mmio_generic_handler, 0x8080000);
    }

    /// Add irq_id to this cell
    pub fn irqchip_adjust_target(&mut self, irq_id: u32) {
        let gicd_base = HvSystemConfig::get().platform_info.arch.gicd_base;
        let irouter = (gicd_base + GICD_IROUTER + 8 * irq_id as u64) as *mut u64;
        let mpidr: u64 = get_cpu_data(self.cpu_set.first_cpu().unwrap()).mpidr;

        unsafe {
            let route = mpidr_to_cpuid(irouter.read_volatile());
            if !self.owns_cpu(route) {
                warn!("adjust irq {} target -> cpu {}", irq_id, mpidr_to_cpuid(mpidr));
                irouter.write_volatile(mpidr & 0xff);
                warn!("now target = {:#x?}", irouter.read_volatile());
            }
        }
    }

    /// Commit the change of cell's irq mapping. It must be done when changing the cell's irq mapping.
    pub fn irqchip_adjust_mappings(&mut self) {
        let rc = root_cell();
        let rc_r = rc.read();

        for n in 32..1024 {
            if self.irq_in_cell(n) {
                if rc_r.irq_in_cell(n) {
                    panic!("irq {} in root cell", n);
                }
                self.irqchip_adjust_target(n);
            }
        }
    }

    /// Clear the cell's irqs and return its mapping to root cell. Called when destroy this cell.
    pub fn irqchip_exit(&self) {
        /* ensure all SPIs of the cell are masked and deactivated */
        self.irqchip_reset();
        let gicd_base = HvSystemConfig::get().platform_info.arch.gicd_base;
        let rc = root_cell();
        let mut rc_w = rc.write();
        /* set all pins of the old cell in the root cell */
        for chip in &self.config().irq_chips().to_vec() {
            if chip.address != gicd_base {
                continue;
            }
            for (idx, &mask) in chip.pin_bitmap.iter().enumerate() {
                rc_w.irq_bitmap[chip.pin_base as usize / 32 + idx] |= mask;
            }
        }
        /* mask out pins again that actually didn't belong to the root cell */
        for chip in &rc_w.config().irq_chips().to_vec() {
            if chip.address != gicd_base {
                continue;
            }
            for (idx, &mask) in chip.pin_bitmap.iter().enumerate() {
                rc_w.irq_bitmap[chip.pin_base as usize / 32 + idx] &= mask;
            }
        }
    }

    /// Mask and deactivate all SPIs of the cell.
    pub fn irqchip_reset(&self) {
        let gicd_base = HvSystemConfig::get().platform_info.arch.gicd_base;
        for (idx, &mask) in self.irq_bitmap.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            unsafe {
                write_volatile(
                    (gicd_base + GICD_ICENABLER + idx as u64 * 4) as *mut u32,
                    mask,
                );
                write_volatile(
                    (gicd_base + GICD_ICACTIVER + idx as u64 * 4) as *mut u32,
                    mask,
                );
            }
        }
    }
}