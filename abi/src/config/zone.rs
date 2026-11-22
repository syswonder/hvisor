// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.

/// Generate the common constructor and accessors for a concrete zone config.
///
/// The structure itself is declared explicitly in each architecture module so
/// cbindgen can see it. The common fields live in HvZoneConfigCommon.
macro_rules! impl_zone_config {
    ($name:ident, $arch:ty) => {
        impl $name {
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                zone_id: u32,
                cpus: u64,
                num_memory_regions: u32,
                memory_regions: [$crate::config::common::HvConfigMemoryRegion;
                    $crate::config::common::CONFIG_MAX_MEMORY_REGIONS],
                interrupts_bitmap: [$crate::config::common::BitmapWord;
                            $crate::config::common::CONFIG_MAX_INTERRUPTS
                                / $crate::config::common::CONFIG_INTERRUPTS_BITMAP_BITS_PER_WORD],
                num_ivc_configs: u32,
                ivc_configs: [$crate::config::common::HvIvcConfig;
                    $crate::config::common::CONFIG_MAX_IVC_CONFIGS],
                entry_point: u64,
                kernel_load_paddr: u64,
                kernel_size: u64,
                dtb_load_paddr: u64,
                dtb_size: u64,
                name: [u8; $crate::config::common::CONFIG_NAME_MAXLEN],
                arch: $arch,
                num_pci_bus: u64,
                pci: [$crate::config::common::HvPciConfig;
                    $crate::config::common::CONFIG_PCI_BUS_MAXNUM],
                num_pci_devs: u64,
                alloc_pci_devs: [$crate::config::pci::HvPciDevConfig;
                    $crate::config::common::CONFIG_MAX_PCI_DEV],
            ) -> Self {
                Self {
                    common: $crate::config::common::HvZoneConfigCommon {
                        zone_id,
                        cpus,
                        num_memory_regions,
                        memory_regions,
                        interrupts_bitmap,
                        num_ivc_configs,
                        ivc_configs,
                        entry_point,
                        kernel_load_paddr,
                        kernel_size,
                        dtb_load_paddr,
                        dtb_size,
                        name,
                        num_pci_bus,
                        pci_config: pci,
                        num_pci_devs,
                        alloc_pci_devs,
                    },
                    arch_config: arch,
                }
            }

            pub fn memory_regions(&self) -> &[$crate::config::common::HvConfigMemoryRegion] {
                self.common.memory_regions()
            }

            pub fn interrupts_bitmap(&self) -> &[$crate::config::common::BitmapWord] {
                self.common.interrupts_bitmap()
            }

            pub fn cpus(&self) -> ::alloc::vec::Vec<u64> {
                self.common.cpus()
            }

            pub fn ivc_config(&self) -> &[$crate::config::common::HvIvcConfig] {
                self.common.ivc_config()
            }

            #[allow(unused)]
            pub fn pci_config(&self) -> &[$crate::config::common::HvPciConfig] {
                self.common.pci_config()
            }
        }
    };
}

pub(crate) use impl_zone_config;
