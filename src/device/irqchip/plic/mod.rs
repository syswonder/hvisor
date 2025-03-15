use crate::config::root_zone_config;
use crate::memory::GuestPhysAddr;
use crate::platform::__board::*;
use crate::zone::Zone;
use crate::{arch::cpu::ArchCpu, percpu::this_cpu_data};
use riscv::register::hvip;
use riscv_decode::Instruction;
use spin::{Once, RwLock};
pub fn primary_init_early() {
    let root_config = root_zone_config();
    init_plic(
        root_config.arch_config.plic_base as usize,
        root_config.arch_config.plic_size as usize,
    );
}
pub fn primary_init_late() {
    //nothing to do
}
pub fn percpu_init() {
    //nothing to do
}
pub fn inject_irq(_irq: usize, _is_hardware: bool) {
    //nothing to do
}
pub static PLIC: Once<RwLock<Plic>> = Once::new();
pub fn host_plic<'a>() -> &'a RwLock<Plic> {
    PLIC.get().expect("Uninitialized hypervisor plic!")
}
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
    pub fn set_priority(&self, irq_id: usize, priority: u32) {
        let addr = self.base + PLIC_PRIORITY_BASE + irq_id * 4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, priority);
        }
    }
    pub fn read_enable(&self, context: usize, irq_base: usize) -> u32 {
        let addr = self.base + PLIC_ENABLE_BASE + context * 0x80 + irq_base;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_enable(&self, context: usize, irq_base: usize, value: u32) {
        let addr = self.base + PLIC_ENABLE_BASE + context * 0x80 + irq_base;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }
    pub fn set_threshold(&self, context: usize, value: u32) {
        let addr = self.base + PLIC_GLOBAL_SIZE + context * 0x1000;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, value);
        }
    }
    ///TODO:move to vplic
    pub fn emul_claim(&self, context: usize) -> u32 {
        self.claim_complete[context]
    }
    pub fn emul_complete(&mut self, context: usize, irq_id: u32) {
        let addr = self.base + PLIC_GLOBAL_SIZE + 0x1000 * context + 0x4;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, irq_id as u32);
        }

        self.claim_complete[context] = 0;
        unsafe {
            hvip::clear_vseip();
        }
    }
}
pub fn vplic_global_emul_handler(
    current_cpu: &mut ArchCpu,
    addr: GuestPhysAddr,
    inst: Instruction,
) {
    //TODO:check irq id for vm
    let host_plic = host_plic();
    let offset = addr.wrapping_sub(host_plic.read().base);
    // priority/pending/enable
    if offset >= PLIC_PRIORITY_BASE && offset < PLIC_ENABLE_BASE {
        // priority/pending
        match inst {
            Instruction::Sw(i) => {
                // guest write irq priority
                //TODO:check irq id for vm
                let irq_id = offset / 4;
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_plic.write().set_priority(irq_id, value);
                debug!(
                    "PLIC set priority write addr@{:#x} irq id {} valuse{:#x}",
                    addr, irq_id, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= PLIC_ENABLE_BASE && offset < PLIC_GLOBAL_SIZE {
        //enable
        match inst {
            Instruction::Lw(i) => {
                // guest read
                let vcontext = (offset - 0x002000) / 0x80;
                let first_cpu = this_cpu_data()
                    .zone
                    .as_ref()
                    .unwrap()
                    .read()
                    .cpu_set
                    .first_cpu()
                    .unwrap();
                let context = vcontext + first_cpu * 2;
                let irq_base = (offset - 0x002000) % 0x80;
                let value = host_plic.read().read_enable(context, irq_base);
                current_cpu.x[i.rd() as usize] = value as usize;
                debug!(
                    "PLIC set enable read addr@{:#x} -> context {}=>{}  irq_base {}~{} value {:#x}",
                    addr,
                    vcontext,
                    context,
                    irq_base * 8,
                    irq_base * 8 + 31,
                    value
                );
            }
            Instruction::Sw(i) => {
                // guest write irq enable
                let vcontext = (offset - 0x002000) / 0x80;
                let first_cpu = this_cpu_data()
                    .zone
                    .as_ref()
                    .unwrap()
                    .read()
                    .cpu_set
                    .first_cpu()
                    .unwrap();
                let context = vcontext + first_cpu * 2;
                let irq_base = (offset - 0x002000) % 0x80;
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_plic.write().set_enable(context, irq_base, value);

                debug!(
                    "PLIC set enable write addr@{:#x} -> context{}=>{}  irq_base {}~{} value {:#x}",
                    addr,
                    vcontext,
                    context,
                    irq_base * 8,
                    irq_base * 8 + 31,
                    value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}
pub fn vplic_hart_emul_handler(current_cpu: &mut ArchCpu, addr: GuestPhysAddr, inst: Instruction) {
    trace!("handle PLIC access addr@{:#x}", addr);
    let host_plic = host_plic();
    let offset = addr.wrapping_sub(host_plic.read().base);
    // threshold/claim/complete
    if offset >= PLIC_GLOBAL_SIZE && offset < PLIC_TOTAL_SIZE {
        let vcontext = (offset - PLIC_GLOBAL_SIZE) / 0x1000;
        let first_cpu = this_cpu_data()
            .zone
            .as_ref()
            .unwrap()
            .read()
            .cpu_set
            .first_cpu()
            .unwrap();
        let context = vcontext + first_cpu * 2;
        let index = (offset - PLIC_GLOBAL_SIZE) & 0xfff;
        if index == 0 {
            // threshold
            match inst {
                Instruction::Sw(i) => {
                    // guest write threshold register to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    host_plic.write().set_threshold(context, value);
                    debug!(
                        "PLIC set threshold write addr@{:#x} context{} -> {:#x}",
                        addr, context, value
                    );
                }
                _ => panic!("Unexpected instruction threshold {:?}", inst),
            }
        } else if index == 0x4 {
            // claim/complete
            // htracking!("claim/complete");
            match inst {
                Instruction::Lw(i) => {
                    // guest read claim from plic core
                    current_cpu.x[i.rd() as usize] = host_plic.read().emul_claim(context) as usize;
                    debug!(
                        "PLIC claim read addr@{:#x} context{} -> {:#x}",
                        addr,
                        context,
                        host_plic.read().claim_complete[context]
                    );
                }
                Instruction::Sw(i) => {
                    // guest write complete to plic core
                    let value = current_cpu.x[i.rs2() as usize] as u32;
                    host_plic.write().emul_complete(context, value);
                    // todo: guest pa -> host pa
                    debug!(
                        "PLIC complete write addr@:{:#x} context {} -> {:#x}",
                        addr, context, value
                    );
                }
                _ => panic!("Unexpected instruction claim/complete {:?}", inst),
            }
        } else {
            panic!("Invalid address: {:#x}", addr);
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}
pub fn init_plic(plic_base: usize, plic_size: usize) {
    let plic = Plic::new(plic_base, plic_size);
    PLIC.call_once(|| RwLock::new(plic));
}
impl Zone {
    pub fn arch_irqchip_reset(&self) {
        //TODO
    }
}
