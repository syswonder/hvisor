use fdt::Fdt;
use spin::{Once, RwLock};

use crate::platform::qemu_riscv64::PLIC_MAX_CONTEXT;

pub fn irqchip_init_early(host_fdt: &Fdt) {
    let plic_info = host_fdt.find_node("/soc/plic").unwrap();
    init_plic(
        plic_info.reg().unwrap().next().unwrap().starting_address as usize,
        plic_info.reg().unwrap().next().unwrap().size.unwrap(),
    );
}

pub static PLIC: Once<RwLock<Plic>> = Once::new();

pub struct Plic {
    pub base: usize,
    pub size: usize,
    pub claim_complete: [u32; PLIC_MAX_CONTEXT],
}

impl Plic {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
            claim_complete: [0u32; PLIC_MAX_CONTEXT],
        }
    }
}

pub fn init_plic(plic_base: usize, plic_size: usize) {
    let plic = Plic::new(plic_base, plic_size);
    PLIC.call_once(|| RwLock::new(plic));
}
