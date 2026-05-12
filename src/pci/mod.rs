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

#![allow(dead_code)]

#[cfg(all(target_arch = "aarch64", irq_gicv2, irq_gicv3))]
compile_error!("invalid: irq_gicv2 and irq_gicv3 are mutually exclusive");

#[cfg(all(ecam_pcie, dwc_pcie))]
compile_error!("invalid: ecam_pcie and dwc_pcie are mutually exclusive");
#[cfg(all(ecam_pcie, loongarch64_pcie))]
compile_error!("invalid: ecam_pcie and loongarch64_pcie are mutually exclusive");
#[cfg(all(dwc_pcie, loongarch64_pcie))]
compile_error!("invalid: dwc_pcie and loongarch64_pcie are mutually exclusive");

pub mod config_accessors;
pub mod mem_alloc;
pub mod pci_access;
pub mod pci_config;
pub mod pci_handler;
pub mod pci_struct;
pub mod vpci_dev;

#[cfg(test)]
pub mod pci_test;

pub type PciConfigAddress = u64;
