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

README: [中文](./README-zh.md) | [English](./README.md)

hvisor is a Type-1 bare-metal virtual machine monitor implemented in Rust, featuring a separation kernel design to provide efficient hardware resource virtualization and isolation. This virtual machine monitor allows strict system environment separation, ensuring performance and security of the virtualized environments through distinct regions.

## Features

- **Separation Kernel Design**: The virtual machine is divided into three regions: zone0 (management zone), zoneU (user zone), and zoneR (real-time zone), with strict isolation between them.
- **Simple and Lightweight**: hvisor is implemented in Rust with a minimal design.
  - CPU Virtualization: Static partitioning of physical CPUs (pCPUs), without dynamic scheduling.
  - Memory Virtualization: Pre-allocated virtual machine memory space via configuration files.
  - I/O Virtualization: Supports device passthrough and virtio paravirtualization.
- **Multi-platform Support**: Supports various architectures, including `aarch64`, `riscv64`, and `loongarch64`.
- **Virtual Machine Management**: Virtual machines are managed through a Linux environment in zone0 (root-linux), with basic management tasks (create, start, stop, delete) handled via the command-line tool [hvisor-tool](https://github.com/syswonder/hvisor-tool).
- **Formal Verification**: Part of the virtual machine monitor code is undergoing formal verification using the [verus](https://github.com/verus-lang/verus) tool.

## Device Support

| **Category**              | **Device**            | **Supported Architectures** | **Notes**                       |
| ------------------------- | --------------------- | --------------------------- | ------------------------------- |
| **Virtio Devices**        | virtio-blk            | `aarch64`                   |                                 |
|                           | virtio-net            | `aarch64`                   |                                 |
|                           | virtio-console        | `aarch64`, `loongarch64`    |                                 |
|                           | virtio-gpu            | `aarch64`                   | Only supports QEMU              |
| **Serial Devices/UARTs**  | PL011                 | `aarch64`                   |                                 |
|                           | imx-uart              | `aarch64`                   | NXP i.MX8MP                     |
|                           | NS16550A              | `loongarch64`               |                                 |
|                           | xuartps               | `aarch64`                   | Xilinx Ultrascale+ MPSoC ZCU102 |
| **Interrupt Controllers** | GIC irq controller    | `aarch64`                   |                                 |
|                           | 7A2000 irq controller | `loongarch64`               |                                 |
|                           | PLIC                  | `riscv64`                   |                                 |
|                           | AIA-APIC              | `riscv64`                   | Only supports MSI mode          |
| **PCIe Passthrough**      | PCIe                  | `aarch64`, `riscv`          |                                 |
| **GPU Passthrough**       | GPU                   | `aarch64`                   | NXP i.MX8MP                     |

## Supported Boards

### aarch64

- [x] QEMU virt aarch64
- [x] NXP i.MX8MP
- [x] Xilinx Ultrascale+ MPSoC ZCU102
- [ ] Rockchip RK3588
- [ ] Rockchip RK3568
- [ ] Forlinx OK6254-C

### riscv64

- [x] QEMU virt riscv64
- [ ] FPGA XiangShan(KunMingHu) on S2C Prodigy S7-19PS-2
- [ ] FPGA  RocketChip on Xilinx Ultrascale+ MPSoC ZCU102

### loongarch64

- [x] Loongson 3A5000 and 7A2000 bridge chip
- [ ] Loongson 3A6000

## Getting Started

Please refer to the hvisor documentation for the quick start guide, which includes build and run instructions for all supported platforms: [hvisor Documentation](https://hvisor.syswonder.org/)

## Roadmap

- Support for Android non-root on the NXP i.MX8MP hardware platform
- Support for running hvisor on the `x86_64` architecture

## Acknowledgments

This project is based on [RVM1.5](https://github.com/rcore-os/RVM1.5) and [jailhouse](https://github.com/siemens/jailhouse).