# README  
README: [中文](./README-zh.md) | [English](./README.md)  

This repository contains the command-line tools and kernel modules associated with [hvisor](https://github.com/syswonder/hvisor). The command-line tools include the Virtio daemon for providing Virtio devices. Both the command-line tools and kernel modules need to be compiled separately and then used on the root Linux zone0 of the managed virtual machine. The structure of the repository is as follows:

```
hvisor-tool
	-tools: Contains the command-line tools and Virtio daemon
	-driver: Kernel modules for hvisor
	-examples: Example configuration files for different environments
```

## Compilation Steps

All operations should be performed in the `hvisor-tool` directory on an x86 host for cross-compilation.

* Compile the command-line tools and kernel modules

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux VIRTIO_GPU=[y/n] ROOT=/path/to/target_rootfs
```

Where `<arch>` should be either `arm64` or `riscv`.

`<log>` can be one of the following: `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, or `LOG_FATAL`, to control the log output level of the Virtio daemon.

`/path/to/your-linux` is the kernel source directory for the root Linux. Specific compilation options can be found in [Makefile](./Makefile), [tools/Makefile](./tools/Makefile), and [driver/Makefile](./driver/Makefile).

For example, to compile the command-line tools for `arm64`, you can run:

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

This will generate the tools in `tools/hvisor` and the kernel module in `driver/hvisor.ko`, which you can copy to the root Linux root filesystem and use.

## Usage Steps

### Kernel Module

Before using the command-line tools and Virtio daemon, the kernel module needs to be loaded on zone0 to allow user-space programs to interact with the Hypervisor:

```
insmod hvisor.ko
```

To unload the kernel module, use the following command:

```
rmmod hvisor.ko
```

### Command-line Tools

On root Linux zone0, the command-line tools can be used to create and shut down other virtual machines.

* Start a new virtual machine

  hvisor-tool starts a new virtual machine using a configuration file:

  ```
  ./hvisor zone start <vm_config.json>
  ```

  `<vm_config.json>` is a file describing the virtual machine configuration. For example:

  * [Start zone1 on QEMU-aarch64](./examples/qemu-aarch64/with_virtio_blk_console/zone1_linux.json): You can refer to [here](./examples/qemu-aarch64/with_virtio_blk_console/README.md) for how to configure and start a Linux on zone1.

  * [Start zone1 on NXP-aarch64](./examples/nxp-aarch64/zone1_linux.json): When using this file to start zone1, the Virtio daemon needs to be started first, and the corresponding configuration file is [virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json).

* Shut down the virtual machine with ID 1:

```
./hvisor zone shutdown -id 1
```

* Print information about all current virtual machines:

```
./hvisor zone list
```

### Virtio Daemon

The Virtio daemon provides Virtio MMIO devices to the virtual machines. Currently, it supports four devices: Virtio-blk, Virtio-net, Virtio-console, and Virtio-gpu.

#### Prerequisites

To use the Virtio daemon, a node named `hvisor_virtio_device` should be added to the **Root Linux device tree**, for example:

```dts
hvisor_virtio_device {
    compatible = "hvisor";
    interrupt-parent = <0x01>;
    interrupts = <0x00 0x20 0x01>;
};
```

This way, when hvisor injects an interrupt with the number `32 + 0x20`, it will trigger the interrupt handler registered in `hvisor.ko` and wake up the Virtio daemon.

If the `32+0x20` interrupt has already been occupied by a device, in addition to modifying the device tree node mentioned above, you also need to modify the `IRQ_WAKEUP_VIRTIO_DEVICE` in Hvisor. For ARM architectures, if the value filled in interrupts is `0xa`, then `IRQ_WAKEUP_VIRTIO_DEVICE` should be set to `32+0xa`. For RISC-V architectures, there is no need to add 32, simply set the two values to be equal.

#### Starting and Creating Virtio Devices

On Root Linux, execute the following example commands:

```c
// Note: Start the daemon before starting the zones
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start <vm_config.json>
```

The `nohup ... &` part indicates that this command will create a daemon, and its log output will be saved in the `nohup.out` file in the current directory.

`virtio_cfg.json` is a JSON file that describes the Virtio devices, such as [virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json). The example file will perform the following actions:

1. **Memory Mapping**

It first maps the RAM memory area of the virtual machine with ID 1 (`zone1`) to the address space of the Virtio daemon using `mmap`.

2. **Create Virtio-blk Device**

A Virtio-blk device is created, and `zone1` will communicate with this device via a specific MMIO region, starting at address `0xa003c00` with a length of `0x200`. The interrupt number for this device is set to 78, and the disk image used is `rootfs2.ext4`.

3. **Create Virtio-console Device**

A Virtio-console device is created for the main serial port of `zone1`. Root Linux should execute the command `screen /dev/pts/x` to enter this virtual console, where `x` can be found in the `nohup.out` log file.

To return to the main console, press the shortcut `ctrl+a+d`. In QEMU, press `ctrl+a ctrl+a+d`. To re-enter the virtual console, execute `screen -r [SID]`, where SID is the process ID of the `screen` session.

4. **Create Virtio-net Device**

If the `net` device's `status` attribute is set to `enable`, a Virtio-net device will be created. The MMIO region for this device starts at address `0xa003600` with a length of `0x200`, and the interrupt number is set to 75. The MAC address for the device will be `00:16:3e:10:10:10`, and it will be used by the virtual machine with ID 1, connected to the Tap device named `tap0`.

5. **Create Virtio-gpu Device**

To use the Virtio-gpu device, the `VIRTIO_GPU=y` option must be added to the `hvisor-tool` compile command, and `libdrm` should be installed along with other configurations. For more details, please refer to [hvisor-book](https://hvisor.syswonder.org/chap04/subchap03/VirtIO/GPUDevice.html) and the [configuration example](./examples/qemu-aarch64/with_virtio_gpu/README.md). If the `gpu` device's `status` attribute is set to `enable`, a Virtio-gpu device will be created, with the MMIO region starting at `0xa003400`, the length set to `0x200`, and the interrupt number set to 74. The default scanout dimensions are a width of `1280px` and a height of `800px`.

#### Shut down Virtio Devices

To shut down the Virtio daemon and all the created devices, execute the following command:

```
pkill hvisor-virtio
``` 