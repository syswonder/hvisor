# README

README: [中文](./README-zh.md) | [English](./README.md)

This repository contains command-line tools and kernel modules associated with [hvisor](https://github.com/syswonder/hvisor). The command-line tools also include a Virtio daemon for providing Virtio devices. Both the command-line tools and kernel modules need to be compiled separately and used on the root Linux of the managing virtual machine. The overall structure of the repository is as follows:

```
hvisor-tool
    -tools: Contains command-line tools and the Virtio daemon
    -driver: Kernel module corresponding to hvisor
```

## Compilation Steps

The following operations are performed in the `hvisor-tool` directory on an x86 host for cross-compilation.

* Compile the command-line tools and kernel module

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux
```

Where `<arch>` should be either `arm64` or `riscv`.

`<log>` can be one of `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, or `LOG_FATAL`.

`/path/to/your-linux` is the kernel source directory of the root Linux. For specific compilation options, please refer to [Makefile](./Makefile), [tools/Makefile](./tools/Makefile), and [driver/Makefile](./driver/Makefile).

For example, to compile the command-line tools for `arm64`, you can execute:

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

This will generate `tools/hvisor` and `driver/hvisor.ko`, which can be copied to the root filesystem of the root Linux for use.

## Usage Steps

### Kernel Module

Before using the command-line tools and the Virtio daemon, you need to load the kernel module to facilitate interaction between user-space programs and the Hypervisor:

```
insmod hvisor.ko
```

To unload the kernel module, use:

```
rmmod hvisor.ko
```

### Command-line Tools

In root Linux, the command-line tools can be used to create and shutdown other virtual machines.

* **Prerequisites**

**Note:** Since the root Linux needs to load the non-root Linux image and dtb file into the physical memory region of the non-root Linux, it is necessary to modify the **device tree of the root Linux**:

1. The memory node should include the physical memory region of the non-root Linux, for example:

   ```c
   // Original device tree of root Linux:
   memory@50000000 {
       device_type = "memory";
       reg = <0x0 0x50000000 0x0 0x40000000>;
   };
   // Device tree of non-root Linux
   memory@90000000 {
       device_type = "memory";
       reg = <0x0 0x90000000 0x0 0x40000000>;
   };
   // Modified device tree of root Linux:
   memory@50000000 {
       device_type = "memory";
       reg = <0x0 0x50000000 0x0 0x80000000>;
   };
   ```

2. Add a reserved-memory node to set the memory region of the non-root Linux as reserved memory to prevent root Linux from using it, for example:

   ```
   reserved-memory {
       #address-cells = <0x02>;
       #size-cells = <0x02>;
       ranges;
       nonroot@90000000 {
           no-map;
           reg = <0x00 0x90000000 0x00 0x40000000>;
       };
   };
   ```

   Note, after adding the reserved-memory node, the bootargs of the root Linux kernel do not need to include information like `mem=1G`.

* Start a new virtual machine

hvisor-tool starts a new virtual machine through a configuration file:

```
./hvisor zone start <vm_config.json>
```

`<vm_config.json>` is a file describing the configuration of a virtual machine, such as [nxp_linux.json](./examples/nxp_linux.json).

> **Note: If you want to start a new virtual machine via the command line instead of a configuration file, please refer to [hvisor-tool_old](https://github.com/syswonder/hvisor-tool/commit/3478fc6720f89090c1b5aa913da168f49f95bca0)**. The command-line startup method will gradually be replaced by configuration files, so please upgrade to the latest hvisor-tool.

* Shutdown a virtual machine with id 1:

```
./hvisor zone shutdown -id 1
```

### Virtio Daemon

The Virtio daemon can provide Virtio MMIO devices for virtual machines, currently supporting three types of devices: Virtio-blk, Virtio-net, and Virtio-console devices.

* **Prerequisites**

To use the Virtio daemon, a node named `hvisor_device` needs to be added to the **device tree of root Linux**, for example:

```dts
hvisor_device {
    compatible = "hvisor";
    interrupt-parent = <0x01>;
    interrupts = <0x00 0x20 0x01>;
};
```

In this way, when hvisor injects an interrupt with number `32+0x20` into the root Linux, it will enter the interrupt handler registered in `hvisor.ko`, waking up the Virtio daemon.

* Start and create Virtio devices

On the root Linux, execute the following example command:

```
nohup ./hvisor virtio start virtio_cfg.json &
```

Where `virtio_cfg.json` is a JSON file describing Virtio devices, such as [virtio_cfg.json](./examples/virtio_cfg.json). This example file will sequentially perform:

1. First create a Virtio-blk device, where virtual machine with id 1 will communicate with this device through an MMIO region starting at address `0xa003c00` with a length of `0x200`. The device interrupt number is set to 78, and the corresponding disk image is `rootfs2.ext4`.
2. Then create a Virtio-console device for the output of the main serial port of the virtual machine with id 1. The root Linux needs to execute the `screen /dev/pts/x` command to enter this virtual console, where `x` can be viewed from the output information of nohup.out.
3. Since the `status` property of the `net` device is `disable`, the Virtio-net device will not be created. If the `status` property of the `net` device is `enable`, a Virtio-net device will be created with an MMIO region starting at `0xa003600` with a length of `0x200`, an interrupt number of 75, and a MAC address of `00:16:3e:10:10:10`, used by the virtual machine with id 1, connected to a Tap device named `tap0`.
4. The command `nohup ... &` indicates that this command will create a daemon process.

* Shutdown Virtio devices

Execute the following command to shut down the Virtio daemon and all created devices:

```
pkill hvisor
```

For more information, such as configuring the root Linux environment, please refer to: [Using Virtio Devices on hvisor](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial).