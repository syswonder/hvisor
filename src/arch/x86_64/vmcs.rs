#![allow(non_camel_case_types)]
use crate::{
    arch::{
        msr::Msr,
        s2pt::Stage2PageFaultInfo,
        vmx::{
            VmxExitInfo, VmxInstructionError, VmxInterruptInfo, VmxInterruptionType, VmxIoExitInfo,
        },
    },
    error::{HvError, HvResult},
    memory::MemFlags,
};
use bit_field::BitField;
use x86::{
    bits64::vmx,
    vmx::{vmcs::control::PrimaryControls, Result as VmResult, VmFail},
};

macro_rules! vmcs_read {
    ($field_enum: ident, u64) => {
        impl $field_enum {
            pub fn read(self) -> x86::vmx::Result<u64> {
                #[cfg(target_pointer_width = "64")]
                unsafe {
                    vmx::vmread(self as u32)
                }
                #[cfg(target_pointer_width = "32")]
                unsafe {
                    let field = self as u32;
                    Ok(vmx::vmread(field)? + (vmx::vmread(field + 1)? << 32))
                }
            }
        }
    };
    ($field_enum: ident, $ux: ty) => {
        impl $field_enum {
            pub fn read(self) -> x86::vmx::Result<$ux> {
                unsafe { vmx::vmread(self as u32).map(|v| v as $ux) }
            }
        }
    };
}

macro_rules! vmcs_write {
    ($field_enum: ident, u64) => {
        impl $field_enum {
            pub fn write(self, value: u64) -> x86::vmx::Result<()> {
                #[cfg(target_pointer_width = "64")]
                unsafe {
                    vmx::vmwrite(self as u32, value)
                }
                #[cfg(target_pointer_width = "32")]
                unsafe {
                    let field = self as u32;
                    vmx::vmwrite(field, value & 0xffff_ffff)?;
                    vmx::vmwrite(field + 1, value >> 32)?;
                    Ok(())
                }
            }
        }
    };
    ($field_enum: ident, $ux: ty) => {
        impl $field_enum {
            pub fn write(self, value: $ux) -> x86::vmx::Result<()> {
                unsafe { vmx::vmwrite(self as u32, value as u64) }
            }
        }
    };
}

/// 16-Bit Control Fields. (SDM Vol. 3D, Appendix B.1.1)
#[derive(Clone, Copy, Debug)]
pub enum VmcsControl16 {
    /// Virtual-processor identifier (VPID).
    VPID = 0x0,
    /// Posted-interrupt notification vector.
    POSTED_INTERRUPT_NOTIFICATION_VECTOR = 0x2,
    /// EPTP index.
    EPTP_INDEX = 0x4,
}
vmcs_read!(VmcsControl16, u16);
vmcs_write!(VmcsControl16, u16);

/// 64-Bit Control Fields. (SDM Vol. 3D, Appendix B.2.1)
#[derive(Clone, Copy, Debug)]
pub enum VmcsControl64 {
    /// Address of I/O bitmap A (full).
    IO_BITMAP_A_ADDR = 0x2000,
    /// Address of I/O bitmap B (full).
    IO_BITMAP_B_ADDR = 0x2002,
    /// Address of MSR bitmaps (full).
    MSR_BITMAPS_ADDR = 0x2004,
    /// VM-exit MSR-store address (full).
    VMEXIT_MSR_STORE_ADDR = 0x2006,
    /// VM-exit MSR-load address (full).
    VMEXIT_MSR_LOAD_ADDR = 0x2008,
    /// VM-entry MSR-load address (full).
    VMENTRY_MSR_LOAD_ADDR = 0x200A,
    /// Executive-VMCS pointer (full).
    EXECUTIVE_VMCS_PTR = 0x200C,
    /// PML address (full).
    PML_ADDR = 0x200E,
    /// TSC offset (full).
    TSC_OFFSET = 0x2010,
    /// Virtual-APIC address (full).
    VIRT_APIC_ADDR = 0x2012,
    /// APIC-access address (full).
    APIC_ACCESS_ADDR = 0x2014,
    /// Posted-interrupt descriptor address (full).
    POSTED_INTERRUPT_DESC_ADDR = 0x2016,
    /// VM-function controls (full).
    VM_FUNCTION_CONTROLS = 0x2018,
    /// EPT pointer (full).
    EPTP = 0x201A,
    /// EOI-exit bitmap 0 (full).
    EOI_EXIT0 = 0x201C,
    /// EOI-exit bitmap 1 (full).
    EOI_EXIT1 = 0x201E,
    /// EOI-exit bitmap 2 (full).
    EOI_EXIT2 = 0x2020,
    /// EOI-exit bitmap 3 (full).
    EOI_EXIT3 = 0x2022,
    /// EPTP-list address (full).
    EPTP_LIST_ADDR = 0x2024,
    /// VMREAD-bitmap address (full).
    VMREAD_BITMAP_ADDR = 0x2026,
    /// VMWRITE-bitmap address (full).
    VMWRITE_BITMAP_ADDR = 0x2028,
    /// Virtualization-exception information address (full).
    VIRT_EXCEPTION_INFO_ADDR = 0x202A,
    /// XSS-exiting bitmap (full).
    XSS_EXITING_BITMAP = 0x202C,
    /// ENCLS-exiting bitmap (full).
    ENCLS_EXITING_BITMAP = 0x202E,
    /// Sub-page-permission-table pointer (full).
    SUBPAGE_PERM_TABLE_PTR = 0x2030,
    /// TSC multiplier (full).
    TSC_MULTIPLIER = 0x2032,
}
vmcs_read!(VmcsControl64, u64);
vmcs_write!(VmcsControl64, u64);

/// 32-Bit Control Fields. (SDM Vol. 3D, Appendix B.3.1)
#[derive(Clone, Copy, Debug)]
pub enum VmcsControl32 {
    /// Pin-based VM-execution controls.
    PINBASED_EXEC_CONTROLS = 0x4000,
    /// Primary processor-based VM-execution controls.
    PRIMARY_PROCBASED_EXEC_CONTROLS = 0x4002,
    /// Exception bitmap.
    EXCEPTION_BITMAP = 0x4004,
    /// Page-fault error-code mask.
    PAGE_FAULT_ERR_CODE_MASK = 0x4006,
    /// Page-fault error-code match.
    PAGE_FAULT_ERR_CODE_MATCH = 0x4008,
    /// CR3-target count.
    CR3_TARGET_COUNT = 0x400A,
    /// VM-exit controls.
    VMEXIT_CONTROLS = 0x400C,
    /// VM-exit MSR-store count.
    VMEXIT_MSR_STORE_COUNT = 0x400E,
    /// VM-exit MSR-load count.
    VMEXIT_MSR_LOAD_COUNT = 0x4010,
    /// VM-entry controls.
    VMENTRY_CONTROLS = 0x4012,
    /// VM-entry MSR-load count.
    VMENTRY_MSR_LOAD_COUNT = 0x4014,
    /// VM-entry interruption-information field.
    VMENTRY_INTERRUPTION_INFO_FIELD = 0x4016,
    /// VM-entry exception error code.
    VMENTRY_EXCEPTION_ERR_CODE = 0x4018,
    /// VM-entry instruction length.
    VMENTRY_INSTRUCTION_LEN = 0x401A,
    /// TPR threshold.
    TPR_THRESHOLD = 0x401C,
    /// Secondary processor-based VM-execution controls.
    SECONDARY_PROCBASED_EXEC_CONTROLS = 0x401E,
    /// PLE_Gap.
    PLE_GAP = 0x4020,
    /// PLE_Window.
    PLE_WINDOW = 0x4022,
}
vmcs_read!(VmcsControl32, u32);
vmcs_write!(VmcsControl32, u32);

