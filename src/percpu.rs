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
use alloc::vec::Vec;
use spin::{Mutex, RwLock};

use crate::arch::cpu::{this_cpu_id, ArchCpu};
use crate::consts::{INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::memory::addr::VirtAddr;
use crate::zone::Zone;
use crate::{arch, ENTERED_CPUS};
use core::fmt::Debug;
use core::sync::atomic::Ordering;

// global_asm!(include_str!("./arch/aarch64/page_table.S"),);

#[repr(C)]
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub dtb_ipa: usize,
    pub arch_cpu: ArchCpu,
    pub zone: Option<Arc<RwLock<Zone>>>,
    pub ctrl_lock: Mutex<()>,
    pub boot_cpu: bool,
    // percpu stack
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'static mut PerCpu {
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = vaddr as *mut Self;
        unsafe {
            ret.write_volatile(PerCpu {
                id: cpu_id,
                cpu_on_entry: INVALID_ADDRESS,
                dtb_ipa: INVALID_ADDRESS,
                arch_cpu: ArchCpu::new(cpu_id),
                zone: None,
                ctrl_lock: Mutex::new(()),
                boot_cpu: false,
            })
        };
        #[cfg(target_arch = "riscv64")]
        {
            use crate::arch::csr::{write_csr, CSR_SSCRATCH};
            write_csr!(
                CSR_SSCRATCH,
                &ret.as_mut().unwrap().arch_cpu as *const _ as usize
            ); //arch cpu pointer
        }
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
            self.zone.clone().unwrap().read().gpm.activate();
        }
    }
}

pub fn get_cpu_data<'a>(cpu_id: usize) -> &'a mut PerCpu {
    let cpu_data: usize = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
    unsafe { &mut *(cpu_data as *mut PerCpu) }
}

pub fn this_cpu_data<'a>() -> &'a mut PerCpu {
    get_cpu_data(this_cpu_id())
}

#[allow(unused)]
pub fn this_zone() -> Arc<RwLock<Zone>> {
    this_cpu_data().zone.clone().unwrap()
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