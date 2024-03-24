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
mod platform;

use crate::zone::zone_create;
use crate::{config::DTB_ADDR, platform::qemu_aarch64::TENANTS};
use crate::device::irqchip::gicv3::gicd::enable_gic_are_ns;
use crate::consts::MAX_CPU_NUM;
use arch::{cpu::cpu_start, entry::arch_entry};
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use percpu::PerCpu;

static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);
static ACTIVATED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);
static MASTER_CPU: AtomicI32 = AtomicI32::new(-1);

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

fn wait_for(condition: impl Fn() -> bool) {
    while condition() {
        core::hint::spin_loop();
    }
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32) {
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}

fn primary_init_early(dtb: usize) {
    logging::init();
    info!("Logging is enabled.");
    // let system_config = HvSystemConfig::get();
    // let revision = system_config.revision;
    info!(
        "\n\
        Initializing hypervisor...\n\
        build_mode = {}\n\
        log_level = {}\n\
        arch = {}\n\
        vendor = {}\n\
        stats = {}",
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
        option_env!("ARCH").unwrap_or(""),
        option_env!("VENDOR").unwrap_or(""),
        option_env!("STATS").unwrap_or("off"),
    );

    memory::heap::init();
    memory::heap::heap_test();

    // system_config.check()?;

    // info!("System config: {:#x?}", system_config);

    memory::frame::init_frame_allocator();
    memory::frame::frame_allocator_test();

    info!("host dtb: {:#x}", dtb);
    let host_fdt = unsafe { fdt::Fdt::from_ptr(dtb as *const u8) }.unwrap();

    crate::arch::mm::init_hv_page_table(&host_fdt).unwrap();
    device::irqchip::irqchip_init(&host_fdt);

    for zone_id in 0..TENANTS.len() {
        info!(
            "guest{} addr: {:#x}, dtb addr: {:#x}",
            zone_id,
            TENANTS[zone_id].0.as_ptr() as usize,
            TENANTS[zone_id].1.as_ptr() as usize
        );
        let vm_paddr_start: usize = TENANTS[zone_id].0.as_ptr() as usize;
        zone_create(zone_id, vm_paddr_start, TENANTS[zone_id].1.as_ptr(), DTB_ADDR);
    }

    // // unsafe {
    // //     // We should activate new hv-pt here in advance,
    // //     // in case of triggering data aborts in `zone::init()`
    // //     memory::hv_page_table().read().activate();
    // // }

    // // do_zone_create(unsafe { nr1_config_ptr().as_ref().unwrap() })?;

    // INIT_EARLY_OK.store(1, Ordering::Release);
    loop {}
}

fn primary_init_late() {
    info!("Primary CPU init late...");

    #[cfg(target_arch = "aarch64")]
    enable_gic_are_ns();

    INIT_LATE_OK.store(1, Ordering::Release);
}

fn per_cpu_init(cpu: &mut PerCpu) {
    if cpu.zone.is_none() {
        warn!("zone is not created for cpu {}", cpu.id);
    } else {
        unsafe {
            memory::hv_page_table().read().activate();
            cpu.zone.clone().unwrap().read().gpm_activate();
        };
    }

    println!("CPU {} init OK.", cpu.id);
}

// fn per_cpu_init() {
//     let cpu_data = this_cpu_data();

//     if cpu_data.zone.is_none() {
//         cpu_data.zone = Some(root_zone());
//     }

//     // gicv3_cpu_init();
//     todo!();
//     // unsafe {
//     //     memory::hv_page_table().read().activate();
//     //     this_zone().read().gpm_activate();
//     // };

//     // enable_ipi();
//     // enable_irqs();

//     println!("CPU {} init OK.", cpu_data.id);
// }

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
    // println!("Hello, world!");
    // println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    // println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    // println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    // println!(
    //     "boot_stack top=bottom={:#x}, lower_bound={:#x}",
    //     boot_stack_top as usize, boot_stack_lower_bound as usize
    // );

    let mut is_primary = false;
    if MASTER_CPU.load(Ordering::Acquire) == -1 {
        MASTER_CPU.store(cpuid as i32, Ordering::Release);
        is_primary = true;
        #[cfg(target_arch = "riscv64")]
        clear_bss();
    }

    let cpu = PerCpu::new(cpuid);

    println!(
        "Hello from CPU {}, &cpu_data = {:#x?}, &dtb = {:#x}!",
        cpu.id, cpu as *const _, host_dtb
    );

    if is_primary {
        wakeup_secondary_cpus(cpu.id, host_dtb);
    }

    ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for(|| PerCpu::entered_cpus() < MAX_CPU_NUM as _);
    assert_eq!(PerCpu::entered_cpus(), MAX_CPU_NUM as _);

    println!(
        "{} CPU {} entered.",
        if is_primary { "Primary  " } else { "Secondary" },
        cpu.id
    );

    if is_primary {
        primary_init_early(host_dtb); // create root zone here
    } else {
        wait_for_counter(&INIT_EARLY_OK, 1);
    }

    per_cpu_init(cpu);

    INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    wait_for_counter(&INITED_CPUS, MAX_CPU_NUM as _);
    cpu.cpu_init(DTB_ADDR);

    if is_primary {
        primary_init_late();
    } else {
        wait_for_counter(&INIT_LATE_OK, 1);
    }

    cpu.run_vm();

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
