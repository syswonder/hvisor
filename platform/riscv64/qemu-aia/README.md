
# Qemu RISC-V with Advanced Interrupt Architecture (AIA)

If you are unfamiliar with the usage of hvisor config and hvisor-riscv, please refer to the corresponding JSON in scripts/boot_zone1.sh, which corresponds to board.rs.

For content not specified in board.rs, please refer to the comments in `platform/riscv64/qemu-plic/board.rs` and `platform/riscv64/README.md`.

Especially the explanation regarding HW_IRQS.

For current code, please confirm the zonex's IMSIC_S_BASE is the same as the global IMSIC_S_BASE.

In the current configuration, IOMMU provides interrupt remapping. Disabling the riscv_iommu feature will cause the system to fail to boot. If you disable riscv_iommu, please ensure that the virtual machine does not use MSI.
