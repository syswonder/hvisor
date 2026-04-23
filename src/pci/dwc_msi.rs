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

use alloc::collections::btree_map::BTreeMap;
#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::{Lazy, Mutex};

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
use crate::cpu_data::this_cpu_data;
use crate::error::HvResult;
#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
use crate::event::send_event;
use crate::memory::Frame;
#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
use crate::{
    consts::{IPI_EVENT_DWC_MSI_INJECT, MAX_CPU_NUM},
    device::irqchip::inject_irq,
    hypercall::SGI_IPI_ID,
};

// DBI MSI register offsets
pub const PCIE_MSI_ADDR_LO: usize = 0x820;
pub const PCIE_MSI_ADDR_HI: usize = 0x824;
pub const PCIE_MSI_INTR0_ENABLE: usize = 0x828;
pub const PCIE_MSI_INTR0_MASK: usize = 0x82c;
pub const PCIE_MSI_INTR0_STATUS: usize = 0x830;

/// DesignWare native MSI domain configuration
/// Each PCIe RC (domain) has a set of 32 MSI vectors that can be distributed
/// across multiple VMs. This structure manages the allocation of these vectors
/// and provides storage for the MSI doorbell address.
#[derive(Debug, Copy, Clone)]
pub struct DwMsiBitOwner {
    /// Target vCPU used for injection when forwarding this MSI
    pub target_cpu: usize,
    /// First hardware MSI bit in this allocation
    pub start_hwirq_bit: u32,
    /// Number of contiguous vectors in this allocation
    pub num_vectors: u32,
}

impl DwMsiBitOwner {
    #[inline]
    pub fn contains_hwbit(&self, hwbit: u32) -> bool {
        hwbit >= self.start_hwirq_bit && hwbit < self.start_hwirq_bit + self.num_vectors
    }
}

#[derive(Debug)]
pub struct DwMsiDomain {
    /// Next MSI vector index to allocate (0-31)
    /// When a VM allocates N vectors, it gets indices [next_alloc, next_alloc+N)
    pub next_alloc: u32,

    /// Physical page frame for the doorbell address
    /// Each domain has one unique doorbell that is written by hvisor
    pub doorbell_frame: Frame,

    /// Shared physical interrupt line used by this DWC RC MSI block
    pub irq: u32,

    /// Per-zone allocation records for fast hwbit -> zone/cpu lookup
    pub bit_owners: Vec<DwMsiBitOwner>,
}

impl DwMsiDomain {
    /// Create a new DW MSI domain for a PCIe RC
    pub fn new(irq: u32) -> HvResult<Self> {
        let doorbell_frame = Frame::new_zero()?;

        Ok(Self {
            next_alloc: 0,
            doorbell_frame,
            irq,
            bit_owners: Vec::new(),
        })
    }

    /// Allocate a contiguous range of MSI vectors for a VM
    /// Returns the starting vector index if successful, or error if not enough vectors available
    pub fn allocate(&mut self, num_vectors: u32) -> HvResult<u32> {
        // Check if there are enough vectors left (32 total vectors per domain)
        if self.next_alloc + num_vectors > 32 {
            return hv_result_err!(EINVAL, "Not enough MSI vectors available in domain");
        }

        let alloc_offset = self.next_alloc;
        self.next_alloc += num_vectors;

        Ok(alloc_offset)
    }

    /// Allocate MSI vectors for a zone and record the ownership mapping.
    pub fn allocate_for_cpu(&mut self, target_cpu: usize, num_vectors: u32) -> HvResult<u32> {
        let start_hwirq_bit = self.allocate(num_vectors)?;
        self.bit_owners.push(DwMsiBitOwner {
            target_cpu,
            start_hwirq_bit,
            num_vectors,
        });
        info!(
            "DW MSI owner added: cpu {}, vectors {}, range [{}..{}), total_owner_records {}",
            target_cpu,
            num_vectors,
            start_hwirq_bit,
            start_hwirq_bit + num_vectors,
            self.bit_owners.len()
        );
        Ok(start_hwirq_bit)
    }

