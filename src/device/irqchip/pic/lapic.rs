use crate::{
    arch::{
        cpu::{this_cpu_id, ArchCpu},
        idt::IdtVector,
        ipi::{self, IpiDeliveryMode},
        msr::Msr::{self, *},
        vmcs::Vmcs,
    },
    device::irqchip::{inject_vector, pic::hpet},
    error::HvResult,
    percpu::{this_cpu_data, this_zone, CpuSet},
};
use alloc::collections::vec_deque::VecDeque;
use bit_field::BitField;
use core::{arch::x86_64::_rdtsc, u32};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};

const PHYS_LAPIC_TIMER_INTR_FREQ: u64 = 100;
const VIRT_LAPIC_TIMER_FREQ_MHZ: u64 = 1000; // 1000 MHz
const VIRT_LAPIC_TIMER_NANOS_PER_TICK: u64 = 1000 / VIRT_LAPIC_TIMER_FREQ_MHZ;

/// A virtual local APIC timer. (SDM Vol. 3C, Section 10.5.4)
pub struct VirtLocalApicTimer {
    is_enabled: bool,
    lvt_timer_bits: u32,
    divide_shift: u8,
    initial_count: u32,
    last_start_ns: u64,
    deadline_ns: u64,
    deadline_tsc: u64,
    timer_mode: TimerMode,
}

impl VirtLocalApicTimer {
    pub const fn new() -> Self {
        Self {
            is_enabled: true,
            lvt_timer_bits: 0x1_0000, // masked
            divide_shift: 0,
            initial_count: 0,
            last_start_ns: 0,
            deadline_ns: 0,
            deadline_tsc: 0,
            timer_mode: TimerMode::TscDeadline,
        }
    }

    pub fn set_enable(&mut self, is_enabled: u8) {
        self.is_enabled = is_enabled != 0;
    }

    /// Check if an interrupt generated. if yes, update it's states.
    pub fn check_interrupt(&mut self) -> bool {
        if !self.is_enabled || self.is_masked() {
            return false;
        }

        match self.timer_mode {
            TimerMode::OneShot => {
                if self.deadline_ns != 0 && hpet::current_time_nanos() >= self.deadline_ns {
                    self.deadline_ns = 0;
                    return true;
                }
            }
            TimerMode::Periodic => {
                let hpet_ns = hpet::current_time_nanos();
                if self.deadline_ns != 0 && hpet_ns >= self.deadline_ns {
                    self.deadline_ns += self.interval_ns();
                    return true;
                }
            }
            TimerMode::TscDeadline => {
                if (self.deadline_tsc != 0) && unsafe { _rdtsc() } >= self.deadline_tsc {
                    self.deadline_tsc = 0;
                    return true;
                }
            }
            _ => {}
        }
        return false;
    }

    /// Whether the timer interrupt is masked.
    pub const fn is_masked(&self) -> bool {
        self.lvt_timer_bits & (1 << 16) != 0
    }

    /// Whether the timer mode is periodic.
    pub const fn is_periodic(&self) -> bool {
        let timer_mode = (self.lvt_timer_bits >> 17) & 0b11;
        timer_mode == TimerMode::Periodic as _
    }

    /// The timer interrupt vector number.
    pub const fn vector(&self) -> u8 {
        (self.lvt_timer_bits & 0xff) as u8
    }

    /// LVT Timer Register. (SDM Vol. 3A, Section 10.5.1, Figure 10-8)
    pub const fn lvt_timer(&self) -> u32 {
        self.lvt_timer_bits
    }

    /// Divide Configuration Register. (SDM Vol. 3A, Section 10.5.4, Figure 10-10)
    pub const fn divide(&self) -> u32 {
        let dcr = self.divide_shift.wrapping_sub(1) as u32 & 0b111;
        (dcr & 0b11) | ((dcr & 0b100) << 1)
    }

    /// Initial Count Register.
    pub const fn initial_count(&self) -> u32 {
        self.initial_count
    }

    /// Current Count Register.
    pub fn current_counter(&self) -> u32 {
        let elapsed_ns = hpet::current_time_nanos() - self.last_start_ns;
        let elapsed_cycles = (elapsed_ns / VIRT_LAPIC_TIMER_NANOS_PER_TICK) >> self.divide_shift;

        match self.timer_mode {
            TimerMode::OneShot => {
                if elapsed_cycles < self.initial_count as u64 {
                    return self.initial_count - elapsed_cycles as u32;
                }
            }
            TimerMode::Periodic => {
                if self.initial_count != 0 {
                    return self.initial_count
                        - (elapsed_cycles % self.initial_count as u64) as u32;
                }
            }
            _ => {}
        }
        return 0;
    }

    /// Set LVT Timer Register.
    pub fn set_lvt_timer(&mut self, bits: u32) -> HvResult {
        let timer_mode = bits.get_bits(17..19);
        self.timer_mode = match timer_mode {
            0 => TimerMode::OneShot,
            1 => TimerMode::Periodic,
            _ => TimerMode::TscDeadline,
        };
        self.lvt_timer_bits = bits;
        self.set_deadline();
        Ok(())
    }

    /// Set Initial Count Register.
    pub fn set_initial_count(&mut self, initial: u32) -> HvResult {
        self.initial_count = initial;
        self.set_deadline();
        Ok(())
    }

    /// Set Divide Configuration Register.
    pub fn set_divide(&mut self, dcr: u32) -> HvResult {
        let shift = (dcr & 0b11) | ((dcr & 0b1000) >> 1);
        self.divide_shift = (shift + 1) as u8 & 0b111;
        self.set_deadline();
        Ok(())
    }

    pub fn set_tsc_deadline(&mut self, ddl: u64) -> HvResult {
        self.deadline_tsc = ddl;
        Ok(())
    }

    const fn interval_ns(&self) -> u64 {
        (self.initial_count as u64 * VIRT_LAPIC_TIMER_NANOS_PER_TICK) << self.divide_shift
    }

    fn set_deadline(&mut self) {
        if self.initial_count != 0 {
            self.last_start_ns = hpet::current_time_nanos();
            self.deadline_ns = self.last_start_ns + self.interval_ns();
        } else {
            self.deadline_ns = 0;
        }
    }
}

pub struct VirtLocalApic {
    pub phys_lapic: LocalApic,
    pub virt_lapic_timer: VirtLocalApicTimer,
}

impl VirtLocalApic {
    pub fn new() -> Self {
        let mut lapic = LocalApicBuilder::new()
            .timer_vector(IdtVector::APIC_TIMER_VECTOR as _)
            .error_vector(IdtVector::APIC_ERROR_VECTOR as _)
            .spurious_vector(IdtVector::APIC_SPURIOUS_VECTOR as _)
            .build()
            .unwrap();

        unsafe { lapic.enable() };

        // calibrate phys lapic timer
        let mut best_freq_hz = 0;
        for _ in 0..5 {
            unsafe { lapic.set_timer_initial(u32::MAX) };
            let hpet_start = hpet::current_ticks();
            hpet::wait_millis(10);
            let ticks = u32::MAX - unsafe { lapic.timer_current() };
            let hpet_end = hpet::current_ticks();

            let nanos = hpet::ticks_to_nanos(hpet_end.wrapping_sub(hpet_start));
            let ticks_per_sec = (ticks as u64 * 1_000_000_000 / nanos) as u32;

            if ticks_per_sec > best_freq_hz {
                best_freq_hz = ticks_per_sec;
            }
        }
        println!(
            "Calibrated LAPIC frequency: {}.{:03} MHz",
            best_freq_hz / 1_000_000,
            best_freq_hz % 1_000_000 / 1_000,
        );

        unsafe {
            lapic.set_timer_mode(TimerMode::Periodic);
            lapic.set_timer_divide(TimerDivide::Div256);
            lapic.set_timer_initial((best_freq_hz as u64 / PHYS_LAPIC_TIMER_INTR_FREQ) as u32);
        }

        Self {
            phys_lapic: lapic,
            virt_lapic_timer: VirtLocalApicTimer::new(),
        }
    }

    pub const fn msr_range() -> core::ops::Range<u32> {
        0x800..0x840
    }

    pub fn phys_local_apic<'a>() -> &'a mut LocalApic {
        &mut this_cpu_data().arch_cpu.virt_lapic.phys_lapic
    }

