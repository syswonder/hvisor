use crate::{
    arch::cpu::this_cpu_id,
    config::CONFIG_MAGIC_VERSION,
    device::virtio_trampoline::MAX_DEVS,
    hypercall::{HyperCall, HyperCallResult},
    percpu::this_zone,
    zone::{Zone, ZoneInfo},
};
use spin::RwLock;

impl<'a> HyperCall<'a> {
    pub fn hv_ivc_info(&mut self, ivc_info_ipa: u64) -> HyperCallResult {
        warn!("hv_ivc_info is not implemented for x86_64");
        HyperCallResult::Ok(0)
    }

    pub fn wait_for_interrupt(&mut self, irq_list: &mut [u64; MAX_DEVS + 1]) {
        trace!("wait_for_interrupt is not need for x86_64");
    }

    pub fn hv_get_real_pa(&mut self, config_addr: u64) -> u64 {
        unsafe {
            this_zone()
                .read()
                .gpm
                .page_table_query(config_addr as _)
                .unwrap()
                .0 as _
        }
    }

    pub fn hv_zone_config_check(&self, magic_version: *mut u64) -> HyperCallResult {
        let magic_version = unsafe {
            this_zone()
                .read()
                .gpm
                .page_table_query(magic_version as usize)
                .unwrap()
                .0 as *mut u64
        };
        unsafe {
            *magic_version = CONFIG_MAGIC_VERSION as _;
        }
        debug!(
            "hv_zone_config_check: finished writing current magic version ({:#x})",
            CONFIG_MAGIC_VERSION
        );
        HyperCallResult::Ok(0)
    }

    pub fn check_cpu_id(&self) {
        let cpuid = this_cpu_id();
        trace!("CPU ID: {} Start Zone", cpuid);
    }

    pub fn hv_virtio_get_irq(&self, virtio_irq: *mut u32) -> HyperCallResult {
        let virtio_irq = unsafe {
            this_zone()
                .read()
                .gpm
                .page_table_query(virtio_irq as usize)
                .unwrap()
                .0 as *mut u32
        };
        unsafe {
            (*virtio_irq) = crate::device::virtio_trampoline::IRQ_WAKEUP_VIRTIO_DEVICE as _;
        };
        HyperCallResult::Ok(0)
    }
}
