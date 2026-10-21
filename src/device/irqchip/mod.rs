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
use crate::config::HvZoneConfig;
use crate::cpu_data::{get_cpu_data, this_cpu_data, GuestCpuIrqState};
use crate::hypercall::SGI_IPI_ID;
use crate::zone::Zone;

#[cfg(target_arch = "loongarch64")]
pub fn handle_virtio_irq_event() {
    ls7a2000::sync_guest_irqs();
}

#[cfg(not(target_arch = "loongarch64"))]
pub fn handle_virtio_irq_event() {
    crate::device::virtio_trampoline::handle_virtio_irq();
}

/// Synchronize posted IRQ state owned by the current vCPU.
///
#[inline]
pub fn flush_current_guest_irqs() {
    #[cfg(all(irq_gicv2, target_arch = "aarch64"))]
    gicv2::gic::flush_current_guest_irqs();
    #[cfg(all(irq_gicv3, target_arch = "aarch64"))]
    gicv3::flush_current_guest_irqs();
}

/// Run the controller-independent pending claim/retry flow.
pub fn flush_guest_irq_state(
    state: &GuestCpuIrqState,
    install: impl FnMut(usize) -> bool,
    rearm_maintenance: impl FnOnce(),
) {
    if state.flush_pending_with(install) {
        rearm_maintenance();
    }
}

/// Publish a guest software IRQ in the target guest CPU's common pending
/// bitmap and kick its fixed home pCPU. `source_cpu` is used only by GICv2 to
/// reconstruct the SGI source field in the virtual LR.
pub fn post_guest_irq(target_cpu: usize, irq_id: usize, source_cpu: Option<usize>) -> bool {
    if irq_id >= crate::config::CONFIG_MAX_INTERRUPTS {
        return false;
    }
    let target = get_cpu_data(target_cpu);
    #[cfg(not(irq_gicv2))]
    let _ = source_cpu;
    let control = target.ctrl_lock.lock();
    if target.zone.is_none() || target.vcpu.state.is_stopped() {
        return false;
    }
    #[cfg(irq_gicv2)]
    if irq_id < 16 {
        let Some(source_cpu) = source_cpu.filter(|source_cpu| *source_cpu < 8) else {
            return false;
        };
        let source_bit = 1u8 << source_cpu;
        let previous_sources = target.vcpu.arch.sgi_source[irq_id]
            .fetch_or(source_bit, core::sync::atomic::Ordering::AcqRel);
        // A repeated SGI from the same source coalesces with the occurrence
        // already represented by the source bit, including while it is being
        // claimed by the target CPU.
        if previous_sources & source_bit != 0 {
            return true;
        }
    }
    if !target.vcpu.irq.set(irq_id) {
        return false;
    }
    drop(control);
    kick_vcpu(target_cpu);
    true
}

/// Ring the host doorbell for the pCPU to which the target vCPU is pinned.
///
/// Current mapping is pCPU:vCPU=1:1. A local target relies on the common
/// exit/entry flush hook and does not receive a self IPI. A future scheduler
/// can replace `target_pcpu_id` with a resolved `VcpuTarget`.
pub fn kick_vcpu(target_pcpu_id: usize) {
    let target = get_cpu_data(target_pcpu_id);
    if target.zone.is_none() || target.vcpu.state.is_stopped() {
        return;
    }
    if target_pcpu_id != this_cpu_data().id {
        crate::arch::ipi::arch_send_event(target_pcpu_id as u64, SGI_IPI_ID);
    }
}

#[cfg(all(irq_gicv2, target_arch = "aarch64"))]
pub mod gicv2;
#[cfg(all(irq_gicv2, target_arch = "aarch64"))]
pub use gicv2::{
    gic::inject_irq, gicd::set_ispender, percpu_init, primary_init_early, primary_init_late,
    vgic::set_sgi_irq,
};

#[cfg(all(irq_gicv3, target_arch = "aarch64"))]
pub mod gicv3;
#[cfg(all(irq_gicv3, target_arch = "aarch64"))]
pub use gicv3::{
    gicd::set_ispender, inject_irq, percpu_init, primary_init_early, primary_init_late,
};

