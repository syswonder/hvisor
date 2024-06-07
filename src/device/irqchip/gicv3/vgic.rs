use alloc::sync::Arc;

use super::{gicd::GICD_LOCK, is_spi, Gic};
use crate::{
    consts::MAX_CPU_NUM,
    device::irqchip::gicv3::{gicd::*, gicr::*, host_gicd_base, host_gicr_base, PER_GICR_SIZE},
    error::HvResult,
    memory::{mmio_perform_access, MMIOAccess},
    percpu::{get_cpu_data, this_zone},
    zone::Zone,
};

pub fn reg_range(base: usize, n: usize, size: usize) -> core::ops::Range<usize> {
    base..(base + (n - 1) * size)
}

impl Zone {
    pub fn vgicv3_mmio_init(&mut self, fdt: &fdt::Fdt) {
        let gic = Gic::new(fdt);
        self.mmio_region_register(gic.gicd_base, gic.gicd_size, vgicv3_dist_handler, 0);
        for cpu in 0..MAX_CPU_NUM {
            let gicr_base = host_gicr_base(cpu);
            debug!("registering gicr {} at {:#x?}", cpu, gicr_base);
            self.mmio_region_register(gicr_base, PER_GICR_SIZE, vgicv3_redist_handler, cpu);
        }
    }

    pub fn irq_bitmap_init(&mut self, fdt: &fdt::Fdt) {
        for node in fdt.all_nodes() {
            if node.name == "timer" {
                continue;
            }
            if let Some(int_iter) = node.interrupts() {
                for int_n in int_iter {
                    // When interrupt cell-size = 3, the first cell is a flag indicating if the interrupt is an SPI
                    // So we need to bitwise-and with u32::MAX to get the real interrupt number
                    let real_int_n = (int_n & u32::MAX as usize) + 32;
                    self.insert_irq_to_bitmap(real_int_n as u32);
                }
            }
        }
		if self.id == 1 {
			self.insert_irq_to_bitmap(61); // serial3
			self.insert_irq_to_bitmap(56);  // mmc2
		} else if self.id == 0 { 
			self.insert_irq_to_bitmap(55);  // mmc1
			self.insert_irq_to_bitmap(59);  // serial@30890000
		}
        self.insert_irq_to_bitmap(96);  // gpio 0
        self.insert_irq_to_bitmap(97);  // gpio 0
        self.insert_irq_to_bitmap(98);  // gpio 1
        self.insert_irq_to_bitmap(99);  // gpio 1
        self.insert_irq_to_bitmap(100); // gpio 2
        self.insert_irq_to_bitmap(101); // gpio 2
        self.insert_irq_to_bitmap(102); // gpio 3
        self.insert_irq_to_bitmap(103); // gpio 3
        self.insert_irq_to_bitmap(104); // gpio 4
        self.insert_irq_to_bitmap(105); // gpio 4

        self.insert_irq_to_bitmap(150); // ethernet1
        self.insert_irq_to_bitmap(151); // ethernet1
        self.insert_irq_to_bitmap(152); // ethernet1

        self.insert_irq_to_bitmap(166); // ethernet2
        self.insert_irq_to_bitmap(167); // ethernet2

        self.insert_irq_to_bitmap(52);  // caam_secvio
        self.insert_irq_to_bitmap(51);  // snvs-rtc-l
        self.insert_irq_to_bitmap(36);  // snvs-powerkey
        self.insert_irq_to_bitmap(54);  // mmc0

        self.insert_irq_to_bitmap(43);

        self.insert_irq_to_bitmap(69);  // i2c
        self.insert_irq_to_bitmap(39);  // pmu
        self.insert_irq_to_bitmap(34);  // pmu
    
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

    fn insert_irq_to_bitmap(&mut self, irq: u32) {
        assert!(irq < 1024); // 1024 is the maximum number of interrupts supported by GICv3 (GICD_TYPER.ITLinesNumber)  
        let irq_index = irq / 32;
        let irq_bit = irq % 32;
        self.irq_bitmap[irq_index as usize] |= 1 << irq_bit;
    }
}

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

