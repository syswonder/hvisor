use crate::{
    arch::{
        cpu::ArchCpu,
        msr::Msr,
        s2pt::Stage2PageFaultInfo,
        vmcs::{self, *},
    },
    consts::PAGE_SIZE,
    error::{HvError, HvResult},
    memory::{Frame, GuestPhysAddr, HostPhysAddr, MemFlags, PhysAddr},
};
use bit_field::BitField;
use bitflags::{bitflags, Flags};
use core::fmt::{Debug, Formatter, Result};
use raw_cpuid::CpuId;
use x86::{
    bits64::vmx,
    dtables,
    dtables::DescriptorTablePointer,
    segmentation::SegmentSelector,
    vmx::{vmcs::control::*, vmcs::*, VmFail},
};
use x86_64::{
    registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags},
    structures::gdt,
};

bitflags! {
    pub struct FeatureControlFlags: u64 {
        // Lock bit: when set, locks this MSR from being written. when clear,
        // VMXON causes a #GP.
        const LOCKED = 1 << 0;
        // Enable VMX inside SMX operation.
        const VMXON_ENABLED_INSIDE_SMX = 1 << 1;
        // Enable VMX outside SMX operation.
        const VMXON_ENABLED_OUTSIDE_SMX = 1 << 2;
    }
}

/// Exit Qualification for CR access. (SDM Vol. 3C, Section 27.2.1, Table 27-5)
#[derive(Debug)]
pub struct VmxCrAccessInfo {
    /// Control register number (CR0/CR3/CR4).
    pub cr_n: u8,
    /// Access type (0 = MOV to CR; 1 = MOV from CR; 2 = CLTS; 3 = LMSW).
    pub access_type: u8,
    /// LMSW operand type.
    pub lmsw_op_type: u8,
    /// General register.
    pub gpr: u8,
    /// LMSW source.
    pub lmsw_src: u16,
}

impl VmxCrAccessInfo {
    pub fn new() -> HvResult<Self> {
        let qualification = VmcsReadOnlyNW::EXIT_QUALIFICATION.read()?;
        Ok(VmxCrAccessInfo {
            cr_n: qualification.get_bits(0..=3) as _,
            access_type: qualification.get_bits(4..=5) as _,
            lmsw_op_type: qualification.get_bit(6) as _,
            gpr: qualification.get_bits(8..=11) as _,
            lmsw_src: qualification.get_bits(16..=31) as _,
        })
    }
}

/// VM-Exit Informations. (SDM Vol. 3C, Section 24.9.1)
#[derive(Debug)]
pub struct VmxExitInfo {
    /// VM-entry failure. (0 = true VM exit; 1 = VM-entry failure)
    pub entry_failure: bool,
    /// Basic exit reason.
    pub exit_reason: VmxExitReason,
    /// For VM exits resulting from instruction execution, this field receives
    /// the length in bytes of the instruction whose execution led to the VM exit.
    pub exit_instruction_length: u32,
    /// Guest `RIP` where the VM exit occurs.
    pub guest_rip: usize,
}

impl VmxExitInfo {
    pub fn new() -> HvResult<Self> {
        let full_reason = VmcsReadOnly32::EXIT_REASON.read()?;
        Ok(Self {
            exit_reason: full_reason
                .get_bits(0..16)
                .try_into()
                .expect("Unknown VM-exit reason"),
            entry_failure: full_reason.get_bit(31),
            exit_instruction_length: VmcsReadOnly32::VMEXIT_INSTRUCTION_LEN.read()?,
            guest_rip: VmcsGuestNW::RIP.read()?,
        })
    }
}

