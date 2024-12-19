use x86_64::structures::idt::{Entry, HandlerFunc, InterruptDescriptorTable};

const NUM_INT: usize = 256;

pub struct IdtStruct {
    table: InterruptDescriptorTable,
}

impl IdtStruct {
    pub fn new() -> Self {
        extern "C" {
            #[link_name = "_hyp_trap_vector"]
            static ENTRIES: [extern "C" fn(); NUM_INT];
        }
        let mut idt = Self {
            table: InterruptDescriptorTable::new(),
        };
        let entries = unsafe {
            core::slice::from_raw_parts_mut(
                &mut idt.table as *mut _ as *mut Entry<HandlerFunc>,
                NUM_INT,
            )
        };
        for i in 0..NUM_INT {
            entries[i].set_handler_fn(unsafe { core::mem::transmute(ENTRIES[i]) });
        }
        idt
    }

    pub fn load(&'static self) {
        self.table.load();
    }
}
