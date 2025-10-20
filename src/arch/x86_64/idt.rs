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

use crate::{error::HvResult, zone::this_zone_id};
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use core::u32;
use spin::{Mutex, Once};
use x86_64::structures::idt::{Entry, HandlerFunc, InterruptDescriptorTable};

const VECTOR_CNT: usize = 256;

#[allow(non_snake_case)]
pub mod IdtVector {
    pub const VIRT_IPI_VECTOR: u8 = 0xef;
    pub const APIC_ERROR_VECTOR: u8 = 0xfc;
    pub const APIC_SPURIOUS_VECTOR: u8 = 0xfd;
    pub const APIC_TIMER_VECTOR: u8 = 0xfe;
}

pub struct IdtStruct {
    table: InterruptDescriptorTable,
}

impl IdtStruct {
    pub fn new() -> Self {
        extern "C" {
            #[link_name = "_hyp_trap_vector"]
            static ENTRIES: [extern "C" fn(); VECTOR_CNT];
        }
        let mut idt = Self {
            table: InterruptDescriptorTable::new(),
        };
        let entries = unsafe {
            core::slice::from_raw_parts_mut(
                &mut idt.table as *mut _ as *mut Entry<HandlerFunc>,
                VECTOR_CNT,
            )
        };
        for i in 0..VECTOR_CNT {
            entries[i].set_handler_fn(unsafe { core::mem::transmute(ENTRIES[i]) });
        }
        idt
    }

    pub fn load(&'static self) {
        self.table.load();
    }
}