    /// Find the owner record by hardware MSI bit.
    pub fn find_owner_by_hwbit(&self, hwbit: u32) -> Option<DwMsiBitOwner> {
        self.bit_owners
            .iter()
            .copied()
            .find(|owner| owner.contains_hwbit(hwbit))
    }

    /// Get the physical address of the doorbell for this domain
    pub fn doorbell_paddr(&self) -> u64 {
        self.doorbell_frame.start_paddr() as u64
    }
}

/// Global storage for DW MSI domain configurations
/// Indexed by domain ID
pub static DW_MSI_DOMAINS: Lazy<Mutex<BTreeMap<u8, DwMsiDomain>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
static DWC_MSI_IPI_PENDING_IRQS: Lazy<Mutex<Vec<VecDeque<usize>>>> = Lazy::new(|| {
    let mut queues = Vec::with_capacity(MAX_CPU_NUM);
    for _ in 0..MAX_CPU_NUM {
        queues.push(VecDeque::new());
    }
    Mutex::new(queues)
});

/// Initialize DW MSI domain for a given domain ID
pub fn init_dwc_msi_domain(domain_id: u8, irq: u32) -> HvResult<()> {
    let domain = DwMsiDomain::new(irq)?;
    let doorbell_paddr = domain.doorbell_paddr();
    DW_MSI_DOMAINS.lock().insert(domain_id, domain);
    info!(
        "Initialized DW MSI domain {} with doorbell at {:#x}, irq {}",
        domain_id, doorbell_paddr, irq
    );
    Ok(())
}

/// Get mutable reference to a DW MSI domain
pub fn get_dwc_msi_domain_mut(
    domain_id: u8,
) -> Option<spin::MutexGuard<'static, BTreeMap<u8, DwMsiDomain>>> {
    let domains = DW_MSI_DOMAINS.lock();
    if domains.contains_key(&domain_id) {
        drop(domains);
        Some(DW_MSI_DOMAINS.lock())
    } else {
        None
    }
}

/// Get the doorbell physical address for a specific domain
/// Returns 0 if domain not found
pub fn get_domain_doorbell_paddr(domain_id: u8) -> u64 {
    let domains = DW_MSI_DOMAINS.lock();
    domains
        .get(&domain_id)
        .map(|domain| domain.doorbell_paddr())
        .unwrap_or(0)
}

/// Get the shared hardware MSI IRQ for a specific domain
/// Returns 0 if domain not found
pub fn get_domain_msi_irq(domain_id: u8) -> u32 {
    let domains = DW_MSI_DOMAINS.lock();
    domains
        .get(&domain_id)
        .map(|domain| domain.irq)
        .unwrap_or(0)
}

/// Check whether an IRQ is used as a shared DWC MSI hardware interrupt
pub fn is_dwc_msi_irq(irq: u32) -> bool {
    let domains = DW_MSI_DOMAINS.lock();
    domains.values().any(|domain| domain.irq == irq)
}

/// Check whether an IRQ matches any configured DWC MSI hardware interrupt.
pub fn is_dwc_msi_hwirq(irq: u32) -> bool {
    is_dwc_msi_irq(irq)
}

/// Return all domain IDs that share the specified DWC MSI hardware IRQ.
pub fn get_domains_by_irq(irq: u32) -> Vec<u8> {
    let domains = DW_MSI_DOMAINS.lock();
    domains
        .iter()
        .filter_map(|(domain_id, domain)| {
            if domain.irq == irq {
                Some(*domain_id)
            } else {
                None
            }
        })
        .collect()
}

