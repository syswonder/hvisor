#![no_std] // 禁用标准库链接
#![no_main]
// 不使用main入口，使用自己定义实际入口_start，因为我们还没有初始化堆栈指针
#![feature(asm_const)]
#![feature(naked_functions)] //  surpport naked function
// 支持内联汇编
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

//#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;
mod cell;
mod config;
mod consts;
mod device;
mod header;
mod hypercall;
mod memory;
mod panic;
mod percpu;

use crate::cell::root_cell;
use crate::percpu::this_cpu_data;
use config::HvSystemConfig;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use device::gicv3::gicv3_cpu_init;
use error::HvResult;
use header::HvHeader;
use percpu::PerCpu;
static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);
static ERROR_NUM: AtomicI32 = AtomicI32::new(0);
fn has_err() -> bool {
    ERROR_NUM.load(Ordering::Acquire) != 0
}

fn wait_for(condition: impl Fn() -> bool) -> HvResult {
    while !has_err() && condition() {
        core::hint::spin_loop();
    }
    if has_err() {
        hv_result_err!(EBUSY, "Other cpu init failed!")
    } else {
        Ok(())
    }
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32) -> HvResult {
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}
fn primary_init_early() -> HvResult {
    logging::init();
    info!("Logging is enabled.");

    let system_config = HvSystemConfig::get();
    let revision = system_config.revision;
    info!(
        "\n\
        Initializing hypervisor...\n\
        config_signature = {:?}\n\
        config_revision = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        arch = {}\n\
        vendor = {}\n\
        stats = {}\n\
        ",
        core::str::from_utf8(&system_config.signature),
        revision,
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
        option_env!("ARCH").unwrap_or(""),
        option_env!("VENDOR").unwrap_or(""),
        option_env!("STATS").unwrap_or("off"),
    );

    memory::init_heap();
    system_config.check()?;
    info!("Hypervisor header: {:#x?}", HvHeader::get());
    info!("System config: {:#x?}", system_config);

    memory::init_frame_allocator();
    cell::init()?;

    INIT_EARLY_OK.store(1, Ordering::Release);
    Ok(())
}

fn primary_init_late() {
    info!("Primary CPU init late...");
    // Do nothing...
    INIT_LATE_OK.store(1, Ordering::Release);
}

fn main(cpu_data: &mut PerCpu) -> HvResult {
    println!("Hello");
    println!(
        "cpuid{} vaddr{:#x?} phyid{} &cpu_data{:#x?}",
        cpu_data.id,
        cpu_data.self_vaddr,
        this_cpu_data().id,
        cpu_data as *const _
    );
    let is_primary = cpu_data.id == 0;
    let online_cpus = HvHeader::get().online_cpus;
    wait_for(|| PerCpu::entered_cpus() < online_cpus)?;
    println!(
        "{} CPU {} entered.",
        if is_primary { "Primary" } else { "Secondary" },
        cpu_data.id
    );

    if is_primary {
        primary_init_early()?;
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1)?
    }

    unsafe { root_cell().gpm.activate() };
    println!("CPU {} init OK.", cpu_data.id);
    INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for_counter(&INITED_CPUS, online_cpus)?;

    if is_primary {
        primary_init_late();
    } else {
        wait_for_counter(&INIT_LATE_OK, 1)?
    }
    gicv3_cpu_init();
    cpu_data.activate_vmm()
}
extern "C" fn entry(cpu_data: &mut PerCpu) -> () {
    if let Err(_e) = main(cpu_data) {}
}
