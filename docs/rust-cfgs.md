# hvisor 中使用的 Rust `cfg` 一览

本文说明 **由 Kconfig / `.config` 注入的自定义 `cfg` 名**（经 [`kconfig/cfg_map.toml`](../kconfig/cfg_map.toml) 与 [`build.rs`](../build.rs) 转为 `cargo:rustc-cfg=...`），以及源码里实际出现的 **`#[cfg(...)]` / `cfg!(...)`** 用法。自定义名须与 **`cfg_map.toml` 右侧字符串** 完全一致（否则 `build.rs` 会告警或 rust-analyzer 不一致）。

---

## 1. 自定义 `cfg`（Kconfig → `rustc --cfg`）

下表 **`cfg` 列**为在 Rust 里写的标识符；**`CONFIG_*` 列**为 `.config` / defconfig 中的键。

| `cfg` 名 | `CONFIG_*` | 含义（简要） | 在 `src/` 中的使用 |
|----------|--------------|----------------|-------------------|
| `irq_gicv2` | `CONFIG_IRQ_GICV2` | AArch64 GICv2 | [`device/irqchip/mod.rs`](../src/device/irqchip/mod.rs)、[`pci/mod.rs`](../src/pci/mod.rs)（`compile_error` 互斥） |
| `irq_gicv3` | `CONFIG_IRQ_GICV3` | AArch64 GICv3 | 同上 |
| `plic` | `CONFIG_PLIC` | RISC-V PLIC | [`device/irqchip`](../src/device/irqchip/mod.rs)、[`arch/riscv64/trap.rs`](../src/arch/riscv64/trap.rs)、[`ipi.rs`](../src/arch/riscv64/ipi.rs) |
| `dp1000_plic` | `CONFIG_DP1000_PLIC` | PLIC 变体 | [`plic/plic.rs`](../src/device/irqchip/plic/plic.rs) `cfg!` |
| `aia` | `CONFIG_AIA` | RISC-V AIA | irqchip、trap |
| `aclint` | `CONFIG_ACLINT` | RISC-V ACLINT | irqchip、ipi |
| `sstc` | `CONFIG_SSTC` | RISC-V Sstc | [`arch/riscv64/cpu.rs`](../src/arch/riscv64/cpu.rs) `cfg!` |
| `eic770x_soc` | `CONFIG_EIC770X_SOC` | EIC770x SoC | irqchip `mmio_init` |
| `eic7700_sysreg` | `CONFIG_EIC7700_SYSREG` | EIC7700 系统寄存器 | [`device/mod.rs`](../src/device/mod.rs)、irqchip |
| `sifive_ccache` | `CONFIG_SIFIVE_CCACHE` | SiFive ccache | 同上 |
| `loongson_7a2000` | `CONFIG_LOONGSON_7A2000` | LoongArch 7A2000 中断 | **当前 `src/` 中无 `#[cfg(loongson_7a2000)]`**；LoongArch 中断路径由 **`target_arch = "loongarch64"`** 统一包含 [`ls7a2000`](../src/device/irqchip/ls7a2000)。该符号仍可由 Kconfig/defconfig 打开并传给 `rustc`，供后续或外设条件使用。 |
| `loongson_3a5000` | `CONFIG_LOONGSON_3A5000` | 3A5000 CPU 画像 | **当前 `src/` 中无引用**（仅映射与 defconfig） |
| `loongson_3a6000` | `CONFIG_LOONGSON_3A6000` | 3A6000 CPU 画像 | **当前 `src/` 中无引用** |
| `pl011` | `CONFIG_PL011` | ARM PL011 UART | [`device/uart/mod.rs`](../src/device/uart/mod.rs) |
| `imx_uart` | `CONFIG_IMX_UART` | i.MX UART | 同上 |
| `xuartps` | `CONFIG_XUARTPS` | Xilinx XUARTPS | 同上 |
| `uart_16550` | `CONFIG_UART_16550` | 16550 通用 | 同上 |
| `uart16550a` | `CONFIG_UART16550A` | 16550A（x86） | 同上、[`uart16550a.rs`](../src/device/uart/uart16550a.rs) |
| `loongson_uart` | `CONFIG_LOONGSON_UART` | Loongson UART | uart/mod |
| `pci` | `CONFIG_PCI` | PCI/PCIe 总开关 | [`main.rs`](../src/main.rs)、[`platform/mod.rs`](../src/platform/mod.rs)、[`zone.rs`](../src/zone.rs) 等 |
| `ecam_pcie` | `CONFIG_PCIE_ECAM` | ECAM 配置空间 | [`pci/*`](../src/pci/)、[`pci/config_accessors`](../src/pci/config_accessors/mod.rs)、`compile_error` 互斥 |
| `dwc_pcie` | `CONFIG_PCIE_DWC` | DesignWare PCIe | 同上、[`pci_handler.rs`](../src/pci/pci_handler.rs)、[`zone.rs`](../src/zone.rs) |
| `loongarch64_pcie` | `CONFIG_PCIE_LOONGARCH64` | LoongArch PCIe | pci 各模块、`config_accessors` |
| `no_pcie_bar_realloc` | `CONFIG_NO_PCIE_BAR_REALLOC` | 不重分配 BAR | [`pci_struct.rs`](../src/pci/pci_struct.rs)、[`pci_config.rs`](../src/pci/pci_config.rs) |
| `iommu` | `CONFIG_IOMMU` | IOMMU 总开关 | [`main.rs`](../src/main.rs)、[`zone.rs`](../src/zone.rs)、[`pci_config.rs`](../src/pci/pci_config.rs)、iommu 子模块等 |
| `share_s2pt` | `CONFIG_SHARE_S2PT` | 与 IOMMU 共享 Stage-2 页表 | [`pci_config.rs`](../src/pci/pci_config.rs) |
| `arm_smmu` | `CONFIG_ARM_SMMU` | ARM SMMU | [`device/iommu`](../src/device/iommu/) |
| `intel_vtd` | `CONFIG_INTEL_VTD` | Intel VT-d | iommu、[`arch/x86_64`](../src/arch/x86_64/)、[`pci_handler.rs`](../src/pci/pci_handler.rs)、[`irqchip/pic`](../src/device/irqchip/pic/mod.rs) |
| `riscv_iommu` | `CONFIG_RISCV_IOMMU` | RISC-V IOMMU | iommu |
| `graphics` | `CONFIG_GRAPHICS` | 帧缓冲 / 图形 | [`logging.rs`](../src/logging.rs)、[`arch/x86_64/entry.rs`](../src/arch/x86_64/entry.rs)、[`pio.rs`](../src/arch/x86_64/pio.rs)、[`uart16550a.rs`](../src/device/uart/uart16550a.rs) |
| `split_screen` | `CONFIG_SPLIT_SCREEN` | 分屏布局 | [`arch/x86_64/graphics.rs`](../src/arch/x86_64/graphics.rs)、[`boot.rs`](../src/arch/x86_64/boot.rs) |
| `print_timestamp` | `CONFIG_PRINT_TIMESTAMP` | 日志打时间戳 | [`logging.rs`](../src/logging.rs) |
| `extioi_debug` | `CONFIG_EXTIOI_DEBUG` | LoongArch EXTIOI 调试 | [`device/irqchip/ls7a2000/chip.rs`](../src/device/irqchip/ls7a2000/chip.rs) |

