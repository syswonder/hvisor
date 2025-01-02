#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use crate::error::{HvError, HvResult};
use crate::memory::{Frame, PhysAddr};
use bit_field::BitField;
use bitflags::{bitflags, Flags};
use raw_cpuid::CpuId;
use x86::dtables::{self, DescriptorTablePointer};
use x86::msr::{
    IA32_EFER, IA32_FEATURE_CONTROL, IA32_FS_BASE, IA32_GS_BASE, IA32_PAT, IA32_VMX_BASIC,
    IA32_VMX_ENTRY_CTLS, IA32_VMX_EXIT_CTLS, IA32_VMX_PINBASED_CTLS, IA32_VMX_PROCBASED_CTLS,
    IA32_VMX_PROCBASED_CTLS2, IA32_VMX_TRUE_ENTRY_CTLS, IA32_VMX_TRUE_EXIT_CTLS,
    IA32_VMX_TRUE_PINBASED_CTLS, IA32_VMX_TRUE_PROCBASED_CTLS,
};
use x86::segmentation::SegmentSelector;
use x86::vmx::vmcs::control::{
    EntryControls, ExitControls, PinbasedControls, PrimaryControls, SecondaryControls,
};
use x86::vmx::vmcs::*;
use x86::{bits64::vmx, vmx::VmFail};
use x86_64::registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags};
use x86_64::registers::model_specific::Msr;

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

pub fn vmread(field: u32) -> x86::vmx::Result<u64> {
    unsafe { vmx::vmread(field as u32) }
}

