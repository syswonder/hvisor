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

#[cfg(target_arch = "aarch64")]
pub fn gic_handle_irq() {
    #[cfg(feature = "gicv2")]
    gicv2::gic::gicv2_handle_irq();
    #[cfg(feature = "gicv3")]
    gicv3::gicv3_handle_irq_el1();
}

#[cfg(target_arch = "aarch64")]
pub fn gic_send_event(cpu_id: u64, sgi_num: u64) {
    #[cfg(feature = "gicv3")]
    {
        /*Actually, the value passed to ICC_SGI1R_EL1 should be derived from
        the MPIDR of the target CPU. However, since we cannot access this
        register on the sender side, we have reverse-engineered a value
        here using the cpu_id.
        Due to differences in how some CPU implementations (e.g., RK3568 and RK3588)
        encode affinity values in MPIDR, we use conditional compilation to handle
        platform-specific mappings between cpu_id and interrupt target affinity.
        */

        use crate::arch::cpu::cpuid_to_mpidr_affinity;
        use crate::arch::sysreg::write_sysreg;
        let (aff3, aff2, aff1, aff0) = cpuid_to_mpidr_affinity(cpu_id);
        let target_list = 1 << aff0;
        let irm: u64 = 0 << 40;
        let sgi_id: u64 = sgi_num << 24;
        let val: u64 = (aff1 << 16) | (aff2 << 32) | (aff3 << 48) | irm | sgi_id | target_list;
        write_sysreg!(icc_sgi1r_el1, val);
        debug!("write sgi sys value = {:#x}", val);
    }
    #[cfg(feature = "gicv2")]
    {
        let sgi_id: u64 = sgi_num;
        let target_list: u64 = 1 << cpu_id;
        set_sgi_irq(sgi_id as usize, target_list as usize, 0);
    }
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
        #[cfg(all(feature = "plic", target_arch = "riscv64"))]
        {
            self.vplic_mmio_init(hv_config);
        }
    }
}

#[cfg(all(feature = "aclint", target_arch = "riscv64"))]
pub mod aclint;

#[cfg(all(feature = "plic", target_arch = "riscv64"))]
pub mod plic;

#[cfg(all(feature = "plic", target_arch = "riscv64"))]
pub use plic::{host_plic, inject_irq, percpu_init, primary_init_late};

#[cfg(all(feature = "aia", target_arch = "riscv64"))]
pub mod aia;

#[cfg(all(feature = "aia", target_arch = "riscv64"))]
pub use aia::aplic::{host_aplic, inject_irq, percpu_init, primary_init_late, vaplic_emul_handler};

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

#[cfg(target_arch = "loongarch64")]
pub mod ls7a2000;

#[cfg(target_arch = "loongarch64")]
pub use ls7a2000::{inject_irq, percpu_init, primary_init_early, primary_init_late};
