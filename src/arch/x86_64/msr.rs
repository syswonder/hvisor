use crate::{
    arch::msr::Msr::*,
    device::irqchip::pic::lapic::VirtLocalApic,
    error::HvResult,
    memory::{Frame, HostPhysAddr},
};
use x86::msr::{rdmsr, wrmsr};

numeric_enum_macro::numeric_enum! {
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
/// X86 model-specific registers. (SDM Vol. 4)
pub enum Msr {
    /// APIC Location and Status (R/W) See Table 35-2. See Section 10.4.4, Local APIC Status and Location.
    IA32_APIC_BASE = 0x1b,
    IA32_FEATURE_CONTROL = 0x3a,
    IA32_PAT = 0x277,

    IA32_VMX_BASIC = 0x480,
    IA32_VMX_PINBASED_CTLS = 0x481,
    IA32_VMX_PROCBASED_CTLS = 0x482,
    IA32_VMX_EXIT_CTLS = 0x483,
    IA32_VMX_ENTRY_CTLS = 0x484,
    IA32_VMX_MISC = 0x485,
    IA32_VMX_CR0_FIXED0 = 0x486,
    IA32_VMX_CR0_FIXED1 = 0x487,
    IA32_VMX_CR4_FIXED0 = 0x488,
    IA32_VMX_CR4_FIXED1 = 0x489,
    IA32_VMX_PROCBASED_CTLS2 = 0x48b,
    IA32_VMX_EPT_VPID_CAP = 0x48c,
    IA32_VMX_TRUE_PINBASED_CTLS = 0x48d,
    IA32_VMX_TRUE_PROCBASED_CTLS = 0x48e,
    IA32_VMX_TRUE_EXIT_CTLS = 0x48f,
    IA32_VMX_TRUE_ENTRY_CTLS = 0x490,

    /// X2APIC Msr

    /// TSC Target of Local APIC s TSC Deadline Mode (R/W)  See Table 35-2
    IA32_TSC_DEADLINE = 0x6e0,

    /// ID register.
    IA32_X2APIC_APICID = 0x802,
    /// Version register.
    IA32_X2APIC_VERSION = 0x803,
    /// End-Of-Interrupt register.
    IA32_X2APIC_EOI = 0x80B,
    /// Logical Destination Register.
    IA32_X2APIC_LDR = 0x80D,
    /// Spurious Interrupt Vector register.
    IA32_X2APIC_SIVR = 0x80F,

    /// In-Service register bits [31:0].
    IA32_X2APIC_ISR0 = 0x810,
    /// In-Service register bits [63:32].
    IA32_X2APIC_ISR1 = 0x811,
    /// In-Service register bits [95:64].
    IA32_X2APIC_ISR2 = 0x812,
    /// In-Service register bits [127:96].
    IA32_X2APIC_ISR3 = 0x813,
    /// In-Service register bits [159:128].
    IA32_X2APIC_ISR4 = 0x814,
    /// In-Service register bits [159:128].
    IA32_X2APIC_ISR5 = 0x815,
    /// In-Service register bits [191:160].
    IA32_X2APIC_ISR6 = 0x816,
    /// In-Service register bits [223:192].
    IA32_X2APIC_ISR7 = 0x817,

    /// Interrupt Request register bits [31:0].
    IA32_X2APIC_IRR0 = 0x820,
    /// Interrupt Request register bits [63:32].
    IA32_X2APIC_IRR1 = 0x821,
    /// Interrupt Request register bits [95:64].
    IA32_X2APIC_IRR2 = 0x822,
    /// Interrupt Request register bits [127:96].
    IA32_X2APIC_IRR3 = 0x823,
    /// Interrupt Request register bits [159:128].
    IA32_X2APIC_IRR4 = 0x824,
    /// Interrupt Request register bits [159:128].
    IA32_X2APIC_IRR5 = 0x825,
    /// Interrupt Request register bits [191:160].
    IA32_X2APIC_IRR6 = 0x826,
    /// Interrupt Request register bits [223:192].
    IA32_X2APIC_IRR7 = 0x827,

    /// Error Status register.
    IA32_X2APIC_ESR = 0x828,
    /// Interrupt Command register.
    IA32_X2APIC_ICR = 0x830,
    /// LVT Timer Interrupt register.
    IA32_X2APIC_LVT_TIMER = 0x832,
    /// LVT Thermal Sensor Interrupt register.
    IA32_X2APIC_LVT_THERMAL = 0x833,
    /// LVT Performance Monitor register.
    IA32_X2APIC_LVT_PMI = 0x834,
    /// LVT LINT0 register.
    IA32_X2APIC_LVT_LINT0 = 0x835,
    /// LVT LINT1 register.
    IA32_X2APIC_LVT_LINT1 = 0x836,
    /// LVT Error register.
    IA32_X2APIC_LVT_ERROR = 0x837,
    /// Initial Count register.
    IA32_X2APIC_INIT_COUNT = 0x838,
    /// Current Count register.
    IA32_X2APIC_CUR_COUNT = 0x839,
    /// Divide Configuration register.
    IA32_X2APIC_DIV_CONF = 0x83E,

    IA32_EFER = 0xc000_0080,
    IA32_STAR = 0xc000_0081,
    IA32_LSTAR = 0xc000_0082,
    IA32_CSTAR = 0xc000_0083,
    IA32_FMASK = 0xc000_0084,

    IA32_FS_BASE = 0xc000_0100,
    IA32_GS_BASE = 0xc000_0101,
    IA32_KERNEL_GSBASE = 0xc000_0102,
}
}

