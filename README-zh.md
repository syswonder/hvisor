<p align = "center">
<br><br>
<img src="https://www.syswonder.org/_media/hvisor-logo.svg">
<br><br>
<!-- <img src="https://img.shields.io/badge/hvisor-orange" /> -->
<a href="https://github.com/syswonder/hvisor/actions/workflows/ci.yml"><img src="https://github.com/syswonder/hvisor/actions/workflows/ci.yml/badge.svg?branch=dev" alt="CI" style="max-width: 100%;"></a>
<img src="https://img.shields.io/github/stars/syswonder/hvisor?color=yellow" />
<img src="https://img.shields.io/github/license/syswonder/hvisor?color=red" />
<img src="https://img.shields.io/github/contributors/syswonder/hvisor?color=blue" />
<img src="https://img.shields.io/github/languages/code-size/syswonder/hvisor?color=green">
<img src="https://img.shields.io/github/repo-size/syswonder/hvisor?color=white">
<img src="https://img.shields.io/github/languages/top/syswonder/hvisor?color=orange">
<br><br>
</p>

README：[中文](./README-zh.md) | [English](./README.md)

hvisor 是一个用 Rust 实现的 Type-1 裸机虚拟机监控器，采用分离内核（Separation Kernel）设计，提供高效的硬件资源虚拟化和隔离能力。该虚拟机监控器实现了严格的虚拟机环境分离，通过不同的区域（zone）确保虚拟化环境的性能和安全性。

## 特性

- **分离内核设计**：虚拟机被划分为三个区域：zone0（管理区）、zoneU（用户区）、zoneR（实时区），之间有严格的隔离。
- **简洁轻量**：该虚拟机监控器采用 Rust 实现，具有简洁的设计。
  - CPU 虚拟化：静态分区的物理 CPU（pCPUs），不进行动态调度。
  - 内存虚拟化：通过配置文件对虚拟机内存空间进行预分配。
  - I/O 虚拟化：支持设备直通和 virtio 半虚拟化。
- **多平台支持**：支持多种架构，包括 `aarch64`、`riscv64` 和 `loongarch64`。
- **虚拟机管理**：虚拟机通过 zone0（root-linux）中的 Linux 环境进行管理，管理任务通过命令行工具 [hvisor-tool](https://github.com/syswonder/hvisor-tool) 完成，提供创建、启动、停止和删除虚拟机的基本管理功能。
- **形式化验证**：部分 hvisor 代码正在使用 [verus](https://github.com/verus-lang/verus) 工具进行形式化验证。

## 设备支持

| **类别**           | **设备**              | **支持架构**                         | **备注                                    |
| ------------------ | --------------------- | ----------------------------------- | ---------------------------------------- |
| **Virtio 设备**    | virtio-blk            | `aarch64`, `riscv64`                |                                          |
|                    | virtio-net            | `aarch64`                           |                                          |
|                    | virtio-console        | `aarch64`, `riscv64`, `loongarch64` |                                          |
|                    | virtio-gpu            | `aarch64`                           | 仅支持 QEMU                               |
| **串行设备/UARTs** | PL011                 | `aarch64`                            |                                          |
|                    | imx-uart              | `aarch64`                           | NXP i.MX8MP                              |
|                    | NS16550A              | `loongarch64`                       |                                          |
|                    | xuartps               | `aarch64`                           | Xilinx Ultrascale+ MPSoC ZCU102          |
|                    | uart16550             | `aarch64`                           | Rockchip RK3568/RK3588, Forlinx OK6254-C |
| **中断控制器**     | GIC irq controller    | `aarch64`                            |                                          |
|                    | 7A2000 irq controller | `loongarch64`                       |                                          |
|                    | PLIC                  | `riscv64`                           |                                          |
|                    | AIA                   | `riscv64`                           | 仅支持 MSI 模式                           |
| **PCIe 直通**      | PCIe                  | `aarch64`, `riscv64`                |                                          |
| **GPU 直通**       | GPU                   | `aarch64`                           | NXP i.MX8MP                              |

## 板卡支持

### aarch64

- [x] QEMU virt aarch64
- [x] NXP i.MX8MP
- [x] Xilinx Ultrascale+ MPSoC ZCU102
- [x] Rockchip RK3588
- [x] Rockchip RK3568
- [x] Forlinx OK6254-C

### riscv64

- [x] QEMU virt riscv64
- [x] Milk-V Megrez
- [x] Sifive Hifive Premier P550
- [ ] FPGA 香山（昆明湖）on S2C Prodigy S7-19PS-2
- [ ] FPGA  RocketChip on Xilinx Ultrascale+ MPSoC ZCU102

### loongarch64

- [x] Loongson 3A5000+7A2000
- [ ] Loongson 3A6000

## 开始使用

请参阅 hvisor 文档中的 **《hvisor 快速上手指南》**，了解所有支持平台的构建和运行教程：[hvisor 文档](https://hvisor.syswonder.org/)

## 路线图

- 支持在 NXP i.MX8MP 硬件平台上实现 Android nonroot
- 支持在 `x86_64` 架构上运行 hvisor

## 致谢

本项目的部分实现参考了 [RVM1.5](https://github.com/rcore-os/RVM1.5) 和 [jailhouse](https://github.com/siemens/jailhouse)。