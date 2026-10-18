# Changelog

> ⚠️ Please update this file for any changes to the hvisor project along with your name and GitHub profile link under the CURRENT section.

## CURRENT - next release

- [refactor] **all archs**: Cache the per-CPU slot pointer in an architecture register (`TPIDR_EL2` / `IA32_GS_BASE` / root CSR `SAVE0` / `sscratch`) so `this_cpu_id()`/`this_cpu_data()` stop re-deriving it on every access, and remove the dead `percpu` crate with its `.percpu` link-time sections (23 boards). ([agicy](https://github.com/agicy))

## CURRENT - v0.6

- [feature] **PCIe/virtio**: Add emulated virtio PCI device support. ([PR #287](https://github.com/syswonder/hvisor/pull/287), [ZZJJWarth](https://github.com/ZZJJWarth))
- [ci] Update CI workflow, add license / format checks, restore manual workflow triggers, and update CI terminal output. ([PR #302](https://github.com/syswonder/hvisor/pull/302), [PR #306](https://github.com/syswonder/hvisor/pull/306), [PR #360](https://github.com/syswonder/hvisor/pull/360), [Xingyu Chen](https://github.com/dallasxy))
- [feature] **aarch64**: Extend VCPU state handling. ([PR #303](https://github.com/syswonder/hvisor/pull/303), [Jingyu Liu](https://github.com/liulog))
- [feature] **riscv64**: Add virtual **RISC-V IOMMU** support. ([PR #307](https://github.com/syswonder/hvisor/pull/307), [Jingyu Liu](https://github.com/liulog))
- [bugfix] Detect and diagnose hvisor memory region overlap with the root zone, and add overlap checks between MMIO passthrough and MMIO intercept handler regions. ([PR #311](https://github.com/syswonder/hvisor/pull/311), [PR #384](https://github.com/syswonder/hvisor/pull/384), [Inquisitor-201](https://github.com/Inquisitor-201), [Solicey](https://github.com/Solicey))
- [bugfix] **aarch64**: Use `dc civac` instead of `dc ivac` in `invalidate_dcache_range`. ([PR #312](https://github.com/syswonder/hvisor/pull/312), [Inquisitor-201](https://github.com/Inquisitor-201))
- [refactor] Remove redundant frame allocator wrappers, fix alignment parameter semantics, and fix `alloc_contiguous` alignment. ([PR #316](https://github.com/syswonder/hvisor/pull/316), [PR #359](https://github.com/syswonder/hvisor/pull/359), [Nehckl](https://github.com/Inquisitor-201), [Jingyu Liu](https://github.com/liulog))
- [refactor] Migrate platform options from Cargo features to **Kconfig**. ([PR #319](https://github.com/syswonder/hvisor/pull/319), [Xingyu Chen](https://github.com/dallasxy))
- [feature] **PCIe**: Add **DWC MSI** injection and **SR-IOV** support (VF enumeration, capability handler, ARI). ([PR #321](https://github.com/syswonder/hvisor/pull/321), [PR #358](https://github.com/syswonder/hvisor/pull/358), [Zhongkai Xu](https://github.com/ZhongkaiXu))
- [feature] **aarch64**: Support **OpenHarmony** on dayu200 platform. ([PR #357](https://github.com/syswonder/hvisor/pull/357), [Stone749990226](https://github.com/Stone749990226))
- [feature] **x86_64**: Boot **Asterinas** as zone1 via Multiboot2, with virtio-console support, x2APIC timer virtualization, IOAPIC fixes, and dedicated Jenkins CI. ([PR #362](https://github.com/syswonder/hvisor/pull/362), [PR #376](https://github.com/syswonder/hvisor/pull/376), [yyda](https://github.com/yydawx))
- [config] Complete PCI fields in zone examples. ([PR #363](https://github.com/syswonder/hvisor/pull/363), [Jaxtonmax](https://github.com/Jaxtonmax))
- [feature] **riscv64**: Add **RVA23** ISA features for QEMU platforms. ([PR #364](https://github.com/syswonder/hvisor/pull/364), [Jingyu Liu](https://github.com/liulog))
- [platform] **aarch64**: Add support for **Jetson Orin** with UEFI memory map virtualization. ([PR #366](https://github.com/syswonder/hvisor/pull/366), [Ren HangQi](https://github.com/ForeverYolo))
- [feature] **x86_64**: Add a hypercall to query the ECAM base. ([PR #377](https://github.com/syswonder/hvisor/pull/377), [yyda](https://github.com/yydawx))
- [platform] **riscv64**: Add initial support for **Spacemit k3-com260**. ([PR #378](https://github.com/syswonder/hvisor/pull/378), [Jingyu Liu](https://github.com/liulog))
- [feature] **rk3588 (sysoul_x3300)**: Add Linux NPU / GPU / display zone configurations via SCMI. ([PR #379](https://github.com/syswonder/hvisor/pull/379), [agicy](https://github.com/agicy))
- [feature] **loongarch64**: Make SMP IPI and Virtio IRQ delivery stateful, track guest-to-physical CPU mappings, and fix Virtio consuming too much CPU. ([PR #380](https://github.com/syswonder/hvisor/pull/380), [Xinhao Li](https://github.com/li041), [Stl_](https://github.com/weifenjihe))
- [ci] **aarch64**: Add hardware CI for rk3568 and phytium-pi. ([PR #381](https://github.com/syswonder/hvisor/pull/381), [Xingyu Chen](https://github.com/dallasxy))
- [bugfix] **loongarch64**: Preserve firmware PCI bridge bus numbers. ([PR #386](https://github.com/syswonder/hvisor/pull/386), [Stl_](https://github.com/weifenjihe))
- [infra] Support older Python environments in Kconfig tooling and fix the LoongArch release build. ([Xinhao Li](https://github.com/li041), [Stl_](https://github.com/weifenjihe))
- [bugfix] **aarch64**: Refine early boot assembly. ([Nehckl](https://github.com/Inquisitor-201))

## History Release

## hvisor - v0.5

- [platform] **aarch64**: Add support for **sysoul_x3300**. ([agicy](https://github.com/agicy))
- [feature] **aarch64**: Add dual-zone Linux / Android deployment. ([agicy](https://github.com/agicy))
- [feature] **riscv64**: Add initial support for **RISC-V IOMMU**. ([Jingyu Liu](https://github.com/liulog))
- [feature] **riscv-iommu**: Add command queue support. ([Jingyu Liu](https://github.com/liulog))
- [feature] **riscv64/PCIe**: Support MSI irq-remapping on `qemu-aia`. ([Jingyu Liu](https://github.com/liulog))
- [feature] **riscv64/PCIe**: Improve virtual PCI support on `qemu-aia`. ([Jingyu Liu](https://github.com/liulog))
- [feature] **PCIe**: Improve bus enumeration, validate firmware bus range, and extend config space to 4 KB. ([wheatfox](https://github.com/enkerewpo))
- [bugfix] **PCIe**: Fix subsystem resource leakage. ([Xingyu Chen](https://github.com/dallasxy))
- [refactor] Decouple IOMMU implementation from architecture-specific code and reorganize related modules. ([Jingyu Liu](https://github.com/liulog))
- [refactor] Separate lock-free fields from `Zone` and improve encapsulation. ([Xinhao Li](https://github.com/li041))
- [bugfix] Improve page-table rollback behavior on allocation failure. ([Xinhao Li](https://github.com/li041))
- [bugfix] Fix linear map merge errors and zone0 startup data abort on **rk3588**. ([Xinhao Li](https://github.com/li041))
- [bugfix] Fix zone interface handling in `arch_zone_reset`. ([Xinhao Li](https://github.com/li041))
- [bugfix] Address several PCI / GIC related regressions. ([Zhongkai Xu](https://github.com/ZhongkaiXu))
- [bugfix] Fix lost-wakeup ordering issues and improve synchronization robustness in cross-core wakeup paths. ([agicy](https://github.com/agicy))
- [ci] Add **sysoul_x3300** to the build matrix. ([agicy](https://github.com/agicy))
- [ci] Expand board build / test coverage. ([Jingyu Liu](https://github.com/liulog))
- [ci] Add CI / CD support for **x86_64**. ([Tianhong Liu](https://github.com/Solicey))
- [ci] Add performance benchmarking scripts for QEMU platforms. ([Xinhao Li](https://github.com/li041))

## hvisor - v0.4

- [platform] **x86**: Add support for ECX-2300F-PEG. ([Tianhong Liu](https://github.com/Solicey))
- [platform] **riscv64**: Add support for dp-1000. ([Jingyu Liu](https://github.com/liulog))
- [feature] **PCIe**: Add support for ecam/dwc/loongarch PCIe. ([Xingyu Chen](https://github.com/dallasxy))
- [feature] Improve store style of interrupt. ([agicy](https://github.com/agicy))
- [feature] Clarify hvisor memory layout. ([agicy](https://github.com/agicy))
- [feature] **riscv64**: support logical cpu id for riscv ([Jingyu Liu](https://github.com/liulog))
- [refactor] Improve per_cpu struct. ([Xinhao Li](https://github.com/li041))
- [bugfix] Corrupted cpuid retrieved by boot_cpuid_get in Release mode. ([Xinhao Li](https://github.com/li041))
- [bugfix] **aarch64**: Add support for cmd MOVI. ([Zhongkai Xu](https://github.com/ZhongkaiXu))
- [bugfix] **virtio**: Slove the race conditions caused by spinlock. ([Jingyu Liu](https://github.com/liulog))
- [ci] Update dependencies, add ccache support, and improve build/tooling workflows. ([Xingyu Chen](https://github.com/dallasxy),[Jingyu Liu](https://github.com/liulog),[wheatfox](https://github.com/enkerewpo))

## hvisor - v0.3

- [platform] **x86**: Added comprehensive support for the x86_64 architecture. ([Tianhong Liu](https://github.com/Solicey))
- [bugfix] **aarch64**: IOMMU & ITS Improvements ([Zhongkai Xu](https://github.com/ZhongkaiXu))

## hvisor - v0.2

- [platform] **riscv64**: Add support for Megrez / Milk-V platforms (zone0/zone1 boot, uart2, virtio, Ethernet, SATA passthrough, NPU, updated device-tree). ([Jingyu Liu](https://github.com/liulog))
- [platform] **riscv64**: Add support for SiFive HiFive Premier P550. ([Jingyu Liu](https://github.com/liulog))
- [platform] **aarch64**: Add support for Phytium-Pi. ([Zixu Bao](https://github.com/Baozixu99))
- [platform] **aarch64**: Improve QEMU GICv2/GICv3 configurations and add zone1-linux support. ([agicy](https://github.com/agicy))
- [platform] **loongarch64**: Add support for Loongson 3A5 / 3A6 platforms and improve clock and trap handling. ([wheatfox](https://github.com/enkerewpo))
- [feature] Add **aarch32** support. ([Guowei Li](https://github.com/KouweiLee))
- [feature] **riscv64** enhancements: g-stage dynamic detection, hypervisor_v0_6 (EIC770x SoC), and syscrg emulation. ([Jingyu Liu](https://github.com/liulog))
- [infra] Unify UART / MPIDR mapping, centralize IOMMU configuration, remove redundant arch feature flags, and tidy Cargo/zone/hypercall code. ([Nehckl](https://github.com/Inquisitor-201), [Ren HangQi](https://github.com/ForeverYolo))
- [infra/tool] **aarch64**: Optimized the structure of GIC parameters ([Ren HangQi](https://github.com/ForeverYolo))
- [ci/misc] Update dependencies, add ccache support, and improve build/tooling workflows. ([Jingyu Liu](https://github.com/liulog))

### hvisor v0.1.2

- [feature] riscv64: add virtio support in qemu-aia to boot zone1. ([CHonghao](https://github.com/CHonghaohao))
- [feature] pci support for loongarch64 ([wheatfox](https://github.com/enkerewpo), [Zhongkai Xu](https://github.com/ZhongkaiXu))
- [ci] support running CI with the latest hvisor-tool and the configuration files in hvisor ([CHonghao](https://github.com/CHonghaohao))
- [platform] support for rk3568 ([Xingyu Chen](https://github.com/dallasxy))
- [feature] riscv64: add virtio support ([Jingyu Liu](https://github.com/liulog))
- [feature] riscv64: add vplic struct ([Jingyu Liu](https://github.com/liulog))
- [feature] riscv64: add aclint support ([Jingyu Liu](https://github.com/liulog))

### hvisor v0.1.1

- [platform] seperate board definitions into `platform` folder with re-designed cargo feature system for hvisor ([wheatfox](https://github.com/enkerewpo))

### hvisor v0.1.0

- [platform] architecture officially supported: riscv64, loongarch64 ([Jingyu Liu](https://github.com/liulog), [wheatfox](https://github.com/enkerewpo))
- [tool] adapting hvisor-tool virtio-gpu, virtio-console ([KouweiLee](https://github.com/KouweiLee), [Roxy](https://github.com/Misaka19986), [wheatfox](https://github.com/enkerewpo))
- [bugfix] refactor aarch64 pagetable code ([Xingyu Chen](https://github.com/dallasxy))
- [platform] Xilinx Ultrascale+ ZCU102 PS processor support ([Ren HangQi](https://github.com/ForeverYolo))
- [platform] Loongson 3A5000+7A2000 support ([wheatfox](https://github.com/enkerewpo), [BoneInscri](https://github.com/BoneInscri))
- [feature] SMMUv3 support ([Zhongkai Xu](https://github.com/ZhongkaiXu))
- [feature] PCIe support ([Zhongkai Xu](https://github.com/ZhongkaiXu), [Xingyu Chen](https://github.com/dallasxy), [Ren HangQi](https://github.com/ForeverYolo))
- [feature] network interface card support ([Ren HangQi](https://github.com/ForeverYolo))
- [feature] riscv64: IOMMU support ([Jingyu Liu](https://github.com/liulog))
- [feature] aarch64: GICv2 support ([Ren HangQi](https://github.com/ForeverYolo))
- [feature] basic inter-vm communication(ivc) support ([KouweiLee](https://github.com/KouweiLee))
- [test] unittest and github ci support ([wheatfox](https://github.com/enkerewpo))
- [tool] hvisor-tool: support virtio-console, virtio-blk, virtio-net ([KouweiLee](https://github.com/KouweiLee))
- [platform] basic support for riscv64 ([likey99](https://github.com/likey99))
- [tool] aarch64: management tool in root zone linux, can create, stop, suspend and destroy working zones ([KouweiLee](https://github.com/KouweiLee))
- [platform] basic support for aarch64 with root and nonroot zone booting ([Nehckl](https://github.com/Inquisitor-201))