    pub fn rdmsr(&mut self, msr: Msr) -> HvResult<u64> {
        if msr != IA32_X2APIC_CUR_COUNT {
            // info!("lapic rdmsr: {:?}", msr,);
        }

        match msr {
            IA32_X2APIC_APICID => Ok(this_cpu_id() as u64),
            IA32_X2APIC_VERSION => Ok(0x50014), // Max LVT Entry: 0x5, Version: 0x14
            IA32_X2APIC_LDR => Ok(this_cpu_id() as u64), // logical apic id
            IA32_X2APIC_SIVR => Ok(((self.virt_lapic_timer.is_enabled as u64 & 0x1) << 8) | 0xff), // SDM Vol. 3A, Section 10.9, Figure 10-23 (with Software Enable bit)
            IA32_X2APIC_ISR0 | IA32_X2APIC_ISR1 | IA32_X2APIC_ISR2 | IA32_X2APIC_ISR3
            | IA32_X2APIC_ISR4 | IA32_X2APIC_ISR5 | IA32_X2APIC_ISR6 | IA32_X2APIC_ISR7 => {
                // info!("read ISR");
                Ok(0x0)
            }
            IA32_X2APIC_IRR0 | IA32_X2APIC_IRR1 | IA32_X2APIC_IRR2 | IA32_X2APIC_IRR3
            | IA32_X2APIC_IRR4 | IA32_X2APIC_IRR5 | IA32_X2APIC_IRR6 | IA32_X2APIC_IRR7 => {
                // info!("read IRR");
                Ok(0x0)
            }
            IA32_X2APIC_ESR => Ok(0x0),
            IA32_X2APIC_LVT_TIMER => Ok(self.virt_lapic_timer.lvt_timer() as u64),
            IA32_X2APIC_LVT_THERMAL
            | IA32_X2APIC_LVT_PMI
            | IA32_X2APIC_LVT_LINT0
            | IA32_X2APIC_LVT_LINT1
            | IA32_X2APIC_LVT_ERROR => {
                Ok(0x1_0000) // SDM Vol. 3A, Section 10.5.1, Figure 10-8 (with Mask bit)
            }
            IA32_X2APIC_INIT_COUNT => Ok(self.virt_lapic_timer.initial_count() as u64),
            IA32_X2APIC_CUR_COUNT => Ok(self.virt_lapic_timer.current_counter() as u64),
            IA32_X2APIC_DIV_CONF => Ok(self.virt_lapic_timer.divide() as u64),
            _ => hv_result_err!(ENOSYS),
        }
    }

    pub fn wrmsr(&mut self, msr: Msr, value: u64) -> HvResult {
        if (msr != IA32_X2APIC_ICR && msr != IA32_TSC_DEADLINE) && (value >> 32) != 0 {
            return hv_result_err!(EINVAL); // all registers except ICR are 32-bits
        }
        if msr == IA32_TSC_DEADLINE {
            self.virt_lapic_timer.set_tsc_deadline(value);
            return Ok(());
        }

        if msr == IA32_X2APIC_INIT_COUNT {
            //info!("{:?}, value: {:x}", msr, value);
        }
        match msr {
            IA32_X2APIC_EOI => {
                if value != 0 {
                    hv_result_err!(EINVAL) // write a non-zero value causes #GP
                } else {
                    Ok(())
                }
            }
            IA32_X2APIC_SIVR => {
                self.virt_lapic_timer.set_enable(((value >> 8) & 1) as _);
                Ok(())
            }
            IA32_X2APIC_ICR => {
                // info!("ICR value: {:x}", value);
                ipi::send_ipi(value);
                Ok(())
            }
            IA32_X2APIC_ESR
            | IA32_X2APIC_LVT_THERMAL
            | IA32_X2APIC_LVT_PMI
            | IA32_X2APIC_LVT_LINT0
            | IA32_X2APIC_LVT_LINT1
            | IA32_X2APIC_LVT_ERROR => {
                Ok(()) // ignore these register writes
            }
            IA32_X2APIC_LVT_TIMER => self.virt_lapic_timer.set_lvt_timer(value as u32),
            IA32_X2APIC_INIT_COUNT => self.virt_lapic_timer.set_initial_count(value as u32),
            IA32_X2APIC_DIV_CONF => self.virt_lapic_timer.set_divide(value as u32),
            _ => hv_result_err!(ENOSYS),
        }
    }

    pub fn check_timer_interrupt(&mut self) {
        if self.virt_lapic_timer.check_interrupt() {
            inject_vector(this_cpu_id(), self.virt_lapic_timer.vector(), None, false);
        }
    }
}
