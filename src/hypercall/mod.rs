#![allow(dead_code)]
use crate::error::HvResult;
use crate::percpu::PerCpu;
use crate::zone::find_zone_by_id;
use core::convert::TryFrom;
use core::sync::atomic::{AtomicU32, Ordering};
use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u64)]
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum HyperCallCode {
        HypervisorDisable = 0,
        HypervisorZoneCreate = 1,
        HypervisorZoneStart = 2,
        HypervisorZoneSetLoadable = 3,
        HypervisorZoneDestroy = 4,
    }
}

pub const SGI_INJECT_ID: u64 = 0;
pub const SGI_EVENT_ID: u64 = 15;
pub const SGI_RESUME_ID: u64 = 14;
pub const COMM_REGION_ABI_REVISION: u16 = 1;

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
        match code {
            HyperCallCode::HypervisorDisable => self.hypervisor_disable(),
            HyperCallCode::HypervisorZoneCreate => self.hypervisor_zone_create(arg0),
            HyperCallCode::HypervisorZoneSetLoadable => self.hypervisor_zone_set_loadable(arg0),
            HyperCallCode::HypervisorZoneStart => self.hypervisor_zone_start(arg0),
            HyperCallCode::HypervisorZoneDestroy => self.hypervisor_zone_destroy(arg0),
        }
    }

    fn hypervisor_disable(&mut self) -> HyperCallResult {
        let cpus = PerCpu::activated_cpus();

        static TRY_DISABLE_CPUS: AtomicU32 = AtomicU32::new(0);
        TRY_DISABLE_CPUS.fetch_add(1, Ordering::SeqCst);
        while TRY_DISABLE_CPUS.load(Ordering::Acquire) < cpus {
            core::hint::spin_loop();
        }
        info!("Handle hvc disable");
        self.cpu_data.deactivate_vmm()?;
        unreachable!()
    }

    fn hypervisor_zone_create(&mut self, config_address: u64) -> HyperCallResult {
        todo!();
        // info!(
        //     "handle hvc zone create, config_address = {:#x?}",
        //     config_address
        // );

        // let zone = self.cpu_data.zone.clone().unwrap();
        // if !Arc::ptr_eq(&zone, &root_zone()) {
        //     return hv_result_err!(EPERM, "Creation over non-root zones: unsupported!");
        // }
        // info!("prepare to suspend root_zone");

        // let root_zone = root_zone().clone();
        // root_zone.read().suspend();

        // // todo: 检查新zone是否和已有zone同id或同名
        // let config_address = zone.write().gpm_query(config_address as _);

        // let cfg_pages_offs = config_address as usize & (PAGE_SIZE - 1);
        // todo!();
        // let cfg_mapping = memory::hv_page_table().write().map_temporary(
        //     align_down(config_address),
        //     align_up(cfg_pages_offs + size_of::<HvZoneDesc>()),
        //     MemFlags::READ,
        // )?;

        // let desc: &HvZoneDesc = unsafe {
        //     ((cfg_mapping + cfg_pages_offs) as *const HvZoneDesc)
        //         .as_ref()
        //         .unwrap()
        // };

        // do_zone_create(desc)?;

        // info!("zone create done!");
        // HyperCallResult::Ok(0)
    }

    fn hypervisor_zone_set_loadable(&mut self, zone_id: u64) -> HyperCallResult {
        todo!();
        // info!("handle hvc zone set loadable");
        // let zone = zone_management_prologue(self.cpu_data, zone_id)?;
        // let mut zone_w = zone.write();
        // if zone_w.loadable {
        //     root_zone().read().resume();
        //     return HyperCallResult::Ok(0);
        // }

        // zone_w.cpu_set.iter().for_each(|cpu_id| park_cpu(cpu_id));
        // zone_w.loadable = true;
        // info!("zone.mem_regions() = {:#x?}", zone_w.config().mem_regions());
        // let mem_regs: Vec<HvMemoryRegion> = zone_w.config().mem_regions().to_vec();

        // // remap to rootzone
        // let root_zone = root_zone();
        // let mut root_zone_w = root_zone.write();

        // mem_regs.iter().for_each(|mem| {
        //     if mem.flags.contains(MemFlags::LOADABLE) {
        //         root_zone_w.mem_region_map_partial(&MemoryRegion::new_with_offset_mapper(
        //             mem.phys_start as GuestPhysAddr,
        //             mem.phys_start as HostPhysAddr,
        //             mem.size as _,
        //             mem.flags,
        //         ));
        //     }
        // });
        // root_zone_w.resume();
        // info!("set loadbable done!");
        // HyperCallResult::Ok(0)
    }

    pub fn hypervisor_zone_start(&mut self, zone_id: u64) -> HyperCallResult {
        todo!();
        // info!("handle hvc zone start");
        // prepare_zone_start(find_zone_by_id(zone_id as _).unwrap())?;
        // HyperCallResult::Ok(0)
    }

    fn hypervisor_zone_destroy(&mut self, zone_id: u64) -> HyperCallResult {
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
