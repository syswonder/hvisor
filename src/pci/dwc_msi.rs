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
use spin::{Lazy, Mutex};

use crate::error::HvResult;
use crate::memory::Frame;

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
#[derive(Debug)]
pub struct DwMsiDomain {
    /// Next MSI vector index to allocate (0-31)
    /// When a VM allocates N vectors, it gets indices [next_alloc, next_alloc+N)
    pub next_alloc: u32,

    /// Physical page frame for the doorbell address
    /// Each domain has one unique doorbell that is written by hvisor
    pub doorbell_frame: Frame,
}

impl DwMsiDomain {
    /// Create a new DW MSI domain for a PCIe RC
    pub fn new() -> HvResult<Self> {
        let doorbell_frame = Frame::new_zero()?;

        Ok(Self {
            next_alloc: 0,
            doorbell_frame,
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

    /// Get the physical address of the doorbell for this domain
    pub fn doorbell_paddr(&self) -> u64 {
        self.doorbell_frame.start_paddr() as u64
    }
}

/// Global storage for DW MSI domain configurations
/// Indexed by domain ID
pub static DW_MSI_DOMAINS: Lazy<Mutex<BTreeMap<u8, DwMsiDomain>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// Initialize DW MSI domain for a given domain ID
pub fn init_dwc_msi_domain(domain_id: u8) -> HvResult<()> {
    let domain = DwMsiDomain::new()?;
    let doorbell_paddr = domain.doorbell_paddr();
    DW_MSI_DOMAINS.lock().insert(domain_id, domain);
    info!(
        "Initialized DW MSI domain {} with doorbell at {:#x}",
        domain_id, doorbell_paddr
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