/// Find bit ownership for a specific domain and hardware MSI bit.
pub fn get_domain_owner_by_hwirq_bit(domain_id: u8, hwbit: u32) -> Option<DwMsiBitOwner> {
    let domains = DW_MSI_DOMAINS.lock();
    domains
        .get(&domain_id)
        .and_then(|domain| domain.find_owner_by_hwbit(hwbit))
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
fn get_domain_dbi_base(domain_id: u8) -> Option<usize> {
    let ecam_base = crate::platform::ROOT_PCI_CONFIG
        .iter()
        .find(|cfg| cfg.domain == domain_id)
        .map(|cfg| cfg.ecam_base)?;

    crate::platform::ROOT_DWC_ATU_CONFIG
        .iter()
        .find(|cfg| cfg.ecam_base == ecam_base)
        .map(|cfg| cfg.dbi_base as usize)
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
fn first_set_bit(mask: u32) -> Option<u32> {
    if mask == 0 {
        None
    } else {
        Some(mask.trailing_zeros())
    }
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
fn find_target_cpu(domain_id: u8, irq_bit: usize) -> Option<usize> {
    get_domain_owner_by_hwirq_bit(domain_id, irq_bit as u32).map(|owner| owner.target_cpu)
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
fn dwc_msi_pending_irq_bit(domain_id: u8) -> Option<u32> {
    let dbi_base = get_domain_dbi_base(domain_id)?;
    let status =
        unsafe { core::ptr::read_volatile((dbi_base + PCIE_MSI_INTR0_STATUS) as *const u32) };
    first_set_bit(status)
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
pub fn get_domain_id_by_irq(irq: u32) -> Option<u8> {
    let domains = DW_MSI_DOMAINS.lock();
    domains.iter().find_map(|(domain_id, domain)| {
        if domain.irq == irq {
            Some(*domain_id)
        } else {
            None
        }
    })
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
fn enqueue_dwc_msi_ipi_irq(target_cpu: usize, irq_id: usize) -> bool {
    if target_cpu >= MAX_CPU_NUM {
        error!(
            "DWC MSI enqueue failed: invalid target cpu {}, irq {}",
            target_cpu, irq_id
        );
        return false;
    }

    let mut queues = DWC_MSI_IPI_PENDING_IRQS.lock();
    if let Some(queue) = queues.get_mut(target_cpu) {
        queue.push_back(irq_id);
        true
    } else {
        false
    }
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
fn pop_dwc_msi_ipi_irq(cpu_id: usize) -> Option<usize> {
    if cpu_id >= MAX_CPU_NUM {
        return None;
    }

    let mut queues = DWC_MSI_IPI_PENDING_IRQS.lock();
    queues.get_mut(cpu_id).and_then(|queue| queue.pop_front())
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
pub fn dwc_msi_transfer_and_inject(domain_id: u8, irq_id: usize) {
    if let Some(irq_bit) = dwc_msi_pending_irq_bit(domain_id) {
        if let Some(target_cpu) = find_target_cpu(domain_id, irq_bit as usize) {
            if target_cpu == 0 {
                inject_irq(irq_id, true);
            } else {
                if enqueue_dwc_msi_ipi_irq(target_cpu, irq_id) {
                    send_event(target_cpu, SGI_IPI_ID as usize, IPI_EVENT_DWC_MSI_INJECT);
                } else {
                    error!(
                        "Failed to enqueue DWC MSI irq {} for target cpu {}",
                        irq_id, target_cpu
                    );
                }
            }
        } else {
            error!("No target cpu found for DWC msi irq bit {}!", irq_bit);
        }
    } else {
        error!("No pending DWC msi irq found!");
    }
}

#[cfg(all(feature = "dwc_pcie", feature = "dwc_msi"))]
pub fn handle_dwc_msi_inject_event() {
    let cpu_id = this_cpu_data().id;
    if let Some(irq_id) = pop_dwc_msi_ipi_irq(cpu_id) {
        inject_irq(irq_id, true);
    } else {
        warn!("No pending DWC MSI IPI irq for cpu {}", cpu_id);
    }
}
