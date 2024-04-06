#![allow(dead_code)]
use crate::arch::ipi::arch_send_event;
use crate::consts::DTB_IPA;
use crate::error::HvResult;
use crate::event::IPI_EVENT_WAKEUP;
use crate::percpu::{get_cpu_data, PerCpu};
use crate::zone::zone_create;

use core::convert::TryFrom;
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
        HvZoneStart = 11,
        HvZoneDestroy = 12,
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
                HyperCallCode::HvZoneStart => self.hv_zone_start(&*(arg0 as *const ZoneInfo)),
                HyperCallCode::HvZoneDestroy => self.hv_zone_destroy(arg0),
            }
        }
    }

    fn hypervisor_disable(&mut self) -> HyperCallResult {
        todo!();
        // let cpus = PerCpu::activated_cpus();

        // static TRY_DISABLE_CPUS: AtomicU32 = AtomicU32::new(0);
        // TRY_DISABLE_CPUS.fetch_add(1, Ordering::SeqCst);
        // while TRY_DISABLE_CPUS.load(Ordering::Acquire) < cpus {
        //     core::hint::spin_loop();
        // }
        // info!("Handle hvc disable");
        // self.cpu_data.deactivate_vmm()?;
        // unreachable!()
    }

    pub fn hv_zone_start(&mut self, zone_info: &ZoneInfo) -> HyperCallResult {
        info!("handle hvc zone start");
        let zone = zone_create(zone_info.id as _, zone_info.dtb_phys_addr as _, DTB_IPA)?;
        let boot_cpu = zone.read().cpu_set.first_cpu().unwrap();

        let target_data = get_cpu_data(boot_cpu as _);
        let _lock = target_data.ctrl_lock.lock();

        if !target_data.arch_cpu.psci_on {
            arch_send_event(boot_cpu as _, SGI_IPI_ID, IPI_EVENT_WAKEUP);
            // target_data.arch_cpu.psci_on = true;
        } else {
            error!("hv_zone_start: cpu {} already on", boot_cpu);
            return hv_result_err!(EBUSY);
        };

        HyperCallResult::Ok(0)
    }

    fn hv_zone_destroy(&mut self, _zone_id: u64) -> HyperCallResult {
        #[cfg(target_arch = "invalid")]
        {
            info!("handle hvc zone destroy");
            let zone = zone_management_prologue(self.cpu_data, zone_id)?;
            let mut zone_w = zone.write();
            let root_zone = root_zone();
            let mut root_zone_w = root_zone.write();
            // return zone's cpus to root_zone
            zone_w.cpu_set.iter().for_each(|cpu_id| {
                park_cpu(cpu_id);
                root_zone_w.cpu_set.set_bit(cpu_id);
                get_cpu_data(cpu_id).zone = Some(root_zone.clone());
            });
            // return loadable ram memory to root_zone
            let mem_regs: Vec<HvMemoryRegion> = zone_w.config().mem_regions().to_vec();
            mem_regs.iter().for_each(|mem| {
                if !(mem.flags.contains(MemFlags::COMMUNICATION)
                    || mem.flags.contains(MemFlags::ROOTSHARED))
                {
                    root_zone_w.mem_region_map_partial(&MemoryRegion::new_with_offset_mapper(
                        mem.phys_start as _,
                        mem.phys_start as _,
                        mem.size as _,
                        mem.flags,
                    ));
                }
            });
            // TODO：arm_zone_dcaches_flush， invalidate zone mems in cache
            zone_w.cpu_set.iter().for_each(|id| {
                get_cpu_data(id).cpu_on_entry = INVALID_ADDRESS;
            });
            drop(root_zone_w);
            zone_w.gicv3_exit();
            zone_w.adjust_irq_mappings();
            drop(zone_w);
            // Drop the zone will destroy zone's MemorySet so that all page tables will free
            drop(zone);
            remove_zone(zone_id as _);
            root_zone.read().resume();
            // TODO: config commit
            info!("zone destroy succeed");
        }
        HyperCallResult::Ok(0)
    }
}
