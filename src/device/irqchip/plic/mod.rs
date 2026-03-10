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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>
//

#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_mut)]
#![deny(unused)]

mod plic;
mod vplic;

use crate::arch::cpu::this_cpu_id;
use crate::arch::zone::HvArchZoneConfig;
use crate::config::HvZoneConfig;
use crate::config::{BitmapWord, CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD};
use crate::consts::MAX_CPU_NUM;
use crate::cpu_data::this_cpu_data;
use crate::error::HvResult;
use crate::memory::mmio::MMIOAccess;
use crate::platform::*;
use crate::zone::Zone;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use plic::*;
use spin::{Mutex, Once};
use vplic::*;

/*
    Due to hvisor is a static partitioning hypervisor.
    The irq is assigned to a specific zone, a zone has its own harts.
    So we assume different harts will don't access the same plic register.
    For physical plic, we don't add lock for it.
*/

// Physical PLIC
static PLIC: Once<Plic> = Once::new();
// VPLIC_MAP, one VPLIC per VM
static VPLIC_MAP: Mutex<BTreeMap<usize, Arc<VirtualPLIC>>> = Mutex::new(BTreeMap::new());

pub fn init_plic(plic_base: usize) {
    PLIC.call_once(|| Plic::new(plic_base));
}

pub fn host_plic<'a>() -> &'a Plic {
    PLIC.get().expect("Uninitialized hypervisor plic!")
}

pub fn primary_init_early() {
    // Init the physical PLIC global part
    init_plic(PLIC_BASE);
    host_plic().init_global(
        BOARD_PLIC_INTERRUPTS_NUM,
        MAX_CPU_NUM * NUM_CONTEXTS_PER_HART,
    );
}

pub fn primary_init_late() {
    info!("PLIC do nothing in primary_init_late");
}

pub fn percpu_init() {
    host_plic().init_per_hart(this_cpu_id());
}

pub fn plic_get_hwirq() -> u32 {
    let context_id = this_cpu_id() * NUM_CONTEXTS_PER_HART + 1;
    host_plic().plic_get_hwirq(context_id)
}

pub fn inject_irq(irq: usize, is_hardware: bool) {
    debug!("inject_irq: {} is_hardware: {}", irq, is_hardware);
    let vcontext_id = pcontext_to_vcontext(this_cpu_id() * NUM_CONTEXTS_PER_HART + 1);
    // this_cpu_data()
    //     .zone
    //     .as_ref()
    //     .unwrap()
    //     .read()
    //     .get_vplic()
    //     .inject_irq(vcontext_id, irq, is_hardware);
    let vplic = {
        let zone = this_cpu_data().zone.as_ref().unwrap().read();
        zone.get_vplic()
    };
    // Avoid holding the read lock when calling inject_irq
    vplic.inject_irq(vcontext_id, irq, is_hardware);
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
    let index = vcontext_id / NUM_CONTEXTS_PER_HART;
    // convert to physical hart S-mode
    pcpu_set[index] * NUM_CONTEXTS_PER_HART + 1
}

/// Convert pcontext id to vcontext id.
pub fn pcontext_to_vcontext(_pcontext_id: usize) -> usize {
    // vcpu is the pcpus index of the pcpu_set
    let pcpu_set = this_cpu_data()
        .zone
        .as_ref()
        .unwrap()
        .read()
        .cpu_set
        .iter()
        .collect::<Vec<_>>();
    let pcpu_id = this_cpu_id();
    let mut index = 0;
    for (i, &id) in pcpu_set.iter().enumerate() {
        if id == pcpu_id {
            index = i;
            break;
        }
    }
    // convert to virtual hart S-mode
    index * NUM_CONTEXTS_PER_HART + 1
}

/// handle Zone's plic mmio access.
pub fn vplic_handler(mmio: &mut MMIOAccess, _arg: usize) -> HvResult {
    // let value = this_cpu_data()
    //     .zone
    //     .as_ref()
    //     .unwrap()
    //     .read()
    //     .get_vplic()
    //     .vplic_emul_access(mmio.address, mmio.size, mmio.value, mmio.is_write);
    let vplic = {
        let zone = this_cpu_data().zone.as_ref().unwrap().read();
        zone.get_vplic()
    };
    // Avoid holding the read lock when calling vplic_emul_access
    let value = vplic.vplic_emul_access(mmio.address, mmio.size, mmio.value, mmio.is_write);
    if !mmio.is_write {
        // read from vplic
        mmio.value = value as usize;
    }
    Ok(())
}

/// Update hart line handler.
pub fn update_hart_line() {
    let pcontext_id = this_cpu_id() * NUM_CONTEXTS_PER_HART + 1;
    let vcontext_id = pcontext_to_vcontext(pcontext_id);
    // this_cpu_data()
    //     .zone
    //     .as_ref()
    //     .unwrap()
    //     .read()
    //     .get_vplic()
    //     .update_hart_line(vcontext_id);
    let vplic = {
        let zone = this_cpu_data().zone.as_ref().unwrap().read();
        zone.get_vplic()
    };
    // Avoid holding the read lock when calling update_hart_line
    vplic.update_hart_line(vcontext_id);
}

/// Print all keys in the VPLIC_MAP for debugging purposes.
/// This function acquires the lock internally and is safe to call from outside.
#[allow(unused)]
fn print_keys() {
    let map = VPLIC_MAP.lock();
    print_keys_from_map(&map);
}

/// Helper: print keys from an already-locked map reference.
/// Useful to avoid nested locking when called from within a locked scope.
fn print_keys_from_map(map: &BTreeMap<usize, Arc<VirtualPLIC>>) {
    info!("VPLIC_MAP keys:");
    for (&key, _) in map.iter() {
        info!("    Zone {}'s VPLIC is in VPLIC_MAP", key);
    }
}

impl Zone {
    /// Initial the virtual PLIC related to thiz Zone.
    pub fn vplic_init(&mut self, config: &HvZoneConfig) {
        // Create a new VirtualPLIC for this Zone.
        let mut map = VPLIC_MAP.lock();
        if map.contains_key(&self.id) {
            panic!("VirtualPLIC for Zone {} already exists!", self.id);
        }
        let vplic = vplic::VirtualPLIC::new(
            config.arch_config.plic_base,
            BOARD_PLIC_INTERRUPTS_NUM,
            self.cpu_num * NUM_CONTEXTS_PER_HART,
        );
        // Insert into Map <zone_id, vplic>
        map.insert(self.id, Arc::new(vplic));
        info!("VirtualPLIC for Zone {} initialized successfully", self.id);
        print_keys_from_map(&map);
    }

    pub fn get_vplic(&self) -> Arc<VirtualPLIC> {
        VPLIC_MAP
            .lock()
            .get(&self.id)
            .expect("No vplic exists for current zone.")
            .clone()
    }

    pub fn arch_irqchip_reset(&self) {
        // We should make sure only one cpu to do this.
        // This func will only be called by one root zone's cpu.
        let host_plic = host_plic();
        let _vplic = self.get_vplic();
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
                        let pcontext_id = cpuid * NUM_CONTEXTS_PER_HART + 1;
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
            let pcontext_id = cpuid * NUM_CONTEXTS_PER_HART + 1;
            info!("Reset pcontext_id {} threshold to 0", pcontext_id);
            host_plic.set_threshold(pcontext_id, 0);
            // At the same time, clear the events related to this cpu.
            info!("Clear events related to cpu {}", cpuid);
            crate::event::clear_events(cpuid);
        });

        let mut map = VPLIC_MAP.lock();
        map.remove(&self.id);
        print_keys_from_map(&map);
    }

    fn insert_irq_to_bitmap(&mut self, irq: u32) {
        let irq_index = irq / 32;
        let irq_bit = irq % 32;
        self.irq_bitmap[irq_index as usize] |= 1 << irq_bit;
    }

    /// irq_bitmap_init, and set these irqs' hw bit in vplic to true.
    pub fn irq_bitmap_init(&mut self, irqs_bitmap: &[BitmapWord]) {
        // insert to zone.irq_bitmap
        for i in 0..irqs_bitmap.len() {
            let word = irqs_bitmap[i];

            for j in 0..CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD {
                if ((word >> j) & 1) == 1 {
                    let irq_id = (i * CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD + j) as u32;

                    // They are hardware interrupts.
                    if HW_IRQS.iter().any(|&x| x == irq_id) {
                        self.get_vplic().vplic_set_hw(irq_id as usize, true);
                        info!("Set irq {} to hardware interrupt", irq_id);
                    }

                    self.insert_irq_to_bitmap(irq_id);
                }
            }
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
