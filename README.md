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

hvisor is a Type-1 bare-metal hypervisor implemented in Rust, leveraging the separation kernel design to offer robust hardware resource virtualization and isolation. The hypervisor allows for strict separation of system environments across different zones, ensuring both performance and security in a virtualized environment.

<!-- 🚧 This project is work in progress -->

## Features

- **Separation Kernel Design**: Virtual machines are classified into three zones: zone0 (management), zoneU (user), and zoneR (real-time), with strict isolation between them.
- **Multi-Platform Support**: Works on a variety of architectures, including aarch64, riscv64, and loongarch64.
- **Virtual Machine Management**: VMs are managed through a Linux environment in zone0 (root-linux), where administrative tasks are performed using command-line tool [hvisor-tool](https://github.com/syswonder/hvisor-tool).
- **Device Support**: Includes virtio devices, serial devices, interrupt controllers, PCIe support, etc.
  - virtio-blk (aarch64), virtio-net (aarch64), virtio-console (aarch64, loongarch64), virtio-gpu(aarch64)
  - Serial devices/UARTs:
    - PL011 (aarch64)
    - imx-uart (NXP i.MX8MP, aarch64)
    - NS16550A (loongarch64)
    - xuartps (Xilinx Ultrascale+ MPSoC ZCU102, aarch64)
  - Interrupt controllers:
    - GIC irq controller (aarch64)
    - 7A2000 irq controller (loongarch64)
    - PLIC (riscv64)
    - AIA-APIC (hvisor now only support msi mode) (riscv64)
  - PCIe passthrough (aarch64)
  - GPU passthrough (NXP i.MX8MP, aarch64)
- **Architecture Hardware Features Support**: 
  - GICv2, GICv3 (aarch64)
  - ARM Virtualization (aarch64)
  - LVZ(Loongson Virtualization) Extension (loongarch64)
  - H extension (riscv64)
  - SMMUv3 (aarch64)


## Supported Platforms

### aarch64

- [x] QEMU virt aarch64
- [x] NXP i.MX8MP
- [x] Raspberry Pi 4B
- [x] Xilinx Ultrascale+ MPSoC ZCU102
- [ ] Rockchip RK3588

### riscv64

- [x] QEMU virt riscv64
- [ ] FPGA RocketChip

### loongarch64

- [x] Loongson 3A5000 with 7A2000 bridge
- [ ] Loongson 3A6000

## Getting Started

Please refer to the Quick Start Guide section of the hvisor documentation for detailed build and running tutorials for all supported platforms: [hvisor documentation](https://hvisor.syswonder.org/)

## Roadmap

- To support Android nonroot zone on NXP i.MX8MP hardware platform
- To support hvisor on x86_64 architecture

## Acknowledgement

This project is based on [RVM1.5](https://github.com/rcore-os/RVM1.5) and [jailhouse](https://github.com/siemens/jailhouse).
