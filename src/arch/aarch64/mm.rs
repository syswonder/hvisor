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
use core::sync::atomic::AtomicU32;

use spin::RwLock;

use crate::{arch::Stage2PageTable, consts::MAX_CPU_NUM, memory::MemorySet, wait_for};

use super::sysreg::read_sysreg;

const PARANGE_TABLE: [usize; 6] = [32, 36, 40, 42, 44, 48];
static MIN_PARANGE: RwLock<u64> = RwLock::new(0x7);
static PARANGE_OK_CPUS: AtomicU32 = AtomicU32::new(0);

pub fn arch_setup_parange() {
    let temp_parange = read_sysreg!(id_aa64mmfr0_el1) & 0xf;
    let mut p = MIN_PARANGE.write();
    *p = p.min(temp_parange);
    drop(p);

    PARANGE_OK_CPUS.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    wait_for(|| PARANGE_OK_CPUS.load(core::sync::atomic::Ordering::SeqCst) < MAX_CPU_NUM as _);
}

pub fn get_parange() -> u64 {
    assert!(PARANGE_OK_CPUS.load(core::sync::atomic::Ordering::SeqCst) == MAX_CPU_NUM as _);
    *MIN_PARANGE.read()
}

pub fn get_parange_bits() -> usize {
    assert!(PARANGE_OK_CPUS.load(core::sync::atomic::Ordering::SeqCst) == MAX_CPU_NUM as _);
    PARANGE_TABLE[*MIN_PARANGE.read() as usize]
}

pub fn is_s2_pt_level3() -> bool {
    get_parange_bits() < 44
}

pub fn new_s2_memory_set() -> MemorySet<Stage2PageTable> {
    MemorySet::new(if is_s2_pt_level3() { 3 } else { 4 })
}