**说明**：[`build.rs`](../build.rs) 会为 `cfg_map.toml` 中**每一个**映射值打印 `cargo:rustc-check-cfg=cfg(...)`，因此上表中的名即使暂未出现在 `#[cfg]` 里，也属于**合法自定义 cfg**（例如便于 `rust-analyzer` 与将来代码一致）。

此外，**`platform/*/*/board.rs`** 中也可能出现 **`#[cfg(all(graphics))]`** 等（随板级代码演进）。

---

## 2. 编译器内置 / 标准 `cfg`（非 Kconfig）

源码中大量使用的内置条件（**不由** `cfg_map.toml` 提供）：

| 条件 | 典型用途 |
|------|-----------|
| `target_arch = "aarch64"` / `"riscv64"` / `"loongarch64"` / `"x86_64"` | 架构分支，常与上述自定义 `cfg` 组合为 `all(...)` |
| `test` | 单元测试、[`main.rs`](../src/main.rs)、[`panic.rs`](../src/panic.rs)、[`pci/mod.rs`](../src/pci/mod.rs)、[`tests.rs`](../src/tests.rs) |
| `target_pointer_width = "32"` / `"64"` | [`arch/x86_64/vmcs.rs`](../src/arch/x86_64/vmcs.rs) |
| `not(...)` | 与上述条件搭配，表示反向分支 |

---

## 3. `cfg!` 宏（编译期布尔）

| 表达式 | 文件 |
|--------|------|
| `cfg!(sstc)` | [`arch/riscv64/cpu.rs`](../src/arch/riscv64/cpu.rs) |
| `cfg!(iommu)` | [`zone.rs`](../src/zone.rs) |
| `cfg!(dp1000_plic)` | [`device/irqchip/plic/plic.rs`](../src/device/irqchip/plic/plic.rs) |

---

## 4. `compile_error!` 中使用的互斥组合（`src/pci/mod.rs`）

下列 **非法组合** 在编译期直接报错（与 Kconfig 中 PCIe `choice` 等互补）：

- `all(target_arch = "aarch64", irq_gicv2, irq_gicv3)` — GICv2 与 GICv3 不能同时启用  
- `all(ecam_pcie, dwc_pcie)`、`all(ecam_pcie, loongarch64_pcie)`、`all(dwc_pcie, loongarch64_pcie)` — PCIe 配置空间访问后端互斥  

---

## 5. 维护约定

1. **新增 Kconfig 开关并在 Rust 里使用**：在 [`kconfig/Kconfig`](../kconfig/Kconfig) 增加 `config`，在 [`kconfig/cfg_map.toml`](../kconfig/cfg_map.toml) 增加 `CONFIG_*` → **`cfg` 名**，源码写 **`#[cfg(该名)]`**；一般**无需**改 `build.rs`（已按映射表统一 `rustc-cfg` / `rustc-check-cfg`）。  
2. **重命名 `cfg`**：同时改 **`cfg_map.toml`**、全仓库 **`#[cfg` / `cfg!`** 与文档。  
3. **IDE**：`make vscode` 通过 [`tools/kconfig/kconfig_cli.py`](../tools/kconfig/kconfig_cli.py) `vscode-cfgs` 子命令，按 **`.config` + `cfg_map.toml`** 生成 `rust-analyzer.cargo.cfgs`，应与上表一致。

---

## 6. 生成方式（便于以后重新扫描）

在仓库根执行（仅供参考，输出需人工对照 Kconfig）：

```bash
rg '#\[cfg' src --no-heading -o | sed 's/#\[cfg(//;s/)\]//' | sort -u
rg 'cfg!\(' src --no-heading
```

以 **`cfg_map.toml`** 与 **`build.rs`** 为「配置侧真相」，以 **`rg '#\[cfg'`** 结果为「源码侧引用」；二者差异见上文 **§1 表内说明列**。