/// Natural-Width Control Fields. (SDM Vol. 3D, Appendix B.4.1)
#[derive(Clone, Copy, Debug)]
pub enum VmcsControlNW {
    /// CR0 guest/host mask.
    CR0_GUEST_HOST_MASK = 0x6000,
    /// CR4 guest/host mask.
    CR4_GUEST_HOST_MASK = 0x6002,
    /// CR0 read shadow.
    CR0_READ_SHADOW = 0x6004,
    /// CR4 read shadow.
    CR4_READ_SHADOW = 0x6006,
    /// CR3-target value 0.
    CR3_TARGET_VALUE0 = 0x6008,
    /// CR3-target value 1.
    CR3_TARGET_VALUE1 = 0x600A,
    /// CR3-target value 2.
    CR3_TARGET_VALUE2 = 0x600C,
    /// CR3-target value 3.
    CR3_TARGET_VALUE3 = 0x600E,
}
vmcs_read!(VmcsControlNW, usize);
vmcs_write!(VmcsControlNW, usize);

/// 16-Bit Guest-State Fields. (SDM Vol. 3D, Appendix B.1.2)
pub enum VmcsGuest16 {
    /// Guest ES selector.
    ES_SELECTOR = 0x800,
    /// Guest CS selector.
    CS_SELECTOR = 0x802,
    /// Guest SS selector.
    SS_SELECTOR = 0x804,
    /// Guest DS selector.
    DS_SELECTOR = 0x806,
    /// Guest FS selector.
    FS_SELECTOR = 0x808,
    /// Guest GS selector.
    GS_SELECTOR = 0x80a,
    /// Guest LDTR selector.
    LDTR_SELECTOR = 0x80c,
    /// Guest TR selector.
    TR_SELECTOR = 0x80e,
    /// Guest interrupt status.
    INTERRUPT_STATUS = 0x810,
    /// PML index.
    PML_INDEX = 0x812,
}
vmcs_read!(VmcsGuest16, u16);
vmcs_write!(VmcsGuest16, u16);

/// 64-Bit Guest-State Fields. (SDM Vol. 3D, Appendix B.2.3)
#[derive(Clone, Copy, Debug)]
pub enum VmcsGuest64 {
    /// VMCS link pointer (full).
    LINK_PTR = 0x2800,
    /// Guest IA32_DEBUGCTL (full).
    IA32_DEBUGCTL = 0x2802,
    /// Guest IA32_PAT (full).
    IA32_PAT = 0x2804,
    /// Guest IA32_EFER (full).
    IA32_EFER = 0x2806,
    /// Guest IA32_PERF_GLOBAL_CTRL (full).
    IA32_PERF_GLOBAL_CTRL = 0x2808,
    /// Guest PDPTE0 (full).
    PDPTE0 = 0x280A,
    /// Guest PDPTE1 (full).
    PDPTE1 = 0x280C,
    /// Guest PDPTE2 (full).
    PDPTE2 = 0x280E,
    /// Guest PDPTE3 (full).
    PDPTE3 = 0x2810,
    /// Guest IA32_BNDCFGS (full).
    IA32_BNDCFGS = 0x2812,
    /// Guest IA32_RTIT_CTL (full).
    IA32_RTIT_CTL = 0x2814,
}
vmcs_read!(VmcsGuest64, u64);
vmcs_write!(VmcsGuest64, u64);

/// 32-Bit Guest-State Fields. (SDM Vol. 3D, Appendix B.3.3)
#[derive(Clone, Copy, Debug)]
pub enum VmcsGuest32 {
    /// Guest ES limit.
    ES_LIMIT = 0x4800,
    /// Guest CS limit.
    CS_LIMIT = 0x4802,
    /// Guest SS limit.
    SS_LIMIT = 0x4804,
    /// Guest DS limit.
    DS_LIMIT = 0x4806,
    /// Guest FS limit.
    FS_LIMIT = 0x4808,
    /// Guest GS limit.
    GS_LIMIT = 0x480A,
    /// Guest LDTR limit.
    LDTR_LIMIT = 0x480C,
    /// Guest TR limit.
    TR_LIMIT = 0x480E,
    /// Guest GDTR limit.
    GDTR_LIMIT = 0x4810,
    /// Guest IDTR limit.
    IDTR_LIMIT = 0x4812,
    /// Guest ES access rights.
    ES_ACCESS_RIGHTS = 0x4814,
    /// Guest CS access rights.
    CS_ACCESS_RIGHTS = 0x4816,
    /// Guest SS access rights.
    SS_ACCESS_RIGHTS = 0x4818,
    /// Guest DS access rights.
    DS_ACCESS_RIGHTS = 0x481A,
    /// Guest FS access rights.
    FS_ACCESS_RIGHTS = 0x481C,
    /// Guest GS access rights.
    GS_ACCESS_RIGHTS = 0x481E,
    /// Guest LDTR access rights.
    LDTR_ACCESS_RIGHTS = 0x4820,
    /// Guest TR access rights.
    TR_ACCESS_RIGHTS = 0x4822,
    /// Guest interruptibility state.
    INTERRUPTIBILITY_STATE = 0x4824,
    /// Guest activity state.
    ACTIVITY_STATE = 0x4826,
    /// Guest SMBASE.
    SMBASE = 0x4828,
    /// Guest IA32_SYSENTER_CS.
    IA32_SYSENTER_CS = 0x482A,
    /// VMX-preemption timer value.
    VMX_PREEMPTION_TIMER_VALUE = 0x482E,
}
vmcs_read!(VmcsGuest32, u32);
vmcs_write!(VmcsGuest32, u32);

