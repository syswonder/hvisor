# README

README: [中文](./README-zh.md) | [English](./README.md)

This repository contains command-line tools and kernel modules affiliated with [hvisor](https://github.com/syswonder/hvisor). The command-line tools include a Virtio daemon for providing Virtio devices. Both the command-line tools and kernel modules need to be compiled separately and used on the management virtual machine's root Linux. The repository structure is as follows:

```
hvisor-tool
    - tools: Contains command-line tools and Virtio daemon
    - driver: Kernel module for hvisor
```

## Compilation Steps

The following operations are performed in the `hvisor-tool` directory on an x86 host for cross-compilation.

- Compile the command-line tools and kernel modules

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux 
```

Where `<arch>` should be either `arm64` or `riscv`.

`<log>` can be one of `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, or `LOG_FATAL`.

`/path/to/your-linux` is the kernel source directory of the root Linux. For specific compilation options, see [Makefile](./Makefile), [tools/Makefile](./tools/Makefile), and [driver/Makefile](./driver/Makefile).

For example, to compile the command-line tools for `arm64`, you can execute:

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

Then you can find `tools/hvisor` and `driver/hvisor.ko`. Copy them to the disk of the root Linux and use them.

## Usage Steps

### Kernel Module

Before using the command-line tools and Virtio daemon, the kernel module needs to be loaded to facilitate interaction between user-space programs and the Hypervisor:

```
insmod hvisor.ko
```

To unload the kernel module:

```
rmmod hvisor.ko
```

### Command-line Tools

In root Linux, the command-line tools can be used to create and shut down other virtual machines.

- Start a new virtual machine

hvisor-tool starts a new virtual machine through a configuration file:

```bash
./hvisor zone start <vm_config.json>
```

`<vm_config.json>` is a file describing the configuration of a virtual machine, such as [nxp_linux.json](./examples/nxp_linux.json).

> **Note: If you want to start a new virtual machine via command line instead of a configuration file, please refer to [hvisor-tool_old](https://github.com/syswonder/hvisor-tool/commit/3478fc6720f89090c1b5aa913da168f49f95bca0)**. The command-line startup method will gradually be replaced by configuration files, so please upgrade to the latest hvisor-tool.

- Shut down the virtual machine with id 1:

```bash
./hvisor zone shutdown -id 1
```

### Virtio Daemon

The Virtio daemon can provide Virtio MMIO devices for virtual machines, currently supporting two types of devices: Virtio block device and Virtio network device.

- Start and create Virtio devices

The following example will start Virtio block, network, and console devices simultaneously:

```bash
nohup ./hvisor virtio start \
    --device blk,addr=0xa003c00,len=0x200,irq=78,zone_id=1,img=rootfs2.ext4 \
    --device net,addr=0xa003600,len=0x200,irq=75,zone_id=1,tap=tap0 \
    --device console,addr=0xa003800,len=0x200,irq=76,zone_id=1 &
```

The specific meaning of the above command is:

1. First, create a Virtio block device. The virtual machine with id 1 will communicate with the device through an MMIO region with a starting address of `0xa003c00` and a length of `0x200`. The interrupt number for the device is set to 78, corresponding to the disk image `rootfs2.ext4`.
2. Then, create a Virtio network device with an MMIO region starting at `0xa003600` and a length of `0x200`, with an interrupt number of 75. This device will be used by the virtual machine with id 1 and connected to the Tap device named `tap0`.
3. Finally, create a Virtio console device for the main serial port output of the virtual machine with id 1. In root Linux, execute `screen /dev/pts/x` to enter the virtual console, where `x` can be found from the output information of nohup.out.
4. `nohup ... &` indicates that this command will create a daemon process.

- Shut down Virtio devices

Execute this command to shut down the Virtio daemon and all created devices:

```bash
pkill hvisor
```

For more information, such as the environment setup of root Linux, refer to: [Using Virtio Devices on hvisor](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial)