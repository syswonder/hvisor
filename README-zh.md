<!-- # hvisor  -->

<p align = "center">
<br><br>
<img src="https://www.syswonder.org/_media/hvisor-logo.svg">
<br><br>
<!-- <img src="https://img.shields.io/badge/hvisor-orange" /> -->
<img src="https://img.shields.io/github/stars/syswonder/hvisor?color=yellow" />
<img src="https://img.shields.io/github/license/syswonder/hvisor?color=red" />
<img src="https://img.shields.io/github/contributors/syswonder/hvisor?color=blue" />
<img src="https://img.shields.io/github/languages/code-size/syswonder/hvisor?color=green">
<img src="https://img.shields.io/github/repo-size/syswonder/hvisor?color=white">
<img src="https://img.shields.io/github/languages/top/syswonder/hvisor?color=orange">
<br><br>
</p>

README：[中文](./README-zh.md) | [English](./README.md)

hvisor 是一个用 Rust 实现的 Type-1 裸金属（bare-metal）虚拟机管理器（hypervisor），基于分离内核（Separation Kernel）设计，提供强大的硬件资源虚拟化与隔离能力。该虚拟机管理器通过不同的区域对系统环境进行严格分离，确保虚拟化环境下的高性能与高安全性。

<!-- 🚧 本项目仍在开发中 -->

## 特性

- **分离内核设计**：虚拟机被划分为三个区域：zone0（管理区）、zoneU（用户区）和 zoneR（实时区），并保持严格的隔离。
- **多平台支持**：可运行于多种架构，包括 aarch64、riscv64 和 loongarch64。
- **虚拟机管理**：在 zone0（root-linux）中通过 Linux 环境管理虚拟机，并使用命令行工具 [hvisor-tool](https://github.com/syswonder/hvisor-tool) 进行操作。
- **设备支持**：支持 virtio 设备、串行设备、中断控制器、PCIe 直通等功能。
  - virtio-blk（aarch64）、virtio-net（aarch64）、virtio-console（aarch64、loongarch64）
  - 串行设备/UART：
    - PL011（aarch64）
    - imx-uart（NXP i.MX8MP，aarch64）
    - NS16550A（loongarch64）
    - xuartps（Xilinx Ultrascale+ MPSoC ZCU102，aarch64）
  - 中断控制器：
    - GIC 中断控制器（aarch64）
    - 7A2000 中断控制器（loongarch64）
    - PLIC（riscv64）
    - AIA-APIC（目前仅支持 MSI 模式）（riscv64）
  - PCIe 直通（aarch64、riscv64）
  - GPU 直通（NXP i.MX8MP，aarch64）
- **架构级硬件特性支持**：
  - GICv2、GICv3（aarch64）
  - ARM 虚拟化扩展（aarch64）
  - LVZ（Loongson Virtualization）虚拟化扩展（loongarch64）
  - H 扩展（riscv64）
  - SMMUv3（aarch64）

## 支持的平台

### aarch64

- [x] QEMU virt aarch64
- [x] NXP i.MX8MP
- [x] 树莓派 4B
- [x] Xilinx Ultrascale+ MPSoC ZCU102
- [ ] Rockchip RK3588

### riscv64

- [x] QEMU virt riscv64
- [ ] FPGA RocketChip

### loongarch64

- [x] 龙芯 3A5000 搭配 7A2000 桥接芯片
- [ ] 龙芯 3A6000

## 快速开始

请参考 hvisor 文档中的 **hvisor 快速上手指南**，获取各个平台的详细构建和运行教程：[hvisor 文档](https://hvisor.syswonder.org/)

## 未来规划

- 在 NXP i.MX8MP 硬件平台上支持 Android 非 root 区域
- 使 hvisor 兼容 x86_64 架构

## 声明

本项目基于 [RVM1.5](https://github.com/rcore-os/RVM1.5) 和 [jailhouse](https://github.com/siemens/jailhouse) 进行开发。