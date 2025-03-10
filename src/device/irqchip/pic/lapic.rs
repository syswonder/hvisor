use crate::{
    arch::{
        apic::current_time_nanos,
        cpu::ArchCpu,
        msr::Msr::{self, *},
    },
    error::HvResult,
};
use bit_field::BitField;

const APIC_FREQ_MHZ: u64 = 1000; // 1000 MHz
const APIC_CYCLE_NANOS: u64 = 1000 / APIC_FREQ_MHZ;

/// Local APIC timer modes.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerMode {
    /// Timer only fires once.
    OneShot = 0b00,
    /// Timer fires periodically.
    Periodic = 0b01,
    /// Timer fires at an absolute time.
    TscDeadline = 0b10,
}

/// A virtual local APIC timer. (SDM Vol. 3C, Section 10.5.4)
pub struct VirtApicTimer {
    is_enabled: u8,
    lvt_timer_bits: u32,
    divide_shift: u8,
    initial_count: u32,
    last_start_ns: u64,
    deadline_ns: u64,
}

impl VirtApicTimer {
    pub const fn new() -> Self {
        Self {
            is_enabled: 1,
            lvt_timer_bits: 0x1_0000, // masked
            divide_shift: 0,
            initial_count: 0,
            last_start_ns: 0,
            deadline_ns: 0,
        }
    }

    pub fn set_enable(&mut self, is_enabled: u8) {
        self.is_enabled = is_enabled;
    }

    /// Check if an interrupt generated. if yes, update it's states.
    pub fn check_interrupt(&mut self) -> bool {
        if self.deadline_ns == 0 {
            return false;
        } else if current_time_nanos() >= self.deadline_ns {
            if self.is_periodic() {
                self.deadline_ns += self.interval_ns();
            } else {
                self.deadline_ns = 0;
            }
            if self.is_enabled != 0 {
                return !self.is_masked();
            }
        }
        false
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
        let elapsed_ns = current_time_nanos() - self.last_start_ns;
        let elapsed_cycles = (elapsed_ns / APIC_CYCLE_NANOS) >> self.divide_shift;
        if self.is_periodic() {
            self.initial_count - (elapsed_cycles % self.initial_count as u64) as u32
        } else if elapsed_cycles < self.initial_count as u64 {
            self.initial_count - elapsed_cycles as u32
        } else {
            0
        }
    }

    /// Set LVT Timer Register.
    pub fn set_lvt_timer(&mut self, bits: u32) -> HvResult {
        let timer_mode = bits.get_bits(17..19);
        /*if timer_mode == TimerMode::TscDeadline as _ {
            return hv_result_err!(EINVAL); // TSC deadline mode was not supported
        } else */
        if timer_mode == 0b11 {
            return hv_result_err!(EINVAL); // reserved
        }
        self.lvt_timer_bits = bits;
        self.start_timer();
        Ok(())
    }

    /// Set Initial Count Register.
    pub fn set_initial_count(&mut self, initial: u32) -> HvResult {
        self.initial_count = initial;
        self.start_timer();
        Ok(())
    }

    /// Set Divide Configuration Register.
    pub fn set_divide(&mut self, dcr: u32) -> HvResult {
        let shift = (dcr & 0b11) | ((dcr & 0b1000) >> 1);
        self.divide_shift = (shift + 1) as u8 & 0b111;
        self.start_timer();
        Ok(())
    }

    const fn interval_ns(&self) -> u64 {
        (self.initial_count as u64 * APIC_CYCLE_NANOS) << self.divide_shift
    }

    fn start_timer(&mut self) {
        if self.initial_count != 0 {
            self.last_start_ns = current_time_nanos();
            self.deadline_ns = self.last_start_ns + self.interval_ns();
        } else {
            self.deadline_ns = 0;
        }
    }
}

pub struct VirtLocalApic;

impl VirtLocalApic {
    pub const fn msr_range() -> core::ops::Range<u32> {
        0x800..0x840
    }

    pub fn rdmsr(arch_cpu: &mut ArchCpu, msr: Msr) -> HvResult<u64> {
        let apic_timer = arch_cpu.apic_timer_mut();
        trace!("lapic rdmsr: {:?}", msr,);
        match msr {
            IA32_X2APIC_APICID => Ok(arch_cpu.cpuid as u64),
            IA32_X2APIC_VERSION => Ok(0x50014), // Max LVT Entry: 0x5, Version: 0x14
            IA32_X2APIC_LDR => Ok(0x0),         // TODO: IPI
            IA32_X2APIC_SIVR => Ok(((apic_timer.is_enabled as u64 & 0x1) << 8) | 0xff), // SDM Vol. 3A, Section 10.9, Figure 10-23 (with Software Enable bit)
            IA32_X2APIC_LVT_TIMER => Ok(apic_timer.lvt_timer() as u64),
            IA32_X2APIC_LVT_THERMAL
            | IA32_X2APIC_LVT_PMI
            | IA32_X2APIC_LVT_LINT0
            | IA32_X2APIC_LVT_LINT1
            | IA32_X2APIC_LVT_ERROR => {
                Ok(0x1_0000) // SDM Vol. 3A, Section 10.5.1, Figure 10-8 (with Mask bit)
            }
            IA32_X2APIC_INIT_COUNT => Ok(apic_timer.initial_count() as u64),
            IA32_X2APIC_CUR_COUNT => Ok(apic_timer.current_counter() as u64),
            IA32_X2APIC_DIV_CONF => Ok(apic_timer.divide() as u64),
            _ => hv_result_err!(ENOSYS),
        }
    }

    pub fn wrmsr(arch_cpu: &mut ArchCpu, msr: Msr, value: u64) -> HvResult {
        if msr != IA32_X2APIC_ICR && (value >> 32) != 0 {
            return hv_result_err!(EINVAL); // all registers except ICR are 32-bits
        }
        let apic_timer = arch_cpu.apic_timer_mut();
        trace!("lapic wrmsr: {:?}, value: {:x}", msr, value);
        match msr {
            IA32_X2APIC_EOI => {
                if value != 0 {
                    hv_result_err!(EINVAL) // write a non-zero value causes #GP
                } else {
                    Ok(())
                }
            }
            IA32_X2APIC_SIVR => {
                apic_timer.set_enable(((value >> 8) & 1) as _);
                Ok(())
            }
            IA32_X2APIC_LVT_THERMAL
            | IA32_X2APIC_LVT_PMI
            | IA32_X2APIC_LVT_LINT0
            | IA32_X2APIC_LVT_LINT1
            | IA32_X2APIC_LVT_ERROR => {
                Ok(()) // ignore these register writes
            }
            IA32_X2APIC_LVT_TIMER => apic_timer.set_lvt_timer(value as u32),
            IA32_X2APIC_INIT_COUNT => apic_timer.set_initial_count(value as u32),
            IA32_X2APIC_DIV_CONF => apic_timer.set_divide(value as u32),
            _ => hv_result_err!(ENOSYS),
        }
    }
}
