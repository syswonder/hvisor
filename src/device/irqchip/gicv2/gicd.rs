#![allow(unused_variables)]
#![allow(dead_code)]


use spin::Mutex;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};
use crate::device::irqchip::gicv2::gic_ref::GicRef;
use crate::device::irqchip::gicv2::GICV2;
// Hypervisor running in non-secure mode，can only access non-secure registers.

// GICV2 interrupt layout.
pub const GICV2_INTS_MAX: usize = 1024;
pub const GICV2_SGIS_NUM: usize = 16;
pub const GICV2_PPIS_NUM: usize = 16;
pub const GICV2_PRIVATE_INTS_NUM: usize = GICV2_SGIS_NUM + GICV2_PPIS_NUM;


// GICD Register numbers;
pub const GICV2_INT_REGS_NUM: usize = GICV2_INTS_MAX / 32;
pub const GICV2_PRIO_REGS_NUM: usize = GICV2_INTS_MAX * 8 / 32;
pub const GICV2_TARGET_REGS_NUM: usize = GICV2_INTS_MAX * 8 / 32;
pub const GICV2_CONFIG_REGS_NUM: usize = GICV2_INTS_MAX / 16;
pub const GICV2_SGI_REGS_NUM: usize = GICV2_SGIS_NUM * 8 / 32;
/* OPTIONAL */
pub const GICV2_NSACR_REGS_NUM: usize = GICV2_INTS_MAX * 2 / 32;


// GICD BITS
pub const GICD_CTLR_EN_BIT: usize = 0x1;

// GICD Register offsets.
pub const GICD_CTRL_REG_OFFSET: usize = 0x0000;
pub const GICD_TYPER_REG_OFFSET: usize = 0x0004;
pub const GICD_IIDR_REG_OFFSET: usize = 0x0008;
pub const GICD_IGROUPR_REG_OFFSET: usize = 0x0080;
pub const GICD_ISENABLER_REG_OFFSET: usize = 0x0100;
pub const GICD_ICENABLER_REG_OFFSET: usize = 0x0180;
pub const GICD_ISPENDR_REG_OFFSET: usize = 0x0200;
pub const GICD_ICPENDR_REG_OFFSET: usize = 0x0280;
pub const GICD_ISACTIVER_REG_OFFSET: usize = 0x0300;
pub const GICD_ICACTIVER_REG_OFFSET: usize = 0x0380;
pub const GICD_IPRIORITYR_REG_OFFSET: usize = 0x0400;
pub const GICD_ITARGETSR_REG_OFFSET: usize = 0x0800;
pub const GICD_ICFGR_REG_OFFSET: usize = 0x0C00;
pub const GICD_NSACR_REG_OFFSET: usize = 0x0E00;
pub const GICD_SGIR_REG_OFFSET: usize = 0x0F00;
pub const GICD_CPENDSGIR_REG_OFFSET: usize = 0x0F10;
pub const GICD_SPENDSGIR_REG_OFFSET: usize = 0x0F20;
pub const GICD_IDENTIFICATION_NUM: usize = 12;
pub const GICD_IDENTIFICATION_OFFSET: usize = 0x0FE0;
pub const GICD_END: usize = 0x1000;
pub const GICD_SGIR_ROUTING_SHIFT: usize = 24;
pub const GICD_SGIR_TARGET_LIST_FILTER_SHIFT: usize = 16;
// GICD Register layout.
register_structs! {
    #[allow(non_snake_case)]
    pub GicDistributer {
        (0x0000 => CTRL: ReadWrite<u32>),
        (0x0004 => TYPER: ReadOnly<u32>),
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => reserve0),
        (0x0080 => IGROUPR: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0100 => ISENABLER: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0180 => ICENABLER: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0200 => ISPENDR: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0280 => ICPENDR: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0300 => ISACTIVER: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0380 => ICACTIVER: [ReadWrite<u32>; GICV2_INT_REGS_NUM]),
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; GICV2_PRIO_REGS_NUM]),
        (0x0800 => ITARGETSR: [ReadWrite<u32>; GICV2_TARGET_REGS_NUM]),
        (0x0C00 => ICFGR: [ReadWrite<u32>; GICV2_CONFIG_REGS_NUM]),
        (0x0d00 => reserve1),
        (0x0E00 => NSACR: [ReadWrite<u32>; GICV2_NSACR_REGS_NUM]),
        (0x0F00 => SGIR: ReadWrite<u32>),
        (0x0f04 => reserve2),
        (0x0F10 => CPENDSGIR: [ReadWrite<u32>; GICV2_SGI_REGS_NUM]),
        (0x0F20 => SPENDSGIR: [ReadWrite<u32>; GICV2_SGI_REGS_NUM]),
        (0x0F30 => reserved3),
        (0x0FD0 => IDENTIFICATION: [ReadOnly<u32>; GICD_IDENTIFICATION_NUM]),
        (0x1000 => @END),
    }
}
unsafe impl Sync for GicDistributer {}