/// Natural-Width Guest-State Fields. (SDM Vol. 3D, Appendix B.4.3)
#[derive(Clone, Copy, Debug)]
pub enum VmcsGuestNW {
    /// Guest CR0.
    CR0 = 0x6800,
    /// Guest CR3.
    CR3 = 0x6802,
    /// Guest CR4.
    CR4 = 0x6804,
    /// Guest ES base.
    ES_BASE = 0x6806,
    /// Guest CS base.
    CS_BASE = 0x6808,
    /// Guest SS base.
    SS_BASE = 0x680A,
    /// Guest DS base.
    DS_BASE = 0x680C,
    /// Guest FS base.
    FS_BASE = 0x680E,
    /// Guest GS base.
    GS_BASE = 0x6810,
    /// Guest LDTR base.
    LDTR_BASE = 0x6812,
    /// Guest TR base.
    TR_BASE = 0x6814,
    /// Guest GDTR base.
    GDTR_BASE = 0x6816,
    /// Guest IDTR base.
    IDTR_BASE = 0x6818,
    /// Guest DR7.
    DR7 = 0x681A,
    /// Guest RSP.
    RSP = 0x681C,
    /// Guest RIP.
    RIP = 0x681E,
    /// Guest RFLAGS.
    RFLAGS = 0x6820,
    /// Guest pending debug exceptions.
    PENDING_DBG_EXCEPTIONS = 0x6822,
    /// Guest IA32_SYSENTER_ESP.
    IA32_SYSENTER_ESP = 0x6824,
    /// Guest IA32_SYSENTER_EIP.
    IA32_SYSENTER_EIP = 0x6826,
}
vmcs_read!(VmcsGuestNW, usize);
vmcs_write!(VmcsGuestNW, usize);

/// 16-Bit Host-State Fields. (SDM Vol. 3D, Appendix B.1.3)
#[derive(Clone, Copy, Debug)]
pub enum VmcsHost16 {
    /// Host ES selector.
    ES_SELECTOR = 0xC00,
    /// Host CS selector.
    CS_SELECTOR = 0xC02,
    /// Host SS selector.
    SS_SELECTOR = 0xC04,
    /// Host DS selector.
    DS_SELECTOR = 0xC06,
    /// Host FS selector.
    FS_SELECTOR = 0xC08,
    /// Host GS selector.
    GS_SELECTOR = 0xC0A,
    /// Host TR selector.
    TR_SELECTOR = 0xC0C,
}
vmcs_read!(VmcsHost16, u16);
vmcs_write!(VmcsHost16, u16);

/// 64-Bit Host-State Fields. (SDM Vol. 3D, Appendix B.2.4)
#[derive(Clone, Copy, Debug)]
pub enum VmcsHost64 {
    /// Host IA32_PAT (full).
    IA32_PAT = 0x2C00,
    /// Host IA32_EFER (full).
    IA32_EFER = 0x2C02,
    /// Host IA32_PERF_GLOBAL_CTRL (full).
    IA32_PERF_GLOBAL_CTRL = 0x2C04,
}
vmcs_read!(VmcsHost64, u64);
vmcs_write!(VmcsHost64, u64);