    for irq in 0..irqs_per_reg {
        if zone_r.irq_in_zone((first_irq + irq) as _) {
            trace!("restrict visit irq {}", first_irq + irq);
            access_mask |= irq_bits << (irq * bits_per_irq);
        }
    }

    if !mmio.is_write {
        /* Restrict the read value */
        mmio_perform_access(gicd_base, mmio);
        mmio.value &= access_mask;
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

pub fn vgicv3_redist_handler(mmio: &mut MMIOAccess, cpu: usize) -> HvResult {
    trace!("gicr({}) mmio = {:#x?}", cpu, mmio);
    let gicr_base = host_gicr_base(cpu);
    match mmio.address {
        GICR_TYPER => {
            mmio_perform_access(gicr_base, mmio);
            if cpu == MAX_CPU_NUM - 1 {
                mmio.value |= GICR_TYPER_LAST;
            }
        }
        GICR_IIDR | 0xffd0..=0xfffc => {
            // Read-only registers that might be used by a zone to find the redistributor corresponding to a CPU. Keep them accessible.
            mmio_perform_access(gicr_base, mmio);
        }
        GICR_SYNCR => {
            mmio.value = 0;
        }
        _ => {
            if Arc::ptr_eq(&this_zone(), get_cpu_data(cpu).zone.as_ref().unwrap()) {
                // ignore access to foreign redistributors
                mmio_perform_access(gicr_base, mmio);
            } else {
                trace!("*** gicv3_gicr_mmio_handler: ignore access to foreign redistributors ***");
            }
        }
    }
    HvResult::Ok(())
}

// The return value should be the register value to be read.
fn vgicv3_handle_irq_ops(mmio: &mut MMIOAccess, irq: u32) -> HvResult {
    let zone = this_zone();
    let zone_r = zone.read();

    if !is_spi(irq) || !zone_r.irq_in_zone(irq) {
        debug!(
            "gicd-mmio: skip irq {} access, reg = {:#x?}",
            irq, mmio.address
        );
        return Ok(());
    }

    mmio_perform_access(host_gicd_base(), mmio);

    Ok(())
}

fn vgicv3_dist_misc_access(mmio: &mut MMIOAccess, gicd_base: usize) -> HvResult {
    let reg = mmio.address;
    if reg_range(GICDV3_PIDR0, 4, 4).contains(&reg)
        || reg_range(GICDV3_PIDR4, 4, 4).contains(&reg)
        || reg_range(GICDV3_CIDR0, 4, 4).contains(&reg)
        || reg == GICD_CTLR
        || reg == GICD_TYPER
        || reg == GICD_IIDR
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

pub fn vgicv3_dist_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    trace!("gicd mmio = {:#x?}", mmio);
    let gicd_base = host_gicd_base();
    let reg = mmio.address;

    match reg {
        reg if reg_range(GICD_IROUTER, 1024, 8).contains(&reg) => {
            vgicv3_handle_irq_ops(mmio, (reg - GICD_IROUTER) as u32 / 8)
        }
        reg if reg_range(GICD_ITARGETSR, 1024, 1).contains(&reg) => {
            vgicv3_handle_irq_ops(mmio, (reg - GICD_ITARGETSR) as u32)
        }
        reg if reg_range(GICD_ICENABLER, 32, 4).contains(&reg)
            || reg_range(GICD_ISENABLER, 32, 4).contains(&reg)
            || reg_range(GICD_ICPENDR, 32, 4).contains(&reg)
            || reg_range(GICD_ISPENDR, 32, 4).contains(&reg)
            || reg_range(GICD_ICACTIVER, 32, 4).contains(&reg)
            || reg_range(GICD_ISACTIVER, 32, 4).contains(&reg) =>
        {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, true, gicd_base)
        }
        reg if reg_range(GICD_IGROUPR, 32, 4).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x7f) / 4, 1, false, gicd_base)
        }
        reg if reg_range(GICD_ICFGR, 64, 4).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0xff) / 4, 2, false, gicd_base)
        }
        reg if reg_range(GICD_IPRIORITYR, 255, 4).contains(&reg) => {
            restrict_bitmask_access(mmio, (reg & 0x3ff) / 4, 8, false, gicd_base)
        }
        _ => vgicv3_dist_misc_access(mmio, gicd_base),
    }
}