/// A GICD/GICR ISPENDR0 or ICPENDR0 value: SGIs in bits [15:0], PPIs in
/// bits [31:16].
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub(crate) struct PendingR0(pub u32);

#[cfg(target_arch = "aarch64")]
impl PendingR0 {
    pub fn sgis(self) -> u16 {
        bit_field::BitField::get_bits(&self.0, 0..16) as u16
    }

    pub fn set_sgis(&mut self, sgis: u16) {
        bit_field::BitField::set_bits(&mut self.0, 0..16, sgis as u32);
    }
}

#[cfg(target_arch = "aarch64")]
pub fn gic_handle_irq() {
    #[cfg(irq_gicv2)]
    gicv2::gic::gicv2_handle_irq();
    #[cfg(irq_gicv3)]
    gicv3::gicv3_handle_irq_el1();
}

#[cfg(target_arch = "aarch64")]
pub fn gic_send_event(cpu_id: u64, sgi_num: u64) {
    #[cfg(irq_gicv3)]
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
    #[cfg(irq_gicv2)]
    {
        let sgi_id: u64 = sgi_num;
        let target_list: u64 = 1 << cpu_id;
        set_sgi_irq(sgi_id as usize, target_list as usize, 0);
    }
}

impl Zone {
    pub fn virqc_init(&mut self, _config: &HvZoneConfig) {
        #[cfg(all(plic, target_arch = "riscv64"))]
        {
            self.vplic_init(_config);
        }
        #[cfg(all(aia, target_arch = "riscv64"))]
        {
            self.vaplic_init(_config);
            self.vimsic_init(_config);
        }
    }

    pub fn mmio_init(&mut self, hv_config: &HvArchZoneConfig) {
        #[cfg(all(irq_gicv2, target_arch = "aarch64"))]
        {
            self.vgicv2_mmio_init(hv_config);
            self.vgicv2_remap_init(hv_config);
        }
        #[cfg(all(irq_gicv3, target_arch = "aarch64"))]
        {
            self.vgicv3_mmio_init(hv_config);
        }
        #[cfg(all(plic, target_arch = "riscv64"))]
        {
            self.vplic_mmio_init(hv_config);
        }
        #[cfg(all(aia, target_arch = "riscv64"))]
        {
            self.vaplic_mmio_init(hv_config);
        }
        #[cfg(all(hypervisor_v0_6, target_arch = "riscv64"))]
        {
            #[cfg(sifive_ccache)]
            self.virtual_sifive_ccache_mmio_init();
            #[cfg(eic7700_sysreg)]
            self.virtual_syscon_mmio_init();
        }
        #[cfg(target_arch = "x86_64")]
        {
            self.ioapic_mmio_init(hv_config);
            // self.pci_config_space_mmio_init(hv_config);
        }
    }
}

#[cfg(all(aclint, target_arch = "riscv64"))]
pub mod aclint;

#[cfg(all(plic, target_arch = "riscv64"))]
pub mod plic;

#[cfg(all(plic, target_arch = "riscv64"))]
pub use plic::{inject_irq, percpu_init, primary_init_late};

#[cfg(all(aia, target_arch = "riscv64"))]
pub mod aia;

#[cfg(all(aia, target_arch = "riscv64"))]
pub use aia::{host_aplic, inject_irq, percpu_init, primary_init_late};

#[cfg(target_arch = "riscv64")]
pub fn primary_init_early() {
    // aclint is local interrupt controller
    // plic & aia is global interrupt controller
    #[cfg(plic)]
    plic::primary_init_early();
    #[cfg(aia)]
    aia::primary_init_early();
    #[cfg(aclint)]
    aclint::aclint_init(crate::platform::ACLINT_SSWI_BASE);
}

#[cfg(target_arch = "loongarch64")]
pub mod ls7a2000;

#[cfg(target_arch = "x86_64")]
pub mod pic;

#[cfg(target_arch = "loongarch64")]
pub use ls7a2000::{inject_irq, percpu_init, primary_init_early, primary_init_late};

#[cfg(target_arch = "x86_64")]
pub use pic::{inject_irq, inject_vector, percpu_init, primary_init_early, primary_init_late};
