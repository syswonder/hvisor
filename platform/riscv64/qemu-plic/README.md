# Platform Configuration for RISC-V QEMU

This document describes the platform setup for running **hvisor** on the RISC-V QEMU virtual machine.

## Configuration Files Overview

- **`platform.mk`**:  
  Defines QEMU parameters for the current platform, including:
  - Number of harts (CPU cores)
  - Memory size
  - Virtio peripherals
  - PCI devices

- **`board.rs`**:  
  Contains board-specific configuration for **hvisor**, including:
  - Root zone settings
  - General platform initialization parameters

## Environment Setup

> **Recommended QEMU version**: `7.2.0` (tested and verified)

### Prerequisites

- A compiled Linux kernel
- Two root filesystems (`rootfs`)

### Generate Device Tree Blob (DTB)

Run the following command to generate the device tree file:

```bash
make BID=riscv64/qemu-plic dtb
```

### Prepare Root Filesystem

Copy the following files into the root filesystem of your primary Linux instance:

- configs/virtio-backend.json

- configs/zone1-linux-virtio.json

- zone1-linux.dtb

- Compiled hvisor binary

- hvisor.ko kernel module

- Scripts from the scripts/ directory (optional)

### Boot Instructions

Boot into the root Linux shell.

Execute the boot script:

```bash
./boot_linux.sh
```

After the system starts, connect to the non-root zone’s virtual serial console using:

```bash
screen /dev/xxx (Replace xxx with the actual number.)
```

### Notes
In the current configuration, the virtio-console backend runs inside root Linux, not in QEMU.
Both the root zone and zone1 are assigned one virtio-pci-blk device(using wired-interupt) each.