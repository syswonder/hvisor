use tock_registers::interfaces::Readable;
use crate::zone::Zone;
use crate::arch::zone::HvArchZoneConfig;
use crate::device::irqchip::gicv2::{gicd, GICV2};
use crate::device::irqchip::gicv2::gicc::GICC;
use crate::memory::{mmio_perform_access, MMIOAccess, MemFlags, MemoryRegion};
use crate::device::irqchip::gicv2::gicd::{get_max_int_num, is_spi, GICD, GICD_SGIR_REG_OFFSET, GICD_CTRL_REG_OFFSET, GICD_ICACTIVER_REG_OFFSET, GICD_ICENABLER_REG_OFFSET, GICD_ICFGR_REG_OFFSET, GICD_ICPENDR_REG_OFFSET, GICD_IDENTIFICATION_NUM, GICD_IDENTIFICATION_OFFSET, GICD_IGROUPR_REG_OFFSET, GICD_IIDR_REG_OFFSET, GICD_IPRIORITYR_REG_OFFSET, GICD_ISACTIVER_REG_OFFSET, GICD_ISENABLER_REG_OFFSET, GICD_ISPENDR_REG_OFFSET, GICD_ITARGETSR_REG_OFFSET, GICD_LOCK, GICD_TYPER_REG_OFFSET, GICV2_CONFIG_REGS_NUM, GICV2_INT_REGS_NUM, GICV2_PRIO_REGS_NUM, GICV2_TARGET_REGS_NUM, GICD_SGIR_ROUTING_SHIFT, GICD_SGIR_TARGET_LIST_FILTER_SHIFT};
use crate::error::HvResult;
use crate::percpu::this_zone;


const GICV2_REG_WIDTH: usize = 4;


impl Zone {
    // trap all Guest OS accesses to the GIC Distributor registers.
    pub fn vgicv2_mmio_init(&mut self, arch:&HvArchZoneConfig) {
        if arch.gicd_base == 0 {
            panic!("vgicv2_mmio_init: gicd_base is null");
        }
        self.mmio_region_register(arch.gicd_base,arch.gicd_size,vgicv2_dist_handler,0);
    }

    // remap the GIC CPU interface register address space to point to the GIC virtual CPU interface registers.
    pub fn vgicv2_remap_init(&mut self, arch:&HvArchZoneConfig) {
        if arch.gicc_base == 0 || arch.gicv_base == 0 || arch.gicc_size == 0 || arch.gicv_size == 0 {
            panic!("vgicv2_remap_init: gic related address is null");
        }
        if arch.gicv_size != arch.gicc_size {
            panic!("vgicv2_remap_init: gicv_size not equal to gicc_size");
        }
        // map gicv memory region to gicc memory region.
        self.gpm.insert(MemoryRegion::new_with_offset_mapper(arch.gicc_base,arch.gicv_base,arch.gicc_size,MemFlags::READ | MemFlags::WRITE)).unwrap();
    }

    // store the interrupt number in the irq_bitmap.
    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {
        // Enable each cpu's sgi and ppi access permission
        self.irq_bitmap[0] = 0xffff_ffff;
        for irq in irqs {
            self.insert_irq_to_bitmap(*irq);
        }
        for (index, &word) in self.irq_bitmap.iter().enumerate() {
            for bit_position in 0..32 {
                if word & (1 << bit_position) != 0 {
                    let interrupt_number = index * 32 + bit_position;
                    info!(
                        "Found interrupt in Zone {} irq_bitmap: {}",
                        self.id, interrupt_number
                    );
                }
            }
        }
    }

    // insert the interrupt number into the irq_bitmap.
    fn insert_irq_to_bitmap(&mut self, irq: u32) {
        assert!(irq < get_max_int_num() as u32);
        let irq_index = irq / 32;
        let irq_bit = irq % 32;
        self.irq_bitmap[irq_index as usize] |= 1 << irq_bit;
    }
}

pub fn reg_range(base: usize, n: usize, size: usize) -> core::ops::Range<usize> {
    base..(base + (n - 1) * size)
}

// extend from gicv3, support half-word and byte access.
fn restrict_bitmask_access(
    mmio: &mut MMIOAccess,
    reg_index: usize,
    bits_per_irq: usize,
    is_poke: bool,
    gicd_base: usize,
) -> HvResult {
    let zone = this_zone();
    let zone_r = zone.read();
    let mut access_mask: usize = 0;
    /*
     * In order to avoid division, the number of bits per irq is limited
     * to powers of 2 for the moment.
     */
    let irqs_per_reg = 32 / bits_per_irq;
    let irq_bits = (1 << bits_per_irq) - 1;
    /* First, extract the first interrupt affected by this access */
    let first_irq = reg_index * irqs_per_reg;
    trace!("restrict_bitmask_access: first_irq: {}", first_irq);
    trace!("mmio.address: {:#x}", mmio.address);
    trace!("mmio.size: {:#x}", mmio.size);
    trace!("mmio.value: {:#x}", mmio.value);
    for irq in 0..irqs_per_reg {
        if zone_r.irq_in_zone((first_irq + irq) as _) {
                trace!("restrict visit irq {}", first_irq + irq);
            access_mask |= irq_bits << (irq * bits_per_irq);
        }
    }

    let mut other_mask: usize = 0;
    for offset in 0..mmio.size {
        other_mask |= 0xff << ((mmio.address + offset) % 4) * 8;
    }
    trace!("access_mask: {:#x}, other_mask: {:#x}", access_mask, other_mask);

    // add the mask of the other bits in the register
    let address = mmio.address & 0xfffffffc;
    let size = 4;
    let offset = mmio.address & 0x3;
    let value = mmio.value << (offset * 8);
    let real_mask = access_mask & other_mask;
    trace!("address: {:#x}, size: {:#x}, value: {:#x}, real_mask: {:#x}", address, size, value, real_mask);

    let offset = mmio.address & 0x3;
    mmio.address = mmio.address & 0xfffffffc;
    mmio.size = 4;
    mmio.value <<= offset * 8;
    access_mask = access_mask & other_mask;

    if !mmio.is_write {
        /* Restrict the read value */
        mmio_perform_access(gicd_base, mmio);
        mmio.value &= access_mask;
        mmio.value >>= offset * 8;
        return Ok(());
    }

    if !is_poke {
        /*
         * Modify the existing value of this register by first reading
         * it into mmio->value
         * Relies on a spinlock since we need two mmio accesses.
         */
        let access_val = mmio.value;

        let _lock = GICD_LOCK.lock();

        mmio.is_write = false;
        mmio_perform_access(gicd_base, mmio);

        mmio.is_write = true;
        mmio.value &= !access_mask;
        mmio.value |= access_val & access_mask;
        mmio_perform_access(gicd_base, mmio);

        // drop lock automatically here
    } else {
        mmio.value &= access_mask;
        mmio_perform_access(gicd_base, mmio);
    }
    Ok(())
}


// general GIC Distributor register access.
fn vgicv2_dist_misc_access(mmio: &mut MMIOAccess, gicd_base: usize) -> HvResult {
    let reg = mmio.address;
    if  reg == GICD_CTRL_REG_OFFSET
        || reg == GICD_TYPER_REG_OFFSET
        || reg == GICD_IIDR_REG_OFFSET
        || reg_range(GICD_IDENTIFICATION_OFFSET, GICD_IDENTIFICATION_NUM, GICV2_REG_WIDTH).contains(&reg)
    {
        if !mmio.is_write {
            // ignore write
            mmio_perform_access(gicd_base, mmio);
        }
    } else {
        todo!()
    }

    Ok(())
}

pub fn set_sgi_irq(irq_id: usize, target_list: usize, routing_mode: usize) {
    let val = irq_id | target_list << GICD_SGIR_TARGET_LIST_FILTER_SHIFT | routing_mode << GICD_SGIR_ROUTING_SHIFT;
    trace!("set_sgi_irq: irq_id: {}, target_list: {}, routing_mode: {}", irq_id, target_list, routing_mode);
    trace!("ISENABLER: {:#x}", GICD.get_isenabler(0));
    GICD.set_sgir(val as u32);
}

// Handle GIC Distributor register accesses.
pub fn vgicv2_dist_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    let gicd_base = GICV2.gicd_base;
    let reg = mmio.address;

    match reg {
        reg if reg == GICD_SGIR_REG_OFFSET => {
            if !mmio.is_write {
                return Ok(());
            }
            mmio_perform_access(gicd_base,mmio);
            Ok(())
        }
        reg if reg_range(GICD_ITARGETSR_REG_OFFSET, GICV2_TARGET_REGS_NUM, GICV2_REG_WIDTH).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x3ff) / 4, 8, true, gicd_base)
        }
        reg if reg_range(GICD_ICENABLER_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg)
            || reg_range(GICD_ISENABLER_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg)
            || reg_range(GICD_ICPENDR_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg)
            || reg_range(GICD_ISPENDR_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg)
            || reg_range(GICD_ICACTIVER_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg)
            || reg_range(GICD_ISACTIVER_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg) =>
            {
                restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, true, gicd_base)
            }
        reg if reg_range(GICD_IGROUPR_REG_OFFSET, GICV2_INT_REGS_NUM, GICV2_REG_WIDTH).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, false, gicd_base)
        }
        reg if reg_range(GICD_ICFGR_REG_OFFSET, GICV2_CONFIG_REGS_NUM, GICV2_REG_WIDTH).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0xff) / 4, 2, false, gicd_base)
        }
        reg if reg_range(GICD_IPRIORITYR_REG_OFFSET, GICV2_PRIO_REGS_NUM, GICV2_REG_WIDTH).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x3ff) / 4, 8, false, gicd_base)
        }
        _ => vgicv2_dist_misc_access(mmio, gicd_base),
    }
}