numeric_enum_macro::numeric_enum! {
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
/// VMX basic exit reasons. (SDM Vol. 3D, Appendix C)
pub enum VmxExitReason {
    EXCEPTION_NMI = 0,
    EXTERNAL_INTERRUPT = 1,
    TRIPLE_FAULT = 2,
    INIT = 3,
    SIPI = 4,
    SMI = 5,
    OTHER_SMI = 6,
    INTERRUPT_WINDOW = 7,
    NMI_WINDOW = 8,
    TASK_SWITCH = 9,
    CPUID = 10,
    GETSEC = 11,
    HLT = 12,
    INVD = 13,
    INVLPG = 14,
    RDPMC = 15,
    RDTSC = 16,
    RSM = 17,
    VMCALL = 18,
    VMCLEAR = 19,
    VMLAUNCH = 20,
    VMPTRLD = 21,
    VMPTRST = 22,
    VMREAD = 23,
    VMRESUME = 24,
    VMWRITE = 25,
    VMOFF = 26,
    VMON = 27,
    CR_ACCESS = 28,
    DR_ACCESS = 29,
    IO_INSTRUCTION = 30,
    MSR_READ = 31,
    MSR_WRITE = 32,
    INVALID_GUEST_STATE = 33,
    MSR_LOAD_FAIL = 34,
    MWAIT_INSTRUCTION = 36,
    MONITOR_TRAP_FLAG = 37,
    MONITOR_INSTRUCTION = 39,
    PAUSE_INSTRUCTION = 40,
    MCE_DURING_VMENTRY = 41,
    TPR_BELOW_THRESHOLD = 43,
    APIC_ACCESS = 44,
    VIRTUALIZED_EOI = 45,
    GDTR_IDTR = 46,
    LDTR_TR = 47,
    EPT_VIOLATION = 48,
    EPT_MISCONFIG = 49,
    INVEPT = 50,
    RDTSCP = 51,
    PREEMPTION_TIMER = 52,
    INVVPID = 53,
    WBINVD = 54,
    XSETBV = 55,
    APIC_WRITE = 56,
    RDRAND = 57,
    INVPCID = 58,
    VMFUNC = 59,
    ENCLS = 60,
    RDSEED = 61,
    PML_FULL = 62,
    XSAVES = 63,
    XRSTORS = 64,
    PCONFIG = 65,
    SPP_EVENT = 66,
    UMWAIT = 67,
    TPAUSE = 68,
    LOADIWKEY = 69,
}
}

/// VM instruction error numbers. (SDM Vol. 3C, Section 30.4)
pub struct VmxInstructionError(u32);

impl VmxInstructionError {
    pub fn as_str(&self) -> &str {
        match self.0 {
            0 => "OK",
            1 => "VMCALL executed in VMX root operation",
            2 => "VMCLEAR with invalid physical address",
            3 => "VMCLEAR with VMXON pointer",
            4 => "VMLAUNCH with non-clear VMCS",
            5 => "VMRESUME with non-launched VMCS",
            6 => "VMRESUME after VMXOFF (VMXOFF and VMXON between VMLAUNCH and VMRESUME)",
            7 => "VM entry with invalid control field(s)",
            8 => "VM entry with invalid host-state field(s)",
            9 => "VMPTRLD with invalid physical address",
            10 => "VMPTRLD with VMXON pointer",
            11 => "VMPTRLD with incorrect VMCS revision identifier",
            12 => "VMREAD/VMWRITE from/to unsupported VMCS component",
            13 => "VMWRITE to read-only VMCS component",
            15 => "VMXON executed in VMX root operation",
            16 => "VM entry with invalid executive-VMCS pointer",
            17 => "VM entry with non-launched executive VMCS",
            18 => "VM entry with executive-VMCS pointer not VMXON pointer (when attempting to deactivate the dual-monitor treatment of SMIs and SMM)",
            19 => "VMCALL with non-clear VMCS (when attempting to activate the dual-monitor treatment of SMIs and SMM)",
            20 => "VMCALL with invalid VM-exit control fields",
            22 => "VMCALL with incorrect MSEG revision identifier (when attempting to activate the dual-monitor treatment of SMIs and SMM)",
            23 => "VMXOFF under dual-monitor treatment of SMIs and SMM",
            24 => "VMCALL with invalid SMM-monitor features (when attempting to activate the dual-monitor treatment of SMIs and SMM)",
            25 => "VM entry with invalid VM-execution control fields in executive VMCS (when attempting to return from SMM)",
            26 => "VM entry with events blocked by MOV SS",
            28 => "Invalid operand to INVEPT/INVVPID",
            _ => "[INVALID]",
        }
    }
}

