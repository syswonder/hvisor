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

//! The main module and entrypoint
//!
//! Various facilities of hvisor are implemented as submodules. The most
//! important ones are:
//!
//! - [`memory`]: Memory management
//! - [`hypercall`]: Hypercall handling
//! - [`device`]: Device management
//! - [`arch`]: Architecture's related

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
// #![feature(core_panic)]
// #![deny(warnings, missing_docs)]
#![feature(proc_macro_hygiene)]
// unittest
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_main)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate alloc;
extern crate buddy_system_allocator;
#[macro_use]
mod error;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod logging;
mod arch;
mod config;
mod consts;
mod device;
mod event;
mod hypercall;
mod memory;
mod panic;
mod percpu;
mod platform;
mod zone;

mod pci;

#[cfg(test)]
mod tests;

use crate::arch::iommu::iommu_init;
use crate::arch::mm::arch_setup_parange;
use crate::consts::{hv_end, mem_pool_start, MAX_CPU_NUM};
use arch::{cpu::cpu_start, entry::arch_entry};
use config::root_zone_config;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use percpu::PerCpu;
use zone::{add_zone, zone_create};

static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);
static MASTER_CPU: AtomicI32 = AtomicI32::new(-1);

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let mut p = sbss as *mut u8;
    while p < ebss as _ {
        unsafe {
            *p = 0;
            p = p.add(1);
        };
    }
}

fn wait_for(condition: impl Fn() -> bool) {
    while condition() {
        core::hint::spin_loop();
    }
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32) {
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}

fn primary_init_early() {
    extern "C" {
        fn __core_end();
    }
    logging::init();
    info!("Logging is enabled.");
    info!("__core_end = {:#x?}", __core_end as usize);
    info!("mem_pool_start = {:#x?}", mem_pool_start() as usize);
    info!("hv_end = {:#x?}", hv_end() as usize);
    // let system_config = HvSystemConfig::get();
    // let revision = system_config.revision;
    info!("Hypervisor initialization in progress...");
    info!(
        "build_mode: {}, log_level: {}, arch: {}, stats: {}",
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
        option_env!("ARCH").unwrap_or(""),
        option_env!("STATS").unwrap_or("off"),
    );
    memory::frame::init();
    memory::frame::test();
    event::init();

    arch::stage2_mode_detect();

    device::irqchip::primary_init_early();

    iommu_init();

    #[cfg(not(test))]
    {
        let zone = zone_create(root_zone_config()).unwrap();
        add_zone(zone);
    }
    INIT_EARLY_OK.store(1, Ordering::Release);
}

fn primary_init_late() {
    info!("Primary CPU init late...");
    device::irqchip::primary_init_late();

    INIT_LATE_OK.store(1, Ordering::Release);
}

fn per_cpu_init(cpu: &mut PerCpu) {
    if cpu.zone.is_none() {
        warn!("CPU {} is not bound to zone0 (root zone)", cpu.id);
    }
}

fn wakeup_secondary_cpus(this_id: usize, host_dtb: usize) {
    for cpu_id in 0..MAX_CPU_NUM {
        if cpu_id == this_id {
            continue;
        }
        cpu_start(cpu_id, arch_entry as _, host_dtb);
    }
}

fn rust_main(cpuid: usize, host_dtb: usize) {
    arch::trap::install_trap_vector();

    let mut is_primary = false;
    extern "C" {
        fn skernel();
    }
    println!("Hello, start HVISOR at {:#x?}!", skernel as usize);
    if MASTER_CPU.load(Ordering::Acquire) == -1 {
        MASTER_CPU.store(cpuid as i32, Ordering::Release);
        is_primary = true;
        memory::heap::init();
        memory::heap::test();
    }

    let cpu = PerCpu::new(cpuid);

    println!(
        "Booting CPU {}: {:p} arch:{:p}, DTB: {:#x}",
        cpu.id, cpu as *const _, &cpu.arch_cpu as *const _, host_dtb
    );

    if is_primary {
        wakeup_secondary_cpus(cpu.id, host_dtb);
    }

    ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for(|| PerCpu::entered_cpus() < MAX_CPU_NUM as _);
    assert_eq!(PerCpu::entered_cpus(), MAX_CPU_NUM as _);

    println!(
        "{} CPU {} has entered.",
        if is_primary { "Primary" } else { "Secondary" },
        cpu.id
    );

    arch_setup_parange();
    // #[cfg(target_arch = "aarch64")]
    // setup_parange();

    if is_primary {
        primary_init_early(); // create root zone here
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1);
    }

    per_cpu_init(cpu);
    device::irqchip::percpu_init();

    INITED_CPUS.fetch_add(1, Ordering::SeqCst);

    wait_for_counter(&INITED_CPUS, MAX_CPU_NUM as _);

    if is_primary {
        primary_init_late();
    } else {
        wait_for_counter(&INIT_LATE_OK, 1);
    }

    // run all unit tests before starting the root zone
    // CAUTION: test_main will quit qemu after all tests are done
    #[cfg(test)]
    if is_primary {
        test_main();
    }

    cpu.run_vm();
}
