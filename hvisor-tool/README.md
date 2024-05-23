# README
README: [中文](./README-zh.md) | [English](./README.md)

This repository includes the command line tools and kernel module associated with [hvisor](https://github.com/syswonder/hvisor). The command line tools also include a Virtio daemon used to provide Virtio devices. Both the command line tools and the kernel module need to be compiled separately and used on the root Linux. The structure of the entire repository is:

```
hvisor-tool
	-tools: command line tools and virtio daemon
	-driver: hvisor kernel module
```

## Compilation

The following operations are performed in the `hvisor-tool` directory on an x86 host for cross-compilation.

* Compile the command line tools

```bash
make tools
```

* Compile the kernel module

```bash
make driver KDIR=/path/to/your/linux
```

* Compile both the command line tools and the kernel module

```bash
make all KDIR=/path/to/your/linux
```

`KDIR` needs to be set to the kernel directory of the root Linux.

## How to use

### Kernel Module

Before using the command line tools and Virtio daemon, you need to load the kernel module to facilitate interaction between user-space programs and the Hypervisor:

```
insmod hvisor.ko
```

To unload the kernel module:

```
rmmod hvisor.ko
```

### Command Line Tools

In root Linux, command line tools can be used to create and shut down other virtual machines.

To create a virtual machine with id 1, this command loads the operating system image file `Image` to the real physical address `0x70000000` and loads the device tree file `linux2.dtb` to the real physical address `0x91000000`:

```
./hvisor zone start --kernel Image,addr=0x70000000 --dtb linux2.dtb,addr=0x91000000 --id 1
```

To shut down the virtual machine with id 1:

```
./hvisor zone shutdown -id 1
```

### Virtio Daemon

The Virtio daemon provides Virtio MMIO devices for virtual machines. Currently, it supports two types of devices: Virtio block device and Virtio network device.

* Create and start Virtio devices

The following example starts both a Virtio block device and a network device simultaneously:

```
nohup ./hvisor virtio start \
	--device blk,addr=0xa003c00,len=0x200,irq=78,zone_id=1,img=rootfs2.ext4 \
    --device net,addr=0xa003600,len=0x200,irq=75,zone_id=1,tap=tap0  &
```

The meaning of the above command is:

1. First, create a Virtio block device. The virtual machine with id 1 will communicate with this device through an MMIO region with a starting address of `0xa003c00` and a length of `0x200`. The device interrupt number is set to 78, and it corresponds to the disk image `rootfs2.ext4`.
2. Next, create a Virtio network device. The MMIO region starts at `0xa003600` with a length of `0x200`, and the device interrupt number is set to 75. This device is used by the virtual machine with id 1 and is connected to a Tap device named `tap0`.
3. `nohup ... &` indicates that this command will create a daemon process.

* Shutting down Virtio devices

Execute this command to shut down the Virtio daemon and all created devices:

```
pkill hvisor
```

For more information, such as configuring the root Linux environment, refer to: [Using Virtio Devices on hvisor](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial)