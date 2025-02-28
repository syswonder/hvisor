pub fn arch_send_event(cpu_id: u64, _sgi_num: u64) {
    sbi_rt::send_ipi(1 << cpu_id, 0);
}
