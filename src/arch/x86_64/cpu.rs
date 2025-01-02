use crate::arch::gdt::GdtStruct;
use crate::arch::lapic::{busy_wait, local_apic};
use crate::arch::vmx::*;
use crate::consts::{core_end, PER_CPU_SIZE};
use crate::error::{HvError, HvResult};
use crate::memory::{addr::phys_to_virt, Frame, PhysAddr, PAGE_SIZE};
use crate::percpu::this_cpu_data;
use alloc::boxed::Box;
use core::arch::{asm, global_asm};
use core::mem::size_of;
use core::time::Duration;
use raw_cpuid::CpuId;
use x86_64::structures::tss::TaskStateSegment;

const AP_START_PAGE_IDX: u8 = 6;
const AP_START_PAGE_PADDR: PhysAddr = AP_START_PAGE_IDX as usize * PAGE_SIZE;
const VM_EXIT_INSTR_LEN_VMCALL: u8 = 3;

global_asm!(
    include_str!("ap_start.S"),
    ap_start_page_paddr = const AP_START_PAGE_PADDR,
);

macro_rules! save_regs_to_stack {
    () => {
        "
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rdi
        push rsi
        push rbp
        sub rsp, 8
        push rbx
        push rdx
        push rcx
        push rax"
    };
}

macro_rules! restore_regs_from_stack {
    () => {
        "
        pop rax
        pop rcx
        pop rdx
        pop rbx
        add rsp, 8
        pop rbp
        pop rsi
        pop rdi
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15"
    };
}

unsafe fn setup_ap_start_page(cpuid: usize) {
    extern "C" {
        fn ap_start16();
        fn ap_end();
        fn ap_entry32();
    }
    const U64_PER_PAGE: usize = PAGE_SIZE / 8;

    let ap_start_page_ptr = phys_to_virt(AP_START_PAGE_PADDR) as *mut usize;
    let ap_start_page = core::slice::from_raw_parts_mut(ap_start_page_ptr, U64_PER_PAGE);
    core::ptr::copy_nonoverlapping(
        ap_start16 as *const usize,
        ap_start_page_ptr,
        (ap_end as usize - ap_start16 as usize) / 8,
    );
    ap_start_page[U64_PER_PAGE - 2] = core_end() as usize + (cpuid + 1) * PER_CPU_SIZE;
    ap_start_page[U64_PER_PAGE - 1] = ap_entry32 as usize;
}

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    unsafe { setup_ap_start_page(cpuid) };

    let lapic = local_apic();

    // Intel SDM Vol 3C, Section 8.4.4, MP Initialization Example
    unsafe { lapic.send_init_ipi(cpuid as u32) };
    busy_wait(Duration::from_millis(10)); // 10ms
    unsafe { lapic.send_sipi(AP_START_PAGE_IDX, cpuid as u32) };
    busy_wait(Duration::from_micros(200)); // 200us
    unsafe { lapic.send_sipi(AP_START_PAGE_IDX, cpuid as u32) };
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub usr: [u64; 15],

    // pushed by 'trap.S'
    pub vector: u64,
    pub error_code: u64,

    // pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