pub fn vmwrite<T: Into<u64>>(field: u32, value: T) -> x86::vmx::Result<()> {
    unsafe { vmx::vmwrite(field as u32, value.into()) }
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

pub unsafe fn enable_vmxon() -> HvResult {
    let mut ctrl_reg = Msr::new(IA32_FEATURE_CONTROL);
    let ctrl_flag = FeatureControlFlags::from_bits_truncate(ctrl_reg.read());
    let locked = ctrl_flag.contains(FeatureControlFlags::LOCKED);
    let vmxon_outside = ctrl_flag.contains(FeatureControlFlags::VMXON_ENABLED_OUTSIDE_SMX);
    if !locked {
        ctrl_reg.write(
            (ctrl_flag
                | FeatureControlFlags::LOCKED
                | FeatureControlFlags::VMXON_ENABLED_OUTSIDE_SMX)
                .bits(),
        )
    } else if !vmxon_outside {
        return Err(hv_err!(EPERM, "VMX disabled by BIOS"));
    }
    Ok(())
}

pub unsafe fn get_vmcs_revision_id() -> u32 {
    let vmx_basic_reg = Msr::new(IA32_VMX_BASIC);
    let vmx_basic_flag = vmx_basic_reg.read();
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

// natural-width
type unw = u64;

pub unsafe fn setup_vmcs_host(vmx_exit: usize) -> HvResult {
    vmwrite::<u64>(host::IA32_PAT_FULL, Msr::new(IA32_PAT).read())?;
    vmwrite::<u64>(host::IA32_EFER_FULL, Msr::new(IA32_EFER).read())?;

    vmwrite::<unw>(host::CR0, Cr0::read_raw())?;
    vmwrite::<unw>(host::CR3, Cr3::read_raw().0.start_address().as_u64())?;
    vmwrite::<unw>(host::CR4, Cr4::read_raw())?;

    vmwrite::<u16>(host::ES_SELECTOR, x86::segmentation::es().bits())?;
    vmwrite::<u16>(host::CS_SELECTOR, x86::segmentation::cs().bits())?;
    vmwrite::<u16>(host::SS_SELECTOR, x86::segmentation::ss().bits())?;
    vmwrite::<u16>(host::DS_SELECTOR, x86::segmentation::ds().bits())?;
    vmwrite::<u16>(host::FS_SELECTOR, x86::segmentation::fs().bits())?;
    vmwrite::<u16>(host::GS_SELECTOR, x86::segmentation::gs().bits())?;

    vmwrite::<unw>(host::FS_BASE, Msr::new(IA32_FS_BASE).read())?;
    vmwrite::<unw>(host::GS_BASE, Msr::new(IA32_GS_BASE).read())?;

    let tr = unsafe { x86::task::tr() };
    let mut gdtp = DescriptorTablePointer::<u64>::default();
    let mut idtp = DescriptorTablePointer::<u64>::default();
    unsafe {
        dtables::sgdt(&mut gdtp);
        dtables::sidt(&mut idtp);
    }

    vmwrite::<u16>(host::TR_SELECTOR, tr.bits())?;
    vmwrite::<unw>(host::TR_BASE, get_tr_base(tr, &gdtp))?;
    vmwrite::<unw>(host::GDTR_BASE, gdtp.base as unw)?;
    vmwrite::<unw>(host::IDTR_BASE, idtp.base as unw)?;
    vmwrite::<unw>(host::RIP, vmx_exit as unw)?;

    vmwrite::<unw>(host::IA32_SYSENTER_ESP, 0)?;
    vmwrite::<unw>(host::IA32_SYSENTER_EIP, 0)?;
    vmwrite::<u32>(host::IA32_SYSENTER_CS, 0)?;

    // VmcsHostNW::RSP.write(0)?; // TODO
    Ok(())
}

pub unsafe fn setup_vmcs_guest(entry: usize) -> HvResult {
    // Enable protected mode and paging.
    let cr0_guest = Cr0Flags::PROTECTED_MODE_ENABLE
        | Cr0Flags::EXTENSION_TYPE
        | Cr0Flags::NUMERIC_ERROR
        | Cr0Flags::PAGING;
    let cr0_host_owned =
        Cr0Flags::NUMERIC_ERROR | Cr0Flags::NOT_WRITE_THROUGH | Cr0Flags::CACHE_DISABLE;
    let cr0_read_shadow = Cr0Flags::NUMERIC_ERROR;

    vmwrite::<unw>(guest::CR0, cr0_guest.bits())?;
    vmwrite::<unw>(control::CR0_GUEST_HOST_MASK, cr0_host_owned.bits())?;
    vmwrite::<unw>(control::CR0_READ_SHADOW, cr0_read_shadow.bits())?;

    // Enable physical address extensions that required in IA-32e mode.
    let cr4_guest = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION | Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;
    let cr4_host_owned = Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;
    let cr4_read_shadow = 0;

    vmwrite::<unw>(guest::CR4, cr4_guest.bits())?;
    vmwrite::<unw>(control::CR4_GUEST_HOST_MASK, cr4_host_owned.bits())?;
    vmwrite::<unw>(control::CR4_READ_SHADOW, cr4_read_shadow)?;

    macro_rules! set_guest_segment {
        ($seg: ident, $access_rights: expr) => {{
            use guest::*;
            vmwrite::<u16>(concat_idents!($seg, _SELECTOR), 0)?;
            vmwrite::<unw>(concat_idents!($seg, _BASE), 0)?;
            vmwrite::<u32>(concat_idents!($seg, _LIMIT), 0xffff)?;
            vmwrite::<u32>(concat_idents!($seg, _ACCESS_RIGHTS), $access_rights)?;
        }};
    }

    set_guest_segment!(ES, 0x93); // 16-bit, present, data, read/write, accessed
    set_guest_segment!(CS, 0x209b); // 64-bit, present, code, exec/read, accessed
    set_guest_segment!(SS, 0x93);
    set_guest_segment!(DS, 0x93);
    set_guest_segment!(FS, 0x93);
    set_guest_segment!(GS, 0x93);
    set_guest_segment!(TR, 0x8b); // present, system, 32-bit TSS busy
    set_guest_segment!(LDTR, 0x82); // present, system, LDT

    vmwrite::<unw>(guest::GDTR_BASE, 0)?;
    vmwrite::<u32>(guest::GDTR_LIMIT, 0xffff)?;
    vmwrite::<unw>(guest::IDTR_BASE, 0)?;
    vmwrite::<u32>(guest::IDTR_LIMIT, 0xffff)?;

    vmwrite::<unw>(guest::CR3, Cr3::read_raw().0.start_address().as_u64())?;
    vmwrite::<unw>(guest::DR7, 0x400)?;
    vmwrite::<unw>(guest::RSP, 0)?;
    vmwrite::<unw>(guest::RIP, entry as unw)?;
    vmwrite::<unw>(guest::RFLAGS, 0x2)?;
    vmwrite::<unw>(guest::PENDING_DBG_EXCEPTIONS, 0)?;
    vmwrite::<unw>(guest::IA32_SYSENTER_ESP, 0)?;
    vmwrite::<unw>(guest::IA32_SYSENTER_EIP, 0)?;
    vmwrite::<u32>(guest::IA32_SYSENTER_CS, 0)?;

    vmwrite::<u32>(guest::INTERRUPTIBILITY_STATE, 0)?;
    vmwrite::<u32>(guest::ACTIVITY_STATE, 0)?;
    vmwrite::<u32>(guest::VMX_PREEMPTION_TIMER_VALUE, 0)?;

    vmwrite::<u64>(guest::LINK_PTR_FULL, u64::MAX)?;
    vmwrite::<u64>(guest::IA32_DEBUGCTL_FULL, 0)?;
    vmwrite::<u64>(guest::IA32_PAT_FULL, Msr::new(IA32_PAT).read())?;
    vmwrite::<u64>(guest::IA32_EFER_FULL, Msr::new(IA32_EFER).read())?;

    Ok(())
}

pub unsafe fn setup_vmcs_control() -> HvResult {
    // Intercept NMI, pass-through external interrupts.
    set_control(
        control::PINBASED_EXEC_CONTROLS,
        Msr::new(IA32_VMX_TRUE_PINBASED_CTLS),
        Msr::new(IA32_VMX_PINBASED_CTLS).read() as u32,
        PinbasedControls::NMI_EXITING.bits(),
        0,
    )?;

    // Activate secondary controls, disable CR3 load/store interception.
    set_control(
        control::PRIMARY_PROCBASED_EXEC_CONTROLS,
        Msr::new(IA32_VMX_TRUE_PROCBASED_CTLS),
        Msr::new(IA32_VMX_PROCBASED_CTLS).read() as u32,
        PrimaryControls::SECONDARY_CONTROLS.bits(),
        (PrimaryControls::CR3_LOAD_EXITING | PrimaryControls::CR3_STORE_EXITING).bits(),
    )?;

    // Enable RDTSCP, INVPCID.
    set_control(
        control::SECONDARY_PROCBASED_EXEC_CONTROLS,
        Msr::new(IA32_VMX_PROCBASED_CTLS2),
        0,
        (SecondaryControls::ENABLE_RDTSCP | SecondaryControls::ENABLE_INVPCID).bits(),
        0,
    )?;

    // Switch to 64-bit host, switch IA32_PAT/IA32_EFER on VM exit.
    set_control(
        control::VMEXIT_CONTROLS,
        Msr::new(IA32_VMX_TRUE_EXIT_CTLS),
        Msr::new(IA32_VMX_EXIT_CTLS).read() as u32,
        (ExitControls::HOST_ADDRESS_SPACE_SIZE
            | ExitControls::SAVE_IA32_PAT
            | ExitControls::LOAD_IA32_PAT
            | ExitControls::SAVE_IA32_EFER
            | ExitControls::LOAD_IA32_EFER)
            .bits(),
        0,
    )?;

    // Switch to 64-bit guest, load guest IA32_PAT/IA32_EFER on VM entry.
    set_control(
        control::VMENTRY_CONTROLS,
        Msr::new(IA32_VMX_TRUE_ENTRY_CTLS),
        Msr::new(IA32_VMX_ENTRY_CTLS).read() as u32,
        (EntryControls::IA32E_MODE_GUEST
            | EntryControls::LOAD_IA32_PAT
            | EntryControls::LOAD_IA32_EFER)
            .bits(),
        0,
    )?;

    // No MSR switches if hypervisor doesn't use and there is only one vCPU.
    vmwrite::<u32>(control::VMEXIT_MSR_STORE_COUNT, 0)?;
    vmwrite::<u32>(control::VMEXIT_MSR_LOAD_COUNT, 0)?;
    vmwrite::<u32>(control::VMENTRY_MSR_LOAD_COUNT, 0)?;

    // Pass-through exceptions, I/O instructions, and MSR read/write.
    vmwrite::<u32>(control::EXCEPTION_BITMAP, 0)?;
    vmwrite::<u64>(control::IO_BITMAP_A_ADDR_FULL, 0)?;
    vmwrite::<u64>(control::IO_BITMAP_B_ADDR_FULL, 0)?;
    vmwrite::<u64>(control::MSR_BITMAPS_ADDR_FULL, 0)?;

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
        return Err(hv_err!(
            EPERM,
            format!("can not set and clear the same bit in {:#x}", control)
        ));
    }
    if (allowed1 & set) != set {
        // failed if set 0-bits in allowed1
        return Err(hv_err!(
            EPERM,
            format!("can not set bits {:#x} in {:#x}", set, control)
        ));
    }
    if (allowed0 & clear) != 0 {
        // failed if clear 1-bits in allowed0
        return Err(hv_err!(
            EPERM,
            format!("can not clear bits {:#x} in {:#x}", clear, control)
        ));
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

pub unsafe fn advance_guest_rip(instr_len: u8) -> HvResult {
    Ok(vmwrite::<unw>(
        guest::RIP,
        (vmread(guest::RIP)? + instr_len as u64),
    )?)
}

pub unsafe fn instruction_error() -> u32 {
    vmread(ro::VM_INSTRUCTION_ERROR).unwrap() as u32
}

pub unsafe fn set_host_rsp(paddr: usize) -> HvResult {
    Ok(vmwrite::<unw>(host::RSP, paddr as unw)?)
}

pub unsafe fn exit_info() -> HvResult<VmxExitInfo> {
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
