//! The main module and entrypoint
//!
//! Various facilities of hvisor are implemented as submodules. The most
//! important ones are:
//!
//! - [`memory`]: Memory management
//! - [`hypercall`]: Hypercall handling
//! - [`device`]: Device management
//! - [`arch`]: Architecture's related

#![no_std] // 禁用标准库链接
#![no_main]
// 不使用main入口，使用自己定义实际入口_start，因为我们还没有初始化堆栈指针
#![feature(asm_const)]
#![feature(naked_functions)] //  surpport naked function
#![feature(core_panic)]
// 支持内联汇编
// #![deny(warnings, missing_docs)] // 将warnings作为error
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
mod control;
mod device;
mod hypercall;
mod memory;
mod num;
mod panic;
mod percpu;
mod zone;

use crate::consts::nr1_config_ptr;
use crate::control::do_zone_create;
use crate::control::prepare_zone_start;
use crate::control::wait_for_poweron;
use crate::percpu::this_cpu_data;
use crate::percpu::this_zone;
use crate::{consts::MAX_CPU_NUM, zone::root_zone};
use arch::cpu::cpu_start;
use arch::entry::arch_entry;
use config::HvSystemConfig;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use error::HvResult;
use memory::addr::virt_to_phys;
use percpu::PerCpu;

#[cfg(target_arch = "aarch64")]
use device::gicv3::gicv3_cpu_init;

static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static ACTIVATED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);
static MASTER_CPU: AtomicI32 = AtomicI32::new(-1);

fn wait_for(condition: impl Fn() -> bool) {
    while condition() {
        core::hint::spin_loop();
    }
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32) {
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}

fn primary_init_early() -> HvResult {
    logging::init();
    info!("Logging is enabled.");

    // let system_config = HvSystemConfig::get();
    // let revision = system_config.revision;
    // info!(
    //     "\n\
    //     Initializing hypervisor...\n\
    //     config_signature = {:?}\n\
    //     config_revision = {}\n\
    //     build_mode = {}\n\
    //     log_level = {}\n\
    //     arch = {}\n\
    //     vendor = {}\n\
    //     stats = {}\n\
    //     ",
    //     core::str::from_utf8(&system_config.signature),
    //     revision,
    //     option_env!("MODE").unwrap_or(""),
    //     option_env!("LOG").unwrap_or(""),
    //     option_env!("ARCH").unwrap_or(""),
    //     option_env!("VENDOR").unwrap_or(""),
    //     option_env!("STATS").unwrap_or("off"),
    // );

    memory::heap::init();
    // system_config.check()?;

    // info!("System config: {:#x?}", system_config);

    memory::frame::init();
    memory::init_hv_page_table()?;
    todo!();
    // zone::init()?;

    // unsafe {
    //     // We should activate new hv-pt here in advance,
    //     // in case of triggering data aborts in `zone::init()`
    //     memory::hv_page_table().read().activate();
    // }

    // do_zone_create(unsafe { nr1_config_ptr().as_ref().unwrap() })?;

    INIT_EARLY_OK.store(1, Ordering::Release);
    Ok(())
}

fn primary_init_late() {
    info!("Primary CPU init late...");

    #[cfg(target_arch = "aarch64")]
    enable_gic_are_ns();

    INIT_LATE_OK.store(1, Ordering::Release);
}

fn per_cpu_init() {
    let cpu_data = this_cpu_data();

    if cpu_data.zone.is_none() {
        cpu_data.zone = Some(root_zone());
    }

    // gicv3_cpu_init();
    todo!();
    // unsafe {
    //     memory::hv_page_table().read().activate();
    //     this_zone().read().gpm_activate();
    // };

    // enable_ipi();
    // enable_irqs();

    println!("CPU {} init OK.", cpu_data.id);
}

fn wakeup_secondary_cpus(this_id: usize) {
    for cpu_id in 0..MAX_CPU_NUM {
        if cpu_id == this_id {
            continue;
        }
        cpu_start(cpu_id, arch_entry as _, 0);
    }
}

fn rust_main(cpuid: usize) {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
        fn __core_end(); // end of kernel
        fn gdtb();
        fn vmimg();
    }
    // println!("Hello, world!");
    // println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    // println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    // println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    // println!(
    //     "boot_stack top=bottom={:#x}, lower_bound={:#x}",
    //     boot_stack_top as usize, boot_stack_lower_bound as usize
    // );

    let cpu_data = PerCpu::new(cpuid);

    println!(
        "cpu_id = {}, &cpu_data = {:#x?}",
        cpu_data.id, cpu_data as *const _
    );

    let mut is_primary = false;
    if MASTER_CPU.load(Ordering::Acquire) == -1 {
        MASTER_CPU.store(cpuid as i32, Ordering::Release);
        is_primary = true;
    }

    if is_primary {
        // Set PHYS_VIRT_OFFSET early.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            addr::PHYS_VIRT_OFFSET =
                HV_BASE - HvSystemConfig::get().hypervisor_memory.phys_start as usize
        };
        wakeup_secondary_cpus(cpu_data.id);
    }

    wait_for(|| PerCpu::entered_cpus() < MAX_CPU_NUM as _);
    assert_eq!(PerCpu::entered_cpus(), MAX_CPU_NUM as _);

    println!(
        "{} CPU {} entered.",
        if is_primary { "Primary" } else { "Secondary" },
        cpu_data.id
    );

    if is_primary {
        primary_init_early(); // create root zone here
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1);
    }

    // per_cpu_init();

    // INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    // wait_for_counter(&INITED_CPUS, MAX_CPU_NUM as _)?;

    // if is_primary {
    //     primary_init_late();
    // } else {
    //     wait_for_counter(&INIT_LATE_OK, 1)?
    // }

    // cpu_data.activate_vmm();
    // wait_for_counter(&ACTIVATED_CPUS, MAX_CPU_NUM as _)?;

    // if cpu_data.id == 0 {
    //     prepare_zone_start(this_zone())?;
    //     cpu_data.start_zone();
    // } else {
    //     wait_for_poweron();
    // }
}
