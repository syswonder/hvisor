use self::vectors::*;
use crate::device::irqchip::i8259::enable_irq;
use core::time::Duration;
use raw_cpuid::CpuId;
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::instructions::port::Port;

type TimeValue = Duration;

pub mod vectors {
    pub const APIC_TIMER_VECTOR: u8 = 0xf0;
    pub const APIC_SPURIOUS_VECTOR: u8 = 0xf1;
    pub const APIC_ERROR_VECTOR: u8 = 0xf2;
}

static mut LOCAL_APIC: Option<LocalApic> = None;
static mut CPU_FREQ_MHZ: u64 = 4_000;
const LAPIC_TICKS_PER_SEC: u64 = 1_000_000_000; // TODO: need to calibrate
const TICKS_PER_SEC: u64 = 1;

pub fn local_apic<'a>() -> &'a mut LocalApic {
    // It's safe as LAPIC is per-cpu.
    unsafe { LOCAL_APIC.as_mut().unwrap() }
}

pub fn current_ticks() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * 1_000 / unsafe { CPU_FREQ_MHZ }
}

pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(ticks_to_nanos(current_ticks()))
}

pub fn busy_wait(duration: Duration) {
    busy_wait_until(current_time() + duration);
}

fn busy_wait_until(deadline: TimeValue) {
    while current_time() < deadline {
        core::hint::spin_loop();
    }
}

pub fn init_primary() {
    println!("Initializing Local APIC...");

    unsafe {
        // Disable 8259A interrupt controllers
        Port::<u8>::new(0x20).write(0xff);
        Port::<u8>::new(0xA0).write(0xff);
    }

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(APIC_TIMER_VECTOR as _)
        .error_vector(APIC_ERROR_VECTOR as _)
        .spurious_vector(APIC_SPURIOUS_VECTOR as _)
        .build()
        .unwrap();

    if let Some(freq) = CpuId::new()
        .get_processor_frequency_info()
        .map(|info| info.processor_max_frequency())
    {
        if freq > 0 {
            println!("Got TSC frequency by CPUID: {} MHz", freq);
            unsafe { CPU_FREQ_MHZ = freq as u64 }
        }
    }

    /*if let Some(sth) = CpuId::new().get_processor_brand_string() {
        println!("{:?}", sth);
    }*/

    unsafe {
        lapic.enable();
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_divide(TimerDivide::Div256);
        lapic.set_timer_initial((LAPIC_TICKS_PER_SEC / TICKS_PER_SEC) as u32);
    }

    unsafe {
        LOCAL_APIC = Some(lapic);
    }

    enable_irq();
}
