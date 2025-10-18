// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//  Solicey <lzoi_lth@163.com>

use crate::{
    arch::{
        acpi::{self, *},
        boot::BootParams,
        hpet, iommu, ipi,
        mm::new_s2_memory_set,
        msr::{
            get_msr_bitmap,
            Msr::{self, *},
            MsrBitmap,
        },
        pio::{get_pio_bitmap, PortIoBitmap},
        vmcs::*,
        vmx::*,
    },
    consts::{self, core_end, PER_CPU_SIZE},
    device::irqchip::pic::{check_pending_vectors, clear_vectors, ioapic, lapic::VirtLocalApic},
    error::{HvError, HvResult},
    memory::{
        addr::{phys_to_virt, PHYS_VIRT_OFFSET},
        mm::PARKING_MEMORY_SET,
        Frame, GuestPhysAddr, HostPhysAddr, MemFlags, MemoryRegion, PhysAddr, PAGE_SIZE,
        PARKING_INST_PAGE,
    },
    percpu::{this_cpu_data, this_zone},
    platform::{ROOT_ZONE_BOOT_STACK, ROOT_ZONE_CMDLINE},
    zone::{find_zone, this_zone_id},
};
use alloc::boxed::Box;
use bit_field::BitField;
use core::{
    arch::{asm, global_asm},
    fmt::{Debug, Formatter, Result},
    mem::size_of,
    ptr::copy_nonoverlapping,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
use raw_cpuid::CpuId;
use x86::{
    bits64::vmx,
    dtables::{self, DescriptorTablePointer},
    vmx::vmcs::control::{
        EntryControls, ExitControls, PinbasedControls, PrimaryControls, SecondaryControls,
    },
};
use x86_64::registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags};

const AP_START_PAGE_IDX: u8 = 6;
const AP_START_PAGE_PADDR: PhysAddr = AP_START_PAGE_IDX as usize * PAGE_SIZE;

static VMXON_DONE: AtomicU32 = AtomicU32::new(0);

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
    const U64_PER_PAGE: usize = PAGE_SIZE / size_of::<u64>();

    let ap_start_page_ptr = AP_START_PAGE_PADDR as *mut u64;
    let ap_start_page = core::slice::from_raw_parts_mut(ap_start_page_ptr, U64_PER_PAGE);
    core::ptr::copy_nonoverlapping(
        ap_start16 as *const u64,
        ap_start_page_ptr,
        (ap_end as usize - ap_start16 as usize) / 8,
    );
    ap_start_page[U64_PER_PAGE - 2] = (core_end() + (cpuid + 1) * PER_CPU_SIZE) as u64;
    ap_start_page[U64_PER_PAGE - 1] = ap_entry32 as u64;
}

