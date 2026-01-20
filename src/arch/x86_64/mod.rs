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
//  Solicey <lzoi_lth@163.com>

#![allow(unused)]
pub mod acpi;
pub mod boot;
pub mod consts;
pub mod cpu;
pub mod cpuid;
pub mod entry;
pub mod graphics;
pub mod hpet;
pub mod hypercall;
pub mod idt;
pub mod iommu;
pub mod ipi;
pub mod mm;
pub mod mmio;
pub mod msr;
pub mod paging;
pub mod pci;
pub mod pio;
pub mod s1pt;
pub mod s2pt;
pub mod time;
pub mod trap;
pub mod vmcs;
pub mod vmx;
pub mod zone;

pub use s1pt::Stage1PageTable;
pub use s2pt::stage2_mode_detect;
pub use s2pt::Stage2PageTable;
