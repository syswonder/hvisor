<p align = "center">
<br><br>
<img src="https://www.syswonder.org/_media/hvisor-logo.svg">
<br><br>
<!-- <img src="https://img.shields.io/badge/hvisor-orange" /> -->
<a href="https://github.com/syswonder/hvisor/actions/workflows/ci.yml"><img src="https://github.com/syswonder/hvisor/actions/workflows/ci.yml/badge.svg?branch=dev" alt="CI" style="max-width: 100%;"></a>
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
- **Simple and Lightweight**: Implemented in Rust with a minimal design.
  - CPU Virtualization: Static partitioning of physical CPUs (pCPUs), without dynamic scheduling.
  - Memory Virtualization: Pre-allocated virtual machine memory space via configuration files.
  - I/O Virtualization: Supports device passthrough and virtio paravirtualization.
- **Multi-platform Support**: Supports various architectures, including `aarch64`, `riscv64`, `loongarch64` and `x86_64`.
- **Virtual Machine Management**: Virtual machines are managed through a Linux environment in zone0 (root-linux), with basic management tasks (create, start, stop, delete) handled via the command-line tool [hvisor-tool](https://github.com/syswonder/hvisor-tool).
- **Formal Verification**: Part of the hvisor code is undergoing formal verification using the [verus](https://github.com/verus-lang/verus) tool.

## Device Support

| **Category**                     | **Device**             | **Supported Architectures**                  | **Notes**                              |
|----------------------------------|------------------------|----------------------------------------------|----------------------------------------|
| **Virtio Devices**               | virtio-blk             | `aarch64`, `riscv64`, `loongarch64`,`x86_64` |                                        |
|                                  | virtio-net             | `aarch64`,`x86_64`                           |                                        |
|                                  | virtio-console         | `aarch64`, `riscv64`, `loongarch64`,`x86_64` |                                        |
|                                  | virtio-gpu             | `aarch64`                                    | QEMU only                              |
| **Serial Devices/UARTs**         | PL011                  | `aarch64`                                    |                                        |
|                                  | imx-uart               | `aarch64`                                    | NXP i.MX8MP                            |
|                                  | NS16550A               | `loongarch64`                                |                                        |
|                                  | xuartps                | `aarch64`                                    | Xilinx Ultrascale+ MPSoC ZCU102        |
|                                  | uart16550              | `aarch64`                                    | Rockchip RK3568/RK3588, Forlinx OK6254-C|
|                                  | uart16550a             | `x86_64`                                     |                                        |
| **Interrupt Controllers**        | GIC irq controller     | `aarch64`                                    |                                        |
|                                  | 7A2000 irq controller  | `loongarch64`                                |                                        |
|                                  | PLIC                   | `riscv64`                                    |                                        |
|                                  | AIA                    | `riscv64`                                    | MSI mode only                          |
|                                  | APIC                   | `x86_64`                                     |                                        |
| **Device Passthrough(Zone0)**    | All                    |  All                                         |                                        |
| **Device Passthrough(ZoneU)**    | PCIe                   | `aarch64`, `riscv64`, `loongarch64`,`x86_64` |                                        |
|                                  | GPU / HDMI             | `aarch64`, `loongarch64`                     | NXP i.MX8MP, 3A6000                    |
|                                  | eMMC                   | `aarch64`, `riscv64`                         | NXP i.MX8MP                            |
|                                  | USB                    | `aarch64`, `x86_64`                          | NXP i.MX8MP                            |
|                                  | SATA                   | `riscv64`, `loongarch64`, `x86_64`           | megrez, 3A6000                         |
|                                  | Ethernet               | `aarch64`, `riscv64`, `loongarch64`,`x86_64` | NXP i.MX8MP, megrez, 3A6000            |

## Supported Boards

### aarch64

- [x] QEMU virt aarch64
- [x] NXP i.MX8MP
- [x] Xilinx Ultrascale+ MPSoC ZCU102
- [x] Rockchip RK3588
- [x] Rockchip RK3568
- [x] Forlinx OK6254-C
- [x] Phytium Pi

### riscv64

- [x] QEMU virt riscv64
- [x] Milk-V Megrez 
- [x] Sifive Hifive Premier P550
- [x] dp-1000
- [ ] FPGA XiangShan(KunMingHu) on S2C Prodigy S7-19PS-2

### loongarch64

- [x] Loongson 3A5000 (7A2000 bridge chip)
- [x] Loongson 3A6000 (7A2000 bridge chip)

### x86_64

- [x] QEMU Q35
- [x] ASUS NUC14MNK
- [x] ECX-2300F-PEG

## Supported Guest OS

- [x] Linux 6.13
- [x] Zephyr AArch64
- [x] Zephyr AArch32
- [x] RT-Thread
- [ ] Android
- [ ] OpenHarmony

## Getting Started

Please refer to the hvisor documentation for quick start guides, build and run instructions for all supported platforms: [hvisor Documentation](https://hvisor.syswonder.org/)

## Roadmap
### Completed
- [CHANGELOG](./CHANGELOG.md)
- Support for USB zoneU passthrough
- Support for PCIe bus virtualization

### Planned
- Support for Android
- Support for OpenHarmony  
- Support for ARMv9
- Support for GICv4
- Support for Cache Coloring
- Support for SR-IOV
- Support for NPU zoneU passthrough
- Support for Nvidia GPU zoneU passthrough
- Web Management tool
- Device Tree configuration tool
- Support for Nvidia Orin
- Support for Nvidia Thor
- Support for Raspberry Pi 5
- Support for IOMMU virtualization
- Support for Clock Controller virtualization
- Support for pinctrl virtualization
- Support for booting zoneU / zoneR without zone0

## Acknowledgments

Some implementations of this project reference [RVM1.5](https://github.com/rcore-os/RVM1.5) and [jailhouse](https://github.com/siemens/jailhouse).