impl Msr {
    /// Read 64 bits msr register.
    #[inline(always)]
    pub fn read(self) -> u64 {
        unsafe { rdmsr(self as _) }
    }

    /// Write 64 bits to msr register.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this write operation has no unsafe side
    /// effects.
    #[inline(always)]
    pub unsafe fn write(self, value: u64) {
        wrmsr(self as _, value)
    }
}

#[derive(Debug)]
pub struct MsrBitmap {
    frame: Frame,
}

impl MsrBitmap {
    pub fn uninit() -> Self {
        Self {
            frame: unsafe { Frame::from_paddr(0) },
        }
    }

    pub fn passthrough_all() -> HvResult<Self> {
        Ok(Self {
            frame: Frame::new_zero()?,
        })
    }

    pub fn intercept_all() -> HvResult<Self> {
        let mut frame = Frame::new()?;
        frame.fill(u8::MAX);
        Ok(Self { frame })
    }

    pub fn intercept_def() -> HvResult<Self> {
        // Intercept IA32_APIC_BASE MSR accesses
        let mut bitmap = Self {
            frame: Frame::new_zero()?,
        };

        bitmap.set_read_intercept(IA32_APIC_BASE, true);
        bitmap.set_write_intercept(IA32_APIC_BASE, true);

        bitmap.set_read_intercept(IA32_TSC_DEADLINE, true);
        bitmap.set_write_intercept(IA32_TSC_DEADLINE, true);

        // Intercept all x2APIC MSR accesses
        for addr in VirtLocalApic::msr_range() {
            if let Ok(msr) = Msr::try_from(addr) {
                bitmap.set_read_intercept(msr, true);
                bitmap.set_write_intercept(msr, true);
            }
        }
        Ok(bitmap)
    }

    pub fn phys_addr(&self) -> HostPhysAddr {
        self.frame.start_paddr()
    }

    pub fn set_read_intercept(&mut self, msr: Msr, intercept: bool) {
        self.set_intercept(msr as u32, false, intercept);
    }

    pub fn set_write_intercept(&mut self, msr: Msr, intercept: bool) {
        self.set_intercept(msr as u32, true, intercept);
    }

    fn set_intercept(&mut self, msr: u32, is_write: bool, intercept: bool) {
        let offset = if msr <= 0x1fff {
            if !is_write {
                0 // Read bitmap for low MSRs (0x0000_0000..0x0000_1FFF)
            } else {
                2 // Write bitmap for low MSRs (0x0000_0000..0x0000_1FFF)
            }
        } else if (0xc000_0000..=0xc000_1fff).contains(&msr) {
            if !is_write {
                1 // Read bitmap for high MSRs (0xC000_0000..0xC000_1FFF)
            } else {
                3 // Write bitmap for high MSRs (0xC000_0000..0xC000_1FFF)
            }
        } else {
            unreachable!()
        } * 1024;
        let bitmap =
            unsafe { core::slice::from_raw_parts_mut(self.frame.as_mut_ptr().add(offset), 1024) };
        let msr = msr & 0x1fff;
        let byte = (msr / 8) as usize;
        let bits = msr % 8;
        if intercept {
            bitmap[byte] |= 1 << bits;
        } else {
            bitmap[byte] &= !(1 << bits);
        }
    }
}
