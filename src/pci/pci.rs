#![allow(unused_imports)]

use alloc::vec::Vec;
use core::{ptr, usize};

use crate::config::{HvPciConfig, CONFIG_MAX_PCI_DEV};
use crate::memory::addr::align_down;
use crate::pci::pcibar::BarType;
use crate::pci::{get_ecam_base, init_ecam_base};
use crate::percpu::this_zone;
use crate::zone::this_zone_id;
use crate::{
    error::HvResult,
    memory::MMIOAccess,
    memory::{mmio_perform_access, GuestPhysAddr, MemFlags, MemoryRegion},
    zone::Zone,
};

use super::bridge::BridgeConfig;
use super::endpoint::EndpointConfig;
use super::pcibar::BarRegion;
use super::phantom_cfg::PhantomCfg;
use super::{
    cfg_base, CFG_CAP_PTR_OFF, CFG_CLASS_CODE_OFF, CFG_CMD_OFF, ECAM_BASE, NUM_BAR_REGS_TYPE0,
    NUM_BAR_REGS_TYPE1, PHANTOM_DEV_HEADER,
};

#[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
use crate::arch::iommu::iommu_add_device;

#[derive(Debug)]
pub struct PciRoot {
    endpoints: Vec<EndpointConfig>,
    bridges: Vec<BridgeConfig>,
    alloc_devs: Vec<usize>, // include host bridge
    phantom_devs: Vec<PhantomCfg>,
    bar_regions: Vec<BarRegion>,
}
impl PciRoot {
    pub fn new() -> Self {
        let r = Self {
            endpoints: Vec::new(),
            bridges: Vec::new(),
            alloc_devs: Vec::new(),
            phantom_devs: Vec::new(),
            bar_regions: Vec::new(),
        };
        r
    }

    pub fn is_assigned_device(&self, bdf: usize) -> bool {
        if self.alloc_devs.contains(&bdf) {
            true
        } else {
            false
        }
    }

    pub fn bars_register(&mut self) {
        self.ep_bars_init();
        self.bridge_bars_init();
        self.get_bars_regions();
    }

    fn get_bars_regions(&mut self) {
        for ep in self.endpoints.iter() {
            let regions = ep.get_regions();
            for mut region in regions {
                if region.size < 0x1000 {
                    region.size = 0x1000;
                }
                self.bar_regions.push(region);
            }
        }
        for bridge in self.bridges.iter() {
            let regions = bridge.get_regions();
            for mut region in regions {
                if region.size < 0x1000 {
                    region.size = 0x1000;
                }
                self.bar_regions.push(region);
            }
        }
        info!("PCI BAR regions init done");
    }

    fn ep_bars_init(&mut self) {
        for ep in self.endpoints.iter_mut() {
            let cfg_base = cfg_base(ep.bdf);
            let offsets: [usize; NUM_BAR_REGS_TYPE0] = [0x10, 0x14, 0x18, 0x1c, 0x20, 0x24];
            for bar_id in 0..NUM_BAR_REGS_TYPE0 {
                unsafe {
                    let reg_ptr = (cfg_base + offsets[bar_id]) as *mut u32;
                    let origin_val = *reg_ptr;
                    *reg_ptr = 0xffffffffu32;
                    let new_val = *reg_ptr;
                    ep.bars_init(bar_id, origin_val, new_val);
                    *reg_ptr = origin_val;
                }
            }
        }
    }

    fn bridge_bars_init(&mut self) {
        for bridge in self.bridges.iter_mut() {
            let cfg_base = cfg_base(bridge.bdf);
            let offsets: [usize; NUM_BAR_REGS_TYPE1] = [0x10, 0x14];
            for bar_id in 0..NUM_BAR_REGS_TYPE1 {
                unsafe {
                    let reg_ptr = (cfg_base + offsets[bar_id]) as *mut u32;
                    let origin_val = *reg_ptr;
                    *reg_ptr = 0xffffffffu32;
                    let new_val = *reg_ptr;
                    bridge.bars_init(bar_id, origin_val, new_val);
                    *reg_ptr = origin_val;
                }
            }
        }
    }
}

impl Zone {
    pub fn pci_init(
        &mut self,
        pci_config: &HvPciConfig,
        num_pci_devs: usize,
        alloc_pci_devs: &[u64; CONFIG_MAX_PCI_DEV],
    ) {
        if num_pci_devs == 0 {
            return;
        }

        info!("PCIe init!");

        init_ecam_base(pci_config.ecam_base as _);

        for idx in 0..num_pci_devs {
            info!(
                "PCIe device assigned to zone {}: {:#x}",
                self.id, alloc_pci_devs[idx]
            );
            self.pciroot.alloc_devs.push(alloc_pci_devs[idx] as _);
            #[cfg(all(feature = "platform_qemu", target_arch = "aarch64"))]
            if alloc_pci_devs[idx] != 0 {
                iommu_add_device(self.id, alloc_pci_devs[idx] as _);
            }
        }

        if self.id == 0 {
            self.root_pci_init(pci_config);
        } else {
            self.virtual_pci_mmio_init(pci_config);
            self.virtual_pci_device_init(pci_config);
        }
    }