/// 32-Bit Host-State Field. (SDM Vol. 3D, Appendix B.3.4)
#[derive(Clone, Copy, Debug)]
pub enum VmcsHost32 {
    /// Host IA32_SYSENTER_CS.
    IA32_SYSENTER_CS = 0x4C00,
}
vmcs_read!(VmcsHost32, u32);
vmcs_write!(VmcsHost32, u32);

/// Natural-Width Host-State Fields. (SDM Vol. 3D, Appendix B.4.4)
#[derive(Clone, Copy, Debug)]
pub enum VmcsHostNW {
    /// Host CR0.
    CR0 = 0x6C00,
    /// Host CR3.
    CR3 = 0x6C02,
    /// Host CR4.
    CR4 = 0x6C04,
    /// Host FS base.
    FS_BASE = 0x6C06,
    /// Host GS base.
    GS_BASE = 0x6C08,
    /// Host TR base.
    TR_BASE = 0x6C0A,
    /// Host GDTR base.
    GDTR_BASE = 0x6C0C,
    /// Host IDTR base.
    IDTR_BASE = 0x6C0E,
    /// Host IA32_SYSENTER_ESP.
    IA32_SYSENTER_ESP = 0x6C10,
    /// Host IA32_SYSENTER_EIP.
    IA32_SYSENTER_EIP = 0x6C12,
    /// Host RSP.
    RSP = 0x6C14,
    /// Host RIP.
    RIP = 0x6C16,
}
vmcs_read!(VmcsHostNW, usize);
vmcs_write!(VmcsHostNW, usize);

/// 64-Bit Read-Only Data Fields. (SDM Vol. 3D, Appendix B.2.2)
#[derive(Clone, Copy, Debug)]
pub enum VmcsReadOnly64 {
    /// Guest-physical address (full).
    GUEST_PHYSICAL_ADDR = 0x2400,
}
vmcs_read!(VmcsReadOnly64, u64);

/// 32-Bit Read-Only Data Fields. (SDM Vol. 3D, Appendix B.3.2)
#[derive(Clone, Copy, Debug)]
pub enum VmcsReadOnly32 {
    /// VM-instruction error.
    VM_INSTRUCTION_ERROR = 0x4400,
    /// Exit reason.
    EXIT_REASON = 0x4402,
    /// VM-exit interruption information.
    VMEXIT_INTERRUPTION_INFO = 0x4404,
    /// VM-exit interruption error code.
    VMEXIT_INTERRUPTION_ERR_CODE = 0x4406,
    /// IDT-vectoring information field.
    IDT_VECTORING_INFO = 0x4408,
    /// IDT-vectoring error code.
    IDT_VECTORING_ERR_CODE = 0x440A,
    /// VM-exit instruction length.
    VMEXIT_INSTRUCTION_LEN = 0x440C,
    /// VM-exit instruction information.
    VMEXIT_INSTRUCTION_INFO = 0x440E,
}
vmcs_read!(VmcsReadOnly32, u32);

/// Natural-Width Read-Only Data Fields. (SDM Vol. 3D, Appendix B.4.2)
#[derive(Clone, Copy, Debug)]
pub enum VmcsReadOnlyNW {
    /// Exit qualification.
    EXIT_QUALIFICATION = 0x6400,
    /// I/O RCX.
    IO_RCX = 0x6402,
    /// I/O RSI.
    IO_RSI = 0x6404,
    /// I/O RDI.
    IO_RDI = 0x6406,
    /// I/O RIP.
    IO_RIP = 0x6408,
    /// Guest-linear address.
    GUEST_LINEAR_ADDR = 0x640A,
}
vmcs_read!(VmcsReadOnlyNW, usize);

pub struct Vmcs;

impl Vmcs {
    pub fn load(paddr: usize) -> VmResult<()> {
        unsafe { vmx::vmptrld(paddr as _) }
    }

    pub fn clear(paddr: usize) -> VmResult<()> {
        unsafe { vmx::vmclear(paddr as _) }
    }

