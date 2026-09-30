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
//
// Per-CPU data model
// ------------------
// Each physical CPU owns one PerCpu slot of PER_CPU_SIZE bytes starting at
// PER_CPU_ARRAY_PTR + cpuid * PER_CPU_SIZE; the per-CPU stack sits above the
// slot. `id` is deliberately the first field: the per-arch register cache
// reads the id back from slot offset 0.
//
// `PerCpu::new` caches the slot base once per CPU in an architecture register
// (also materialized in `self_ptr` on x86_64, where a gs-relative load needs
// the pointer in memory). `this_cpu_data()`/`this_cpu_id()` are then a 1-2
// instruction read instead of a CPU-id lookup plus table scan:
//
//   | arch        | register       | guest can corrupt it?                  |
//   |-------------|----------------|----------------------------------------|
//   | aarch64     | TPIDR_EL2      | no - EL2-private                       |
//   | riscv64     | CSR_SSCRATCH   | no - HS level, guest uses VSSCRATCH    |
//   | x86_64      | IA32_GS_BASE   | no - VMCS host-state reloads per exit  |
//   | loongarch64 | root CSR SAVE0 | no - root CSR, guest GCSR file separate|
//
// Safety invariants
// -----------------
// 1. The register is written only in `PerCpu::new`. Log records (logging.rs
//    routes every record through this_cpu_data().id) can only fire after the
//    logger is installed in `primary_init_early`, which every CPU reaches
//    only after `PerCpu::new` (ENTERED_CPUS gate), so no core ever reads the
//    cache before writing it. `println!` does not go through the logger.
// 2. riscv64: sscratch must keep pointing at `&PerCpu.arch_cpu` because
//    trap.S swaps x31/sscratch to reach guest registers at ArchCpu offset 0.
//    set_this_cpu_pointer() takes the slot base and adds the field offset, so
//    this invariant never depends on callers.
// 3. x86_64: setup_vmcs_host snapshots IA32_GS_BASE into the VMCS host-state
//    field on every run/idle, i.e. strictly after PerCpu::new on the same
//    CPU, so hardware reloads the cached base on each VM exit.
// 4. loongarch64: SAVE3/SAVE4 stay reserved for the trap handoff; SAVE0 is
//    per-core root state and the guest's SAVE0 lives in the GCSR file.
//
use alloc::sync::Arc;
use spin::Mutex;

use crate::arch::cpu::{set_this_cpu_pointer, this_cpu_id, this_cpu_pointer, ArchCpu};
use crate::consts::{INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
use crate::zone::Zone;
use crate::ENTERED_CPUS;
use core::fmt::{Debug, Formatter, Result};
use core::sync::atomic::{AtomicU8, Ordering};

// global_asm!(include_str!("./arch/aarch64/page_table.S"),);

/// VCpu lifecycle states
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VcpuState {
    /// Initial state or after PSCI CPU_OFF. Not in any run queue.
    Stopped = 0,
    /// In a pCPU's run queue, waiting to be scheduled.
    Ready = 1,
    /// Currently executing on a pCPU.
    Running = 2,
    /// Blocked by WFI/CPU_SUSPEND. Not in any run queue, awaiting interrupt wakeup.
    Blocked = 3,
}

impl VcpuState {
    fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Stopped,
            1 => Self::Ready,
            2 => Self::Running,
            3 => Self::Blocked,
            _ => panic!("invalid vcpu state {}", value),
        }
    }
}

#[repr(transparent)]
pub struct VcpuStateCell {
    state: AtomicU8,
}

impl VcpuStateCell {
    pub const fn new(state: VcpuState) -> Self {
        Self {
            state: AtomicU8::new(state as u8),
        }
    }

    pub fn load(&self) -> VcpuState {
        VcpuState::from_raw(self.state.load(Ordering::Acquire))
    }

    pub fn store(&self, state: VcpuState) {
        self.state.store(state as u8, Ordering::Release);
    }

    pub fn is_stopped(&self) -> bool {
        self.load() == VcpuState::Stopped
    }

    pub fn is_ready(&self) -> bool {
        self.load() == VcpuState::Ready
    }

    pub fn is_running(&self) -> bool {
        self.load() == VcpuState::Running
    }

    pub fn is_blocked(&self) -> bool {
        self.load() == VcpuState::Blocked
    }

    /// Logical CPU is up from the hypervisor’s view: not `Stopped` (includes Ready, Running, Blocked).
    pub fn is_online(&self) -> bool {
        !self.is_stopped()
    }
}

impl Debug for VcpuStateCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.load().fmt(f)
    }
}

#[repr(C)]
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub dtb_ipa: usize,
    pub vcpu_state: VcpuStateCell,
    pub arch_cpu: ArchCpu,
    pub zone: Option<Arc<Zone>>,
    pub ctrl_lock: Mutex<()>,
    pub boot_cpu: bool,
    /// Slot base address of this PerCpu. Written once by `PerCpu::new`; x86_64
    /// materializes it here because gs-segment accesses need the pointer in
    /// memory, other architectures ignore it and read the register cache.
    pub self_ptr: usize,
    // percpu stack
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'static mut PerCpu {
        let arch_cpu = ArchCpu::new(cpu_id);
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + arch_cpu.cpuid as usize * PER_CPU_SIZE;
        let ret = vaddr as *mut Self;
        unsafe {
            ret.write_volatile(PerCpu {
                id: arch_cpu.cpuid,
                cpu_on_entry: INVALID_ADDRESS,
                dtb_ipa: INVALID_ADDRESS,
                vcpu_state: VcpuStateCell::new(VcpuState::Stopped),
                arch_cpu,
                zone: None,
                ctrl_lock: Mutex::new(()),
                boot_cpu: false,
                self_ptr: ret as usize,
            })
        };
        // Each CPU caches its own PerCpu slot base in the architecture register
        // backing this_cpu_pointer()/this_cpu_id(). All later per-CPU accesses
        // on this core read that cache instead of re-deriving the slot address.
        set_this_cpu_pointer(ret as usize);
        unsafe { ret.as_mut().unwrap() }
    }

    pub fn run_vm(&mut self) {
        if !self.boot_cpu {
            info!("CPU{}: Idling the CPU before starting VM...", self.id);
            self.arch_cpu.idle();
        }
        info!("CPU{}: Running the VM...", self.id);
        self.arch_cpu.run();
    }

    pub fn entered_cpus() -> u32 {
        ENTERED_CPUS.load(Ordering::Acquire)
    }

    pub fn activate_gpm(&self) {
        unsafe {
            self.zone.clone().unwrap().read().gpm().activate();
        }
    }
}

pub fn get_cpu_data<'a>(cpu_id: usize) -> &'a mut PerCpu {
    let cpu_data: usize = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
    unsafe { &mut *(cpu_data as *mut PerCpu) }
}

pub fn this_cpu_data<'a>() -> &'a mut PerCpu {
    // Slot base is cached per CPU in an architecture register at PerCpu::new
    // time, so this is a 1-2 instruction read with no CPU-id lookup involved.
    unsafe { &mut *(this_cpu_pointer() as *mut PerCpu) }
}

#[allow(unused)]
pub fn this_zone() -> Arc<Zone> {
    this_cpu_data().zone.clone().unwrap()
}

/// Enter blocked state and wait until another CPU resumes it.
#[allow(unused)]
pub fn vcpu_suspend() {
    info!("cpu {} suspending...", this_cpu_data().id);
    this_cpu_data().vcpu_state.store(VcpuState::Blocked);
    loop {
        // TODO: use wfi to optimize the loop
        if !this_cpu_data().vcpu_state.is_blocked() {
            break;
        }
        core::hint::spin_loop();
    }
    // Remote sets `Ready` to leave Blocked; this hart then marks itself `Running` again.
    this_cpu_data().vcpu_state.store(VcpuState::Running);
    info!("cpu {} resumed from suspend.", this_cpu_data().id);
}

/// Wait for other CPUs in the cpu set to enter Blocked state.
#[allow(unused)]
pub fn wait_for_other_vcpus_suspend(cpu_set: CpuSet) {
    let this_cpu_id = this_cpu_id();
    for target_cpu_id in cpu_set.iter() {
        if target_cpu_id == this_cpu_id {
            continue;
        }
        loop {
            if get_cpu_data(target_cpu_id).vcpu_state.is_blocked() {
                break;
            }
            core::hint::spin_loop();
        }
    }
}

/// Signal other CPUs in the cpu set to resume from Blocked.
#[allow(unused)]
pub fn signal_other_vcpus_resume(cpu_set: CpuSet) {
    let this_cpu_id = this_cpu_id();
    for target_cpu_id in cpu_set.iter() {
        if target_cpu_id == this_cpu_id {
            continue;
        }
        get_cpu_data(target_cpu_id)
            .vcpu_state
            .store(VcpuState::Ready);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CpuSet {
    pub max_cpu_id: usize,
    pub bitmap: u64,
}

impl CpuSet {
    pub fn new(max_cpu_id: usize, bitmap: u64) -> Self {
        Self { max_cpu_id, bitmap }
    }
    #[allow(unused)]
    pub fn set_bit(&mut self, id: usize) {
        assert!(id <= self.max_cpu_id);
        self.bitmap |= 1 << id;
    }
    #[allow(unused)]
    pub fn clear_bit(&mut self, id: usize) {
        assert!(id <= self.max_cpu_id);
        self.bitmap &= !(1 << id);
    }
    pub fn contains_cpu(&self, id: usize) -> bool {
        id <= self.max_cpu_id && (self.bitmap & (1 << id)) != 0
    }
    #[allow(unused)]
    pub fn first_cpu(&self) -> Option<usize> {
        (0..=self.max_cpu_id).find(move |&i| self.contains_cpu(i))
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        (0..=self.max_cpu_id).filter(move |&i| self.contains_cpu(i))
    }
    pub fn iter_except<'a>(&'a self, id: usize) -> impl Iterator<Item = usize> + 'a {
        (0..=self.max_cpu_id).filter(move |&i| self.contains_cpu(i) && i != id)
    }
}

#[test_case]
fn test_cpuset() {
    use alloc::vec::Vec;

    let mut cpuset = CpuSet::new(3, 0b1010);
    assert_eq!(cpuset.contains_cpu(0), false);
    assert_eq!(cpuset.contains_cpu(1), true);
    assert_eq!(cpuset.contains_cpu(2), false);
    assert_eq!(cpuset.contains_cpu(3), true);
    cpuset.set_bit(0);
    assert_eq!(cpuset.contains_cpu(0), true);
    assert_eq!(cpuset.contains_cpu(1), true);
    assert_eq!(cpuset.contains_cpu(2), false);
    assert_eq!(cpuset.contains_cpu(3), true);
    cpuset.clear_bit(1);
    assert_eq!(cpuset.contains_cpu(0), true);
    assert_eq!(cpuset.contains_cpu(1), false);
    assert_eq!(cpuset.contains_cpu(2), false);
    assert_eq!(cpuset.contains_cpu(3), true);
    assert_eq!(cpuset.first_cpu(), Some(0));
    assert_eq!(cpuset.iter().collect::<Vec<_>>(), vec![0, 3]);
    assert_eq!(cpuset.iter_except(0).collect::<Vec<_>>(), vec![3]);
}
