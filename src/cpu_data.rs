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
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Result};
use core::sync::atomic::{fence, AtomicU64, AtomicU8, Ordering};
use spin::Mutex;

use crate::arch::{
    cpu::{store_cpu_pointer_to_reg, this_cpu_id, ArchCpu},
    ArchVcpu, GUEST_IRQ_NR, GUEST_LINES,
};
use crate::consts::{INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
use crate::zone::Zone;
use crate::ENTERED_CPUS;

#[repr(C)]
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub dtb_ipa: usize,
    // Currently each pCPU is bound to exactly one vCPU (temporary 1:1 mapping).
    // TODO: Move the vCPU out of PerCpu once fixed 1:N binding is supported.
    pub vcpu: Vcpu,
    pub arch_cpu: ArchCpu,
    pub zone: Option<Arc<Zone>>,
    pub ctrl_lock: Mutex<()>,
    pub boot_cpu: bool,
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
                vcpu: Vcpu::new(),
                arch_cpu,
                zone: None,
                ctrl_lock: Mutex::new(()),
                boot_cpu: false,
            })
        };
        unsafe {
            let pointer = &ret.as_mut().unwrap().arch_cpu as *const _ as usize;
            store_cpu_pointer_to_reg(pointer);
        }
        // #[cfg(target_arch = "riscv64")]
        // {
        //     use crate::arch::csr::{write_csr, CSR_SSCRATCH};
        //     write_csr!(
        //         CSR_SSCRATCH,
        //         &ret.as_mut().unwrap().arch_cpu as *const _ as usize
        //     ); //arch cpu pointer
        // }
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
    // Note: this_cpu_id() should return logical cpu_id 0..BOARD_NCPUS
    get_cpu_data(this_cpu_id())
}

#[allow(unused)]
pub fn this_zone() -> Arc<Zone> {
    this_cpu_data().zone.clone().unwrap()
}

/// Per-vCPU state: lifecycle, posted guest IRQs and architecture-private data.
//
// Temporarily colocated with its pinned host pCPU. This object, including its
// `arch` member, is the unit that can later move out of `PerCpu` when fixed
// 1:N vCPU scheduling returns.
pub struct Vcpu {
    pub state: VcpuStateCell,
    pub irq: GuestCpuIrqState,
    pub arch: ArchVcpu,
}

impl Vcpu {
    pub const fn new() -> Self {
        Self {
            state: VcpuStateCell::new(VcpuState::Stopped),
            irq: GuestCpuIrqState::new(),
            arch: ArchVcpu::new(),
        }
    }

    /// Mark `Running` before guest entry.
    //
    // The fence pairs with `kick_vcpu()` so a posted IRQ is never missed.
    pub fn set_running(&self) {
        self.state.store(VcpuState::Running);
        fence(Ordering::SeqCst);
    }

    /// Drop the pending bitmap and architecture-private IRQ state. Call with
    /// the pCPU's `ctrl_lock` held, on zone shutdown or before handing a
    /// stopped vCPU to a zone.
    pub fn clear_irq_state(&self) {
        self.irq.clear_all();
        self.arch.clear_irq_state();
    }
}

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

const GUEST_IRQ_WORD_BITS: usize = u64::BITS as usize;
const GUEST_IRQ_WORDS: usize = (GUEST_IRQ_NR + 63) / 64;

const LINE_MASK: [u64; GUEST_IRQ_WORDS] = {
    let mut mask = [0; GUEST_IRQ_WORDS];
    let mut i = 0;
    while i < GUEST_LINES.len() {
        let irq_id = GUEST_LINES[i];
        mask[irq_id / GUEST_IRQ_WORD_BITS] |= 1 << (irq_id % GUEST_IRQ_WORD_BITS);
        i += 1;
    }
    mask
};

/// Pending interrupt inputs of one guest CPU, numbered below `GUEST_IRQ_NR`.
//
// An edge bit asks for an install at the next flush. A level bit holds the
// line value; `line_dirty` marks lines the next flush must sync.
// Architecture-private metadata remains in `ArchVcpu`; this state is the
// common producer/consumer contract.
pub struct GuestCpuIrqState {
    pending: [AtomicU64; GUEST_IRQ_WORDS],
    line_dirty: [AtomicU64; GUEST_IRQ_WORDS],
}

impl GuestCpuIrqState {
    pub const fn new() -> Self {
        Self {
            pending: [const { AtomicU64::new(0) }; GUEST_IRQ_WORDS],
            line_dirty: [const { AtomicU64::new(0) }; GUEST_IRQ_WORDS],
        }
    }

    pub const fn is_line(irq_id: usize) -> bool {
        irq_id < GUEST_IRQ_NR
            && LINE_MASK[irq_id / GUEST_IRQ_WORD_BITS] & (1 << (irq_id % GUEST_IRQ_WORD_BITS)) != 0
    }

