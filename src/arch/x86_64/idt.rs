use core::u32;

use crate::{error::HvResult, zone::this_zone_id};
use alloc::collections::btree_map::BTreeMap;
use spin::{Mutex, Once};
use x86_64::structures::idt::{Entry, HandlerFunc, InterruptDescriptorTable};

const VECTOR_CNT: usize = 256;

#[allow(non_snake_case)]
pub mod IdtVector {
    pub const ALLOC_START: u8 = 0x20;
    pub const ALLOC_END: u8 = 0xdf;

    pub const VIRT_IPI_VECTOR: u8 = 0xe0;
    pub const APIC_TIMER_VECTOR: u8 = 0xf0;
    pub const APIC_SPURIOUS_VECTOR: u8 = 0xf1;
    pub const APIC_ERROR_VECTOR: u8 = 0xf2;
}

lazy_static::lazy_static! {
    static ref ALLOC_VECTORS: Mutex<AllocVectors> = {
        Mutex::new(AllocVectors::new())
    };
}

struct AllocVectors {
    // key: (zone_id, host vector) value: guest vector
    hv_to_gv: BTreeMap<(usize, u8), u32>,
    // key: (zone_id, guest vector) value: host vector
    gv_to_hv: BTreeMap<(usize, u32), u8>,
}

impl AllocVectors {
    fn new() -> Self {
        Self {
            hv_to_gv: BTreeMap::new(),
            gv_to_hv: BTreeMap::new(),
        }
    }
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

pub fn get_host_vector(gv: u32, zone_id: usize) -> Option<u8> {
    let mut alloc_vectors = ALLOC_VECTORS.lock();

    if let Some(&hv) = alloc_vectors.gv_to_hv.get(&(zone_id, gv)) {
        return Some(hv);
    }

    for hv in IdtVector::ALLOC_START..=IdtVector::ALLOC_END {
        if !alloc_vectors.hv_to_gv.contains_key(&(zone_id, hv)) {
            alloc_vectors.hv_to_gv.insert((zone_id, hv), gv);
            alloc_vectors.gv_to_hv.insert((zone_id, gv), hv);

            // info!("gv: {:x}, hv: {:x}", gv, hv);

            return Some(hv);
        }
    }

    None
}

pub fn get_guest_vector(hv: u8, zone_id: usize) -> Option<u32> {
    let alloc_vectors = ALLOC_VECTORS.lock();

    if let Some(&gv) = alloc_vectors.hv_to_gv.get(&(zone_id, hv)) {
        if gv != u32::MAX {
            return Some(gv);
        }
    }

    None
}

pub fn clear_vectors(hv: u8, zone_id: usize) {
    let mut alloc_vectors = ALLOC_VECTORS.lock();

    if let Some(&gv) = alloc_vectors.hv_to_gv.get(&(zone_id, hv)) {
        alloc_vectors.hv_to_gv.remove_entry(&(zone_id, hv));
        alloc_vectors.gv_to_hv.remove_entry(&(zone_id, gv));
    }
}
