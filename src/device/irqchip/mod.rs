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
use crate::arch::zone::HvArchZoneConfig;
use crate::zone::Zone;

#[cfg(all(feature = "gicv2", target_arch = "aarch64"))]
pub mod gicv2;
#[cfg(all(feature = "gicv2", target_arch = "aarch64"))]
pub use gicv2::{
    gic::inject_irq, gicd::set_ispender, percpu_init, primary_init_early, primary_init_late,
    vgic::set_sgi_irq,
};

#[cfg(all(feature = "gicv3", target_arch = "aarch64"))]
pub mod gicv3;
#[cfg(all(feature = "gicv3", target_arch = "aarch64"))]
pub use gicv3::{
    gicd::set_ispender, inject_irq, percpu_init, primary_init_early, primary_init_late,
};

#[cfg(all(feature = "aclint", target_arch = "riscv64"))]
pub mod aclint;

#[cfg(all(feature = "plic", target_arch = "riscv64"))]
pub mod plic;

#[cfg(all(feature = "aia", target_arch = "riscv64"))]
pub mod aia;

#[cfg(target_arch = "riscv64")]
pub fn primary_init_early() {
    // aclint is local interrupt controller
    // plic & aia is global interrupt controller
    #[cfg(feature = "plic")]
    plic::primary_init_early();
    #[cfg(feature = "aia")]
    aia::aplic::primary_init_early();
    #[cfg(feature = "aclint")]
    aclint::aclint_init(crate::platform::ACLINT_SSWI_BASE);
}

pub fn gic_handle_irq() {
    #[cfg(all(feature = "gicv2", target_arch = "aarch64"))]
    gicv2::gic::gicv2_handle_irq();
    #[cfg(all(feature = "gicv3", target_arch = "aarch64"))]
    gicv3::gicv3_handle_irq_el1();
}

impl Zone {
    pub fn mmio_init(&mut self, hv_config: &HvArchZoneConfig) {
        #[cfg(all(feature = "gicv2", target_arch = "aarch64"))]
        {
            self.vgicv2_mmio_init(hv_config);
            self.vgicv2_remap_init(hv_config);
        }
        #[cfg(all(feature = "gicv3", target_arch = "aarch64"))]
        {
            self.vgicv3_mmio_init(hv_config);
        }
        #[cfg(target_arch = "x86_64")]
        {
            self.ioapic_mmio_init(hv_config);
            // self.pci_config_space_mmio_init(hv_config);
        }
    }
}

#[cfg(all(feature = "plic", target_arch = "riscv64"))]
pub use plic::{
    host_plic, inject_irq, percpu_init, primary_init_late, vplic_global_emul_handler,
    vplic_hart_emul_handler,
};

#[cfg(all(feature = "aia", target_arch = "riscv64"))]
pub use aia::aplic::{host_aplic, inject_irq, percpu_init, primary_init_late, vaplic_emul_handler};

#[cfg(target_arch = "loongarch64")]
pub mod ls7a2000;

#[cfg(target_arch = "x86_64")]
pub mod pic;

#[cfg(target_arch = "loongarch64")]
pub use ls7a2000::{inject_irq, percpu_init, primary_init_early, primary_init_late};

#[cfg(target_arch = "x86_64")]
pub use pic::{inject_irq, inject_vector, percpu_init, primary_init_early, primary_init_late};
