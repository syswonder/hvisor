// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//

#![allow(dead_code)]
pub mod gic;
pub mod gicd;
pub mod gich;
pub mod gicr;
pub mod gits;
pub mod lr;
pub mod msix_backend;
pub mod pending;
pub mod vgic;

use core::arch::asm;
use core::ptr::write_volatile;

use alloc::vec::Vec;
use gicr::init_lpi_prop;
use gits::gits_init;
use spin::{Lazy, Once};

use self::gicd::{enable_gic_are_ns, GICD_ICACTIVER, GICD_ICENABLER};
use self::gich::{list_regs_num, write_lr};
use self::gicr::{enable_ipi, GICR_BASE_LAYOUT};
use self::pending::{PendingIrqs, PENDING_VIRQS};
use crate::arch::aarch64::sysreg::{read_sysreg, smc_arg1, write_sysreg};
use crate::arch::zone::GicConfig;
use crate::config::root_zone_config;
use crate::consts::{self, MAX_CPU_NUM};

use crate::device::irqchip::gicv3::gits::gits_reset;
use crate::zone::Zone;

pub static GIC: Once<Gic> = Once::new();
pub const PER_GICR_SIZE: usize = 0x20000;

// GICR register offsets and fields
const GICR_TYPER_AFFINITY_VALUE_SHIFT: usize = 32;
const GICR_TYPER_AFFINITY_VALUE_MASK: u64 = 0xFFFFFFFF << GICR_TYPER_AFFINITY_VALUE_SHIFT;

#[derive(Debug)]
pub struct Gic {
    pub gicd_base: usize,
    pub gicr_base: usize,
    pub gicd_size: usize,
    pub gicr_size: usize,
    pub gits_base: usize,
    pub gits_size: usize,
}

pub fn host_gicd_base() -> usize {
    GIC.get().unwrap().gicd_base
}

pub static CPU_UNUSED_GICR_BASE: Lazy<Vec<usize>> = Lazy::new(|| {
    let unused_bases = GICR_BASE_LAYOUT.unused_bases();
    debug!("Unused GICR bases: {:#x?}", unused_bases);
    unused_bases
});

static CPU_GICR_BASE: Lazy<Vec<usize>> = Lazy::new(|| {
    let bases = GICR_BASE_LAYOUT.cpu_bases();
    info!("GICR bases: {:#x?}", bases);
    bases
});

pub fn host_gicr_base(id: usize) -> usize {
    assert!(id < consts::MAX_CPU_NUM);
    CPU_GICR_BASE[id]
}

pub fn host_gits_base() -> usize {
    GIC.get().unwrap().gits_base
}

pub fn host_gicd_size() -> usize {
    GIC.get().unwrap().gicd_size
}

pub fn host_gicr_size() -> usize {
    GIC.get().unwrap().gicr_size
}

pub fn host_gits_size() -> usize {
    GIC.get().unwrap().gits_size
}

pub fn is_spi(irqn: u32) -> bool {
    irqn > 31 && irqn < 1020
}

pub fn is_sgi(irqn: u32) -> bool {
    irqn < 16
}

pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #0xf") };
}

pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #0xf") };
}

pub fn primary_init_early() {
    let root_config = root_zone_config();
    match root_config.arch_config.gic_config {
        GicConfig::Gicv2(_) => {
            panic!("GICv2 is not supported in this version of hvisor");
        }
        GicConfig::Gicv3(ref gicv3_config) => {
            info!("GICv3 detected");
            GIC.call_once(|| Gic {
                gicd_base: gicv3_config.gicd_base,
                gicr_base: gicv3_config.gicr_base,
                gicd_size: gicv3_config.gicd_size,
                gicr_size: gicv3_config.gicr_size,
                gits_base: gicv3_config.gits_base,
                gits_size: gicv3_config.gits_size,
            });
            info!(
                "GIC Distributor base: {:#x}, size: {:#x}",
                GIC.get().unwrap().gicd_base,
                GIC.get().unwrap().gicd_size
            );
            info!(
                "GIC Redistributor base: {:#x}, size: {:#x}",
                GIC.get().unwrap().gicr_base,
                GIC.get().unwrap().gicr_size
            );
            info!(
                "GIC ITS base: {:#x}, size: {:#x}",
                GIC.get().unwrap().gits_base,
                GIC.get().unwrap().gits_size
            );
        }
    }
    init_lpi_prop();

    if host_gits_base() != 0 && host_gits_size() != 0 {
        gits_init();
    }

    PENDING_VIRQS.call_once(|| PendingIrqs::new(MAX_CPU_NUM));
    debug!("gic = {:#x?}", GIC.get().unwrap());
}

pub fn primary_init_late() {
    enable_gic_are_ns();
    enable_irqs();
}

pub fn percpu_init() {
    gicc_init();
    enable_ipi();
}

//TODO: add Distributor init
pub fn gicc_init() {
    //TODO: add Redistributor init
    let sdei_ver = unsafe { smc_arg1!(0xc4000020) }; //sdei_check();

    // Make ICC_EOIR1_EL1 provide priority drop functionality only. ICC_DIR_EL1 provides interrupt deactivation functionality.
    let _ctlr = read_sysreg!(icc_ctlr_el1);
    write_sysreg!(icc_ctlr_el1, 0x2);
    // Set Interrupt Controller Interrupt Priority Mask Register
    let pmr = read_sysreg!(icc_pmr_el1);
    write_sysreg!(icc_pmr_el1, 0xf0);
    // Enable group 1 irq
    let _igrpen = read_sysreg!(icc_igrpen1_el1);
    write_sysreg!(icc_igrpen1_el1, 0x1);

    gicv3_clear_pending_irqs();
    let _vtr = read_sysreg!(ich_vtr_el2);
    let vmcr = ((pmr & 0xff) << 24) | (1 << 1); //VPMR|VENG1
    write_sysreg!(ich_vmcr_el2, vmcr);
    write_sysreg!(ich_hcr_el2, 0x1); //enable virt cpu interface

    info!("gicc init done, sdei_ver = {}", sdei_ver);
}

fn gicv3_clear_pending_irqs() {
    let vtr = read_sysreg!(ich_vtr_el2) as usize;
    for i in 0..list_regs_num() {
        write_lr(i, 0) //clear lr
    }
    let num_priority_bits = (vtr >> 29) + 1;
    /* Clear active priority bits */
    if num_priority_bits >= 5 {
        write_sysreg!(ICH_AP1R0_EL2, 0); //Interrupt Controller Hyp Active Priorities Group 1 Register 0 No interrupt active
    }
    if num_priority_bits >= 6 {
        write_sysreg!(ICH_AP1R1_EL2, 0);
    }
    if num_priority_bits > 6 {
        write_sysreg!(ICH_AP1R2_EL2, 0);
        write_sysreg!(ICH_AP1R3_EL2, 0);
    }
}

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        let gicd_base = host_gicd_base();
        let zone = self.read();
        for (idx, &mask) in zone.irq_bitmap().iter().enumerate() {
            if idx == 0 {
                continue;
            }
            unsafe {
                write_volatile((gicd_base + GICD_ICENABLER + idx * 4) as *mut u32, mask);
                write_volatile((gicd_base + GICD_ICACTIVER + idx * 4) as *mut u32, mask);
            }
        }
        if host_gits_size() != 0 {
            gits_reset(self.id());
        }
    }
}
