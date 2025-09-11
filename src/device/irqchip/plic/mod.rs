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
// Authors: Jingyu Liu <liujingyu24s@ict.ac.cn>
//

pub mod plic;
pub mod vplic;

pub use self::plic::*;
use self::vplic::*;
use crate::arch::zone::HvArchZoneConfig;
use crate::config::root_zone_config;
use crate::config::HvZoneConfig;
use crate::consts::{MAX_CPU_NUM, MAX_ZONE_NUM};
use crate::error::HvResult;
use crate::memory::mmio::MMIOAccess;
use crate::memory::GuestPhysAddr;
use crate::percpu::this_zone;
use crate::platform::__board::*;
use crate::platform::BOARD_PLIC_INTERRUPTS_NUM;
use crate::zone::Zone;
use crate::{arch::cpu::ArchCpu, percpu::this_cpu_data};
use alloc::vec::Vec;
use heapless::FnvIndexMap;
use riscv_decode::Instruction;
use riscv_h::register::hvip;
use spin::Once;

/*
   Due to hvisor is a static partitioning hypervisor.
   The irq is assigned to a specific zone, a zone has its own harts.
   So we assume different harts will don't access the same plic register.
   For physical plic, we don't add lock for it.
*/

// Physical PLIC
pub static PLIC: Once<Plic> = Once::new();
// The MAX_ZONE_NUM should be the power of 2.
static mut VPLIC_MAP: Option<FnvIndexMap<usize, VirtualPLIC, MAX_ZONE_NUM>> = None;

pub fn init_plic(plic_base: usize) {
    PLIC.call_once(|| Plic::new(plic_base));
}

pub fn host_plic<'a>() -> &'a Plic {
    PLIC.get().expect("Uninitialized hypervisor plic!")
}

pub fn primary_init_early() {
    // Init the physical PLIC global part
    let root_config = root_zone_config();
    init_plic(root_config.arch_config.plic_base as usize);
    host_plic().init_global(BOARD_PLIC_INTERRUPTS_NUM, MAX_CPU_NUM * 2);

    unsafe {
        VPLIC_MAP = Some(FnvIndexMap::new());
    }
}

pub fn primary_init_late() {
    info!("PLIC do nothing in primary_init_late");
}

pub fn percpu_init() {
    host_plic().init_per_hart(this_cpu_data().id);
}

pub fn inject_irq(irq: usize, is_hardware: bool) {
    debug!("inject_irq: {} is_hardware: {}", irq, is_hardware);
    let vcontext_id = pcontext_to_vcontext(this_cpu_data().id * 2 + 1);
    this_cpu_data()
        .zone
        .as_ref()
        .unwrap()
        .read()
        .get_vplic()
        .inject_irq(vcontext_id, irq, is_hardware);
}

/// Convert vcontext id to pcontext id.
pub fn vcontext_to_pcontext(vcontext_id: usize) -> usize {
    let pcpu_set = this_cpu_data()
        .zone
        .as_ref()
        .unwrap()
        .read()
        .cpu_set
        .iter()
        .collect::<Vec<_>>();
    let index = vcontext_id / 2;
    // convert to physical hart S-mode
    pcpu_set[index] * 2 + 1
}

/// Convert pcontext id to vcontext id.
pub fn pcontext_to_vcontext(pcontext_id: usize) -> usize {
    // vcpu is the pcpus index of the pcpu_set
    let pcpu_set = this_cpu_data()
        .zone
        .as_ref()
        .unwrap()
        .read()
        .cpu_set
        .iter()
        .collect::<Vec<_>>();
    let pcpu_id = this_cpu_data().id;
    let mut index = 0;
    for (i, &id) in pcpu_set.iter().enumerate() {
        if id == pcpu_id {
            index = i;
            break;
        }
    }
    // convert to virtual hart S-mode
    index * 2 + 1
}

/// handle Zone's plic mmio access.
pub fn vplic_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    let value = this_cpu_data()
        .zone
        .as_ref()
        .unwrap()
        .read()
        .get_vplic()
        .vplic_emul_access(mmio.address, mmio.size, mmio.value, mmio.is_write);
    if !mmio.is_write {
        // read from vplic
        mmio.value = value as usize;
    }
    Ok(())
}

/// Update hart line handler.
pub fn update_hart_line() {
    let pcontext_id = this_cpu_data().id * 2 + 1;
    let vcontext_id = pcontext_to_vcontext(pcontext_id);
    this_cpu_data()
        .zone
        .as_ref()
        .unwrap()
        .read()
        .get_vplic()
        .update_hart_line(vcontext_id);
}