// GICD is globally unique.
pub static GICD: GicRef<GicDistributer> = unsafe { GicRef::new(GICV2.gicd_base as *const GicDistributer) };
pub static GICD_LOCK: Mutex<()> = Mutex::new(());

impl GicDistributer {
    // init GICD globally and enable it.
    pub fn global_init(&self) {
        let prev = self.CTRL.get();
        // Enable the distributor.
        self.CTRL.set(prev | GICD_CTLR_EN_BIT as u32);
        info!("GICV2: GICD global init done");
    }

    // because some registers are banked, we need every cpu call this function to reset their own interrupt.
    pub fn cpu_init(&self) {
        // Reset private interrupts.
        for i in 0..GICV2_PRIVATE_INTS_NUM / 32 {
            self.ICENABLER[i].set(u32::MAX);
            self.ICPENDR[i].set(u32::MAX);
            self.ICACTIVER[i].set(u32::MAX);
        }
        // Clear pending sgis.
        for i in 0..GICV2_SGI_REGS_NUM / 4 {
            self.CPENDSGIR[i].set(u32::MAX);
            self.SPENDSGIR[i].set(u32::MAX);
        }
        // Set all private interrupts has the lowest priority.
        for i in 0..GICV2_PRIVATE_INTS_NUM / 4 {
            self.IPRIORITYR[i].set(u32::MAX);
        }
    }

    pub fn set_isenabler(&self, index: usize, value: u32) {
        self.ISENABLER[index].set(value);
    }

    pub fn set_isactiver(&self, index: usize, value: u32) {
        self.ISACTIVER[index].set(value);
    }

    pub fn get_isactiver(&self, index: usize) -> u32 {
        self.ISACTIVER[index].get()
    }

    pub fn set_icenabler(&self, index: usize, value: u32) {
        self.ICENABLER[index].set(value);
    }

    pub fn set_icactiver(&self, index: usize, value: u32) {
        self.ICACTIVER[index].set(value);
    }

    pub fn get_itargetsr(&self, index: usize) -> u32 {
        self.ITARGETSR[index].get()
    }

    pub fn set_sgir(&self, value: u32) {
        self.SGIR.set(value);
    }

    pub fn get_isenabler(&self, index: usize) -> u32 { self.ISENABLER[index].get() }

    pub fn set_icpender(&self, index: usize, value: u32) {
        self.ICPENDR[index].set(value);
    }

    pub fn get_ispendr(&self, index: usize) -> u32 {
        self.ISPENDR[index].get()
    }

    pub fn get_spendsgir(&self, index: usize) -> u32 {
        self.SPENDSGIR[index].get()
    }

    pub fn set_cpendsgir(&self, index: usize, value: u32) {
        self.CPENDSGIR[index].set(value);
    }

}

// Get the maximum number of interrupt IDs that the GIC supports.
pub fn get_max_int_num() -> usize {
    let value = (GICD.TYPER.get() & 0b11111) as usize;
    (value + 1) * 32
}

// Check if the interrupt is a sgi.
pub fn is_spi(irqn: usize) -> bool {
    irqn > 31 && irqn < 1020
}

// Get the base address of GICD.
pub fn host_gicd_base() -> usize {
    GICV2.gicd_base
}

