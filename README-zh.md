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

hvisor 是一个用 Rust 实现的 Type-1 裸机虚拟机监控程序，采用分离内核设计，提供强大的硬件资源虚拟化和隔离功能。该虚拟机监控程序允许在不同区域之间严格分离系统环境，确保虚拟化环境中的性能和安全性。

🚧 该项目正在开发中

## 特性

- **分离内核设计**：虚拟机分为三个区域：zone0（管理）、zoneU（用户）和 zoneR（实时），区域之间严格隔离。
- **多平台支持**：支持多种架构，包括 aarch64、riscv64 和 loongarch64。
- **虚拟机管理**：虚拟机通过 zone0（root-linux）中的 Linux 环境进行管理，使用 [hvisor-tool](https://github.com/syswonder/hvisor-tool) 执行管理任务。
- **设备支持**：包括 virtio 设备、串口设备、中断控制器、PCIe 支持等。
  - virtio-blk (aarch64, riscv64), virtio-net (aarch64, riscv64), virtio-console (aarch64, riscv64, loongarch64)
  - 串口设备/UART：
    - PL011 (aarch64)
    - imx-uart (NXP i.MX8MP, aarch64)
    - NS16550A (Loongson 3A5000, loongarch64)
    - xuartps (Xilinx Ultrascale+ MPSoC ZCU102, aarch64)
  - 中断控制器:
    - GIC 中断控制器 (aarch64)
    - 7A2000 中断控制器 (loongarch64)
    - PLIC (riscv64)
    - APLIC（暂只支持 msi 模式）(riscv64)
  - PCIe 直通 (aarch64, riscv64)
  - GPU 直通 (NXP i.MX8MP, aarch64)
- **架构硬件特性支持**：
  - GICv2, GICv3 (aarch64)
  - ARM 虚拟化 (aarch64)
  - LVZ（龙芯虚拟化）扩展 (loongarch64)
  - H 扩展 (riscv64)
  - SMMUv3 (aarch64)

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

- [x] 龙芯 3A5000（7A2000 桥片）
- [ ] 龙芯 3A6000

## 快速开始

请参阅 hvisor 文档的《hvisor 快速上手指南》部分，了解所有支持平台的详细构建和运行教程：[hvisor 文档](https://hvisor.syswonder.org/)

## 路线图

- 支持在 NXP i.MX8MP 硬件平台上运行 Android nonroot zone
- 支持 hvisor 在 x86_64 架构上运行

## 致谢

该项目基于 [RVM1.5](https://github.com/rcore-os/RVM1.5) 和 [jailhouse](https://github.com/siemens/jailhouse)。