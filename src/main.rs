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
// #![feature(core_panic)]
// 支持内联汇编
// #![deny(warnings, missing_docs)] // 将warnings作为error

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
mod consts;
mod device;
mod event;
mod hypercall;
mod memory;
mod panic;
mod percpu;
mod platform;
mod zone;
mod config;

#[cfg(target_arch = "aarch64")]
mod ivc;

mod pci;
mod tests;

#[cfg(target_arch = "aarch64")]
use crate::arch::mm::setup_parange;
use crate::consts::MAX_CPU_NUM;
use arch::{cpu::cpu_start, entry::arch_entry};
use config::root_zone_config;
use zone::zone_create;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use percpu::PerCpu;

#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
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

fn primary_init_early() {
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
    event::init(MAX_CPU_NUM);

    device::irqchip::primary_init_early();
    // crate::arch::mm::init_hv_page_table().unwrap();

    #[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
    iommu_init();

    #[cfg(not(test))]
    zone_create(root_zone_config()).unwrap();
    
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
    println!("Hello, HVISOR!");
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

    #[cfg(target_arch = "aarch64")]
    setup_parange();
    
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
