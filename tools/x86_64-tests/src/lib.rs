// SPDX-License-Identifier: MulanPSL-2.0

#[cfg(test)]
#[path = "../../../src/arch/x86_64/boot_modules.rs"]
mod boot_modules;
#[cfg(test)]
#[path = "../../../src/arch/x86_64/cpuid.rs"]
mod cpuid;

#[cfg(test)]
mod tests {
    use super::boot_modules::{relocate_modules, BootModule, RelocationError, MAX_MODULES};
    use super::cpuid;

    #[test]
    fn relocation_delays_copy_that_would_overwrite_pending_source() {
        assert_eq!(MAX_MODULES, 16);

        let mut modules = [
            BootModule {
                src: 0x1000,
                size: 0x100,
                dst: 0x3000,
                pending: true,
            },
            BootModule {
                src: 0x3000,
                size: 0x100,
                dst: 0x4000,
                pending: true,
            },
        ];
        let mut order = Vec::new();

        let moved = relocate_modules(&mut modules, |module| {
            order.push((module.src, module.dst));
        })
        .unwrap();

        assert_eq!(moved, 2);
        assert_eq!(order, [(0x3000, 0x4000), (0x1000, 0x3000)]);
        assert!(modules.iter().all(|module| !module.pending));
    }

    #[test]
    fn relocation_rejects_overlapping_destinations() {
        let mut modules = [
            BootModule {
                src: 0x1000,
                size: 0x200,
                dst: 0x5000,
                pending: true,
            },
            BootModule {
                src: 0x2000,
                size: 0x200,
                dst: 0x5100,
                pending: true,
            },
        ];

        let err = relocate_modules(&mut modules, |_| {}).unwrap_err();

        assert_eq!(err, RelocationError::DestinationOverlap);
    }

    #[test]
    fn relocation_rejects_cyclic_moves() {
        let mut modules = [
            BootModule {
                src: 0x1000,
                size: 0x100,
                dst: 0x2000,
                pending: true,
            },
            BootModule {
                src: 0x2000,
                size: 0x100,
                dst: 0x1000,
                pending: true,
            },
        ];

        let err = relocate_modules(&mut modules, |_| {}).unwrap_err();

        assert_eq!(err, RelocationError::CyclicDependency);
    }

    #[test]
    fn vendor_leaf_exposes_synthetic_frequency_leaves_without_lowering_host_limit() {
        assert_eq!(cpuid::vendor_leaf_max_basic(0x07), 0x07);
        assert_eq!(cpuid::vendor_leaf_max_basic(0x14), 0x16);
        assert_eq!(cpuid::vendor_leaf_max_basic(0x15), 0x16);
        assert_eq!(cpuid::vendor_leaf_max_basic(0x16), 0x16);
        assert_eq!(cpuid::vendor_leaf_max_basic(0x20), 0x20);
        assert_eq!(cpuid::vendor_leaf_max_basic(u32::MAX), u32::MAX);
    }

    #[test]
    fn tsc_frequency_leaf_reports_one_mhz_crystal_ratio() {
        let leaf = cpuid::tsc_frequency_leaf(2400);

        assert_eq!(leaf.eax, 1);
        assert_eq!(leaf.ebx, 2400);
        assert_eq!(leaf.ecx, 1_000_000);
        assert_eq!(leaf.edx, 0);
    }

    #[test]
    fn processor_frequency_leaf_reports_base_max_and_bus_frequency() {
        let leaf = cpuid::processor_frequency_leaf(2100);

        assert_eq!(leaf.eax, 2100);
        assert_eq!(leaf.ebx, 2100);
        assert_eq!(leaf.ecx, 2100);
        assert_eq!(leaf.edx, 0);
    }
}
