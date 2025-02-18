# README  
README: [中文](./README-zh.md) | [English](./README.md)  

This repository contains the command-line tools and kernel modules associated with [hvisor](https://github.com/syswonder/hvisor). It includes the Virtio daemon to provide Virtio devices. Both the command-line tools and kernel modules need to be compiled separately and used on the root Linux of the virtual machine (zone0). The repository structure is as follows:  

```
hvisor-tool
	-tools: Includes the command-line tools and Virtio daemon
	-driver: Kernel module for hvisor
```  

## Compilation Steps  

The following operations are performed in the `hvisor-tool` directory of an x86 host, using cross-compilation.  

### Install libdrm

We need libdrm to compile Virtio-gpu, considering **arm64** as target platform

```shell
wget https://dri.freedesktop.org/libdrm/libdrm-2.4.100.tar.gz
tar -xzvf libdrm-2.4.100.tar.gz
cd libdrm-2.4.100
```

tips: Versions beyond 2.4.100 will be compiled in a different way, check https://dri.freedesktop.org/libdrm for more versions

```shell
# install to your aarch64-linux-gnu compiler
./configure --host=aarch64-linux-gnu --prefix=/usr/aarch64-linux-gnu && make && make install

# install to `install` folder under libdrm
mkdir install
./configure --host=aarch64-linux-gnu --prefix=/path_to_install/install && make && make install

# notice that `prefix` must be an absolute path
```

To support libdrm in your language server, just link include path to your setting files

And finally, we need to edit `include_dirs` under hvisor-tool/tools/Makefiles

```
include_dirs := -I../include -I./include -I../cJSON/ -I/usr/aarch64-linux-gnu/include -I/usr/aarch64-linux-gnu/include/libdrm -L/usr/aarch64-linux-gnu/lib -ldrm -pthread
```

tips: You should edit it with your own install path

### Compile the command-line tools and kernel module  

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux 
```  

- `<arch>` should be one of `arm64` or `riscv`.  
- `<log>` can be one of `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARN`, `LOG_ERROR`, or `LOG_FATAL`, controlling the log level of the Virtio daemon.  
- `/path/to/your-linux` is the kernel source directory of the root Linux. Specific compilation options can be found in the [Makefile](./Makefile), [tools/Makefile](./tools/Makefile), and [driver/Makefile](./driver/Makefile).  

For example, to compile the command-line tools for `arm64`, execute:  

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```  

This generates `tools/hvisor` and `driver/hvisor.ko`. Copy them to the root filesystem of root Linux to use them.  

## Usage Steps  

### Kernel Module  

Before using the command-line tools or Virtio daemon, load the kernel module on zone0 to enable user-space programs to interact with the Hypervisor:  

```bash
insmod hvisor.ko
```  

To unload the kernel module:  

```bash
rmmod hvisor.ko
```  

### Command-Line Tools  

In the root Linux-zone0, the command-line tools can be used to create and shut down other virtual machines.  

- Start a new virtual machine  

  hvisor-tool starts a new virtual machine using a configuration file:  

  ```bash
  ./hvisor zone start <vm_config.json>
  ```  

  `<vm_config.json>` describes the virtual machine's configuration. For example:  

  - [Start zone1 on QEMU-aarch64](./examples/qemu-aarch64/zone1_linux.json): Before starting zone1 with this file, start the Virtio daemon using the corresponding configuration file [virtio_cfg.json](./examples/qemu-aarch64/virtio_cfg.json).  
  - [Start zone1 on NXP-aarch64](./examples/nxp-aarch64/zone1_linux.json): Similar steps apply using the corresponding configuration file [virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json).  

- Shut down a virtual machine with ID 1:  

```bash
./hvisor zone shutdown -id 1
```  

- List all running virtual machines:  

```bash
./hvisor zone list
```  

### Virtio Daemon  

The Virtio daemon provides Virtio MMIO devices to virtual machines. It currently supports Virtio-blk, Virtio-net, and Virtio-console devices.  

#### Prerequisites  

To use the Virtio daemon, add an `hvisor_device` node to the **Root Linux device tree**, for example:  

```dts
hvisor_device {
    compatible = "hvisor";
    interrupt-parent = <0x01>;
    interrupts = <0x00 0x20 0x01>;
};
```  

This setup ensures that when hvisor injects an interrupt with the number `32+0x20` into Root Linux, the interrupt handler registered in `hvisor.ko` is triggered to wake up the Virtio daemon.  

#### Starting and Creating Virtio Devices  

On Root Linux, execute the following commands:  

```bash
// Start the daemon first, then start zones
nohup ./hvisor virtio start virtio_cfg.json &
./hvisor zone start <vm_config.json>
```  

The `nohup ... &` command creates a daemon process, with logs saved in `nohup.out` in the current folder.  

`virtio_cfg.json` is a JSON file describing Virtio devices, such as [virtio_cfg.json](./examples/nxp-aarch64/virtio_cfg.json). The example performs:  

1. **Address Space Mapping**  

   Maps the RAM memory region of virtual machine `zone1` (ID=1) into the address space of the Virtio daemon using `mmap`.  

2. **Creating Virtio-blk Device**  

   Creates a Virtio-blk device with an MMIO region starting at `0xa003c00`, length `0x200`, interrupt number 78, and backing disk image `rootfs2.ext4`.  

3. **Creating Virtio-console Device**  

   Creates a Virtio-console device for the primary serial output of `zone1`. Access the virtual console on root Linux using:  

   ```bash
   screen /dev/pts/x
   ```  

   Replace `x` with the appropriate value from `nohup.out`. Exit the console with `ctrl+a+d`. Re-enter with:  

   ```bash
   screen -r [SID]
   ```  

   Replace `[SID]` with the session ID of the screen process.  

4. **Creating Virtio-net Device**  

   If the `status` attribute of the `net` device is `disable`, no Virtio-net device is created. If set to `enable`, a Virtio-net device is created with an MMIO region starting at `0xa003600`, length `0x200`, interrupt number 75, MAC address `00:16:3e:10:10:10`, and connected to a Tap device named `tap0`.  

5. **Creating Virtio-gpu Device**

   Creates a Virtio-gpu device with an MMIO region starting at `0xa003400` with length `0x200`, interrupt number 74. Default scanout size `width=1280px, height=800px`.

#### Probing Physical GPU Device in Root Linux

To probe physical GPU device in `Root Linux`, you should edit files under `hvisor/src/platform` to probe GPU device on PCI bus.Add your interrupt number of Virtio-gpu device to `ROOT_ZONE_IRQS`

After booting Root Linux, you can do `dmesg | grep drm` or `lspci` to check if your GPU device is working. To check the devices supported by libdrm, do `apt install libdrm-tests` and `modetest`. 

#### Shutting Down Virtio Devices  

To shut down the Virtio daemon and all devices it created, execute:  

```bash
pkill hvisor-virtio
```  

For more information, such as configuring the root Linux environment, see: [Using Virtio devices on hvisor](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial)