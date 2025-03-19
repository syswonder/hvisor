use sbi_rt::HartMask;

// arch_send_event
pub fn arch_send_event(cpu_id: u64, _sgi_num: u64) {
    info!("arch_send_event: cpu_id: {}", cpu_id);
    #[cfg(feature = "aclint")]
    crate::device::irqchip::aclint::aclint_send_ipi(cpu_id as usize);
    #[cfg(not(feature = "aclint"))]
    sbi_rt::send_ipi(HartMask::from_mask_base(1 << cpu_id, 0));
}