    /// Set a level line and mark it dirty. Returns whether the level changed.
    pub fn set_level(&self, irq_id: usize, level: bool) -> bool {
        debug_assert!(Self::is_line(irq_id));
        let word = irq_id / GUEST_IRQ_WORD_BITS;
        let mask = 1u64 << (irq_id % GUEST_IRQ_WORD_BITS);
        let old = if level {
            self.pending[word].fetch_or(mask, Ordering::AcqRel)
        } else {
            self.pending[word].fetch_and(!mask, Ordering::AcqRel)
        };
        if (old & mask != 0) == level {
            return false;
        }
        self.line_dirty[word].fetch_or(mask, Ordering::Release);
        true
    }

    /// Take the dirty lines and pass each one's current level to `sync`.
    pub fn take_dirty_lines(&self, mut sync: impl FnMut(usize, bool)) {
        for word in 0..GUEST_IRQ_WORDS {
            if LINE_MASK[word] == 0 {
                continue;
            }
            let mut dirty = self.line_dirty[word].swap(0, Ordering::AcqRel);
            if dirty == 0 {
                continue;
            }
            let levels = self.pending[word].load(Ordering::Acquire);
            while dirty != 0 {
                let bit = dirty.trailing_zeros() as usize;
                dirty &= dirty - 1;
                sync(word * GUEST_IRQ_WORD_BITS + bit, levels & (1 << bit) != 0);
            }
        }
    }

    pub fn set(&self, irq_id: usize) -> bool {
        if irq_id >= GUEST_IRQ_NR {
            return false;
        }
        self.pending[irq_id / GUEST_IRQ_WORD_BITS]
            .fetch_or(1u64 << (irq_id % GUEST_IRQ_WORD_BITS), Ordering::Release);
        true
    }

    /// Claim pending IRQ numbers and let the architecture install each one.
    /// Returns `true` when `install` refused one; it is put back and a retry
    /// (e.g. GIC maintenance) must stay armed.
    //
    // The architecture owns the number space and the install path.
    pub fn flush_pending_with(&self, mut install: impl FnMut(usize) -> bool) -> bool {
        for word_index in 0..self.pending.len() {
            while let Some(irq_id) = self.claim_pending_irq(word_index) {
                if install(irq_id) {
                    continue;
                }
                self.set(irq_id);
                return true;
            }
        }
        false
    }

    /// Atomically claim the lowest pending IRQ number in one bitmap word.
    pub fn claim_pending_irq(&self, word_index: usize) -> Option<usize> {
        let word = self.pending.get(word_index)?;
        let edges = !LINE_MASK[word_index];
        loop {
            let old = word.load(Ordering::Acquire);
            if old & edges == 0 {
                return None;
            }
            let bit = (old & edges).trailing_zeros() as usize;
            let mask = 1u64 << bit;
            if word
                .compare_exchange(old, old & !mask, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some(word_index * GUEST_IRQ_WORD_BITS + bit);
            }
        }
    }

    pub fn clear(&self, irq_id: usize) -> bool {
        if irq_id >= GUEST_IRQ_NR {
            return false;
        }
        self.pending[irq_id / GUEST_IRQ_WORD_BITS]
            .fetch_and(!(1u64 << (irq_id % GUEST_IRQ_WORD_BITS)), Ordering::AcqRel);
        true
    }

    pub fn clear_all(&self) {
        for word in 0..GUEST_IRQ_WORDS {
            let old = self.pending[word].swap(0, Ordering::AcqRel);
            if old & LINE_MASK[word] != 0 {
                self.line_dirty[word].fetch_or(old & LINE_MASK[word], Ordering::Release);
            }
        }
    }

    pub fn has_pending_edges(&self) -> bool {
        (0..GUEST_IRQ_WORDS)
            .any(|word| self.pending[word].load(Ordering::Acquire) & !LINE_MASK[word] != 0)
    }

    pub fn pending_mask(&self, first_irq: usize, count: usize) -> u64 {
        if count == 0 || count > GUEST_IRQ_WORD_BITS {
            return 0;
        }
        let word = first_irq / GUEST_IRQ_WORD_BITS;
        let offset = first_irq % GUEST_IRQ_WORD_BITS;
        let mut value = self
            .pending
            .get(word)
            .map(|pending| pending.load(Ordering::Acquire) >> offset)
            .unwrap_or(0);
        if offset != 0 {
            value |= self
                .pending
                .get(word + 1)
                .map(|pending| pending.load(Ordering::Acquire) << (GUEST_IRQ_WORD_BITS - offset))
                .unwrap_or(0);
        }
        if count == GUEST_IRQ_WORD_BITS {
            value
        } else {
            value & ((1u64 << count) - 1)
        }
    }
}

/// Enter blocked state and wait until another CPU resumes it.
#[allow(unused)]
pub fn vcpu_suspend() {
    info!("cpu {} suspending...", this_cpu_data().id);
    this_cpu_data().vcpu.state.store(VcpuState::Blocked);
    loop {
        // TODO: use wfi to optimize the loop
        if !this_cpu_data().vcpu.state.is_blocked() {
            break;
        }
        core::hint::spin_loop();
    }
    // Remote sets `Ready` to leave Blocked; this hart then marks itself `Running` again.
    this_cpu_data().vcpu.set_running();
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
            if get_cpu_data(target_cpu_id).vcpu.state.is_blocked() {
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
            .vcpu
            .state
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
