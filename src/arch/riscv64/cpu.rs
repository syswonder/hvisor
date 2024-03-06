use core::arch::asm;

pub fn this_cpu_id() -> u64 {
    let hart_id: u64;

    unsafe {
        asm!("csrr {}, mhartid", out(reg) hart_id);
    }

    hart_id
}
