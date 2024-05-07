#![allow(dead_code)]
use crate::consts::{DTB_IPA, INVALID_ADDRESS, PAGE_SIZE};
use crate::device::virtio_trampoline::{HVISOR_DEVICE, MAX_DEVS, MAX_REQ, VIRTIO_IRQS};
use crate::error::HvResult;
use crate::percpu::{get_cpu_data, PerCpu};
use crate::zone::{find_zone, is_this_root_zone, remove_zone, root_zone, zone_create};

use crate::event::{send_event, IPI_EVENT_SHUTDOWN, IPI_EVENT_VIRTIO_INJECT_IRQ, IPI_EVENT_WAKEUP};
use alloc::sync::Arc;
use core::convert::TryFrom;
use core::sync::atomic::{fence, Ordering};

use numeric_enum_macro::numeric_enum;

#[repr(C)]
#[derive(Debug)]
pub struct ZoneInfo {
    id: u64,
    image_phys_addr: u64,
    dtb_phys_addr: u64,
}

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum HyperCallCode {
        HvVirtioInit = 9,
        HvVirtioInjectIrq = 10,
        HvZoneStart = 11,
        HvZoneShutdown = 12,
    }
}
pub const SGI_IPI_ID: u64 = 7;

pub type HyperCallResult = HvResult<usize>;

pub struct HyperCall<'a> {
    cpu_data: &'a mut PerCpu,
}

impl<'a> HyperCall<'a> {
    pub fn new(cpu_data: &'a mut PerCpu) -> Self {
        Self { cpu_data }
    }

    pub fn hypercall(&mut self, code: u64, arg0: u64, _arg1: u64) -> HyperCallResult {
        let code = match HyperCallCode::try_from(code) {
            Ok(code) => code,
            Err(_) => {
                warn!("hypercall id={} unsupported!", code);
                return Ok(0);
            }
        };
        unsafe {
            match code {
                HyperCallCode::HvVirtioInit => self.hv_virtio_init(arg0),
                HyperCallCode::HvVirtioInjectIrq => self.hv_virtio_inject_irq(),
                HyperCallCode::HvZoneStart => self.hv_zone_start(&*(arg0 as *const ZoneInfo)),
                HyperCallCode::HvZoneShutdown => self.hv_zone_shutdown(arg0),
            }
        }
    }

    // only root zone calls the function and set virtio shared region between el1 and el2.
    fn hv_virtio_init(&mut self, shared_region_addr: u64) -> HyperCallResult {
        info!(
            "handle hvc init virtio, shared_region_addr = {:#x?}",
            shared_region_addr
        );
        let zone = self.cpu_data.zone.clone().unwrap();
        if !Arc::ptr_eq(&zone, &root_zone()) {
            return hv_result_err!(EPERM, "Init virtio over non-root zones: unsupported!");
        }
        let shared_region_addr_pa = shared_region_addr as usize;
        assert!(shared_region_addr_pa % PAGE_SIZE == 0);
        // let offset = shared_region_addr_pa & (PAGE_SIZE - 1);
        // memory::hv_page_table()
        // 	.write()
        // 	.insert(MemoryRegion::new_with_offset_mapper(
        // 		HVISOR_DEVICE_REGION_BASE,
        // 		shared_region_addr as _,
        // 		PAGE_SIZE,
        // 		MemFlags::READ | MemFlags::WRITE,
        // 	))?;
        // TODO: flush tlb
        HVISOR_DEVICE
            .lock()
            .set_base_addr(shared_region_addr_pa as _);
        info!("hvisor device region base is {:#x?}", shared_region_addr_pa);
        HyperCallResult::Ok(0)
    }

    // Inject virtio device's irq to non root when a virtio device finishes one IO request. Only root zone calls.
    fn hv_virtio_inject_irq(&self) -> HyperCallResult {
        if !is_this_root_zone() {
            return hv_result_err!(
                EPERM,
                "Virtio send irq operation over non-root zones: unsupported!"
            );
        }
        let dev = HVISOR_DEVICE.lock();
        let mut map_irq = VIRTIO_IRQS.lock();
        let region = dev.region();
        while !dev.is_res_list_empty() {
            let res_front = region.res_front as usize;
            let irq_id = region.res_list[res_front].irq_id as u64;
            let target_zone = region.res_list[res_front].target_zone;
            // TODO: only the first cpu receives the irq, is that reasonable???
            let target_cpu = find_zone(target_zone as _)
                .unwrap()
                .read()
                .cpu_set
                .first_cpu()
                .unwrap();
            let irq_list = map_irq.entry(target_cpu).or_insert([0; MAX_DEVS + 1]);
            if !irq_list[1..=irq_list[0] as usize].contains(&irq_id) {
                let len = irq_list[0] as usize;
                assert!(len + 1 < MAX_DEVS);
                irq_list[len + 1] = irq_id;
                irq_list[0] += 1;
                send_event(
                    target_cpu as _,
                    SGI_IPI_ID as _,
                    IPI_EVENT_VIRTIO_INJECT_IRQ,
                );
            }

            fence(Ordering::SeqCst);
            region.res_front = (region.res_front + 1) & (MAX_REQ - 1);
            fence(Ordering::SeqCst);
        }
        drop(dev);
        HyperCallResult::Ok(0)
    }

    pub fn hv_zone_start(&mut self, zone_info: &ZoneInfo) -> HyperCallResult {
        info!("handle hvc zone start");
        let zone = zone_create(zone_info.id as _, zone_info.dtb_phys_addr as _, DTB_IPA)?;
        let boot_cpu = zone.read().cpu_set.first_cpu().unwrap();

        let target_data = get_cpu_data(boot_cpu as _);
        let _lock = target_data.ctrl_lock.lock();

        if !target_data.arch_cpu.psci_on {
            send_event(boot_cpu, SGI_IPI_ID as _, IPI_EVENT_WAKEUP);
        } else {
            error!("hv_zone_start: cpu {} already on", boot_cpu);
            return hv_result_err!(EBUSY);
        };
        drop(_lock);
        HyperCallResult::Ok(0)
    }

    fn hv_zone_shutdown(&mut self, zone_id: u64) -> HyperCallResult {
        info!("handle hvc zone shutdown, id={}", zone_id);
        if zone_id == 0 {
            return hv_result_err!(EINVAL);
        }
        let zone = match find_zone(zone_id as _) {
            Some(zone) => zone,
            _ => return hv_result_err!(EEXIST),
        };
        let zone_r = zone.read();

        // // return zone's cpus to root_zone
        zone_r.cpu_set.iter().for_each(|cpu_id| {
            let _lock = get_cpu_data(cpu_id).ctrl_lock.lock();
            get_cpu_data(cpu_id).zone = None;
            get_cpu_data(cpu_id).cpu_on_entry = INVALID_ADDRESS;
            send_event(cpu_id, SGI_IPI_ID as _, IPI_EVENT_SHUTDOWN);
        });

        zone_r.arch_irqchip_reset();

        drop(zone_r);
        drop(zone);
        remove_zone(zone_id as _);

        HyperCallResult::Ok(0)
    }
}