impl From<u32> for VmxInstructionError {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Debug for VmxInstructionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "VmxInstructionError({}, {:?})", self.0, self.as_str())
    }
}

/// VM-Entry / VM-Exit Interruption-Information Field. (SDM Vol. 3C, Section 24.8.3, 24.9.2)
#[derive(Debug)]
pub struct VmxInterruptInfo {
    /// Vector of interrupt or exception.
    pub vector: u8,
    /// Determines details of how the injection is performed.
    pub int_type: VmxInterruptionType,
    /// For hardware exceptions that would have delivered an error code on the stack.
    pub err_code: Option<u32>,
    /// Whether the field is valid.
    pub valid: bool,
}

impl VmxInterruptInfo {
    pub fn new() -> HvResult<Self> {
        // SDM Vol. 3C, Section 24.9.2
        let info = VmcsReadOnly32::VMEXIT_INTERRUPTION_INFO.read()?;
        Ok(VmxInterruptInfo {
            vector: info.get_bits(0..8) as u8,
            int_type: VmxInterruptionType::try_from(info.get_bits(8..11) as u8).unwrap(),
            err_code: if info.get_bit(11) {
                Some(VmcsReadOnly32::VMEXIT_INTERRUPTION_ERR_CODE.read()?)
            } else {
                None
            },
            valid: info.get_bit(31),
        })
    }

    /// Convert from the interrupt vector and the error code.
    pub fn from(vector: u8, err_code: Option<u32>) -> Self {
        Self {
            vector,
            int_type: VmxInterruptionType::from_vector(vector),
            err_code,
            valid: true,
        }
    }

    /// Raw bits for writing to VMCS.
    pub fn bits(&self) -> u32 {
        let mut bits = self.vector as u32;
        bits |= (self.int_type as u32) << 8;
        bits.set_bit(11, self.err_code.is_some());
        bits.set_bit(31, self.valid);
        bits
    }
}

numeric_enum_macro::numeric_enum! {
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// The interruption type (bits 10:8) in VM-Entry Interruption-Information Field
/// and VM-Exit Interruption-Information Field. (SDM Vol. 3C, Section 24.8.3, 24.9.2)
pub enum VmxInterruptionType {
    /// External interrupt
    External = 0,
    /// Reserved
    Reserved = 1,
    /// Non-maskable interrupt (NMI)
    NMI = 2,
    /// Hardware exception (e.g,. #PF)
    HardException = 3,
    /// Software interrupt (INT n)
    SoftIntr = 4,
    /// Privileged software exception (INT1)
    PrivSoftException = 5,
    /// Software exception (INT3 or INTO)
    SoftException = 6,
    /// Other event
    Other = 7,
}
}

impl VmxInterruptionType {
    /// Whether the exception/interrupt with `vector` has an error code.
    pub const fn vector_has_error_code(vector: u8) -> bool {
        use x86::irq::*;
        matches!(
            vector,
            DOUBLE_FAULT_VECTOR
                | INVALID_TSS_VECTOR
                | SEGMENT_NOT_PRESENT_VECTOR
                | STACK_SEGEMENT_FAULT_VECTOR
                | GENERAL_PROTECTION_FAULT_VECTOR
                | PAGE_FAULT_VECTOR
                | ALIGNMENT_CHECK_VECTOR
        )
    }

    /// Determine interruption type by the interrupt vector.
    pub const fn from_vector(vector: u8) -> Self {
        // SDM Vol. 3C, Section 24.8.3
        use x86::irq::*;
        match vector {
            DEBUG_VECTOR => Self::PrivSoftException,
            NONMASKABLE_INTERRUPT_VECTOR => Self::NMI,
            BREAKPOINT_VECTOR | OVERFLOW_VECTOR => Self::SoftException,
            // SDM Vol. 3A, Section 6.15: All other vectors from 0 to 21 are exceptions.
            0..=VIRTUALIZATION_VECTOR => Self::HardException,
            32..=255 => Self::External,
            _ => Self::Other,
        }
    }

