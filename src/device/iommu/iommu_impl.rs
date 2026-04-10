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
//      Jingyu Liu <liujingyu24s@ict.ac.cn>

use super::Iommu;
use alloc::boxed::Box;
use spin::Once;

/// Global IOMMU implementation instance
/// Note: concret implementations don't contain any mutable global state, only common interfaces define in the trait.
static IOMMU_IMPL: Once<Box<dyn Iommu + Sync + Send>> = Once::new();

// Dispatch to the appropriate IOMMU implementation based on hardware support
fn iommu_impl_init() -> Box<dyn Iommu + Sync + Send> {
    #[cfg(feature = "arm_smmu")]
    return Box::new(super::arm_smmu::ArmSmmu);

    #[cfg(feature = "intel_vtd")]
    return Box::new(super::intel_vtd::IntelVtd);

    #[cfg(feature = "riscv_iommu")]
    return Box::new(super::riscv_iommu::RiscvIommu);

    // Default return DummyIommu if no IOMMU support
    #[cfg(not(any(feature = "arm_smmu", feature = "intel_vtd", feature = "riscv_iommu",)))]
    return Box::new(super::dummy_iommu::DummyIommu);
}

/// Get the global IOMMU implementation instance
/// Note: IOMMU_IMPL is immutable after initialization
pub(super) fn iommu_impl() -> &'static dyn Iommu {
    IOMMU_IMPL.call_once(|| iommu_impl_init()).as_ref()
}
