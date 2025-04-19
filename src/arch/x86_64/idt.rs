use crate::{error::HvResult, zone::this_zone_id};
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use core::u32;
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
    static ref ALLOC_VECTORS: Mutex<RemapVectorsUnlocked> = {
        Mutex::new(RemapVectorsUnlocked::new())
    };
}

static REMAP_VECTORS: Once<RemapVectors> = Once::new();

struct RemapVectors {
    inner: Vec<Mutex<RemapVectorsUnlocked>>,
}

impl RemapVectors {
    fn new(max_zones: usize) -> Self {
        let mut vs = vec![];
        for _ in 0..max_zones {
            let v = Mutex::new(RemapVectorsUnlocked::new());
            vs.push(v)
        }
        Self { inner: vs }
    }

    fn get_host_vector(&self, gv: u32, zone_id: usize) -> Option<u8> {
        let mut vectors = self.inner.get(zone_id).unwrap().lock();

        if let Some(&hv) = vectors.gv_to_hv.get(&gv) {
            return Some(hv);
        }

        for hv in IdtVector::ALLOC_START..=IdtVector::ALLOC_END {
            if !vectors.hv_to_gv.contains_key(&hv) {
                vectors.hv_to_gv.insert(hv, gv);
                vectors.gv_to_hv.insert(gv, hv);

                // info!("gv: {:x}, hv: {:x}", gv, hv);
                return Some(hv);
            }
        }

        None
    }

    fn get_guest_vector(&self, hv: u8, zone_id: usize) -> Option<u32> {
        let mut vectors = self.inner.get(zone_id).unwrap().lock();

        if let Some(&gv) = vectors.hv_to_gv.get(&hv) {
            if gv != u32::MAX {
                return Some(gv);
            }
        }

        None
    }

    fn clear_vectors(&self, hv: u8, zone_id: usize) {
        let mut vectors = self.inner.get(zone_id).unwrap().lock();

        if let Some(&gv) = vectors.hv_to_gv.get(&hv) {
            vectors.hv_to_gv.remove_entry(&hv);
            vectors.gv_to_hv.remove_entry(&gv);
        }
    }
}

struct RemapVectorsUnlocked {
    // key: host vector value: guest vector
    hv_to_gv: BTreeMap<u8, u32>,
    // key: guest vector value: host vector
    gv_to_hv: BTreeMap<u32, u8>,
}

impl RemapVectorsUnlocked {
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
    REMAP_VECTORS.get().unwrap().get_host_vector(gv, zone_id)
}

pub fn get_guest_vector(hv: u8, zone_id: usize) -> Option<u32> {
    REMAP_VECTORS.get().unwrap().get_guest_vector(hv, zone_id)
}

pub fn clear_vectors(hv: u8, zone_id: usize) {
    REMAP_VECTORS.get().unwrap().clear_vectors(hv, zone_id);
}

pub fn init(max_zones: usize) {
    REMAP_VECTORS.call_once(|| RemapVectors::new(max_zones));
}