pub fn cpu_start(cpuid: usize, start_addr: usize, opaque: usize) {
    unsafe { setup_ap_start_page(cpuid) };

    let lapic = VirtLocalApic::phys_local_apic();
    let apic_id = acpi::get_apic_id(cpuid);

    // Intel SDM Vol 3C, Section 8.4.4, MP Initialization Example
    unsafe { lapic.send_init_ipi(apic_id as u32) };
    hpet::busy_wait(Duration::from_millis(50)); // 10ms
    unsafe { lapic.send_sipi(AP_START_PAGE_IDX, apic_id as u32) };
    hpet::busy_wait(Duration::from_micros(2000)); // 200us
    unsafe { lapic.send_sipi(AP_START_PAGE_IDX, apic_id as u32) };
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
pub struct ArchCpu {
    // guest_regs and host_stack_top should always be at first.
    guest_regs: GeneralRegisters,
    host_stack_top: u64,
    pub cpuid: usize,
    pub power_on: bool,
    pub virt_lapic: VirtLocalApic,
    vmx_on: bool,
    vmcs_revision_id: u32,
    vmxon_region: VmxRegion,
    vmcs_region: VmxRegion,
    vm_launch_guest_regs: GeneralRegisters,
}

impl ArchCpu {
    pub fn new(cpuid: usize) -> Self {
        let cpuid = this_cpu_id();
        Self {
            guest_regs: GeneralRegisters::default(),
            host_stack_top: 0,
            cpuid,
            power_on: false,
            virt_lapic: VirtLocalApic::new(),
            vmx_on: false,
            vmcs_revision_id: 0,
            vmxon_region: VmxRegion::fake_init(),
            vmcs_region: VmxRegion::fake_init(),
            vm_launch_guest_regs: GeneralRegisters::default(),
        }
    }

    /// Advance guest `RIP` by `instr_len` bytes.
    pub fn advance_guest_rip(&mut self, instr_len: u8) -> HvResult {
        Ok(VmcsGuestNW::RIP.write(VmcsGuestNW::RIP.read()? + instr_len as usize)?)
    }

    pub fn cr(&self, cr_idx: usize) -> usize {
        (|| -> HvResult<usize> {
            Ok(match cr_idx {
                4 => {
                    let host_mask = VmcsControlNW::CR4_GUEST_HOST_MASK.read()?;
                    (VmcsControlNW::CR4_READ_SHADOW.read()? & host_mask)
                        | (VmcsGuestNW::CR4.read()? & !host_mask)
                }
                _ => unreachable!(),
            })
        })()
        .expect("Failed to read guest control register")
    }

    pub fn idle(&mut self) -> ! {
        unsafe { self.virt_lapic.phys_lapic.end_of_interrupt() };

        assert!(this_cpu_id() == self.cpuid);

        self.power_on = false;
        self.activate_vmx().unwrap();

        // info!("idle! cpuid: {:x}", self.cpuid);

        PARKING_MEMORY_SET.call_once(|| {
            let parking_code: [u8; 2] = [0xeb, 0xfe]; // jump short -2
            unsafe {
                PARKING_INST_PAGE[..2].copy_from_slice(&parking_code);
            }

            let mut gpm = new_s2_memory_set();
            gpm.insert(MemoryRegion::new_with_offset_mapper(
                0 as GuestPhysAddr,
                unsafe { &PARKING_INST_PAGE as *const _ as HostPhysAddr - PHYS_VIRT_OFFSET },
                PAGE_SIZE,
                MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
            ))
            .unwrap();
            gpm
        });

        self.setup_vmcs(0, true).unwrap();
        self.host_stack_top = (core_end() + (self.cpuid + 1) * PER_CPU_SIZE) as _;

        unsafe {
            PARKING_MEMORY_SET.get().unwrap().activate();
            self.vmx_launch();
        }
    }

    /// Guest general-purpose registers.
    pub fn regs(&self) -> &GeneralRegisters {
        &self.guest_regs
    }

    /// Mutable reference of guest general-purpose registers.
    pub fn regs_mut(&mut self) -> &mut GeneralRegisters {
        &mut self.guest_regs
    }

    pub fn run(&mut self) {
        if self.power_on {
            // x86 wake up cpu will send ipi twice, but we only want once
            return;
        }

        unsafe { self.virt_lapic.phys_lapic.end_of_interrupt() };

        assert!(this_cpu_id() == self.cpuid);
        let mut per_cpu = this_cpu_data();

        // info!("run! cpuid: {:x}", self.cpuid);

        self.power_on = true;
        self.activate_vmx().unwrap();

        if !per_cpu.boot_cpu {
            if let Some(ipi_info) = ipi::get_ipi_info(self.cpuid) {
                per_cpu.cpu_on_entry = ipi_info.lock().start_up_addr;
            }
            // VmcsGuestNW::RIP.write(per_cpu.cpu_on_entry).unwrap();
            // info!("AP start up! addr: {:x}", per_cpu.cpu_on_entry);
        }

        self.setup_vmcs(per_cpu.cpu_on_entry, false).unwrap();
        per_cpu.activate_gpm();

        if per_cpu.boot_cpu {
            // must be called after activate_gpm()
            iommu::activate();
            self.guest_regs = self.vm_launch_guest_regs.clone();
        }

        while VMXON_DONE.load(Ordering::Acquire) < unsafe { consts::MAX_CPU_NUM } as u32 - 1 {
            core::hint::spin_loop();
        }

        self.host_stack_top = (core_end() + (self.cpuid + 1) * PER_CPU_SIZE) as _;

        clear_vectors(self.cpuid);

        unsafe { self.vmx_launch() };

        loop {}
    }

    pub fn set_boot_cpu_vm_launch_regs(&mut self, rax: u64, rsi: u64) {
        self.vm_launch_guest_regs.rax = rax;
        self.vm_launch_guest_regs.rsi = rsi;
    }

    fn activate_vmx(&mut self) -> HvResult {
        if self.vmx_on {
            return Ok(());
        }
        assert!(check_vmx_support());
        // assert!(!is_vmx_enabled());

        // enable VMXON
        unsafe { enable_vmxon().unwrap() };

        // TODO: check related registers

        // get VMCS revision identifier in IA32_VMX_BASIC MSR
        self.vmcs_revision_id = get_vmcs_revision_id();
        self.vmxon_region = VmxRegion::new(self.vmcs_revision_id, false);

        unsafe { execute_vmxon(self.vmxon_region.start_paddr() as u64).unwrap() };

        self.vmx_on = true;
        VMXON_DONE.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn set_cr(&mut self, cr_idx: usize, val: u64) -> HvResult {
        match cr_idx {
            0 => {
                // Retrieve/validate restrictions on CR0
                //
                // In addition to what the VMX MSRs tell us, make sure that
                // - NW and CD are kept off as they are not updated on VM exit and we
                //   don't want them enabled for performance reasons while in root mode
                // - PE and PG can be freely chosen (by the guest) because we demand
                //   unrestricted guest mode support anyway
                // - ET is ignored
                let must0 = Msr::IA32_VMX_CR0_FIXED1.read();
                // & !(Cr0Flags::NOT_WRITE_THROUGH | Cr0Flags::CACHE_DISABLE).bits();
                let must1 = Msr::IA32_VMX_CR0_FIXED0.read()
                    & !(Cr0Flags::PAGING | Cr0Flags::PROTECTED_MODE_ENABLE).bits();
                VmcsGuestNW::CR0.write(((val & must0) | must1) as _)?;
                VmcsControlNW::CR0_READ_SHADOW.write(val as _)?;
                VmcsControlNW::CR0_GUEST_HOST_MASK.write((must1 | !must0) as _)?;
            }
            3 => VmcsGuestNW::CR3.write(val as _)?,
            4 => {
                let cr4_host_owned = Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;
                let cr4_read_shadow = 0;
                let val = val | Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS.bits();
                VmcsGuestNW::CR4.write(val as _)?;
                VmcsControlNW::CR4_GUEST_HOST_MASK.write(cr4_host_owned.bits() as _)?;
                VmcsControlNW::CR4_READ_SHADOW.write(cr4_read_shadow)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    // after activate_vmx
    fn setup_vmcs(&mut self, entry: GuestPhysAddr, is_idle: bool) -> HvResult {
        self.vmcs_region = VmxRegion::new(self.vmcs_revision_id, false);

        let start_paddr = self.vmcs_region.start_paddr() as usize;
        Vmcs::clear(start_paddr)?;
        Vmcs::load(start_paddr)?;

        self.setup_vmcs_host(&self.host_stack_top as *const _ as usize)?;
        self.setup_vmcs_guest(entry, ROOT_ZONE_BOOT_STACK)?;
        self.setup_vmcs_control()?;

        Ok(())
    }

    fn setup_vmcs_control(&mut self) -> HvResult {
        // intercept NMI and external interrupts
        use PinbasedControls as PinCtrl;
        Vmcs::set_control(
            VmcsControl32::PINBASED_EXEC_CONTROLS,
            Msr::IA32_VMX_TRUE_PINBASED_CTLS,
            Msr::IA32_VMX_PINBASED_CTLS.read() as u32,
            (PinCtrl::NMI_EXITING | PinCtrl::EXTERNAL_INTERRUPT_EXITING).bits(),
            0,
        )?;

        // use I/O bitmaps and MSR bitmaps, activate secondary controls,
        // disable CR3 load/store interception
        use PrimaryControls as CpuCtrl;
        Vmcs::set_control(
            VmcsControl32::PRIMARY_PROCBASED_EXEC_CONTROLS,
            Msr::IA32_VMX_TRUE_PROCBASED_CTLS,
            Msr::IA32_VMX_PROCBASED_CTLS.read() as u32,
            (CpuCtrl::HLT_EXITING
                // | CpuCtrl::RDTSC_EXITING 
                | CpuCtrl::USE_IO_BITMAPS
                | CpuCtrl::USE_MSR_BITMAPS
                | CpuCtrl::SECONDARY_CONTROLS)
                .bits(),
            (CpuCtrl::CR3_LOAD_EXITING | CpuCtrl::CR3_STORE_EXITING).bits(),
        )?;

        // enable EPT, RDTSCP, INVPCID, and unrestricted guest
        use SecondaryControls as CpuCtrl2;
        Vmcs::set_control(
            VmcsControl32::SECONDARY_PROCBASED_EXEC_CONTROLS,
            Msr::IA32_VMX_PROCBASED_CTLS2,
            0,
            (CpuCtrl2::ENABLE_EPT
                | CpuCtrl2::ENABLE_RDTSCP
                // | CpuCtrl2::VIRTUALIZE_X2APIC
                | CpuCtrl2::ENABLE_INVPCID
                | CpuCtrl2::UNRESTRICTED_GUEST)
                .bits(),
            0,
        )?;

        // load guest IA32_PAT/IA32_EFER on VM entry
        use EntryControls as EntryCtrl;
        Vmcs::set_control(
            VmcsControl32::VMENTRY_CONTROLS,
            Msr::IA32_VMX_TRUE_ENTRY_CTLS,
            Msr::IA32_VMX_ENTRY_CTLS.read() as u32,
            (EntryCtrl::LOAD_IA32_PAT | EntryCtrl::LOAD_IA32_EFER).bits(),
            0,
        )?;

        // switch to 64-bit host, acknowledge interrupt info, switch IA32_PAT/IA32_EFER on VM exit
        use ExitControls as ExitCtrl;
        Vmcs::set_control(
            VmcsControl32::VMEXIT_CONTROLS,
            Msr::IA32_VMX_TRUE_EXIT_CTLS,
            Msr::IA32_VMX_EXIT_CTLS.read() as u32,
            (ExitCtrl::HOST_ADDRESS_SPACE_SIZE
                | ExitCtrl::ACK_INTERRUPT_ON_EXIT
                | ExitCtrl::SAVE_IA32_PAT
                | ExitCtrl::LOAD_IA32_PAT
                | ExitCtrl::SAVE_IA32_EFER
                | ExitCtrl::LOAD_IA32_EFER)
                .bits(),
            0,
        )?;

        // no MSR switches if hypervisor doesn't use and there is only one vCPU
        VmcsControl32::VMEXIT_MSR_STORE_COUNT.write(0)?;
        VmcsControl32::VMEXIT_MSR_LOAD_COUNT.write(0)?;
        VmcsControl32::VMENTRY_MSR_LOAD_COUNT.write(0)?;

        // pass-through exceptions, set I/O bitmap and MSR bitmaps
        VmcsControl32::EXCEPTION_BITMAP.write(0)?;

        if self.power_on {
            let pio_bitmap = get_pio_bitmap(this_zone_id());
            VmcsControl64::IO_BITMAP_A_ADDR.write(pio_bitmap.a.start_paddr() as _)?;
            VmcsControl64::IO_BITMAP_B_ADDR.write(pio_bitmap.b.start_paddr() as _)?;
            VmcsControl64::MSR_BITMAPS_ADDR
                .write(get_msr_bitmap(this_zone_id()).phys_addr() as _)?;
        }

        // set virtual-APIC page address
        // self.virt_lapic.vapic_page = Frame::new_zero().unwrap();
        // VmcsControl64::VIRT_APIC_ADDR.write(self.virt_lapic.vapic_page.start_paddr() as _);
        Ok(())
    }

    fn setup_vmcs_guest(&mut self, entry: GuestPhysAddr, rsp: GuestPhysAddr) -> HvResult {
        let cr0_guest = Cr0Flags::EXTENSION_TYPE | Cr0Flags::NUMERIC_ERROR;
        let cr4_guest = Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;

        self.set_cr(0, cr0_guest.bits());
        self.set_cr(3, 0);
        self.set_cr(4, cr4_guest.bits());

        macro_rules! set_guest_segment {
            ($seg: ident, $access_rights: expr) => {{
                use VmcsGuest16::*;
                use VmcsGuest32::*;
                use VmcsGuestNW::*;
                concat_idents!($seg, _SELECTOR).write(0)?;
                concat_idents!($seg, _BASE).write(0)?;
                concat_idents!($seg, _LIMIT).write(0xffff)?;
                concat_idents!($seg, _ACCESS_RIGHTS).write($access_rights)?;
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

        VmcsGuestNW::GDTR_BASE.write(0)?;
        VmcsGuest32::GDTR_LIMIT.write(0xffff)?;
        VmcsGuestNW::IDTR_BASE.write(0)?;
        VmcsGuest32::IDTR_LIMIT.write(0xffff)?;

        VmcsGuestNW::DR7.write(0x400)?;
        VmcsGuestNW::RSP.write(rsp)?;
        VmcsGuestNW::RIP.write(entry)?;
        VmcsGuestNW::RFLAGS.write(0x2)?;
        VmcsGuestNW::PENDING_DBG_EXCEPTIONS.write(0)?;
        VmcsGuestNW::IA32_SYSENTER_ESP.write(0)?;
        VmcsGuestNW::IA32_SYSENTER_EIP.write(0)?;
        VmcsGuest32::IA32_SYSENTER_CS.write(0)?;

        VmcsGuest32::INTERRUPTIBILITY_STATE.write(0)?;
        VmcsGuest32::ACTIVITY_STATE.write(0)?;
        VmcsGuest32::VMX_PREEMPTION_TIMER_VALUE.write(0)?;

        VmcsGuest64::LINK_PTR.write(u64::MAX)?; // SDM Vol. 3C, Section 24.4.2
        VmcsGuest64::IA32_DEBUGCTL.write(0)?;
        VmcsGuest64::IA32_PAT.write(Msr::IA32_PAT.read())?;
        VmcsGuest64::IA32_EFER.write(0)?;

        // for AP start up, set CS_BASE to entry address, and RIP to 0.
        if self.power_on && !this_cpu_data().boot_cpu {
            VmcsGuestNW::RIP.write(0)?;
            VmcsGuestNW::CS_BASE.write(entry)?;
        }

        Ok(())
    }

    fn setup_vmcs_host(&mut self, rsp: GuestPhysAddr) -> HvResult {
        VmcsHost64::IA32_PAT.write(Msr::IA32_PAT.read())?;
        VmcsHost64::IA32_EFER.write(Msr::IA32_EFER.read())?;

        VmcsHostNW::CR0.write(Cr0::read_raw() as _)?;
        VmcsHostNW::CR3.write(Cr3::read_raw().0.start_address().as_u64() as _)?;
        VmcsHostNW::CR4.write(Cr4::read_raw() as _)?;

        VmcsHost16::ES_SELECTOR.write(x86::segmentation::es().bits())?;
        VmcsHost16::CS_SELECTOR.write(x86::segmentation::cs().bits())?;
        VmcsHost16::SS_SELECTOR.write(x86::segmentation::ss().bits())?;
        VmcsHost16::DS_SELECTOR.write(x86::segmentation::ds().bits())?;
        VmcsHost16::FS_SELECTOR.write(x86::segmentation::fs().bits())?;
        VmcsHost16::GS_SELECTOR.write(x86::segmentation::gs().bits())?;
        VmcsHostNW::FS_BASE.write(Msr::IA32_FS_BASE.read() as _)?;
        VmcsHostNW::GS_BASE.write(Msr::IA32_GS_BASE.read() as _)?;

        let tr = unsafe { x86::task::tr() };
        let mut gdtp = DescriptorTablePointer::<u64>::default();
        let mut idtp = DescriptorTablePointer::<u64>::default();
        unsafe {
            dtables::sgdt(&mut gdtp);
            dtables::sidt(&mut idtp);
        }
        VmcsHost16::TR_SELECTOR.write(tr.bits())?;
        VmcsHostNW::TR_BASE.write(get_tr_base(tr, &gdtp) as _)?;
        VmcsHostNW::GDTR_BASE.write(gdtp.base as _)?;
        VmcsHostNW::IDTR_BASE.write(idtp.base as _)?;
        VmcsHostNW::RSP.write(rsp)?;
        VmcsHostNW::RIP.write(Self::vmx_exit as usize)?;

        VmcsHostNW::IA32_SYSENTER_ESP.write(0)?;
        VmcsHostNW::IA32_SYSENTER_EIP.write(0)?;
        VmcsHost32::IA32_SYSENTER_CS.write(0)?;
        Ok(())
    }

    fn vmexit_handler(&mut self) {
        crate::arch::trap::handle_vmexit(self).unwrap();
        if (self.power_on) {
            check_pending_vectors(self.cpuid);
        }
    }

    unsafe fn vmx_entry_failed() -> ! {
        panic!("{}", Vmcs::instruction_error().unwrap().as_str());
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

    #[naked]
    unsafe extern "C" fn vmx_launch(&mut self) -> ! {
        asm!(
            // "mov    [rdi + {host_stack_top}], rsp", // save current RSP to host_stack_top
            "mov    rsp, rdi",                      // set RSP to guest regs area
            restore_regs_from_stack!(),
            "vmlaunch",
            "jmp    {failed}",
            // host_stack_top = const size_of::<GeneralRegisters>(),
            failed = sym Self::vmx_entry_failed,
            options(noreturn),
        )
    }
}

pub fn this_cpu_id() -> usize {
    crate::arch::acpi::get_cpu_id(this_apic_id())
}

pub fn this_apic_id() -> usize {
    match CpuId::new().get_feature_info() {
        Some(info) => info.initial_local_apic_id() as usize,
        None => {
            panic!("can not find apic id!");
            0
        }
    }
}

fn get_tr_base(
    tr: x86::segmentation::SegmentSelector,
    gdt: &x86::dtables::DescriptorTablePointer<u64>,
) -> u64 {
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

impl Debug for ArchCpu {
    fn fmt(&self, f: &mut Formatter) -> Result {
        (|| -> HvResult<Result> {
            Ok(f.debug_struct("ArchCpu")
                .field("guest_regs", &self.guest_regs)
                .field("rip", &VmcsGuestNW::RIP.read()?)
                .field("rsp", &VmcsGuestNW::RSP.read()?)
                .field("rflags", &VmcsGuestNW::RFLAGS.read()?)
                .field("cr0", &VmcsGuestNW::CR0.read()?)
                .field("cr3", &VmcsGuestNW::CR3.read()?)
                .field("cr4", &VmcsGuestNW::CR4.read()?)
                .field("gdtr_base", &VmcsGuestNW::GDTR_BASE.read()?)
                .field("cs_selector", &VmcsGuest16::CS_SELECTOR.read()?)
                .finish())
        })()
        .unwrap()
    }
}

pub fn store_cpu_pointer_to_reg(pointer: usize) {
    // println!("x86_64 doesn't support store cpu pointer to reg, pointer: {:#x}", pointer);
    return;
}

pub fn get_target_cpu(irq: usize, zone_id: usize) -> usize {
    ioapic::get_irq_cpu(irq, zone_id)
}
