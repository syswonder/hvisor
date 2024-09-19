# README

README: [中文](./README-zh.md) | [English](./README.md)

This repository contains command-line tools and kernel modules associated with [hvisor](https://github.com/syswonder/hvisor). The command-line tools also include the Virtio daemon, which provides Virtio devices. Both the command-line tools and the kernel module need to be compiled separately for use on the root Linux managing the virtual machine. The structure of the repository is as follows:

```
hvisor-tool
    - tools: contains command-line tools and the Virtio daemon
    - driver: kernel module corresponding to hvisor
```

## Compilation Steps

The following operations should be performed under the `hvisor-tool` directory on an x86 host for cross-compilation.

* Compile the command-line tools and kernel modules:

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux 
```

Where `<arch>` should be either `arm64` or `riscv`.

`<log>` can be one of `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, or `LOG_FATAL`, controlling the log output level of the Virtio daemon.

`/path/to/your-linux` is the kernel source directory of the root Linux. For specific compilation options, refer to [Makefile](./Makefile), [tools/Makefile](./tools/Makefile), and [driver/Makefile](./driver/Makefile).

For example, to compile command-line tools for `arm64`, run the following:

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

Afterwards, you can find the `tools/hvisor` and `driver/hvisor.ko` files. Copy them to the root Linux file system and use them.

## Usage Steps

### Kernel Module

Before using the command-line tools and Virtio daemon, load the kernel module to enable interaction between user-space programs and the hypervisor:

```
insmod hvisor.ko
```

To unload the kernel module:

```
rmmod hvisor.ko
```

### Command-Line Tools

On root Linux, the command-line tools can be used to create and shut down other virtual machines.

* **Pre-requisites**

**Note:** Since root Linux needs to load the Non-root Linux image and dtb file into the physical memory region for Non-root Linux, **modifications to the device tree of Root Linux** are required:

1. The memory node must include the physical memory region for Non-root Linux, for example:

   ```c
   // Original Root Linux device tree:
   memory@50000000 {
       device_type = "memory";
       reg = <0x0 0x50000000 0x0 0x40000000>;
   };
   // Non-root Linux device tree:
   memory@90000000 {
       device_type = "memory";
       reg = <0x0 0x90000000 0x0 0x40000000>;
   };
   // Modified Root Linux device tree:
   memory@50000000 {
       device_type = "memory";
       reg = <0x0 0x50000000 0x0 0x80000000>;
   };
   ```

2. Add a reserved-memory node to reserve the memory region for Non-root Linux and prevent Root Linux from using it, for example:

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

​    Note that after adding the reserved-memory node, the bootargs in Root Linux's kernel parameters do not need to include `mem=1G`.

* Start a new virtual machine

hvisor-tool starts a new virtual machine through a configuration file:

```
./hvisor zone start <vm_config.json>
```

`<vm_config.json>` is a file describing the configuration of a virtual machine, such as [nxp_linux.json](./examples/nxp_linux.json).

> **Note: If you want to start a new virtual machine via command-line instead of a configuration file, please refer to [hvisor-tool_old](https://github.com/syswonder/hvisor-tool/commit/3478fc6720f89090c1b5aa913da168f49f95bca0)**. Command-line startup will gradually be replaced by configuration files, so please upgrade to the latest hvisor-tool.

* Shut down the virtual machine with id 1:

```
./hvisor zone shutdown -id 1
```

* Print information about all current virtual machines:

```
./hvisor zone list
```

### Virtio Daemon

The Virtio daemon provides Virtio MMIO devices for virtual machines, currently supporting three types of devices: Virtio-blk, Virtio-net, and Virtio-console devices.

#### Pre-requisites

To use the Virtio daemon, add a node named `hvisor_device` to the **Root Linux device tree**, for example:

```dts
hvisor_device {
    compatible = "hvisor";
    interrupt-parent = <0x01>;
    interrupts = <0x00 0x20 0x01>;
};
```

This way, when hvisor injects an interrupt with number `32+0x20` into Root Linux, the interrupt handler registered in `hvisor.ko` will wake up the Virtio daemon.

#### Virtio Device Startup and Creation

On root Linux, execute the following command:

```c
// Start virtio daemon first, then start each zone
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start <vm_config.json>
```

Here, `nohup ... &` means the command will create a daemon process, and its log output will be saved in the `nohup.out` file in the current directory.

`virtio_cfg.json` is a JSON file describing Virtio devices, such as [virtio_cfg.json](./examples/virtio_cfg.json). This example file performs the following steps:

1. Address space mapping

The RAM memory region of virtual machine `zone1` (starting at `0x50000000`, size `0x30000000`) is mapped to the address space of the Virtio daemon via `mmap`.

2. Create a Virtio-blk device

A Virtio-blk device is created, and `zone1` communicates with the device through an MMIO region starting at `0xa003c00`, with a length of `0x200`. The device interrupt number is set to 78, and the disk image used is `rootfs2.ext4`.

3. Create a Virtio-console device

A Virtio-console device is created for the primary serial output of `zone1`. Root Linux needs to execute the `screen /dev/pts/x` command to access this virtual console, where `x` can be found in the `nohup.out` log file.

To return to the main console, press the shortcut `ctrl+a+d`. To reconnect to the virtual console, run `screen -r [SID]`, where SID is the process ID of the screen session.

4. Create a Virtio-net device

Since the `status` attribute of the `net` device is set to `disable`, no Virtio-net device is created. If the `status` attribute of the `net` device is set to `enable`, a Virtio-net device will be created with an MMIO region starting at `0xa003600`, with a length of `0x200`, interrupt number 75, and MAC address `00:16:3e:10:10:10`. It will be used by virtual machine id 1 and connected to a Tap device named `tap0`.

#### Shut Down Virtio Devices

Execute the following command to shut down the Virtio daemon and all created devices:

```
pkill hvisor-virtio
```

For more information, such as the root Linux environment configuration, refer to: [Using Virtio Devices on hvisor](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial).