use alloc::sync::Arc;
use spin::{Mutex, RwLock};

use crate::arch::cpu::{this_cpu_id, ArchCpu};
use crate::consts::{INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
use crate::error::HvResult;
use crate::memory::addr::VirtAddr;
use crate::zone::Zone;
use crate::ENTERED_CPUS;
use core::fmt::Debug;
use core::sync::atomic::Ordering;

// global_asm!(include_str!("./arch/aarch64/page_table.S"),);

#[repr(C)]
pub struct PerCpu {
    pub id: usize,
    pub cpu_on_entry: usize,
    pub arch_cpu: ArchCpu,
    pub zone: Option<Arc<RwLock<Zone>>>,
    pub ctrl_lock: Mutex<()>,
    pub boot_cpu: bool,
    // percpu stack
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'static mut PerCpu {
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = unsafe { &mut *(vaddr as *mut Self) };
        *ret = PerCpu {
            id: cpu_id,
            cpu_on_entry: INVALID_ADDRESS,
            arch_cpu: ArchCpu::new(cpu_id),
            zone: None,
            ctrl_lock: Mutex::new(()),
            boot_cpu: false,
        };
        #[cfg(target_arch = "riscv64")]
        {
            use crate::arch::csr::CSR_SSCRATCH;
            write_csr!(CSR_SSCRATCH, &ret.arch_cpu as *const _ as usize); //arch cpu pointer
        }
        ret
    }

    pub fn run_vm(&mut self) {
        if !self.boot_cpu {
            info!("CPU{}: Idling the CPU before starting VM...", self.id);
            self.arch_cpu.idle();
        }
        info!("CPU{}: Running virtual machine...", self.id);
        self.arch_cpu.run();
    }

    pub fn entered_cpus() -> u32 {
        ENTERED_CPUS.load(Ordering::Acquire)
    }

    /*should be in vcpu*/
    // pub fn arch_shutdown_self(&mut self) -> HvResult {
    //     /*irqchip reset*/
    //     gicv3_cpu_shutdown();
    //     /* Free the guest */
    //     HCR_EL2.set(0x80000000);
    //     VTCR_EL2.set(0x80000000);
    //     /* Remove stage-2 mappings */
    //     unsafe {
    //         isb();
    //         arm_paging_vcpu_flush_tlbs();
    //     }
    //     /* TLB flush needs the zone's VMID */
    //     VTTBR_EL2.set(0);
    //     /* we will restore the root zone state with the MMU turned off,
    //      * so we need to make sure it has been committed to memory */
    //     todo!();
    //     // unsafe {
    //     //     let page_offset: u64 = todo!();
    //     //     virt2phys_el2(self.guest_reg(), page_offset);
    //     // }
    //     // Ok(())
    // }
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

pub fn check_events() {
    todo!();
    // let cpu_data: &mut PerCpu = this_cpu_data();
    // let mut _lock = Some(cpu_data.ctrl_lock.lock());
    // while cpu_data.need_suspend {
    //     cpu_data.suspended = true;
    //     _lock = None; // release lock here
    //     while cpu_data.need_suspend {}
    //     _lock = Some(cpu_data.ctrl_lock.lock()); // acquire lock again
    // }
    // cpu_data.suspended = false;

    // let mut reset = false;
    // if cpu_data.park {
    //     cpu_data.park = false;
    //     cpu_data.wait_for_poweron = true;
    // } else if cpu_data.reset {
    //     cpu_data.reset = false;
    //     if cpu_data.cpu_on_entry != INVALID_ADDRESS {
    //         cpu_data.wait_for_poweron = false;
    //         reset = true;
    //     } else {
    //         cpu_data.wait_for_poweron = true; // prepare to park
    //     }
    // }
    // drop(_lock);

    // if cpu_data.wait_for_poweron {
    //     info!("check_events: park current cpu");
    //     park_current_cpu();
    // } else if reset {
    //     info!(
    //         "check_events: reset current cpu -> {:#x?}",
    //         cpu_data.cpu_on_entry
    //     );
    //     todo!();
    //     // reset_current_cpu(cpu_data.cpu_on_entry);
    // }
}

// #[allow(unused)]
// pub fn test_cpu_el1() {
//     info!("hello from el2");
//     let mut gpm: MemorySet<Stage2PageTable> = MemorySet::new();
//     info!("set gpm for zone1");
//     gpm.insert(MemoryRegion::new_with_offset_mapper(
//         0x00000000 as GuestPhysAddr,
//         0x7fa00000 as HostPhysAddr,
//         0x00100000 as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::NO_HUGEPAGES,
//     ));
//     gpm.insert(MemoryRegion::new_with_offset_mapper(
//         0x09000000 as GuestPhysAddr,
//         0x09000000 as HostPhysAddr,
//         0x00001000 as usize,
//         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
//     ));
//     unsafe {
//         gpm.activate();
//     }
//     reset_current_cpu();
// }

pub fn park_current_cpu() {
    todo!();
    // let cpu_data = this_cpu_data();
    // let _lock = cpu_data.ctrl_lock.lock();
    // cpu_data.park = false;
    // cpu_data.wait_for_poweron = true;
    // drop(_lock);

    // // reset current cpu -> pc = 0x0 (wfi)
    // PARKING_MEMORY_SET.call_once(|| {
    //     let parking_code: [u8; 8] = [0x7f, 0x20, 0x03, 0xd5, 0xff, 0xff, 0xff, 0x17]; // 1: wfi; b 1b
    //     unsafe {
    //         PARKING_INST_PAGE[..8].copy_from_slice(&parking_code);
    //     }

    //     let mut gpm = MemorySet::<Stage2PageTable>::new();
    //     gpm.insert(MemoryRegion::new_with_offset_mapper(
    //         0 as GuestPhysAddr,
    //         unsafe { &PARKING_INST_PAGE as *const _ as HostPhysAddr - PHYS_VIRT_OFFSET },
    //         PAGE_SIZE,
    //         MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
    //     ))
    //     .unwrap();
    //     gpm
    // });
    // reset_current_cpu(0);
    // unsafe {
    //     PARKING_MEMORY_SET.get().unwrap().activate();
    // }
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
    pub fn from_cpuset_slice(cpu_set: &[u8]) -> Self {
        if cpu_set.len() != 8 {
            todo!("Cpu_set should be 8 bytes!");
        }
        let cpu_set_long: u64 = cpu_set
            .iter()
            .enumerate()
            .fold(0, |acc, (i, x)| acc | (*x as u64) << (i * 8));
        Self::new(cpu_set.len() * 8 - 1, cpu_set_long)
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
