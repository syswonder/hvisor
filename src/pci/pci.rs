// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//
use core::{panic, ptr, usize};

use crate::config::{HvPciConfig, CONFIG_MAX_PCI_DEV};
use crate::memory::addr::align_down;
use crate::memory::mmio_perform_access;
use crate::pci::pcibar::BarType;
use crate::pci::phantom_cfg::find_phantom_dev;
use crate::pci::{get_ecam_base, init_bdf_shift, init_ecam_base};
use crate::percpu::this_zone;
use crate::{
    error::HvResult,
    memory::MMIOAccess,
    memory::{GuestPhysAddr, MemFlags, MemoryRegion},
    zone::Zone,
};
use alloc::vec::Vec;

use super::bridge::BridgeConfig;
use super::endpoint::EndpointConfig;
use super::pcibar::BarRegion;
use super::phantom_cfg::{add_phantom_devices, generate_vep_by_bdf, PhantomCfg};
use super::{
    cfg_base, extract_reg_addr, get_bdf_shift, CFG_EXT_CAP_PTR_OFF, NUM_BAR_REGS_TYPE0,
    NUM_BAR_REGS_TYPE1,
};

#[cfg(all(feature = "iommu", target_arch = "aarch64"))]
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

    pub fn is_bridge(&self, bdf: usize) -> bool {
        match self.bridges.iter().find(|&b| b.bdf == bdf) {
            Some(b) => true,
            None => false,
        }
    }

    pub fn bars_register(&mut self) {
        self.ep_bars_init();
        self.bridge_bars_init();
        self.get_bars_regions();
    }

    pub fn generate_vdevs(&self) {
        for ep in self.endpoints.iter() {
            add_phantom_devices(ep.generate_vep());
        }
        for bridge in self.bridges.iter() {
            add_phantom_devices(bridge.generate_vbridge());
        }
    }

    fn get_bars_regions(&mut self) {
        for ep in self.endpoints.iter() {
            let regions = ep.get_regions();
            for mut region in regions {
                if region.size < 0x1000 {
                    // unnecessary unless you use qemu pci-test-dev
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

        let mut hv_addr_prefix: u64 = 0;
        let mut loong_ht_prefix: u64 = 0;
        let mut bdf_shift: usize = 12;

        #[cfg(all(target_arch = "loongarch64"))]
        {
            info!("change bdf shift to 8 for loongson");
            bdf_shift = 8;
            /* turn to virtual address and add 0xe prefix for HT accessing */
            hv_addr_prefix = 0x8000_0000_0000_0000;
            loong_ht_prefix = 0xe00_0000_0000
        }

        init_bdf_shift(bdf_shift);

        init_ecam_base((pci_config.ecam_base + hv_addr_prefix + loong_ht_prefix) as _);

        info!("PCIe ECAM base: {:#x}", get_ecam_base());

        for idx in 0..num_pci_devs {
            info!(
                "PCIe device assigned to zone {}: {:#x}:{:#x}.{:#x}",
                self.id,
                alloc_pci_devs[idx] >> 8,
                (alloc_pci_devs[idx] >> 3) & 0b11111,
                alloc_pci_devs[idx] & 0b111
            );
            self.pciroot.alloc_devs.push(alloc_pci_devs[idx] as _);
            #[cfg(all(feature = "iommu", target_arch = "aarch64"))]
            if alloc_pci_devs[idx] != 0 {
                iommu_add_device(self.id, alloc_pci_devs[idx] as _);
            }
        }

        if self.id == 0 {
            self.root_pci_init(pci_config, hv_addr_prefix, loong_ht_prefix);
        } else {
            self.virtual_pci_mmio_init(pci_config, hv_addr_prefix, loong_ht_prefix);
        }
        self.virtual_pci_device_init(pci_config);
    }

    pub fn root_pci_init(
        &mut self,
        pci_config: &HvPciConfig,
        hv_addr_prefix: u64,
        loong_ht_prefix: u64,
    ) {
        // Virtual ECAM

        self.mmio_region_register(
            pci_config.ecam_base as _,
            pci_config.ecam_size as _,
            mmio_pci_handler,
            (pci_config.ecam_base + hv_addr_prefix + loong_ht_prefix) as _,
        );

        // self.gpm.insert(MemoryRegion::new_with_offset_mapper(
        //             pci_config.ecam_base as GuestPhysAddr,
        //             (pci_config.ecam_base + loong_ht_prefix) as _,
        //             pci_config.ecam_size as _,
        //             MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
        //         ))
        //         .ok();

        info!(
            "pci handler args : {:#x}",
            pci_config.ecam_base + hv_addr_prefix + loong_ht_prefix
        );

        if pci_config.io_size != 0 {
            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    pci_config.io_base as GuestPhysAddr,
                    pci_config.io_base as _,
                    pci_config.io_size as _,
                    MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
                ))
                .ok();
        }

        if pci_config.mem32_size != 0 {
            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    pci_config.mem32_base as GuestPhysAddr,
                    pci_config.mem32_base as _,
                    pci_config.mem32_size as _,
                    MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
                ))
                .ok();
        }

        if pci_config.mem64_size != 0 {
            self.gpm
                .insert(MemoryRegion::new_with_offset_mapper(
                    pci_config.mem64_base as GuestPhysAddr,
                    pci_config.mem64_base as _,
                    pci_config.mem64_size as _,
                    MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
                ))
                .ok();
        }
    }

    //probe pci mmio
    pub fn virtual_pci_mmio_init(
        &mut self,
        pci_config: &HvPciConfig,
        hv_addr_prefix: u64,
        loong_ht_prefix: u64,
    ) {
        self.mmio_region_register(
            pci_config.ecam_base as _,
            pci_config.ecam_size as _,
            mmio_pci_handler,
            (pci_config.ecam_base + hv_addr_prefix + loong_ht_prefix) as _,
        );

        if pci_config.io_size != 0 {
            self.mmio_region_register(
                pci_config.io_base as _,
                pci_config.io_size as _,
                mmio_pci_bar_handler,
                (pci_config.io_base + hv_addr_prefix) as _,
            );
        }

        if pci_config.mem32_size != 0 {
            self.mmio_region_register(
                pci_config.mem32_base as _,
                pci_config.mem32_size as _,
                mmio_pci_bar_handler,
                (pci_config.mem32_base + hv_addr_prefix) as _,
            );
        }

        if pci_config.mem64_size != 0 {
            self.mmio_region_register(
                pci_config.mem64_base as _,
                pci_config.mem64_size as _,
                mmio_pci_bar_handler,
                (pci_config.mem64_base + hv_addr_prefix) as _,
            );
        }

        info!("PCIe MMIO init done!");
    }

    pub fn virtual_pci_device_init(&mut self, pci_config: &HvPciConfig) {
        for bdf in self.pciroot.alloc_devs.clone() {
            if bdf != 0 {
                let base = cfg_base(bdf) + 0xe;
                let header_val = unsafe { ptr::read_volatile(base as *mut u8) };
                match header_val & 0b1111111 {
                    0b0 => self.pciroot.endpoints.push(EndpointConfig::new(bdf)),
                    0b1 => self.pciroot.bridges.push(BridgeConfig::new(bdf)),
                    _ => error!(
                        "bdf {:#x} unsupported device type: {}!",
                        bdf,
                        header_val & 0b1111111
                    ),
                };
            } else {
                // host bridge
                self.pciroot.bridges.push(BridgeConfig::new(bdf));
            }
        }

        trace!("pciroot = {:?}", self.pciroot);
        self.pciroot.bars_register();
        if self.id != 0 {
            self.pci_bars_register(pci_config);
        }
        self.pciroot.generate_vdevs();
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
                    MemFlags::READ | MemFlags::WRITE | MemFlags::IO,
                ))
                .ok();
        }
    }
}

pub fn mmio_pci_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    // info!("mmio pci: {:#x}", mmio.address);
    let zone = this_zone();
    let mut binding = zone.write();
    let zone_id = binding.id;

    let reg_addr = extract_reg_addr(mmio.address);
    let bdf_shift = get_bdf_shift();
    let bdf = (mmio.address >> bdf_shift) & 0xffff;

    let is_assigned = binding.pciroot.is_assigned_device(bdf);
    let is_bridge = binding.pciroot.is_bridge(bdf);

    match is_assigned {
        true => {
            mmio_perform_access(base, mmio);
            if reg_addr == 0x150 {
                mmio.value = mmio.value & 0x00ffffff;
                mmio.value += 0x1a000000;
            }
            // if (reg_addr >= CFG_EXT_CAP_PTR_OFF) && !is_bridge {
            //     mmio.value = match binding.pciroot.endpoints.iter().find(|&ep| ep.bdf == bdf) {
            //         Some(ep) => ep.skip_sriov(mmio.value),
            //         None => {
            //             error!("Endpoint {:x}:{:x}.{:x} doesn't exist!", bdf >> 8, (bdf >> 3) &0b11111, bdf & 0b111);
            //             mmio.value
            //         }
            //     }
            // }
            return Ok(());
        }
        false => {
            let header_addr = cfg_base(bdf);
            let header_val = unsafe { ptr::read_volatile(header_addr as *mut u32) };
            if header_val == 0xffffffffu32 || header_val == 0 {
                if reg_addr == 0 && mmio.is_write == false {
                    mmio.value = header_val as _;
                    return Ok(());
                } else {
                    panic!("invalid access to empty device {:x}:{:x}.{:x}, addr: {:#x}, reg_addr: {:#x}!", bdf >> 8, (bdf >> 3) & 0b11111, bdf & 0b111, mmio.address, reg_addr);
                }
            } else {
                // device exists, so we try to get the phantom device
                let pdev = match binding
                    .pciroot
                    .phantom_devs
                    .iter_mut()
                    .find(|dev| dev.bdf == bdf)
                {
                    Some(dev) => dev,
                    None => {
                        let new_dev = find_phantom_dev(bdf);
                        binding.pciroot.phantom_devs.push(new_dev);
                        binding
                            .pciroot
                            .phantom_devs
                            .iter_mut()
                            .find(|dev| dev.bdf == bdf)
                            .unwrap()
                    }
                };
                pdev.phantom_mmio_handler(mmio, base, zone_id)
            }
        }
    }
}

pub fn mmio_pci_bar_handler(mmio: &mut MMIOAccess, base: usize) -> HvResult {
    panic!("mmio pci bar: {:#x}", mmio.address + base);
    mmio_perform_access(base, mmio);
    Ok(())
}
