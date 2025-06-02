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

extern crate fdt_rs;

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

#[cfg(target_arch = "aarch64")]
mod ivc;

mod pci;

#[cfg(test)]
mod tests;

#[cfg(target_arch = "aarch64")]
use crate::arch::mm::setup_parange;
use crate::consts::MAX_CPU_NUM;
use arch::{cpu::cpu_start, entry::arch_entry};
use config::root_zone_config;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use fdt_rs::{base::DevTree, prelude::FallibleIterator};
use percpu::PerCpu;
use platform::BOARD_NCPUS;
use zone::{add_zone, zone_create};

#[cfg(all(feature = "iommu", target_arch = "aarch64"))]
use crate::arch::iommu::iommu_init;

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

fn primary_init_early(ncpu: usize) {
    extern "C" {
        fn __core_end();
    }
    logging::init();
    info!("Logging is enabled.");
    info!("__core_end = {:#x?}", __core_end as usize);
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
    event::init(ncpu);

    device::irqchip::primary_init_early();
    // crate::arch::mm::init_hv_page_table().unwrap();

    #[cfg(all(feature = "iommu", target_arch = "aarch64"))]
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
        warn!("zone is not created for cpu {}", cpu.id);
    }
    // unsafe {
    //     memory::hv_page_table().read().activate();
    // };
    info!("CPU {} hv_pt_install OK.", cpu.id);
}

fn wakeup_secondary_cpus(this_id: usize, host_dtb: usize, ncpu: usize) {
    for cpu_id in 0..ncpu {
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
        #[cfg(target_arch = "riscv64")]
        clear_bss();
        memory::heap::init();
        memory::heap::test();
    }

    let cpu = PerCpu::new(cpuid);

    println!(
        "Booting CPU {}: {:p} arch:{:p}, DTB: {:#x}",
        cpu.id, cpu as *const _, &cpu.arch_cpu as *const _, host_dtb
    );

    // Don't you wanna know how many cpu(s) on board? :D
    let mut ncpu: usize = 0;
    #[cfg(all(
        any(target_arch = "aarch64", target_arch = "riscv64"),
        not(feature = "no_autodetect_ncpus")
    ))]
    {
        let devtree = unsafe { DevTree::from_raw_pointer(host_dtb as *const u8).unwrap() };

        let mut node_iter = devtree.nodes();
        while let Some(node) = node_iter.next().unwrap() {
            if node.name().unwrap().starts_with("cpu@") {
                ncpu += 1;
            }
        }
    }

    // If we failed to detect, just use default value.
    if ncpu == 0 {
        if is_primary {
            println!(
                "Failed to count cpu(s) from devicetree. Using default value {}.",
                BOARD_NCPUS
            );
        }
        ncpu = BOARD_NCPUS;
    } else if ncpu > MAX_CPU_NUM {
        if is_primary {
            println!("{} cpu(s) detected, but using only {}.", ncpu, MAX_CPU_NUM);
        }
        ncpu = MAX_CPU_NUM;
    }

    if is_primary {
        #[cfg(target_arch = "aarch64")]
        {
            println!("Using {} cpu(s) on this system.", ncpu);
        }

        unsafe {
            consts::NCPU = ncpu;
        }
        wakeup_secondary_cpus(cpu.id, host_dtb, ncpu);
    }

    ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for(|| PerCpu::entered_cpus() < ncpu as _);
    assert_eq!(PerCpu::entered_cpus(), ncpu as _);

    println!(
        "{} CPU {} has entered.",
        if is_primary { "Primary" } else { "Secondary" },
        cpu.id
    );

    #[cfg(target_arch = "aarch64")]
    setup_parange(ncpu);

    if is_primary {
        primary_init_early(ncpu); // create root zone here
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1);
    }

    per_cpu_init(cpu);
    device::irqchip::percpu_init();

    INITED_CPUS.fetch_add(1, Ordering::SeqCst);

    wait_for_counter(&INITED_CPUS, ncpu as _);

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
