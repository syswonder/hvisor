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
pub use hvisor_abi::{
    BitmapWord, HvConfigMemoryRegion, HvIvcConfig, HvPciConfig, HvZoneBootMode,
    CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD, CONFIG_MAGIC_VERSION, CONFIG_MAX_INTERRUPTS,
    CONFIG_MAX_IVC_CONFIGS, CONFIG_MAX_MEMORY_REGIONS, CONFIG_MAX_PCI_DEV, CONFIG_NAME_MAXLEN,
    CONFIG_PCI_BUS_MAXNUM, MEM_TYPE_IO, MEM_TYPE_RAM, MEM_TYPE_VIRTIO,
};

pub use hvisor_abi::HvPciDevConfig;

#[cfg(target_arch = "aarch64")]
pub use hvisor_abi::config::aarch64::Aarch64ZoneConfig as HvZoneConfig;
#[cfg(target_arch = "loongarch64")]
pub use hvisor_abi::config::loongarch64::Loongarch64ZoneConfig as HvZoneConfig;
#[cfg(target_arch = "riscv64")]
pub use hvisor_abi::config::riscv64::Riscv64ZoneConfig as HvZoneConfig;
#[cfg(target_arch = "x86_64")]
pub use hvisor_abi::config::x86_64::X86_64ZoneConfig as HvZoneConfig;

use spin::Once;

use crate::platform;

pub static mut HV_ROOT_ZONE_CONFIG: Once<HvZoneConfig> = Once::new();

pub fn init() {
    unsafe { HV_ROOT_ZONE_CONFIG.call_once(|| platform::platform_root_zone_config()) };
}

pub fn root_zone_config() -> &'static HvZoneConfig {
    init();
    unsafe { HV_ROOT_ZONE_CONFIG.get().unwrap() }
}

#[allow(unused)]
pub const IVC_PROTOCOL_USER: u32 = 0x0;

#[allow(unused)]
pub const IVC_PROTOCOL_HVISOR: u32 = 0x1;

#[macro_export]
macro_rules! pci_dev {
    ($domain:expr, $bus:expr, $dev:expr, $func:expr => $v_bus:expr, $v_dev:expr, $v_func:expr, $dev_type:expr) => {
        HvPciDevConfig {
            domain: $domain,
            bus: $bus,
            device: $dev,
            function: $func,
            v_bus: $v_bus,
            v_device: $v_dev,
            v_function: $v_func,
            dev_type: $dev_type,
        }
    };
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HvDwcAtuConfig {
    // ECAM (Enhanced Configuration Access Mechanism) base address
    // This is used to match with HvPciConfig::ecam_base
    pub ecam_base: u64,
    pub dbi_base: u64,
    pub dbi_size: u64,
    pub apb_base: u64,
    pub apb_size: u64,
    pub cfg_base: u64,
    pub cfg_size: u64,
    // set 1 if io base use atu0, when hvisor need set mmio for io
    // normally, when num-viewport less than 4, io_cfg_atu_shared is 1, otherwise is 0
    pub io_cfg_atu_shared: u64,
    // choose the atu index for io and cfg access, when io_cfg_atu_shared is 1, io and cfg use the same atu index, otherwise use different atu index
    pub io_atu_index: u64,
    // Shared hardware interrupt ID for this DWC RC MSI block
    pub dw_msi_irq: u64,
}

impl HvDwcAtuConfig {
    pub const fn new_empty() -> Self {
        // Use ATU_UNUSED for ATU indices that are not used by default
        // ATU_UNUSED is u32::MAX, cast to usize for consistency
        // Default ATU types: CFG0=4, CFG1=5, MEM=0, IO=2
        Self {
            ecam_base: 0,
            dbi_base: 0,
            dbi_size: 0,
            apb_base: 0,
            apb_size: 0,
            cfg_base: 0,
            cfg_size: 0,
            io_cfg_atu_shared: 0,
            io_atu_index: 0,
            dw_msi_irq: 0,
        }
    }
}

impl Default for HvDwcAtuConfig {
    fn default() -> Self {
        Self::new_empty()
    }
}
pub const fn get_irqs_bitmap<const N: usize>(
    numbers: &[u32; N],
) -> [BitmapWord; CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD] {
    assert!(
        CONFIG_MAX_INTERRUPTS % CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD == 0,
        "Configuration error: CONFIG_MAX_INTERRUPTS must be a multiple of 32 for a [u32] bitmap without rounding.",
    );

    let mut bitmap = [0; CONFIG_MAX_INTERRUPTS / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD];

    let mut i = 0;
    while i < N {
        let num = numbers[i];

        assert!(
            (num as usize) < CONFIG_MAX_INTERRUPTS,
            "Input IRQ number is out of bounds. It must be less than CONFIG_MAX_INTERRUPTS.",
        );

        let word_index = num as usize / CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD;
        let bit_index = num as usize & (CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD - 1);

        bitmap[word_index] |= (1 as BitmapWord) << bit_index;
        i += 1;
    }

    bitmap
}
