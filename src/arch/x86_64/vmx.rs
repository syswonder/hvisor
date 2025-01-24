use crate::{
    arch::{msr::Msr, s2pt::Stage2PageFaultInfo},
    consts::PAGE_SIZE,
    error::{HvError, HvResult},
    memory::{Frame, GuestPhysAddr, HostPhysAddr, HostVirtAddr, MemFlags, PhysAddr},
};
use bit_field::BitField;
use bitflags::{bitflags, Flags};
use raw_cpuid::CpuId;
use x86::{
    bits64::vmx,
    dtables,
    dtables::DescriptorTablePointer,
    segmentation::SegmentSelector,
    vmx::{vmcs::control::*, vmcs::*, VmFail},
};
use x86_64::registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags};

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

fn vmread(field: u32) -> x86::vmx::Result<u64> {
    unsafe { vmx::vmread(field as u32) }
}

fn vmwrite<T: Into<u64>>(field: u32, value: T) -> x86::vmx::Result<()> {
    unsafe { vmx::vmwrite(field as u32, value.into()) }
}

const ZERO: u64 = 0;

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

#[derive(Debug)]
pub struct VmxRegion {
    frame: Frame,
}

impl VmxRegion {
    pub fn uninit() -> Self {
        Self {
            frame: unsafe { Frame::from_paddr(0) },
        }
    }