    /// For software interrupt, software exception, or privileged software
    /// exception, we need to set VM-Entry Instruction Length Field.
    pub const fn is_soft(&self) -> bool {
        matches!(
            *self,
            Self::SoftIntr | Self::SoftException | Self::PrivSoftException
        )
    }
}

/// Exit Qualification for I/O Instructions. (SDM Vol. 3C, Section 27.2.1, Table 27-5)
#[derive(Debug)]
pub struct VmxIoExitInfo {
    /// Size of access.
    pub access_size: u8,
    /// Direction of the attempted access (0 = OUT, 1 = IN).
    pub is_in: bool,
    /// String instruction (0 = not string; 1 = string).
    pub is_string: bool,
    /// REP prefixed (0 = not REP; 1 = REP).
    pub is_repeat: bool,
    /// Port number. (as specified in DX or in an immediate operand)
    pub port: u16,
}

impl VmxIoExitInfo {
    pub fn new() -> HvResult<Self> {
        // SDM Vol. 3C, Section 27.2.1, Table 27-5
        let qualification = VmcsReadOnlyNW::EXIT_QUALIFICATION.read()?;
        Ok(VmxIoExitInfo {
            access_size: qualification.get_bits(0..3) as u8 + 1,
            is_in: qualification.get_bit(3),
            is_string: qualification.get_bit(4),
            is_repeat: qualification.get_bit(5),
            port: qualification.get_bits(16..32) as u16,
        })
    }
}

#[derive(Debug)]
pub struct VmxRegion {
    frame: Frame,
}

impl VmxRegion {
    pub fn fake_init() -> Self {
        Self {
            frame: unsafe { Frame::from_paddr(0) },
        }
    }

    pub fn new(revision_id: u32, shadow_indicator: bool) -> Self {
        let frame = Frame::new_zero().unwrap();
        unsafe {
            (*(frame.start_paddr() as *mut u32))
                .set_bits(0..=30, revision_id)
                .set_bit(31, shadow_indicator);
        }
        Self { frame }
    }

    pub fn start_paddr(&self) -> PhysAddr {
        self.frame.start_paddr()
    }
}

pub fn check_vmx_support() -> bool {
    if let Some(feature) = CpuId::new().get_feature_info() {
        feature.has_vmx()
    } else {
        false
    }
}

pub fn enable_vmxon() -> HvResult {
    let mut ctrl_reg = Msr::IA32_FEATURE_CONTROL;
    let ctrl_flag = FeatureControlFlags::from_bits_truncate(ctrl_reg.read());
    let locked = ctrl_flag.contains(FeatureControlFlags::LOCKED);
    let vmxon_outside = ctrl_flag.contains(FeatureControlFlags::VMXON_ENABLED_OUTSIDE_SMX);
    if !locked {
        unsafe {
            ctrl_reg.write(
                (ctrl_flag
                    | FeatureControlFlags::LOCKED
                    | FeatureControlFlags::VMXON_ENABLED_OUTSIDE_SMX)
                    .bits(),
            )
        }
    } else if !vmxon_outside {
        return hv_result_err!(EPERM, "VMX disabled by BIOS");
    }
    Ok(())
}

pub unsafe fn execute_vmxon(start_paddr: u64) -> HvResult {
    // enable VMX using the VMXE bit
    Cr4::write(Cr4::read() | Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS);
    // execute VMXON
    vmx::vmxon(start_paddr)?;

    Ok(())
}

pub fn get_vmcs_revision_id() -> u32 {
    let vmx_basic_flag = Msr::IA32_VMX_BASIC.read();
    vmx_basic_flag.get_bits(0..=30) as u32
}

pub fn is_vmx_enabled() -> bool {
    Cr4::read().contains(Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS)
}
