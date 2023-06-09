use crate::memory::addr::VirtAddr;
use spin::Mutex;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
register_structs! {
    #[allow(non_snake_case)]
    GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => IGROUPR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => ISPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Priority Registers.
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => ITARGETSR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Configuration Registers.
        (0x0c00 => ICFGR: [ReadWrite<u32>; 0x40]),
        (0x0d00 => _reserved_1),
        /// Software Generated Interrupt Register.
        (0x0f00 => SGIR: WriteOnly<u32>),
        (0x0f04 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    GicCpuInterfaceRegs {
        /// CPU Interface Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Priority Mask Register.
        (0x0004 => PMR: ReadWrite<u32>),
        /// Binary Point Register.
        (0x0008 => BPR: ReadWrite<u32>),
        /// Interrupt Acknowledge Register.
        (0x000c => IAR: ReadOnly<u32>),
        /// End of Interrupt Register.
        (0x0010 => EOIR: WriteOnly<u32>),
        /// Running Priority Register.
        (0x0014 => RPR: ReadOnly<u32>),
        /// Highest Priority Pending Interrupt Register.
        (0x0018 => HPPIR: ReadOnly<u32>),
        (0x001c => _reserved_1),
        /// CPU Interface Identification Register.
        (0x00fc => IIDR: ReadOnly<u32>),
        (0x0100 => _reserved_2),
        /// Deactivate Interrupt Register.
        (0x1000 => DIR: WriteOnly<u32>),
        (0x1004 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    GicHypervisorRegs {
        /// Hypervisor Control Register.
        (0x0000 => HCR: ReadWrite<u32>),
        /// VGIC Type Register.
        (0x0004 => VTR: ReadOnly<u32>),
        /// Virtual Machine Control Register.
        (0x0008 => VMCR: ReadWrite<u32>),
        (0x000c => _reserved_0),
        // Maintenance Interrupt Status Register.
        (0x0010 => MISR: ReadOnly<u32>),
        (0x0014 => _reserved_1),
        // End of Interrupt Status Registers 0 and 1.
        (0x0020 => EISR0: ReadOnly<u32>),
        (0x0024 => EISR1: ReadOnly<u32>),
        (0x0028 => _reserved_2),
        // Empty List Register Status Registers 0 and 1.
        (0x0030 => ELSR0: ReadOnly<u32>),
        (0x0034 => ELSR1: ReadOnly<u32>),
        (0x0038 => _reserved_3),
        // Active Priorities Register.
        (0x00f0 => APR: ReadWrite<u32>),
        (0x00f4 => _reserved_4),
        // List Registers 0-63.
        (0x0100 => LR: [ReadWrite<u32>; 0x40]),
        (0x0200 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    GicVcpuInterfaceRegs {
        /// CPU Interface Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Priority Mask Register.
        (0x0004 => PMR: ReadWrite<u32>),
        /// Binary Point Register.
        (0x0008 => BPR: ReadWrite<u32>),
        /// Interrupt Acknowledge Register.
        (0x000c => IAR: ReadOnly<u32>),
        /// End of Interrupt Register.
        (0x0010 => EOIR: WriteOnly<u32>),
        /// Running Priority Register.
        (0x0014 => RPR: ReadOnly<u32>),
        /// Highest Priority Pending Interrupt Register.
        (0x0018 => HPPIR: ReadOnly<u32>),
        (0x001c => _reserved_1),
        /// CPU Interface Identification Register.
        (0x00fc => IIDR: ReadOnly<u32>),
        (0x0100 => _reserved_2),
        /// Deactivate Interrupt Register.
        (0x1000 => DIR: WriteOnly<u32>),
        (0x1004 => @END),
    }
}

struct Gic {
    gicd_base: VirtAddr,
    gicc_base: VirtAddr,
    gich_base: VirtAddr,
    gicv_base: VirtAddr,
    max_irqs: usize,
}
impl Gic {
    const fn new(
        gicd_base: VirtAddr,
        gicc_base: VirtAddr,
        gich_base: VirtAddr,
        gicv_base: VirtAddr,
    ) -> Self {
        Self {
            gicd_base,
            gicc_base,
            gich_base,
            gicv_base,
            max_irqs: 0,
        }
    }

    const fn gicd(&self) -> &GicDistributorRegs {
        unsafe { &*(self.gicd_base as *const _) }
    }

    const fn gicc(&self) -> &GicCpuInterfaceRegs {
        unsafe { &*(self.gicc_base as *const _) }
    }

    const fn gich(&self) -> &GicHypervisorRegs {
        unsafe { &*{ self.gich_base as *const _ } }
    }

    const fn gicv(&self) -> &GicVcpuInterfaceRegs {
        unsafe { &*{ self.gicv_base as *const _ } }
    }

    fn cpu_num(&self) -> usize {
        ((self.gicd().TYPER.get() as usize >> 5) & 0b111) + 1
    }

    fn configure_interrupt(&self, vector: usize, tm: TriggerMode, pol: Polarity) {
        // Only configurable for SPI interrupts
        assert!(vector < self.max_irqs);
        assert!(vector >= SPI_BASE);
        // TODO: polarity should actually be configure through a GPIO controller
        assert!(matches!(pol, Polarity::ActiveHigh));

        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let reg_ndx = vector >> 4;
        let bit_shift = ((vector & 0xf) << 1) + 1;
        let mut reg_val = self.gicd().ICFGR[reg_ndx].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }
        self.gicd().ICFGR[reg_ndx].set(reg_val);
    }

    fn set_enable(&self, vector: usize, enable: bool) {
        assert!(vector < self.max_irqs);
        let reg = vector / 32;
        let mask = 1 << (vector % 32);
        if enable {
            self.gicd().ISENABLER[reg].set(mask);
        } else {
            self.gicd().ICENABLER[reg].set(mask);
        }
    }

    fn lr_num(&self) -> usize {
        (self.gich().VTR.get() as usize & 0b11111) + 1
    }

    fn read_lr(&self, id: usize) -> u32 {
        self.gich().LR[id].get()
    }

    fn write_lr(&self, id: usize, val: u32) {
        self.gich().LR[id].set(val)
    }

    fn pending_irq(&self) -> Option<usize> {
        let iar = self.gicc().IAR.get();
        if iar >= 0x3fe {
            // spurious
            None
        } else {
            Some(iar as _)
        }
    }

    fn inject_irq(&self, irq_id: usize) {
        let elsr: u64 = (self.gich().ELSR1.get() as u64) << 32 | self.gich().ELSR0.get() as u64;
        let lr_num = self.lr_num();
        let mut lr_idx = -1 as isize;
        for i in 0..lr_num {
            if (1 << i) & elsr > 0 {
                if lr_idx == -1 {
                    lr_idx = i as isize;
                }
                continue;
            }

            // overlap
            let lr_val = self.read_lr(i) as usize;
            if (i & LR_VIRTIRQ_MASK) == irq_id {
                return;
            }
        }
        debug!("To Inject IRQ {}, find lr {}", irq_id, lr_idx);

        if lr_idx == -1 {
            return;
        } else {
            let mut val = 0;

            val = irq_id as u32;
            val |= LR_PENDING_BIT;

            if false
            /* sgi */
            {
                todo!()
            } else {
                val |= ((irq_id << 10) & LR_PHYSIRQ_MASK) as u32;
                val |= LR_HW_BIT;
            }

            debug!("To write lr {} val {}", lr_idx, val);
            self.write_lr(lr_idx as usize, val);
        }
    }

    fn eoi(&self, vector: usize) {
        self.gicc().EOIR.set(vector as _);
    }

    fn init(&mut self) {
        self.max_irqs = ((self.gicd().TYPER.get() as usize & 0b11111) + 1) * 32;

        let gicd = self.gicd();
        let gicc = self.gicc();
        let gich = self.gich();
        let gicv = self.gicv();

        for i in (0..self.max_irqs).step_by(32) {
            gicd.ICENABLER[i / 32].set(u32::MAX);
            gicd.ICPENDR[i / 32].set(u32::MAX);
        }
        if self.cpu_num() > 1 {
            for i in (SPI_BASE..self.max_irqs).step_by(4) {
                // Set external interrupts to target cpu 0
                gicd.ITARGETSR[i / 4].set(0x01_01_01_01);
            }
        }
        // Initialize all the SPIs to edge triggered
        for i in SPI_BASE..self.max_irqs {
            self.configure_interrupt(i, TriggerMode::Edge, Polarity::ActiveHigh);
        }

        // enable GIC
        gicd.CTLR.set(1);
        gicc.CTLR.set(0x201); // EOIMode | En
        gich.VMCR.set(0x201);
        gich.HCR.set(1);
        // unmask interrupts at all priority levels
        gicc.PMR.set(0xff);
    }
}