/// Print all keys in the VPLIC_MAP for debugging purposes.
fn print_keys() {
    info!("VPLIC_MAP keys:");
    unsafe {
        if let Some(map) = &VPLIC_MAP {
            for (&key, _) in map.iter() {
                info!("Zone {} in VPLIC_MAP", key);
            }
        }
    }
}

impl Zone {
    /// Initial the virtual PLIC related to thiz Zone.
    pub fn vplic_init(&mut self, config: &HvZoneConfig) {
        // Create a new VirtualPLIC for this Zone.
        unsafe {
            if let Some(map) = &mut VPLIC_MAP {
                if map.contains_key(&self.id) {
                    panic!("VirtualPLIC for Zone {} already exists!", self.id);
                }
                let vplic = vplic::VirtualPLIC::new(
                    config.arch_config.plic_base,
                    BOARD_PLIC_INTERRUPTS_NUM,
                    self.cpu_num * 2,
                );
                // Insert into Map <zone_id, vplic>
                let _ = map.insert(self.id, vplic);
            } else {
                panic!("VPLIC_MAP is not initialized!");
            }
        }
        info!("VirtualPLIC for Zone {} initialized successfully", self.id);
        print_keys();
    }

    pub fn get_vplic(&self) -> &VirtualPLIC {
        unsafe {
            VPLIC_MAP
                .as_ref()
                .expect("VPLIC_MAP is not initialized!")
                .get(&self.id)
                .expect("VirtualPLIC for this Zone does not exist!")
        }
    }

    pub fn arch_irqchip_reset(&self) {
        // We should make sure only one cpu to do this.
        // This func will only be called by one root zone's cpu.
        let host_plic = host_plic();
        let vplic = self.get_vplic();
        for (index, &word) in self.irq_bitmap.iter().enumerate() {
            for bit_position in 0..32 {
                if word & (1 << bit_position) != 0 {
                    let irq_id = index * 32 + bit_position;
                    // Skip the irq_id which is not in HW_IRQS
                    if !HW_IRQS.iter().any(|&x| x == irq_id as _) {
                        continue;
                    }
                    // Reset priority
                    info!("Reset irq_id {} priority to 0", irq_id);
                    host_plic.set_priority(irq_id, 0);
                    // Reset enable
                    self.cpu_set.iter().for_each(|cpuid| {
                        let pcontext_id = cpuid * 2 + 1;
                        info!(
                            "Reset pcontext_id {} irq_id {} enable to false",
                            pcontext_id, irq_id
                        );
                        host_plic.set_enable_num(pcontext_id, irq_id, false);
                    });
                }
            }
        }
        self.cpu_set.iter().for_each(|cpuid| {
            // Reset threshold
            let pcontext_id = cpuid * 2 + 1;
            info!("Reset pcontext_id {} threshold to 0", pcontext_id);
            host_plic.set_threshold(pcontext_id, 0);
            // At the same time, clear the events related to this cpu.
            info!("Clear events related to cpu {}", cpuid);
            crate::event::clear_events(cpuid);
        });

        unsafe {
            if let Some(map) = &mut VPLIC_MAP {
                map.remove(&self.id);
            } else {
                panic!("VPLIC_MAP is not initialized!");
            }
        }
        print_keys();
    }

    fn insert_irq_to_bitmap(&mut self, irq: u32) {
        let irq_index = irq / 32;
        let irq_bit = irq % 32;
        self.irq_bitmap[irq_index as usize] |= 1 << irq_bit;
    }

    /// irq_bitmap_init, and set these irqs' hw bit in vplic to true.
    pub fn irq_bitmap_init(&mut self, irqs: &[u32]) {
        // insert to zone.irq_bitmap
        for irq in irqs {
            let irq_id = *irq;
            // They are hardware interrupts.
            if HW_IRQS.iter().any(|&x| x == irq_id) {
                self.get_vplic().vplic_set_hw(irq_id as usize, true);
                info!("Set irq {} to hardware interrupt", irq_id);
            }
            self.insert_irq_to_bitmap(irq_id);
        }
        // print irq_bitmap
        for (index, &word) in self.irq_bitmap.iter().enumerate() {
            for bit_position in 0..32 {
                if word & (1 << bit_position) != 0 {
                    let interrupt_number = index * 32 + bit_position;
                    info!(
                        "Found interrupt in Zone {} irq_bitmap: {}",
                        self.id, interrupt_number
                    );
                }
            }
        }
    }

    pub fn vplic_mmio_init(&mut self, arch: &HvArchZoneConfig) {
        if arch.plic_base == 0 {
            panic!("vplic_mmio_init: plic_base is null");
        }
        self.mmio_region_register(arch.plic_base, arch.plic_size, vplic_handler, 0);
    }
}
