pub fn arch_send_event(cpu_id: u64, sgi_num: u64) {
    info!("arch_send_event");
    sbi_rt::send_ipi(1 << cpu_id, 0);
}