    pub fn root_pci_init(&mut self, pci_config: &HvPciConfig) {
        // Virtual ECAM

        self.mmio_region_register(
            pci_config.ecam_base as _,
            pci_config.ecam_size as _,
            mmio_pci_handler,
            pci_config.ecam_base as _,
        );

        if pci_config.io_size != 0 {
            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    pci_config.io_base as GuestPhysAddr,
                    pci_config.io_base as _,
                    pci_config.io_size as _,
                    MemFlags::READ | MemFlags::WRITE,
                ))
                .ok();
        }

        if pci_config.mem32_size != 0 {
            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    pci_config.mem32_base as GuestPhysAddr,
                    pci_config.mem32_base as _,
                    pci_config.mem32_size as _,
                    MemFlags::READ | MemFlags::WRITE,
                ))
                .ok();
        }

        if pci_config.mem64_size != 0 {
            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    pci_config.mem64_base as GuestPhysAddr,
                    pci_config.mem64_base as _,
                    pci_config.mem64_size as _,
                    MemFlags::READ | MemFlags::WRITE,
                ))
                .ok();
        }
    }

    //probe pci mmio
    pub fn virtual_pci_mmio_init(&mut self, pci_config: &HvPciConfig) {
        self.mmio_region_register(
            pci_config.ecam_base as _,
            pci_config.ecam_size as _,
            mmio_pci_handler,
            pci_config.ecam_base as _,
        );

        if pci_config.io_size != 0 {
            self.mmio_region_register(
                pci_config.io_base as _,
                pci_config.io_size as _,
                mmio_pci_handler,
                pci_config.io_base as _,
            );
        }

        if pci_config.mem32_size != 0 {
            self.mmio_region_register(
                pci_config.mem32_base as _,
                pci_config.mem32_size as _,
                mmio_pci_handler,
                pci_config.mem32_base as _,
            );
        }

        if pci_config.mem64_size != 0 {
            self.mmio_region_register(
                pci_config.mem64_base as _,
                pci_config.mem64_size as _,
                mmio_pci_handler,
                pci_config.mem64_base as _,
            );
        }
    }

    pub fn virtual_pci_device_init(&mut self, pci_config: &HvPciConfig) {
        for bdf in self.pciroot.alloc_devs.clone() {
            if bdf != 0 {
                let base = cfg_base(bdf) + 0xe;
                let header_val = unsafe { ptr::read_volatile(base as *mut u8) };
                match header_val & 0b1111111 {
                    0b0 => self.pciroot.endpoints.push(EndpointConfig::new(bdf)),
                    0b1 => self.pciroot.bridges.push(BridgeConfig::new(bdf)),
                    _ => error!("unsupported device type!"),
                };
            } else {
                // host bridge
            }
        }

        trace!("pciroot = {:?}", self.pciroot);
        self.pciroot.bars_register();
        self.pci_bars_register(pci_config);
    }

    fn pci_bars_register(&mut self, pci_config: &HvPciConfig) {
        for region in self.pciroot.bar_regions.iter_mut() {
            let (cpu_base, pci_base) = match region.bar_type {
                BarType::IO => (pci_config.io_base as usize, pci_config.pci_io_base as usize),
                BarType::Mem32 => (
                    pci_config.mem32_base as usize,
                    pci_config.pci_mem32_base as usize,
                ),
                BarType::Mem64 => (
                    pci_config.mem64_base as usize,
                    pci_config.pci_mem64_base as usize,
                ),
                _ => panic!("Unknown BAR type!"),
            };

            region.start = cpu_base + region.start - pci_base;
            region.start = align_down(region.start);

            info!(
                "pci bar region: type: {:?}, base: {:#x}, size:{:#x}",
                region.bar_type, region.start, region.size
            );

            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    region.start as GuestPhysAddr,
                    region.start,
                    region.size,
                    MemFlags::READ | MemFlags::WRITE,
                ))
                .ok();
        }
    }
}

pub fn mmio_pci_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    let reg_addr = mmio.address & 0xfff;
    let bdf = mmio.address >> 12;
    let function = bdf & 0x7;
    let device = (bdf >> 3) & 0b11111;
    let bus = bdf >> 8;

    let zone = this_zone();
    let mut binding = zone.write();
    let is_assigned = binding.pciroot.is_assigned_device(bdf);

    match is_assigned {
        true => {
            mmio_perform_access(base, mmio);
        }
        false => {
            match reg_addr {
                0 => {
                    let header_addr = base + mmio.address;
                    let header_val = unsafe { ptr::read_volatile(header_addr as *mut u32) };
                    if header_val == 0xffffffffu32 {
                        // empty device
                        mmio.value = 0xffffffffu32 as _;
                    } else {
                        // phantom device
                        let command_addr = CFG_CMD_OFF + header_addr;
                        let command = unsafe { ptr::read_volatile(command_addr as *mut u16) };
                        binding
                            .pciroot
                            .phantom_devs
                            .push(PhantomCfg::new(bdf, command)); // add phantom devs, for accessing virtual command register
                        mmio.value = PHANTOM_DEV_HEADER as _;
                    }
                }
                CFG_CMD_OFF => {
                    if let Some(pdev) = binding
                        .pciroot
                        .phantom_devs
                        .iter_mut()
                        .find(|pdev| pdev.bdf == bdf)
                    {
                        if mmio.is_write {
                            pdev.set_cmd(mmio.value as _);
                        } else {
                            mmio.value = pdev.get_cmd() as _;
                        }
                    }
                }
                CFG_CLASS_CODE_OFF => {
                    if !mmio.is_write {
                        mmio.value = 0x1f000010;
                    }
                }
                CFG_CAP_PTR_OFF => {
                    // can't see any capabilities
                    mmio.value = 0x0;
                }
                _ => {
                    mmio_perform_access(base, mmio);
                }
            }
        }
    }
    if mmio.is_write == true {
        trace!(
            "ecam write {} bytes, {:x}:{:x}:{:x} off:{:#x} -> {:#x}",
            mmio.size,
            bus,
            device,
            function,
            reg_addr,
            mmio.value
        );
    } else {
        trace!(
            "ecam read  {} bytes, {:x}:{:x}:{:x} off:{:#x} -> {:#x}",
            mmio.size,
            bus,
            device,
            function,
            reg_addr,
            mmio.value
        );
    }

    Ok(())
}
