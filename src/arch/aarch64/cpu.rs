pub fn cpu_start(hartid: usize, start_addr: usize, opaque: usize) -> _ {
    psci::cpu_on(cpu_id | 0x80000000, virt_to_phys(arch_entry as _) as _, 0).unwrap_or_else(
        |err| {
            if let psci::error::Error::AlreadyOn = err {
            } else {
                panic!("can't wake up cpu {}", cpu_id);
            }
        },
    );
}