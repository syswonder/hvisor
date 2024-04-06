use alloc::sync::Arc;
use spin::{Mutex, RwLock};

use crate::arch::cpu::{this_cpu_id, ArchCpu};
use crate::consts::{INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
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
    pub pending_event: Option<usize>,
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
            pending_event: None,
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
        self.arch_cpu.run();
    }

    pub fn entered_cpus() -> u32 {
        ENTERED_CPUS.load(Ordering::Acquire)
    }

    pub fn activate_gpm(&self) {
        unsafe { self.zone.clone().unwrap().read().gpm.activate(); }
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