    pub fn new(revision_id: u32, shadow_indicator: bool) -> HvResult<Self> {
        let frame = Frame::new_zero()?;
        unsafe {
            (*(frame.start_paddr() as *mut u32))
                .set_bits(0..=30, revision_id)
                .set_bit(31, shadow_indicator);
        }
        Ok(Self { frame })
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

pub fn is_vmx_enabled() -> bool {
    Cr4::read().contains(Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS)
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

pub fn get_vmcs_revision_id() -> u32 {
    let vmx_basic_flag = Msr::IA32_VMX_BASIC.read();
    vmx_basic_flag.get_bits(0..=30) as u32
}

pub unsafe fn execute_vmxon(start_paddr: u64) -> HvResult {
    // enable VMX using the VMXE bit
    Cr4::write(Cr4::read() | Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS);
    // execute VMXON
    vmx::vmxon(start_paddr)?;

    Ok(())
}

pub unsafe fn enable_vmcs(start_paddr: u64) -> HvResult {
    vmx::vmclear(start_paddr)?;
    vmx::vmptrld(start_paddr)?;

    Ok(())
}

pub fn setup_vmcs_host(vmx_exit: HostVirtAddr) -> HvResult {
    vmwrite(host::IA32_PAT_FULL, Msr::IA32_PAT.read())?;
    vmwrite(host::IA32_EFER_FULL, Msr::IA32_EFER.read())?;

    vmwrite(host::CR0, Cr0::read_raw())?;
    vmwrite(host::CR3, Cr3::read_raw().0.start_address().as_u64())?;
    vmwrite(host::CR4, Cr4::read_raw())?;

    vmwrite(host::ES_SELECTOR, x86::segmentation::es().bits())?;
    vmwrite(host::CS_SELECTOR, x86::segmentation::cs().bits())?;
    vmwrite(host::SS_SELECTOR, x86::segmentation::ss().bits())?;
    vmwrite(host::DS_SELECTOR, x86::segmentation::ds().bits())?;
    vmwrite(host::FS_SELECTOR, x86::segmentation::fs().bits())?;
    vmwrite(host::GS_SELECTOR, x86::segmentation::gs().bits())?;

    vmwrite(host::FS_BASE, Msr::IA32_FS_BASE.read())?;
    vmwrite(host::GS_BASE, Msr::IA32_GS_BASE.read())?;

    let tr = unsafe { x86::task::tr() };
    let mut gdtp = DescriptorTablePointer::<u64>::default();
    let mut idtp = DescriptorTablePointer::<u64>::default();
    unsafe {
        dtables::sgdt(&mut gdtp);
        dtables::sidt(&mut idtp);
    }

    vmwrite(host::TR_SELECTOR, tr.bits())?;
    vmwrite(host::TR_BASE, get_tr_base(tr, &gdtp))?;
    vmwrite(host::GDTR_BASE, gdtp.base as u64)?;
    vmwrite(host::IDTR_BASE, idtp.base as u64)?;
    vmwrite(host::RIP, vmx_exit as u64)?;

    vmwrite(host::IA32_SYSENTER_ESP, ZERO)?;
    vmwrite(host::IA32_SYSENTER_EIP, ZERO)?;
    vmwrite(host::IA32_SYSENTER_CS, ZERO)?;

    // VmcsHostNW::RSP.write(ZERO)?; // TODO
    Ok(())
}

pub fn setup_vmcs_guest(entry: GuestPhysAddr) -> HvResult {
    // Enable protected mode and paging.
    let cr0_guest = Cr0Flags::EXTENSION_TYPE | Cr0Flags::NUMERIC_ERROR;
    let cr0_host_owned =
        Cr0Flags::NUMERIC_ERROR | Cr0Flags::NOT_WRITE_THROUGH | Cr0Flags::CACHE_DISABLE;
    let cr0_read_shadow = Cr0Flags::NUMERIC_ERROR;

    vmwrite(guest::CR0, cr0_guest.bits())?;
    vmwrite(control::CR0_GUEST_HOST_MASK, cr0_host_owned.bits())?;
    vmwrite(control::CR0_READ_SHADOW, cr0_read_shadow.bits())?;

    // Enable physical address extensions that required in IA-32e mode.
    let cr4_guest = Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;
    let cr4_host_owned = Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;
    let cr4_read_shadow = ZERO;

    vmwrite(guest::CR4, cr4_guest.bits())?;
    vmwrite(control::CR4_GUEST_HOST_MASK, cr4_host_owned.bits())?;
    vmwrite(control::CR4_READ_SHADOW, cr4_read_shadow)?;

    macro_rules! set_guest_segment {
        ($seg: ident, $access_rights: expr) => {{
            use guest::*;
            vmwrite(concat_idents!($seg, _SELECTOR), ZERO)?;
            vmwrite(concat_idents!($seg, _BASE), ZERO)?;
            vmwrite(concat_idents!($seg, _LIMIT), 0xffff_u64)?;
            vmwrite(concat_idents!($seg, _ACCESS_RIGHTS), $access_rights as u64)?;
        }};
    }

    set_guest_segment!(ES, 0x93); // 16-bit, present, data, read/write, accessed
    set_guest_segment!(CS, 0x9b); // 16-bit, present, code, exec/read, accessed
    set_guest_segment!(SS, 0x93);
    set_guest_segment!(DS, 0x93);
    set_guest_segment!(FS, 0x93);
    set_guest_segment!(GS, 0x93);
    set_guest_segment!(TR, 0x8b); // present, system, 32-bit TSS busy
    set_guest_segment!(LDTR, 0x82); // present, system, LDT

    vmwrite(guest::GDTR_BASE, ZERO)?;
    vmwrite(guest::GDTR_LIMIT, 0xffff_u64)?;
    vmwrite(guest::IDTR_BASE, ZERO)?;
    vmwrite(guest::IDTR_LIMIT, 0xffff_u64)?;

    vmwrite(guest::CR3, ZERO)?;
    vmwrite(guest::DR7, 0x400_u64)?;
    vmwrite(guest::RSP, ZERO)?;
    vmwrite(guest::RIP, entry as u64)?;
    vmwrite(guest::RFLAGS, 0x2_u64)?;
    vmwrite(guest::PENDING_DBG_EXCEPTIONS, ZERO)?;
    vmwrite(guest::IA32_SYSENTER_ESP, ZERO)?;
    vmwrite(guest::IA32_SYSENTER_EIP, ZERO)?;
    vmwrite(guest::IA32_SYSENTER_CS, ZERO)?;

    vmwrite(guest::INTERRUPTIBILITY_STATE, ZERO)?;
    vmwrite(guest::ACTIVITY_STATE, ZERO)?;
    vmwrite(guest::VMX_PREEMPTION_TIMER_VALUE, ZERO)?;

    vmwrite(guest::LINK_PTR_FULL, u64::MAX)?;
    vmwrite(guest::IA32_DEBUGCTL_FULL, ZERO)?;
    vmwrite(guest::IA32_PAT_FULL, Msr::IA32_PAT.read())?;
    vmwrite(guest::IA32_EFER_FULL, ZERO)?;

    Ok(())
}

pub fn setup_vmcs_control(msr_bitmap: HostPhysAddr) -> HvResult {
    // Intercept NMI and external interrupts.
    set_control(
        control::PINBASED_EXEC_CONTROLS,
        Msr::IA32_VMX_TRUE_PINBASED_CTLS,
        Msr::IA32_VMX_PINBASED_CTLS.read() as u32,
        (PinbasedControls::NMI_EXITING | PinbasedControls::EXTERNAL_INTERRUPT_EXITING).bits(),
        0,
    )?;

    // Intercept all I/O instructions, use MSR bitmaps, activate secondary controls,
    // disable CR3 load/store interception.
    set_control(
        control::PRIMARY_PROCBASED_EXEC_CONTROLS,
        Msr::IA32_VMX_TRUE_PROCBASED_CTLS,
        Msr::IA32_VMX_PROCBASED_CTLS.read() as u32,
        (PrimaryControls::UNCOND_IO_EXITING
            | PrimaryControls::USE_MSR_BITMAPS
            | PrimaryControls::SECONDARY_CONTROLS)
            .bits(),
        (PrimaryControls::CR3_LOAD_EXITING | PrimaryControls::CR3_STORE_EXITING).bits(),
    )?;

    // Enable EPT, RDTSCP, INVPCID, and unrestricted guest.
    set_control(
        control::SECONDARY_PROCBASED_EXEC_CONTROLS,
        Msr::IA32_VMX_PROCBASED_CTLS2,
        0,
        (SecondaryControls::ENABLE_EPT
            | SecondaryControls::ENABLE_RDTSCP
            | SecondaryControls::ENABLE_INVPCID
            | SecondaryControls::UNRESTRICTED_GUEST)
            .bits(),
        0,
    )?;

    // Switch to 64-bit host, acknowledge interrupt info, switch IA32_PAT/IA32_EFER on VM exit.
    set_control(
        control::VMEXIT_CONTROLS,
        Msr::IA32_VMX_TRUE_EXIT_CTLS,
        Msr::IA32_VMX_EXIT_CTLS.read() as u32,
        (ExitControls::HOST_ADDRESS_SPACE_SIZE
            | ExitControls::ACK_INTERRUPT_ON_EXIT
            | ExitControls::SAVE_IA32_PAT
            | ExitControls::LOAD_IA32_PAT
            | ExitControls::SAVE_IA32_EFER
            | ExitControls::LOAD_IA32_EFER)
            .bits(),
        0,
    )?;

    // Load guest IA32_PAT/IA32_EFER on VM entry.
    set_control(
        control::VMENTRY_CONTROLS,
        Msr::IA32_VMX_TRUE_ENTRY_CTLS,
        Msr::IA32_VMX_ENTRY_CTLS.read() as u32,
        (EntryControls::LOAD_IA32_PAT | EntryControls::LOAD_IA32_EFER).bits(),
        0,
    )?;

    // No MSR switches if hypervisor doesn't use and there is only one vCPU.
    vmwrite(control::VMEXIT_MSR_STORE_COUNT, ZERO)?;
    vmwrite(control::VMEXIT_MSR_LOAD_COUNT, ZERO)?;
    vmwrite(control::VMENTRY_MSR_LOAD_COUNT, ZERO)?;

    // Pass-through exceptions, don't use I/O bitmap, set MSR bitmaps.
    vmwrite(control::EXCEPTION_BITMAP, ZERO)?;
    vmwrite(control::IO_BITMAP_A_ADDR_FULL, ZERO)?;
    vmwrite(control::IO_BITMAP_B_ADDR_FULL, ZERO)?;
    vmwrite(control::MSR_BITMAPS_ADDR_FULL, msr_bitmap as u64)?;

    Ok(())
}

fn get_tr_base(tr: SegmentSelector, gdt: &DescriptorTablePointer<u64>) -> u64 {
    let index = tr.index() as usize;
    let table_len = (gdt.limit as usize + 1) / core::mem::size_of::<u64>();
    let table = unsafe { core::slice::from_raw_parts(gdt.base, table_len) };
    let entry = table[index];
    if entry & (1 << 47) != 0 {
        // present
        let base_low = entry.get_bits(16..40) | entry.get_bits(56..64) << 24;
        let base_high = table[index + 1] & 0xffff_ffff;
        base_low | base_high << 32
    } else {
        // no present
        0
    }
}

pub fn set_control(
    control: u32,
    capability_msr: Msr,
    old_value: u32,
    set: u32,
    clear: u32,
) -> HvResult<()> {
    let cap = unsafe { capability_msr.read() };
    let allowed0 = cap as u32;
    let allowed1 = (cap >> 32) as u32;
    assert_eq!(allowed0 & allowed1, allowed0);
    debug!(
        "set {:#x}: {:#x} (+{:#x}, -{:#x})",
        control, old_value, set, clear
    );
    if (set & clear) != 0 {
        return hv_result_err!(
            EINVAL,
            format!("can not set and clear the same bit in {:#x}", control)
        );
    }
    if (allowed1 & set) != set {
        // failed if set 0-bits in allowed1
        return hv_result_err!(
            EINVAL,
            format!("can not set bits {:#x} in {:#x}", set, control)
        );
    }
    if (allowed0 & clear) != 0 {
        // failed if clear 1-bits in allowed0
        return hv_result_err!(
            EINVAL,
            format!("can not clear bits {:#x} in {:#x}", clear, control)
        );
    }
    // SDM Vol. 3C, Section 31.5.1, Algorithm 3
    let flexible = !allowed0 & allowed1; // therse bits can be either 0 or 1
    let unknown = flexible & !(set | clear); // hypervisor untouched bits
    let default = unknown & old_value; // these bits keep unchanged in old value
    let fixed1 = allowed0; // these bits are fixed to 1
    vmwrite(control, fixed1 | default | set)?;
    Ok(())
}

impl From<VmFail> for HvError {
    fn from(err: VmFail) -> Self {
        hv_err!(EFAULT, format!("VMX instruction failed: {:?}", err))
    }
}

pub fn advance_guest_rip(instr_len: u8) -> HvResult {
    unsafe {
        Ok(vmwrite(
            guest::RIP,
            (vmread(guest::RIP)? + instr_len as u64),
        )?)
    }
}

pub fn instruction_error() -> u32 {
    vmread(ro::VM_INSTRUCTION_ERROR).unwrap() as u32
}

pub fn set_host_rsp(rsp: HostPhysAddr) -> HvResult {
    Ok(vmwrite(host::RSP, rsp as u64)?)
}

pub fn set_guest_page_table(cr3: GuestPhysAddr) -> HvResult {
    Ok(vmwrite(guest::CR3, cr3 as u64)?)
}

pub fn set_guest_stack_pointer(rsp: GuestPhysAddr) -> HvResult {
    Ok(vmwrite(guest::RSP, rsp as u64)?)
}

pub fn set_s2ptp(s2ptp: u64) -> HvResult {
    Ok(vmwrite(control::EPTP_FULL, s2ptp as u64)?)
}

pub fn guest_rip() -> u64 {
    vmread(guest::RIP).unwrap() as u64
}

pub fn guest_rsp() -> u64 {
    vmread(guest::RSP).unwrap() as u64
}

pub fn guest_cr3() -> u64 {
    vmread(guest::CR3).unwrap() as u64
}

pub fn exit_info() -> HvResult<VmxExitInfo> {
    let full_reason = vmread(ro::EXIT_REASON)? as u32;
    Ok(VmxExitInfo {
        exit_reason: full_reason
            .get_bits(0..16)
            .try_into()
            .expect("Unknown VM-exit reason"),
        entry_failure: full_reason.get_bit(31),
        exit_instruction_length: vmread(ro::VMEXIT_INSTRUCTION_LEN)? as u32,
        guest_rip: vmread(guest::RIP)? as usize,
    })
}

pub fn s2pt_violation_info() -> HvResult<Stage2PageFaultInfo> {
    // SDM Vol. 3C, Section 27.2.1, Table 27-7
    let qualification = vmread(ro::EXIT_QUALIFICATION)? as u64;
    let fault_guest_paddr = vmread(ro::GUEST_PHYSICAL_ADDR_FULL)? as usize;
    let mut access_flags = MemFlags::empty();
    if qualification.get_bit(0) {
        access_flags |= MemFlags::READ;
    }
    if qualification.get_bit(1) {
        access_flags |= MemFlags::WRITE;
    }
    if qualification.get_bit(2) {
        access_flags |= MemFlags::EXECUTE;
    }
    Ok(Stage2PageFaultInfo {
        access_flags,
        fault_guest_paddr,
    })
}

pub fn io_exit_info() -> HvResult<VmxIoExitInfo> {
    // SDM Vol. 3C, Section 27.2.1, Table 27-5
    let qualification = vmread(ro::EXIT_QUALIFICATION)?;
    Ok(VmxIoExitInfo {
        access_size: qualification.get_bits(0..3) as u8 + 1,
        is_in: qualification.get_bit(3),
        is_string: qualification.get_bit(4),
        is_repeat: qualification.get_bit(5),
        port: qualification.get_bits(16..32) as u16,
    })
}

pub fn allow_interrupt() -> HvResult<bool> {
    let rflags = vmread(guest::RFLAGS)?;
    let block_state = vmread(guest::INTERRUPTIBILITY_STATE)?;
    Ok(
        rflags as u64 & x86_64::registers::rflags::RFlags::INTERRUPT_FLAG.bits() != 0
            && block_state == 0,
    )
}

pub fn inject_event(vector: u8, err_code: Option<u32>) -> HvResult {
    // SDM Vol. 3C, Section 24.8.3
    let err_code = if VmxInterruptionType::vector_has_error_code(vector) {
        err_code.or_else(|| Some(vmread(ro::VMEXIT_INTERRUPTION_ERR_CODE).unwrap() as u32))
    } else {
        None
    };
    let int_info = VmxInterruptInfo::from(vector, err_code);
    if let Some(err_code) = int_info.err_code {
        vmwrite(control::VMENTRY_EXCEPTION_ERR_CODE, err_code)?;
    }
    if int_info.int_type.is_soft() {
        vmwrite(
            control::VMENTRY_INSTRUCTION_LEN,
            vmread(ro::VMEXIT_INSTRUCTION_LEN)?,
        )?;
    }
    vmwrite(control::VMENTRY_INTERRUPTION_INFO_FIELD, int_info.bits())?;
    Ok(())
}

/// If enable, a VM exit occurs at the beginning of any instruction if
/// `RFLAGS.IF` = 1 and there are no other blocking of interrupts.
/// (see SDM, Vol. 3C, Section 24.4.2)
pub fn set_interrupt_window(enable: bool) -> HvResult {
    let mut ctrl = vmread(control::PRIMARY_PROCBASED_EXEC_CONTROLS)? as u32;
    let bits = PrimaryControls::INTERRUPT_WINDOW_EXITING.bits();
    if enable {
        ctrl |= bits
    } else {
        ctrl &= !bits
    }
    vmwrite(control::PRIMARY_PROCBASED_EXEC_CONTROLS, ctrl)?;
    Ok(())
}

pub fn interrupt_exit_info() -> HvResult<VmxInterruptInfo> {
    // SDM Vol. 3C, Section 24.9.2
    let info = vmread(ro::VMEXIT_INTERRUPTION_INFO)?;
    Ok(VmxInterruptInfo {
        vector: info.get_bits(0..8) as u8,
        int_type: VmxInterruptionType::try_from(info.get_bits(8..11) as u8).unwrap(),
        err_code: if info.get_bit(11) {
            Some(vmread(ro::VMEXIT_INTERRUPTION_ERR_CODE)? as u32)
        } else {
            None
        },
        valid: info.get_bit(31),
    })
}
