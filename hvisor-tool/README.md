# README

README: [中文](./README-zh.md) | [English](./README.md)

This repository contains command-line tools and kernel modules associated with [hvisor](https://github.com/syswonder/hvisor). The command-line tools also include the Virtio daemon, which provides Virtio devices. The command-line tools and kernel modules need to be compiled separately and used in the root Linux zone0 of the virtual machine manager. The structure of the repository is as follows:

```
hvisor-tool
	-tools: Contains command-line tools and the Virtio daemon
	-driver: Kernel module corresponding to hvisor
```

## Compilation Steps

The following operations are performed under the `hvisor-tool` directory on an x86 host for cross-compilation.

* Compile command-line tools and kernel modules

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux 
```

Where `<arch>` should be one of `arm64` or `riscv`.

`<log>` can be one of `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, or `LOG_FATAL`, which controls the log level of the Virtio daemon.

`/path/to/your-linux` is the kernel source directory for root Linux. For specific compilation options, refer to the [Makefile](./Makefile), [tools/Makefile](./tools/Makefile), and [driver/Makefile](./driver/Makefile).

For example, to compile command-line tools for `arm64`, you can run:

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

This will generate the `tools/hvisor` binary and `driver/hvisor.ko` module, which can be copied to the root Linux file system for use.

## Usage Steps

### Kernel Module

Before using the command-line tools and Virtio daemon, you need to load the kernel module on zone0 to enable interaction between user-space programs and the hypervisor:

```
insmod hvisor.ko
```

To unload the kernel module, use the following command:

```
rmmod hvisor.ko
```

### Command-Line Tools

In root Linux-zone0, the command-line tool can be used to create and shut down other virtual machines.

* Start a new virtual machine

  hvisor-tool can start a new virtual machine using a configuration file:

  ```
  ./hvisor zone start <vm_config.json>
  ```

  `<vm_config.json>` is a file that describes the configuration of a virtual machine. For example:

  * [Start zone1 on QEMU-aarch64](./examples/qemu-aarch64/zone1_linux.json): When using this file to directly start zone1, you must first start the Virtio daemon, with the corresponding configuration file being [virtio_cfg.json](./examples/qemu-aarch64/virtio_cfg.json).

  * [Start zone1 on NXP-aarch64](./examples/nxp-aarch64/zone1_linux.json): Similar to the QEMU example, first start the Virtio daemon with [virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json).

* To shut down a virtual machine with ID 1:

```
./hvisor zone shutdown -id 1
```

* To list all currently running virtual machines:

```
./hvisor zone list
```

### Virtio Daemon

The Virtio daemon provides Virtio MMIO devices for virtual machines. Currently, it supports three types of devices: Virtio-blk, Virtio-net, and Virtio-console.

#### Prerequisites

To use the Virtio daemon, a node named `hvisor_device` must be added to the **device tree of Root Linux**, for example:

```dts
hvisor_device {
    compatible = "hvisor";
    interrupt-parent = <0x01>;
    interrupts = <0x00 0x20 0x01>;
};
```

This ensures that when the hypervisor injects interrupt number `32+0x20` into Root Linux, the interrupt handler registered in `hvisor.ko` will be invoked, waking up the Virtio daemon.

#### Starting and Creating Virtio Devices

On Root Linux, execute the following example commands:

```bash
// Ensure the daemon is started before launching the zones
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start <vm_config.json>
```

The command `nohup ... &` starts a daemon process, with its log output saved to the `nohup.out` file in the current directory.

`virtio_cfg.json` is a JSON file that describes Virtio devices, such as [virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json). This example file performs the following steps:

1. Address space mapping

Maps the RAM region of virtual machine `zone1` (ID 1) (starting at address `0x50000000`, size `0x30000000`) into the Virtio daemon's address space using `mmap`.

2. Create Virtio-blk device

Creates a Virtio-blk device, with `zone1` communicating with this device through an MMIO region starting at `0xa003c00` and a length of `0x200`. The device interrupt number is set to 78, and the disk image used is `rootfs2.ext4`.

3. Create Virtio-console device

Creates a Virtio-console device for `zone1`'s primary serial output. Root Linux can access this virtual console using the `screen /dev/pts/x` command, where `x` can be found in the `nohup.out` log file.

To return to the main console, press the shortcut `ctrl+a+d`. To re-enter the virtual console, run `screen -r [SID]`, where SID is the screen session ID.

4. Create Virtio-net device

Since the `status` attribute of the `net` device is set to `disable`, a Virtio-net device is not created. If the `status` attribute is set to `enable`, a Virtio-net device would be created with an MMIO region starting at `0xa003600`, length `0x200`, interrupt number 75, and MAC address `00:16:3e:10:10:10`, which would be used by the virtual machine with ID 1 and connected to the `tap0` Tap device.

#### Shutting Down Virtio Devices

To shut down the Virtio daemon and all created devices, run the following command:

```
pkill hvisor-virtio
```

For more information, such as configuring the root Linux environment, refer to: [Using Virtio Devices on hvisor](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial).