/// General-Purpose Registers for 64-bit x86 architecture.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct GeneralRegisters {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    _unused_rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct ArchCpu {
    // guest_regs and host_stack_top should always be at the first.
    guest_regs: GeneralRegisters,
    host_stack_top: u64,
    pub cpuid: usize,
    pub power_on: bool,
    pub gdt: GdtStruct,
    vmcs_revision_id: u32,
    vmxon_region: VmxRegion,
    vmcs_region: VmxRegion,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        let boxed = Box::new(TaskStateSegment::new());
        let tss = Box::leak(boxed);
        Self {
            cpuid,
            power_on: false,
            gdt: GdtStruct::new(tss),
            vmcs_revision_id: 0,
            vmxon_region: VmxRegion::uninit(),
            vmcs_region: VmxRegion::uninit(),
            guest_regs: GeneralRegisters::default(),
            host_stack_top: 0,
        }
    }

    pub unsafe fn init(&mut self, entry: usize, dtb: usize) {
        self.activate_vmx();
        self.setup_vmcs(entry);
    }

    unsafe fn activate_vmx(&mut self) {
        assert!(check_vmx_support());
        assert!(!is_vmx_enabled());

        // enable VMXON
        enable_vmxon().unwrap();

        // TODO: check related registers

        // get VMCS revision identifier in IA32_VMX_BASIC MSR
        self.vmcs_revision_id = get_vmcs_revision_id();
        self.vmxon_region = VmxRegion::new(self.vmcs_revision_id, false).unwrap();

        execute_vmxon(self.vmxon_region.start_paddr() as u64).unwrap();

        info!(
            "VMX enabled, region: 0x{:x}",
            self.vmxon_region.start_paddr(),
        );
    }

    unsafe fn setup_vmcs(&mut self, entry: usize) {
        self.vmcs_region = VmxRegion::new(self.vmcs_revision_id, false).unwrap();

        enable_vmcs(self.vmcs_region.start_paddr() as u64).unwrap();
        setup_vmcs_host(Self::vmx_exit as usize).unwrap();
        setup_vmcs_guest(entry).unwrap();
        setup_vmcs_control().unwrap();

        info!(
            "VMCS enabled, region: 0x{:x}",
            self.vmcs_region.start_paddr(),
        );
    }

    pub fn run(&mut self) -> ! {
        assert!(this_cpu_id() == self.cpuid);
        // TODO: this_cpu_data().cpu_on_entry
        unsafe {
            self.init(test_guest as usize, this_cpu_data().dtb_ipa);
            set_host_rsp(&self.host_stack_top as *const _ as usize).unwrap();
            self.vmx_launch();
        }
        loop {}
    }

    pub fn idle(&mut self) -> ! {
        assert!(this_cpu_id() == self.cpuid);
        unsafe { self.init(0, this_cpu_data().dtb_ipa) };
        loop {}
    }

    #[naked]
    unsafe extern "C" fn vmx_launch(&mut self) -> ! {
        asm!(
            "mov    [rdi + {host_stack_top}], rsp", // save current RSP to host_stack_top
            "mov    rsp, rdi",                      // set RSP to guest regs area
            restore_regs_from_stack!(),
            "vmlaunch",
            "jmp    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>(),
            failed = sym Self::vmx_entry_failed,
            options(noreturn),
        )
    }

    #[naked]
    unsafe extern "C" fn vmx_exit(&mut self) -> ! {
        asm!(
            save_regs_to_stack!(),
            "mov    r15, rsp",                      // save temporary RSP to r15
            "mov    rdi, rsp",                      // set the first arg to RSP
            "mov    rsp, [rsp + {host_stack_top}]", // set RSP to host_stack_top
            "call   {vmexit_handler}",              // call vmexit_handler
            "mov    rsp, r15",                      // load temporary RSP from r15
            restore_regs_from_stack!(),
            "vmresume",
            "jmp    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>(),
            vmexit_handler = sym Self::vmexit_handler,
            failed = sym Self::vmx_entry_failed,
            options(noreturn),
        );
    }

    unsafe fn vmx_entry_failed() -> ! {
        panic!("VMX instruction error: {}", instruction_error());
    }

    unsafe fn vmexit_handler(&mut self) {
        let exit_info = exit_info().unwrap();
        debug!("vmexit rax:{} {:#x?}", self.guest_regs.rax, exit_info);
        advance_guest_rip(VM_EXIT_INSTR_LEN_VMCALL).unwrap();
    }
}

pub fn this_cpu_id() -> usize {
    match CpuId::new().get_feature_info() {
        Some(info) => info.initial_local_apic_id() as usize,
        None => 0,
    }
}

#[naked]
unsafe extern "C" fn test_guest() -> ! {
    core::arch::asm!(
        "
        mov     rax, 0
        mov     rdi, 2
        mov     rsi, 3
        mov     rdx, 3
        mov     rcx, 3
    2:
        vmcall
        add     rax, 1
        jmp     2b",
        options(noreturn),
    );
}
