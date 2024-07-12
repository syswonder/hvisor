# README
README：[中文](./README-zh.md) | [English](./README.md)

本仓库包含附属于[hvisor](https://github.com/syswonder/hvisor)的命令行工具及内核模块，命令行工具中还包含了Virtio守护进程，用于提供Virtio设备。命令行工具以及内核模块需要单独编译后，在管理虚拟机root linux上使用。整个仓库结构为：

```
hvisor-tool
	-tools: 包含命令行工具以及Virtio守护进程
	-driver: hvisor对应的内核模块
```

## 编译步骤

以下操作均在x86主机的目录`hvisor-tool`下，进行交叉编译。

* 编译命令行工具及内核模块

```bash
make all ARCH=<arch> LOG=<log> KDIR=/path/to/your-linux 
```

其中，`<arch>`应该为`arm64`和`riscv`之一。

`<log>`为`LOG_TRACE`、`LOG_DEBUG`、`LOG_INFO`、`LOG_WARN`、`LOG_ERROR`、`LOG_FATAL`之一。

`/path/to/your-linux`为root linux的kernel源码目录。具体的编译选项请见[Makefile](./Makefile)、[tools/Makefile](./tools/Makefile)、[driver/Makefile](./driver/Makefile)。

例如，要编译面向`arm64`的命令行工具，可以执行：

```bash
make all ARCH=arm64 LOG=LOG_WARN KDIR=~/linux
```

即可在`tools/hvisor`和`driver/hvisor.ko`，将其复制到root linux的根文件系统，使用即可。

## 使用步骤

### 内核模块

使用命令行工具、Virtio守护进程之前，需要加载内核模块，便于用户态程序与Hyperviosr进行交互：

```
insmod hvisor.ko
```

卸载内核模块的操作为：

```
rmmod hvisor.ko
```

### 命令行工具

在root linux中，使用命令行工具可以创建、关闭其他虚拟机。

* 启动新的虚拟机

hvisor-tool通过一个配置文件启动一个新的虚拟机：

```
./hvisor zone start <vm_config.json>
```

`<vm_config.json>`是描述一个虚拟机配置的文件，例如[nxp_linux.json](./examples/nxp_linux.json)。

> **注意：如果想通过命令行而非配置文件启动新虚拟机，请转到[hvisor-tool_old](https://github.com/syswonder/hvisor-tool/commit/3478fc6720f89090c1b5aa913da168f49f95bca0)**。命令行的启动方式会逐渐被配置文件取代，请升级为最新的hvisor-tool。

* 关闭id为1的虚拟机：

```
./hvisor zone shutdown -id 1
```

### Virtio守护进程

Virtio守护进程可为虚拟机提供Virtio MMIO设备，目前支持两种设备：Virtio块设备和Virtio网络设备。

* Virtio设备的启动和创建

下面的示例会同时启动Virtio块设备、网络设备和控制台设备：

```
nohup ./hvisor virtio start \
	--device blk,addr=0xa003c00,len=0x200,irq=78,zone_id=1,img=rootfs2.ext4 \
	--device net,addr=0xa003600,len=0x200,irq=75,zone_id=1,tap=tap0 \
	--device console,addr=0xa003800,len=0x200,irq=76,zone_id=1 &
```

上述命令的具体含义为：

1. 首先创建一个Virtio块设备，id为1的虚拟机会通过一片MMIO区域与该设备通信，这片MMIO区域的起始地址为`0xa003c00`，长度为`0x200`。同时设置设备中断号为78，对应磁盘镜像为`rootfs2.ext4`。
2. 之后再创建一个Virtio网络设备，MMIO区域的起始地址为`0xa003600`，长度为`0x200`，设备中断号为75，由id为1的虚拟机使用，连接到名为`tap0`的Tap设备。
3. 最后创建一个Virtio控制台设备，用于id为1的虚拟机主串口的输出。root linux需要执行`screen /dev/pts/x`命令进入该虚拟控制台，其中`x`可通过nohup.out的输出信息查看。
4. `nohup ... &`说明该命令会创建一个守护进程。

* 关闭Virtio设备

执行该命令即可关闭Virtio守护进程及所有创建的设备：

```
pkill hvisor
```

更多信息，例如root linux的环境配置，可参考：[在hvisor上使用Virtio设备](https://report.syswonder.org/#/2024/20240415_Virtio_devices_tutorial)