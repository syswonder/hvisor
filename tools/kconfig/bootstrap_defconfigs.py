#!/usr/bin/env python3
"""Generate platform/<arch>/<board>/kconfig/defconfig from legacy cargo/features lines."""
from __future__ import annotations

import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]

# platform/<arch>/... -> Kconfig arch choice line
ARCH_LINE: dict[str, str] = {
    "aarch64": "CONFIG_ARCH_AARCH64=y",
    "riscv64": "CONFIG_ARCH_RISCV64=y",
    "loongarch64": "CONFIG_ARCH_LOONGARCH64=y",
    "x86_64": "CONFIG_ARCH_X86_64=y",
}
    "gicv2": ["CONFIG_IRQ_GICV2=y"],
    "gicv3": ["CONFIG_IRQ_GICV3=y"],
    "pl011": ["CONFIG_PL011=y"],
    "imx_uart": ["CONFIG_IMX_UART=y"],
    "xuartps": ["CONFIG_XUARTPS=y"],
    "uart_16550": ["CONFIG_UART_16550=y"],
    "uart16550a": ["CONFIG_UART16550A=y"],
    "loongson_uart": ["CONFIG_LOONGSON_UART=y"],
    "pci": ["CONFIG_PCI=y"],
    "ecam_pcie": ["CONFIG_PCIE_ECAM=y"],
    "dwc_pcie": ["CONFIG_PCIE_DWC=y"],
    "loongarch64_pcie": ["CONFIG_PCIE_LOONGARCH64=y"],
    "no_pcie_bar_realloc": ["CONFIG_NO_PCIE_BAR_REALLOC=y"],
    "arm_smmu": ["CONFIG_ARM_SMMU=y"],
    "intel_vtd": ["CONFIG_INTEL_VTD=y"],
    "riscv_iommu": ["CONFIG_RISCV_IOMMU=y"],
    "share_s2pt": ["CONFIG_SHARE_S2PT=y"],
    "aclint": ["CONFIG_ACLINT=y"],
    "plic": ["CONFIG_PLIC=y"],
    "dp1000_plic": ["CONFIG_DP1000_PLIC=y"],
    "aia": ["CONFIG_AIA=y"],
    "sstc": ["CONFIG_SSTC=y"],
    "eic770x_soc": ["CONFIG_EIC770X_SOC=y"],
    "eic7700_sysreg": ["CONFIG_EIC7700_SYSREG=y"],
    "sifive_ccache": ["CONFIG_SIFIVE_CCACHE=y"],
    "loongson_7a2000": ["CONFIG_LOONGSON_7A2000=y"],
    "loongson_3a5000": ["CONFIG_LOONGSON_3A5000=y"],
    "loongson_3a6000": ["CONFIG_LOONGSON_3A6000=y"],
    "graphics": ["CONFIG_GRAPHICS=y"],
    "split_screen": ["CONFIG_SPLIT_SCREEN=y"],
}


def features_to_configs(lines: list[str]) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for raw in lines:
        w = raw.strip()
        if not w:
            continue
        if w not in FEATURE_LINES:
            raise SystemExit(f"unknown feature token: {w!r}")
        for ln in FEATURE_LINES[w]:
            if ln not in seen:
                seen.add(ln)
                out.append(ln)
    return sorted(out)


def main() -> None:
    for feat_path in sorted(ROOT.glob("platform/*/")):
        arch = feat_path.name
        for board_path in sorted(feat_path.glob("*/")):
            board = board_path.name
            legacy = board_path / "cargo" / "features"
            if not legacy.is_file():
                continue
            lines = legacy.read_text().splitlines()
            configs = features_to_configs(lines)
            arch_ln = ARCH_LINE.get(arch)
            if arch_ln is None:
                raise SystemExit(f"unknown arch directory {arch!r}")
            configs = sorted(set([*configs, arch_ln]))
            out_dir = board_path / "kconfig"
            out_dir.mkdir(parents=True, exist_ok=True)
            (out_dir / "defconfig").write_text("\n".join(configs) + "\n")
            print(f"wrote {out_dir / 'defconfig'} ({len(configs)} symbols)")


if __name__ == "__main__":
    main()