    /// Whether the guest interrupts are blocked. (SDM Vol. 3C, Section 24.4.2, Table 24-3)
    pub fn allow_interrupt() -> HvResult<bool> {
        let rflags = VmcsGuestNW::RFLAGS.read().unwrap();
        let block_state = VmcsGuest32::INTERRUPTIBILITY_STATE.read().unwrap();
        Ok(
            rflags as u64 & x86_64::registers::rflags::RFlags::INTERRUPT_FLAG.bits() != 0
                && block_state == 0,
        )
    }

    pub fn inject_interrupt(vector: u8, err_code: Option<u32>) -> HvResult {
        // SDM Vol. 3C, Section 24.8.3
        let err_code = if VmxInterruptionType::vector_has_error_code(vector) {
            err_code.or_else(|| Some(VmcsReadOnly32::VMEXIT_INTERRUPTION_ERR_CODE.read().unwrap()))
        } else {
            None
        };
        let int_info = VmxInterruptInfo::from(vector, err_code);
        if let Some(err_code) = int_info.err_code {
            VmcsControl32::VMENTRY_EXCEPTION_ERR_CODE.write(err_code)?;
        }
        if int_info.int_type.is_soft() {
            VmcsControl32::VMENTRY_INSTRUCTION_LEN
                .write(VmcsReadOnly32::VMEXIT_INSTRUCTION_LEN.read()?)?;
        }
        VmcsControl32::VMENTRY_INTERRUPTION_INFO_FIELD.write(int_info.bits())?;
        Ok(())
    }

    pub fn instruction_error() -> HvResult<VmxInstructionError> {
        Ok(VmcsReadOnly32::VM_INSTRUCTION_ERROR.read()?.into())
    }

    /// If enable, a VM exit occurs at the beginning of any instruction if
    /// `RFLAGS.IF` = 1 and there are no other blocking of interrupts.
    /// (see SDM, Vol. 3C, Section 24.4.2)
    pub fn set_interrupt_window(enable: bool) -> HvResult {
        let mut ctrl: u32 = VmcsControl32::PRIMARY_PROCBASED_EXEC_CONTROLS.read()?;
        let bits = PrimaryControls::INTERRUPT_WINDOW_EXITING.bits();
        if enable {
            ctrl |= bits
        } else {
            ctrl &= !bits
        }
        VmcsControl32::PRIMARY_PROCBASED_EXEC_CONTROLS.write(ctrl)?;
        Ok(())
    }

    pub fn set_control(
        control: VmcsControl32,
        capability_msr: Msr,
        old_value: u32,
        set: u32,
        clear: u32,
    ) -> HvResult {
        let cap = capability_msr.read();
        let allowed0 = cap as u32;
        let allowed1 = (cap >> 32) as u32;
        assert_eq!(allowed0 & allowed1, allowed0);
        debug!(
            "set {:?}: {:#x} (+{:#x}, -{:#x})",
            control, old_value, set, clear
        );
        if (set & clear) != 0 {
            return hv_result_err!(
                EINVAL,
                format!("can not set and clear the same bit in {:?}", control)
            );
        }
        if (allowed1 & set) != set {
            // failed if set 0-bits in allowed1
            warn!("allow1: {:x}", allowed1);
            return hv_result_err!(
                EINVAL,
                format!("can not set bits {:#x} in {:?}", set, control)
            );
        }
        if (allowed0 & clear) != 0 {
            // failed if clear 1-bits in allowed0
            return hv_result_err!(
                EINVAL,
                format!("can not clear bits {:#x} in {:?}", clear, control)
            );
        }
        // SDM Vol. 3C, Section 31.5.1, Algorithm 3
        let flexible = !allowed0 & allowed1; // therse bits can be either 0 or 1
        let unknown = flexible & !(set | clear); // hypervisor untouched bits
        let default = unknown & old_value; // these bits keep unchanged in old value
        let fixed1 = allowed0; // these bits are fixed to 1
        control.write(fixed1 | default | set)?;
        Ok(())
    }
}

impl From<VmFail> for HvError {
    fn from(err: VmFail) -> Self {
        match err {
            VmFail::VmFailValid => hv_err!(EFAULT, Vmcs::instruction_error().unwrap().as_str()),
            _ => hv_err!(EFAULT, format!("VMX instruction failed: {:?}", err)),
        }
    }
}
