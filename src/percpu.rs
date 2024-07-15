use alloc::sync::Arc;
use spin::{Mutex, RwLock};

use crate::arch::cpu::{this_cpu_id, ArchCpu};
use crate::consts::{DTB_IPA, INVALID_ADDRESS, PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
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
    pub opaque: usize,
    // percpu stack
}

impl PerCpu {
    pub fn new<'a>(cpu_id: usize) -> &'static mut PerCpu {
        let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
        let ret = unsafe { &mut *(vaddr as *mut Self) };
        #[cfg(target_arch = "loongarch64")]
        {
            // let _tmp = PerCpu {
            //     id: cpu_id,
            //     cpu_on_entry: INVALID_ADDRESS,
            //     arch_cpu: ArchCpu::new(cpu_id),
            //     zone: None,
            //     ctrl_lock: Mutex::new(()),
            //     boot_cpu: false,
            //     opaque: DTB_IPA,
            // };
            println!("percpu: setting ret");
            // *ret = _tmp; // cannot run on board? why?
            (*ret).id = cpu_id;
            (*ret).cpu_on_entry = INVALID_ADDRESS;
            (*ret).arch_cpu = ArchCpu::new(cpu_id);
            // (*ret).zone = None;
            (*ret).ctrl_lock = Mutex::new(());
            (*ret).boot_cpu = false;
            (*ret).opaque = DTB_IPA;
            println!("percpu: setting ret done, dump: ");
            println!("percpu: ret.id: {:#x}", (*ret).id);
            println!("percpu: ret.cpu_on_entry: {:#x}", (*ret).cpu_on_entry);
            println!("percpu: ret.arch_cpu: {:?}", (*ret).arch_cpu);
            println!("percpu: ret.ctrl_lock: {:?}", (*ret).ctrl_lock);
            println!("percpu: ret.boot_cpu: {:?}", (*ret).boot_cpu);
            println!("percpu: ret.opaque: {:#x}", (*ret).opaque);
        }
        #[cfg(not(target_arch = "loongarch64"))]
        {
            *ret = PerCpu {
                id: cpu_id,
                cpu_on_entry: INVALID_ADDRESS,
                arch_cpu: ArchCpu::new(cpu_id),
                zone: None,
                ctrl_lock: Mutex::new(()),
                boot_cpu: false,
                opaque: DTB_IPA,
            };
        }

        #[cfg(target_arch = "riscv64")]
        {
            use crate::arch::csr::{write_csr, CSR_SSCRATCH};
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
