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
pub mod gicd;
pub mod gicr;
pub mod gits;
pub mod vgic;

use core::arch::asm;
use core::ptr::write_volatile;
use core::sync::atomic::AtomicU64;

use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use gicr::init_lpi_prop;
use gits::gits_init;
use spin::{Lazy, Mutex, Once};

use self::gicd::{enable_gic_are_ns, GICD_ICACTIVER, GICD_ICENABLER};
use self::gicr::enable_ipi;
use crate::arch::aarch64::sysreg::{read_sysreg, smc_arg1, write_sysreg};
use crate::arch::cpu::{cpuid_to_mpidr_affinity, this_cpu_id};
use crate::arch::zone::GicConfig;
use crate::config::root_zone_config;
use crate::consts::{self, MAX_CPU_NUM};

use crate::event::check_events;
use crate::hypercall::SGI_IPI_ID;
use crate::zone::Zone;

const ICH_HCR_UIE: u64 = 1 << 1;
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
    let lr_num: usize = (vtr & 0xf) + 1;
    for i in 0..lr_num {
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

static TIMER_INTERRUPT_COUNTER: AtomicU64 = AtomicU64::new(0);
// how often to print timer interrupt counter
const TIMER_INTERRUPT_PRINT_INTERVAL: u64 = 50;

pub fn gicv3_handle_irq_el1() {
    while let Some(irq_id) = pending_irq() {
        if irq_id < 8 {
            trace!("sgi get {}, try to handle...", irq_id);
            deactivate_irq(irq_id);
            let mut ipi_handled = false;
            if irq_id == SGI_IPI_ID as _ {
                ipi_handled = check_events();
            }
            if !ipi_handled {
                trace!("sgi get {}, inject", irq_id);
                inject_irq(irq_id, false);
            }
        } else if irq_id < 16 {
            warn!("skip sgi {}", irq_id);
            deactivate_irq(irq_id);
        } else {
            if irq_id == 27 {
                // virtual timer interrupt
                TIMER_INTERRUPT_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
                if TIMER_INTERRUPT_COUNTER.load(core::sync::atomic::Ordering::SeqCst)
                    % TIMER_INTERRUPT_PRINT_INTERVAL
                    == 0
                {
                    trace!(
                        "Virtual timer interrupt, counter = {}",
                        TIMER_INTERRUPT_COUNTER.load(core::sync::atomic::Ordering::SeqCst)
                    );
                }
            } else if irq_id == 25 {
                // maintenace interrupt
                handle_maintenace_interrupt();
            } else if irq_id > 31 {
                //inject phy irq
                trace!("*** get spi_irq id = {}", irq_id);
            } else {
                warn!("not konw irq id = {}", irq_id);
            }
            if irq_id != 25 {
                inject_irq(irq_id, true);
            }
            deactivate_irq(irq_id);
        }
    }
    trace!("handle done")
}

fn pending_irq() -> Option<usize> {
    let iar = read_sysreg!(icc_iar1_el1) as usize;
    if iar == 0x3ff {
        None
    } else {
        Some(iar as _)
    }
}

fn deactivate_irq(irq_id: usize) {
    write_sysreg!(icc_eoir1_el1, irq_id as u64);
    if irq_id < 16 || irq_id == 25 {
        write_sysreg!(icc_dir_el1, irq_id as u64);
    }
}

fn read_lr(id: usize) -> u64 {
    let id = id as u64;
    match id {
        //TODO get lr size from gic reg
        0 => read_sysreg!(ich_lr0_el2),
        1 => read_sysreg!(ich_lr1_el2),
        2 => read_sysreg!(ich_lr2_el2),
        3 => read_sysreg!(ich_lr3_el2),
        4 => read_sysreg!(ich_lr4_el2),
        5 => read_sysreg!(ich_lr5_el2),
        6 => read_sysreg!(ich_lr6_el2),
        7 => read_sysreg!(ich_lr7_el2),
        8 => read_sysreg!(ich_lr8_el2),
        9 => read_sysreg!(ich_lr9_el2),
        10 => read_sysreg!(ich_lr10_el2),
        11 => read_sysreg!(ich_lr11_el2),
        12 => read_sysreg!(ich_lr12_el2),
        13 => read_sysreg!(ich_lr13_el2),
        14 => read_sysreg!(ich_lr14_el2),
        15 => read_sysreg!(ich_lr15_el2),
        _ => {
            error!("lr over");
            loop {}
        }
    }
}

fn write_lr(id: usize, val: u64) {
    let id = id as u64;
    match id {
        0 => write_sysreg!(ich_lr0_el2, val),
        1 => write_sysreg!(ich_lr1_el2, val),
        2 => write_sysreg!(ich_lr2_el2, val),
        3 => write_sysreg!(ich_lr3_el2, val),
        4 => write_sysreg!(ich_lr4_el2, val),
        5 => write_sysreg!(ich_lr5_el2, val),
        6 => write_sysreg!(ich_lr6_el2, val),
        7 => write_sysreg!(ich_lr7_el2, val),
        8 => write_sysreg!(ich_lr8_el2, val),
        9 => write_sysreg!(ich_lr9_el2, val),
        10 => write_sysreg!(ich_lr10_el2, val),
        11 => write_sysreg!(ich_lr11_el2, val),
        12 => write_sysreg!(ich_lr12_el2, val),
        13 => write_sysreg!(ich_lr13_el2, val),
        14 => write_sysreg!(ich_lr14_el2, val),
        15 => write_sysreg!(ich_lr15_el2, val),
        _ => {
            error!("lr over");
            loop {}
        }
    }
}

// virtual interrupts waiting to inject
static PENDING_VIRQS: Once<PendingIrqs> = Once::new();
pub const MAINTENACE_INTERRUPT: u64 = 25;
struct PendingIrqs {
    inner: Vec<Mutex<VecDeque<(usize, bool)>>>,
}

impl PendingIrqs {
    fn new(max_cpus: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_cpus {
            let v = Mutex::new(VecDeque::new());
            vs.push(v)
        }
        Self { inner: vs }
    }

    fn add_irq(&self, irq_id: usize, is_hardware: bool) -> Option<()> {
        match self.inner.get(this_cpu_id()) {
            Some(pending_irqs) => {
                let mut irqs = pending_irqs.lock();
                irqs.push_back((irq_id, is_hardware));
                Some(())
            }
            _ => None,
        }
    }

    fn fetch_irq(&self) -> Option<(usize, bool)> {
        match self.inner.get(this_cpu_id()) {
            Some(pending_irqs) => {
                let mut irqs = pending_irqs.lock();
                irqs.pop_front()
            }
            _ => None,
        }
    }
}

// Enable or disable an underflow maintenace interrupt.
fn enable_maintenace_interrupt(is_enable: bool) {
    trace!("enable_maintenace_interrupt, is_enable is {}", is_enable);
    let mut hcr = read_sysreg!(ich_hcr_el2);
    trace!("hcr is {}", hcr);
    if is_enable {
        hcr |= ICH_HCR_UIE;
    } else {
        hcr &= !ICH_HCR_UIE;
    }
    write_sysreg!(ich_hcr_el2, hcr);
}

fn handle_maintenace_interrupt() {
    trace!("handle_maintenace_interrupt");
    let pending_irqs = PENDING_VIRQS.get().unwrap();
    while let Some((irq_id, is_hardware)) = pending_irqs.fetch_irq() {
        let is_injected: bool = inject_irq(irq_id, is_hardware);
        if is_injected {
            trace!("inject pending irq in maintenace interrupt");
        }
        if !is_injected {
            pending_irqs.add_irq(irq_id, is_hardware);
            enable_maintenace_interrupt(true);
            return;
        }
    }
    enable_maintenace_interrupt(false);
}

/// Inject virtual interrupt to vCPU, return whether it not needs to add pending queue.
pub fn inject_irq(irq_id: usize, is_hardware: bool) -> bool {
    // mask
    const LR_VIRTIRQ_MASK: usize = (1 << 32) - 1;

    let elsr: u64 = read_sysreg!(ich_elrsr_el2);
    let vtr = read_sysreg!(ich_vtr_el2) as usize;
    let lr_num: usize = (vtr & 0xf) + 1;
    let mut free_ir = -1 as isize;
    for i in 0..lr_num {
        // find a free list register
        if (1 << i) & elsr > 0 {
            if free_ir == -1 {
                free_ir = i as isize;
            }
            continue;
        }
        let lr_val = read_lr(i) as usize;
        // if a virtual interrupt is enabled and equals to the physical interrupt irq_id
        if (lr_val & LR_VIRTIRQ_MASK) == irq_id {
            trace!("virtual irq {} enables again", irq_id);
            return true;
        }
    }
    trace!("To Inject IRQ {}, find lr {}", irq_id, free_ir);

    if free_ir == -1 {
        trace!("all list registers are valid, add to pending queue");
        // If all list registers are valid, add this virtual irq to pending queue,
        // and enable an underflow maintenace interrupt. When list registers are
        // all invalid or only one is valid, the maintenace interrupt will occur,
        // hvisor will execute handle_maintenace_interrupt function.
        PENDING_VIRQS
            .get()
            .unwrap()
            .add_irq(irq_id, is_hardware)
            .unwrap();
        enable_maintenace_interrupt(true);
        return false;
    } else {
        let mut val = irq_id as u64; //v intid
        val |= 1 << 60; //group 1
        val |= 1 << 62; //state pending

        if !is_sgi(irq_id as _) && is_hardware {
            val |= 1 << 61; //map hardware
            val |= (irq_id as u64) << 32; //pINTID
        }
        write_lr(free_ir as usize, val);
        return true;
    }
}

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

static CPU_GICR_BASE: Lazy<Vec<usize>> = Lazy::new(|| {
    let mut bases = vec![0; MAX_CPU_NUM];
    let gic = GIC.get().unwrap();
    let base = gic.gicr_base;
    let mut found_cpus = 0;

    // Scan through all GICR frames once
    let mut curr_base = base;

    for _ in 0..MAX_CPU_NUM {
        let typer =
            unsafe { core::ptr::read_volatile((curr_base + gicr::GICR_TYPER) as *const u64) };
        let affinity = (typer & GICR_TYPER_AFFINITY_VALUE_MASK) >> GICR_TYPER_AFFINITY_VALUE_SHIFT;

        // Find which CPU this GICR belongs to
        if let Some(cpu_id) = (0..MAX_CPU_NUM).position(|cpu_id| {
            let (aff3, aff2, aff1, aff0) = cpuid_to_mpidr_affinity(cpu_id as u64);
            let aff = (aff3 << 24) | (aff2 << 16) | (aff1 << 8) | aff0;
            aff == affinity
        }) {
            bases[cpu_id] = curr_base;
            found_cpus += 1;
        }
        curr_base += PER_GICR_SIZE;
    }

    if found_cpus != MAX_CPU_NUM {
        panic!(
            "Could not find GICR for all CPUs, only found {}",
            found_cpus
        );
    }
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

impl Zone {
    pub fn arch_irqchip_reset(&self) {
        let gicd_base = host_gicd_base();
        for (idx, &mask) in self.irq_bitmap.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            unsafe {
                write_volatile((gicd_base + GICD_ICENABLER + idx * 4) as *mut u32, mask);
                write_volatile((gicd_base + GICD_ICACTIVER + idx * 4) as *mut u32, mask);
            }
        }
    }
}
